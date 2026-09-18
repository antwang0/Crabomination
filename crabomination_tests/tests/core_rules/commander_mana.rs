//! Mana provenance in Commander — "when that mana is spent to …" riders
//! (CR 106, CR 903.4). Path of Ancestry's scry trigger and Opal Palace's
//! +1/+1 counters, plus the command-zone cast path's handling of
//! spend-restricted mana generally.

use crabomination::card::{
    CardDefinition, CardType, CounterType, CreatureType, Subtypes, Supertype,
};
use crabomination::catalog;
use crabomination::format::Format;
use crabomination::game::*;
use crabomination::game::types::StackItem;
use crabomination::mana::{Color, SpendRestriction, cost, g};

/// A free-to-name legendary Bear, so the shared-creature-type half of Path of
/// Ancestry has something to share with. Costs {G} so a rider actually has
/// mana to be spent on.
fn bear_commander() -> CardDefinition {
    CardDefinition {
        name: "Test Bear Commander",
        cost: cost(&[g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bear],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        ..Default::default()
    }
}

/// A Commander game with seat 0 led by `bear_commander`, seat 0 holding
/// priority in its own pre-combat main phase.
fn commander_game() -> (GameState, crabomination::card::CardId) {
    let mut g = game_with_format(Format::Commander, 2);
    let cmd = g.seat_commanders(0, vec![bear_commander()])[0];
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    (g, cmd)
}

// ── Path of Ancestry ──────────────────────────────────────────────────────

/// Path of Ancestry: the mana is spent on a creature spell that shares a
/// creature type with the commander, so its ability triggers (CR 603.2) and
/// the trigger goes on the stack *above* the spell it funded, resolving first
/// (CR 603.3).
#[test]
fn path_of_ancestry_triggers_on_a_shared_type_creature() {
    let (mut g, _cmd) = commander_game();
    // Grizzly Bears is a Bear, like the commander. {1}{G}.
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0]
        .mana_pool
        .add_restricted(Color::Green, 2, SpendRestriction::CommanderTypeScry);
    g.add_card_to_library(0, catalog::forest());
    g.perform_action(GameAction::CastSpell {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("rider mana is unrestricted and pays");

    assert_eq!(g.stack.len(), 2, "the spell and its funder's trigger");
    assert!(
        matches!(g.stack[0], StackItem::Spell { .. }),
        "the spell is underneath",
    );
    assert!(
        matches!(&g.stack[1], StackItem::Trigger { controller, .. } if *controller == 0),
        "the scry trigger resolves first",
    );
}

/// The same mana on a creature spell that shares *no* type with the commander
/// doesn't trigger — nothing extra reaches the stack.
#[test]
fn path_of_ancestry_is_silent_without_a_shared_type() {
    let (mut g, _cmd) = commander_game();
    // Llanowar Elves is an Elf Druid; the commander is a Bear.
    let elves = g.add_card_to_hand(0, catalog::llanowar_elves());
    g.players[0]
        .mana_pool
        .add_restricted(Color::Green, 1, SpendRestriction::CommanderTypeScry);
    g.perform_action(GameAction::CastSpell {
        card_id: elves,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    assert_eq!(g.stack.len(), 1, "no trigger for an unshared type");
}

/// The rider names a *creature* spell, so a noncreature spell funded by the
/// same mana never triggers.
#[test]
fn path_of_ancestry_is_silent_on_a_noncreature_spell() {
    let (mut g, _cmd) = commander_game();
    let bolt = g.add_card_to_hand(0, catalog::giant_growth());
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0]
        .mana_pool
        .add_restricted(Color::Green, 1, SpendRestriction::CommanderTypeScry);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    assert_eq!(g.stack.len(), 1, "an instant is not a creature spell");
}

/// The card itself: Path of Ancestry enters tapped and its ability taps for a
/// colour in the commander's identity (CR 903.4).
#[test]
fn path_of_ancestry_enters_tapped_and_taps_for_identity_mana() {
    let (mut g, _cmd) = commander_game();
    let land = g.add_card_to_hand(0, catalog::path_of_ancestry());
    g.perform_action(GameAction::PlayLand(land)).expect("play");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).unwrap().tapped, "enters tapped");

    g.battlefield_find_mut(land).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap for identity mana");
    // The commander's identity is mono-green, so that is the only legal pip.
    // The rider parks it in the restricted bucket, which the unrestricted
    // `amount` / `total` readers deliberately don't see.
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 0);
    assert_eq!(
        g.players[0].mana_pool.restricted_breakdown(),
        vec![("{G}".to_string(), 1, SpendRestriction::CommanderTypeScry)],
    );
}

// ── Opal Palace ───────────────────────────────────────────────────────────

/// Opal Palace: mana spent to cast your commander gives it one additional
/// +1/+1 counter per command-zone cast this game, the cast in progress
/// included (ruling 2020-11-10).
#[test]
fn opal_palace_counters_include_the_cast_in_progress() {
    let (mut g, cmd) = commander_game();
    g.players[0]
        .mana_pool
        .add_restricted(Color::Green, 1, SpendRestriction::CommanderCastCounters);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("first cast from the command zone");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(cmd).expect("commander resolved").counter_count(CounterType::PlusOnePlusOne),
        1,
        "first cast counts itself",
    );
}

/// The third cast from the command zone is worth three counters — and the
/// {4} of tax it now costs is itself payable from the rider's mana.
#[test]
fn opal_palace_counters_scale_with_the_cast_count() {
    let (mut g, cmd) = commander_game();
    g.commander_cast_count.insert(cmd, 2);
    // {G} printed + {4} tax (CR 903.8: {2} per prior cast).
    g.players[0]
        .mana_pool
        .add_restricted(Color::Green, 5, SpendRestriction::CommanderCastCounters);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("third cast");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(cmd).expect("commander resolved").counter_count(CounterType::PlusOnePlusOne),
        3,
    );
}

/// The rider names *your commander*: ordinary mana leaves it a plain 2/2, and
/// so does the rider's mana spent on something that isn't the commander.
#[test]
fn opal_palace_counters_only_the_commander_and_only_its_own_mana() {
    // Plain mana → no counters.
    let (mut g, cmd) = commander_game();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(cmd).unwrap().counter_count(CounterType::PlusOnePlusOne),
        0,
    );

    // Rider mana on a non-commander creature → no counters either.
    let (mut g, _cmd) = commander_game();
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0]
        .mana_pool
        .add_restricted(Color::Green, 2, SpendRestriction::CommanderCastCounters);
    g.perform_action(GameAction::CastSpell {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne),
        0,
    );
}

/// The card itself: Opal Palace's first ability is the plain `{T}: Add {C}`.
#[test]
fn opal_palace_taps_for_colorless() {
    let (mut g, _cmd) = commander_game();
    let land = g.add_card_to_battlefield(0, catalog::opal_palace());
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap for {C}");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1);
}

// ── The command-zone cast path and spend-restricted mana ──────────────────

/// CR 903.8's cast is a cast like any other: mana restricted to creature
/// spells may pay for a creature commander. Before the payment described what
/// it was funding, the command-zone path paid with an empty `SpellKind`, which
/// no restricted mana is allowed to fund — Cavern of Souls could not cast the
/// commander it named.
#[test]
fn command_zone_cast_accepts_mana_restricted_to_what_it_is() {
    let (mut g, cmd) = commander_game();
    g.players[0]
        .mana_pool
        .add_restricted(Color::Green, 1, SpendRestriction::CreatureOnly);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("creature-only mana funds a creature commander");
    assert_eq!(g.commander_cast_count.get(&cmd).copied(), Some(1));
}

/// …and only what it is: noncreature-only mana still can't.
#[test]
fn command_zone_cast_rejects_mana_restricted_against_it() {
    let (mut g, cmd) = commander_game();
    g.players[0].mana_pool.add_restricted(
        Color::Green,
        1,
        SpendRestriction::NoncreatureSpellsOnly,
    );
    assert!(
        g.perform_action(GameAction::CastFromCommandZone {
            card_id: cmd,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
            alternative: false,
            pitch_card: None,
        })
        .is_err(),
        "a creature commander is a creature spell",
    );
    assert_eq!(g.commander_cast_count.get(&cmd).copied(), None);
    assert!(g.players[0].command.iter().any(|c| c.id == cmd), "put back");
}

/// CR 903.3 — `spell_kind_for` is what tells the payment the spell is the
/// payer's own commander; a copy or an opponent's card is not.
#[test]
fn spell_kind_marks_only_your_own_commander() {
    let (g, cmd) = commander_game();
    let commander = g.players[0].command.iter().find(|c| c.id == cmd).unwrap();
    assert!(g.spell_kind_for(0, commander).commander);
    assert!(!g.spell_kind_for(1, commander).commander, "not seat 1's");
}

// ── Bot support ───────────────────────────────────────────────────────────

/// A rider is spendable on anything, so the auto-tapper reaches for its
/// source. Path of Ancestry has no other ability: when the auto-tapper
/// skipped every spend-restricted source it was a land that produced nothing
/// at all for a headless seat, and a deck running it was a land short.
#[test]
fn auto_tap_reaches_a_rider_source() {
    let (mut g, _cmd) = commander_game();
    let land = g.add_card_to_battlefield(0, catalog::path_of_ancestry());
    g.battlefield_find_mut(land).unwrap().tapped = false;
    // One more land for Grizzly Bears' generic pip.
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(forest).unwrap().tapped = false;
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    g.perform_action(GameAction::CastSpell {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("auto-tap pays {1}{G} from Path of Ancestry + a Forest");
    assert!(g.battlefield_find(land).unwrap().tapped, "the Path was tapped");
    // And the rider it carried fired: Bears shares a type with the commander.
    assert_eq!(g.stack.len(), 2, "spell + scry trigger");
}

/// A real restriction stays opaque to the auto-tapper — it can't know whether
/// what it is funding is allowed, so it leaves the source for the controller.
#[test]
fn auto_tap_still_skips_a_real_restriction() {
    let (mut g, _cmd) = commander_game();
    // Ancient Ziggurat: "{T}: Add one mana of any color. Spend this mana only
    // to cast creature spells."
    let zig = g.add_card_to_battlefield(0, catalog::ancient_ziggurat());
    g.battlefield_find_mut(zig).unwrap().tapped = false;
    let elves = g.add_card_to_hand(0, catalog::llanowar_elves());
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: elves,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "a restricted source is not auto-tapped",
    );
}
