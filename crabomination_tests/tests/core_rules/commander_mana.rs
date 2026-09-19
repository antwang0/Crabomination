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
    // Grizzly Bears is a Bear, like the commander. {1}{G}: one rider pip pays
    // the {G} (restricted drains first), plain mana pays the {1}, so exactly
    // one pip is spent and CR 106.6a's count is one.
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0]
        .mana_pool
        .add_restricted(Color::Green, 1, SpendRestriction::CommanderTypeScry);
    g.players[0].mana_pool.add_colorless(1);
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

/// CR 106.6a — "a separate delayed triggered ability is created for each mana
/// produced". Two Path of Ancestry pips on one creature spell (Mana Reflection
/// doubling the source) scry twice, and the ruling is explicit that this is
/// two scry-1 triggers rather than one scry 2.
#[test]
fn cr_106_6a_two_rider_pips_fire_two_triggers() {
    let (mut g, _cmd) = commander_game();
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    // Two rider pips and nothing else: both are spent on {1}{G}.
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
    .expect("cast");
    assert_eq!(g.stack.len(), 3, "the spell and one trigger per rider pip");
    assert!(matches!(g.stack[0], StackItem::Spell { .. }));
    assert!(matches!(g.stack[1], StackItem::Trigger { .. }));
    assert!(matches!(g.stack[2], StackItem::Trigger { .. }));
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
    // {G} printed + {4} tax (CR 903.8: {2} per prior cast). One rider pip for
    // the {G}, plain mana for the tax, so the CR 106.6a count is one and the
    // counters are purely the cast count.
    g.players[0]
        .mana_pool
        .add_restricted(Color::Green, 1, SpendRestriction::CommanderCastCounters);
    g.players[0].mana_pool.add_colorless(4);
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

/// CR 106.6a for the other rider: two doubled Opal Palace pips spent on the
/// commander are two counters for each prior command-zone cast, not one.
#[test]
fn cr_106_6a_two_opal_pips_double_the_counters() {
    let (mut g, cmd) = commander_game();
    g.commander_cast_count.insert(cmd, 1);
    // {G} printed + {2} tax, paid entirely from three rider pips.
    g.players[0]
        .mana_pool
        .add_restricted(Color::Green, 3, SpendRestriction::CommanderCastCounters);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("second cast");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(cmd).expect("resolved").counter_count(CounterType::PlusOnePlusOne),
        6,
        "3 pips x 2 command-zone casts",
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
    g.add_card_to_library(0, catalog::forest());
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

// ── CR 903.4 — an identity with no colours in it ──────────────────────────

/// CR 903.4 / rulings 2020-11-10 on Command Tower, Path of Ancestry and Opal
/// Palace: "If your commander is a card that has no colors in its color
/// identity, the ability produces no mana. It doesn't produce {C}."
#[test]
fn cr_903_4_a_colorless_commander_adds_no_mana_and_not_colorless() {
    use crabomination::card::{CardDefinition, CardType, Supertype};
    let colorless_commander = CardDefinition {
        name: "Test Colorless Commander",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        power: 2,
        toughness: 2,
        ..Default::default()
    };
    let mut g = game_with_format(Format::Commander, 2);
    g.seat_commanders(0, vec![colorless_commander]);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    assert!(g.commander_identity_colors(0).is_empty(), "no colours in the identity");

    for land in [catalog::command_tower(), catalog::path_of_ancestry()] {
        let mut g = g.clone();
        let id = g.add_card_to_battlefield(0, land);
        g.battlefield_find_mut(id).unwrap().tapped = false;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("the ability still activates");
        assert_eq!(g.players[0].mana_pool.total(), 0);
        assert_eq!(g.players[0].mana_pool.restricted_total(), 0);
        assert_eq!(g.players[0].mana_pool.colorless_amount(), 0, "and not {{C}}");
    }
}

/// The other half of the same ruling is the one place the engine answers
/// against the rules, deliberately: a seat with *no* commander keeps the
/// pre-Commander "any color", because Command Tower / Arcane Signet /
/// Commander's Sphere are fixing in the two-player cube pool and a dead land
/// there is a worse approximation than a permissive one.
#[test]
fn cr_903_4_no_commander_keeps_the_any_color_fallback() {
    let mut g = two_player_game();
    assert_eq!(g.commander_identity_colors(0).len(), 5);
    let id = g.add_card_to_battlefield(0, catalog::command_tower());
    g.battlefield_find_mut(id).unwrap().tapped = false;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap");
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

// ── CR 903.5e — Commander games do not use sideboards ─────────────────────

/// CR 903.5e. `validate_deck_refs` takes a flat card list — which is what lets
/// CR 903.5a count the commander with the 99 — so it never sees a sideboard,
/// and a Commander deck could carry one past validation.
#[test]
fn cr_903_5e_a_commander_deck_may_not_have_a_sideboard() {
    use crabomination::format::{Deck, DeckError, validate_commander_deck};
    let mut main: Vec<CardDefinition> = vec![catalog::forest(); 99];
    main[0] = catalog::llanowar_elves();
    let legal = Deck {
        commanders: vec![bear_commander()],
        main: main.clone(),
        ..Default::default()
    };
    assert!(validate_commander_deck(&legal).is_ok(), "the 99 + 1 is legal to start with");

    let with_side = Deck { sideboard: vec![catalog::forest()], ..legal };
    let (generic, cmd) = validate_commander_deck(&with_side).expect_err("a sideboard is illegal");
    assert!(cmd.is_empty(), "nothing Commander-specific is wrong with the list");
    assert!(
        generic.contains(&DeckError::SideboardNotAllowed { found: 1 }),
        "CR 903.5e, got {generic:?}",
    );
}

// ── CR 106.6 — a rider narrows nothing, and the estimate has to agree ──────

/// CR 106.6b/c. Path of Ancestry's "when that mana is spent…" is an ability
/// the mana *triggers*, not a restriction on what it may pay for, so the
/// floating green funds an activated ability's `{G}` like any other green.
///
/// The engine already did this; what did not was the bot's own estimate —
/// `ManaPool::total`/`amount` exclude the restricted bucket wholesale, so a
/// seat holding only this green read `by_color [0,0,0,0,0]`. On a four-seat
/// pod board that cleared `sink::AB_SAC` for Haywire Mite's `{G}, Sacrifice
/// this creature` while `pick_sacrifice_value` went on to build the
/// activation the engine then accepted and paid for — the gate audit's
/// forty-second find, which read as "an activation completed with no mana
/// anywhere" because the dump printed `total`.
#[test]
fn cr_106_6_a_rider_pays_an_activated_ability() {
    let (mut g, _cmd) = commander_game();
    let mite = g.add_card_to_battlefield(0, catalog::haywire_mite());
    g.clear_sickness(mite);
    let prey = g.add_card_to_battlefield(1, catalog::sol_ring());
    g.players[0].mana_pool.add_restricted(Color::Green, 1, SpendRestriction::CommanderTypeScry);

    g.perform_action(GameAction::ActivateAbility {
        card_id: mite,
        ability_index: 0,
        target: Some(Target::Permanent(prey)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("the rider green pays {G}");
    assert_eq!(g.players[0].mana_pool.restricted_total(), 0, "the rider pip was spent");
    assert!(g.battlefield_find(mite).is_none(), "and the sacrifice half was paid");
}

/// The control: a *real* restriction does not pay for this. Ancient
/// Ziggurat's green funds creature spells only, so the same activation is
/// rejected with the pool untouched — which is why the estimate is right to
/// keep ignoring non-rider restricted mana.
#[test]
fn cr_106_6_a_real_restriction_does_not_pay_an_activated_ability() {
    let (mut g, _cmd) = commander_game();
    let mite = g.add_card_to_battlefield(0, catalog::haywire_mite());
    g.clear_sickness(mite);
    let prey = g.add_card_to_battlefield(1, catalog::sol_ring());
    g.players[0].mana_pool.add_restricted(Color::Green, 1, SpendRestriction::CreatureOnly);

    g.perform_action(GameAction::ActivateAbility {
        card_id: mite,
        ability_index: 0,
        target: Some(Target::Permanent(prey)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect_err("creature-only mana can't fund an ability");
    assert_eq!(g.players[0].mana_pool.restricted_total(), 1, "nothing was spent");
    assert!(g.battlefield_find(mite).is_some(), "and nothing was sacrificed");
}

/// The invariant the two halves above rest on, asserted over every variant:
/// `is_rider()` — which is `label().is_none()`, the player-facing "there is
/// nothing to explain here" — must mean `allows()` is true for *every* kind.
/// `allows` now answers the rider half once by short-circuiting on the
/// predicate, so this is what stops a future variant being given a label and
/// a bare `=> true` arm, which is the shape that hid this bug.
///
/// The match is exhaustive on purpose: a new variant breaks this test's
/// compile, which is the point.
#[test]
fn cr_106_6_every_rider_allows_every_payment() {
    use crabomination::card::{CardDefinition, CreatureType};
    let all = {
        // Exhaustive by construction — the match forces a new variant to be
        // added to `all` below.
        fn _exhaustive(r: SpendRestriction) {
            use SpendRestriction::*;
            match r {
                InstantSorceryOnly | ArtifactOnly | CreatureOfTypeUncounterable(_)
                | CreatureOfType(_) | CreatureOfAnyTypes(_) | LandAbilitiesOnly | CreatureOnly
                | CreatureSpellsOrAbilities | NoNonartifactSpells | AbilitiesOnly
                | LessonSpellsOnly | DevoidSpellsOnly | InstantSorceryUncounterable
                | EquipmentOnly | ColorlessSpellsOrAbilities | HighMvOrX | DragonOrOmenSpell
                | EnchantmentSpell | MulticoloredSpell | ColoredSpellWithoutX
                | PlaneswalkerSpellsOnly
                | LegendarySpell | NoncreatureSpellsOnly | RoomSpellsOrDoors
                | FaceDownSpellsOrTurnFaceUp | CreatureHaste | CommanderTypeScry
                | CommanderCastCounters => {}
            }
        }
        use SpendRestriction::*;
        vec![
            InstantSorceryOnly,
            ArtifactOnly,
            CreatureOfTypeUncounterable(CreatureType::Bear),
            CreatureOfType(CreatureType::Bear),
            CreatureOfAnyTypes([CreatureType::Bear, CreatureType::Elf, CreatureType::Elf]),
            LandAbilitiesOnly,
            CreatureOnly,
            CreatureSpellsOrAbilities,
            NoNonartifactSpells,
            AbilitiesOnly,
            LessonSpellsOnly,
            DevoidSpellsOnly,
            InstantSorceryUncounterable,
            EquipmentOnly,
            ColorlessSpellsOrAbilities,
            HighMvOrX,
            DragonOrOmenSpell,
            EnchantmentSpell,
            MulticoloredSpell,
            ColoredSpellWithoutX,
            PlaneswalkerSpellsOnly,
            LegendarySpell,
            NoncreatureSpellsOnly,
            RoomSpellsOrDoors,
            FaceDownSpellsOrTurnFaceUp,
            CreatureHaste,
            CommanderTypeScry,
            CommanderCastCounters,
        ]
    };
    // A spanning-enough set of payments: a creature spell, a noncreature
    // spell, and an activated ability of a land.
    let kinds = [
        catalog::grizzly_bears().spell_kind(),
        catalog::sol_ring().spell_kind(),
        CardDefinition::default().ability_spend_kind(),
        catalog::forest().ability_spend_kind(),
    ];
    for r in all {
        let permits_all = kinds.iter().all(|k| r.allows(k));
        assert_eq!(
            r.is_rider(),
            permits_all,
            "{r:?}: is_rider() and 'allows every payment' must agree — a rider \
             that reports a label is mana `available_mana` will not count",
        );
    }
}

/// Titans' Nest — "spend this mana only to cast a spell that's one or more
/// colors without {X} in its mana cost": a colored spell yes; a colorless
/// spell, an {X} spell, an ability, or a bare payment no.
#[test]
fn colored_spell_without_x_restriction() {
    let r = SpendRestriction::ColoredSpellWithoutX;
    assert!(r.allows(&catalog::grizzly_bears().spell_kind()));
    assert!(!r.allows(&catalog::sol_ring().spell_kind()), "colorless");
    assert!(!r.allows(&catalog::fireball().spell_kind()), "has {{X}}");
    assert!(!r.allows(&catalog::forest().ability_spend_kind()), "an ability");
    assert!(!r.allows(&crabomination::mana::SpellKind::default()), "not a cast");
}
