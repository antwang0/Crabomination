//! Commander batch — the Edgar Markov suite (`decks::cmdr_edgar`).

use crabomination::card::{CardDefinition, CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase_n(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn main_phase() -> GameState {
    main_phase_n(2)
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 12);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

/// Put `def` onto seat 0's battlefield and fire its ETB.
fn etb(g: &mut GameState, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(0, def);
    g.fire_self_etb_triggers(id, 0);
    drain_stack(g);
    id
}

fn activate_at(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), GameError> {
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

fn activate(g: &mut GameState, id: CardId, index: usize) -> Result<(), GameError> {
    activate_at(g, id, index, None)
}

fn loyalty(g: &mut GameState, id: CardId, index: usize) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: id,
        ability_index: index,
        target: None,
        x_value: None,
    })
    .expect("loyalty ability");
    drain_stack(g);
}

/// Seat 0 attacks seat 1 with `attackers` and runs combat to its end.
fn swing(g: &mut GameState, attackers: &[CardId]) {
    for &a in attackers {
        g.clear_sickness(a);
    }
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

fn tokens_named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == seat && c.is_token && c.definition.name == name)
        .count()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let cp = g.computed_permanent(id).expect("on battlefield");
    (cp.power, cp.toughness)
}

fn has_kw(g: &GameState, id: CardId, kw: Keyword) -> bool {
    g.computed_permanent(id).unwrap().keywords().contains(&kw)
}

fn cast_spell(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn script(g: &mut GameState, answers: Vec<DecisionAnswer>) {
    g.decider = Box::new(ScriptedDecider::new(answers));
}

// ── Creatures ────────────────────────────────────────────────────────────────

/// Edgar, Ancient Bloodlord eats another creature for a counter and menace,
/// and its own death trigger pays a life for the fodder.
#[test]
fn edgar_ancient_bloodlord_sacrifices_for_a_counter_and_menace() {
    let mut g = main_phase();
    let edgar = g.add_card_to_battlefield(0, catalog::edgar_ancient_bloodlord());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, edgar, 0).expect("sacrifice outlet");
    assert!(g.battlefield_find(bears).is_none(), "the Bears were sacrificed");
    assert_eq!(g.battlefield_find(edgar).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(has_kw(&g, edgar, Keyword::Menace));
    assert_eq!(g.players[0].life, 21, "another creature died");
}

/// Bloodline Recollector becomes prepared at an end step after three deaths,
/// then its Ancestral Craving copy draws three and costs three life.
#[test]
fn bloodline_recollector_prepares_after_three_deaths() {
    let mut g = main_phase();
    stock_libraries(&mut g, 10);
    let rec = g.add_card_to_battlefield(0, catalog::bloodline_recollector());
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(rec).unwrap().counter_count(CounterType::Prepared), 0);
    g.players[1].creatures_died_this_turn = 3;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(rec).unwrap().counter_count(CounterType::Prepared),
        1,
        "prepared once, however many end steps pass"
    );
    flood(&mut g, 0);
    let hand = g.players[1].hand.len();
    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: rec,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the prepared copy");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand + 3);
    assert_eq!(g.players[1].life, 17);
}

/// Bloodline Keeper mints flyers, and with five Vampires flips into the lord.
#[test]
fn bloodline_keeper_mints_then_transforms() {
    let mut g = main_phase();
    let keeper = g.add_card_to_battlefield(0, catalog::bloodline_keeper());
    g.clear_sickness(keeper);
    activate(&mut g, keeper, 0).expect("tap for a Vampire");
    assert_eq!(tokens_named(&g, 0, "Vampire"), 1);
    assert!(activate(&mut g, keeper, 1).is_err(), "only two Vampires");
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    }
    activate(&mut g, keeper, 1).expect("five Vampires: transform");
    assert_eq!(g.battlefield_find(keeper).unwrap().definition.name, "Lord of Lineage");
    let token = g.battlefield.iter().find(|c| c.is_token).unwrap().id;
    assert_eq!(pt(&g, token), (4, 4), "Other Vampires get +2/+2");
    assert_eq!(pt(&g, keeper), (5, 5));
}

/// Sanctum Seeker drains *each* opponent per attacking Vampire in a pod, and
/// its controller gains just 1 per trigger.
#[test]
fn sanctum_seeker_drains_each_opponent_per_vampire_attack() {
    let mut g = main_phase_n(4);
    let seeker = g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    let vamp = g.add_card_to_battlefield(0, catalog::oathsworn_vampire());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    swing(&mut g, &[seeker, vamp, bears]);
    assert_eq!(g.players[0].life, 22, "two Vampire attack triggers");
    assert_eq!(g.players[2].life, 18);
    assert_eq!(g.players[3].life, 18);
    assert_eq!(g.players[1].life, 20 - 2 - 3 - 2 - 2, "drained twice, then hit for 7");
}

/// Master of Dark Rites' {B}{B}{B} only pays for Vampire/Cleric/Demon spells.
#[test]
fn master_of_dark_rites_mana_is_tribal() {
    let mut g = main_phase();
    let master = g.add_card_to_battlefield(0, catalog::master_of_dark_rites());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(master);
    g.perform_action(GameAction::ActivateAbility {
        card_id: master,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("sacrifice for mana");
    let bones = g.add_card_to_hand(0, catalog::read_the_bones());
    assert!(cast_spell(&mut g, bones, None).is_err(), "not a Vampire, Cleric, or Demon");
    let baron = g.add_card_to_hand(0, catalog::markov_baron());
    cast_spell(&mut g, baron, None).expect("a Vampire spell");
    assert!(g.battlefield_find(baron).is_some());
}

/// Elenda grows off every other death and leaves lifelinkers equal to her
/// last-known power.
#[test]
fn elenda_grows_then_splits_into_tokens() {
    let mut g = main_phase();
    let elenda = g.add_card_to_battlefield(0, catalog::elenda_the_dusk_rose());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    flood(&mut g, 0);
    cast_spell(&mut g, bolt, Some(Target::Permanent(bears))).unwrap();
    assert_eq!(pt(&g, elenda), (2, 2));
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_spell(&mut g, bolt2, Some(Target::Permanent(elenda))).unwrap();
    assert_eq!(tokens_named(&g, 0, "Vampire"), 2, "X = her power, 2");
    let tok = g.battlefield.iter().find(|c| c.is_token).unwrap().id;
    assert!(has_kw(&g, tok, Keyword::Lifelink));
}

/// Charismatic Conqueror: an opponent who won't tap their new creature hands
/// the Conqueror's controller a lifelinker; one who taps it doesn't.
#[test]
fn charismatic_conqueror_taxes_untapped_entries() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::charismatic_conqueror());
    for (answer, tokens) in [(false, 1), (true, 1)] {
        script(&mut g, vec![DecisionAnswer::Bool(answer)]);
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        flood(&mut g, 1);
        let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
        g.perform_action(GameAction::CastSpell {
            card_id: bears,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("opponent casts Bears");
        drain_stack(&mut g);
        assert_eq!(tokens_named(&g, 0, "Vampire"), tokens, "answer {answer}");
        assert_eq!(g.battlefield_find(bears).unwrap().tapped, answer);
    }
}

/// Edgar, Charmed Groom dies into his Coffin, which mints a lifelinker each
/// upkeep and flips back into Edgar on the third.
#[test]
fn edgar_charmed_groom_cycles_through_the_coffin() {
    let mut g = main_phase();
    let edgar = g.add_card_to_battlefield(0, catalog::edgar_charmed_groom());
    let seeker = g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    assert_eq!(pt(&g, seeker), (4, 5), "Other Vampires +1/+1");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    cast_spell(&mut g, bolt, Some(Target::Permanent(edgar))).unwrap();
    cast_spell(&mut g, bolt2, Some(Target::Permanent(edgar))).unwrap();
    let coffin = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Edgar Markov's Coffin")
        .expect("returned transformed")
        .id;
    for n in 1..=3 {
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(tokens_named(&g, 0, "Vampire"), n);
    }
    let back = g.battlefield_find(coffin).unwrap();
    assert_eq!(back.definition.name, "Edgar, Charmed Groom", "third counter flips it");
    assert_eq!(back.counter_count(CounterType::Bloodline), 0);
}

/// Clavileño turns an attacking Vampire into a Demon that leaves a card and a
/// tapped 4/3 flyer behind when it dies.
#[test]
fn clavileno_blesses_an_attacking_vampire() {
    let mut g = main_phase();
    stock_libraries(&mut g, 5);
    g.add_card_to_battlefield(0, catalog::clavileno_first_of_the_blessed());
    let seeker = g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    swing(&mut g, &[seeker]);
    let cp = g.computed_permanent(seeker).unwrap();
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Demon));
    let hand = g.players[0].hand.len();
    g.step = TurnStep::PostCombatMain;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    cast_spell(&mut g, bolt, Some(Target::Permanent(seeker))).unwrap();
    cast_spell(&mut g, bolt2, Some(Target::Permanent(seeker))).unwrap();
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card (the bolts left hand)");
    let demon = g.battlefield.iter().find(|c| c.is_token).expect("Vampire Demon");
    assert!(demon.tapped);
    assert_eq!((demon.definition.power, demon.definition.toughness), (4, 3));
}

/// The Vampire lords: Markov Baron pumps the others, not itself.
#[test]
fn markov_baron_pumps_other_vampires() {
    let mut g = main_phase();
    let baron = g.add_card_to_battlefield(0, catalog::markov_baron());
    let seeker = g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, baron), (2, 2));
    assert_eq!(pt(&g, seeker), (4, 5));
    assert_eq!(pt(&g, bears), (2, 2));
    assert!(g.battlefield_find(baron).unwrap().definition.keywords.contains(&Keyword::Convoke));
}

/// Champion of Dusk draws and bleeds per Vampire, itself included.
#[test]
fn champion_of_dusk_draws_per_vampire() {
    let mut g = main_phase();
    stock_libraries(&mut g, 10);
    g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    let hand = g.players[0].hand.len();
    etb(&mut g, catalog::champion_of_dusk());
    assert_eq!(g.players[0].hand.len(), hand + 2);
    assert_eq!(g.players[0].life, 18);
}

/// Mavren Fein makes one token per attack with nontoken Vampires.
#[test]
fn mavren_fein_mints_on_a_vampire_attack() {
    let mut g = main_phase();
    let mavren = g.add_card_to_battlefield(0, catalog::mavren_fein_dusk_apostle());
    let seeker = g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    swing(&mut g, &[mavren, seeker]);
    assert_eq!(tokens_named(&g, 0, "Vampire"), 1, "one or more — one trigger");
}

/// Patron of the Vein kills on entry, then exiles the body and feeds the
/// Vampires.
#[test]
fn patron_of_the_vein_kills_exiles_and_grows() {
    let mut g = main_phase();
    let seeker = g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let patron = etb(&mut g, catalog::patron_of_the_vein());
    assert!(g.battlefield_find(bears).is_none());
    assert!(g.exile.iter().any(|c| c.id == bears), "exiled from the graveyard");
    assert_eq!(g.battlefield_find(seeker).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(patron).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Forerunner of the Legion stacks a Vampire on top of the library.
#[test]
fn forerunner_of_the_legion_tutors_to_the_top() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.add_card_to_library(0, catalog::sanctum_seeker());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    script(&mut g, vec![DecisionAnswer::Bool(true)]);
    etb(&mut g, catalog::forerunner_of_the_legion());
    assert_eq!(g.players[0].library.first().unwrap().definition.name, "Sanctum Seeker");
}

/// Rakish Heir grows each Vampire that connects.
#[test]
fn rakish_heir_grows_connecting_vampires() {
    let mut g = main_phase();
    let heir = g.add_card_to_battlefield(0, catalog::rakish_heir());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    swing(&mut g, &[heir, bears]);
    assert_eq!(g.battlefield_find(heir).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Olivia Voldaren pings a creature into a Vampire, grows, then steals it.
#[test]
fn olivia_voldaren_converts_then_steals() {
    let mut g = main_phase();
    let olivia = g.add_card_to_battlefield(0, catalog::olivia_voldaren());
    let hill = g.add_card_to_battlefield(1, catalog::sanctum_seeker());
    activate_at(&mut g, olivia, 0, Some(Target::Permanent(hill))).expect("ping");
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate_at(&mut g, olivia, 0, Some(Target::Permanent(bears))).expect("ping the Bears");
    assert!(g.computed_permanent(bears).unwrap().subtypes().creature_types.contains(&CreatureType::Vampire));
    assert_eq!(g.battlefield_find(olivia).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    activate_at(&mut g, olivia, 1, Some(Target::Permanent(bears))).expect("steal the new Vampire");
    assert_eq!(g.battlefield_find(bears).unwrap().controller, 0);
}

/// Drana and Linvala shuts off an opponent's creature abilities and borrows
/// them.
#[test]
fn drana_and_linvala_locks_and_borrows_abilities() {
    let mut g = main_phase();
    let drana = g.add_card_to_battlefield(0, catalog::drana_and_linvala());
    let tim = g.add_card_to_battlefield(1, catalog::prodigal_sorcerer());
    g.clear_sickness(tim);
    g.clear_sickness(drana);
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: tim,
            ability_index: 0,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_err(),
        "the opponent's creature is locked"
    );
    activate_at(&mut g, drana, 0, Some(Target::Player(1))).expect("Drana pings with Tim's ability");
    assert_eq!(g.players[1].life, 19);
}

/// Oathsworn Vampire comes back from the graveyard only on a lifegain turn,
/// and enters tapped.
#[test]
fn oathsworn_vampire_recasts_after_lifegain() {
    let mut g = main_phase();
    let vamp = g.add_card_to_graveyard(0, catalog::oathsworn_vampire());
    flood(&mut g, 0);
    let recast = |g: &mut GameState| {
        g.perform_action(GameAction::CastFlashback {
            card_id: vamp,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    assert!(recast(&mut g).is_err(), "no life gained yet");
    g.players[0].life_gained_this_turn = 1;
    recast(&mut g).expect("gained life this turn");
    drain_stack(&mut g);
    assert!(g.battlefield_find(vamp).unwrap().tapped, "enters tapped");
}

/// Florian looks at as many cards as the opponents lost in total.
#[test]
fn florian_digs_for_the_total_life_lost() {
    let mut g = main_phase_n(3);
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.add_card_to_battlefield(0, catalog::florian_voldaren_scion());
    g.players[1].life_lost_this_turn = 2;
    g.players[2].life_lost_this_turn = 1;
    let exiled = g.exile.len();
    g.fire_step_triggers(TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.exile.len(), exiled + 1, "one card exiled out of the top three");
    assert_eq!(g.players[0].library.len(), 5);
}

/// Elenda's Hierophant grows on lifegain and splits on death.
#[test]
fn elendas_hierophant_grows_on_lifegain() {
    let mut g = main_phase();
    let hiero = g.add_card_to_battlefield(0, catalog::elendas_hierophant());
    // Life gained through Sanctum Seeker's drain.
    let seeker = g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    swing(&mut g, &[seeker]);
    assert_eq!(g.battlefield_find(hiero).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    g.step = TurnStep::PostCombatMain;
    cast_spell(&mut g, bolt, Some(Target::Permanent(hiero))).unwrap();
    assert_eq!(tokens_named(&g, 0, "Vampire"), 2);
}

/// Vampire Nocturnus lights up while the top card is black.
#[test]
fn vampire_nocturnus_keys_off_a_black_top_card() {
    let mut g = main_phase();
    let noc = g.add_card_to_battlefield(0, catalog::vampire_nocturnus());
    let seeker = g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, noc), (3, 3), "green on top");
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::read_the_bones());
    assert_eq!(pt(&g, noc), (5, 4));
    assert!(has_kw(&g, noc, Keyword::Flying));
    assert_eq!(pt(&g, seeker), (5, 5));
    assert_eq!(pt(&g, bears), (2, 2));
}

/// Creeping Bloodsucker pings each opponent and gains what it dealt.
#[test]
fn creeping_bloodsucker_drains_the_table() {
    let mut g = main_phase_n(4);
    g.add_card_to_battlefield(0, catalog::creeping_bloodsucker());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 23);
    for p in 1..4 {
        assert_eq!(g.players[p].life, 19);
    }
}

/// Carmen grows off any sacrifice and reanimates up to her power on attack.
#[test]
fn carmen_grows_and_reanimates() {
    let mut g = main_phase();
    let carmen = g.add_card_to_battlefield(0, catalog::carmen_cruel_skymarcher());
    let idol = g.add_card_to_battlefield(0, catalog::idol_of_oblivion());
    activate(&mut g, idol, 1).expect("sacrifice the Idol");
    assert_eq!(g.battlefield_find(carmen).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.players[0].life, 21);
    let bears = idol;
    g.clear_sickness(carmen);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: carmen,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_some(), "the Idol (MV 2 <= power 3) came back");
}

// ── Instants and sorceries ───────────────────────────────────────────────────

/// Bilbo's Gambit bounces a spell; gifted, it also silences the turn.
#[test]
fn bilbos_gambit_bounces_and_silences_when_gifted() {
    let mut g = main_phase();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    flood(&mut g, 1);
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.perform_action(GameAction::CastSpell {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("opponent casts");
    g.perform_action(GameAction::PassPriority).unwrap();
    assert_eq!(g.priority.player_with_priority, 0);
    let gambit = g.add_card_to_hand(0, catalog::bilbos_gambit());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastGift {
        card_id: gambit,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("gifted Gambit");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bears), "back in hand");
    assert_eq!(tokens_named(&g, 1, "Treasure"), 1, "the gift");
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bears,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "players can't cast spells this turn"
    );
}

/// Olivia's Wrath shrinks every non-Vampire by the Vampire count.
#[test]
fn olivias_wrath_spares_vampires() {
    let mut g = main_phase();
    let seeker = g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    g.add_card_to_battlefield(0, catalog::oathsworn_vampire());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wrath = g.add_card_to_hand(0, catalog::olivias_wrath());
    flood(&mut g, 0);
    cast_spell(&mut g, wrath, None).unwrap();
    assert!(g.battlefield_find(bears).is_none());
    assert_eq!(pt(&g, seeker), (3, 4));
}

/// Farewell exiles exactly the chosen categories.
#[test]
fn farewell_exiles_the_chosen_modes() {
    let mut g = main_phase();
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let heart = g.add_card_to_battlefield(0, catalog::glass_cast_heart());
    let necro = g.add_card_to_battlefield(0, catalog::necropotence());
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let farewell = g.add_card_to_hand(0, catalog::farewell());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: farewell,
        spree_modes: vec![1, 3],
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("creatures and graveyards");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none());
    assert!(g.battlefield_find(heart).is_some(), "artifacts weren't chosen");
    assert!(g.battlefield_find(necro).is_some(), "enchantments weren't chosen");
    assert!(g.players[1].graveyard.is_empty());
}

/// Clever Concealment phases out the permanents it targets.
#[test]
fn clever_concealment_phases_out_targets() {
    let mut g = main_phase();
    let seeker = g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    let heart = g.add_card_to_battlefield(0, catalog::glass_cast_heart());
    let spell = g.add_card_to_hand(0, catalog::clever_concealment());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(seeker)),
        additional_targets: vec![Target::Permanent(heart)],
        mode: None,
        x_value: None,
    })
    .expect("two targets");
    drain_stack(&mut g);
    assert!(g.battlefield_find(seeker).is_none());
    assert!(g.battlefield_find(heart).is_none());
}

/// The choose-a-type spells: And They Shall Know No Fear, Kindred Dominance,
/// Bloodline Bidding and Pact of the Serpent, all naming Vampire.
#[test]
fn chosen_type_spells_follow_the_named_type() {
    // And They Shall Know No Fear.
    let mut g = main_phase();
    let seeker = g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    script(&mut g, vec![DecisionAnswer::CreatureType(CreatureType::Vampire)]);
    let fear = g.add_card_to_hand(0, catalog::and_they_shall_know_no_fear());
    flood(&mut g, 0);
    cast_spell(&mut g, fear, None).unwrap();
    assert_eq!(pt(&g, seeker), (4, 4));
    assert!(has_kw(&g, seeker, Keyword::Indestructible));
    assert_eq!(pt(&g, bears), (2, 2));

    // Kindred Dominance.
    let mut g = main_phase();
    let seeker = g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    script(&mut g, vec![DecisionAnswer::CreatureType(CreatureType::Vampire)]);
    let dom = g.add_card_to_hand(0, catalog::kindred_dominance());
    flood(&mut g, 0);
    cast_spell(&mut g, dom, None).unwrap();
    assert!(g.battlefield_find(seeker).is_some());
    assert!(g.battlefield_find(bears).is_none());

    // Bloodline Bidding.
    let mut g = main_phase();
    let v = g.add_card_to_graveyard(0, catalog::sanctum_seeker());
    let b = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    script(&mut g, vec![DecisionAnswer::CreatureType(CreatureType::Vampire)]);
    let bid = g.add_card_to_hand(0, catalog::bloodline_bidding());
    flood(&mut g, 0);
    cast_spell(&mut g, bid, None).unwrap();
    assert!(g.battlefield_find(v).is_some());
    assert!(g.battlefield_find(b).is_none());

    // Pact of the Serpent — X counts the *target's* creatures of the type.
    let mut g = main_phase();
    stock_libraries(&mut g, 10);
    g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    g.add_card_to_battlefield(0, catalog::oathsworn_vampire());
    g.add_card_to_battlefield(1, catalog::rakish_heir());
    script(&mut g, vec![DecisionAnswer::CreatureType(CreatureType::Vampire)]);
    let pact = g.add_card_to_hand(0, catalog::pact_of_the_serpent());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    cast_spell(&mut g, pact, Some(Target::Player(0))).unwrap();
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2);
    assert_eq!(g.players[0].life, 18);
}

/// New Blood taps a Vampire to steal a creature, which becomes a Vampire.
#[test]
fn new_blood_steals_and_converts() {
    let mut g = main_phase();
    let seeker = g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let blood = g.add_card_to_hand(0, catalog::new_blood());
    flood(&mut g, 0);
    cast_spell(&mut g, blood, Some(Target::Permanent(bears))).unwrap();
    assert!(g.battlefield_find(seeker).unwrap().tapped, "the additional cost");
    assert_eq!(g.battlefield_find(bears).unwrap().controller, 0);
    assert!(g.computed_permanent(bears).unwrap().subtypes().creature_types.contains(&CreatureType::Vampire));
}

// ── Artifacts ────────────────────────────────────────────────────────────────

/// Orcrist pays a Treasure per creature of the named type on a hit.
#[test]
fn orcrist_mints_treasures_per_named_creature() {
    let mut g = main_phase();
    let seeker = g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    g.add_card_to_battlefield(0, catalog::oathsworn_vampire());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sword = g.add_card_to_battlefield(0, catalog::orcrist_goblin_cleaver());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: sword, target: seeker }).expect("equip");
    drain_stack(&mut g);
    assert_eq!(pt(&g, seeker), (5, 6));
    assert!(has_kw(&g, seeker, Keyword::Trample));
    script(&mut g, vec![DecisionAnswer::CreatureType(CreatureType::Vampire)]);
    swing(&mut g, &[seeker]);
    assert_eq!(tokens_named(&g, 0, "Treasure"), 2);
}

/// Banner of Kinship counts the named type as it enters and scales with it.
#[test]
fn banner_of_kinship_counts_fellowship() {
    let mut g = main_phase();
    let seeker = g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    g.add_card_to_battlefield(0, catalog::oathsworn_vampire());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    script(&mut g, vec![DecisionAnswer::CreatureType(CreatureType::Vampire)]);
    let banner = g.add_card_to_hand(0, catalog::banner_of_kinship());
    flood(&mut g, 0);
    cast_spell(&mut g, banner, None).unwrap();
    assert_eq!(g.battlefield_find(banner).unwrap().counter_count(CounterType::Fellowship), 2);
    assert_eq!(pt(&g, seeker), (5, 6));
    assert_eq!(pt(&g, bears), (2, 2));
}

/// Idol of Oblivion draws only on a token turn.
#[test]
fn idol_of_oblivion_needs_a_token() {
    let mut g = main_phase();
    stock_libraries(&mut g, 5);
    let idol = g.add_card_to_battlefield(0, catalog::idol_of_oblivion());
    assert!(activate(&mut g, idol, 0).is_err(), "no token yet");
    let heart = g.add_card_to_battlefield(0, catalog::glass_cast_heart());
    activate(&mut g, heart, 0).expect("make a Vampire");
    let hand = g.players[0].hand.len();
    activate(&mut g, idol, 0).expect("created a token this turn");
    assert_eq!(g.players[0].hand.len(), hand + 1);
    g.battlefield_find_mut(idol).unwrap().tapped = false;
    activate(&mut g, idol, 1).expect("the Eldrazi");
    assert_eq!(tokens_named(&g, 0, "Eldrazi"), 1);
}

/// Chronicle of Victory: +2/+2, first strike, trample, and a card per cast.
#[test]
fn chronicle_of_victory_crowns_the_named_type() {
    let mut g = main_phase();
    stock_libraries(&mut g, 5);
    let seeker = g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    script(&mut g, vec![DecisionAnswer::CreatureType(CreatureType::Vampire)]);
    let chron = g.add_card_to_hand(0, catalog::chronicle_of_victory());
    flood(&mut g, 0);
    cast_spell(&mut g, chron, None).unwrap();
    assert_eq!(pt(&g, seeker), (5, 6));
    assert!(has_kw(&g, seeker, Keyword::FirstStrike));
    assert!(has_kw(&g, seeker, Keyword::Trample));
    let hand = g.players[0].hand.len();
    let baron = g.add_card_to_hand(0, catalog::markov_baron());
    cast_spell(&mut g, baron, None).unwrap();
    assert_eq!(g.players[0].hand.len(), hand + 1, "cast a Vampire: draw");
}

/// Glass-Cast Heart: Blood on a Vampire attack, and lifelinkers for life.
#[test]
fn glass_cast_heart_makes_blood_and_vampires() {
    let mut g = main_phase();
    let heart = g.add_card_to_battlefield(0, catalog::glass_cast_heart());
    let seeker = g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    swing(&mut g, &[seeker]);
    assert_eq!(tokens_named(&g, 0, "Blood"), 1);
    activate(&mut g, heart, 0).expect("{B}, {T}, pay 1 life");
    assert_eq!(tokens_named(&g, 0, "Vampire"), 1);
    assert_eq!(g.players[0].life, 20 + 1 - 1, "Seeker's drain, then the life payment");
    g.battlefield_find_mut(heart).unwrap().tapped = false;
    assert!(activate(&mut g, heart, 1).is_err(), "needs thirteen Blood");
}

/// Heirloom Blade finds a creature that shares a type with the fallen one.
#[test]
fn heirloom_blade_finds_a_kindred_creature() {
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::sanctum_seeker());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let vamp = g.add_card_to_battlefield(0, catalog::oathsworn_vampire());
    let blade = g.add_card_to_battlefield(0, catalog::heirloom_blade());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: blade, target: vamp }).expect("equip");
    drain_stack(&mut g);
    assert_eq!(pt(&g, vamp), (5, 3));
    script(&mut g, vec![DecisionAnswer::Bool(true)]);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    cast_spell(&mut g, bolt, Some(Target::Permanent(vamp))).unwrap();
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Sanctum Seeker"));
    assert_eq!(g.players[0].library.len(), 2, "the miss went to the bottom");
}

// ── Enchantments ─────────────────────────────────────────────────────────────

/// Gleaming Splendor pays once per opponent's second draw — each opponent's.
#[test]
fn gleaming_splendor_pays_on_each_opponents_second_draw() {
    let mut g = main_phase_n(3);
    stock_libraries(&mut g, 10);
    let splendor = g.add_card_to_battlefield(0, catalog::gleaming_splendor());
    let draw_two = |g: &mut GameState, a: usize, b: usize| {
        flood(g, 0);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: splendor,
            ability_index: 0,
            target: Some(Target::Player(a)),
            additional_targets: vec![Target::Player(b)],
            x_value: None,
            mode: None,
        })
        .expect("two target players");
        drain_stack(g);
        tokens_named(g, 0, "Treasure")
    };
    assert_eq!(draw_two(&mut g, 1, 0), 0, "seat 1's first draw");
    assert_eq!(draw_two(&mut g, 1, 2), 1, "seat 1's second draw, seat 2's first");
    assert_eq!(draw_two(&mut g, 2, 0), 2, "seat 2's second draw");
    assert_eq!(draw_two(&mut g, 1, 0), 2, "a third draw pays nothing, nor do mine");
}

/// Necropotence: life for cards that arrive at the end step; discards exiled.
#[test]
fn necropotence_banks_cards_for_the_end_step() {
    let mut g = main_phase();
    stock_libraries(&mut g, 5);
    let necro = g.add_card_to_battlefield(0, catalog::necropotence());
    let hand = g.players[0].hand.len();
    activate(&mut g, necro, 0).expect("pay 1 life");
    activate(&mut g, necro, 0).expect("pay 1 life");
    assert_eq!(g.players[0].life, 18);
    assert_eq!(g.players[0].hand.len(), hand);
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

/// Shared Animosity pumps each attacker by its kin among the attackers.
#[test]
fn shared_animosity_counts_attacking_kin() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::shared_animosity());
    let a = g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    let b = g.add_card_to_battlefield(0, catalog::oathsworn_vampire());
    let c = g.add_card_to_battlefield(0, catalog::rakish_heir());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[1].life = 100;
    swing(&mut g, &[a, b, c, bears]);
    // Seeker (Vampire Knight) shares with Oathsworn (Vampire Knight) and the
    // Heir (Vampire); the Bears share with nobody.
    // Each Vampire shares with the other two (+2/+0); the Bears get nothing.
    // 5 + 4 + 4 + 2 combat damage, and Sanctum Seeker drains once per Vampire.
    assert_eq!(g.players[1].life, 100 - 15 - 3);
}

/// Etchings of the Chosen: anthem, and a sacrifice for indestructible.
#[test]
fn etchings_of_the_chosen_protects_for_a_sacrifice() {
    let mut g = main_phase();
    let seeker = g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    g.add_card_to_battlefield(0, catalog::oathsworn_vampire());
    script(&mut g, vec![DecisionAnswer::CreatureType(CreatureType::Vampire)]);
    let etch = g.add_card_to_hand(0, catalog::etchings_of_the_chosen());
    flood(&mut g, 0);
    cast_spell(&mut g, etch, None).unwrap();
    assert_eq!(pt(&g, seeker), (4, 5));
    activate_at(&mut g, etch, 0, Some(Target::Permanent(seeker))).expect("sacrifice Oathsworn");
    assert!(has_kw(&g, seeker, Keyword::Indestructible));
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_creature()).count(), 1);
}

/// March of the Canonized: X lifelinkers, and a Demon at devotion seven.
#[test]
fn march_of_the_canonized_mints_x_then_demons() {
    let mut g = main_phase();
    let march = g.add_card_to_hand(0, catalog::march_of_the_canonized());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: march,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(tokens_named(&g, 0, "Vampire"), 3);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(tokens_named(&g, 0, "Vampire Demon"), 0, "devotion 2");
    g.add_card_to_battlefield(0, catalog::vampire_nocturnus()); // BBB
    g.add_card_to_battlefield(0, catalog::edgar_charmed_groom()); // WB
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(tokens_named(&g, 0, "Vampire Demon"), 1, "devotion 7");
}

/// Renewed Solidarity copies each token of the type that entered this turn.
#[test]
fn renewed_solidarity_copies_new_tokens() {
    let mut g = main_phase();
    script(&mut g, vec![DecisionAnswer::CreatureType(CreatureType::Vampire)]);
    let sol = g.add_card_to_hand(0, catalog::renewed_solidarity());
    flood(&mut g, 0);
    cast_spell(&mut g, sol, None).unwrap();
    let heart = g.add_card_to_battlefield(0, catalog::glass_cast_heart());
    activate(&mut g, heart, 0).expect("a Vampire token");
    let tok = g.battlefield.iter().find(|c| c.is_token).unwrap().id;
    assert_eq!(pt(&g, tok), (2, 1));
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(tokens_named(&g, 0, "Vampire"), 2);
}

// ── Planeswalkers ────────────────────────────────────────────────────────────

/// Sorin, Lord of Innistrad: lifelinker, then an anthem emblem.
#[test]
fn sorin_lord_of_innistrad_tokens_and_emblem() {
    let mut g = main_phase();
    let sorin = g.add_card_to_battlefield(0, catalog::sorin_lord_of_innistrad());
    loyalty(&mut g, sorin, 0);
    assert_eq!(tokens_named(&g, 0, "Vampire"), 1);
    let tok = g.battlefield.iter().find(|c| c.is_token).unwrap().id;
    g.battlefield_find_mut(sorin).unwrap().loyalty_uses_this_turn = 0;
    loyalty(&mut g, sorin, 1);
    assert_eq!(pt(&g, tok), (2, 1), "emblem: +1/+0");
}

/// Sorin, Solemn Visitor: +1 team pump with lifelink; −2 a flyer.
#[test]
fn sorin_solemn_visitor_pumps_and_mints() {
    let mut g = main_phase();
    let sorin = g.add_card_to_battlefield(0, catalog::sorin_solemn_visitor());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    loyalty(&mut g, sorin, 0);
    assert_eq!(pt(&g, bears), (3, 2));
    assert!(has_kw(&g, bears, Keyword::Lifelink));
    g.battlefield_find_mut(sorin).unwrap().loyalty_uses_this_turn = 0;
    loyalty(&mut g, sorin, 1);
    let tok = g.battlefield.iter().find(|c| c.is_token).expect("2/2 flyer").id;
    assert!(has_kw(&g, tok, Keyword::Flying));
}

// ── Lands ────────────────────────────────────────────────────────────────────

fn play_land(g: &mut GameState, def: CardDefinition) -> CardId {
    let id = g.add_card_to_hand(0, def);
    g.priority.player_with_priority = 0;
    g.players[0].lands_played_this_turn = 0;
    g.perform_action(GameAction::PlayLand(id)).expect("play land");
    drain_stack(g);
    id
}

/// The Battlebond crowd lands enter tapped in a duel and untapped in a pod.
#[test]
fn crowd_lands_enter_untapped_with_two_opponents() {
    for make in [catalog::vault_of_champions, catalog::luxury_suite, catalog::spectator_seating] {
        let mut duel = main_phase();
        let a = play_land(&mut duel, make());
        assert!(duel.battlefield_find(a).unwrap().tapped, "one opponent: tapped");
        let mut pod = main_phase_n(4);
        let b = play_land(&mut pod, make());
        assert!(!pod.battlefield_find(b).unwrap().tapped, "three opponents: untapped");
    }
}

/// Haunted Ridge (slow land), Foreboding Ruins (reveal land) and Minas
/// Tirith (legend check) enter tapped or untapped by their conditions.
#[test]
fn conditional_lands_check_their_conditions() {
    let mut g = main_phase();
    let ridge = play_land(&mut g, catalog::haunted_ridge());
    assert!(g.battlefield_find(ridge).unwrap().tapped, "no other lands");
    g.add_card_to_battlefield(0, catalog::swamp());
    g.add_card_to_battlefield(0, catalog::swamp());
    let ridge2 = play_land(&mut g, catalog::haunted_ridge());
    assert!(!g.battlefield_find(ridge2).unwrap().tapped);

    let mut g = main_phase();
    let ruins = play_land(&mut g, catalog::foreboding_ruins());
    assert!(g.battlefield_find(ruins).unwrap().tapped, "nothing to reveal");
    g.add_card_to_hand(0, catalog::mountain());
    script(&mut g, vec![DecisionAnswer::Bool(true)]);
    let ruins2 = play_land(&mut g, catalog::foreboding_ruins());
    assert!(!g.battlefield_find(ruins2).unwrap().tapped, "revealed a Mountain");

    let mut g = main_phase();
    let tirith = play_land(&mut g, catalog::minas_tirith());
    assert!(g.battlefield_find(tirith).unwrap().tapped);
    g.add_card_to_battlefield(0, catalog::olivia_voldaren());
    let tirith2 = play_land(&mut g, catalog::minas_tirith());
    assert!(!g.battlefield_find(tirith2).unwrap().tapped, "a legendary creature");
}

/// Minas Tirith draws only after a two-creature attack.
#[test]
fn minas_tirith_draws_after_a_wide_attack() {
    let mut g = main_phase();
    stock_libraries(&mut g, 5);
    let tirith = g.add_card_to_battlefield(0, catalog::minas_tirith());
    assert!(activate(&mut g, tirith, 1).is_err());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    swing(&mut g, &[a, b]);
    g.step = TurnStep::PostCombatMain;
    let hand = g.players[0].hand.len();
    activate(&mut g, tirith, 1).expect("attacked with two");
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Fetid Heath filters one hybrid into two of W/B.
#[test]
fn fetid_heath_filters_into_two() {
    let mut g = main_phase();
    let heath = g.add_card_to_battlefield(0, catalog::fetid_heath());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: heath,
        ability_index: 3,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("{W/B}, {T}: add {B}{B}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 2);
    assert_eq!(g.players[0].mana_pool.amount(Color::White), 0);
}

/// Vault of the Archangel grants deathtouch and lifelink to the team.
#[test]
fn vault_of_the_archangel_arms_the_team() {
    let mut g = main_phase();
    let vault = g.add_card_to_battlefield(0, catalog::vault_of_the_archangel());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, vault, 1).unwrap();
    assert!(has_kw(&g, bears, Keyword::Deathtouch));
    assert!(has_kw(&g, bears, Keyword::Lifelink));
}

/// Westvale Abbey makes Clerics, then eats five creatures to become Ormendahl.
#[test]
fn westvale_abbey_flips_into_ormendahl() {
    let mut g = main_phase();
    let abbey = g.add_card_to_battlefield(0, catalog::westvale_abbey());
    activate(&mut g, abbey, 1).expect("{5}, {T}, pay 1 life");
    assert_eq!(tokens_named(&g, 0, "Human Cleric"), 1);
    assert_eq!(g.players[0].life, 19);
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    g.battlefield_find_mut(abbey).unwrap().tapped = false;
    activate(&mut g, abbey, 2).expect("sacrifice five");
    let o = g.battlefield_find(abbey).unwrap();
    assert_eq!(o.definition.name, "Ormendahl, Profane Prince");
    assert!(!o.tapped, "then untap it");
    assert_eq!(pt(&g, abbey), (9, 7));
}

/// Accursed Duneyard regenerates only the listed undead types.
#[test]
fn accursed_duneyard_regenerates_vampires() {
    let mut g = main_phase();
    let yard = g.add_card_to_battlefield(0, catalog::accursed_duneyard());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(activate_at(&mut g, yard, 1, Some(Target::Permanent(bears))).is_err());
    let seeker = g.add_card_to_battlefield(0, catalog::sanctum_seeker());
    activate_at(&mut g, yard, 1, Some(Target::Permanent(seeker))).expect("a Vampire");
    let wrath = g.add_card_to_hand(0, catalog::kindred_dominance());
    script(&mut g, vec![DecisionAnswer::CreatureType(CreatureType::Goblin)]);
    flood(&mut g, 0);
    cast_spell(&mut g, wrath, None).unwrap();
    assert!(g.battlefield_find(seeker).is_some(), "regenerated");
    assert!(g.battlefield_find(bears).is_none());
}
