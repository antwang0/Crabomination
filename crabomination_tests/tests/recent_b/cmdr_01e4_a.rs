//! Commander: the 2026-10-20 (`01E4rMUL`) two-gap pair closers
//! (`decks::cmdr_01e4_a`).

use crabomination::card::{CardId, CardType, CounterType, CreatureType, Keyword, Subtypes, TokenDefinition};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn cast_x(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x })
        .expect("cast");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    cast_x(g, id, target, None);
}

fn activate(g: &mut GameState, id: CardId, index: usize) -> Result<(), GameError> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn body(name: &str, power: i32, kind: CreatureType) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power,
        toughness: power,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![kind], ..Default::default() },
        ..Default::default()
    }
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn attack_with(g: &mut GameState, attackers: &[CardId]) {
    for &a in attackers {
        g.clear_sickness(a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

/// Dragonloft Idol is a 3/3 until you control a Dragon, then a 4/4 flier
/// with trample.
#[test]
fn dragonloft_idol_grows_wings_beside_a_dragon() {
    let mut g = pod(3);
    let idol = g.add_card_to_battlefield(0, catalog::dragonloft_idol());
    assert_eq!(pt(&g, idol), (3, 3));
    g.add_card_to_battlefield(1, catalog::scion_of_ugin());
    assert_eq!(pt(&g, idol), (3, 3), "an opponent's Dragon does nothing");
    g.add_card_to_battlefield(0, catalog::scion_of_ugin());
    assert_eq!(pt(&g, idol), (4, 4));
    let idol = g.computed_permanent(idol).unwrap();
    assert!(idol.keywords().contains(&Keyword::Flying) && idol.keywords().contains(&Keyword::Trample));
}

/// CR 603.4 — Webweaver Changeling's lifegain is an intervening if: two
/// creature cards in the graveyard gain nothing, three gain 5.
#[test]
fn cr_603_4_webweaver_changeling_counts_creature_cards() {
    let mut g = pod(2);
    for _ in 0..2 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    g.add_card_to_graveyard(0, catalog::island());
    let first = g.add_card_to_hand(0, catalog::webweaver_changeling());
    let life = g.players[0].life;
    cast(&mut g, first, None);
    assert_eq!(g.players[0].life, life, "two creature cards and a land: no trigger");
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let second = g.add_card_to_hand(0, catalog::webweaver_changeling());
    cast(&mut g, second, None);
    assert_eq!(g.players[0].life, life + 5);
}

/// Fathom Fleet Boarder costs 2 life alone, nothing beside another Pirate.
#[test]
fn fathom_fleet_boarder_wants_another_pirate() {
    let mut g = pod(2);
    let life = g.players[0].life;
    let first = g.add_card_to_hand(0, catalog::fathom_fleet_boarder());
    cast(&mut g, first, None);
    assert_eq!(g.players[0].life, life - 2, "no other Pirate");
    let second = g.add_card_to_hand(0, catalog::fathom_fleet_boarder());
    cast(&mut g, second, None);
    assert_eq!(g.players[0].life, life - 2, "the first Boarder is another Pirate");
}

/// Deadeye Tormentor's raid discard fires only after you attacked.
#[test]
fn deadeye_tormentor_raids_for_a_discard() {
    let mut g = pod(3);
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_hand(1, catalog::island());
    let quiet = g.add_card_to_hand(0, catalog::deadeye_tormentor());
    cast(&mut g, quiet, Some(Target::Player(1)));
    assert_eq!(g.players[1].hand.len(), 2, "no attack this turn");
    g.players[0].attacked_this_turn = true;
    let raider = g.add_card_to_hand(0, catalog::deadeye_tormentor());
    cast(&mut g, raider, Some(Target::Player(1)));
    assert_eq!(g.players[1].hand.len(), 1);
}

/// Scarred Vinebreeder's pump costs an Elf card from your graveyard; a
/// non-Elf creature card does not pay it.
#[test]
fn scarred_vinebreeder_exiles_an_elf_card() {
    let mut g = pod(2);
    let breeder = g.add_card_to_battlefield(0, catalog::scarred_vinebreeder());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    assert!(activate(&mut g, breeder, 0).is_err(), "a Bear is no Elf");
    let elf = g.add_card_to_graveyard(0, catalog::llanowar_elves());
    activate(&mut g, breeder, 0).expect("exile the Elf");
    assert_eq!(pt(&g, breeder), (4, 4));
    assert!(g.exile.iter().any(|c| c.id == elf));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear));
}

/// Consuming Fervor: +3/+3, and the enchanted creature takes a -1/-1 counter
/// at its controller's upkeep.
#[test]
fn consuming_fervor_shrinks_its_host_each_upkeep() {
    let mut g = pod(2);
    let bear = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let fervor = g.add_card_to_hand(0, catalog::consuming_fervor());
    cast(&mut g, fervor, Some(Target::Permanent(bear)));
    assert_eq!(pt(&g, bear), (5, 5));
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::End;
    while !(g.active_player_idx == 0 && g.step == TurnStep::Upkeep) {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::MinusOneMinusOne), 1);
    assert_eq!(pt(&g, bear), (4, 4));
}

/// Hydrolash shrinks attacking creatures only, and draws.
#[test]
fn hydrolash_blunts_only_attackers() {
    let mut g = pod(2);
    let attacker = g.add_token_to_battlefield(0, &body("Bear", 3, CreatureType::Bear));
    let home = g.add_token_to_battlefield(0, &body("Wolf", 3, CreatureType::Wolf));
    attack_with(&mut g, &[attacker]);
    let hand = g.players[0].hand.len();
    let lash = g.add_card_to_hand(0, catalog::hydrolash());
    cast(&mut g, lash, None);
    assert_eq!(pt(&g, attacker), (1, 3));
    assert_eq!(pt(&g, home), (3, 3));
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Archangel's Light gains 2 a graveyard card, then the graveyard joins the
/// library.
#[test]
fn archangels_light_counts_then_shuffles_the_graveyard() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::island());
    }
    let library = g.players[0].library.len();
    let life = g.players[0].life;
    let light = g.add_card_to_hand(0, catalog::archangels_light());
    cast(&mut g, light, None);
    assert_eq!(g.players[0].life, life + 6);
    assert!(g.players[0].graveyard.iter().all(|c| c.id == light), "only the resolved Light is left");
    assert_eq!(g.players[0].library.len(), library + 3);
}

/// Abnormal Endurance: +2/+0, and the creature comes back tapped when it
/// dies this turn.
#[test]
fn abnormal_endurance_returns_the_creature_tapped() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let endurance = g.add_card_to_hand(0, catalog::abnormal_endurance());
    cast(&mut g, endurance, Some(Target::Permanent(bears)));
    assert_eq!(pt(&g, bears), (4, 2));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(bears)));
    let back = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").expect("returned");
    assert!(back.tapped);
}

/// Luxurious Libation pumps by X and leaves a green-white Citizen.
#[test]
fn luxurious_libation_pumps_by_x_and_makes_a_citizen() {
    let mut g = pod(2);
    let bear = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let libation = g.add_card_to_hand(0, catalog::luxurious_libation());
    cast_x(&mut g, libation, Some(Target::Permanent(bear)), Some(3));
    assert_eq!(pt(&g, bear), (5, 5));
    let citizen = g.battlefield.iter().find(|c| c.definition.name == "Citizen").expect("a Citizen");
    assert_eq!(citizen.controller, 0);
}

/// Otherworldly Outburst leaves a 3/2 Eldrazi Horror when its target dies
/// this turn.
#[test]
fn otherworldly_outburst_leaves_a_horror_behind() {
    let mut g = pod(2);
    let bear = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let outburst = g.add_card_to_hand(0, catalog::otherworldly_outburst());
    cast(&mut g, outburst, Some(Target::Permanent(bear)));
    assert_eq!(pt(&g, bear), (3, 2));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(bear)));
    let horror = g.battlefield.iter().find(|c| c.definition.name == "Eldrazi Horror").expect("a Horror");
    assert_eq!(pt(&g, horror.id), (3, 2));
}

/// Burden of Guilt taps the creature it enchants for {1}.
#[test]
fn burden_of_guilt_taps_its_host() {
    let mut g = pod(2);
    let theirs = g.add_token_to_battlefield(1, &body("Bear", 2, CreatureType::Bear));
    let burden = g.add_card_to_hand(0, catalog::burden_of_guilt());
    cast(&mut g, burden, Some(Target::Permanent(theirs)));
    activate(&mut g, burden, 0).expect("tap it");
    assert!(g.battlefield_find(theirs).unwrap().tapped);
}

/// Sunset Pyramid draws once a brick counter; with the three spent the draw
/// is gone and only the scry is left.
#[test]
fn sunset_pyramid_draws_three_times() {
    let mut g = pod(2);
    let pyramid = g.add_card_to_hand(0, catalog::sunset_pyramid());
    cast(&mut g, pyramid, None);
    let hand = g.players[0].hand.len();
    for _ in 0..3 {
        activate(&mut g, pyramid, 0).expect("draw");
        g.battlefield_find_mut(pyramid).unwrap().tapped = false;
    }
    assert_eq!(g.players[0].hand.len(), hand + 3);
    assert!(activate(&mut g, pyramid, 0).is_err(), "no brick counter left");
    activate(&mut g, pyramid, 1).expect("the scry needs no counter");
}

/// Squad Captain counts the other creatures you control, not an opponent's.
#[test]
fn squad_captain_enters_with_a_counter_per_other_creature() {
    let mut g = pod(3);
    for _ in 0..2 {
        g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    }
    g.add_token_to_battlefield(1, &body("Bear", 2, CreatureType::Bear));
    let captain = g.add_card_to_hand(0, catalog::squad_captain());
    cast(&mut g, captain, None);
    assert_eq!(g.battlefield_find(captain).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Strength in Numbers: +X/+X and trample, X every attacking creature.
#[test]
fn strength_in_numbers_counts_the_attackers() {
    let mut g = pod(2);
    let a = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let b = g.add_token_to_battlefield(0, &body("Wolf", 2, CreatureType::Wolf));
    attack_with(&mut g, &[a, b]);
    let strength = g.add_card_to_hand(0, catalog::strength_in_numbers());
    cast(&mut g, strength, Some(Target::Permanent(a)));
    assert_eq!(pt(&g, a), (4, 4));
    assert!(g.computed_permanent(a).unwrap().keywords().contains(&Keyword::Trample));
}

/// Campaign of Vengeance drains the defending player once per attacker.
#[test]
fn campaign_of_vengeance_drains_per_attacker() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::campaign_of_vengeance());
    let a = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let b = g.add_token_to_battlefield(0, &body("Wolf", 2, CreatureType::Wolf));
    let (mine, theirs, bystander) = (g.players[0].life, g.players[1].life, g.players[2].life);
    attack_with(&mut g, &[a, b]);
    assert_eq!(g.players[1].life, theirs - 2);
    assert_eq!(g.players[0].life, mine + 2);
    assert_eq!(g.players[2].life, bystander, "only the defending player loses");
}

/// Delirium — Mournwillow stops blockers with power 2 or less only with four
/// card types in your graveyard.
#[test]
fn mournwillow_needs_delirium() {
    for delirium in [false, true] {
        let mut g = pod(2);
        let small = g.add_token_to_battlefield(1, &body("Bear", 2, CreatureType::Bear));
        let big = g.add_token_to_battlefield(1, &body("Beast", 3, CreatureType::Beast));
        g.add_card_to_graveyard(0, catalog::island());
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.add_card_to_graveyard(0, catalog::archangels_light());
        if delirium {
            g.add_card_to_graveyard(0, catalog::lightning_bolt());
        }
        let willow = g.add_card_to_hand(0, catalog::mournwillow());
        cast(&mut g, willow, None);
        attack_with(&mut g, &[willow]);
        g.step = TurnStep::DeclareBlockers;
        let mut g2 = g.clone();
        let small_blocks = g.perform_action(GameAction::DeclareBlockers(vec![(small, willow)])).is_ok();
        assert_eq!(small_blocks, !delirium, "delirium {delirium}");
        g2.perform_action(GameAction::DeclareBlockers(vec![(big, willow)])).expect("power 3 still blocks");
    }
}

/// Lurking Roper stays tapped through the untap step and untaps on lifegain.
#[test]
fn lurking_roper_untaps_on_lifegain() {
    let mut g = pod(2);
    let roper = g.add_card_to_battlefield(0, catalog::lurking_roper());
    g.battlefield_find_mut(roper).unwrap().tapped = true;
    let light = g.add_card_to_hand(0, catalog::archangels_light());
    g.add_card_to_graveyard(0, catalog::island());
    cast(&mut g, light, None);
    assert!(!g.battlefield_find(roper).unwrap().tapped);
}

/// Lurking Chupacabra shrinks an opposing creature when one of yours
/// explores.
#[test]
fn lurking_chupacabra_answers_an_explore() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::lurking_chupacabra());
    let theirs = g.add_token_to_battlefield(1, &body("Beast", 3, CreatureType::Beast));
    let walker = g.add_card_to_hand(0, catalog::merfolk_branchwalker());
    cast(&mut g, walker, Some(Target::Permanent(theirs)));
    assert_eq!(pt(&g, theirs), (1, 1));
}

/// CR 603.2c — Nefarious Imp scries once for a batch of departures.
#[test]
fn cr_603_2c_nefarious_imp_scries_once_a_batch() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::nefarious_imp());
    let a = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let b = g.add_token_to_battlefield(0, &body("Wolf", 2, CreatureType::Wolf));
    let gyre = g.add_card_to_hand(0, catalog::captivating_gyre());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: gyre,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    let scries = drain_stack(&mut g)
        .iter()
        .filter(|e| format!("{e:?}").starts_with("Scry"))
        .count();
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    assert_eq!(scries, 1, "one scry for both");
}

/// Void Stalker puts itself and its target into their owners' libraries.
#[test]
fn void_stalker_takes_its_target_along() {
    let mut g = pod(2);
    let stalker = g.add_card_to_battlefield(0, catalog::void_stalker());
    g.clear_sickness(stalker);
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let (mine, others) = (g.players[0].library.len(), g.players[1].library.len());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: stalker,
        ability_index: 0,
        target: Some(Target::Permanent(theirs)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), mine + 1);
    assert_eq!(g.players[1].library.len(), others + 1);
}

/// Bristlebane Outrider gets +2/+0 once another creature entered under your
/// control this turn.
#[test]
fn bristlebane_outrider_grows_after_another_arrival() {
    let mut g = pod(2);
    let outrider = g.add_card_to_hand(0, catalog::bristlebane_outrider());
    cast(&mut g, outrider, None);
    assert_eq!(pt(&g, outrider), (3, 5), "its own arrival doesn't count");
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bears, None);
    assert_eq!(pt(&g, outrider), (5, 5));
}

/// Shinewend spends its counter to destroy an enchantment, and dies a 0/0.
#[test]
fn shinewend_trades_its_counter_for_an_enchantment() {
    let mut g = pod(2);
    let wend = g.add_card_to_hand(0, catalog::shinewend());
    cast(&mut g, wend, None);
    let campaign = g.add_card_to_battlefield(1, catalog::campaign_of_vengeance());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wend,
        ability_index: 0,
        target: Some(Target::Permanent(campaign)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(campaign).is_none());
    assert!(g.battlefield_find(wend).is_none(), "a 0/0 with no counter dies");
}

/// Sequestered Stash mills five and may stack an artifact card on top.
#[test]
fn sequestered_stash_mills_then_stacks_an_artifact() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let stash = g.add_card_to_battlefield(0, catalog::sequestered_stash());
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    let graveyard = g.players[0].graveyard.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Cards(vec![ring])]));
    activate(&mut g, stash, 1).expect("activate");
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(ring), "the Sol Ring is on top");
    assert_eq!(g.players[0].graveyard.len(), graveyard + 5 + 1 - 1, "five milled, the Stash in, the Ring out");
}

fn activate_on(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), GameError> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

/// CR 107.6 — Safehold Sentry's {Q} cost: it must be tapped, and untaps.
#[test]
fn cr_107_6_safehold_sentry_pays_the_untap_symbol() {
    let mut g = pod(2);
    let sentry = g.add_card_to_battlefield(0, catalog::safehold_sentry());
    g.clear_sickness(sentry);
    assert!(activate(&mut g, sentry, 0).is_err(), "untapped: {{Q}} can't be paid");
    g.battlefield_find_mut(sentry).unwrap().tapped = true;
    activate(&mut g, sentry, 0).expect("tapped: untap it");
    assert!(!g.battlefield_find(sentry).unwrap().tapped);
    assert_eq!(pt(&g, sentry), (2, 4));
}

/// Scarblade Elite kills for an Assassin card from your graveyard.
#[test]
fn scarblade_elite_exiles_an_assassin_to_kill() {
    let mut g = pod(2);
    let elite = g.add_card_to_battlefield(0, catalog::scarblade_elite());
    g.clear_sickness(elite);
    let theirs = g.add_token_to_battlefield(1, &body("Bear", 2, CreatureType::Bear));
    assert!(activate_on(&mut g, elite, 0, Some(Target::Permanent(theirs))).is_err(), "no Assassin card");
    g.add_card_to_graveyard(0, catalog::scarblade_elite());
    activate_on(&mut g, elite, 0, Some(Target::Permanent(theirs))).expect("exile the Assassin");
    assert!(g.battlefield_find(theirs).is_none());
}

/// Thrash of Raptors needs another Dinosaur for +2/+0 and trample.
#[test]
fn thrash_of_raptors_hunts_in_a_pack() {
    let mut g = pod(2);
    let thrash = g.add_card_to_battlefield(0, catalog::thrash_of_raptors());
    assert_eq!(pt(&g, thrash), (3, 3));
    g.add_card_to_battlefield(0, catalog::orazca_raptor());
    assert_eq!(pt(&g, thrash), (5, 3));
    assert!(g.computed_permanent(thrash).unwrap().keywords().contains(&Keyword::Trample));
}

/// Sigil Blessing: +3/+3 on the target, +1/+1 on your others, nothing on an
/// opponent's.
#[test]
fn sigil_blessing_lifts_the_team() {
    let mut g = pod(2);
    let star = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let mate = g.add_token_to_battlefield(0, &body("Wolf", 2, CreatureType::Wolf));
    let theirs = g.add_token_to_battlefield(1, &body("Beast", 2, CreatureType::Beast));
    let blessing = g.add_card_to_hand(0, catalog::sigil_blessing());
    cast(&mut g, blessing, Some(Target::Permanent(star)));
    assert_eq!(pt(&g, star), (5, 5));
    assert_eq!(pt(&g, mate), (3, 3));
    assert_eq!(pt(&g, theirs), (2, 2));
}

/// Ajani's Comrade grows at the beginning of combat only beside an Ajani.
#[test]
fn ajanis_comrade_needs_an_ajani() {
    for ajani in [false, true] {
        let mut g = pod(2);
        let comrade = g.add_card_to_battlefield(0, catalog::ajanis_comrade());
        if ajani {
            g.add_card_to_battlefield(0, catalog::ajani_goldmane());
        }
        g.step = TurnStep::PreCombatMain;
        while g.step != TurnStep::BeginCombat {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(comrade).unwrap().counter_count(CounterType::PlusOnePlusOne), u32::from(ajani));
    }
}

/// Incremental Growth puts one, two and three counters on three creatures.
#[test]
fn incremental_growth_counts_one_two_three() {
    let mut g = pod(2);
    let ids: Vec<CardId> = (0..3).map(|_| g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear))).collect();
    let growth = g.add_card_to_hand(0, catalog::incremental_growth());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: growth,
        target: Some(Target::Permanent(ids[0])),
        additional_targets: vec![Target::Permanent(ids[1]), Target::Permanent(ids[2])],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let counters: Vec<u32> =
        ids.iter().map(|&id| g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne)).collect();
    assert_eq!(counters, vec![1, 2, 3]);
}

/// CR 603.12 — Slumbering Walker removes a counter at your end step and, when
/// it does, returns a small creature card from your graveyard.
#[test]
fn cr_603_12_slumbering_walker_reanimates_when_it_sheds_a_counter() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let walker = g.add_card_to_hand(0, catalog::slumbering_walker());
    cast(&mut g, walker, None);
    assert_eq!(pt(&g, walker), (2, 5));
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
    }
    drain_stack(&mut g);
    assert_eq!(pt(&g, walker), (3, 6));
    assert!(g.battlefield_find(bears).is_some(), "the Bears came back");
}

/// Exuberant Wolfbear makes a Human you control a 4/4 as it attacks.
#[test]
fn exuberant_wolfbear_lends_its_size_to_a_human() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let wolfbear = g.add_card_to_battlefield(0, catalog::exuberant_wolfbear());
    let human = g.add_token_to_battlefield(0, &body("Soldier", 1, CreatureType::Human));
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack_with(&mut g, &[wolfbear]);
    assert_eq!(pt(&g, human), (4, 4));
}

/// Elvish Eulogist gains a life per Elf card in your graveyard.
#[test]
fn elvish_eulogist_counts_elf_cards() {
    let mut g = pod(2);
    let eulogist = g.add_card_to_battlefield(0, catalog::elvish_eulogist());
    g.add_card_to_graveyard(0, catalog::llanowar_elves());
    g.add_card_to_graveyard(0, catalog::elvish_mystic());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let life = g.players[0].life;
    activate(&mut g, eulogist, 0).expect("sacrifice");
    assert_eq!(g.players[0].life, life + 3, "two Elves in the yard, then the Eulogist itself");
}

/// Shi'ar Soldier bounces only during your turn.
#[test]
fn shiar_soldier_bounces_on_your_turn_only() {
    let mut g = pod(2);
    let soldier = g.add_card_to_battlefield(0, catalog::shiar_soldier());
    g.clear_sickness(soldier);
    let mine = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    g.active_player_idx = 1;
    assert!(activate_on(&mut g, soldier, 0, Some(Target::Permanent(mine))).is_err(), "not your turn");
    g.active_player_idx = 0;
    activate_on(&mut g, soldier, 0, Some(Target::Permanent(mine))).expect("your turn");
    assert!(g.battlefield_find(mine).is_none());
    assert!(g.computed_permanent(soldier).unwrap().subtypes().creature_types.contains(&CreatureType::Shiar));
}

/// Great Hall of Starnheim trades itself and a creature for a 4/4 Angel.
#[test]
fn great_hall_of_starnheim_makes_an_angel() {
    let mut g = pod(2);
    let hall = g.add_card_to_battlefield(0, catalog::great_hall_of_starnheim());
    assert!(activate(&mut g, hall, 1).is_err(), "no creature to sacrifice");
    let bear = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    activate(&mut g, hall, 1).expect("sacrifice both");
    assert!(g.battlefield_find(hall).is_none() && g.battlefield_find(bear).is_none());
    let angel = g.battlefield.iter().find(|c| c.definition.name == "Angel Warrior").expect("an Angel");
    assert_eq!(pt(&g, angel.id), (4, 4));
}

/// Revelation of Power adds flying and lifelink only to a creature with a
/// counter.
#[test]
fn revelation_of_power_rewards_a_counter() {
    for counter in [false, true] {
        let mut g = pod(2);
        let bear = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
        if counter {
            g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
        }
        let revelation = g.add_card_to_hand(0, catalog::revelation_of_power());
        cast(&mut g, revelation, Some(Target::Permanent(bear)));
        let flies = g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Flying);
        assert_eq!(flies, counter);
    }
}

/// Llanowar Augur's sacrifice works only during your upkeep.
#[test]
fn llanowar_augur_waits_for_your_upkeep() {
    let mut g = pod(2);
    let augur = g.add_card_to_battlefield(0, catalog::llanowar_augur());
    let bear = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    assert!(activate_on(&mut g, augur, 0, Some(Target::Permanent(bear))).is_err(), "main phase");
    g.step = TurnStep::Upkeep;
    activate_on(&mut g, augur, 0, Some(Target::Permanent(bear))).expect("upkeep");
    assert_eq!(pt(&g, bear), (5, 5));
}

/// Sultai Flayer gains 4 for a toughness-4 creature's death, not a smaller
/// one's.
#[test]
fn sultai_flayer_counts_only_the_big_deaths() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::sultai_flayer());
    let small = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let big = g.add_token_to_battlefield(0, &body("Beast", 4, CreatureType::Beast));
    let life = g.players[0].life;
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(small)));
    assert_eq!(g.players[0].life, life);
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(big)));
    assert_eq!(g.players[0].life, life + 4);
}

/// Loxodon Lifechanter may set your life to your creatures' total toughness.
#[test]
fn loxodon_lifechanter_resets_life_to_total_toughness() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    g.add_token_to_battlefield(0, &body("Beast", 5, CreatureType::Beast));
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let lifechanter = g.add_card_to_hand(0, catalog::loxodon_lifechanter());
    cast(&mut g, lifechanter, None);
    assert_eq!(g.players[0].life, 11, "5 + its own 6");
}

/// Bolster 5 — Sandsteppe Mastodon's counters go on the least-toughness
/// creature you control.
#[test]
fn sandsteppe_mastodon_bolsters_the_smallest() {
    let mut g = pod(2);
    let small = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let mastodon = g.add_card_to_hand(0, catalog::sandsteppe_mastodon());
    cast(&mut g, mastodon, None);
    assert_eq!(g.battlefield_find(small).unwrap().counter_count(CounterType::PlusOnePlusOne), 5);
    assert_eq!(g.battlefield_find(mastodon).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Grasp of the Hieromancer's attack trigger taps a creature of the
/// defending player.
#[test]
fn grasp_of_the_hieromancer_taps_a_defender() {
    let mut g = pod(3);
    let bear = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let theirs = g.add_token_to_battlefield(1, &body("Beast", 3, CreatureType::Beast));
    let grasp = g.add_card_to_hand(0, catalog::grasp_of_the_hieromancer());
    cast(&mut g, grasp, Some(Target::Permanent(bear)));
    assert_eq!(pt(&g, bear), (3, 3));
    attack_with(&mut g, &[bear]);
    assert!(g.battlefield_find(theirs).unwrap().tapped);
}

/// Elvish Branchbender animates a Forest as an X/X, X your Elves.
#[test]
fn elvish_branchbender_counts_elves() {
    let mut g = pod(2);
    let bender = g.add_card_to_battlefield(0, catalog::elvish_branchbender());
    g.clear_sickness(bender);
    g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    activate_on(&mut g, bender, 0, Some(Target::Permanent(forest))).expect("animate");
    assert_eq!(pt(&g, forest), (2, 2));
}

/// CR 702.100a / 707.2 — Watchful Radstag's token copy enters as a plain 2/2
/// (counters aren't copied), so it can't re-evolve the evolved original: one
/// bigger arrival makes one copy. Sweep seed 931184 (8 seats) ended in a CR
/// 104.4b draw with 403 Radstags on one board.
#[test]
fn cr_702_100a_watchful_radstag_copies_once_per_evolve() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::watchful_radstag());
    let beast = g.add_card_to_hand(0, catalog::webweaver_changeling());
    cast(&mut g, beast, None);
    let radstags = g.battlefield.iter().filter(|c| c.definition.name == "Watchful Radstag").count();
    assert_eq!(radstags, 2, "the original and one copy");
    assert!(g.game_over.is_none());
}

/// CR 702.100a / 707.2 — Radstags of different sizes: each evolves once for a
/// bigger arrival and each copy is a fresh 2/2, so nothing re-evolves
/// (sweep seed 931184).
#[test]
fn cr_702_100a_uneven_radstags_settle() {
    let mut g = pod(2);
    let big = g.add_card_to_battlefield(0, catalog::watchful_radstag());
    g.battlefield_find_mut(big).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.add_card_to_battlefield(0, catalog::watchful_radstag());
    let beast = g.add_card_to_hand(0, catalog::webweaver_changeling());
    cast(&mut g, beast, None);
    let radstags: Vec<(i32, i32)> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Watchful Radstag")
        .map(|c| pt(&g, c.id))
        .collect();
    assert_eq!(radstags.len(), 4, "two evolved originals and two copies: {radstags:?}");
    assert!(g.game_over.is_none());
}

/// CR 702.100b — a creature evolves only when its evolve ability puts the
/// counter. Under an opponent's Blightbeetle no +1/+1 counter lands, so
/// Watchful Radstag never evolves and makes no copy (bug fix: 8-seat sweep
/// seed 931184 drew a CR 104.4b loop with 403 Radstags).
#[test]
fn cr_702_100b_no_counter_no_evolution_under_blightbeetle() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::blightbeetle());
    let radstag = g.add_card_to_battlefield(0, catalog::watchful_radstag());
    let beast = g.add_card_to_hand(0, catalog::webweaver_changeling());
    cast(&mut g, beast, None);
    assert_eq!(g.battlefield_find(radstag).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    let radstags = g.battlefield.iter().filter(|c| c.definition.name == "Watchful Radstag").count();
    assert_eq!(radstags, 1, "no counter, no evolution, no copy");
    assert!(g.game_over.is_none());
}
