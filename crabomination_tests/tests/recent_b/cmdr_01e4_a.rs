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

/// Infernal Scarring's granted death trigger draws for the creature's
/// controller.
#[test]
fn infernal_scarring_draws_when_its_host_dies() {
    let mut g = pod(2);
    let bear = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let scarring = g.add_card_to_hand(0, catalog::infernal_scarring());
    cast(&mut g, scarring, Some(Target::Permanent(bear)));
    assert_eq!(pt(&g, bear), (4, 2));
    let hand = g.players[0].hand.len();
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(bear)));
    assert_eq!(g.players[0].hand.len(), hand + 1, "Murder left the hand, the trigger drew one");
}

/// Goremand's cast costs a creature, and every opponent sacrifices one.
#[test]
fn goremand_eats_one_and_makes_each_opponent_feed_it() {
    let mut g = pod(3);
    let fodder = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let a = g.add_token_to_battlefield(1, &body("Bear", 2, CreatureType::Bear));
    let b = g.add_token_to_battlefield(2, &body("Bear", 2, CreatureType::Bear));
    let goremand = g.add_card_to_hand(0, catalog::goremand());
    cast(&mut g, goremand, None);
    assert!(g.battlefield_find(fodder).is_none(), "the additional cost");
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    assert!(g.battlefield_find(goremand).is_some());
}

/// Rakshasa Gravecaller's two Zombies only follow an exploited creature.
#[test]
fn rakshasa_gravecaller_needs_an_exploit() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let gravecaller = g.add_card_to_hand(0, catalog::rakshasa_gravecaller());
    cast(&mut g, gravecaller, None);
    let zombies = g.battlefield.iter().filter(|c| c.definition.name == "Zombie").count();
    assert_eq!(zombies, 2);
}

/// Sharpshooter Elf counts your creatures and shoots an opposing flier for
/// that much.
#[test]
fn sharpshooter_elf_shoots_for_its_power() {
    let mut g = pod(2);
    g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let flier = g.add_card_to_battlefield(1, catalog::scion_of_ugin());
    let elf = g.add_card_to_hand(0, catalog::sharpshooter_elf());
    cast(&mut g, elf, Some(Target::Permanent(flier)));
    assert_eq!(pt(&g, elf), (2, 2), "the Bear and itself");
    assert_eq!(g.battlefield_find(flier).unwrap().damage, 2);
}

/// Wolfrider's Saddle arrives riding its own 2/2 Wolf.
#[test]
fn wolfriders_saddle_comes_with_a_wolf() {
    let mut g = pod(2);
    let saddle = g.add_card_to_hand(0, catalog::wolfriders_saddle());
    cast(&mut g, saddle, None);
    let wolf = g.battlefield.iter().find(|c| c.definition.name == "Wolf").expect("a Wolf").id;
    assert_eq!(g.battlefield_find(saddle).unwrap().attached_to, Some(wolf));
    assert_eq!(pt(&g, wolf), (3, 3));
}

/// Morselhoarder pays mana out of its -1/-1 counters, growing as it does.
#[test]
fn morselhoarder_spends_its_counters_for_mana() {
    let mut g = pod(2);
    let hoarder = g.add_card_to_hand(0, catalog::morselhoarder());
    cast(&mut g, hoarder, None);
    assert_eq!(pt(&g, hoarder), (4, 2));
    activate(&mut g, hoarder, 0).expect("mana");
    activate(&mut g, hoarder, 0).expect("mana");
    assert_eq!(pt(&g, hoarder), (6, 4));
    assert!(activate(&mut g, hoarder, 0).is_err(), "no counter left");
}

/// Bridge from Below makes a Zombie from your graveyard for your nontoken
/// creature's death and exiles itself on an opponent's.
#[test]
fn bridge_from_below_works_from_the_graveyard() {
    let mut g = pod(2);
    let bridge = g.add_card_to_graveyard(0, catalog::bridge_from_below());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(mine)));
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Zombie").count(), 1);
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(theirs)));
    assert!(g.exile.iter().any(|c| c.id == bridge), "an opponent's creature died");
}

/// Sigil of the Nayan Gods counts every creature you control.
#[test]
fn sigil_of_the_nayan_gods_counts_your_creatures() {
    let mut g = pod(2);
    let bear = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    g.add_token_to_battlefield(0, &body("Wolf", 2, CreatureType::Wolf));
    g.add_token_to_battlefield(1, &body("Beast", 2, CreatureType::Beast));
    let sigil = g.add_card_to_hand(0, catalog::sigil_of_the_nayan_gods());
    cast(&mut g, sigil, Some(Target::Permanent(bear)));
    assert_eq!(pt(&g, bear), (4, 4));
}

/// Frontier Mastodon's ferocious counter needs a power-4 creature already
/// in play.
#[test]
fn frontier_mastodon_is_ferocious_beside_a_big_creature() {
    for big in [false, true] {
        let mut g = pod(2);
        if big {
            g.add_token_to_battlefield(0, &body("Beast", 4, CreatureType::Beast));
        }
        let mastodon = g.add_card_to_hand(0, catalog::frontier_mastodon());
        cast(&mut g, mastodon, None);
        assert_eq!(g.battlefield_find(mastodon).unwrap().counter_count(CounterType::PlusOnePlusOne), u32::from(big));
    }
}

/// Esper Sojourners taps a permanent when it dies.
#[test]
fn esper_sojourners_taps_on_death() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let sojourners = g.add_card_to_battlefield(0, catalog::esper_sojourners());
    let theirs = g.add_token_to_battlefield(1, &body("Bear", 2, CreatureType::Bear));
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(sojourners)));
    assert!(g.battlefield_find(theirs).unwrap().tapped);
}

/// Bestial Fury: +4/+0 and trample when its host becomes blocked.
#[test]
fn bestial_fury_rages_when_blocked() {
    let mut g = pod(2);
    let bear = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let wall = g.add_token_to_battlefield(1, &body("Wall", 5, CreatureType::Wall));
    let fury = g.add_card_to_hand(0, catalog::bestial_fury());
    cast(&mut g, fury, Some(Target::Permanent(bear)));
    attack_with(&mut g, &[bear]);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, bear)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (6, 2));
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Trample));
}

/// Spell mastery — Send to Sleep's targets skip their next untap only with
/// two instants/sorceries in your graveyard.
#[test]
fn send_to_sleep_needs_spell_mastery_to_hold_them_down() {
    for mastery in [false, true] {
        let mut g = pod(2);
        let a = g.add_token_to_battlefield(1, &body("Bear", 2, CreatureType::Bear));
        if mastery {
            g.add_card_to_graveyard(0, catalog::lightning_bolt());
            g.add_card_to_graveyard(0, catalog::demons_due());
        }
        let sleep = g.add_card_to_hand(0, catalog::send_to_sleep());
        cast(&mut g, sleep, Some(Target::Permanent(a)));
        let c = g.battlefield_find(a).unwrap();
        assert!(c.tapped);
        assert_eq!(c.skip_next_untap, mastery, "mastery {mastery}");
    }
}

/// Glacial Grasp mills the tapped creature's controller, not the caster.
#[test]
fn glacial_grasp_mills_the_creatures_controller() {
    let mut g = pod(3);
    let theirs = g.add_token_to_battlefield(2, &body("Bear", 2, CreatureType::Bear));
    let (mine, others) = (g.players[0].library.len(), g.players[2].library.len());
    let grasp = g.add_card_to_hand(0, catalog::glacial_grasp());
    cast(&mut g, grasp, Some(Target::Permanent(theirs)));
    assert_eq!(g.players[2].library.len(), others - 2);
    assert_eq!(g.players[0].library.len(), mine - 1, "the caster drew one");
    assert!(g.battlefield_find(theirs).unwrap().tapped);
}

/// Kathari Bomber trades itself for two Goblins when it connects.
#[test]
fn kathari_bomber_drops_goblins_and_goes() {
    let mut g = pod(2);
    let bomber = g.add_card_to_battlefield(0, catalog::kathari_bomber());
    attack_with(&mut g, &[bomber]);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    while g.step != TurnStep::EndCombat && g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(bomber).is_none(), "sacrificed");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Goblin").count(), 2);
}

/// Dose of Dawnglow blights 2 only outside your main phase.
#[test]
fn dose_of_dawnglow_blights_off_the_main_phase() {
    for main in [true, false] {
        let mut g = pod(2);
        let bears = g.add_card_to_graveyard(0, catalog::webweaver_changeling());
        if !main {
            g.step = TurnStep::End;
        }
        let dose = g.add_card_to_hand(0, catalog::dose_of_dawnglow());
        cast(&mut g, dose, Some(Target::Permanent(bears)));
        let counters = g.battlefield_find(bears).map_or(0, |c| c.counter_count(CounterType::MinusOneMinusOne));
        assert_eq!(counters, if main { 0 } else { 2 }, "main phase {main}");
    }
}

/// Embereth Skyblazer flies only on its controller's turn and pumps by the
/// number of opponents.
#[test]
fn embereth_skyblazer_counts_opponents() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(3);
    let blazer = g.add_card_to_battlefield(0, catalog::embereth_skyblazer());
    assert!(g.computed_permanent(blazer).unwrap().keywords().contains(&Keyword::Flying));
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(blazer).unwrap().keywords().contains(&Keyword::Flying));
    g.active_player_idx = 0;
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack_with(&mut g, &[blazer]);
    assert_eq!(pt(&g, blazer), (6, 3), "two opponents");
}

/// Vicious Offering: -2/-2, or -5/-5 when kicked by a sacrifice.
#[test]
fn vicious_offering_kicked_by_a_sacrifice() {
    let mut g = pod(2);
    let fodder = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let big = g.add_token_to_battlefield(1, &body("Beast", 5, CreatureType::Beast));
    let offering = g.add_card_to_hand(0, catalog::vicious_offering());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: offering,
        target: Some(Target::Permanent(big)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("kicked cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "the kicker's sacrifice");
    assert!(g.battlefield_find(big).is_none(), "-5/-5");
}

/// Blitz Leech strips every counter from its target.
#[test]
fn blitz_leech_strips_counters() {
    let mut g = pod(2);
    let theirs = g.add_token_to_battlefield(1, &body("Beast", 3, CreatureType::Beast));
    g.battlefield_find_mut(theirs).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    let leech = g.add_card_to_hand(0, catalog::blitz_leech());
    cast(&mut g, leech, Some(Target::Permanent(theirs)));
    assert_eq!(g.battlefield_find(theirs).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!(pt(&g, theirs), (1, 1));
}

/// Excavation Explosion leaves a tapped Powerstone.
#[test]
fn excavation_explosion_leaves_a_tapped_powerstone() {
    let mut g = pod(2);
    let explosion = g.add_card_to_hand(0, catalog::excavation_explosion());
    cast(&mut g, explosion, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, g.players[0].life - 3);
    let stone = g.battlefield.iter().find(|c| c.definition.name == "Powerstone").expect("a Powerstone");
    assert!(stone.tapped);
}

fn equip(g: &mut GameState, equipment: CardId, host: CardId) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment, target: host }).expect("equip");
    drain_stack(g);
}

/// Soulstinger's death passes on as many -1/-1 counters as it carried.
#[test]
fn soulstinger_passes_its_counters_on() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let stinger = g.add_card_to_hand(0, catalog::soulstinger());
    cast(&mut g, stinger, None);
    assert_eq!(pt(&g, stinger), (2, 3), "it put its two counters on itself");
    let theirs = g.add_token_to_battlefield(1, &body("Beast", 4, CreatureType::Beast));
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(stinger)));
    assert_eq!(g.battlefield_find(theirs).unwrap().counter_count(CounterType::MinusOneMinusOne), 2);
}

/// Bladed Bracers grant vigilance to a Human or Angel only; True-Faith Censer
/// gives a Human an extra +1/+0.
#[test]
fn human_matters_equipment() {
    let mut g = pod(2);
    let human = g.add_token_to_battlefield(0, &body("Soldier", 2, CreatureType::Human));
    let bear = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let bracers_h = g.add_card_to_battlefield(0, catalog::bladed_bracers());
    let bracers_b = g.add_card_to_battlefield(0, catalog::bladed_bracers());
    equip(&mut g, bracers_h, human);
    equip(&mut g, bracers_b, bear);
    let vig = |g: &GameState, id| g.computed_permanent(id).unwrap().keywords().contains(&Keyword::Vigilance);
    assert!(vig(&g, human) && !vig(&g, bear));
    let censer = g.add_card_to_battlefield(0, catalog::true_faith_censer());
    equip(&mut g, censer, human);
    assert_eq!(pt(&g, human), (5, 4), "2/2 + bracers 1/1 + censer 1/1 + Human 1/0");
}

/// Avacyn's Collar leaves a Spirit only when the Human it equipped dies.
#[test]
fn avacyns_collar_remembers_a_human() {
    for human in [false, true] {
        let mut g = pod(2);
        let kind = if human { CreatureType::Human } else { CreatureType::Bear };
        let host = g.add_token_to_battlefield(0, &body("Host", 2, kind));
        let collar = g.add_card_to_battlefield(0, catalog::avacyns_collar());
        equip(&mut g, collar, host);
        let murder = g.add_card_to_hand(0, catalog::murder());
        cast(&mut g, murder, Some(Target::Permanent(host)));
        let spirits = g.battlefield.iter().filter(|c| c.definition.name == "Spirit").count();
        assert_eq!(spirits, usize::from(human), "human {human}");
    }
}

/// Invasion of Dominaria enters for 4 life and a card.
#[test]
fn invasion_of_dominaria_gains_and_draws() {
    let mut g = pod(2);
    let (life, hand) = (g.players[0].life, g.players[0].hand.len());
    let invasion = g.add_card_to_hand(0, catalog::invasion_of_dominaria());
    cast(&mut g, invasion, Some(Target::Player(1)));
    assert_eq!(g.players[0].life, life + 4);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}

/// Mindwrack Demon costs 4 life each upkeep until delirium.
#[test]
fn mindwrack_demon_charges_without_delirium() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::mindwrack_demon());
    let life = g.players[0].life;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::End;
    while !(g.active_player_idx == 0 && g.step == TurnStep::Upkeep) {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 4);
}

/// Furystoke Giant arms the rest of the team with a 2-damage tap ability.
#[test]
fn furystoke_giant_arms_the_team() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let giant = g.add_card_to_hand(0, catalog::furystoke_giant());
    cast(&mut g, giant, None);
    let before = g.players[1].life;
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("the granted ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 2);
}

/// Lose Calm borrows a creature with haste and menace.
#[test]
fn lose_calm_borrows_with_menace() {
    let mut g = pod(3);
    let theirs = g.add_token_to_battlefield(2, &body("Beast", 3, CreatureType::Beast));
    g.battlefield_find_mut(theirs).unwrap().tapped = true;
    let calm = g.add_card_to_hand(0, catalog::lose_calm());
    cast(&mut g, calm, Some(Target::Permanent(theirs)));
    let c = g.computed_permanent(theirs).unwrap();
    assert_eq!(c.controller, 0);
    assert!(c.keywords().contains(&Keyword::Menace) && c.keywords().contains(&Keyword::Haste));
    assert!(!g.battlefield_find(theirs).unwrap().tapped);
}

/// Formidable — Dragon Whisperer makes a Dragon only with 8 total power.
#[test]
fn dragon_whisperer_needs_formidable() {
    let mut g = pod(2);
    let whisperer = g.add_card_to_battlefield(0, catalog::dragon_whisperer());
    assert!(activate(&mut g, whisperer, 2).is_err(), "2 power");
    g.add_token_to_battlefield(0, &body("Beast", 6, CreatureType::Beast));
    activate(&mut g, whisperer, 2).expect("8 power");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Dragon"));
}

/// Lapis Lazuli Talisman answers an opponent's blue spell with an untap.
#[test]
fn lapis_lazuli_talisman_untaps_on_any_blue_spell() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::lapis_lazuli_talisman());
    let mine = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    g.battlefield_find_mut(mine).unwrap().tapped = true;
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let blue = g.add_card_to_hand(1, catalog::hydrolash());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: blue, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("a blue spell");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(mine).unwrap().tapped);
}

/// Riddleform becomes a 3/3 flying Sphinx when you cast a noncreature spell.
#[test]
fn riddleform_wakes_on_a_noncreature_spell() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let form = g.add_card_to_battlefield(0, catalog::riddleform());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let due = g.add_card_to_hand(0, catalog::demons_due());
    cast(&mut g, due, None);
    assert_eq!(pt(&g, form), (3, 3));
    assert!(g.computed_permanent(form).unwrap().keywords().contains(&Keyword::Flying));
}

/// Pyrrhic Blast hits for the sacrificed creature's power and draws.
#[test]
fn pyrrhic_blast_flings_and_draws() {
    let mut g = pod(2);
    g.add_token_to_battlefield(0, &body("Beast", 5, CreatureType::Beast));
    let life = g.players[1].life;
    let hand = g.players[0].hand.len();
    let blast = g.add_card_to_hand(0, catalog::pyrrhic_blast());
    cast(&mut g, blast, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 5);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Kithkin Mourncaller draws for an attacking Elf's death, not an idle one.
#[test]
fn kithkin_mourncaller_mourns_attackers() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::kithkin_mourncaller());
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    attack_with(&mut g, &[elf]);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(elf)));
    assert_eq!(g.players[0].hand.len(), hand + 1, "the Bolt left, the draw came in");
}

/// Hold for Questioning taps, investigates, and shuts the host's abilities
/// off.
#[test]
fn hold_for_questioning_locks_activated_abilities() {
    let mut g = pod(2);
    let elf = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    let hold = g.add_card_to_hand(0, catalog::hold_for_questioning());
    cast(&mut g, hold, Some(Target::Permanent(elf)));
    assert!(g.battlefield_find(elf).unwrap().tapped);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Clue"));
    assert!(g.computed_permanent(elf).unwrap().keywords().contains(&Keyword::CantActivateAbilities));
}

/// Winterflame may both tap one creature and burn another.
#[test]
fn winterflame_takes_both_modes() {
    let mut g = pod(2);
    let a = g.add_token_to_battlefield(1, &body("Beast", 3, CreatureType::Beast));
    let b = g.add_token_to_battlefield(1, &body("Bear", 2, CreatureType::Bear));
    let flame = g.add_card_to_hand(0, catalog::winterflame());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: flame,
        spree_modes: vec![0, 1],
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        x_value: None,
    })
    .expect("both modes");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().tapped);
    assert!(g.battlefield_find(b).is_none());
}

/// Heart-Piercer Bow pings a defender's creature from the Equipment.
#[test]
fn heart_piercer_bow_pings_a_blocker_to_be() {
    let mut g = pod(2);
    let bear = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let theirs = g.add_token_to_battlefield(1, &body("Beast", 3, CreatureType::Beast));
    let bow = g.add_card_to_battlefield(0, catalog::heart_piercer_bow());
    equip(&mut g, bow, bear);
    attack_with(&mut g, &[bear]);
    assert_eq!(g.battlefield_find(theirs).unwrap().damage, 1);
}

/// Protomatter Powder returns an artifact card from your graveyard.
#[test]
fn protomatter_powder_returns_an_artifact() {
    let mut g = pod(2);
    let powder = g.add_card_to_battlefield(0, catalog::protomatter_powder());
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    activate_on(&mut g, powder, 0, Some(Target::Permanent(ring))).expect("activate");
    assert!(g.battlefield_find(ring).is_some());
}

/// Angelic Armaments makes its host a white Angel in addition to its other
/// colours and types (layer 4 / layer 5 additions, not replacements).
#[test]
fn angelic_armaments_adds_white_and_angel() {
    let mut g = pod(2);
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let arms = g.add_card_to_battlefield(0, catalog::angelic_armaments());
    equip(&mut g, arms, elf);
    let c = g.computed_permanent(elf).unwrap();
    assert!(c.colors.contains(Color::White) && c.colors.contains(Color::Green));
    assert!(c.subtypes().creature_types.contains(&CreatureType::Angel));
    assert!(c.subtypes().creature_types.contains(&CreatureType::Elf));
    assert!(c.keywords().contains(&Keyword::Flying));
    assert_eq!(pt(&g, elf), (3, 3));
}

/// Spell Blast counters only a spell whose mana value is exactly X.
#[test]
fn spell_blast_matches_mana_value_x() {
    let mut g = pod(2);
    let theirs = g.add_card_to_hand(1, catalog::grizzly_bears());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.active_player_idx = 1;
    g.perform_action(GameAction::CastSpell { card_id: theirs, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("the Bears");
    let blast = g.add_card_to_hand(0, catalog::spell_blast());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    assert!(
        g.clone()
            .perform_action(GameAction::CastSpell {
                card_id: blast,
                target: Some(Target::Permanent(theirs)),
                additional_targets: vec![],
                mode: None,
                x_value: Some(3),
            })
            .is_err(),
        "X = 3 can't target a two-drop"
    );
    g.perform_action(GameAction::CastSpell {
        card_id: blast,
        target: Some(Target::Permanent(theirs)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("X = 2");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == theirs), "countered");
}

/// Circle of the Moon Druid is a 4/2 Bear only on its controller's turn.
#[test]
fn circle_of_the_moon_druid_takes_bear_form_on_your_turn() {
    let mut g = pod(2);
    let druid = g.add_card_to_battlefield(0, catalog::circle_of_the_moon_druid());
    assert_eq!(pt(&g, druid), (4, 2));
    let types = g.computed_permanent(druid).unwrap().subtypes().creature_types.clone();
    assert_eq!(types, vec![CreatureType::Bear]);
    g.active_player_idx = 1;
    assert_eq!(pt(&g, druid), (2, 4));
    assert!(g.computed_permanent(druid).unwrap().subtypes().creature_types.contains(&CreatureType::Elf));
}

/// Awakener Druid's Forest stays a 4/5 Treefolk only while the Druid does.
#[test]
fn awakener_druid_wakes_a_forest_while_it_stays() {
    let mut g = pod(2);
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let druid = g.add_card_to_hand(0, catalog::awakener_druid());
    cast(&mut g, druid, Some(Target::Permanent(forest)));
    assert_eq!(pt(&g, forest), (4, 5));
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(druid)));
    assert!(g.computed_permanent(forest).unwrap().card_types().contains(&CardType::Land));
    assert!(!g.computed_permanent(forest).unwrap().card_types().contains(&CardType::Creature));
}

/// Dust Elemental returns three creatures you control.
#[test]
fn dust_elemental_returns_three_of_yours() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    let elemental = g.add_card_to_hand(0, catalog::dust_elemental());
    cast(&mut g, elemental, None);
    let bears = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    let on_board = g.battlefield_find(elemental).is_some();
    assert_eq!(bears + usize::from(on_board), 1, "three of the four creatures went home");
}

/// Vivien's +1 puts both counters on a lone target and gives it trample.
#[test]
fn vivien_arkbow_ranger_plus_one_on_one_target() {
    let mut g = pod(2);
    let vivien = g.add_card_to_hand(0, catalog::vivien_arkbow_ranger());
    cast(&mut g, vivien, None);
    let beast = g.add_token_to_battlefield(0, &body("Beast", 3, CreatureType::Beast));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: vivien,
        ability_index: 0,
        target: Some(Target::Permanent(beast)),
        x_value: None,
    })
    .expect("+1");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(beast).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert!(g.computed_permanent(beast).unwrap().keywords().contains(&Keyword::Trample));
}

/// CR 704.5q — a Curse that Mirrorform turns into a copy of a creature is no
/// longer an Aura: it stops enchanting its player and stays on the
/// battlefield (bug fix: 4-seat sweep seed 934364 kept the Owlbear "attached"
/// to a seat that had left).
#[test]
fn cr_704_5q_a_curse_turned_creature_lets_go_of_its_player() {
    let mut g = pod(3);
    let curse = g.add_card_to_hand(0, catalog::curse_of_opulence());
    cast(&mut g, curse, Some(Target::Player(1)));
    assert_eq!(g.battlefield_find(curse).unwrap().attached_to_player, Some(1));
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mirror = g.add_card_to_hand(0, catalog::mirrorform());
    cast(&mut g, mirror, Some(Target::Permanent(bears)));
    g.check_state_based_actions();
    let copy = g.battlefield_find(curse).expect("the copy stays on the battlefield");
    assert_eq!(copy.definition.name, "Grizzly Bears");
    assert_eq!(copy.attached_to_player, None);
}

/// CR 702.96 — Stirring Address's overload pumps every creature you control.
#[test]
fn cr_702_96_stirring_address_overloads() {
    let mut g = pod(2);
    let a = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let b = g.add_token_to_battlefield(0, &body("Wolf", 2, CreatureType::Wolf));
    let theirs = g.add_token_to_battlefield(1, &body("Beast", 2, CreatureType::Beast));
    let address = g.add_card_to_hand(0, catalog::stirring_address());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: address,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("overload");
    drain_stack(&mut g);
    assert_eq!(pt(&g, a), (4, 4));
    assert_eq!(pt(&g, b), (4, 4));
    assert_eq!(pt(&g, theirs), (2, 2));
}

/// Luxa River Shrine's big gain needs three brick counters.
#[test]
fn luxa_river_shrine_builds_to_two_life() {
    let mut g = pod(2);
    let shrine = g.add_card_to_battlefield(0, catalog::luxa_river_shrine());
    assert!(activate(&mut g, shrine, 1).is_err(), "no bricks yet");
    for _ in 0..3 {
        activate(&mut g, shrine, 0).expect("brick");
        g.battlefield_find_mut(shrine).unwrap().tapped = false;
    }
    let life = g.players[0].life;
    activate(&mut g, shrine, 1).expect("three bricks");
    assert_eq!(g.players[0].life, life + 2);
}

/// Scarab Feast's three targets come from one graveyard.
#[test]
fn scarab_feast_eats_from_a_single_graveyard() {
    let mut g = pod(3);
    let a = g.add_card_to_graveyard(1, catalog::island());
    let b = g.add_card_to_graveyard(1, catalog::island());
    let other = g.add_card_to_graveyard(2, catalog::island());
    let feast = g.add_card_to_hand(0, catalog::scarab_feast());
    flood(&mut g, 0);
    assert!(
        g.clone()
            .perform_action(GameAction::CastSpell {
                card_id: feast,
                target: Some(Target::Permanent(a)),
                additional_targets: vec![Target::Permanent(other)],
                mode: None,
                x_value: None,
            })
            .is_err(),
        "two graveyards"
    );
    g.perform_action(GameAction::CastSpell {
        card_id: feast,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("one graveyard");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == a) && g.exile.iter().any(|c| c.id == b));
}

/// Galvanic Juggernaut stays tapped until another creature dies.
#[test]
fn galvanic_juggernaut_wakes_on_a_death() {
    let mut g = pod(2);
    let jugg = g.add_card_to_battlefield(0, catalog::galvanic_juggernaut());
    g.battlefield_find_mut(jugg).unwrap().tapped = true;
    let bear = g.add_token_to_battlefield(1, &body("Bear", 2, CreatureType::Bear));
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(bear)));
    assert!(!g.battlefield_find(jugg).unwrap().tapped);
}

/// Haunted Dead climbs back for two discards.
#[test]
fn haunted_dead_returns_for_two_discards() {
    let mut g = pod(2);
    let dead = g.add_card_to_graveyard(0, catalog::haunted_dead());
    g.add_card_to_hand(0, catalog::island());
    assert!(activate(&mut g, dead, 0).is_err(), "one card in hand");
    g.add_card_to_hand(0, catalog::island());
    activate(&mut g, dead, 0).expect("two cards");
    let c = g.battlefield_find(dead).expect("back");
    assert!(c.tapped);
    assert!(g.players[0].hand.is_empty());
}

/// Castle Raptors is 3/5 untapped, 3/3 tapped.
#[test]
fn castle_raptors_stands_tall_untapped() {
    let mut g = pod(2);
    let raptors = g.add_card_to_battlefield(0, catalog::castle_raptors());
    assert_eq!(pt(&g, raptors), (3, 5));
    g.battlefield_find_mut(raptors).unwrap().tapped = true;
    assert_eq!(pt(&g, raptors), (3, 3));
}

/// Skilled Animator makes an artifact a 5/5 only while it stays.
#[test]
fn skilled_animator_animates_while_it_stays() {
    let mut g = pod(2);
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let animator = g.add_card_to_hand(0, catalog::skilled_animator());
    cast(&mut g, animator, Some(Target::Permanent(ring)));
    assert_eq!(pt(&g, ring), (5, 5));
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(animator)));
    assert!(!g.computed_permanent(ring).unwrap().card_types().contains(&CardType::Creature));
}

/// See Red goes to the graveyard at an end step after no attack.
#[test]
fn see_red_demands_an_attack() {
    let mut g = pod(2);
    let bear = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let red = g.add_card_to_hand(0, catalog::see_red());
    cast(&mut g, red, Some(Target::Permanent(bear)));
    assert_eq!(pt(&g, bear), (4, 3));
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(red).is_none(), "no attack this turn");
}

/// Grim Contest: each creature deals damage equal to its toughness to the
/// other.
#[test]
fn grim_contest_trades_toughness_damage() {
    let mut g = pod(2);
    let mine = g.add_token_to_battlefield(0, &body("Beast", 4, CreatureType::Beast));
    let theirs = g.add_token_to_battlefield(1, &body("Bear", 2, CreatureType::Bear));
    let contest = g.add_card_to_hand(0, catalog::grim_contest());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: contest,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "4 damage to the 2/2");
    assert_eq!(g.battlefield_find(mine).expect("survives").damage, 2);
}

/// CR 603.2 — Phantasmal Dragon is sacrificed when it becomes a target.
#[test]
fn cr_603_2_phantasmal_dragon_dies_to_a_target() {
    let mut g = pod(2);
    let dragon = g.add_card_to_battlefield(1, catalog::phantasmal_dragon());
    let familiar = g.add_card_to_battlefield(0, catalog::pseudodragon_familiar());
    activate_on(&mut g, familiar, 0, Some(Target::Permanent(dragon))).expect("activate");
    assert!(g.battlefield_find(dragon).is_none());
}

/// Infernal Reckoning exiles a colorless creature and gains its power.
#[test]
fn infernal_reckoning_exiles_colorless_for_life() {
    let mut g = pod(2);
    let golem = g.add_token_to_battlefield(1, &body("Golem", 4, CreatureType::Golem));
    let life = g.players[0].life;
    let reckoning = g.add_card_to_hand(0, catalog::infernal_reckoning());
    cast(&mut g, reckoning, Some(Target::Permanent(golem)));
    assert!(g.battlefield_find(golem).is_none());
    assert_eq!(g.players[0].life, life + 4);
}

/// Evershrike's Gift returns from the graveyard by blighting 2.
#[test]
fn evershrikes_gift_returns_for_a_blight() {
    let mut g = pod(2);
    let beast = g.add_token_to_battlefield(0, &body("Beast", 4, CreatureType::Beast));
    let gift = g.add_card_to_graveyard(0, catalog::evershrikes_gift());
    activate(&mut g, gift, 0).expect("activate");
    assert!(g.players[0].hand.iter().any(|c| c.id == gift));
    assert_eq!(g.battlefield_find(beast).unwrap().counter_count(CounterType::MinusOneMinusOne), 2);
}

/// Consecrated by Blood regenerates its host for two other creatures.
#[test]
fn consecrated_by_blood_regenerates_for_two() {
    let mut g = pod(2);
    let host = g.add_token_to_battlefield(0, &body("Beast", 4, CreatureType::Beast));
    let a = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let b = g.add_token_to_battlefield(0, &body("Wolf", 2, CreatureType::Wolf));
    let aura = g.add_card_to_hand(0, catalog::consecrated_by_blood());
    cast(&mut g, aura, Some(Target::Permanent(host)));
    assert_eq!(pt(&g, host), (6, 6));
    activate(&mut g, host, 0).expect("regenerate");
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(host)));
    assert!(g.battlefield_find(host).is_some(), "regenerated");
}

/// Midnight Scavengers returns a small creature card.
#[test]
fn midnight_scavengers_returns_a_small_creature() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let beetle = g.add_card_to_graveyard(0, catalog::dune_beetle());
    let scav = g.add_card_to_hand(0, catalog::midnight_scavengers());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, scav, Some(Target::Permanent(beetle)));
    assert!(g.players[0].hand.iter().any(|c| c.id == beetle));
}

/// Embodiment of Agonies counts different mana costs, not cards: two Murders
/// count once, a land never.
#[test]
fn embodiment_of_agonies_counts_distinct_costs() {
    let mut g = pod(2);
    g.add_card_to_graveyard(0, catalog::murder());
    g.add_card_to_graveyard(0, catalog::murder());
    g.add_card_to_graveyard(0, catalog::sol_ring());
    g.add_card_to_graveyard(0, catalog::island());
    let demon = g.add_card_to_hand(0, catalog::embodiment_of_agonies());
    cast(&mut g, demon, None);
    assert_eq!(g.battlefield_find(demon).expect("alive").counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Rakshasa Deathdealer regenerates out of a Murder.
#[test]
fn rakshasa_deathdealer_regenerates() {
    let mut g = pod(2);
    let demon = g.add_card_to_battlefield(0, catalog::rakshasa_deathdealer());
    activate(&mut g, demon, 0).expect("pump");
    assert_eq!(pt(&g, demon), (4, 4));
    activate(&mut g, demon, 1).expect("regenerate");
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(demon)));
    assert!(g.battlefield_find(demon).is_some());
}

/// Faithful Pikemaster has first strike only on its controller's turn.
#[test]
fn faithful_pikemaster_strikes_first_on_your_turn() {
    let mut g = pod(2);
    let pike = g.add_card_to_battlefield(0, catalog::faithful_pikemaster());
    assert!(g.computed_permanent(pike).unwrap().keywords().contains(&Keyword::FirstStrike));
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(pike).unwrap().keywords().contains(&Keyword::FirstStrike));
}

/// Judge Unworthy hits an attacker for the revealed card's mana value.
#[test]
fn judge_unworthy_reveals_for_damage() {
    let mut g = pod(2);
    g.players[0].library.clear();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::murder());
    }
    let beast = g.add_token_to_battlefield(0, &body("Beast", 3, CreatureType::Beast));
    attack_with(&mut g, &[beast]);
    let judge = g.add_card_to_hand(0, catalog::judge_unworthy());
    cast(&mut g, judge, Some(Target::Permanent(beast)));
    assert!(g.battlefield_find(beast).is_none(), "3 damage from a Murder on top");
}

/// Myth Realized gathers lore and stands up at its lore count.
#[test]
fn myth_realized_animates_at_its_lore_count() {
    let mut g = pod(2);
    let myth = g.add_card_to_battlefield(0, catalog::myth_realized());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, ring, None);
    activate(&mut g, myth, 0).expect("lore");
    assert_eq!(g.battlefield_find(myth).unwrap().counter_count(CounterType::Lore), 2);
    activate(&mut g, myth, 1).expect("animate");
    let c = g.computed_permanent(myth).unwrap();
    assert!(c.card_types().contains(&CardType::Creature));
    assert_eq!((c.power, c.toughness), (2, 2));
}

/// CR 702.148 — Lantern Flare counts creatures; cleaved, it reads X.
#[test]
fn cr_702_148_lantern_flare_cleaves_to_x() {
    let mut g = pod(2);
    g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let big = g.add_token_to_battlefield(1, &body("Beast", 4, CreatureType::Beast));
    let life = g.players[0].life;
    let flare = g.add_card_to_hand(0, catalog::lantern_flare());
    cast(&mut g, flare, Some(Target::Permanent(big)));
    assert_eq!(g.battlefield_find(big).unwrap().damage, 1);
    assert_eq!(g.players[0].life, life + 1);
    let flare = g.add_card_to_hand(0, catalog::lantern_flare());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: flare,
        pitch_card: None,
        target: Some(Target::Permanent(big)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("cleave");
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_none(), "1 + 3 damage");
    assert_eq!(g.players[0].life, life + 4);
}

/// Nantuko Shaman draws only when no land you control is tapped.
#[test]
fn nantuko_shaman_needs_untapped_lands() {
    for tapped in [false, true] {
        let mut g = pod(2);
        let land = g.add_card_to_battlefield(0, catalog::forest());
        g.battlefield_find_mut(land).unwrap().tapped = tapped;
        let shaman = g.add_card_to_hand(0, catalog::nantuko_shaman());
        let hand = g.players[0].hand.len();
        cast(&mut g, shaman, None);
        assert_eq!(g.players[0].hand.len(), hand - 1 + usize::from(!tapped), "tapped land {tapped}");
    }
}

/// Surge Engine sheds defender, turns blue at 5/4, then draws three once.
#[test]
fn surge_engine_climbs_its_ladder() {
    let mut g = pod(2);
    let engine = g.add_card_to_battlefield(0, catalog::surge_engine());
    assert!(activate(&mut g, engine, 1).is_err(), "still has defender");
    activate(&mut g, engine, 0).expect("lose defender");
    let c = g.computed_permanent(engine).unwrap();
    let kws = c.keywords();
    assert!(!kws.contains(&Keyword::Defender) && kws.contains(&Keyword::Unblockable));
    assert!(activate(&mut g, engine, 2).is_err(), "not blue yet");
    activate(&mut g, engine, 1).expect("become blue");
    let c = g.computed_permanent(engine).unwrap();
    assert!(c.colors.contains(Color::Blue));
    assert_eq!((c.power, c.toughness), (5, 4));
    let hand = g.players[0].hand.len();
    activate(&mut g, engine, 2).expect("draw");
    assert_eq!(g.players[0].hand.len(), hand + 3);
    assert!(activate(&mut g, engine, 2).is_err(), "only once");
}

/// Inner-Flame Igniter's third resolution adds first strike.
#[test]
fn inner_flame_igniter_third_time_strikes_first() {
    let mut g = pod(2);
    let igniter = g.add_card_to_battlefield(0, catalog::inner_flame_igniter());
    for n in 1..=3 {
        activate(&mut g, igniter, 0).expect("pump");
        let fs = g.computed_permanent(igniter).unwrap().keywords().contains(&Keyword::FirstStrike);
        assert_eq!(fs, n == 3, "resolution {n}");
    }
    assert_eq!(pt(&g, igniter), (5, 2));
}

/// Fatehold Chronologist enters prepared.
#[test]
fn fatehold_chronologist_enters_prepared() {
    let mut g = pod(2);
    let bird = g.add_card_to_hand(0, catalog::fatehold_chronologist());
    cast(&mut g, bird, None);
    assert_eq!(g.battlefield_find(bird).unwrap().counter_count(CounterType::Prepared), 1);
}

/// Mystery Key cashes itself in for three cards on combat damage.
#[test]
fn mystery_key_draws_three_on_a_hit() {
    let mut g = pod(2);
    let bear = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let key = g.add_card_to_battlefield(0, catalog::mystery_key());
    equip(&mut g, key, bear);
    let hand = g.players[0].hand.len();
    attack_with(&mut g, &[bear]);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    while g.step != TurnStep::EndCombat && g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(key).is_none(), "sacrificed");
    assert_eq!(g.players[0].hand.len(), hand + 3);
}

/// Ray of Frost taps a red creature and strips its abilities.
#[test]
fn ray_of_frost_silences_red() {
    let mut g = pod(2);
    let red = g.add_token_to_battlefield(
        1,
        &TokenDefinition {
            colors: vec![Color::Red],
            keywords: vec![Keyword::Haste],
            ..body("Goblin", 2, CreatureType::Goblin)
        },
    );
    let ray = g.add_card_to_hand(0, catalog::ray_of_frost());
    cast(&mut g, ray, Some(Target::Permanent(red)));
    assert!(g.battlefield_find(red).unwrap().tapped);
    assert!(!g.computed_permanent(red).unwrap().keywords().contains(&Keyword::Haste));
}

/// Tamiyo's Compleation taps its host and strips its abilities.
#[test]
fn tamiyos_compleation_strips_abilities() {
    let mut g = pod(2);
    let hasty = g.add_token_to_battlefield(
        1,
        &TokenDefinition { keywords: vec![Keyword::Haste], ..body("Goblin", 2, CreatureType::Goblin) },
    );
    let aura = g.add_card_to_hand(0, catalog::tamiyos_compleation());
    cast(&mut g, aura, Some(Target::Permanent(hasty)));
    assert!(g.battlefield_find(hasty).unwrap().tapped);
    assert!(!g.computed_permanent(hasty).unwrap().keywords().contains(&Keyword::Haste));
}

/// Ludevic's Test Subject hatches at its fifth counter.
#[test]
fn ludevics_test_subject_hatches_at_five() {
    let mut g = pod(2);
    let egg = g.add_card_to_battlefield(0, catalog::ludevics_test_subject());
    for _ in 0..4 {
        activate(&mut g, egg, 0).expect("feed");
    }
    assert_eq!(pt(&g, egg), (0, 3));
    activate(&mut g, egg, 0).expect("hatch");
    assert_eq!(pt(&g, egg), (13, 13));
    assert_eq!(g.battlefield_find(egg).unwrap().counter_count(CounterType::Hatchling), 0);
}

/// Skaab Goliath needs two creature cards to exile.
#[test]
fn skaab_goliath_exiles_two_creature_cards() {
    let mut g = pod(2);
    g.add_card_to_graveyard(0, catalog::dune_beetle());
    let goliath = g.add_card_to_hand(0, catalog::skaab_goliath());
    flood(&mut g, 0);
    let cast_it = |g: &mut GameState| {
        g.perform_action(GameAction::CastSpell {
            card_id: goliath,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    assert!(cast_it(&mut g).is_err(), "one creature card");
    g.add_card_to_graveyard(0, catalog::dune_beetle());
    cast_it(&mut g).expect("two creature cards");
    drain_stack(&mut g);
    assert!(g.battlefield_find(goliath).is_some());
    assert!(g.players[0].graveyard.is_empty());
}

/// Foreboding Statue transforms at its end step after three omen counters.
#[test]
fn foreboding_statue_transforms_at_three_omens() {
    let mut g = pod(2);
    let statue = g.add_card_to_battlefield(0, catalog::foreboding_statue());
    g.battlefield_find_mut(statue).unwrap().add_counters(CounterType::Omen, 2);
    g.clear_sickness(statue);
    activate(&mut g, statue, 0).expect("tap for mana");
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
    }
    drain_stack(&mut g);
    let c = g.battlefield_find(statue).unwrap();
    assert!(!c.tapped, "untapped");
    assert_eq!(pt(&g, statue), (5, 5));
}

/// Profound Journey returns a permanent card.
#[test]
fn profound_journey_returns_a_permanent_card() {
    let mut g = pod(2);
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    let journey = g.add_card_to_hand(0, catalog::profound_journey());
    cast(&mut g, journey, Some(Target::Permanent(ring)));
    assert!(g.battlefield_find(ring).is_some());
}

/// Angrath's +2 pings the opponent and each of their creatures.
#[test]
fn angrath_minotaur_pirate_pings_a_side() {
    let mut g = pod(3);
    let angrath = g.add_card_to_hand(0, catalog::angrath_minotaur_pirate());
    cast(&mut g, angrath, None);
    let theirs = g.add_token_to_battlefield(1, &body("Elf", 1, CreatureType::Elf));
    let bystander = g.add_token_to_battlefield(2, &body("Elf", 1, CreatureType::Elf));
    let life = g.players[1].life;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: angrath,
        ability_index: 0,
        target: Some(Target::Player(1)),
        x_value: None,
    })
    .expect("+2");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1);
    assert!(g.battlefield_find(theirs).is_none());
    assert!(g.battlefield_find(bystander).is_some());
}

/// Angrath's Fury tutors Angrath out of the graveyard.
#[test]
fn angraths_fury_finds_angrath() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let angrath = g.add_card_to_graveyard(0, catalog::angrath_minotaur_pirate());
    let bear = g.add_token_to_battlefield(1, &body("Bear", 2, CreatureType::Bear));
    let fury = g.add_card_to_hand(0, catalog::angraths_fury());
    let life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: fury,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Player(1)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[1].life, life - 3);
    assert!(g.players[0].hand.iter().any(|c| c.id == angrath));
}

/// Sphinx-Bone Wand grows a counter per spell and burns for the total.
#[test]
fn sphinx_bone_wand_burns_for_its_counters() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let wand = g.add_card_to_battlefield(0, catalog::sphinx_bone_wand());
    g.battlefield_find_mut(wand).unwrap().add_counters(CounterType::Charge, 2);
    let life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let syphon = g.add_card_to_hand(0, catalog::agonizing_syphon());
    let beast = g.add_token_to_battlefield(0, &body("Beast", 5, CreatureType::Beast));
    cast(&mut g, syphon, Some(Target::Permanent(beast)));
    assert_eq!(g.battlefield_find(wand).unwrap().counter_count(CounterType::Charge), 3);
    assert!(g.players[1].life <= life, "the wand found a target");
}

/// Relic Runner can't be blocked after a historic spell.
#[test]
fn relic_runner_slips_by_after_a_historic_spell() {
    let mut g = pod(2);
    let runner = g.add_card_to_battlefield(0, catalog::relic_runner());
    assert!(!g.computed_permanent(runner).unwrap().keywords().contains(&Keyword::Unblockable));
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, ring, None);
    assert!(g.computed_permanent(runner).unwrap().keywords().contains(&Keyword::Unblockable));
}

/// Watchwing Scarecrow reads your creatures' colours.
#[test]
fn watchwing_scarecrow_reads_colors() {
    let mut g = pod(2);
    let crow = g.add_card_to_battlefield(0, catalog::watchwing_scarecrow());
    assert!(!g.computed_permanent(crow).unwrap().keywords().contains(&Keyword::Flying));
    g.add_card_to_battlefield(0, catalog::relic_runner());
    let c = g.computed_permanent(crow).unwrap();
    let kws = c.keywords();
    assert!(kws.contains(&Keyword::Flying) && !kws.contains(&Keyword::Vigilance));
}

/// Counterintelligence returns two creatures.
#[test]
fn counterintelligence_returns_two() {
    let mut g = pod(2);
    let a = g.add_token_to_battlefield(1, &body("Bear", 2, CreatureType::Bear));
    let b = g.add_card_to_battlefield(1, catalog::dune_beetle());
    let spell = g.add_card_to_hand(0, catalog::counterintelligence());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    assert!(g.players[1].hand.iter().any(|c| c.id == b));
}

/// CR 109.5 — a keyword-while static reads "you" as its controller, not seat
/// 0: Faithful Pikemaster strikes first on its own controller's turn.
#[test]
fn cr_109_5_pikemaster_reads_its_own_controller() {
    let mut g = pod(3);
    let pike = g.add_card_to_battlefield(2, catalog::faithful_pikemaster());
    g.active_player_idx = 2;
    assert!(g.computed_permanent(pike).unwrap().keywords().contains(&Keyword::FirstStrike));
    g.active_player_idx = 0;
    assert!(!g.computed_permanent(pike).unwrap().keywords().contains(&Keyword::FirstStrike));
}

/// CR 122.1a — Armor Thrull's +1/+2 counter adds one power and two toughness.
#[test]
fn cr_122_1a_armor_thrull_leaves_a_plus_one_plus_two_counter() {
    let mut g = pod(2);
    let thrull = g.add_card_to_battlefield(0, catalog::armor_thrull());
    g.clear_sickness(thrull);
    let bear = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    activate_on(&mut g, thrull, 0, Some(Target::Permanent(bear))).expect("sacrifice");
    assert!(g.battlefield_find(thrull).is_none());
    assert_eq!(pt(&g, bear), (3, 4));
}

/// Mindstab Thrull trades itself for three discards when unblocked.
#[test]
fn mindstab_thrull_trades_itself_for_three_discards() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_hand(1, catalog::island());
    }
    let thrull = g.add_card_to_battlefield(0, catalog::mindstab_thrull());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack_with(&mut g, &[thrull]);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    assert!(g.battlefield_find(thrull).is_none(), "sacrificed");
    assert_eq!(g.players[1].hand.len(), 1);
}

/// Thrull Champion pumps every Thrull and steals one.
#[test]
fn thrull_champion_pumps_and_steals_thrulls() {
    let mut g = pod(2);
    let champion = g.add_card_to_battlefield(0, catalog::thrull_champion());
    g.clear_sickness(champion);
    let theirs = g.add_card_to_battlefield(1, catalog::armor_thrull());
    assert_eq!(pt(&g, theirs), (2, 4));
    activate_on(&mut g, champion, 0, Some(Target::Permanent(theirs))).expect("steal");
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0);
}

/// Contagion, pitched, puts two -2/-1 counters on one creature.
#[test]
fn contagion_pitches_for_two_minus_two_minus_one_counters() {
    let mut g = pod(2);
    let beast = g.add_token_to_battlefield(1, &body("Beast", 3, CreatureType::Beast));
    let pitch = g.add_card_to_hand(0, catalog::murder());
    let contagion = g.add_card_to_hand(0, catalog::contagion());
    let life = g.players[0].life;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: contagion,
        pitch_card: Some(pitch),
        target: Some(Target::Permanent(beast)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("pitch");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 1);
    assert_eq!(pt(&g, beast), (-1, 1));
}

/// Solitary Camel has lifelink with a Desert in the graveyard.
#[test]
fn solitary_camel_drinks_from_a_desert() {
    let mut g = pod(2);
    let camel = g.add_card_to_battlefield(0, catalog::solitary_camel());
    assert!(!g.computed_permanent(camel).unwrap().keywords().contains(&Keyword::Lifelink));
    g.add_card_to_graveyard(0, catalog::desert());
    assert!(g.computed_permanent(camel).unwrap().keywords().contains(&Keyword::Lifelink));
}

/// Call of the Full Moon: +3/+2 and trample.
#[test]
fn call_of_the_full_moon_pumps_its_host() {
    let mut g = pod(2);
    let bear = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    let aura = g.add_card_to_hand(0, catalog::call_of_the_full_moon());
    cast(&mut g, aura, Some(Target::Permanent(bear)));
    assert_eq!(pt(&g, bear), (5, 4));
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Trample));
}

/// Evolved Sleeper climbs Cleric, then Phyrexian, then draws.
#[test]
fn evolved_sleeper_climbs_its_ladder() {
    let mut g = pod(2);
    let sleeper = g.add_card_to_battlefield(0, catalog::evolved_sleeper());
    activate(&mut g, sleeper, 1).expect("not a Cleric yet: no-op");
    assert_eq!(pt(&g, sleeper), (1, 1));
    activate(&mut g, sleeper, 0).expect("Cleric");
    assert_eq!(pt(&g, sleeper), (2, 2));
    activate(&mut g, sleeper, 1).expect("Phyrexian");
    assert_eq!(pt(&g, sleeper), (3, 3));
    assert!(g.computed_permanent(sleeper).unwrap().keywords().contains(&Keyword::Deathtouch));
    let hand = g.players[0].hand.len();
    activate(&mut g, sleeper, 2).expect("grow");
    assert_eq!(pt(&g, sleeper), (4, 4));
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Stallion of Ashmouth pumps only with delirium.
#[test]
fn stallion_of_ashmouth_needs_delirium() {
    let mut g = pod(2);
    let stallion = g.add_card_to_battlefield(0, catalog::stallion_of_ashmouth());
    assert!(activate(&mut g, stallion, 0).is_err(), "no delirium");
    for card in [catalog::island(), catalog::murder(), catalog::sol_ring(), catalog::dune_beetle()] {
        g.add_card_to_graveyard(0, card);
    }
    activate(&mut g, stallion, 0).expect("delirium");
    assert_eq!(pt(&g, stallion), (4, 4));
}

/// Thriving Rats spends its two energy on an attack counter.
#[test]
fn thriving_rats_spends_energy_on_attack() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let rats = g.add_card_to_hand(0, catalog::thriving_rats());
    cast(&mut g, rats, None);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack_with(&mut g, &[rats]);
    assert_eq!(g.battlefield_find(rats).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Dire Fleet Warmonger eats another creature at combat for +2/+2 and trample.
#[test]
fn dire_fleet_warmonger_eats_for_trample() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let orc = g.add_card_to_battlefield(0, catalog::dire_fleet_warmonger());
    let snack = g.add_token_to_battlefield(0, &body("Bear", 2, CreatureType::Bear));
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    while g.step != TurnStep::BeginCombat {
        let _ = g.advance_step(Vec::new());
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(snack).is_none(), "sacrificed");
    assert_eq!(pt(&g, orc), (5, 5));
    assert!(g.computed_permanent(orc).unwrap().keywords().contains(&Keyword::Trample));
}

/// Splitting the Powerstone draws only off a legendary sacrifice.
#[test]
fn splitting_the_powerstone_draws_off_a_legend() {
    for legendary in [false, true] {
        let mut g = pod(2);
        let fodder = if legendary { catalog::the_mana_rig() } else { catalog::sol_ring() };
        let art = g.add_card_to_battlefield(0, fodder);
        let split = g.add_card_to_hand(0, catalog::splitting_the_powerstone());
        let hand = g.players[0].hand.len();
        cast(&mut g, split, None);
        assert!(g.battlefield_find(art).is_none(), "sacrificed");
        let stones: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Powerstone").collect();
        assert_eq!(stones.len(), 2);
        assert!(stones.iter().all(|c| c.tapped));
        assert_eq!(g.players[0].hand.len(), hand - 1 + usize::from(legendary), "legendary {legendary}");
    }
}

/// The Mana Rig makes a Powerstone off a multicolored spell and digs for X.
#[test]
fn the_mana_rig_powers_up_and_digs() {
    let mut g = pod(2);
    let rig = g.add_card_to_battlefield(0, catalog::the_mana_rig());
    let demon = g.add_card_to_hand(0, catalog::rakshasa_deathdealer());
    cast(&mut g, demon, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Powerstone").count(), 1);
    let hand = g.players[0].hand.len();
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: rig,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: Some(3),
        mode: None,
    })
    .expect("dig");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

/// Shreds of Sanity returns an instant and a sorcery, then discards, then
/// exiles itself.
#[test]
fn shreds_of_sanity_returns_both_halves() {
    let mut g = pod(2);
    let murder = g.add_card_to_graveyard(0, catalog::murder());
    let syphon = g.add_card_to_graveyard(0, catalog::agonizing_syphon());
    let shreds = g.add_card_to_hand(0, catalog::shreds_of_sanity());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: shreds,
        target: Some(Target::Permanent(murder)),
        additional_targets: vec![Target::Permanent(syphon)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "two back, one discarded");
    assert!(g.exile.iter().any(|c| c.id == shreds));
}

/// Curious Homunculus transforms at upkeep with three instants and sorceries
/// in the graveyard; the Reader discounts them.
#[test]
fn curious_homunculus_reads_its_way_up() {
    let mut g = pod(2);
    let homunculus = g.add_card_to_battlefield(0, catalog::curious_homunculus());
    for card in [catalog::murder(), catalog::murder(), catalog::agonizing_syphon()] {
        g.add_card_to_graveyard(0, card);
    }
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Draw {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(pt(&g, homunculus), (3, 4));
}

/// Brand of Ill Omen binds the enchanted creature's controller, not its own.
#[test]
fn brand_of_ill_omen_locks_the_enchanted_controller() {
    for (host_seat, locked) in [(0, true), (1, false)] {
        let mut g = pod(2);
        let host = g.add_card_to_battlefield(host_seat, catalog::grizzly_bears());
        let brand = g.add_card_to_hand(0, catalog::brand_of_ill_omen());
        cast(&mut g, brand, Some(Target::Permanent(host)));
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        flood(&mut g, 0);
        let cast = g.perform_action(GameAction::CastSpell {
            card_id: bear,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        });
        assert_eq!(cast.is_err(), locked, "Brand on seat {host_seat}'s creature");
    }
}

/// CR 301.5c — an Equipment that is also a creature can't equip: a danced
/// Dancing Sword (cmdr_01jy_a's) is refused a new host.
#[test]
fn cr_301_5c_a_creature_equipment_cant_equip() {
    let mut g = pod(2);
    let sword = g.add_card_to_battlefield(0, catalog::dancing_sword());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: sword, target: bear }).expect("equip");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (4, 3));
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(bear)));
    assert_eq!(pt(&g, sword), (2, 1));
    assert!(g.computed_permanent(sword).unwrap().keywords().contains(&Keyword::Flying));
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    assert!(g.perform_action(GameAction::Equip { equipment: sword, target: other }).is_err());
}

/// Harvest Pyre's X is the number of graveyard cards exiled for it.
#[test]
fn harvest_pyre_burns_by_the_graveyard() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::island());
    }
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pyre = g.add_card_to_hand(0, catalog::harvest_pyre());
    cast_x(&mut g, pyre, Some(Target::Permanent(bear)), Some(2));
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].graveyard.len(), 2, "two Islands exiled, the Pyre itself added");
}

/// Ore-Rich Stalactite crafts with four red instants into Cosmium Catalyst,
/// which casts one of them for free.
#[test]
fn ore_rich_stalactite_crafts_and_recasts() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let rock = g.add_card_to_battlefield(0, catalog::ore_rich_stalactite());
    for _ in 0..4 {
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
    }
    activate(&mut g, rock, 1).expect("craft");
    let rock = g.battlefield.iter().find(|c| c.definition.name == "Cosmium Catalyst").expect("crafted").id;
    g.clear_sickness(rock);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life = g.players[1].life;
    activate(&mut g, rock, 0).expect("catalyst");
    assert_eq!(g.players[1].life, life - 3, "a crafted Bolt cast free at the opponent");
}

/// Mystic Skull transforms for {5} into a 5/6 Construct.
#[test]
fn mystic_skull_wakes_up() {
    let mut g = pod(2);
    let skull = g.add_card_to_battlefield(0, catalog::mystic_skull());
    activate(&mut g, skull, 1).expect("transform");
    assert_eq!(pt(&g, skull), (5, 6));
}

/// Dimir Strandcatcher draws at end step off three non-battlefield graveyard
/// arrivals: resolved spells count, a dead creature doesn't.
#[test]
fn dimir_strandcatcher_counts_only_non_battlefield_arrivals() {
    for (bolts, kill_bear, draws) in [(3, false, true), (2, true, false)] {
        let mut g = pod(2);
        g.add_card_to_battlefield(0, catalog::dimir_strandcatcher());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        for i in 0..bolts {
            let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
            let to = if kill_bear && i == 0 { Target::Permanent(bear) } else { Target::Player(1) };
            cast(&mut g, bolt, Some(to));
        }
        let hand = g.players[0].hand.len();
        g.step = TurnStep::PostCombatMain;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::End {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + usize::from(draws), "{bolts} bolts, bear killed {kill_bear}");
    }
}

/// Nevermaker tops a nonland permanent when it leaves the battlefield.
#[test]
fn nevermaker_tops_a_permanent_as_it_leaves() {
    let mut g = pod(2);
    let maker = g.add_card_to_battlefield(0, catalog::nevermaker());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(maker)));
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[1].library.first().map(|c| c.id), Some(bear), "on top");
}

/// CR 101.2 — Memory Vessel: everyone plays from their exiled seven, not
/// their hand, until the Vessel's controller's next turn begins.
#[test]
fn memory_vessel_swaps_hands_for_the_top_seven() {
    let mut g = pod(3);
    let vessel = g.add_card_to_battlefield(0, catalog::memory_vessel());
    let forest = g.add_card_to_hand(0, catalog::forest());
    activate(&mut g, vessel, 0).expect("activate");
    assert!(g.players.iter().all(|p| p.library.len() == 3), "seven exiled from each library");
    g.priority.player_with_priority = 0;
    assert!(
        matches!(g.perform_action(GameAction::PlayLand(forest)), Err(GameError::SpellNameLocked)),
        "the hand is locked"
    );
    let island = g.exile.iter().find(|c| c.owner == 0 && c.definition.name == "Island").unwrap().id;
    g.perform_action(GameAction::PlayLand(island)).expect("an exiled card plays");
    let pass_until = |g: &mut GameState, seat: usize| {
        while g.active_player_idx != seat {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
    };
    pass_until(&mut g, 1);
    assert_eq!(g.players[1].hand_locked_until_turn_of, Some(0), "still locked on seat 1's turn");
    assert!(g.exile.iter().filter(|c| c.owner == 1).all(|c| c.may_play_until.is_some()));
    pass_until(&mut g, 0);
    assert!(g.players.iter().all(|p| p.hand_locked_until_turn_of.is_none()));
    assert!(g.exile.iter().all(|c| c.may_play_until.is_none()), "the window closed");
}

/// Invasion of New Phyrexia makes X Knights; Teferi's −3 taps creatures and
/// shuffles away an opposing permanent of mana value up to the count.
#[test]
fn invasion_of_new_phyrexia_and_teferis_shuffle() {
    let mut g = pod(2);
    let invasion = g.add_card_to_hand(0, catalog::invasion_of_new_phyrexia());
    cast_x(&mut g, invasion, None, Some(2));
    let knights = || g.battlefield.iter().filter(|c| c.definition.name == "Knight" && c.controller == 0).count();
    assert_eq!(knights(), 2);
    let mut g = pod(2);
    let teferi = *catalog::invasion_of_new_phyrexia().back_face.unwrap();
    let teferi = g.add_card_to_battlefield(0, teferi);
    let mine: Vec<_> = (0..2).map(|_| g.add_card_to_battlefield(0, catalog::grizzly_bears())).collect();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: teferi, ability_index: 2, target: None, x_value: None })
        .expect("-3");
    drain_stack(&mut g);
    assert!(mine.iter().all(|&b| g.battlefield_find(b).unwrap().tapped), "both tapped");
    assert!(g.battlefield_find(theirs).is_none(), "an MV-2 permanent shuffled away");
    assert!(g.players[1].library.iter().any(|c| c.id == theirs));
    // One creature tapped: X = 1 can't reach a mana value 2 permanent.
    let mut g = pod(2);
    let teferi = g.add_card_to_battlefield(0, *catalog::invasion_of_new_phyrexia().back_face.unwrap());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: teferi, ability_index: 2, target: None, x_value: None })
        .expect("-3");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_some(), "X = 1 is short of mana value 2");
}

/// Long Rest returns X cards of distinct mana values; eight or more resets
/// life to the starting total.
#[test]
fn long_rest_returns_distinct_values_and_resets_life() {
    let mut g = pod(2);
    let pool = [
        catalog::forest(), catalog::llanowar_elves(), catalog::grizzly_bears(), catalog::hill_giant(),
        catalog::serra_angel(), catalog::murder(), catalog::lightning_bolt(), catalog::sol_ring(),
    ];
    for card in pool {
        g.add_card_to_graveyard(0, card);
    }
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].life = 5;
    let rest = g.add_card_to_hand(0, catalog::long_rest());
    cast_x(&mut g, rest, None, Some(3));
    assert_eq!(g.players[0].hand.len(), 3);
    let mut mvs: Vec<u32> = g.players[0].hand.iter().map(|c| c.definition.cost.cmc()).collect();
    mvs.sort_unstable();
    mvs.dedup();
    assert_eq!(mvs.len(), 3, "different mana values");
    assert_eq!(g.players[0].life, 5);
}

/// Halls of Mist keeps a creature that attacked last turn from attacking.
#[test]
fn halls_of_mist_rests_last_turns_attackers() {
    for attacked_last_turn in [false, true] {
        let mut g = pod(2);
        g.add_card_to_battlefield(1, catalog::halls_of_mist());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
        g.battlefield_find_mut(bear).unwrap().attacked_last_turn = attacked_last_turn;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        let attack = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bear,
            target: AttackTarget::Player(1),
        }]));
        assert_eq!(attack.is_err(), attacked_last_turn);
    }
}

/// Three Bowls of Porridge never repeats a bowl: three activations, three
/// different modes.
#[test]
fn three_bowls_of_porridge_tries_each_bowl_once() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let bowls = g.add_card_to_battlefield(0, catalog::three_bowls_of_porridge());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    activate(&mut g, bowls, 0).expect("bowl one");
    assert_eq!(g.battlefield_find(giant).unwrap().damage, 2);
    g.battlefield_find_mut(bowls).unwrap().tapped = false;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    activate(&mut g, bowls, 0).expect("bowl two");
    assert!(g.battlefield_find(giant).unwrap().tapped, "mode 0 is spent: the tap bowl");
    g.battlefield_find_mut(bowls).unwrap().tapped = false;
    let life = g.players[0].life;
    activate(&mut g, bowls, 0).expect("bowl three");
    assert!(g.battlefield_find(bowls).is_none());
    assert_eq!(g.players[0].life, life + 3);
}

/// Barroom Brawl only reaches the opponent to your left (seat 1 of three, and
/// past a seat that has left the game).
#[test]
fn barroom_brawl_picks_on_the_left() {
    let mut g = pod(3);
    let mine = g.add_card_to_battlefield(0, catalog::hill_giant());
    let left = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let right = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let brawl = g.add_card_to_hand(0, catalog::barroom_brawl());
    flood(&mut g, 0);
    let cast_at = |g: &mut GameState, them| {
        g.perform_action(GameAction::CastSpell {
            card_id: brawl,
            target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Permanent(them)],
            mode: None,
            x_value: None,
        })
    };
    assert!(cast_at(&mut g, right).is_err(), "seat 2 is to the right");
    cast_at(&mut g, left).expect("seat 1 is to the left");
    drain_stack(&mut g);
    assert!(g.battlefield_find(left).is_none(), "the Giant wins the fight");
    assert_eq!(g.opponent_to_left_of(0), Some(1));
    g.players[1].eliminated = true;
    assert_eq!(g.opponent_to_left_of(0), Some(2), "the next living opponent");
}

/// Oona's Prowler's shrink is anyone's to activate.
#[test]
fn oonas_prowler_can_be_shrunk_by_an_opponent() {
    let mut g = pod(2);
    let prowler = g.add_card_to_battlefield(0, catalog::oonas_prowler());
    g.add_card_to_hand(1, catalog::island());
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: prowler,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("an opponent activates");
    drain_stack(&mut g);
    assert_eq!(pt(&g, prowler), (1, 1));
    assert!(g.players[1].hand.is_empty(), "the activating opponent discarded");
}

/// Nahiri's Warcrafting digs as deep as its excess damage.
#[test]
fn nahiris_warcrafting_digs_by_the_excess() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::nahiris_warcrafting());
    let library = g.players[0].library.len();
    cast(&mut g, spell, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].library.len(), library - 1, "one of the three exiled");
    assert!(g.exile.iter().any(|c| c.owner == 0 && c.may_play_until.is_some()));
}

/// Banish into Fable copies itself once per artifact / enchantment clause.
#[test]
fn banish_into_fable_copies_for_artifact_and_enchantment() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::sol_ring());
    g.add_card_to_battlefield(0, catalog::glorious_anthem());
    let targets: Vec<_> = (0..3).map(|_| g.add_card_to_battlefield(1, catalog::grizzly_bears())).collect();
    let banish = g.add_card_to_hand(0, catalog::banish_into_fable());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: banish,
        target: Some(Target::Permanent(targets[0])),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    g.resolve_top_of_stack().expect("the cast trigger");
    assert_eq!(g.stack.len(), 3, "the spell and two copies");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Knight"));
}

/// Gonti's Machinations banks one energy for the turn's first life loss, then
/// cashes two in for a drain.
#[test]
fn gontis_machinations_banks_the_first_loss() {
    let mut g = pod(2);
    let gonti = g.add_card_to_battlefield(0, catalog::gontis_machinations());
    for _ in 0..2 {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast(&mut g, bolt, Some(Target::Player(0)));
    }
    assert_eq!(g.players[0].energy, 1, "only the first loss");
    g.players[0].energy = 2;
    let (mine, theirs) = (g.players[0].life, g.players[1].life);
    activate(&mut g, gonti, 0).expect("cash in");
    assert_eq!((g.players[0].life, g.players[1].life), (mine + 3, theirs - 3));
    assert!(g.battlefield_find(gonti).is_none());
}

/// Die Young pays its two energy into -2/-2.
#[test]
fn die_young_spends_energy_on_the_shrink() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::die_young());
    cast(&mut g, spell, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].energy, 0);
}

/// Quicksmith Spy teaches an artifact to draw while the Spy stays.
#[test]
fn quicksmith_spy_grants_a_draw_ability() {
    let mut g = pod(2);
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let spy = g.add_card_to_hand(0, catalog::quicksmith_spy());
    cast(&mut g, spy, None);
    let hand = g.players[0].hand.len();
    activate(&mut g, ring, 1).expect("the granted draw");
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Shape Anew trades an artifact for the next artifact in its controller's
/// library.
#[test]
fn shape_anew_rerolls_an_artifact() {
    let mut g = pod(2);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let stone = g.add_card_to_library(1, catalog::mind_stone());
    let spell = g.add_card_to_hand(0, catalog::shape_anew());
    cast(&mut g, spell, Some(Target::Permanent(ring)));
    assert!(g.battlefield_find(ring).is_none());
    assert_eq!(g.battlefield_find(stone).map(|c| c.controller), Some(1));
}

/// Sharpened Pitchfork's +1/+1 is for Humans only.
#[test]
fn sharpened_pitchfork_favors_humans() {
    for (bearer, want) in [(catalog::elite_vanguard(), (3, 2)), (catalog::grizzly_bears(), (2, 2))] {
        let mut g = pod(2);
        let fork = g.add_card_to_battlefield(0, catalog::sharpened_pitchfork());
        let c = g.add_card_to_battlefield(0, bearer);
        flood(&mut g, 0);
        g.perform_action(GameAction::Equip { equipment: fork, target: c }).expect("equip");
        drain_stack(&mut g);
        assert_eq!(pt(&g, c), want);
        assert!(g.computed_permanent(c).unwrap().keywords().contains(&Keyword::FirstStrike));
    }
}

/// Eldritch Pact scales with the target player's graveyard.
#[test]
fn eldritch_pact_reads_their_graveyard() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_graveyard(1, catalog::island());
    }
    let (hand, life) = (g.players[1].hand.len(), g.players[1].life);
    let pact = g.add_card_to_hand(0, catalog::eldritch_pact());
    cast(&mut g, pact, Some(Target::Player(1)));
    assert_eq!((g.players[1].hand.len(), g.players[1].life), (hand + 3, life - 3));
}

/// Spirit Away steals a creature and makes it a 4/4 flyer.
#[test]
fn spirit_away_steals_and_lifts() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::spirit_away());
    cast(&mut g, aura, Some(Target::Permanent(bear)));
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0);
    assert_eq!(pt(&g, bear), (4, 4));
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Flying));
}

/// CR 603.8 — Seasinger is sacrificed the moment you control no Islands, and
/// steals only from a player who controls one.
#[test]
fn seasinger_needs_islands_on_both_sides() {
    let mut g = pod(3);
    let isle = g.add_card_to_battlefield(0, catalog::island());
    let singer = g.add_card_to_battlefield(0, catalog::seasinger());
    g.add_card_to_battlefield(1, catalog::island());
    let wet = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dry = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    g.clear_sickness(singer);
    g.priority.player_with_priority = 0;
    let steal = |g: &mut GameState, t| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: singer,
            ability_index: 0,
            target: Some(Target::Permanent(t)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(steal(&mut g, dry).is_err(), "seat 2 has no Island");
    steal(&mut g, wet).expect("seat 1 does");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(wet).unwrap().controller, 0);
    let murder = g.add_card_to_hand(0, catalog::stone_rain());
    cast(&mut g, murder, Some(Target::Permanent(isle)));
    assert!(g.battlefield_find(singer).is_none(), "no Islands: sacrificed");
}

/// Chandra, Pyrogenius's +2 burns each opponent.
#[test]
fn chandra_pyrogenius_burns_the_table() {
    let mut g = pod(3);
    let chandra = g.add_card_to_battlefield(0, catalog::chandra_pyrogenius());
    let life = g.players[1].life;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: chandra, ability_index: 0, target: None, x_value: None })
        .expect("+2");
    drain_stack(&mut g);
    assert_eq!((g.players[1].life, g.players[2].life), (life - 2, life - 2));
}

/// Hunter's Mark costs {G} when it bites a blue permanent you don't control.
#[test]
fn hunters_mark_is_cheap_against_blue() {
    let mut g = pod(2);
    let mine = g.add_card_to_battlefield(0, catalog::hill_giant());
    let drake = g.add_card_to_battlefield(1, catalog::wind_drake());
    let mark = g.add_card_to_hand(0, catalog::hunters_mark());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: mark,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(drake)],
        mode: None,
        x_value: None,
    })
    .expect("{G} is enough");
    drain_stack(&mut g);
    assert!(g.battlefield_find(drake).is_none());
}

/// Graceful Takedown adds every enchanted creature's power to the other
/// creature's.
#[test]
fn graceful_takedown_gangs_up() {
    let mut g = pod(2);
    let enchanted = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let holy = g.add_card_to_hand(0, catalog::holy_strength());
    cast(&mut g, holy, Some(Target::Permanent(enchanted)));
    let giant = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let takedown = g.add_card_to_hand(0, catalog::graceful_takedown());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: takedown,
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![Target::Permanent(other)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none(), "3 from the enchanted Bears + 2 from the other kill a 6/4");
}

/// Quicksmith Rebel arms an artifact with a 2-damage ping.
#[test]
fn quicksmith_rebel_arms_an_artifact() {
    let mut g = pod(2);
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let rebel = g.add_card_to_hand(0, catalog::quicksmith_rebel());
    cast(&mut g, rebel, None);
    let life = g.players[1].life;
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ring,
        ability_index: 1,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2);
}

/// Make a Wish returns two graveyard cards; Expose Evil taps two and
/// investigates.
#[test]
fn make_a_wish_and_expose_evil() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::island());
    }
    let wish = g.add_card_to_hand(0, catalog::make_a_wish());
    cast(&mut g, wish, None);
    assert_eq!((g.players[0].hand.len(), g.players[0].graveyard.len()), (2, 2), "two back, the Wish in the yard");
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let expose = g.add_card_to_hand(0, catalog::expose_evil());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: expose,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Clue"));
}

/// Cauldron's Gift reanimates a creature with an extra +1/+1 counter.
#[test]
fn cauldrons_gift_returns_a_creature_bigger() {
    let mut g = pod(2);
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    let gift = g.add_card_to_hand(0, catalog::cauldrons_gift());
    cast(&mut g, gift, None);
    assert_eq!(pt(&g, giant), (4, 4));
}

/// Rusted Relic is a 5/5 creature only under metalcraft.
#[test]
fn rusted_relic_wakes_with_metalcraft() {
    let mut g = pod(2);
    let relic = g.add_card_to_battlefield(0, catalog::rusted_relic());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    assert!(!g.computed_permanent(relic).unwrap().card_types().contains(&CardType::Creature));
    g.add_card_to_battlefield(0, catalog::mind_stone());
    assert!(g.computed_permanent(relic).unwrap().card_types().contains(&CardType::Creature));
    assert_eq!(pt(&g, relic), (5, 5));
}

/// Dispense Justice takes two attackers with metalcraft.
#[test]
fn dispense_justice_doubles_with_metalcraft() {
    for (artifacts, gone) in [(0, 1), (3, 2)] {
        let mut g = pod(2);
        for _ in 0..artifacts {
            g.add_card_to_battlefield(0, catalog::sol_ring());
        }
        let attackers: Vec<_> = (0..3).map(|_| g.add_card_to_battlefield(1, catalog::grizzly_bears())).collect();
        for &a in &attackers {
            g.clear_sickness(a);
        }
        g.active_player_idx = 1;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::DeclareAttackers(
            attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(0) }).collect(),
        ))
        .expect("attack");
        let justice = g.add_card_to_hand(0, catalog::dispense_justice());
        flood(&mut g, 0);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: justice,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        assert_eq!(attackers.iter().filter(|&&a| g.battlefield_find(a).is_none()).count(), gone, "{artifacts} artifacts");
    }
}

/// Magma Pummeler shrugs damage off its counters and throws it back.
#[test]
fn magma_pummeler_returns_the_damage() {
    let mut g = pod(2);
    let pummeler = g.add_card_to_hand(0, catalog::magma_pummeler());
    cast_x(&mut g, pummeler, None, Some(4));
    assert_eq!(pt(&g, pummeler), (4, 4));
    let life = g.players[1].life;
    g.priority.player_with_priority = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(pummeler)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(pt(&g, pummeler), (1, 1), "three counters off, no damage marked");
    assert_eq!(g.battlefield_find(pummeler).unwrap().damage, 0);
    assert!(g.players[1].life < life || g.battlefield.iter().any(|c| c.controller == 1 && c.damage > 0), "three back at something");
}

/// Feast of Succession shrinks everything and crowns you.
#[test]
fn feast_of_succession_crowns_you() {
    let mut g = pod(2);
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let feast = g.add_card_to_hand(0, catalog::feast_of_succession());
    cast(&mut g, feast, None);
    assert!(g.battlefield_find(giant).is_none());
    assert_eq!(g.monarch, Some(0));
}

/// Nature's Embrace pumps a creature, or turns a land into a two-mana land.
#[test]
fn natures_embrace_reads_its_host() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let embrace = g.add_card_to_hand(0, catalog::natures_embrace());
    cast(&mut g, embrace, Some(Target::Permanent(bear)));
    assert_eq!(pt(&g, bear), (4, 4));
    let mut g = pod(2);
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let embrace = g.add_card_to_hand(0, catalog::natures_embrace());
    cast(&mut g, embrace, Some(Target::Permanent(forest)));
    g.players[0].mana_pool = Default::default();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: forest,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("the granted two-mana ability");
    assert_eq!(g.players[0].mana_pool.total(), 2);
}

/// Invasion of Muraganda grows a creature and has it fight.
#[test]
fn invasion_of_muraganda_grows_and_fights() {
    let mut g = pod(2);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let invasion = g.add_card_to_hand(0, catalog::invasion_of_muraganda());
    cast(&mut g, invasion, None);
    assert_eq!(pt(&g, mine), (3, 3));
    assert!(g.battlefield_find(theirs).is_none(), "a 3/3 beats a 2/2");
}

/// The Crowd Goes Wild supports up to X creatures, then tramples everything
/// with a +1/+1 counter.
#[test]
fn the_crowd_goes_wild_supports_x() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let crowd = g.add_card_to_hand(0, catalog::the_crowd_goes_wild());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: crowd,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!((pt(&g, a), pt(&g, b), pt(&g, c)), ((3, 3), (3, 3), (2, 2)));
    let tramples = |id| g.computed_permanent(id).unwrap().keywords().contains(&Keyword::Trample);
    assert!(tramples(a) && tramples(b) && !tramples(c));
}

/// Spellweaver Duo bounces a tapped creature as it enters.
#[test]
fn spellweaver_duo_bounces_a_tapped_creature() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let duo = g.add_card_to_hand(0, catalog::spellweaver_duo());
    cast(&mut g, duo, None);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
}

/// Jade Seedstones spreads three counters over your creatures.
#[test]
fn jade_seedstones_spreads_three_counters() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let stones = g.add_card_to_hand(0, catalog::jade_seedstones());
    cast(&mut g, stones, None);
    assert_eq!(pt(&g, a), (5, 5), "one creature takes all three");
}

/// Sacellum Godspeaker taps for {G} per fatty in hand.
#[test]
fn sacellum_godspeaker_counts_the_fatties() {
    let mut g = pod(2);
    let speaker = g.add_card_to_battlefield(0, catalog::sacellum_godspeaker());
    g.clear_sickness(speaker);
    g.add_card_to_hand(0, catalog::craw_wurm());
    g.add_card_to_hand(0, catalog::craw_wurm());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: speaker,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap");
    assert_eq!(g.players[0].mana_pool.total(), 2);
}

/// Dredging Claw jumps onto a creature returning from your graveyard.
#[test]
fn dredging_claw_follows_the_dead() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let claw = g.add_card_to_battlefield(0, catalog::dredging_claw());
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let zombify = g.add_card_to_hand(0, catalog::zombify());
    cast(&mut g, zombify, Some(Target::Permanent(giant)));
    assert!(g.battlefield_find(giant).is_some());
    assert_eq!(g.battlefield_find(claw).unwrap().attached_to, Some(giant));
}

/// Centaur Peacemaker gives every player 4 life.
#[test]
fn centaur_peacemaker_feeds_the_table() {
    let mut g = pod(3);
    let before: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    let centaur = g.add_card_to_hand(0, catalog::centaur_peacemaker());
    cast(&mut g, centaur, None);
    assert!(g.players.iter().zip(&before).all(|(p, b)| p.life == b + 4));
}

/// Academy Drake enters with two counters only when kicked.
#[test]
fn academy_drake_kicked_grows() {
    let mut g = pod(2);
    let drake = g.add_card_to_hand(0, catalog::academy_drake());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: drake,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("kicked");
    drain_stack(&mut g);
    assert_eq!(pt(&g, drake), (4, 4));
}

/// Halo Scarab trades itself from the graveyard for a Treasure.
#[test]
fn halo_scarab_leaves_a_treasure() {
    let mut g = pod(2);
    let scarab = g.add_card_to_graveyard(0, catalog::halo_scarab());
    activate(&mut g, scarab, 0).expect("from the graveyard");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"));
    assert!(g.exile.iter().any(|c| c.id == scarab));
}

/// Sea Hag's arrival takes 4 power off each opposing creature.
#[test]
fn sea_hag_softens_the_opposition() {
    let mut g = pod(2);
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let mine = g.add_card_to_battlefield(0, catalog::hill_giant());
    let hag = g.add_card_to_hand(0, catalog::sea_hag());
    cast(&mut g, hag, None);
    assert_eq!((pt(&g, giant), pt(&g, mine)), ((-1, 3), (3, 3)));
}

/// Briber's Purse spends a gem counter to keep a creature home.
#[test]
fn bribers_purse_spends_gems() {
    let mut g = pod(2);
    let purse = g.add_card_to_hand(0, catalog::bribers_purse());
    cast_x(&mut g, purse, None, Some(2));
    assert_eq!(g.battlefield_find(purse).unwrap().counter_count(CounterType::Gem), 2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: purse,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("bribe");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(purse).unwrap().counter_count(CounterType::Gem), 1);
    let bear = g.computed_permanent(bear).unwrap();
    assert!(bear.keywords().contains(&Keyword::CantAttack) && bear.keywords().contains(&Keyword::CantBlock));
}

/// Ent-Draught Basin's X must equal the target's power.
#[test]
fn ent_draught_basin_matches_power_to_x() {
    let mut g = pod(2);
    let basin = g.add_card_to_battlefield(0, catalog::ent_draught_basin());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    let feed = |g: &mut GameState, t| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: basin,
            ability_index: 0,
            target: Some(Target::Permanent(t)),
            additional_targets: vec![],
            x_value: Some(2),
            mode: None,
        })
    };
    assert!(feed(&mut g, giant).is_err(), "power 3 isn't X = 2");
    feed(&mut g, bear).expect("power 2 is");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (3, 3));
}

/// Lat-Nam Adept grows on the second draw of a turn, not the first.
#[test]
fn lat_nam_adept_grows_on_the_second_draw() {
    let mut g = pod(2);
    let adept = g.add_card_to_battlefield(0, catalog::lat_nam_adept());
    let study = g.add_card_to_hand(0, catalog::divination());
    cast(&mut g, study, None);
    assert_eq!(pt(&g, adept), (4, 4));
}

/// Tidy Conclusion gains a life per artifact; Resourceful Return draws with
/// an artifact out.
#[test]
fn tidy_conclusion_and_resourceful_return_count_artifacts() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::sol_ring());
    g.add_card_to_battlefield(0, catalog::mind_stone());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[0].life;
    let tidy = g.add_card_to_hand(0, catalog::tidy_conclusion());
    cast(&mut g, tidy, Some(Target::Permanent(bear)));
    assert_eq!(g.players[0].life, life + 2);
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    let ret = g.add_card_to_hand(0, catalog::resourceful_return());
    let hand = g.players[0].hand.len();
    cast(&mut g, ret, Some(Target::Permanent(giant)));
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "the Giant back plus a card");
}

/// Darling of the Masses makes a Citizen per attack and pumps the others.
#[test]
fn darling_of_the_masses_rallies_citizens() {
    let mut g = pod(2);
    let darling = g.add_card_to_battlefield(0, catalog::darling_of_the_masses());
    attack_with(&mut g, &[darling]);
    let citizen = g.battlefield.iter().find(|c| c.definition.name == "Citizen").expect("a Citizen").id;
    assert_eq!(pt(&g, citizen), (2, 1));
    assert_eq!(pt(&g, darling), (2, 4), "only the others");
}

/// Requiting Hex gains 2 life only when blighted.
#[test]
fn requiting_hex_rewards_the_blight() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::hill_giant());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hex = g.add_card_to_hand(0, catalog::requiting_hex());
    let life = g.players[0].life;
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: hex,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("blighted");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].life, life + 2);
}

/// Wight turns what it kills into a tapped Zombie and exiles the corpse.
#[test]
fn wight_drains_its_victims() {
    let mut g = pod(2);
    let wight = g.add_card_to_battlefield(0, catalog::wight());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(wight);
    let fight = g.add_card_to_hand(0, catalog::aggressive_instinct());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: fight,
        target: Some(Target::Permanent(wight)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "the dead Bears exiled");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Zombie" && c.controller == 0 && c.tapped));
}

/// Lich Lord of Unx drains and mills by the Zombie count.
#[test]
fn lich_lord_of_unx_counts_zombies() {
    let mut g = pod(2);
    let lord = g.add_card_to_battlefield(0, catalog::lich_lord_of_unx());
    g.add_card_to_battlefield(0, catalog::wight());
    let (life, library) = (g.players[1].life, g.players[1].library.len());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: lord,
        ability_index: 1,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("drain");
    drain_stack(&mut g);
    assert_eq!((g.players[1].life, g.players[1].library.len()), (life - 2, library - 2));
}

/// Stitchwing Skaab climbs back from the graveyard for two discards.
#[test]
fn stitchwing_skaab_returns_tapped() {
    let mut g = pod(2);
    let skaab = g.add_card_to_graveyard(0, catalog::stitchwing_skaab());
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    activate(&mut g, skaab, 0).expect("return");
    assert!(g.battlefield_find(skaab).is_some_and(|c| c.tapped));
    assert!(g.players[0].hand.is_empty());
}

/// Cloudseeder's Cloud Sprite blocks only flyers.
#[test]
fn cloudseeder_makes_a_cloud_sprite() {
    let mut g = pod(2);
    let seeder = g.add_card_to_battlefield(0, catalog::cloudseeder());
    g.clear_sickness(seeder);
    g.add_card_to_hand(0, catalog::island());
    activate(&mut g, seeder, 0).expect("seed");
    let sprite = g.battlefield.iter().find(|c| c.definition.name == "Cloud Sprite").expect("token");
    assert!(sprite.definition.keywords.contains(&Keyword::CanBlockOnlyFlying));
}

/// Even the Odds needs fewer creatures than EACH opponent, not their total.
#[test]
fn even_the_odds_checks_each_opponent() {
    for (seat2_creatures, castable) in [(2, true), (0, false)] {
        let mut g = pod(3);
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        for _ in 0..2 {
            g.add_card_to_battlefield(1, catalog::grizzly_bears());
        }
        for _ in 0..seat2_creatures {
            g.add_card_to_battlefield(2, catalog::grizzly_bears());
        }
        let odds = g.add_card_to_hand(0, catalog::even_the_odds());
        flood(&mut g, 0);
        let cast = g.perform_action(GameAction::CastSpell {
            card_id: odds,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        });
        assert_eq!(cast.is_ok(), castable, "seat 2 has {seat2_creatures}");
    }
}

/// For the Family doubles up with four creatures.
#[test]
fn for_the_family_scales_with_the_family() {
    for (extra, want) in [(0, (4, 4)), (3, (6, 6))] {
        let mut g = pod(2);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        for _ in 0..extra {
            g.add_card_to_battlefield(0, catalog::grizzly_bears());
        }
        let spell = g.add_card_to_hand(0, catalog::for_the_family());
        cast(&mut g, spell, Some(Target::Permanent(bear)));
        assert_eq!(pt(&g, bear), want);
    }
}

/// Start makes two Warriors; Finish, from the graveyard, trades one for a kill.
#[test]
fn start_then_finish() {
    let mut g = pod(2);
    let card = g.add_card_to_hand(0, catalog::start_finish());
    cast(&mut g, card, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Warrior").count(), 2);
    let victim = g.add_card_to_battlefield(1, catalog::hill_giant());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastAftermath { card_id: card, target: Some(Target::Permanent(victim)), additional_targets: vec![], mode: None, x_value: None })
        .expect("Finish");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none());
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Warrior").count(), 1);
    assert!(g.exile.iter().any(|c| c.id == card));
}

/// Harsh Sustenance and Spell Snuff's fateful hour.
#[test]
fn harsh_sustenance_counts_creatures() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    let (mine, theirs) = (g.players[0].life, g.players[1].life);
    let harsh = g.add_card_to_hand(0, catalog::harsh_sustenance());
    cast(&mut g, harsh, Some(Target::Player(1)));
    assert_eq!((g.players[0].life, g.players[1].life), (mine + 3, theirs - 3));
}

/// Invasion of Tolvada brings back a permanent card; Liminal Hold holds one
/// until it leaves.
#[test]
fn invasion_of_tolvada_and_liminal_hold() {
    let mut g = pod(2);
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    let invasion = g.add_card_to_hand(0, catalog::invasion_of_tolvada());
    cast(&mut g, invasion, None);
    assert!(g.battlefield_find(ring).is_some());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[0].life;
    let hold = g.add_card_to_hand(0, catalog::liminal_hold());
    cast(&mut g, hold, None);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].life, life + 2);
}

/// Air Nomad Student grows only on turns it stayed home.
#[test]
fn air_nomad_student_studies_at_home() {
    let mut g = pod(2);
    let student = g.add_card_to_battlefield(0, catalog::air_nomad_student());
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(pt(&g, student), (3, 3));
}

/// CR 800.4 — Timely Reinforcements' "fewer creatures than an opponent" is
/// one opponent's board, not the sum: two one-creature opponents don't
/// outnumber a one-creature caster.
#[test]
fn cr_800_4_timely_reinforcements_reads_one_opponent() {
    for (seat1, tokens) in [(1, 0), (2, 3)] {
        let mut g = pod(3);
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        for _ in 0..seat1 {
            g.add_card_to_battlefield(1, catalog::grizzly_bears());
        }
        g.add_card_to_battlefield(2, catalog::grizzly_bears());
        let timely = g.add_card_to_hand(0, catalog::timely_reinforcements());
        cast(&mut g, timely, None);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Soldier").count(), tokens, "seat 1 has {seat1}");
    }
}

/// Tezzeret, Master of Metal's −8 takes every artifact and creature the
/// target opponent controls — and nothing of the other opponent's.
#[test]
fn tezzeret_master_of_metal_ultimate_steals_one_board() {
    let mut g = pod(3);
    let tez = g.add_card_to_battlefield(0, catalog::tezzeret_master_of_metal());
    g.battlefield_find_mut(tez).unwrap().add_counters(CounterType::Loyalty, 5);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: tez,
        ability_index: 2,
        target: Some(Target::Player(1)),
        x_value: None,
    })
    .expect("-8");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ring).unwrap().controller, 0);
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0);
    assert_eq!(g.battlefield_find(other).unwrap().controller, 2);
}

/// Suncleanser strips a creature's counters and keeps new ones off it.
#[test]
fn suncleanser_scrubs_and_locks() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    let cleanser = g.add_card_to_hand(0, catalog::suncleanser());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: cleanser,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0), DecisionAnswer::Target(Target::Permanent(bear))]));
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (2, 2), "counters gone");
    assert!(g.counters_locked_on(bear));
}

/// Profane Procession turns into Tomb of the Dusk Rose on its third exile.
#[test]
fn profane_procession_becomes_the_tomb() {
    let mut g = pod(2);
    let proc_ = g.add_card_to_battlefield(0, catalog::profane_procession());
    for _ in 0..3 {
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        flood(&mut g, 0);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: proc_,
            ability_index: 0,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("exile");
        drain_stack(&mut g);
    }
    let card = g.battlefield_find(proc_).unwrap();
    assert!(card.transformed, "Tomb of the Dusk Rose");
}

/// Salvage Slasher grows with the artifact cards in your graveyard.
#[test]
fn salvage_slasher_counts_scrap() {
    let mut g = pod(2);
    let slasher = g.add_card_to_battlefield(0, catalog::salvage_slasher());
    g.add_card_to_graveyard(0, catalog::sol_ring());
    g.add_card_to_graveyard(0, catalog::mind_stone());
    assert_eq!(pt(&g, slasher), (3, 1));
}

/// Nimbus Champion bounces only what its Warriors can match; Frostfist
/// Strider stuns on arrival.
#[test]
fn nimbus_champion_and_frostfist_strider() {
    let mut g = pod(2);
    let champ = g.add_card_to_battlefield(0, catalog::nimbus_champion());
    let small = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(small))]));
    attack_with(&mut g, &[champ]);
    assert!(g.players[1].hand.iter().any(|c| c.id == small), "power 1 <= one Warrior");
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let strider = g.add_card_to_hand(0, catalog::frostfist_strider());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    g.step = TurnStep::PostCombatMain;
    cast(&mut g, strider, None);
    let bear = g.battlefield_find(bear).unwrap();
    assert!(bear.tapped && bear.counter_count(CounterType::Stun) == 1);
}

/// Sacred Boon turns prevented damage into +0/+1 counters.
#[test]
fn sacred_boon_banks_toughness() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let boon = g.add_card_to_hand(0, catalog::sacred_boon());
    cast(&mut g, boon, Some(Target::Permanent(bear)));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(bear)));
    assert_eq!(pt(&g, bear), (2, 5), "three prevented, three +0/+1 counters");
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0);
}

/// Merchant's Dockhand digs as deep as the artifacts it taps.
#[test]
fn merchants_dockhand_taps_artifacts_to_dig() {
    let mut g = pod(2);
    let hand = g.add_card_to_battlefield(0, catalog::merchants_dockhand());
    g.clear_sickness(hand);
    let rings: Vec<_> = (0..2).map(|_| g.add_card_to_battlefield(0, catalog::mind_stone())).collect();
    let cards = g.players[0].hand.len();
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: hand,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: Some(2),
        mode: None,
    })
    .expect("dig");
    drain_stack(&mut g);
    assert!(rings.iter().all(|&r| g.battlefield_find(r).unwrap().tapped));
    assert_eq!(g.players[0].hand.len(), cards + 1);
}

/// Idol of Endurance holds the small creatures and lets one come back free.
#[test]
fn idol_of_endurance_recasts_a_small_creature() {
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let wurm = g.add_card_to_graveyard(0, catalog::craw_wurm());
    let idol = g.add_card_to_hand(0, catalog::idol_of_endurance());
    cast(&mut g, idol, None);
    assert!(g.exile.iter().any(|c| c.id == bear));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == wurm), "mana value 6 stays");
    activate(&mut g, idol, 0).expect("activate");
    g.players[0].mana_pool = Default::default();
    g.perform_action(GameAction::CastFromZoneWithoutPaying { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast for free");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
}

/// Gaze of Justice taps three white creatures to exile.
#[test]
fn gaze_of_justice_taps_three_white_creatures() {
    let mut g = pod(2);
    let whites: Vec<_> = (0..3).map(|_| g.add_card_to_battlefield(0, catalog::elite_vanguard())).collect();
    let target = g.add_card_to_battlefield(1, catalog::hill_giant());
    let gaze = g.add_card_to_hand(0, catalog::gaze_of_justice());
    cast(&mut g, gaze, Some(Target::Permanent(target)));
    assert!(g.exile.iter().any(|c| c.id == target));
    assert!(whites.iter().all(|&c| g.battlefield_find(c).unwrap().tapped));
}

/// Weight of Conscience needs two creatures that share a creature type.
#[test]
fn weight_of_conscience_needs_a_shared_type() {
    for (second, ok) in [(catalog::hill_giant(), false), (catalog::grizzly_bears(), true)] {
        let mut g = pod(2);
        let target = g.add_card_to_battlefield(1, catalog::craw_wurm());
        let weight = g.add_card_to_hand(0, catalog::weight_of_conscience());
        cast(&mut g, weight, Some(Target::Permanent(target)));
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, second);
        g.priority.player_with_priority = 0;
        let act = g.perform_action(GameAction::ActivateAbility {
            card_id: weight,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        });
        assert_eq!(act.is_ok(), ok, "a Bear and a {}", if ok { "Bear" } else { "Giant" });
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(target).is_none(), ok);
    }
}

/// Candletrap's exile is coven-gated.
#[test]
fn candletrap_exile_needs_coven() {
    let mut g = pod(2);
    let target = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let trap = g.add_card_to_hand(0, catalog::candletrap());
    cast(&mut g, trap, Some(Target::Permanent(target)));
    assert!(activate(&mut g, trap, 0).is_err(), "no coven yet");
    for c in [catalog::llanowar_elves(), catalog::grizzly_bears(), catalog::hill_giant()] {
        g.add_card_to_battlefield(0, c);
    }
    activate(&mut g, trap, 0).expect("powers 1, 2, 3");
    assert!(g.exile.iter().any(|c| c.id == target));
}

#[test]
fn cr_602_2b_paragon_of_modernity_counts_the_colors_spent_to_activate() {
    let mut g = pod(2);
    let paragon = g.add_card_to_battlefield(0, catalog::paragon_of_modernity());
    let activate_paying = |g: &mut GameState, colors: &[Color]| {
        g.players[0].mana_pool = Default::default();
        for &c in colors {
            g.players[0].mana_pool.add(c, 1);
        }
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: paragon,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("{3} paid");
        drain_stack(g);
    };
    activate_paying(&mut g, &[Color::White, Color::White, Color::Blue]);
    assert_eq!(pt(&g, paragon), (3, 3), "two colors: the pump");
    assert_eq!(g.battlefield_find(paragon).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    activate_paying(&mut g, &[Color::White, Color::Blue, Color::Black]);
    assert_eq!(pt(&g, paragon), (4, 4), "three colors: a counter instead of the pump");
    assert_eq!(g.battlefield_find(paragon).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn angels_tomb_and_scroll_of_avacyn_make_and_count_an_angel() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let tomb = g.add_card_to_battlefield(0, catalog::angels_tomb());
    let scroll = g.add_card_to_battlefield(0, catalog::scroll_of_avacyn());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    flood(&mut g, 0);
    cast(&mut g, bears, None);
    let angel = g.computed_permanent(tomb).expect("on the battlefield");
    assert_eq!((angel.power, angel.toughness), (3, 3));
    assert!(angel.card_types().contains(&CardType::Artifact) && angel.card_types().contains(&CardType::Creature));
    assert!(angel.keywords().contains(&Keyword::Flying));
    assert!(angel.colors.contains(Color::White));
    let (life, hand) = (g.players[0].life, g.players[0].hand.len());
    activate(&mut g, scroll, 0).expect("{1}, sacrifice");
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.players[0].life, life + 5, "the Tomb is an Angel this turn");
}

#[test]
fn nettlevine_blight_eats_its_host_and_moves_on_at_the_host_controllers_end_step() {
    let mut g = pod(2);
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let forest = g.add_card_to_battlefield(1, catalog::forest());
    let blight = g.add_card_to_hand(0, catalog::nettlevine_blight());
    cast(&mut g, blight, Some(Target::Permanent(wurm)));
    // Seat 0's own end step: not the host's controller's.
    g.step = TurnStep::PostCombatMain;
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(wurm).is_some(), "only the host's controller's end step");
    while !(g.active_player_idx == 1 && g.step == TurnStep::End) {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(wurm).is_none(), "the Wurm was sacrificed");
    assert_eq!(g.battlefield_find(blight).and_then(|c| c.attached_to), Some(forest), "the Blight moved to the Forest");
}

#[test]
fn count_nefaria_costs_three_less_after_a_sacrifice() {
    let mut g = pod(2);
    let nefaria = g.add_card_to_hand(0, catalog::count_nefaria());
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Black, 4);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell { card_id: nefaria, target: None, additional_targets: vec![], mode: None, x_value: None }).is_err(),
        "{{6}}{{B}} with four mana"
    );
    g.players[0].permanents_sacrificed_this_turn = 1;
    g.perform_action(GameAction::CastSpell { card_id: nefaria, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{3}{B} after a sacrifice");
    drain_stack(&mut g);
    assert!(g.battlefield_find(nefaria).is_some());
}

#[test]
fn cr_702_26_teferi_timeless_voyager_keeps_them_out_through_your_next_turn() {
    let mut g = pod(3);
    let teferi = g.add_card_to_battlefield(0, catalog::teferi_timeless_voyager());
    g.battlefield_find_mut(teferi).unwrap().add_counters(CounterType::Loyalty, 8);
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let other = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: teferi,
        ability_index: 2,
        target: Some(Target::Player(1)),
        x_value: None,
    })
    .expect("−8");
    drain_stack(&mut g);
    assert!(g.phased_out.iter().any(|c| c.id == wurm), "the targeted opponent's creature phased out");
    assert!(g.battlefield_find(other).is_some(), "the other opponent's didn't");
    let pass_to = |g: &mut GameState, seat: usize, step: TurnStep| {
        while !(g.active_player_idx == seat && g.step == step) {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
    };
    pass_to(&mut g, 1, TurnStep::Draw);
    assert!(g.phased_out.iter().any(|c| c.id == wurm), "its controller's next untap step doesn't phase it in");
    pass_to(&mut g, 0, TurnStep::PostCombatMain);
    assert!(g.phased_out.iter().any(|c| c.id == wurm), "still out during your next turn");
    pass_to(&mut g, 1, TurnStep::Draw);
    assert!(g.battlefield_find(wurm).is_some(), "your next turn has ended: it phases in at its controller's untap");
}

#[test]
fn niambi_fetches_teferi_from_the_graveyard_and_the_sentinel_reads_him() {
    let mut g = pod(2);
    let timebender = g.add_card_to_graveyard(0, catalog::teferi_timebender());
    let sentinel = g.add_card_to_battlefield(0, catalog::teferis_sentinel());
    assert_eq!(pt(&g, sentinel), (2, 6));
    let niambi = g.add_card_to_hand(0, catalog::niambi_faithful_healer());
    cast(&mut g, niambi, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == timebender), "Teferi, Timebender came back to hand");
    cast(&mut g, timebender, None);
    assert_eq!(pt(&g, sentinel), (6, 6), "+4/+0 with a Teferi planeswalker");
}

/// CR 106.6 — Guildmages' Forum's mana rides the multicolored creature spell
/// it pays for: that creature enters with a +1/+1 counter; a monocolored one
/// doesn't.
#[test]
fn guildmages_forum_counters_a_multicolored_creature() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let forum_cast = |creature: crabomination::card::CardDefinition, color: Color| {
        let mut g = pod(2);
        let forum = g.add_card_to_battlefield(0, catalog::guildmages_forum());
        let id = g.add_card_to_hand(0, creature);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(color)]));
        g.players[0].mana_pool = Default::default();
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility { card_id: forum, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None })
            .expect("{1}, {T}");
        drain_stack(&mut g);
        g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
            .expect("one mana");
        drain_stack(&mut g);
        g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne)
    };
    assert_eq!(forum_cast(catalog::boros_recruit(), Color::Red), 1, "a {{R/W}} hybrid creature is multicolored");
    assert_eq!(forum_cast(catalog::raging_goblin(), Color::Red), 0, "a mono-red one isn't");
}

#[test]
fn javelin_of_lightning_arms_on_your_turn_only() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let javelin = g.add_card_to_hand(0, catalog::javelin_of_lightning());
    cast(&mut g, javelin, Some(Target::Permanent(bears)));
    assert_eq!(pt(&g, bears), (4, 2), "attached on entry, +2/+0 on your turn");
    assert!(g.computed_permanent(bears).unwrap().keywords().contains(&Keyword::FirstStrike));
    g.active_player_idx = 1;
    assert_eq!(pt(&g, bears), (2, 2), "not on an opponent's turn");
}

#[test]
fn wooden_cane_and_coal_stoker_pay_out_on_entry() {
    let mut g = pod(2);
    let cane = g.add_card_to_hand(0, catalog::wooden_cane());
    cast(&mut g, cane, None);
    let mutant = g.battlefield_find(cane).unwrap().attached_to.expect("attached to its Mutant");
    assert_eq!(pt(&g, mutant), (4, 3));
    let stoker = g.add_card_to_hand(0, catalog::coal_stoker());
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Red, 4);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: stoker, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3, "cast from hand: three red");
}
