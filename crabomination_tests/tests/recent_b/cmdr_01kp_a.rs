//! Commander: the 2026-10-21 (`01KPxZ3H`) single-commander closers
//! (`decks::cmdr_01kp_a`).

use crabomination::card::{ArtifactSubtype, CardDefinition, CardId, CardType, CounterType, Keyword};
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
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

fn cast_x(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>, x_value: Option<u32>) {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value })
        .expect("cast");
    drain_stack(g);
}

fn ready(g: &mut GameState, seat: usize, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn attack(g: &mut GameState, seat: usize, attacks: &[(CardId, usize)]) {
    g.active_player_idx = seat;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.iter().map(|&(a, d)| Attack { attacker: a, target: AttackTarget::Player(d) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

/// CR 508.1 + 603.4 — Squall triggers when anyone's creatures attack one of
/// its controller's opponents (once per opponent attacked), only if one of
/// that opponent's attackers has power or toughness equal to the chosen
/// number; an attack on Squall's controller never triggers it.
#[test]
fn cr_603_4_squall_hits_each_attacked_opponent_with_a_matching_attacker() {
    let mut g = pod(4);
    let squall = g.add_card_to_hand(0, catalog::squall_gunblade_duelist());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    cast_x(&mut g, 0, squall, None, None);
    assert_eq!(g.battlefield_find(squall).and_then(|c| c.chosen_number), Some(2), "the commonest P/T it owns");
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let bears2 = ready(&mut g, 1, catalog::grizzly_bears());
    let life = [g.players[0].life, g.players[2].life, g.players[3].life];
    // Seat 1 attacks seat 2 with a 2/2 (a match), seat 3 with a 3/3 (none) and
    // seat 0 — Squall's controller — with a 2/2.
    attack(&mut g, 1, &[(bears, 2), (giant, 3), (bears2, 0)]);
    assert_eq!(g.players[2].life, life[1] - 3, "a 2/2 attacked seat 2");
    assert_eq!(g.players[3].life, life[2], "a 3/3 matches neither half");
    assert_eq!(g.players[0].life, life[0], "an attack on Squall's controller doesn't trigger it");
}

/// Duelist's Flame needs a blocked creature (CR 509.1h); with X = 2 the
/// trampling Hill Giant puts 4 past a 2/1 blocker, and the granted trigger
/// looks at that many cards, exiles Divination and casts it free.
#[test]
fn duelists_flame_pumps_a_blocked_creature_and_casts_off_its_damage() {
    let mut g = pod(2);
    g.players[0].library.clear();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let div = g.add_card_to_library(0, catalog::divination());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    let lions = g.add_card_to_battlefield(1, catalog::savannah_lions());
    let flame = g.add_card_to_hand(0, catalog::duelists_flame());
    attack(&mut g, 0, &[(giant, 1)]);
    flood(&mut g, 0);
    let early = GameAction::CastSpell {
        card_id: flame,
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    };
    assert!(g.perform_action(early).is_err(), "an unblocked attacker isn't a legal target");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(lions, giant)])).expect("block");
    drain_stack(&mut g);
    cast_x(&mut g, 0, flame, Some(Target::Permanent(giant)), Some(2));
    let c = g.battlefield_find(giant).unwrap();
    assert_eq!(c.power(), 5);
    assert!(g.permanent_has_keyword(giant, &Keyword::Trample));
    let (life, hand) = (g.players[1].life, g.players[0].hand.len());
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, life - 4, "trample past the 2/1");
    assert_eq!(g.players[0].hand.len(), hand + 2, "Divination was cast free");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == div));
    assert_eq!(g.players[0].library.len(), 1, "two drawn, the rest bottomed");
}

fn rex_trigger(g: &mut GameState, rex: CardId, target: CardId) {
    let fx = catalog::captain_rex_nebula().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(rex, 0, Some(Target::Permanent(target)), 0);
    let evs = g.resolve_effect(&fx, &ctx).expect("rex");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

/// Captain Rex Nebula (2022 ruling): the target loses its card types and
/// creature types, becomes a Vehicle artifact with base P/T its mana value and
/// crew 2; crewing it adds Creature but no creature types back; it all ends at
/// cleanup (CR 514.2).
#[test]
fn rex_nebula_turns_a_permanent_into_a_crewable_mana_value_vehicle() {
    let mut g = pod(2);
    let rex = ready(&mut g, 0, catalog::captain_rex_nebula());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    rex_trigger(&mut g, rex, giant);
    let cp = g.computed_permanent(giant).unwrap();
    assert_eq!(cp.card_types(), &[CardType::Artifact], "no longer a creature");
    assert_eq!(cp.subtypes().artifact_subtypes, vec![ArtifactSubtype::Vehicle]);
    assert!(cp.subtypes().creature_types.is_empty(), "loses Giant");
    g.perform_action(GameAction::Crew { vehicle: giant, crew_creatures: vec![rex] }).expect("crew 2");
    let cp = g.computed_permanent(giant).unwrap();
    assert!(cp.card_types().contains(&CardType::Creature) && cp.card_types().contains(&CardType::Artifact));
    assert!(cp.subtypes().creature_types.is_empty(), "crewing doesn't bring Giant back");
    assert_eq!((cp.power, cp.toughness), (4, 4), "base P/T = mana value 4");
    g.do_cleanup(&mut vec![]);
    let cp = g.computed_permanent(giant).unwrap();
    assert_eq!(cp.card_types(), &[CardType::Creature]);
    assert_eq!((cp.power, cp.toughness), (3, 3));
}

/// Crash Land: the Vehicle deals combat damage, the d6 comes up equal to its
/// mana value, so it is sacrificed and then deals that much damage.
#[test]
fn rex_nebula_crash_land_sacrifices_on_its_mana_value() {
    let mut g = pod(2);
    let rex = ready(&mut g, 0, catalog::captain_rex_nebula());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    rex_trigger(&mut g, rex, bears);
    g.perform_action(GameAction::Crew { vehicle: bears, crew_creatures: vec![rex] }).expect("crew 2");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(2)]));
    let life = g.players[1].life;
    attack(&mut g, 0, &[(bears, 1)]);
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(&mut g);
    }
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bears), "sacrificed on a 2");
    assert_eq!(g.players[1].life, life - 4, "2 combat + 2 crash");
}

/// Mysterious Limousine: each entry/attack exiles a creature until it leaves
/// (CR 610.3) and swaps the previous one back; when it leaves, the last one
/// returns too.
#[test]
fn mysterious_limousine_swaps_its_prisoner_and_frees_it_on_leaving() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let limo = g.add_card_to_hand(0, catalog::mysterious_limousine());
    cast_x(&mut g, 0, limo, None, None);
    assert!(g.exile.iter().any(|c| c.id == bears), "the ETB took the only other creature");
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let fx = catalog::mysterious_limousine().triggered_abilities[1].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(limo, 0, Some(Target::Permanent(giant)), 0);
    g.resolve_effect(&fx, &ctx).expect("swap");
    assert!(g.exile.iter().any(|c| c.id == giant));
    assert!(g.battlefield_find(bears).is_some(), "the old prisoner came back");
    let destroy = crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::ExactObjects(vec![limo]) };
    g.resolve_effect(&destroy, &crabomination::game::effects::EffectContext::for_spell(1, None, 0, 0)).expect("destroy");
    assert!(g.battlefield_find(giant).is_some(), "freed when the Limousine left");
}

fn activate(g: &mut GameState, id: CardId, ability_index: usize, target: Option<Target>) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index, target, additional_targets: vec![], x_value: None, mode: None })
        .expect("activate");
    drain_stack(g);
}

/// CR 702.167 / 712 — Tetzin's mill trigger hears double-faced artifacts only
/// (Sol Ring isn't one); crafting with six graveyard artifacts makes The
/// Golden-Gear Colossus, whose entry transforms another double-faced artifact
/// you control (Chalice of Life) and makes two Gnomes.
#[test]
fn cr_702_167_tetzin_crafts_the_colossus_which_flips_a_chalice() {
    let mut g = pod(2);
    g.players[0].library.clear();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::ornithopter());
    }
    let tetzin = g.add_card_to_hand(0, catalog::tetzin_gnome_champion());
    cast_x(&mut g, 0, tetzin, None, None);
    assert_eq!(g.players[0].graveyard.len(), 2, "three milled, one artifact taken");
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    let gy = g.players[0].graveyard.len();
    cast_x(&mut g, 0, ring, None, None);
    assert_eq!(g.players[0].graveyard.len(), gy, "Sol Ring isn't double-faced");
    let chalice = g.add_card_to_battlefield(0, catalog::chalice_of_life());
    for _ in 0..4 {
        g.add_card_to_graveyard(0, catalog::ornithopter());
    }
    activate(&mut g, tetzin, 0, None);
    let colossus = g.battlefield_find(tetzin).expect("returned transformed");
    assert_eq!(colossus.definition.name, "The Golden-Gear Colossus");
    assert_eq!(g.battlefield_find(chalice).unwrap().definition.name, "Chalice of Death");
    let gnomes = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Gnome").count();
    assert_eq!(gnomes, 2);
}

/// Chalice of Life transforms only once its controller is 10 above their
/// starting life total (40 in Commander, CR 903.7), checked after the gain.
#[test]
fn chalice_of_life_transforms_ten_over_the_starting_life() {
    let mut g = pod(2);
    let chalice = g.add_card_to_battlefield(0, catalog::chalice_of_life());
    let start = g.players[0].life;
    g.players[0].life = start + 8;
    activate(&mut g, chalice, 0, None);
    assert_eq!(g.battlefield_find(chalice).unwrap().definition.name, "Chalice of Life", "start + 9");
    g.battlefield_find_mut(chalice).unwrap().tapped = false;
    activate(&mut g, chalice, 0, None);
    assert_eq!(g.battlefield_find(chalice).unwrap().definition.name, "Chalice of Death", "start + 10");
    g.battlefield_find_mut(chalice).unwrap().tapped = false;
    let life = g.players[1].life;
    activate(&mut g, chalice, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 5);
}

/// CR 702.167a "craft with one or more": Sunbird Standard exiles every card
/// in the graveyard; the Effigy counts the colours among them (red, blue) for
/// its P/T and taps for one mana of each.
#[test]
fn cr_702_167_sunbird_effigy_reads_the_colours_it_was_crafted_with() {
    let mut g = pod(2);
    let sunbird = g.add_card_to_battlefield(0, catalog::sunbird_standard());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::opt());
    g.add_card_to_graveyard(0, catalog::shock());
    activate(&mut g, sunbird, 1, None);
    assert!(g.players[0].graveyard.is_empty(), "every graveyard card used to craft it");
    let cp = g.computed_permanent(sunbird).unwrap();
    assert_eq!(cp.def.name, "Sunbird Effigy");
    assert_eq!((cp.power, cp.toughness), (2, 2));
    g.players[0].mana_pool.empty();
    g.clear_sickness(sunbird);
    g.perform_action(GameAction::ActivateAbility { card_id: sunbird, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("tap for mana");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1);
}

/// Unstable Glyphbridge: its caster chooses one creature with power 2 or less
/// for each player and destroys the rest (Hill Giant is never a keeper).
#[test]
fn unstable_glyphbridge_keeps_one_small_creature_a_player() {
    let mut g = pod(3);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let theirs = g.add_card_to_battlefield(1, catalog::savannah_lions());
    let other = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let other2 = g.add_card_to_battlefield(2, catalog::savannah_lions());
    let bridge = g.add_card_to_hand(0, catalog::unstable_glyphbridge());
    cast_x(&mut g, 0, bridge, None, None);
    assert!(g.battlefield_find(mine).is_some());
    assert!(g.battlefield_find(giant).is_none(), "power 3 can't be chosen");
    assert!(g.battlefield_find(theirs).is_some(), "seat 1's only small creature");
    assert_eq!(
        [other, other2].iter().filter(|id| g.battlefield_find(**id).is_some()).count(),
        1,
        "one of seat 2's two"
    );
}

/// CR 508.1 — Sandswirl Wanderglyph: an opponent who casts a spell on their
/// turn can't attack its controller or their planeswalker that turn (another
/// opponent is fine), and one who attacked it can't cast spells after.
#[test]
fn cr_508_1_sandswirl_wanderglyph_splits_casting_from_attacking() {
    let mut g = pod(3);
    let glyph = g.add_card_to_battlefield(0, catalog::unstable_glyphbridge());
    g.transform_permanent(glyph, &mut vec![]);
    assert_eq!(g.battlefield_find(glyph).unwrap().definition.name, "Sandswirl Wanderglyph");
    let jace = g.add_card_to_battlefield(0, catalog::jace_beleren());
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    let opt = g.add_card_to_hand(1, catalog::opt());
    cast_x(&mut g, 1, opt, None, None);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    for target in [AttackTarget::Player(0), AttackTarget::Planeswalker(jace)] {
        let swing = GameAction::DeclareAttackers(vec![Attack { attacker: bears, target }]);
        assert!(g.perform_action(swing).is_err(), "cast this turn: can't attack seat 0 or its walker");
    }
    attack(&mut g, 1, &[(bears, 2)]);

    let mut g = pod(3);
    let glyph = g.add_card_to_battlefield(0, catalog::unstable_glyphbridge());
    g.transform_permanent(glyph, &mut vec![]);
    let bears = ready(&mut g, 2, catalog::grizzly_bears());
    attack(&mut g, 2, &[(bears, 0)]);
    g.step = TurnStep::PostCombatMain;
    flood(&mut g, 2);
    let opt = g.add_card_to_hand(2, catalog::opt());
    g.priority.player_with_priority = 2;
    let cast = GameAction::CastSpell { card_id: opt, target: None, additional_targets: vec![], mode: None, x_value: None };
    assert!(g.perform_action(cast).is_err(), "attacked seat 0 this turn: no spells");
}

fn lore(g: &mut GameState, saga: CardId) {
    let add = crabomination::effect::Effect::AddCounter {
        what: crabomination::effect::Selector::ExactObjects(vec![saga]),
        kind: CounterType::Lore,
        amount: crabomination::card::Value::ONE,
    };
    let evs = g.resolve_effect(&add, &crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

/// CR 714.2c — Storyweave's two lore counters take Teachings of the Kirin
/// from I to III, so chapters II and III both trigger; the Orochi the third
/// chapter returns is the next enchantment creature to enter this turn, and
/// enters with Storyweave's two extra +1/+1 counters (CR 614.1c).
#[test]
fn cr_714_2c_storyweave_skips_teachings_to_its_orochi_with_counters() {
    let mut g = pod(2);
    g.players[0].library.clear();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let saga = g.add_card_to_battlefield(0, catalog::teachings_of_the_kirin());
    lore(&mut g, saga);
    assert_eq!(g.players[0].graveyard.len(), 3, "chapter I milled three");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Spirit"), "and made a Spirit");
    let weave = g.add_card_to_hand(0, catalog::storyweave());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: weave,
        target: Some(Target::Permanent(saga)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let orochi = g.battlefield_find(saga).expect("returned transformed");
    assert_eq!(orochi.definition.name, "Kirin-Touched Orochi");
    assert!(orochi.counter_count(CounterType::PlusOnePlusOne) >= 2, "Storyweave's bonus");
}

/// Kirin-Touched Orochi's attack: exiling a creature card from a graveyard
/// is the reflexive trigger's gate (CR 603.12) — a Spirit follows it.
#[test]
fn kirin_touched_orochi_exiles_then_makes_a_spirit() {
    let mut g = pod(2);
    let saga = ready(&mut g, 0, catalog::teachings_of_the_kirin());
    g.transform_permanent(saga, &mut vec![]);
    let bears = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    attack(&mut g, 0, &[(saga, 1)]);
    assert!(g.exile.iter().any(|c| c.id == bears));
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Spirit"));
}

/// The Lemures' {0}: -1/-0 and a keyword until end of turn, once per
/// activation (Loot, the Anomaly's pair).
#[test]
fn the_lemures_trade_power_for_evasion() {
    for (def, kw) in [
        (catalog::hyalopterous_lemure(), Keyword::Flying),
        (catalog::viscid_lemures(), Keyword::Landwalk(crabomination::card::LandType::Swamp)),
    ] {
        let mut g = pod(2);
        let id = ready(&mut g, 0, def);
        activate(&mut g, id, 0, None);
        activate(&mut g, id, 0, None);
        assert_eq!(g.battlefield_find(id).unwrap().power(), 2);
        assert!(g.permanent_has_keyword(id, &kw));
    }
}

/// Ray of Enfeeblement is -4/-1, or -4/-4 against a white creature.
#[test]
fn ray_of_enfeeblement_is_harsher_on_white() {
    let mut g = pod(2);
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let ray = g.add_card_to_hand(0, catalog::ray_of_enfeeblement());
    cast_x(&mut g, 0, ray, Some(Target::Permanent(giant)), None);
    let c = g.battlefield_find(giant).unwrap();
    assert_eq!((c.power(), c.toughness()), (-1, 2));
    let mut g = pod(2);
    let lions = g.add_card_to_battlefield(1, catalog::serra_angel());
    let ray = g.add_card_to_hand(0, catalog::ray_of_enfeeblement());
    cast_x(&mut g, 0, ray, Some(Target::Permanent(lions)), None);
    assert!(g.battlefield_find(lions).is_none(), "a white 4/4 gets -4/-4 and dies");
}

/// Paired Tactician grows only when another Warrior attacks beside it.
#[test]
fn paired_tactician_needs_a_second_warrior() {
    let mut g = pod(2);
    let pt = ready(&mut g, 0, catalog::paired_tactician());
    attack(&mut g, 0, &[(pt, 1)]);
    assert_eq!(g.battlefield_find(pt).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    let mut g = pod(2);
    let pt = ready(&mut g, 0, catalog::paired_tactician());
    let other = ready(&mut g, 0, catalog::paired_tactician());
    attack(&mut g, 0, &[(pt, 1), (other, 1)]);
    assert_eq!(g.battlefield_find(pt).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Resplendent Marshal's ETB exiles a Warrior card; each other creature that
/// shares a type with it (the Tactician, a Warrior) gets a counter — the
/// Bear doesn't.
#[test]
fn resplendent_marshal_counts_types_shared_with_the_exiled_card() {
    let mut g = pod(2);
    let pt = ready(&mut g, 0, catalog::paired_tactician());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let gone = g.add_card_to_graveyard(0, catalog::paired_tactician());
    let marshal = g.add_card_to_hand(0, catalog::resplendent_marshal());
    cast_x(&mut g, 0, marshal, None, None);
    assert!(g.exile.iter().any(|c| c.id == gone));
    assert_eq!(g.battlefield_find(pt).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!(g.battlefield_find(marshal).unwrap().counter_count(CounterType::PlusOnePlusOne), 0, "other than itself");
}

/// Rush of Battle: +2/+1 to the team, lifelink to its Warriors only.
#[test]
fn rush_of_battle_gives_warriors_lifelink() {
    let mut g = pod(2);
    let pt = ready(&mut g, 0, catalog::paired_tactician());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let rush = g.add_card_to_hand(0, catalog::rush_of_battle());
    cast_x(&mut g, 0, rush, None, None);
    assert_eq!(g.battlefield_find(bear).unwrap().power(), 4);
    assert!(g.permanent_has_keyword(pt, &Keyword::Lifelink));
    assert!(!g.permanent_has_keyword(bear, &Keyword::Lifelink));
}

/// Reluctant Dounguard enters as a 2/2 and sheds a -1/-1 counter for each
/// other creature that enters under its controller — never past zero.
#[test]
fn reluctant_dounguard_warms_up_as_creatures_join() {
    let mut g = pod(2);
    let dg = g.add_card_to_hand(0, catalog::reluctant_dounguard());
    cast_x(&mut g, 0, dg, None, None);
    assert_eq!(g.battlefield_find(dg).unwrap().power(), 2);
    for _ in 0..3 {
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        cast_x(&mut g, 0, bear, None, None);
    }
    let c = g.battlefield_find(dg).unwrap();
    assert_eq!((c.power(), c.toughness()), (4, 4));
    assert_eq!(c.counter_count(CounterType::MinusOneMinusOne), 0);
}

/// Brawn's power-up (CR 702.193): a +1/+1 counter per card in hand, and only
/// once.
#[test]
fn cr_702_193_brawn_powers_up_by_hand_size() {
    let mut g = pod(2);
    let brawn = g.add_card_to_hand(0, catalog::brawn_amadeus_cho());
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::island());
    }
    cast_x(&mut g, 0, brawn, None, None);
    assert_eq!(g.players[0].hand.len(), 4, "three Islands and the ETB draw");
    activate(&mut g, brawn, 0, None);
    assert_eq!(g.battlefield_find(brawn).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
    flood(&mut g, 0);
    let again = GameAction::ActivateAbility { card_id: brawn, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None };
    assert!(g.perform_action(again).is_err(), "activate only once");
}

/// Oko, Lorwyn Liege pays {G} at the start of the first main phase to turn
/// into Oko, Shadowmoor Scion; the Scion's −6 emblem pumps only the chosen
/// creature type (the Bears), written in as it's made (CR 114.4).
#[test]
fn oko_flips_and_its_emblem_keeps_the_chosen_type() {
    let mut g = pod(2);
    let oko = g.add_card_to_battlefield(0, catalog::oko_lorwyn_liege());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let bear2 = ready(&mut g, 0, catalog::grizzly_bears());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    flood(&mut g, 0);
    g.step = TurnStep::PreCombatMain;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(oko).unwrap().definition.name, "Oko, Shadowmoor Scion");
    g.battlefield_find_mut(oko).unwrap().counters.insert(CounterType::Loyalty, 6);
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(crabomination::card::CreatureType::Bear)]));
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: oko, ability_index: 2, target: None, x_value: None })
        .expect("-6");
    drain_stack(&mut g);
    let _ = bear2;
    assert_eq!(g.computed_permanent(bear).unwrap().power, 5, "Bears were chosen");
    assert!(g.permanent_has_keyword(bear, &Keyword::Hexproof));
    assert_eq!(g.computed_permanent(giant).unwrap().power, 3);
}

/// Wasp shrinks a 2/2 to -1/2 as she attacks, then draws for each creature
/// with power below zero.
#[test]
fn wasp_draws_for_each_negative_power_creature() {
    let mut g = pod(2);
    let wasp = ready(&mut g, 0, catalog::wasp_shrinking_savior());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    attack(&mut g, 0, &[(wasp, 1)]);
    assert_eq!(g.computed_permanent(bear).unwrap().power, -1);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Quantum Reduction: cast at instant speed only by teamwork (tapping
/// creatures with total power 2), it leaves a Serra Angel at -1/4 without
/// flying (CR 613.1f).
#[test]
fn quantum_reduction_flashes_in_by_teamwork() {
    let mut g = pod(2);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let qr = g.add_card_to_hand(0, catalog::quantum_reduction());
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    let cast = || GameAction::CastSpell {
        card_id: qr,
        target: Some(Target::Permanent(angel)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    };
    assert!(g.perform_action(cast()).is_err(), "no creatures to tap: no flash");
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.perform_action(cast()).expect("teamwork flash");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "tapped for teamwork");
    let c = g.computed_permanent(angel).unwrap();
    assert_eq!(c.power, -1);
    assert!(!c.keywords().contains(&Keyword::Flying));
}

/// CR 707.10c — Zevlor's next qualifying spell is copied once per other
/// opponent: a Bolt at seat 1 also hits seats 2 and 3. A spell aimed at our
/// own creature doesn't qualify and leaves the watcher armed.
#[test]
fn cr_707_10c_zevlor_copies_a_single_target_spell_at_each_other_opponent() {
    let mut g = pod(4);
    let zev = ready(&mut g, 0, catalog::zevlor_elturel_exile());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, zev, 0, None);
    let giant = g.add_card_to_hand(0, catalog::giant_growth());
    cast_x(&mut g, 0, giant, Some(Target::Permanent(bear)), None);
    let life: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_x(&mut g, 0, bolt, Some(Target::Player(1)), None);
    for (seat, (p, was)) in g.players.iter().zip(&life).enumerate().skip(1) {
        assert_eq!(p.life, was - 3, "seat {seat}");
    }
    assert_eq!(g.players[0].life, life[0]);
}

/// Amulet of Safekeeping wards its controller for {1} (CR 702.21a): a Bolt
/// from a tapped-out opponent is countered. Creature tokens get -1/-0.
#[test]
fn amulet_of_safekeeping_taxes_spells_aimed_at_you() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::amulet_of_safekeeping());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "countered: no {{1}} to pay");
}

/// Uncomfortable Chill shrinks only the opponents' creatures, and draws.
#[test]
fn uncomfortable_chill_shrinks_the_other_side() {
    let mut g = pod(3);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(2, catalog::hill_giant());
    let chill = g.add_card_to_hand(0, catalog::uncomfortable_chill());
    let hand = g.players[0].hand.len() - 1;
    cast_x(&mut g, 0, chill, None, None);
    assert_eq!(g.computed_permanent(theirs).unwrap().power, 1);
    assert_eq!(g.computed_permanent(mine).unwrap().power, 2);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Justice (2004-10-04 ruling): a red spell dealing damage to two creatures at
/// once triggers it once for the total, aimed at the spell's controller; a
/// red creature's combat damage triggers it too.
#[test]
fn justice_punishes_red_damage_once_per_source_event() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::justice());
    g.add_card_to_battlefield(2, catalog::grizzly_bears());
    g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let clasm = g.add_card_to_hand(1, catalog::pyroclasm());
    let life = g.players[1].life;
    g.active_player_idx = 1;
    cast_x(&mut g, 1, clasm, None, None);
    assert_eq!(g.players[1].life, life - 4, "2 + 2 in one trigger");
    let goblin = ready(&mut g, 1, catalog::raging_goblin());
    let life = g.players[1].life;
    attack(&mut g, 1, &[(goblin, 2)]);
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, life - 1, "the Goblin's 1 combat damage");
}

/// Luminesce prevents a black or red source's damage everywhere; Surge of
/// Salvation only to its caster's creatures (CR 615.1).
#[test]
fn luminesce_and_surge_of_salvation_stop_black_and_red_damage() {
    let mut g = pod(2);
    let lum = g.add_card_to_hand(0, catalog::luminesce());
    cast_x(&mut g, 0, lum, None, None);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let life = g.players[0].life;
    cast_x(&mut g, 1, bolt, Some(Target::Player(0)), None);
    assert_eq!(g.players[0].life, life, "Luminesce: no red damage this turn");

    let mut g = pod(2);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let surge = g.add_card_to_hand(0, catalog::surge_of_salvation());
    cast_x(&mut g, 0, surge, None, None);
    assert!(g.permanent_has_keyword(mine, &Keyword::Hexproof));
    let clasm = g.add_card_to_hand(1, catalog::pyroclasm());
    g.active_player_idx = 1;
    cast_x(&mut g, 1, clasm, None, None);
    assert!(g.battlefield_find(mine).is_some(), "Surge covers the caster's creatures");
    assert!(g.battlefield_find(theirs).is_none());
}

/// Tivadar's ETB kills a Goblin; Lightwielder Paladin exiles a red permanent
/// of the player it hit.
#[test]
fn tivadar_and_lightwielder_paladin_hunt_red() {
    let mut g = pod(2);
    let goblin = g.add_card_to_battlefield(1, catalog::raging_goblin());
    let tiv = g.add_card_to_hand(0, catalog::tivadar_of_thorn());
    cast_x(&mut g, 0, tiv, None, None);
    assert!(g.battlefield_find(goblin).is_none());
    let mut g = pod(2);
    let paladin = ready(&mut g, 0, catalog::lightwielder_paladin());
    let piker = g.add_card_to_battlefield(1, catalog::goblin_piker());
    g.battlefield_find_mut(piker).unwrap().tapped = true;
    attack(&mut g, 0, &[(paladin, 1)]);
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(&mut g);
    }
    assert!(g.exile.iter().any(|c| c.id == piker), "the red permanent of the damaged player");
}

/// Myra the Magnificent (CR 717): an instant cast from hand opens an
/// Attraction; her {X}, {T} exiles Divination onto it with a midway counter,
/// and visiting it later copies Divination for free.
#[test]
fn myra_puts_a_midway_spell_on_an_attraction() {
    let mut g = pod(2);
    let myra = ready(&mut g, 0, catalog::myra_the_magnificent());
    let id = g.next_id();
    g.players[0].attraction_deck.push(crabomination::card::CardInstance::new(id, catalog::balloon_stand(), 0));
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast_x(&mut g, 0, opt, None, None);
    let stand = g.battlefield.iter().find(|c| c.definition.name == "Balloon Stand").map(|c| c.id).expect("opened");
    let div = g.add_card_to_graveyard(0, catalog::divination());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: myra,
        ability_index: 0,
        target: Some(Target::Permanent(div)),
        additional_targets: vec![],
        x_value: Some(3),
        mode: None,
    })
    .expect("X = 3");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == div));
    assert_eq!(g.battlefield_find(stand).unwrap().counter_count(CounterType::Midway), 1);
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(2)]));
    let roll = crabomination::effect::Effect::RollToVisitAttractions;
    let evs = g.resolve_effect(&roll, &crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.players[0].hand.len() >= hand + 2, "the copied Divination resolved");
}

/// Abzan Skycaptain's death bolsters 2 (CR 701.39): the least-toughness
/// creature its controller has left gets two +1/+1 counters.
#[test]
fn cr_701_39_abzan_skycaptain_bolsters_on_death() {
    let mut g = pod(2);
    let sky = g.add_card_to_battlefield(0, catalog::abzan_skycaptain());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_x(&mut g, 1, bolt, Some(Target::Permanent(sky)), None);
    assert!(g.battlefield_find(sky).is_none());
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(g.battlefield_find(giant).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Ampryn Tactician's ETB anthem lasts the turn and covers only its side.
#[test]
fn ampryn_tactician_pumps_the_team_on_entry() {
    let mut g = pod(2);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let tac = g.add_card_to_hand(0, catalog::ampryn_tactician());
    cast_x(&mut g, 0, tac, None, None);
    assert_eq!(g.computed_permanent(mine).unwrap().power, 3);
    assert_eq!(g.computed_permanent(tac).unwrap().power, 4);
    assert_eq!(g.computed_permanent(theirs).unwrap().power, 2);
}

/// Drogskol Infantry disturbs from the graveyard into Drogskol Armaments
/// (CR 702.146): enchanted creature gets +2/+2.
#[test]
fn cr_702_146_drogskol_infantry_disturbs_into_armaments() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let inf = g.add_card_to_graveyard(0, catalog::drogskol_infantry());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastDisturb {
        card_id: inf,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
    })
    .expect("disturb");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bears).unwrap().power, 4);
}

/// Scrapwork Cohort makes an artifact Soldier on entry, and unearths
/// (CR 702.84) for {2}{W} with haste.
#[test]
fn cr_702_84_scrapwork_cohort_unearths_and_makes_a_soldier() {
    let mut g = pod(2);
    let co = g.add_card_to_graveyard(0, catalog::scrapwork_cohort());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: co,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("unearth");
    drain_stack(&mut g);
    assert!(g.battlefield_find(co).is_some());
    assert!(g.permanent_has_keyword(co, &Keyword::Haste));
    let soldiers = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Soldier" && c.definition.card_types.contains(&CardType::Artifact))
        .count();
    assert_eq!(soldiers, 1);
}

/// Sergeant-at-Arms (CR 702.33 kicker, 603.4 intervening if): two Soldiers
/// only when kicked.
#[test]
fn cr_702_33_sergeant_at_arms_makes_soldiers_only_when_kicked() {
    for (kick, want) in [(false, 0), (true, 2)] {
        let mut g = pod(2);
        let sgt = g.add_card_to_hand(0, catalog::sergeant_at_arms());
        flood(&mut g, 0);
        let action = if kick {
            GameAction::CastSpellKicked { card_id: sgt, target: None, additional_targets: vec![], mode: None, x_value: None }
        } else {
            GameAction::CastSpell { card_id: sgt, target: None, additional_targets: vec![], mode: None, x_value: None }
        };
        g.perform_action(action).expect("cast");
        drain_stack(&mut g);
        let n = g.battlefield.iter().filter(|c| c.definition.name == "Soldier").count();
        assert_eq!(n, want, "kicked: {kick}");
    }
}

/// Repel the Darkness taps two creatures and draws.
#[test]
fn repel_the_darkness_taps_two_and_draws() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::hill_giant());
    let rep = g.add_card_to_hand(0, catalog::repel_the_darkness());
    let hand = g.players[0].hand.len() - 1;
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: rep,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Herald of the Sun counters another flier — and Lightwalker, given a
/// +1/+1 counter some other way, starts flying.
#[test]
fn herald_of_the_sun_feeds_fliers_and_lightwalker_takes_off() {
    let mut g = pod(2);
    let herald = g.add_card_to_battlefield(0, catalog::herald_of_the_sun());
    let sky = g.add_card_to_battlefield(0, catalog::abzan_skycaptain());
    let walker = g.add_card_to_battlefield(0, catalog::lightwalker());
    flood(&mut g, 0);
    let act = |t| GameAction::ActivateAbility {
        card_id: herald,
        ability_index: 0,
        target: Some(Target::Permanent(t)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    };
    assert!(g.perform_action(act(walker)).is_err(), "Lightwalker isn't flying yet");
    assert!(g.perform_action(act(herald)).is_err(), "another creature");
    g.perform_action(act(sky)).expect("a flier");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(sky).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(!g.permanent_has_keyword(walker, &Keyword::Flying));
    g.battlefield_find_mut(walker).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert!(g.permanent_has_keyword(walker, &Keyword::Flying));
}

/// Steadfast Sentry's death leaves a +1/+1 counter behind.
#[test]
fn steadfast_sentry_leaves_a_counter() {
    let mut g = pod(2);
    let sentry = g.add_card_to_battlefield(0, catalog::steadfast_sentry());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.permanent_has_keyword(sentry, &Keyword::Vigilance));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_x(&mut g, 1, bolt, Some(Target::Permanent(sentry)), None);
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Recumbent Bliss pacifies (CR 508.1c / 509.1b) and offers 1 life each
/// upkeep.
#[test]
fn recumbent_bliss_pacifies() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let bliss = g.add_card_to_hand(0, catalog::recumbent_bliss());
    cast_x(&mut g, 0, bliss, Some(Target::Permanent(giant)), None);
    assert!(g.permanent_has_keyword(giant, &Keyword::CantAttack));
    assert!(g.permanent_has_keyword(giant, &Keyword::CantBlock));
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    let swing = GameAction::DeclareAttackers(vec![Attack { attacker: giant, target: AttackTarget::Player(0) }]);
    assert!(g.perform_action(swing).is_err());
}

/// Fall to Earth exiles a creature and everyone gains 3.
#[test]
fn fall_to_earth_exiles_and_heals_the_table() {
    let mut g = pod(3);
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let fall = g.add_card_to_hand(0, catalog::fall_to_earth());
    let life: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    cast_x(&mut g, 0, fall, Some(Target::Permanent(giant)), None);
    assert!(g.exile.iter().any(|c| c.id == giant));
    for (i, p) in g.players.iter().enumerate() {
        assert_eq!(p.life, life[i] + 3);
    }
}

/// Protective Response needs an attacking or blocking target (CR 506.4).
#[test]
fn protective_response_kills_an_attacker() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let resp = g.add_card_to_hand(0, catalog::protective_response());
    flood(&mut g, 0);
    let cast = GameAction::CastSpell {
        card_id: resp,
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    };
    assert!(g.perform_action(cast.clone()).is_err(), "not attacking yet");
    attack(&mut g, 1, &[(giant, 0)]);
    cast_x(&mut g, 0, resp, Some(Target::Permanent(giant)), None);
    assert!(g.battlefield_find(giant).is_none());
}

/// Blood-Chin Fanatic drains for the sacrificed Warrior's power (CR 602.2b:
/// the sacrifice is a cost; a non-Warrior can't pay it), and Chief of the
/// Scale toughens only its other Warriors.
#[test]
fn cr_602_2b_blood_chin_fanatic_drains_for_a_sacrificed_warrior() {
    let mut g = pod(2);
    let fan = g.add_card_to_battlefield(0, catalog::blood_chin_fanatic());
    let chief = g.add_card_to_battlefield(0, catalog::chief_of_the_scale());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(fan).unwrap().toughness, 4, "another Warrior");
    assert_eq!(g.computed_permanent(chief).unwrap().toughness, 3, "not itself");
    assert_eq!(g.computed_permanent(bears).unwrap().toughness, 2, "not a Warrior");
    let (mine, theirs) = (g.players[0].life, g.players[1].life);
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: fan,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("sacrifice the Chief");
    drain_stack(&mut g);
    assert!(g.battlefield_find(chief).is_none() && g.battlefield_find(bears).is_some());
    assert_eq!(g.players[1].life, theirs - 2);
    assert_eq!(g.players[0].life, mine + 2);
}

/// Fearless Skald backs up another creature (CR 702.164): a counter and
/// double strike until end of turn.
#[test]
fn cr_702_164_fearless_skald_backs_up_with_double_strike() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let skald = g.add_card_to_hand(0, catalog::fearless_skald());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bears))]));
    cast_x(&mut g, 0, skald, None, None);
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.permanent_has_keyword(bears, &Keyword::DoubleStrike));
    assert!(g.permanent_has_keyword(skald, &Keyword::DoubleStrike));
}

/// Chandra's Outrage hits the creature for 4 and its controller for 2.
#[test]
fn chandras_outrage_burns_creature_and_controller() {
    let mut g = pod(3);
    let giant = g.add_card_to_battlefield(2, catalog::hill_giant());
    let out = g.add_card_to_hand(0, catalog::chandras_outrage());
    let life = g.players[2].life;
    cast_x(&mut g, 0, out, Some(Target::Permanent(giant)), None);
    assert!(g.battlefield_find(giant).is_none());
    assert_eq!(g.players[2].life, life - 2);
}

/// Gustha's Scepter stashes a card face down and fetches it back; destroyed
/// (CR 800.4: leaving the battlefield loses control), its stash goes to the
/// graveyard.
#[test]
fn cr_800_4_gusthas_scepter_stash_bins_when_control_is_lost() {
    let mut g = pod(2);
    let sc = ready(&mut g, 0, catalog::gusthas_scepter());
    g.players[0].hand.clear();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let act = |i| GameAction::ActivateAbility {
        card_id: sc,
        ability_index: i,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    };
    g.perform_action(act(0)).expect("stash");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt && c.face_down));
    g.battlefield_find_mut(sc).unwrap().tapped = false;
    g.perform_action(act(1)).expect("fetch");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt));
    g.battlefield_find_mut(sc).unwrap().tapped = false;
    g.perform_action(act(0)).expect("stash again");
    drain_stack(&mut g);
    let shatter = g.add_card_to_hand(1, catalog::shatter());
    cast_x(&mut g, 1, shatter, Some(Target::Permanent(sc)), None);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt), "binned with the Scepter gone");
}

/// Draconian Gate-Bot opens an Attraction (CR 717.4) or destroys one, which
/// goes to its owner's junkyard (CR 717.6).
#[test]
fn cr_717_6_draconian_gate_bot_opens_or_junks_an_attraction() {
    let mut g = pod(2);
    g.seat_attraction_deck(0, vec![catalog::information_booth()]);
    let bot = g.add_card_to_hand(0, catalog::draconian_gate_bot());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    cast_x(&mut g, 0, bot, None, None);
    let booth = g.battlefield.iter().find(|c| c.definition.name == "Information Booth").map(|c| c.id).expect("opened");
    let bot2 = g.add_card_to_hand(1, catalog::draconian_gate_bot());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1), DecisionAnswer::Target(Target::Permanent(booth))]));
    g.active_player_idx = 1;
    cast_x(&mut g, 1, bot2, None, None);
    assert!(g.battlefield_find(booth).is_none());
    assert!(g.players[0].attraction_junkyard.iter().any(|c| c.id == booth));
}

/// "You may play that card this turn. If you do, it enters tapped" (CR
/// 614.1c, Kaldring's grant): a land played and a creature cast from the
/// graveyard through an enters-tapped grant both enter tapped.
#[test]
fn cr_614_1c_an_enters_tapped_grant_taps_the_land_and_the_creature() {
    use crabomination::effect::{Effect, Selector};
    let mut g = pod(2);
    let land = g.add_card_to_graveyard(0, catalog::snow_covered_forest());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    for id in [land, bears] {
        let grant = Effect::Seq(vec![
            Effect::GrantMayPlay {
                what: Selector::Target(0),
                duration: crabomination::card::MayPlayDuration::EndOfThisTurn,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            },
            Effect::EntersTappedAfterGrantedPlay { what: Selector::Target(0) },
        ]);
        let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(id)), 0, 0);
        g.resolve_effect(&grant, &ctx).unwrap();
    }
    g.perform_action(GameAction::PlayLandFromGraveyard(land)).expect("play");
    assert!(g.battlefield_find(land).unwrap().tapped);
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from the graveyard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).unwrap().tapped);
}

fn snow_forests(g: &mut GameState, seat: usize, n: usize) -> Vec<CardId> {
    (0..n).map(|_| g.add_card_to_battlefield(seat, catalog::snow_covered_forest())).collect()
}

/// Boreal Outrider (CR 106.3): a creature spell paid with green snow mana
/// enters with a +1/+1 counter; one paid from plain Forests doesn't.
#[test]
fn cr_106_3_boreal_outrider_counts_snow_of_the_spells_colour() {
    for (snowy, want) in [(true, 1), (false, 0)] {
        let mut g = pod(2);
        g.add_card_to_battlefield(0, catalog::boreal_outrider());
        if snowy {
            snow_forests(&mut g, 0, 2);
        } else {
            for _ in 0..2 {
                g.add_card_to_battlefield(0, catalog::forest());
            }
        }
        let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.perform_action(GameAction::CastSpell { card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None })
            .expect("cast off the lands");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne), want, "snow: {snowy}");
    }
}

/// Jorn's attack untaps his controller's snow permanents; Kaldring (his
/// back) lets a snow land be played from the graveyard, tapped.
#[test]
fn jorn_untaps_snow_and_kaldring_replays_it_tapped() {
    let mut g = pod(2);
    let jorn = ready(&mut g, 0, catalog::jorn_god_of_winter());
    let lands = snow_forests(&mut g, 0, 2);
    let plain = g.add_card_to_battlefield(0, catalog::forest());
    for id in lands.iter().chain([&plain]) {
        g.battlefield_find_mut(*id).unwrap().tapped = true;
    }
    attack(&mut g, 0, &[(jorn, 1)]);
    assert!(lands.iter().all(|id| !g.battlefield_find(*id).unwrap().tapped));
    assert!(g.battlefield_find(plain).unwrap().tapped, "not snow");

    let mut g = pod(2);
    let kaldring = *catalog::jorn_god_of_winter().back_face.unwrap();
    let staff = ready(&mut g, 0, kaldring);
    let land = g.add_card_to_graveyard(0, catalog::snow_covered_forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: staff,
        ability_index: 0,
        target: Some(Target::Permanent(land)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("Kaldring");
    drain_stack(&mut g);
    g.perform_action(GameAction::PlayLandFromGraveyard(land)).expect("play it");
    assert!(g.battlefield_find(land).unwrap().tapped);
}

/// Abominable Treefolk is as big as its controller's snow count and freezes
/// an opposing creature (CR 502.3: it skips its next untap).
#[test]
fn abominable_treefolk_counts_snow_and_freezes() {
    let mut g = pod(2);
    snow_forests(&mut g, 0, 3);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let tf = g.add_card_to_hand(0, catalog::abominable_treefolk());
    cast_x(&mut g, 0, tf, None, None);
    assert_eq!(g.computed_permanent(tf).unwrap().power, 4, "three Forests and itself");
    assert!(g.battlefield_find(bears).unwrap().tapped);
}

/// Conifer Wurm and Hailstorm Valkyrie pump off snow: the Wurm by the snow
/// count, the Valkyrie for {S}{S} (CR 107.4h — mana from snow sources only).
#[test]
fn cr_107_4h_conifer_wurm_and_hailstorm_valkyrie_pump_on_snow() {
    let mut g = pod(2);
    let wurm = ready(&mut g, 0, catalog::conifer_wurm());
    let valk = ready(&mut g, 0, catalog::hailstorm_valkyrie());
    snow_forests(&mut g, 0, 2);
    let act = |id| GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    };
    g.players[0].mana_pool.add(Color::Black, 2);
    assert!(g.perform_action(act(valk)).is_ok(), "the two snow Forests pay {{S}}{{S}}");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(valk).unwrap().power, 4);
    g.players[0].mana_pool.add(Color::Green, 4);
    g.perform_action(act(wurm)).expect("Wurm");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(wurm).unwrap().power, 8, "4 + four snow permanents: the Wurm, the Valkyrie and two Forests");
}

/// Spirit of the Aldergard fetches a snow land and grows with other snow.
#[test]
fn spirit_of_the_aldergard_fetches_snow_and_grows() {
    let mut g = pod(2);
    g.players[0].library.clear();
    let land = g.add_card_to_library(0, catalog::snow_covered_forest());
    g.add_card_to_library(0, catalog::forest());
    snow_forests(&mut g, 0, 2);
    let sp = g.add_card_to_hand(0, catalog::spirit_of_the_aldergard());
    cast_x(&mut g, 0, sp, None, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == land));
    assert_eq!(g.computed_permanent(sp).unwrap().power, 2);
}

/// Glittering Frost makes its land snow (CR 205.4) and adds a mana of any
/// colour when it's tapped; Rime Transfusion's bearer buys snow-only blocks.
#[test]
fn glittering_frost_and_rime_transfusion() {
    let mut g = pod(2);
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let frost = g.add_card_to_hand(0, catalog::glittering_frost());
    cast_x(&mut g, 0, frost, Some(Target::Permanent(forest)), None);
    assert!(
        g.computed_permanent(forest).unwrap().supertypes().contains(&crabomination::card::Supertype::Snow),
        "enchanted land is snow"
    );
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let rime = g.add_card_to_hand(0, catalog::rime_transfusion());
    cast_x(&mut g, 0, rime, Some(Target::Permanent(bears)), None);
    assert_eq!(g.computed_permanent(bears).unwrap().power, 4);
    g.players[0].mana_pool = Default::default();
    g.perform_action(GameAction::ActivateAbility {
        card_id: bears,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("{S} off the frosted Forest");
    drain_stack(&mut g);
    assert!(g.permanent_has_keyword(bears, &Keyword::CantBeBlockedExceptBy(Box::new(
        crabomination::card::SelectionRequirement::Creature
            .and(crabomination::card::SelectionRequirement::HasSupertype(crabomination::card::Supertype::Snow))
    ))));
    assert_eq!(g.players[0].mana_pool.total(), 1, "the Frost's extra mana floats");
}

/// The Three Seasons' chapter II returns snow permanents; chapter III
/// shuffles three cards of each graveyard back.
#[test]
fn the_three_seasons_recurs_snow_then_reshuffles_graveyards() {
    let mut g = pod(2);
    let saga = catalog::the_three_seasons();
    let a = g.add_card_to_graveyard(0, catalog::snow_covered_forest());
    let b = g.add_card_to_graveyard(0, catalog::boreal_outrider());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(a)), 0, 0);
    let mut ctx = ctx;
    ctx.targets.push(Target::Permanent(b));
    g.resolve_effect(&saga.saga_chapters[1].1, &ctx).unwrap();
    assert!(g.players[0].hand.iter().any(|c| c.id == a) && g.players[0].hand.iter().any(|c| c.id == b));
    for _ in 0..4 {
        g.add_card_to_graveyard(0, catalog::island());
        g.add_card_to_graveyard(1, catalog::island());
    }
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&saga.saga_chapters[2].1, &ctx).unwrap();
    assert_eq!((g.players[0].graveyard.len(), g.players[1].graveyard.len()), (1, 1));
}

/// Glacial Revelation keeps every snow permanent in the top six.
#[test]
fn glacial_revelation_keeps_the_snow() {
    let mut g = pod(2);
    g.players[0].library.clear();
    let keep: Vec<_> = (0..2).map(|_| g.add_card_to_library(0, catalog::snow_covered_forest())).collect();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let rev = g.add_card_to_hand(0, catalog::glacial_revelation());
    cast_x(&mut g, 0, rev, None, None);
    assert!(keep.iter().all(|id| g.players[0].hand.iter().any(|c| c.id == *id)));
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Island").count(), 4);
}
