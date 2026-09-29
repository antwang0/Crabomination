//! CR 707.10 — a copy of a spell is not cast. Magecraft says "whenever you
//! cast OR COPY an instant or sorcery spell", so it fires for the copy too
//! (`EventSpec::or_copy`, run by the dispatcher once per `SpellsCopied`),
//! while a plain "whenever you cast" trigger still does not.

use crabomination::catalog;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;
use crabomination::TurnStep;

/// Bolt, then Reverberate copying it: returns how many of `name` seat 0 has.
fn bolt_and_copy(listener: crabomination::card::CardDefinition, name: &str) -> usize {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, listener);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let rev = g.add_card_to_hand(0, catalog::reverberate());
    g.players[0].mana_pool.add(Color::Red, 3);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt");
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: rev,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Reverberate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 14, "the Bolt and its copy both hit");
    g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == name).count()
}

#[test]
fn cr_707_10_magecraft_fires_for_a_copy() {
    // Bolt cast, Reverberate cast, the copy: three.
    assert_eq!(bolt_and_copy(catalog::storm_kiln_artist(), "Treasure"), 3);
}

#[test]
fn cr_707_10_a_cast_trigger_ignores_the_copy() {
    // "Whenever you cast an instant or sorcery spell": the two casts only.
    assert_eq!(bolt_and_copy(catalog::young_pyromancer(), "Elemental"), 2);
}

/// CR 603.2 — "Whenever THIS creature becomes the target of a spell or
/// ability an opponent controls" (Mossdog): an opponent's Giant Growth on
/// another of your creatures doesn't grow it; one on Mossdog does.
#[test]
fn cr_603_2_mossdog_counts_only_itself_being_targeted() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let dog = g.add_card_to_battlefield(0, catalog::mossdog());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let counters = |g: &GameState| g.battlefield_find(dog).unwrap().counter_count(CounterType::PlusOnePlusOne);
    let growth_on = |g: &mut GameState, target| {
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.step = TurnStep::PreCombatMain;
        let spell = g.add_card_to_hand(1, catalog::giant_growth());
        g.players[1].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(target)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("Giant Growth");
        drain_stack(g);
    };
    growth_on(&mut g, bear);
    assert_eq!(counters(&g), 0, "another creature was targeted");
    growth_on(&mut g, dog);
    assert_eq!(counters(&g), 1, "Mossdog itself was targeted");
}

/// CR 305.1 / 601 — "you may CAST spells from among them" (Apex of Power)
/// is no land drop: an exiled Forest is marked cast-only, and playing it is
/// refused, while an exiled spell stays castable.
#[test]
fn cr_305_1_a_cast_permission_does_not_play_a_land() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let forest = g.add_card_to_library(0, catalog::forest());
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    let apex = catalog::apex_of_power();
    g.resolve_effect(&apex.effect, &crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0))
        .expect("resolve");
    let perm = |id| g.exile.iter().find(|c| c.id == id).and_then(|c| c.may_play_until);
    assert!(perm(forest).is_some_and(|p| p.cast_only), "the Forest is cast-only");
    assert!(perm(bolt).is_some(), "the Bolt is castable");
    assert!(g.perform_action(GameAction::PlayLand(forest)).is_err(), "no land drop from a cast permission");
}

/// CR 500.4 — mana kept "as steps and phases end" (Savage Ventmaw) survives a step
/// only while unspent: three kept, two spent, one crosses the step (the
/// whole three used to be re-seeded at every step change), and none of it
/// comes back after being spent too.
#[test]
fn cr_500_4_kept_mana_is_not_restored_once_spent() {
    use crabomination::effect::{Effect, PlayerRef, Value};
    use crabomination::mana::{cost, generic, Color};
    let mut g = two_player_game();
    let keep = Effect::AddManaKeptThisTurnCount { who: PlayerRef::You, color: Color::Green, amount: Value::Const(3) };
    g.resolve_effect(&keep, &crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0)).unwrap();
    g.players[0].mana_pool.pay(&cost(&[generic(2)])).expect("pay two");
    g.empty_mana_pools();
    assert_eq!(g.players[0].mana_pool.total(), 1, "only the unspent one kept");
    g.players[0].mana_pool.pay(&cost(&[generic(1)])).expect("pay the last");
    g.empty_mana_pools();
    assert_eq!(g.players[0].mana_pool.total(), 0);
}

/// Klauth's kept mana is "only to cast spells" (`SpendRestriction::SpellsOnly`):
/// it floats as restricted mana, and an unspent pip survives the step.
#[test]
fn cr_106_6_klauth_mana_is_for_spells_and_kept() {
    use crabomination::effect::{Effect, PlayerRef, Value};
    let mut g = two_player_game();
    let keep = Effect::AddManaKeptThisTurnAnyColors { who: PlayerRef::You, amount: Value::Const(2) };
    g.resolve_effect(&keep, &crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0)).unwrap();
    assert_eq!((g.players[0].mana_pool.total(), g.players[0].mana_pool.restricted_total()), (0, 2));
    g.empty_mana_pools();
    assert_eq!(g.players[0].mana_pool.restricted_total(), 2, "kept across the step");
}

/// CR 614.9 — Heroic Sacrifice redirects damage to "you and CREATURES you
/// control": a Bolt at you goes to the chosen creature, a Bolt at your
/// planeswalker doesn't (Gideon's Sacrifice's "permanents" would take it).
#[test]
fn cr_614_9_heroic_sacrifice_covers_you_and_creatures_only() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let pw = g.add_card_to_battlefield(0, catalog::professor_onyx());
    let hs = catalog::heroic_sacrifice();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(wurm)), 0, 0);
    g.resolve_effect(&hs.effect, &ctx).expect("resolve");
    let bolt_at = |g: &mut GameState, target| {
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.step = TurnStep::PreCombatMain;
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(target), additional_targets: vec![], mode: None, x_value: None })
            .expect("Bolt");
        drain_stack(g);
    };
    let (life, loyalty) = (g.players[0].life, g.battlefield_find(pw).unwrap().counter_count(CounterType::Loyalty));
    bolt_at(&mut g, Target::Player(0));
    assert_eq!(g.players[0].life, life, "redirected to the Wurm");
    assert_eq!(g.battlefield_find(wurm).unwrap().damage, 3);
    bolt_at(&mut g, Target::Permanent(pw));
    assert_eq!(g.battlefield_find(pw).unwrap().counter_count(CounterType::Loyalty), loyalty - 3, "a planeswalker isn't covered");
}

/// CR 106.6 — "Spend this mana only to …" riders are restricted mana:
/// Sliver Hive's colored mana (Sliver spells) and Cultivator Drone's {C}
/// (colorless spells and abilities) float restricted, not free.
#[test]
fn cr_106_6_printed_spend_restrictions_float_restricted() {
    for (def, ability) in [(catalog::sliver_hive(), 1usize), (catalog::cultivator_drone(), 0)] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        g.clear_sickness(id);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: ability,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("tap for mana");
        assert_eq!(g.players[0].mana_pool.restricted_total(), 1);
        assert_eq!(g.players[0].mana_pool.total(), 0);
    }
}
