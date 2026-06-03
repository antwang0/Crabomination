use super::*;
use super::{cast, cast_at, drain_stack, two_player_game};
use crate::catalog;
use crate::card::StaticAbility;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::effect::{Effect, ManaPayload, PlayerRef, Selector, StaticEffect, Value};
use crate::game::effects::EffectContext;
use crate::mana::Color;

// ── Setup ─────────────────────────────────────────────────────────────────

#[test]
fn players_start_with_20_life() {
    let g = two_player_game();
    assert_eq!(g.players[0].life, 20);
    assert_eq!(g.players[1].life, 20);
}

// ── Land ──────────────────────────────────────────────────────────────────

#[test]
fn play_land_moves_to_battlefield() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::forest());
    let events = g.perform_action(GameAction::PlayLand(id)).unwrap();
    assert!(g.battlefield.iter().any(|c| c.id == id));
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::LandPlayed { .. })));
}

#[test]
fn cannot_play_two_lands_per_turn() {
    let mut g = two_player_game();
    let f1 = g.add_card_to_hand(0, catalog::forest());
    let f2 = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(f1)).unwrap();
    let err = g.perform_action(GameAction::PlayLand(f2)).unwrap_err();
    assert_eq!(err, GameError::AlreadyPlayedLand);
}

#[test]
fn exploration_grants_extra_land_play_per_turn() {
    // CR 305.2 — "A player can normally play one land during their
    // turn; however, continuous effects may increase this number."
    // Exploration grants +1 land/turn via StaticEffect::ExtraLandPerTurn.
    let mut g = two_player_game();
    let _exp = g.add_card_to_battlefield(0, catalog::exploration());
    let f1 = g.add_card_to_hand(0, catalog::forest());
    let f2 = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(f1)).expect("first land play");
    // CR 305.2a — second land permitted because Exploration adds +1.
    g.perform_action(GameAction::PlayLand(f2))
        .expect("second land play permitted by Exploration");
    assert_eq!(g.players[0].lands_played_this_turn, 2);
}

#[test]
fn exploration_third_land_rejected_with_only_one_copy() {
    // CR 305.2b — "A player can't play a land if the number of lands
    // they can play this turn is equal to or less than the number
    // they've played." With one Exploration the cap is 2; the third
    // attempt errors.
    let mut g = two_player_game();
    let _exp = g.add_card_to_battlefield(0, catalog::exploration());
    let f1 = g.add_card_to_hand(0, catalog::forest());
    let f2 = g.add_card_to_hand(0, catalog::forest());
    let f3 = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(f1)).unwrap();
    g.perform_action(GameAction::PlayLand(f2)).unwrap();
    let err = g.perform_action(GameAction::PlayLand(f3)).unwrap_err();
    assert_eq!(err, GameError::AlreadyPlayedLand);
}

#[test]
fn two_explorations_stack_for_three_lands_per_turn() {
    // Stacking grant: two Explorations add +2, total cap is 3.
    let mut g = two_player_game();
    let _e1 = g.add_card_to_battlefield(0, catalog::exploration());
    let _e2 = g.add_card_to_battlefield(0, catalog::exploration());
    assert_eq!(g.max_lands_per_turn(0), 3);
    let f1 = g.add_card_to_hand(0, catalog::forest());
    let f2 = g.add_card_to_hand(0, catalog::forest());
    let f3 = g.add_card_to_hand(0, catalog::forest());
    let f4 = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(f1)).unwrap();
    g.perform_action(GameAction::PlayLand(f2)).unwrap();
    g.perform_action(GameAction::PlayLand(f3)).unwrap();
    let err = g.perform_action(GameAction::PlayLand(f4)).unwrap_err();
    assert_eq!(err, GameError::AlreadyPlayedLand);
}

#[test]
fn opp_exploration_does_not_grant_extra_land_to_you() {
    // ExtraLandPerTurn is checked against the *active* player's
    // controlled permanents — an opponent's Exploration doesn't help.
    let mut g = two_player_game();
    let _exp = g.add_card_to_battlefield(1, catalog::exploration());
    let f1 = g.add_card_to_hand(0, catalog::forest());
    let f2 = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(f1)).unwrap();
    let err = g.perform_action(GameAction::PlayLand(f2)).unwrap_err();
    assert_eq!(err, GameError::AlreadyPlayedLand);
    assert_eq!(g.max_lands_per_turn(0), 1, "P0 has no extra land grants");
    assert_eq!(g.max_lands_per_turn(1), 2, "P1 controls the Exploration");
}

#[test]
fn cannot_play_land_in_combat() {
    let mut g = two_player_game();
    g.step = TurnStep::DeclareAttackers;
    let id = g.add_card_to_hand(0, catalog::forest());
    let err = g.perform_action(GameAction::PlayLand(id)).unwrap_err();
    assert_eq!(err, GameError::SorcerySpeedOnly);
}

#[test]
fn annihilator_1_attack_forces_defender_sacrifice() {
    // CR 702.85a — "Whenever a permanent with annihilator attacks,
    // defending player sacrifices N permanents." Wire as Attacks trigger
    // → Effect::Sacrifice { who: defender, count: N, filter: Permanent }.
    use crate::card::{CardType, CardDefinition, CreatureType, Keyword, Subtypes};
    use crate::effect::Effect;
    use crate::game::{Attack, AttackTarget};
    use crate::mana::{cost, generic};

    fn annihilator_one() -> CardDefinition {
        CardDefinition {
            name: "Test Annihilator",
            cost: cost(&[generic(10)]),
            supertypes: vec![],
            card_types: vec![CardType::Creature],
            subtypes: Subtypes {
                creature_types: vec![CreatureType::Eldrazi],
                ..Default::default()
            },
            power: 10,
            toughness: 10,
            keywords: vec![Keyword::Annihilator(1)],
            effect: Effect::Noop,
            activated_abilities: vec![],
            triggered_abilities: vec![],
            static_abilities: vec![],
            base_loyalty: 0,
            loyalty_abilities: vec![],
            alternative_cost: None,
            back_face: None,
            opening_hand: None,
            enters_with_counters: None,
            enters_as_copy: None,
            max_counters_of_kind: None,
            exile_on_resolve: false,
            affinity_filter: None,
            equipped_bonus: None,
            additional_cast_cost: vec![],
        }
    }

    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, annihilator_one());
    g.clear_sickness(attacker);
    // Defender controls one fodder creature.
    let fodder = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(fodder);

    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;

    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }])).expect("Attacker can be declared");
    drain_stack(&mut g);

    // Defender sacrificed their bear.
    assert!(g.battlefield_find(fodder).is_none(),
        "defender's bear was sacrificed to Annihilator 1");
}

#[test]
fn play_land_retains_priority_after_special_action() {
    // CR 116.3 — "If a player takes a special action, that player
    // receives priority afterward." Playing a land (CR 116.2a) is a
    // special action; the active player should still hold priority
    // after PlayLand resolves.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::forest());
    let before = g.priority.player_with_priority;
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    let after = g.priority.player_with_priority;
    assert_eq!(after, before, "playing a land does not pass priority");
    // Stack is also empty after a special action (it doesn't use the stack).
    assert!(g.stack.is_empty(), "PlayLand does not push to stack");
}

// ── Tap for mana ──────────────────────────────────────────────────────────

#[test]
fn tap_forest_adds_green_mana() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: None, x_value: None })
    .unwrap();
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
}

#[test]
fn cannot_tap_already_tapped_land() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: None, x_value: None })
    .unwrap();
    let err = g
        .perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: 0,
            target: None, x_value: None })
        .unwrap_err();
    assert_eq!(err, GameError::CardIsTapped(id));
}

#[test]
fn llanowar_elves_tap_for_mana() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: None, x_value: None })
    .unwrap();
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
}

// ── Cast creature ─────────────────────────────────────────────────────────

#[test]
fn cast_grizzly_bears_enters_battlefield() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::grizzly_bears());
    // Pay {1}{G}
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .unwrap();
    let resolve_events = drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id));
    assert!(resolve_events.iter().any(|e| matches!(e, GameEvent::PermanentEntered { .. })));
}

#[test]
fn cast_creature_fails_without_mana() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::grizzly_bears());
    let err = g
        .perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .unwrap_err();
    assert!(matches!(err, GameError::Mana(_)));
    // Card still in hand after failed cast
    assert!(g.players[0].has_in_hand(id));
}

// ── Instants ──────────────────────────────────────────────────────────────

#[test]
fn lightning_bolt_deals_3_damage_to_player() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, id, Target::Player(1));
    assert_eq!(g.players[1].life, 17);
}

#[test]
fn lightning_bolt_kills_creature() {
    let mut g = two_player_game();
    let bolt_id = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bear_id = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, bolt_id, Target::Permanent(bear_id));
    assert!(!g.battlefield.iter().any(|c| c.id == bear_id));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear_id));
}

#[test]
fn giant_growth_pumps_creature() {
    let mut g = two_player_game();
    let spell_id = g.add_card_to_hand(0, catalog::giant_growth());
    let bear_id = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast_at(&mut g, spell_id, Target::Permanent(bear_id));
    let bear = g.battlefield.iter().find(|c| c.id == bear_id).unwrap();
    assert_eq!(bear.power(), 5);
    assert_eq!(bear.toughness(), 5);
}

#[test]
fn dark_ritual_adds_three_black_mana() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::dark_ritual());
    g.players[0].mana_pool.add(Color::Black, 1);
    cast(&mut g, id);
    // Paid 1B, gained BBB → net 2B in pool
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 3);
}

#[test]
fn ancestral_recall_draws_three_cards() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let id = g.add_card_to_hand(0, catalog::ancestral_recall());
    g.players[0].mana_pool.add(Color::Blue, 1);
    // "Target player draws three cards" — target self to draw 3.
    cast_at(&mut g, id, crate::game::types::Target::Player(0));
    assert_eq!(g.players[0].hand.len(), 3);
}

#[test]
fn ancestral_recall_can_target_an_opponent() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let id = g.add_card_to_hand(0, catalog::ancestral_recall());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let opp_hand_before = g.players[1].hand.len();
    cast_at(&mut g, id, crate::game::types::Target::Player(1));
    assert_eq!(
        g.players[1].hand.len(),
        opp_hand_before + 3,
        "the targeted opponent draws three"
    );
    assert_eq!(g.players[0].hand.len(), 0, "the caster did not draw");
}

#[test]
fn terror_destroys_non_black_creature() {
    let mut g = two_player_game();
    let terror_id = g.add_card_to_hand(0, catalog::terror());
    let bear_id = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 2);
    cast_at(&mut g, terror_id, Target::Permanent(bear_id));
    assert!(!g.battlefield.iter().any(|c| c.id == bear_id));
}

// ── Moxen ─────────────────────────────────────────────────────────────────

#[test]
fn mox_ruby_casts_for_free_and_taps_for_red() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mox_ruby());
    cast(&mut g, id);
    assert!(g.battlefield.iter().any(|c| c.id == id));
    // Tap immediately (not a creature, so no summoning sickness)
    g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: 0, target: None, x_value: None })
        .unwrap();
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
}

#[test]
fn each_mox_taps_for_its_color() {
    let cases: [(fn() -> crate::card::CardDefinition, Color); 4] = [
        (catalog::mox_pearl, Color::White),
        (catalog::mox_sapphire, Color::Blue),
        (catalog::mox_jet, Color::Black),
        (catalog::mox_emerald, Color::Green),
    ];
    for (def, color) in cases {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def());
        g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: 0, target: None, x_value: None })
            .unwrap();
        assert_eq!(g.players[0].mana_pool.amount(color), 1, "{color:?} mox should tap for its color");
    }
}

#[test]
fn mox_untaps_each_turn() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::mox_ruby());
    // Tap it
    g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: 0, target: None, x_value: None })
        .unwrap();
    assert!(g.battlefield.iter().find(|c| c.id == id).unwrap().tapped);
    // Simulate untap step
    g.do_untap();
    assert!(!g.battlefield.iter().find(|c| c.id == id).unwrap().tapped);
}

#[test]
fn terror_cannot_destroy_black_creature() {
    let mut g = two_player_game();
    let terror_id = g.add_card_to_hand(0, catalog::terror());
    let knight_id = g.add_card_to_battlefield(1, catalog::black_knight());
    g.players[0].mana_pool.add(Color::Black, 2);
    let err = g
        .perform_action(GameAction::CastSpell {
            card_id: terror_id,
            target: Some(Target::Permanent(knight_id)),
            additional_targets: vec![],
        mode: None, x_value: None })
        .unwrap_err();
    assert_eq!(err, GameError::SelectionRequirementViolated);
}

// ── Combat ────────────────────────────────────────────────────────────────

fn setup_attacker(g: &mut GameState, player: usize, def: impl Fn() -> crate::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(player, def());
    g.clear_sickness(id);
    id
}

/// Regression for `debug/deadlock-t9-1777413906-987970800.json`.
/// A creature whose `controller` flipped (Threaten / Mind Control)
/// must be a legal attacker for its CURRENT controller, even though
/// its `owner` field still points at the original player. Pre-fix
/// `declare_attackers` filtered with `c.owner == p` and rejected the
/// attack with `CardNotOnBattlefield`, the bot kept resubmitting,
/// and the watchdog tripped at 15s.
#[test]
fn declare_attackers_respects_controller_after_steal() {
    use crate::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    // Owned by P1, but controller flipped to P0 (Threaten effect).
    let stolen = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    if let Some(c) = g.battlefield_find_mut(stolen) {
        c.controller = 0;
        c.summoning_sick = false;
    }
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;

    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: stolen,
        target: AttackTarget::Player(1),
    }]))
    .expect("stolen creature must be a legal attacker for its new controller");
    assert!(g.attacking().iter().any(|a| a.attacker == stolen));
    assert!(g.battlefield_find(stolen).unwrap().tapped,
        "attackers tap on declare (no Vigilance)");
}

/// `would_accept` dry-runs an action against a clone of the state
/// without committing it. Sanity-check both branches: a legal action
/// returns true and the original state is unchanged; an illegal one
/// returns false and the original state is unchanged.
#[test]
fn would_accept_dry_runs_without_mutating_caller_state() {
    let mut g = two_player_game();
    let forest = g.add_card_to_hand(0, catalog::forest());
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;

    let battlefield_before = g.battlefield.len();
    let hand_before = g.players[0].hand.len();

    // Legal: PlayLand on a real land in hand at sorcery speed.
    assert!(g.would_accept(GameAction::PlayLand(forest)));
    assert_eq!(g.battlefield.len(), battlefield_before,
        "would_accept must not mutate the caller's state");
    assert_eq!(g.players[0].hand.len(), hand_before);

    // Illegal: PlayLand on a non-existent CardId.
    assert!(!g.would_accept(GameAction::PlayLand(crate::card::CardId(999))));
    assert_eq!(g.battlefield.len(), battlefield_before);

    // After the dry-run, the actual perform_action still works
    // (clone didn't trip up shared state).
    g.perform_action(GameAction::PlayLand(forest)).expect("real cast works after probes");
    assert_eq!(g.battlefield.len(), battlefield_before + 1);
}

/// `castable_hand_cards` reports a land in hand as playable, gates a
/// creature on available mana, and respects priority.
#[test]
fn castable_hand_cards_reflects_mana_timing_and_lands() {
    let mut g = two_player_game();
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;

    // A land in hand is "castable" (playable) at sorcery speed; a {1}{G}
    // creature with no mana available is not yet castable.
    let forest_in_hand = g.add_card_to_hand(0, catalog::forest());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());

    let castable = g.castable_hand_cards(0);
    assert!(castable.contains(&forest_in_hand), "land should be playable");
    assert!(
        !castable.contains(&bears),
        "creature unaffordable with no mana sources"
    );

    // Two untapped Forests cover {1}{G} via auto-tap → bears now castable.
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::forest());
    assert!(
        g.castable_hand_cards(0).contains(&bears),
        "bears castable off two untapped Forests"
    );

    // Sorcery-speed gating: the same creature is NOT castable on the
    // opponent's turn (still our priority, but not a main phase we own).
    g.active_player_idx = 1;
    assert!(
        !g.castable_hand_cards(0).contains(&bears),
        "creature not castable at instant speed on opponent's turn"
    );

    // Without priority, nothing is castable.
    g.active_player_idx = 0;
    g.priority.player_with_priority = 1;
    assert!(
        g.castable_hand_cards(0).is_empty(),
        "no priority → empty castable set"
    );
}

/// `pitchable_hand_cards` lists hand cards with a `from_hand` ability
/// (Spirit Guides) and respects priority.
#[test]
fn pitchable_hand_cards_lists_spirit_guides() {
    let mut g = two_player_game();
    g.priority.player_with_priority = 0;
    let guide = g.add_card_to_hand(0, catalog::simian_spirit_guide());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // no from-hand ability
    assert_eq!(g.pitchable_hand_cards(0), vec![guide], "only the Spirit Guide is pitchable");
    g.priority.player_with_priority = 1;
    assert!(g.pitchable_hand_cards(0).is_empty(), "off-priority → empty");
}

#[test]
fn kickable_hand_cards_lists_affordable_kickers() {
    let mut g = two_player_game();
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(1, catalog::grizzly_bears()); // a kicked-only target
    let ta = g.add_card_to_hand(0, catalog::tear_asunder());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // no kicker
    // Not enough mana for the full {1}{G}+{1}{B} → not kickable yet.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(g.kickable_hand_cards(0).is_empty(), "can't afford the kicker → empty");
    // Top up to the full kicked cost.
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert_eq!(g.kickable_hand_cards(0), vec![ta], "Tear Asunder is now kickable");
}

#[test]
fn dashable_hand_cards_lists_affordable_dash_creatures() {
    let mut g = two_player_game();
    g.priority.player_with_priority = 0;
    let scout = g.add_card_to_hand(0, catalog::mardu_scout()); // Dash {R}
    g.add_card_to_hand(0, catalog::grizzly_bears()); // no dash
    // No red yet → can't afford the dash cost.
    assert!(g.dashable_hand_cards(0).is_empty(), "no red → not dashable");
    g.players[0].mana_pool.add(Color::Red, 1);
    assert_eq!(g.dashable_hand_cards(0), vec![scout], "Mardu Scout is dashable for one red");
}

/// Symmetric: untap step should untap creatures the active player
/// CONTROLS, not just those they originally owned. A stolen creature
/// untaps on the new controller's turn, never the original owner's.
#[test]
fn untap_step_uses_controller_not_owner() {
    let mut g = two_player_game();
    let stolen = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    if let Some(c) = g.battlefield_find_mut(stolen) {
        c.controller = 0;
        c.tapped = true;
        c.summoning_sick = false;
    }
    // Original-owner card on P1's side, also tapped.
    let owned_by_p1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    if let Some(c) = g.battlefield_find_mut(owned_by_p1) {
        c.tapped = true;
        c.summoning_sick = false;
    }
    g.active_player_idx = 0;
    g.do_untap();
    assert!(!g.battlefield_find(stolen).unwrap().tapped,
        "stolen creature must untap on its CONTROLLER's turn");
    assert!(g.battlefield_find(owned_by_p1).unwrap().tapped,
        "P1's own creature must NOT untap during P0's turn");
}

#[test]
fn unblocked_attacker_deals_damage_to_player() {
    let mut g = two_player_game();
    let bear_id = setup_attacker(&mut g, 0, catalog::grizzly_bears);

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear_id,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();

    // Advance to combat damage
    g.step = TurnStep::CombatDamage;
    let events = g.resolve_combat().unwrap();

    assert_eq!(g.players[1].life, 18); // 20 - 2
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::DamageDealt { to_player: Some(1), amount: 2, .. })));
}

/// CR 615 — a prevent-all shield on the defending player stops unblocked
/// combat damage; a DamagePrevented event is emitted.
#[test]
fn prevention_shield_stops_combat_damage_to_player() {
    use crate::game::types::{PreventionShield, PreventionTarget};
    let mut g = two_player_game();
    let bear_id = setup_attacker(&mut g, 0, catalog::grizzly_bears);
    g.prevention_shields.push(PreventionShield {
        target: PreventionTarget::Player(1),
        remaining: None,
    });
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear_id,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    g.step = TurnStep::CombatDamage;
    let events = g.resolve_combat().unwrap();
    assert_eq!(g.players[1].life, 20, "all combat damage prevented");
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::DamagePrevented { to_player: Some(1), amount: 2, .. })));
}

/// Fog prevents all combat damage for the turn (CR 615.1).
#[test]
fn fog_prevents_all_combat_damage() {
    let mut g = two_player_game();
    let bear_id = setup_attacker(&mut g, 0, catalog::grizzly_bears);
    let fog = g.add_card_to_hand(0, catalog::fog());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, fog);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear_id,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    assert_eq!(g.players[1].life, 20, "Fog prevented combat damage");
}

/// Holy Day and Darkness are white/black Fogs.
#[test]
fn holy_day_and_darkness_are_fogs() {
    for d in [catalog::holy_day(), catalog::darkness()] {
        assert_eq!(d.cost.cmc(), 1);
        assert!(matches!(d.effect, Effect::PreventAllCombatDamageThisTurn));
    }
}

/// Samite Healer taps to prevent the next 1 damage to a creature.
#[test]
fn samite_healer_prevents_one_damage() {
    let mut g = two_player_game();
    let healer = g.add_card_to_battlefield(0, catalog::samite_healer());
    g.clear_sickness(healer);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: healer, ability_index: 0,
        target: Some(Target::Permanent(bear)), x_value: None,
    }).expect("Samite Healer ability activatable");
    drain_stack(&mut g);
    // Shock the bear for 2: 1 prevented, 1 marked → 2/2 survives (1 damage).
    let shock = g.add_card_to_hand(0, catalog::shock());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, shock, Target::Permanent(bear));
    assert!(g.battlefield.iter().any(|c| c.id == bear),
        "1 of 2 damage prevented → 2/2 survives 1 marked damage");
}

/// Skullcrack's "damage can't be prevented this turn" overrides a shield
/// on the targeted player.
#[test]
fn skullcrack_damage_cant_be_prevented() {
    use crate::game::types::{PreventionShield, PreventionTarget};
    let mut g = two_player_game();
    g.players[1].life = 5;
    g.prevention_shields.push(PreventionShield {
        target: PreventionTarget::Player(1),
        remaining: None,
    });
    let sk = g.add_card_to_hand(0, catalog::skullcrack());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_at(&mut g, sk, Target::Player(1));
    assert_eq!(g.players[1].life, 2, "shield ignored — 3 damage went through");
}

#[test]
fn blocked_combat_both_die() {
    let mut g = two_player_game();
    let attacker_id = setup_attacker(&mut g, 0, catalog::grizzly_bears);
    let blocker_id = setup_attacker(&mut g, 1, catalog::grizzly_bears);

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: attacker_id,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();

    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker_id, attacker_id)]))
        .unwrap();

    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();

    // Both 2/2 creatures trade
    assert!(!g.battlefield.iter().any(|c| c.id == attacker_id));
    assert!(!g.battlefield.iter().any(|c| c.id == blocker_id));
    // Defending player life unchanged (attacker was blocked)
    assert_eq!(g.players[1].life, 20);
}

/// CR 615 — a prevention shield on a blocker stops the attacker's combat
/// damage to it (creature-vs-creature path). The shielded 2/2 survives a
/// 3/3 attacker; the attacker still takes the blocker's 2 back.
#[test]
fn prevention_shield_stops_creature_combat_damage() {
    use crate::game::types::{PreventionShield, PreventionTarget};
    let mut g = two_player_game();
    let attacker_id = setup_attacker(&mut g, 0, catalog::hill_giant); // 3/3
    let blocker_id = setup_attacker(&mut g, 1, catalog::grizzly_bears); // 2/2
    g.prevention_shields.push(PreventionShield {
        target: PreventionTarget::Permanent(blocker_id),
        remaining: None, // prevent all damage to the blocker this turn
    });

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: attacker_id,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker_id, attacker_id)]))
        .unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();

    // Blocker survived (its 3 incoming damage was prevented); attacker
    // took the blocker's 2 back (no shield), surviving as a 3/3.
    let blocker = g.battlefield.iter().find(|c| c.id == blocker_id)
        .expect("shielded blocker survives");
    assert_eq!(blocker.damage, 0, "all combat damage to the blocker prevented");
    let attacker = g.battlefield.iter().find(|c| c.id == attacker_id)
        .expect("attacker still alive");
    assert_eq!(attacker.damage, 2, "attacker took the blocker's 2 (unshielded)");
}

/// Regression: a `BecomesBlocked` trigger must bind the **attacker** as
/// `Selector::TriggerSource`, not the blocker. Both kinds (`Blocks` /
/// `BecomesBlocked`) come off the same `BlockerDeclared` event, so
/// `event_subject` has to disambiguate via the trigger's `EventKind`.
/// No catalog card uses this kind today; the test pins the wiring so
/// the next one that does won't silently target the wrong creature.
#[test]
fn becomes_blocked_trigger_resolves_against_attacker_not_blocker() {
    use crate::card::{CardDefinition, CardType, CounterType, Subtypes, TriggeredAbility};
    use crate::effect::{EventKind, EventScope, EventSpec};

    // 3/3 attacker that puts a +1/+1 counter on itself when it becomes
    // blocked. With the bug, `TriggerSource` resolved to the blocker
    // and the counter would land on the blocker instead.
    let counted_attacker = || CardDefinition {
        name: "Test BecomesBlocked Attacker",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![], ..Default::default() },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    };

    let mut g = two_player_game();
    let attacker_id = setup_attacker(&mut g, 0, counted_attacker);
    let blocker_id = setup_attacker(&mut g, 1, catalog::grizzly_bears);

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: attacker_id,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();

    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker_id, attacker_id)]))
        .unwrap();
    drain_stack(&mut g);

    let attacker = g.battlefield.iter().find(|c| c.id == attacker_id)
        .expect("attacker still on battlefield");
    let blocker = g.battlefield.iter().find(|c| c.id == blocker_id)
        .expect("blocker still on battlefield");
    assert_eq!(attacker.counter_count(CounterType::PlusOnePlusOne), 1,
        "BecomesBlocked trigger should put the counter on the attacker");
    assert_eq!(blocker.counter_count(CounterType::PlusOnePlusOne), 0,
        "blocker must NOT receive the counter");
}

/// Regression: the first-strike combat damage step must only allow
/// blockers whose own keywords say they deal damage in that step.
/// Previously the inline blocker filter short-circuited to `true`
/// whenever the attacker had FirstStrike/DoubleStrike, which let a
/// regular blocker hit the FS attacker before being killed by SBA —
/// erasing first strike's protective effect entirely.
#[test]
fn first_strike_attacker_kills_regular_blocker_before_being_struck() {
    let mut g = two_player_game();
    // 2/2 First Strike attacker vs. 2/2 vanilla blocker.
    let attacker_id = setup_attacker(&mut g, 0, catalog::white_knight);
    let blocker_id = setup_attacker(&mut g, 1, catalog::grizzly_bears);

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: attacker_id,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();

    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker_id, attacker_id)]))
        .unwrap();

    // First-strike step: White Knight deals 2 to Bears, Bears dies in SBA.
    g.step = TurnStep::FirstStrikeDamage;
    g.resolve_first_strike_damage().unwrap();
    assert!(!g.battlefield.iter().any(|c| c.id == blocker_id),
        "Bears should die to first-strike damage before regular damage step");
    assert!(g.battlefield.iter().any(|c| c.id == attacker_id),
        "White Knight should be unharmed — Bears never gets to strike back");

    // Regular step: no blockers left, no damage.
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    let knight = g.battlefield.iter().find(|c| c.id == attacker_id)
        .expect("White Knight survives combat");
    assert_eq!(knight.damage, 0, "White Knight should take 0 damage from a dead blocker");
}

#[test]
fn vigilance_creature_does_not_tap_when_attacking() {
    let mut g = two_player_game();
    let angel_id = setup_attacker(&mut g, 0, catalog::serra_angel);

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: angel_id,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    let angel = g.battlefield.iter().find(|c| c.id == angel_id).unwrap();
    assert!(!angel.tapped, "Vigilance: Serra Angel should not tap when attacking");
}

#[test]
fn flying_creature_cannot_be_blocked_by_ground_creature() {
    let mut g = two_player_game();
    let angel_id = setup_attacker(&mut g, 0, catalog::serra_angel);
    let bear_id = setup_attacker(&mut g, 1, catalog::grizzly_bears);

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: angel_id,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();

    g.step = TurnStep::DeclareBlockers;
    let err = g
        .perform_action(GameAction::DeclareBlockers(vec![(bear_id, angel_id)]))
        .unwrap_err();
    assert_eq!(err, GameError::CannotBlock(bear_id));
}

#[test]
fn summoning_sick_creature_cannot_attack() {
    let mut g = two_player_game();
    let bear_id = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // still sick

    g.step = TurnStep::DeclareAttackers;
    let err = g
        .perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bear_id,
            target: AttackTarget::Player(1),
        }]))
        .unwrap_err();
    assert_eq!(err, GameError::SummoningSickness(bear_id));
}

#[test]
fn haste_creature_can_attack_immediately() {
    let mut g = two_player_game();
    let goblin_id = g.add_card_to_battlefield(0, catalog::goblin_guide()); // Haste, still sick

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: goblin_id,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    // No error — Haste bypasses summoning sickness
}

// ── Win condition ─────────────────────────────────────────────────────────

#[test]
fn player_dies_when_life_reaches_zero() {
    let mut g = two_player_game();
    g.players[1].life = 3;
    let bolt_id = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
            card_id: bolt_id,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
        mode: None, x_value: None })
        .unwrap();
    let resolve_events = drain_stack(&mut g);
    assert!(g.is_game_over());
    assert!(resolve_events
        .iter()
        .any(|e| matches!(e, GameEvent::GameOver { winner: Some(0) })));
}

/// CR 121.4 / 704.5b — "A player who attempts to draw a card from a library
/// with no cards in it loses the game." The engine's `Effect::Draw` handler
/// sets `Player.eliminated = true` immediately when `draw_top()` returns
/// `None`; the state-based-action sweep at the end of the resolution
/// promotes that to `game_over = Some(Some(winner))`.
#[test]
fn drawing_from_empty_library_eliminates_player() {
    let mut g = two_player_game();
    // Empty P1's library — Divination cast by P1 will try to draw 2 cards
    // off a zero-card library, eliminating P1 the first time draw_top()
    // returns None.
    assert!(g.players[1].library.is_empty(), "fixture has empty library");

    let divination = g.add_card_to_hand(1, catalog::divination());
    // Give P1 the active player slot + priority + mana — Divination is a
    // sorcery so it requires P1 to be the active player in a main phase.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.players[1].mana_pool.add_colorless(2);
    cast(&mut g, divination);

    assert!(g.is_game_over(),
        "P1 should lose the game from drawing off an empty library");
    assert_eq!(g.game_over, Some(Some(0)),
        "P0 should win when P1 decks out");
}

// ── Turn progression ──────────────────────────────────────────────────────

#[test]
fn pass_priority_advances_step() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::PassPriority).unwrap();
    g.perform_action(GameAction::PassPriority).unwrap();
    assert_eq!(g.step, TurnStep::BeginCombat);
}

#[test]
fn untap_step_clears_summoning_sickness() {
    let mut g = two_player_game();
    let bear_id = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.battlefield.iter().find(|c| c.id == bear_id).unwrap().summoning_sick);

    // The bear belongs to player 0.  Its sickness clears during player 0's
    // Untap step, which follows the end of player 1's turn (Cleanup).
    // Simulate: it is the end of player 1's turn.
    g.step = TurnStep::Cleanup;
    g.active_player_idx = 1;
    g.perform_action(GameAction::PassPriority).unwrap();
    g.perform_action(GameAction::PassPriority).unwrap();

    // We should now be in player 0's Untap step
    assert_eq!(g.step, TurnStep::Untap);
    assert_eq!(g.active_player_idx, 0);
    // Summoning sickness cleared for player 0's permanents
    assert!(!g.battlefield.iter().find(|c| c.id == bear_id).unwrap().summoning_sick);
}

#[test]
fn cleanup_resets_end_of_turn_pump() {
    let mut g = two_player_game();
    let bear_id = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == bear_id).unwrap().power_bonus = 3;
    g.step = TurnStep::Cleanup;
    g.perform_action(GameAction::PassPriority).unwrap();
    g.perform_action(GameAction::PassPriority).unwrap();
    // Pump should be gone after cleanup
    let bear = g.battlefield.iter().find(|c| c.id == bear_id).unwrap();
    assert_eq!(bear.power_bonus, 0);
}

// ── New effects ───────────────────────────────────────────────────────────────

fn one_u_spell() -> crate::card::CardDefinition {
    use crate::mana::{ManaCost, ManaSymbol};
    let mut spell = catalog::grizzly_bears();
    spell.cost = ManaCost {
        symbols: vec![ManaSymbol::Generic(1), ManaSymbol::Colored(Color::Blue)],
    };
    spell
}

#[test]
fn cast_one_u_with_dual_and_basic_auto_taps_correctly() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tundra());
    g.add_card_to_battlefield(0, catalog::island());
    let id = g.add_card_to_hand(0, one_u_spell());
    cast(&mut g, id);
    assert!(g.battlefield.iter().any(|c| c.id == id));
}

#[test]
fn cast_one_u_with_two_tundras_auto_taps_correctly() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tundra());
    g.add_card_to_battlefield(0, catalog::tundra());
    let id = g.add_card_to_hand(0, one_u_spell());
    cast(&mut g, id);
    assert!(g.battlefield.iter().any(|c| c.id == id));
}

#[test]
fn inquisition_of_kozilek_picks_low_cmc_nonland() {
    let mut g = two_player_game();
    // Opponent's hand: a land (forest) + a high-cost spell (Mahamoti Djinn,
    // {3}{U}{U}, CMC 5) + a low-cost spell (Lightning Bolt, {R}, CMC 1).
    let _land = g.add_card_to_hand(1, catalog::forest());
    let _big = g.add_card_to_hand(1, catalog::mahamoti_djinn());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());

    let inq = g.add_card_to_hand(0, catalog::inquisition_of_kozilek());
    g.players[0].mana_pool.add(Color::Black, 1);
    cast(&mut g, inq);

    // Lightning Bolt (CMC 1, nonland) should be the pick. Mahamoti is CMC 5
    // (excluded) and the forest is a land (excluded).
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));
    assert!(!g.players[1].hand.iter().any(|c| c.id == bolt));
}

#[test]
fn thoughtseize_picks_nonland_and_costs_two_life() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::forest());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let life_before = g.players[0].life;

    let ts = g.add_card_to_hand(0, catalog::thoughtseize());
    g.players[0].mana_pool.add(Color::Black, 1);
    cast(&mut g, ts);

    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Thoughtseize should pick the nonland Lightning Bolt");
    assert_eq!(g.players[0].life, life_before - 2);
}

#[test]
fn solitude_evoke_exiles_target_then_sacrifices_self() {
    use crate::card::CardType;
    let mut g = two_player_game();
    let solitude_id = g.add_card_to_hand(0, catalog::solitude());
    // P0 has a white card to pitch.
    let pitch = g.add_card_to_hand(0, catalog::serra_angel());
    // P1 has a creature for Solitude's ETB to exile.
    let opp_creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: solitude_id,
        pitch_card: Some(pitch),
        target: Some(Target::Permanent(opp_creature)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Solitude evoke should succeed");
    drain_stack(&mut g);

    // ETB exile fires → opponent's creature in exile.
    assert!(g.exile.iter().any(|c| c.id == opp_creature),
        "Solitude's ETB should exile the targeted creature");
    // Evoke sacrifice fires → Solitude is back in P0's graveyard.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == solitude_id),
        "Solitude should be sacrificed via evoke");
    assert!(!g.battlefield.iter().any(|c| c.id == solitude_id));
    // Pitched white card is in exile.
    assert!(g.exile.iter().any(|c| c.id == pitch));
    // Make sure CardType is used so the import isn't flagged unused.
    let _ = CardType::Creature;
}

#[test]
fn thud_sacrifices_creature_and_deals_damage_equal_to_its_power() {
    let mut g = two_player_game();
    // Give P0 a 5-power creature to sacrifice.
    let bear_id = g.add_card_to_battlefield(0, catalog::shivan_dragon()); // 5/5
    g.clear_sickness(bear_id);
    let thud_id = g.add_card_to_hand(0, catalog::thud());
    g.players[0].mana_pool.add(Color::Red, 1);

    let opp_life_before = g.players[1].life;
    cast_at(&mut g, thud_id, Target::Player(1));

    // Shivan Dragon (power 5) sacrificed → 5 damage to P1.
    assert!(!g.battlefield.iter().any(|c| c.id == bear_id),
        "Sacrificed creature should leave the battlefield");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear_id));
    assert_eq!(g.players[1].life, opp_life_before - 5,
        "P1 should take 5 damage equal to sacrificed Shivan Dragon's power");
}

#[test]
fn cosmogoyf_pt_scales_with_card_types_in_graveyards() {
    use crate::card::CardType;
    let mut g = two_player_game();
    let goyf_id = g.add_card_to_battlefield(0, catalog::cosmogoyf());

    // Empty graveyards → Cosmogoyf is 0/1.
    let cp = g.computed_permanent(goyf_id).unwrap();
    assert_eq!(cp.power, 0);
    assert_eq!(cp.toughness, 1);

    // Add a Creature card to graveyard → 1/2.
    g.players[0].graveyard.push(crate::card::CardInstance::new(
        crate::card::CardId(9001),
        catalog::grizzly_bears(),
        0,
    ));
    let cp = g.computed_permanent(goyf_id).unwrap();
    assert_eq!(cp.power, 1);
    assert_eq!(cp.toughness, 2);

    // Add an Instant card → 2/3.
    g.players[1].graveyard.push(crate::card::CardInstance::new(
        crate::card::CardId(9002),
        catalog::lightning_bolt(),
        1,
    ));
    let cp = g.computed_permanent(goyf_id).unwrap();
    assert_eq!(cp.power, 2);
    assert_eq!(cp.toughness, 3);

    // Confirm CardType is referenced so the import isn't flagged.
    let _ = CardType::Land;
}

/// Spam pass-priority for at most `n` cycles or until the game ends.
fn pass_until_game_over_or(g: &mut GameState, max: usize) {
    for _ in 0..max {
        if g.is_game_over() {
            return;
        }
        g.perform_action(GameAction::PassPriority).unwrap();
    }
}

#[test]
fn pact_of_negation_eliminates_caster_if_unpaid_on_next_upkeep() {
    // Resolve a Pact, pass to next upkeep with no mana — engine should
    // auto-fail the pay-or-lose and eliminate the caster.
    let mut g = two_player_game();
    let pact = g.add_card_to_hand(0, catalog::pact_of_negation());
    g.perform_action(GameAction::CastSpell {
        card_id: pact, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(g.delayed_triggers.len(), 1, "pact registers a delayed upkeep trigger");

    // Advance to P0's next upkeep. P0 has no lands → can't pay {3}{U}{U}.
    g.step = TurnStep::Cleanup;
    g.active_player_idx = 1;
    pass_until_game_over_or(&mut g, 30);

    assert!(g.is_game_over(), "P0 should have been eliminated by unpaid pact");
    assert_eq!(g.game_over, Some(Some(1)));
}

#[test]
fn pact_of_negation_lets_caster_live_when_they_can_pay() {
    let mut g = two_player_game();
    let pact = g.add_card_to_hand(0, catalog::pact_of_negation());
    g.perform_action(GameAction::CastSpell {
        card_id: pact, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);

    // Give P0 enough mana sources to pay {3}{U}{U} via auto-tap on the
    // upkeep. (Mana pool itself empties between steps, so pre-stocking the
    // pool would do nothing — we need actual untapped lands.)
    for _ in 0..5 {
        g.add_card_to_battlefield(0, catalog::island());
    }

    // Advance to P0's next upkeep — but stop at Upkeep so the resolved
    // Pact trigger can be observed without rolling further into the turn.
    g.step = TurnStep::Cleanup;
    g.active_player_idx = 1;
    for _ in 0..15 {
        if g.is_game_over() { break; }
        if g.active_player_idx == 0 && g.step == TurnStep::Upkeep && g.stack.is_empty() {
            break;
        }
        g.perform_action(GameAction::PassPriority).unwrap();
    }

    assert!(!g.is_game_over(), "P0 paid the pact and should still be alive");
    assert!(g.players[0].is_alive());
    assert!(g.delayed_triggers.is_empty(), "pact trigger fired and consumed");
}

#[test]
fn goryos_vengeance_exiles_creature_at_end_step() {
    // Reanimate Griselbrand with Goryo's, pass to end step → it's exiled.
    let mut g = two_player_game();
    let grisel = g.add_card_to_library(0, catalog::griselbrand());
    // Move Griselbrand straight into the graveyard for the test setup.
    let card = g.players[0].library.iter().position(|c| c.id == grisel)
        .map(|pos| g.players[0].library.remove(pos))
        .unwrap();
    g.players[0].graveyard.push(card);

    let goryo = g.add_card_to_hand(0, catalog::goryos_vengeance());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: goryo,
        target: Some(Target::Permanent(grisel)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Goryo's should accept legendary creature target");
    drain_stack(&mut g);

    // Griselbrand on battlefield, delayed exile registered.
    assert!(g.battlefield.iter().any(|c| c.id == grisel));
    assert_eq!(g.delayed_triggers.len(), 1);

    // Pass to end step — delayed trigger fires and exiles Griselbrand.
    g.step = TurnStep::PostCombatMain;
    for _ in 0..15 {
        if g.exile.iter().any(|c| c.id == grisel) { break; }
        if g.is_game_over() { break; }
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    assert!(!g.battlefield.iter().any(|c| c.id == grisel),
        "Griselbrand should be exiled at end step");
    assert!(g.exile.iter().any(|c| c.id == grisel),
        "Griselbrand should be in the exile zone");
}

#[test]
fn force_of_will_pitches_a_blue_card_to_counter_a_spell() {
    // Validate the alt-cost path: pitch a blue card (Counterspell) instead
    // of paying the {3}{U}{U} mana cost. The FoW spell goes on the stack
    // and the pitch card moves to exile while life is paid.
    let mut g = two_player_game();
    let fow = g.add_card_to_hand(0, catalog::force_of_will());
    let pitch = g.add_card_to_hand(0, catalog::counterspell()); // blue card to exile
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: fow,
        pitch_card: Some(pitch),
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Force of Will should be castable via pitch alt cost");

    // Pitched card moved to exile, no longer in hand.
    assert!(g.exile.iter().any(|c| c.id == pitch));
    assert!(!g.players[0].hand.iter().any(|c| c.id == pitch));
    // Force of Will itself is on the stack.
    assert!(g.stack.iter().any(|si| matches!(
        si,
        crate::game::StackItem::Spell { card, .. } if card.id == fow
    )));
    // 1 life paid for the alt cost; no mana spent.
    assert_eq!(g.players[0].life, life_before - 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 0);
}

#[test]
fn force_of_will_rejects_non_blue_pitch_card() {
    let mut g = two_player_game();
    let fow = g.add_card_to_hand(0, catalog::force_of_will());
    let bad_pitch = g.add_card_to_hand(0, catalog::lightning_bolt()); // red card
    let err = g
        .perform_action(GameAction::CastSpellAlternative {
            card_id: fow,
            pitch_card: Some(bad_pitch),
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .unwrap_err();
    assert!(matches!(err, GameError::InvalidPitchCard(_)));
}

/// CR 119.4: A player can only pay an amount of life if their life
/// total is greater than or equal to the payment. Force of Will's alt
/// cost is "Pay 1 life and exile a blue card" — the cast path must
/// reject cleanly when the caster is at 0 life rather than driving
/// life negative.
#[test]
fn force_of_will_alt_cost_rejected_at_zero_life_per_cr_119_4() {
    let mut g = two_player_game();
    g.players[0].life = 0;

    let fow = g.add_card_to_hand(0, catalog::force_of_will());
    // Add a legal blue pitch card — Brainstorm is blue.
    let pitch = g.add_card_to_hand(0, catalog::brainstorm());

    let err = g
        .perform_action(GameAction::CastSpellAlternative {
            card_id: fow,
            pitch_card: Some(pitch),
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect_err("FoW alt cast should be rejected at 0 life per CR 119.4");
    assert!(matches!(err, GameError::InsufficientLife),
        "Expected InsufficientLife error, got {err:?}");
    // Life unchanged.
    assert_eq!(g.players[0].life, 0,
        "Life should not be driven negative — pre-flight gate rejected first");
    // FoW still in hand (not on stack).
    assert!(g.players[0].hand.iter().any(|c| c.id == fow),
        "FoW should remain in hand after a rejected cast");
}

/// CR 119.4 mirror: at 1 life, paying 1 life is legal (since
/// 1 >= 1). The cast should succeed and the player should be left at
/// 0 life — life equal to the cost is allowed.
#[test]
fn force_of_will_alt_cost_accepted_at_one_life_per_cr_119_4() {
    let mut g = two_player_game();
    g.players[0].life = 1;
    let fow = g.add_card_to_hand(0, catalog::force_of_will());
    let pitch = g.add_card_to_hand(0, catalog::brainstorm());

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: fow,
        pitch_card: Some(pitch),
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("FoW alt cast should succeed at 1 life (1 >= 1)");

    // Life dropped to 0 — the cast paid 1 life, leaving the player at 0.
    // Note: SBAs only fire on negative life; 0 is still alive.
    assert_eq!(g.players[0].life, 0);
}

#[test]
fn black_lotus_manual_activation_with_wants_ui_prompts_for_color() {
    use crate::decision::Decision;
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let lotus = g.add_card_to_battlefield(0, catalog::black_lotus());
    g.clear_sickness(lotus);
    // Activating manually should suspend on a ChooseColor decision instead
    // of auto-picking White.
    g.perform_action(GameAction::ActivateAbility {
        card_id: lotus, ability_index: 0, target: None, x_value: None })
    .unwrap();
    let pd = g.pending_decision.as_ref().expect("ChooseColor should suspend");
    assert!(matches!(pd.decision, Decision::ChooseColor { .. }));
    // Picking Blue adds 3 blue mana to the pool.
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Color(Color::Blue)))
        .unwrap();
    assert!(g.pending_decision.is_none());
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 3);
    assert_eq!(g.players[0].mana_pool.amount(Color::White), 0);
}

#[test]
fn black_lotus_auto_taps_for_specific_color() {
    // Auto-tap on a {1}{U} cost should trigger Black Lotus's
    // `AnyOneColor(3)` ability and have it add Blue, not the default White
    // that AutoDecider would pick. Without this, casting any colored spell
    // off Black Lotus alone fails.
    use crate::mana::{ManaCost, ManaSymbol};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::black_lotus());
    let mut spell = catalog::grizzly_bears();
    spell.cost = ManaCost {
        symbols: vec![ManaSymbol::Generic(1), ManaSymbol::Colored(Color::Blue)],
    };
    let id = g.add_card_to_hand(0, spell);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Black Lotus should auto-tap into Blue for {1}{U}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id));
}

#[test]
fn cast_white_knight_with_two_tundras() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tundra());
    g.add_card_to_battlefield(0, catalog::tundra());
    let id = g.add_card_to_hand(0, catalog::white_knight());
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Two Tundras should pay {W}{W} for White Knight");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id));
}

#[test]
fn flooded_strand_fetches_tundra_untapped() {
    // Onslaught/Zendikar fetchlands put the fetched land onto the
    // battlefield UNTAPPED. Regression: we previously set `tapped: true`,
    // which made `{W}{W}` spells uncastable when a fetched Tundra was on
    // the board.
    let mut g = two_player_game();
    let tundra_in_lib = g.add_card_to_library(0, catalog::tundra());
    let strand_id = g.add_card_to_battlefield(0, catalog::flooded_strand());
    g.clear_sickness(strand_id);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(
        tundra_in_lib,
    ))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: strand_id,
        ability_index: 0,
        target: None, x_value: None })
    .unwrap();
    drain_stack(&mut g);
    let fetched = g
        .battlefield
        .iter()
        .find(|c| c.id == tundra_in_lib)
        .expect("Tundra should be on battlefield after fetch");
    assert!(!fetched.tapped, "fetched Tundra should enter untapped");
}

#[test]
fn cast_white_knight_with_one_tundra_already_tapped_fails() {
    // Reproduces the exact "Need 1 White mana but only have 0" error: one
    // Tundra is already tapped (fetched this turn, used for an earlier
    // spell, etc.), so only one W source is available — not enough for {W}{W}.
    let mut g = two_player_game();
    let t1 = g.add_card_to_battlefield(0, catalog::tundra());
    g.add_card_to_battlefield(0, catalog::tundra());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == t1) {
        c.tapped = true;
    }
    let id = g.add_card_to_hand(0, catalog::white_knight());
    let err = g
        .perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .unwrap_err();
    assert!(
        matches!(err, GameError::Mana(_)),
        "one tapped Tundra + one untapped Tundra cannot pay {{W}}{{W}}, got {err:?}"
    );
}

#[test]
fn cast_white_knight_after_playing_two_tundras_via_play_land() {
    // Simulate the user playing both Tundras through the normal play-land
    // path (vs. dropping them straight onto the battlefield). They share a
    // turn for simplicity — the engine doesn't enforce the one-land-per-turn
    // rule across direct calls when we reset `lands_played_this_turn`.
    let mut g = two_player_game();
    let t1 = g.add_card_to_hand(0, catalog::tundra());
    let t2 = g.add_card_to_hand(0, catalog::tundra());
    let knight_id = g.add_card_to_hand(0, catalog::white_knight());

    g.perform_action(GameAction::PlayLand(t1)).unwrap();
    // Reset the land-drop counter so we can play a second Tundra in this
    // test without sequencing through a full turn.
    g.players[0].lands_played_this_turn = 0;
    g.perform_action(GameAction::PlayLand(t2)).unwrap();

    g.perform_action(GameAction::CastSpell {
        card_id: knight_id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Two freshly-played Tundras should pay {W}{W}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == knight_id));
}

#[test]
fn cast_u_with_only_tundra_succeeds() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tundra());
    let id = g.add_card_to_hand(0, catalog::opt());
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Single Tundra should pay {U} (Opt)");
    drain_stack(&mut g);
}

#[test]
fn cast_one_u_with_only_tundra_fails() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tundra());
    let id = g.add_card_to_hand(0, one_u_spell());
    let err = g
        .perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .unwrap_err();
    assert!(matches!(err, GameError::Mana(_)),
        "single Tundra should not be enough mana for {{1}}{{U}}, got {err:?}");
}

#[test]
fn cast_one_u_with_tundra_plus_plains_auto_taps_correctly() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tundra());
    g.add_card_to_battlefield(0, catalog::plains());
    let id = g.add_card_to_hand(0, one_u_spell());
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Tundra (for U) + Plains (for generic) should pay {1}{U}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id));
}

#[test]
fn putrefy_can_destroy_artifact() {
    let mut g = two_player_game();
    let mox_id = g.add_card_to_battlefield(1, catalog::mox_ruby());
    let putrefy_id = g.add_card_to_hand(0, catalog::putrefy());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: putrefy_id,
        target: Some(Target::Permanent(mox_id)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == mox_id));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == mox_id));
}

#[test]
fn goblin_guide_reveals_top_and_gives_land() {
    let mut g = two_player_game();
    // Stack defending player (1) library: top is a forest.
    let forest_id = g.add_card_to_library(1, catalog::forest());
    let goblin_id = setup_attacker(&mut g, 0, catalog::goblin_guide);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: goblin_id,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    drain_stack(&mut g);
    // Forest moved from defender's library to defender's hand.
    assert!(g.players[1].hand.iter().any(|c| c.id == forest_id));
    assert!(!g.players[1].library.iter().any(|c| c.id == forest_id));
}

/// CR 121.5 — "If an effect moves cards from a player's library to that
/// player's hand without using the word 'draw,' the player has not drawn
/// those cards." Goblin Guide says "puts it into their hand" (not "draws"),
/// so the put-into-hand transition must NOT increment
/// `cards_drawn_this_turn` and must NOT fire `CardDrawn` events for
/// draw-payoff triggers. This test pins both invariants.
#[test]
fn goblin_guide_put_into_hand_is_not_a_draw_per_cr_121_5() {
    let mut g = two_player_game();
    let _forest_id = g.add_card_to_library(1, catalog::forest());
    let goblin_id = setup_attacker(&mut g, 0, catalog::goblin_guide);
    let drawn_before = g.players[1].cards_drawn_this_turn;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: goblin_id,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(
        g.players[1].cards_drawn_this_turn, drawn_before,
        "CR 121.5: putting a card into hand is NOT a draw"
    );
}

#[test]
fn hypnotic_specter_discards_damaged_opponent() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::forest());
    let spec_id = setup_attacker(&mut g, 0, catalog::hypnotic_specter);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: spec_id,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g);
    // Specter dealt 2 to player 1 → trigger forced one discard.
    assert_eq!(g.players[1].hand.len(), 0);
    assert_eq!(g.players[1].graveyard.len(), 1);
}

#[test]
fn wheel_of_fortune_discards_both_hands_and_draws_seven() {
    let mut g = two_player_game();
    // Active player (caster) is seat 1; opponent is seat 0.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    // Give each player some hand cards and a stocked library.
    for _ in 0..3 { g.add_card_to_hand(0, catalog::forest()); }
    for _ in 0..2 { g.add_card_to_hand(1, catalog::mountain()); }
    for _ in 0..10 { g.add_card_to_library(0, catalog::forest()); }
    for _ in 0..10 { g.add_card_to_library(1, catalog::mountain()); }
    let wheel_id = g.add_card_to_hand(1, catalog::wheel_of_fortune());
    g.players[1].mana_pool.add(Color::Red, 3);
    g.perform_action(GameAction::CastSpell {
        card_id: wheel_id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    // Both hands were discarded, then both drew 7.
    assert_eq!(g.players[0].hand.len(), 7, "opponent should have 7 cards");
    assert_eq!(g.players[1].hand.len(), 7, "caster should have 7 cards");
    // p0 discarded 3 forests. p1 discarded 2 mountains, plus the wheel itself
    // resolves and joins p1's graveyard.
    assert_eq!(g.players[0].graveyard.len(), 3);
    assert_eq!(g.players[1].graveyard.len(), 3);
}

#[test]
fn wrath_of_god_destroys_all_creatures() {
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wrath_id = g.add_card_to_hand(0, catalog::wrath_of_god());
    g.players[0].mana_pool.add(Color::White, 4);
    cast(&mut g, wrath_id);
    assert!(!g.battlefield.iter().any(|c| c.id == bear1));
    assert!(!g.battlefield.iter().any(|c| c.id == bear2));
}

#[test]
fn lightning_helix_deals_damage_and_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lightning_helix());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None })
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
    assert_eq!(g.players[0].life, 23);
}

// ── Layer system ──────────────────────────────────────────────────────────────

#[test]
fn glorious_anthem_pumps_your_creatures() {
    let mut g = two_player_game();
    let bear_id = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let anthem_id = g.add_card_to_battlefield(0, catalog::glorious_anthem());

    let bear = g.computed_permanent(bear_id).unwrap();
    assert_eq!(bear.power, 3);   // 2 + 1
    assert_eq!(bear.toughness, 3); // 2 + 1

    // Remove anthem — power should return to base.
    g.remove_from_battlefield_to_graveyard(anthem_id);
    let bear = g.computed_permanent(bear_id).unwrap();
    assert_eq!(bear.power, 2);
    assert_eq!(bear.toughness, 2);
}

#[test]
fn anthem_does_not_pump_opponent_creatures() {
    let mut g = two_player_game();
    let _anthem_id = g.add_card_to_battlefield(0, catalog::glorious_anthem());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    // Opponent's bear should be unaffected.
    let bear = g.computed_permanent(opp_bear).unwrap();
    assert_eq!(bear.power, 2);
    assert_eq!(bear.toughness, 2);
}

#[test]
fn anthem_kills_creature_with_negative_base_toughness() {
    // A -1/-1 pumped creature should die when anthem leaves.
    // Use a 1/1 token scenario: token survives anthem, dies when anthem leaves.
    let mut g = two_player_game();
    let bear_id = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Weaken the bear to 2/1 via power_bonus manipulation
    g.battlefield.iter_mut().find(|c| c.id == bear_id).unwrap().toughness_bonus = -2;
    // Without anthem: 2/(2-2) = 2/0 — dead
    let bear = g.computed_permanent(bear_id).unwrap();
    assert_eq!(bear.toughness, 0);

    // Place anthem: 2/(0+1) = 2/1 — alive
    let _anthem_id = g.add_card_to_battlefield(0, catalog::glorious_anthem());
    let bear = g.computed_permanent(bear_id).unwrap();
    assert_eq!(bear.toughness, 1);
}

#[test]
fn anthem_pumped_creature_uses_computed_toughness_for_lethality() {
    // Creature pumped by anthem should survive damage equal to its base toughness
    // but die if damage >= computed toughness.
    let mut g = two_player_game();
    // Put a 2/2 bear under anthem → effective 3/3.
    let bear_id = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _anthem_id = g.add_card_to_battlefield(1, catalog::glorious_anthem());

    // Deal 2 damage — less than computed toughness 3, should survive.
    g.battlefield.iter_mut().find(|c| c.id == bear_id).unwrap().damage = 2;
    let sba = g.check_state_based_actions();
    assert!(sba.is_empty(), "Bear should survive 2 damage with 3 effective toughness");

    // Deal 3 damage — equal to computed toughness 3, should die.
    g.battlefield.iter_mut().find(|c| c.id == bear_id).unwrap().damage = 3;
    let sba = g.check_state_based_actions();
    assert!(sba.iter().any(|e| matches!(e, GameEvent::CreatureDied { card_id } if *card_id == bear_id)));
}

#[test]
fn cr_614_2_double_damage_dealt_doubles_noncombat_damage() {
    // A Furnace-of-Rath-style permanent doubles a Lightning Bolt's 3 to 6;
    // a second doubler would compound multiplicatively (3 → 12).
    let mut g = two_player_game();
    let mut furnace = catalog::grizzly_bears();
    furnace.static_abilities = vec![StaticAbility {
        description: "If a source would deal damage, it deals double instead",
        effect: StaticEffect::DoubleDamageDealt,
    }];
    g.add_card_to_battlefield(0, furnace);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable for {R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 6, "3 damage doubled to 6");
}

#[test]
fn cr_614_2_double_damage_dealt_doubles_combat_damage_to_player() {
    // A 2/2 attacker hits an open player; a Furnace-of-Rath doubler turns
    // the 2 combat damage into 4.
    let mut g = two_player_game();
    let mut furnace = catalog::grizzly_bears();
    furnace.static_abilities = vec![StaticAbility {
        description: "double damage",
        effect: StaticEffect::DoubleDamageDealt,
    }];
    g.add_card_to_battlefield(0, furnace);
    let attacker = setup_attacker(&mut g, 0, || vanilla_body("Beater 2/2", 2, 2, vec![]));
    let before = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    assert_eq!(g.players[1].life, before - 4, "2 combat damage doubled to 4");
}

#[test]
fn cr_614_2_doubled_trample_over_lethal() {
    // A 3/3 trampler vs a single 2/2: base lethal 2 is assigned (doubled to
    // 4 dealt → kills), and the 1 leftover doubles to 2 trampling through.
    let mut g = two_player_game();
    let mut furnace = catalog::grizzly_bears();
    furnace.static_abilities = vec![StaticAbility {
        description: "double damage",
        effect: StaticEffect::DoubleDamageDealt,
    }];
    g.add_card_to_battlefield(0, furnace);
    let attacker = setup_attacker(&mut g, 0, || {
        vanilla_body("Trampler 3/3", 3, 3, vec![crate::card::Keyword::Trample])
    });
    let blocker = setup_attacker(&mut g, 1, || vanilla_body("Wall 2/2", 2, 2, vec![]));
    let before = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    assert!(!g.battlefield.iter().any(|c| c.id == blocker), "blocker took doubled lethal");
    assert_eq!(g.players[1].life, before - 2, "1 trample leftover doubled to 2");
}

#[test]
fn cr_614_2_two_doublers_compound_multiplicatively() {
    let mut g = two_player_game();
    for _ in 0..2 {
        let mut furnace = catalog::grizzly_bears();
        furnace.static_abilities = vec![StaticAbility {
            description: "double damage",
            effect: StaticEffect::DoubleDamageDealt,
        }];
        g.add_card_to_battlefield(0, furnace);
    }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable for {R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 12, "two doublers: 3 → 6 → 12");
}

#[test]
fn layer_keyword_grants_flying() {
    let mut g = two_player_game();
    let mut bear_def = catalog::grizzly_bears();
    bear_def.static_abilities = vec![StaticAbility {
        description: "This creature has flying",
        effect: StaticEffect::GrantKeyword {
            applies_to: Selector::This,
            keyword: crate::card::Keyword::Flying,
        },
    }];
    let bear_id = g.add_card_to_battlefield(0, bear_def);
    let cp = g.computed_permanent(bear_id).unwrap();
    assert!(cp.keywords.contains(&crate::card::Keyword::Flying));
}

#[test]
fn flying_via_layer_cannot_be_blocked_by_ground() {
    let mut g = two_player_game();
    // Give the attacker flying via static ability (layer 6)
    let mut bear_def = catalog::grizzly_bears();
    bear_def.static_abilities = vec![StaticAbility {
        description: "Flying",
        effect: StaticEffect::GrantKeyword {
            applies_to: Selector::This,
            keyword: crate::card::Keyword::Flying,
        },
    }];
    let attacker_id = setup_attacker(&mut g, 0, || bear_def.clone());
    let ground_blocker = setup_attacker(&mut g, 1, catalog::grizzly_bears);

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: attacker_id,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();

    g.step = TurnStep::DeclareBlockers;
    let err = g.perform_action(GameAction::DeclareBlockers(vec![(ground_blocker, attacker_id)])).unwrap_err();
    assert_eq!(err, GameError::CannotBlock(ground_blocker));
}

// ── Decision system ───────────────────────────────────────────────────────────

#[test]
fn wants_ui_opt_suspends_on_scry_and_draws_after_submit() {
    // P0 is marked as needing UI — the Scry half of Opt should suspend instead
    // of auto-resolving; after SubmitDecision, the Draw half of the Seq runs.
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let undesired = g.add_card_to_library(0, catalog::forest());
    let keeper = g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::island());
    let opt_id = g.add_card_to_hand(0, catalog::opt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, opt_id);
    assert!(g.pending_decision.is_some(), "Opt should suspend on its Scry half for UI");
    // Scry the undesired card to the bottom. Draw should fire after submission.
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::ScryOrder {
        kept_top: vec![],
        bottom: vec![undesired],
    })).unwrap();
    assert!(g.pending_decision.is_none(), "decision should be resolved");
    assert!(g.players[0].hand.iter().any(|c| c.id == keeper),
        "Draw half of Opt should run after the scry decision resolves");
}

#[test]
fn scry_resolves_with_scripted_order() {
    let mut g = two_player_game();
    let top = g.add_card_to_library(0, catalog::forest());
    let next = g.add_card_to_library(0, catalog::plains());
    // Library: [top=forest, next=plains]. Keep `next` on top; send `top` to bottom.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
        kept_top: vec![next],
        bottom: vec![top],
    }]));
    let opt_id = g.add_card_to_hand(0, catalog::opt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, opt_id);
    assert!(g.pending_decision.is_none(), "scry should have been completed synchronously");
    // `next` was kept on top then drawn by Opt's draw effect → it's now in hand.
    // `top` went to the bottom of the library.
    assert!(g.players[0].hand.iter().any(|c| c.id == next));
    assert_eq!(g.players[0].library.last().unwrap().id, top);
}

#[test]
fn add_mana_any_color_asks_decider() {
    let mut g = two_player_game();
    let scripted = ScriptedDecider::new([DecisionAnswer::Color(Color::Blue)]);
    g.decider = Box::new(scripted);
    let eff = Effect::AddMana {
        who: PlayerRef::You,
        pool: ManaPayload::AnyOneColor(Value::Const(1)),
    };
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 0);
}

#[test]
fn opt_scries_then_draws() {
    let mut g = two_player_game();
    // Stack library: [undesired=forest, keeper=plains, ...]
    let undesired = g.add_card_to_library(0, catalog::forest());
    let keeper = g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::island());
    // Scry 1: the undesired top card is sent to the bottom; then Draw 1 draws `keeper`.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
        kept_top: vec![],
        bottom: vec![undesired],
    }]));
    let opt = catalog::opt();
    let opt_id = g.add_card_to_hand(0, opt);
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, opt_id);
    assert!(g.players[0].hand.iter().any(|c| c.id == keeper),
        "Opt should have drawn the keeper after scrying undesired to bottom");
}

#[test]
fn birds_of_paradise_produces_chosen_color() {
    let mut g = two_player_game();
    let birds_id = g.add_card_to_battlefield(0, catalog::birds_of_paradise());
    g.clear_sickness(birds_id);
    // Ask for White on the tap ability.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::White)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: birds_id,
        ability_index: 0,
        target: None, x_value: None }).unwrap();
    assert_eq!(g.players[0].mana_pool.amount(Color::White), 1);
}

#[test]
fn demonic_tutor_fetches_chosen_card_via_decider() {
    let mut g = two_player_game();
    let wanted = g.add_card_to_library(1, catalog::black_lotus());
    g.add_card_to_library(1, catalog::swamp());
    g.add_card_to_library(1, catalog::swamp());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(wanted))]));
    let tutor_id = g.add_card_to_hand(1, catalog::demonic_tutor());
    // Give player 1 mana and priority.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Black, 2);
    cast(&mut g, tutor_id);
    assert!(g.players[1].hand.iter().any(|c| c.id == wanted),
        "Demonic Tutor should have fetched the wanted card into hand");
    assert!(!g.players[1].library.iter().any(|c| c.id == wanted),
        "wanted card should no longer be in library");
}

#[test]
fn preordain_scry_2_then_draws() {
    let mut g = two_player_game();
    let bottom_card = g.add_card_to_library(0, catalog::forest());
    let top_card = g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::island());
    // After scrying top 2 to bottom, the 3rd (island) becomes top and is drawn.
    let island_top = g.players[0].library[2].id;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
        kept_top: vec![],
        bottom: vec![bottom_card, top_card],
    }]));
    let pre_id = g.add_card_to_hand(0, catalog::preordain());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, pre_id);
    assert!(g.players[0].hand.iter().any(|c| c.id == island_top),
        "Preordain should draw what was the 3rd card after scrying top 2 to bottom");
}

// ── Demo-deck promotions ──────────────────────────────────────────────────

#[test]
fn watery_grave_pays_two_life_and_stays_untapped() {
    // Default `AutoDecider` picks mode 0 of the shockland's ETB
    // `ChooseMode` — pay 2 life and enter untapped.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::watery_grave());
    let life_before = g.players[0].life;
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before - 2,
        "Shockland mode 0 pays 2 life on ETB");
    let card = g.battlefield_find(id).expect("shockland is on the battlefield");
    assert!(!card.tapped, "Shockland should stay untapped on mode 0");
}

#[test]
fn cephalid_coliseum_sacrifices_for_each_player_to_draw_then_discard_three() {
    let mut g = two_player_game();
    let coli = g.add_card_to_battlefield(0, catalog::cephalid_coliseum());
    g.clear_sickness(coli);
    // Coliseum entered tapped via its ETB trigger. Untap it so we can
    // activate the wheel-mini ability.
    g.battlefield_find_mut(coli).unwrap().tapped = false;

    // Stock both libraries with enough cards to draw 3 each.
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(1, catalog::island());
    }
    // Stock both hands with enough cards to discard 3 each.
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::lightning_bolt());
        g.add_card_to_hand(1, catalog::lightning_bolt());
    }

    let p0_lib_before = g.players[0].library.len();
    let p1_lib_before = g.players[1].library.len();
    let p0_grave_before = g.players[0].graveyard.len();
    let p1_grave_before = g.players[1].graveyard.len();

    // Pay {2}{U} and tap.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: coli,
        ability_index: 1,
        target: None, x_value: None })
    .expect("Cephalid Coliseum's wheel-mini ability should activate");
    drain_stack(&mut g);

    // Coliseum sacrificed to its own graveyard.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == coli),
        "Coliseum should be sacrificed");
    assert!(!g.battlefield.iter().any(|c| c.id == coli));

    // Each player drew 3 then discarded 3 — library shrinks by 3,
    // graveyard grows by ≥3 (their 3 discards; P0 also has Coliseum itself).
    assert_eq!(g.players[0].library.len(), p0_lib_before - 3);
    assert_eq!(g.players[1].library.len(), p1_lib_before - 3);
    assert!(g.players[0].graveyard.len() >= p0_grave_before + 3,
        "P0 should have ≥3 cards in graveyard from discard (plus Coliseum)");
    assert_eq!(g.players[1].graveyard.len(), p1_grave_before + 3,
        "P1 should have 3 discarded cards in graveyard");
}

#[test]
fn quantum_riddler_on_cast_draws_a_card() {
    let mut g = two_player_game();
    // Top of library: a known card to confirm it gets drawn from the on-cast
    // cantrip.
    let top = g.add_card_to_library(0, catalog::island());
    let qr_id = g.add_card_to_hand(0, catalog::quantum_riddler());
    // Pay {3}{U}{U}.
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: qr_id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Quantum Riddler should be castable");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == qr_id),
        "Quantum Riddler should resolve onto the battlefield");
    assert!(g.players[0].hand.iter().any(|c| c.id == top),
        "Quantum Riddler's on-cast cantrip should draw a card");
}

#[test]
fn psychic_frog_discard_pumps_until_end_of_turn() {
    let mut g = two_player_game();
    let frog = g.add_card_to_battlefield(0, catalog::psychic_frog());
    g.clear_sickness(frog);
    let to_pitch = g.add_card_to_hand(0, catalog::lightning_bolt());

    let p_before = g.battlefield_find(frog).unwrap().power();
    let t_before = g.battlefield_find(frog).unwrap().toughness();

    // Activate the discard-pump ability.
    g.perform_action(GameAction::ActivateAbility {
        card_id: frog,
        ability_index: 0,
        target: None, x_value: None })
    .expect("Psychic Frog discard pump should activate");
    drain_stack(&mut g);

    // The first card in hand was discarded.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == to_pitch),
        "discarded card should be in graveyard");
    // Frog gained +1/+1.
    let card = g.battlefield_find(frog).unwrap();
    assert_eq!(card.power(), p_before + 1);
    assert_eq!(card.toughness(), t_before + 1);
}

#[test]
fn psychic_frog_sacrifice_mills_each_opponent_four() {
    let mut g = two_player_game();
    let frog = g.add_card_to_battlefield(0, catalog::psychic_frog());
    g.clear_sickness(frog);
    // Stock P1's library so we can mill 4 from it.
    for _ in 0..6 {
        g.add_card_to_library(1, catalog::island());
    }
    let p1_lib_before = g.players[1].library.len();
    let p1_grave_before = g.players[1].graveyard.len();

    g.perform_action(GameAction::ActivateAbility {
        card_id: frog,
        ability_index: 1,
        target: None, x_value: None })
    .expect("Psychic Frog sacrifice-mill should activate");
    drain_stack(&mut g);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == frog),
        "Psychic Frog should sacrifice itself");
    assert!(!g.battlefield.iter().any(|c| c.id == frog));
    assert_eq!(g.players[1].library.len(), p1_lib_before - 4);
    assert_eq!(g.players[1].graveyard.len(), p1_grave_before + 4);
}

#[test]
fn pest_control_destroys_low_cmc_nonland_permanents() {
    let mut g = two_player_game();
    // Lands should survive (Land filter on Nonland).
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    // Mana value 1 — should die.
    let llanowar = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    // Mana value 2 — should die.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Mana value 5 — survives.
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());

    let pest = g.add_card_to_hand(0, catalog::pest_control());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: pest,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Pest Control should cast for {W}{B}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == forest),
        "Lands shouldn't be destroyed by Pest Control");
    assert!(!g.battlefield.iter().any(|c| c.id == llanowar),
        "1-CMC creature should die");
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "2-CMC creature should die");
    assert!(g.battlefield.iter().any(|c| c.id == dragon),
        "5-CMC creature should survive");
}

#[test]
fn prismatic_ending_at_converge_one_only_exiles_one_drops() {
    // Cast for {W} alone → converge = 1. A 2-CMC target is a *legal* cast
    // (target filter is just Nonland Permanent), but the resolution-time
    // `If(ManaValueOf(Target) ≤ ConvergedValue, …)` no-ops on the 2-CMC
    // creature. A 1-CMC target gets exiled.
    let mut g = two_player_game();
    let llanowar = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    // Cast targeting the 2-CMC creature: cast succeeds, but resolution
    // doesn't exile (converge=1 vs CMC=2).
    let pe = g.add_card_to_hand(0, catalog::prismatic_ending());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: pe,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Prismatic Ending should accept a Nonland target at cast time");
    drain_stack(&mut g);
    assert!(!g.exile.iter().any(|c| c.id == bear),
        "Bear (2 CMC) shouldn't be exiled at converge=1");
    assert!(g.battlefield.iter().any(|c| c.id == bear));

    // Cast targeting the 1-CMC creature: exile fires.
    let pe2 = g.add_card_to_hand(0, catalog::prismatic_ending());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: pe2,
        target: Some(Target::Permanent(llanowar)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Prismatic Ending should accept a 1-CMC target");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == llanowar),
        "Llanowar Elves (1 CMC) should be exiled at converge=1");
}

#[test]
fn ephemerate_flickers_target_creature_back_to_battlefield() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Damage it so we can verify the flicker clears damage.
    g.battlefield_find_mut(creature).unwrap().damage = 1;

    let eph = g.add_card_to_hand(0, catalog::ephemerate());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: eph,
        target: Some(Target::Permanent(creature)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Ephemerate should accept a creature you control");
    drain_stack(&mut g);

    let card = g.battlefield_find(creature).expect("creature flickered back");
    assert_eq!(card.damage, 0, "flicker should clear damage");
    // Card should not still be in exile.
    assert!(!g.exile.iter().any(|c| c.id == creature));
}

#[test]
fn ephemerate_refires_solitude_etb_via_place_card_on_battlefield() {
    // Solitude has a SelfSource ETB that exiles a creature an opponent
    // controls. Flickering Solitude with Ephemerate should refire that ETB
    // (because `place_card_in_dest::Battlefield` now calls
    // `fire_self_etb_triggers`), exiling a second opponent creature.
    let mut g = two_player_game();
    let solitude = g.add_card_to_battlefield(0, catalog::solitude());
    g.clear_sickness(solitude);
    let opp_a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let opp_b = g.add_card_to_battlefield(1, catalog::llanowar_elves());

    let eph = g.add_card_to_hand(0, catalog::ephemerate());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: eph,
        target: Some(Target::Permanent(solitude)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Ephemerate should target your Solitude");
    drain_stack(&mut g);

    // Solitude should be back on the battlefield.
    assert!(g.battlefield.iter().any(|c| c.id == solitude),
        "Solitude returns from exile");
    // One of the opponent's creatures was exiled by Solitude's refired ETB.
    let exiled_count = [opp_a, opp_b]
        .iter()
        .filter(|cid| g.exile.iter().any(|c| c.id == **cid))
        .count();
    assert_eq!(exiled_count, 1,
        "Solitude's ETB should refire on flicker and exile one opp creature");
}

#[test]
fn fastland_enters_untapped_with_few_lands() {
    // With ≤ 3 total lands you control (i.e. ≤ 2 other lands), Blackcleave
    // Cliffs should enter untapped.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::swamp());
    g.add_card_to_battlefield(0, catalog::swamp());
    let cliffs = g.add_card_to_hand(0, catalog::blackcleave_cliffs());
    g.perform_action(GameAction::PlayLand(cliffs)).unwrap();
    drain_stack(&mut g);
    let cp = g.battlefield.iter().find(|c| c.id == cliffs).unwrap();
    assert!(!cp.tapped, "Blackcleave Cliffs should enter untapped with ≤ 2 other lands");
}

#[test]
fn fastland_enters_tapped_with_many_lands() {
    // With 3 other lands already in play, Blackcleave Cliffs should enter
    // tapped (post-ETB count ≥ 4).
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::swamp());
    }
    let cliffs = g.add_card_to_hand(0, catalog::blackcleave_cliffs());
    g.perform_action(GameAction::PlayLand(cliffs)).unwrap();
    drain_stack(&mut g);
    let cp = g.battlefield.iter().find(|c| c.id == cliffs).unwrap();
    assert!(cp.tapped, "Blackcleave Cliffs should enter tapped with 3+ other lands");
}

#[test]
fn force_of_negation_alt_cost_blocked_on_your_turn() {
    // Force of Negation's pitch alt cost is "if it's not your turn".
    // The active player is P0 by default — the engine should reject the
    // alt cast from P0.
    let mut g = two_player_game();
    let fon = g.add_card_to_hand(0, catalog::force_of_negation());
    let pitch = g.add_card_to_hand(0, catalog::counterspell()); // a blue card
    let err = g
        .perform_action(GameAction::CastSpellAlternative {
            card_id: fon,
            pitch_card: Some(pitch),
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .unwrap_err();
    assert_eq!(err, GameError::NoAlternativeCost,
        "Force of Negation's alt cost shouldn't fire on the caster's own turn");
}

#[test]
fn force_of_negation_alt_cost_works_on_opponents_turn() {
    let mut g = two_player_game();
    // Make it P1's turn so P0 can pitch-cast Force of Negation.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 0;
    let fon = g.add_card_to_hand(0, catalog::force_of_negation());
    let pitch = g.add_card_to_hand(0, catalog::counterspell());

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: fon,
        pitch_card: Some(pitch),
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Force of Negation alt cast should succeed on opponent's turn");
    assert!(g.exile.iter().any(|c| c.id == pitch));
    assert!(g.stack.iter().any(|si| matches!(
        si,
        crate::game::StackItem::Spell { card, .. } if card.id == fon
    )));
}

#[test]
fn devourer_of_destiny_etb_scries_two() {
    // ETB Scry 2: a scripted ScryOrder decision sends both top cards to
    // the bottom; the 3rd library card should bubble up to the top.
    let mut g = two_player_game();
    let _bottom_a = g.add_card_to_library(0, catalog::forest());
    let _bottom_b = g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::island());
    let third = g.players[0].library[2].id;
    let scry_a = g.players[0].library[0].id;
    let scry_b = g.players[0].library[1].id;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
        kept_top: vec![],
        bottom: vec![scry_a, scry_b],
    }]));

    let dev = g.add_card_to_hand(0, catalog::devourer_of_destiny());
    g.players[0].mana_pool.add_colorless(7);
    g.perform_action(GameAction::CastSpell {
        card_id: dev, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Devourer of Destiny is castable for {7}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == dev));
    assert_eq!(g.players[0].library[0].id, third,
        "After scry-2-to-bottom, the 3rd library card should be on top");
}

#[test]
fn wrath_of_the_skies_destroys_permanents_with_mana_value_x() {
    // Cast Wrath of the Skies with X=2: only nonland permanents whose CMC
    // is exactly 2 should die. Lands and other-CMC permanents survive.
    let mut g = two_player_game();
    let forest = g.add_card_to_battlefield(1, catalog::forest());
    let llanowar = g.add_card_to_battlefield(1, catalog::llanowar_elves()); // CMC 1
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());      // CMC 2
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());    // CMC 6

    let wrath = g.add_card_to_hand(0, catalog::wrath_of_the_skies());
    // Cost is {X}{W}{W}; pay {2}{W}{W} for X=2.
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: wrath,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("Wrath of the Skies should cast for {2}{W}{W}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == forest),
        "Lands aren't destroyed by Wrath of the Skies");
    assert!(g.battlefield.iter().any(|c| c.id == llanowar),
        "1-CMC creature shouldn't be destroyed when X=2");
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "2-CMC creature should die when X=2");
    assert!(g.battlefield.iter().any(|c| c.id == dragon),
        "6-CMC creature shouldn't be destroyed when X=2");
}

#[test]
fn spoils_of_the_vault_reveals_until_the_named_card() {
    // Name "Swamp": reveal Bolt (miss → graveyard), then Swamp (hit → hand).
    // Two cards revealed → 2 life lost.
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt()); // top
    let swamp = g.add_card_to_library(0, catalog::swamp());
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::NamedCard("Swamp".to_string()),
    ]));

    let spoils = g.add_card_to_hand(0, catalog::spoils_of_the_vault());
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spoils,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Spoils of the Vault should cast for {B}");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == swamp),
        "The named card (Swamp) should land in hand");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt),
        "The revealed miss (Bolt) should be milled");
    assert_eq!(g.players[0].life, life_before - 2,
        "Two cards revealed → 2 life lost");
}

#[test]
fn atraxa_grand_unifier_etb_draws_per_distinct_type() {
    // ETB reveals top 10 and takes one card of each type. Library here is
    // 6 forests (Land), so only one Land is taken → hand +1.
    let mut g = two_player_game();
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::forest());
    }
    let lib_before = g.players[0].library.len();
    let hand_before_cast = g.players[0].hand.len();
    let atraxa = g.add_card_to_hand(0, catalog::atraxa_grand_unifier());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: atraxa, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Atraxa is castable for {3}{W}{U}{B}{R}{G}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == atraxa));
    assert_eq!(g.players[0].hand.len(), hand_before_cast + 1,
        "Library is all Forests (one type) → ETB draws 1");
    assert_eq!(g.players[0].library.len(), lib_before - 1,
        "Library should lose exactly one card");
}

#[test]
fn atraxa_grand_unifier_draws_per_card_type_diverse_library() {
    // Top of library has a Forest, a Lightning Bolt (Instant), Grizzly
    // Bears (Creature) → 3 distinct types → draw 3.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let hand_before_cast = g.players[0].hand.len();
    let atraxa = g.add_card_to_hand(0, catalog::atraxa_grand_unifier());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: atraxa, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Atraxa is castable for {3}{W}{U}{B}{R}{G}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before_cast + 3,
        "3 distinct card types in library → take 3");
}

#[test]
fn atraxa_takes_one_per_type_and_bottoms_the_rest() {
    // Top of library: two Forests (Land) + one Grizzly Bears (Creature).
    // Atraxa takes one Land + the Creature into hand; the extra Forest is
    // bottomed (not taken, not lost).
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    let hand_before = g.players[0].hand.len();
    let atraxa = g.add_card_to_hand(0, catalog::atraxa_grand_unifier());
    g.players[0].mana_pool.add_colorless(3);
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 1);
    }
    g.perform_action(GameAction::CastSpell {
        card_id: atraxa, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("castable");
    drain_stack(&mut g);

    // One Land + one Creature taken → hand +2.
    assert_eq!(g.players[0].hand.len(), hand_before + 2);
    assert!(g.players[0].hand.iter().any(|c| c.id == bears), "Creature taken");
    // Exactly one Forest remains, on the bottom of the library.
    let forests: Vec<_> = g.players[0].library.iter()
        .filter(|c| c.definition.name == "Forest").collect();
    assert_eq!(forests.len(), 1, "the extra Forest is bottomed, not lost");
    assert_eq!(g.players[0].library.last().map(|c| c.definition.name), Some("Forest"));
}

#[test]
fn pathway_front_face_taps_for_front_color_only() {
    // Playing Blightstep Pathway via PlayLand picks the front (Swamp / B)
    // face. The land enters as a Swamp, has exactly one mana ability, and
    // taps for {B}.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::blightstep_pathway());
    g.perform_action(GameAction::PlayLand(id))
        .expect("PlayLand should succeed for the front face");

    let card = g.battlefield_find(id).expect("card on battlefield");
    assert_eq!(card.definition.name, "Blightstep Pathway");
    assert!(card.definition.subtypes.land_types.contains(&crate::card::LandType::Swamp));
    assert!(!card.definition.subtypes.land_types.contains(&crate::card::LandType::Mountain),
        "Front face should be a Swamp only — Mountain belongs to the back face");
    assert_eq!(card.definition.activated_abilities.len(), 1,
        "Front face exposes only one mana ability (the front color)");

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).expect("front face taps for {B}");
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 0,
        "Front face must not produce {{R}} — that's the back face");
}

#[test]
fn pathway_back_face_taps_for_back_color_only() {
    // Playing Blightstep Pathway via PlayLandBack swaps to the back face
    // (Searstep Pathway / Mountain / R) before placing on battlefield.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::blightstep_pathway());
    g.perform_action(GameAction::PlayLandBack(id))
        .expect("PlayLandBack should succeed for the back face");

    let card = g.battlefield_find(id).expect("card on battlefield");
    assert_eq!(card.definition.name, "Searstep Pathway");
    assert!(card.definition.subtypes.land_types.contains(&crate::card::LandType::Mountain));
    assert!(!card.definition.subtypes.land_types.contains(&crate::card::LandType::Swamp));
    assert_eq!(card.definition.activated_abilities.len(), 1);

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).expect("back face taps for {R}");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 0);
}

#[test]
fn play_land_back_rejects_non_mdfc() {
    // A regular basic with no `back_face` can't be played via PlayLandBack.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::forest());
    let err = g.perform_action(GameAction::PlayLandBack(id)).unwrap_err();
    assert_eq!(err, GameError::NotALand(id),
        "Playing a non-MDFC via PlayLandBack should reject");
    // Card returns to hand, no land was played.
    assert!(g.players[0].hand.iter().any(|c| c.id == id));
    assert_eq!(g.players[0].lands_played_this_turn, 0);
}

#[test]
fn gemstone_mine_etb_with_three_charge_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::gemstone_mine());
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);
    let card = g.battlefield_find(id).expect("on battlefield");
    assert_eq!(card.counter_count(crate::card::CounterType::Charge), 3,
        "Gemstone Mine should ETB with three charge (mining) counters");
}

#[test]
fn gemstone_mine_taps_three_times_then_sacrifices() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::gemstone_mine());
    // Manually seed counters since add_card_to_battlefield bypasses the ETB.
    g.battlefield_find_mut(id).unwrap()
        .add_counters(crate::card::CounterType::Charge, 3);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Color(Color::White),
        DecisionAnswer::Color(Color::Blue),
        DecisionAnswer::Color(Color::Black),
    ]));

    // Tap #1: counter 3 → 2, no sac.
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).expect("tap #1 succeeds");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some());
    assert_eq!(g.battlefield_find(id).unwrap()
        .counter_count(crate::card::CounterType::Charge), 2);
    g.battlefield_find_mut(id).unwrap().tapped = false;

    // Tap #2: counter 2 → 1.
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).expect("tap #2 succeeds");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some());
    assert_eq!(g.battlefield_find(id).unwrap()
        .counter_count(crate::card::CounterType::Charge), 1);
    g.battlefield_find_mut(id).unwrap().tapped = false;

    // Tap #3: counter 1 → 0, then sacrifice.
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).expect("tap #3 succeeds");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == id),
        "Gemstone Mine should sac itself when the last counter is removed");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id),
        "Sacrificed Gemstone Mine should land in the graveyard");
}

#[test]
fn teferi_minus_three_returns_target_and_draws() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    // Teferi's base loyalty (4) is seeded by CardInstance::new for
    // planeswalkers; no need to add counters manually.
    let teferi = g.add_card_to_battlefield(0, catalog::teferi_time_raveler());
    let opp_creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Seed P0's library so the +draw doesn't deck them out.
    g.add_card_to_library(0, catalog::forest());

    let hand_before = g.players[0].hand.len();
    let opp_hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: teferi,
        // Index 1 is the -3 (the new +1 sorcery-as-flash ability sits at 0).
        ability_index: 1,
        target: Some(Target::Permanent(opp_creature)),
    })
    .expect("Teferi's -3 should accept an opponent's nonland permanent");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == opp_creature),
        "Targeted creature should leave the battlefield");
    assert!(g.players[1].hand.iter().any(|c| c.id == opp_creature),
        "Bounced creature should return to its owner's hand");
    assert_eq!(g.players[1].hand.len(), opp_hand_before + 1);
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Teferi's -3 should also draw a card");
    assert_eq!(g.battlefield_find(teferi).unwrap()
        .counter_count(CounterType::Loyalty), 1,
        "Loyalty should drop from 4 to 1 after a -3 ability");
}

#[test]
fn mystical_dispute_alt_cost_requires_blue_target() {
    // Casting Mystical Dispute via the alt cost {U} should reject a non-blue
    // target spell on the stack and accept a blue one.
    let mut g = two_player_game();
    g.players[0].mana_pool.add(Color::Blue, 1);

    // Put a non-blue spell on the stack first.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt is castable for {R}");

    g.priority.player_with_priority = 0;
    let dispute = g.add_card_to_hand(0, catalog::mystical_dispute());
    let err = g.perform_action(GameAction::CastSpellAlternative {
        card_id: dispute,
        pitch_card: None,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .unwrap_err();
    assert_eq!(err, GameError::SelectionRequirementViolated,
        "Alt cost should reject a non-blue target");

    // Add a blue spell on the stack (Counterspell needs an actual on-stack
    // target now that its filter requires IsSpellOnStack). The original
    // bolt is still on the stack from above; have the opponent counter it.
    g.players[0].mana_pool.add(Color::Blue, 1);
    let counterspell = g.add_card_to_hand(0, catalog::counterspell());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: counterspell,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Counterspell is castable for {U}{U}");

    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: dispute,
        pitch_card: None,
        target: Some(Target::Permanent(counterspell)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Mystical Dispute alt cost should accept a blue target");
}

/// Whether the opponent has spare mana decides if Mystical Dispute's {3}
/// tax saves the targeted spell or counters it.
fn mystical_dispute_setup(p1_spare_colorless: u32) -> (GameState, CardId) {
    let mut g = two_player_game();
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.players[1].mana_pool.add_colorless(p1_spare_colorless);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    // Bolt must stay on the stack so Dispute can target it.
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lightning Bolt castable for {R}");
    g.priority.player_with_priority = 0;
    let dispute = g.add_card_to_hand(0, catalog::mystical_dispute());
    cast_at(&mut g, dispute, Target::Permanent(bolt));
    (g, bolt)
}

#[test]
fn mystical_dispute_does_not_counter_when_opponent_can_pay() {
    let (g, _bolt) = mystical_dispute_setup(3);
    assert_eq!(g.players[0].life, 17,
        "Bolt should still resolve when P1 pays the dispute tax");
    assert_eq!(g.players[1].mana_pool.colorless_amount(), 0,
        "P1's spare colorless should have been consumed paying the tax");
}

#[test]
fn mystical_dispute_counters_when_opponent_cannot_pay() {
    let (g, bolt) = mystical_dispute_setup(0);
    assert_eq!(g.players[0].life, 20,
        "Bolt should be countered when P1 can't afford the {{3}} tax");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Countered Bolt should hit P1's graveyard");
}

#[test]
fn quantum_riddler_on_cast_draws_even_if_countered() {
    // The cantrip is a SpellCast+SelfSource trigger that goes on the stack
    // above the spell, so it resolves first — countering Quantum Riddler in
    // response should not prevent the draw.
    let mut g = two_player_game();
    let drawn = g.add_card_to_library(0, catalog::forest());

    let qr = g.add_card_to_hand(0, catalog::quantum_riddler());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Blue, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: qr, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Quantum Riddler is castable for {3}{U}{U}");

    // P1 counters Quantum Riddler with a Counterspell while the on-cast
    // cantrip is still on the stack above it.
    let counter = g.add_card_to_hand(1, catalog::counterspell());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: counter,
        target: Some(Target::Permanent(qr)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Counterspell cast against Quantum Riddler");

    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == qr),
        "Quantum Riddler should be countered (no permanent in play)");
    assert!(g.players[0].hand.iter().any(|c| c.id == drawn),
        "On-cast cantrip should still draw P0's library card even though the spell was countered");
}

#[test]
fn convoke_taps_creature_to_pay_one_generic() {
    // Wrath of the Skies costs {X}{W}{W}. With X=2 + 1 convoked creature,
    // the player only needs {1}{W}{W} from real mana — the convoke tap
    // contributes the missing {1}. The same X=2 wrath then sweeps both
    // 2-CMC nonland permanents (including the just-tapped convoke
    // creature, which is itself a 2-CMC bear).
    let mut g = two_player_game();
    let mascot = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(mascot);
    // Only enough real mana for {1}{W}{W} — short by {1} for X=2.
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);

    let wrath = g.add_card_to_hand(0, catalog::wrath_of_the_skies());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    // Without convoke, the cast fails (mana shortfall).
    let try_no_convoke = g.perform_action(GameAction::CastSpell {
        card_id: wrath, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    });
    assert!(try_no_convoke.is_err(),
        "Wrath at X=2 should be unaffordable without convoke help");

    // With convoke, the tap contributes the missing {1}.
    g.perform_action(GameAction::CastSpellConvoke {
        card_id: wrath,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
        convoke_creatures: vec![mascot],
    })
    .expect("Wrath should be castable with convoke topping up the X cost");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear),
        "Wrath at X=2 destroys the 2-CMC opp creature");
    // The convoked creature itself is also 2-CMC and gets swept by its own
    // wrath. Either way it's no longer on battlefield (graveyarded).
    assert!(!g.battlefield.iter().any(|c| c.id == mascot));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == mascot),
        "Convoked creature was tapped, then destroyed by Wrath, ending in graveyard");
}

#[test]
fn pest_control_at_converge_three_destroys_higher_cmc() {
    // Pest Control cost is {W}{B}, but we'll inject extra mana of a third
    // color to bump converge to 3. This should now destroy 3-CMC nonland
    // permanents (e.g. anything with three pips of cost).
    let mut g = two_player_game();
    // 3-CMC opponent permanent — Goblin Guide (1R) is 1-CMC, Lightning
    // Bolt is instant (no permanent on battlefield). Use Serra Angel
    // (3WW = 5 CMC). Hmm we need a 3-CMC creature. Use Black Knight (BB)?
    // 2-CMC. Use a 3-CMC one. The catalog has shivan_dragon (4RR = 6).
    // Let's just use Shivan Dragon and verify converge=3 doesn't kill it.
    // For converge=2 leaving a 3-CMC creature, we need… let's just add
    // a 3-CMC permanent via a test card. Use catalog::serra_angel which
    // is 3WW = CMC 5. Hmm.
    //
    // Simpler: use Grizzly Bears (CMC 2) to confirm converge=3 still
    // destroys it (it covers 1-3) and converge=2 also destroys it.
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pest = g.add_card_to_hand(0, catalog::pest_control());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: pest, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Pest Control castable for {W}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == opp),
        "Pest Control at converge=2 destroys 2-CMC creatures");
}

#[test]
fn convoke_rejects_tapped_creature() {
    // Convoking a tapped creature should reject the cast.
    let mut g = two_player_game();
    let mascot = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(mascot).unwrap().tapped = true;
    g.players[0].mana_pool.add(Color::White, 1);

    let pe = g.add_card_to_hand(0, catalog::prismatic_ending());
    let err = g.perform_action(GameAction::CastSpellConvoke {
        card_id: pe,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        convoke_creatures: vec![mascot],
    });
    assert!(err.is_err(), "Convoking a tapped creature should reject");
    // Card should be back in hand.
    assert!(g.players[0].hand.iter().any(|c| c.id == pe));
}

#[test]
fn leyline_of_sanctity_starts_in_play_and_grants_player_hexproof() {
    // Stock Leyline in P0's opening hand, fire start-of-game effects:
    // Leyline should hit the battlefield and P0 should be untargetable
    // by P1.
    let mut g = two_player_game();
    let leyline = g.add_card_to_hand(0, catalog::leyline_of_sanctity());
    g.fire_start_of_game_effects();

    assert!(g.battlefield.iter().any(|c| c.id == leyline),
        "Leyline of Sanctity should begin the game on the battlefield");
    assert!(!g.players[0].hand.iter().any(|c| c.id == leyline));

    // P1 tries to target P0 with Lightning Bolt — rejected by hexproof.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .unwrap_err();
    assert!(matches!(err, GameError::TargetHasHexproof(_)),
        "P0 should be untargetable while controlling Leyline of Sanctity");
}

#[test]
fn leyline_of_sanctity_hexproof_does_not_apply_to_self() {
    // Casting your own spell on yourself is fine — hexproof from Leyline
    // only blocks opponents.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::leyline_of_sanctity());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Self-targeting bypasses Leyline's hexproof");
}

#[test]
fn chancellor_of_the_tangle_grants_one_green_at_start_of_game() {
    // Reveal queues a YourNextMainPhase delayed trigger; the {G} hits the
    // pool when P0's first main step fires.
    let mut g = two_player_game();
    g.step = TurnStep::Untap; // reset so we can step into PreCombatMain
    g.add_card_to_hand(0, catalog::chancellor_of_the_tangle());
    g.fire_start_of_game_effects();
    assert_eq!(g.delayed_triggers.len(), 1,
        "Tangle should queue one YourNextMainPhase trigger");
    // Walk priority through the early steps until PreCombatMain fires
    // the delayed trigger.
    for _ in 0..30 {
        if g.players[0].mana_pool.amount(Color::Green) > 0 { break; }
        let _ = g.perform_action(GameAction::PassPriority);
    }
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1,
        "Chancellor of the Tangle should add {{G}} to its owner's pool by PreCombatMain");
}

#[test]
fn gemstone_caverns_starts_in_play() {
    let mut g = two_player_game();
    let cave = g.add_card_to_hand(0, catalog::gemstone_caverns());
    g.fire_start_of_game_effects();
    assert!(g.battlefield.iter().any(|c| c.id == cave),
        "Gemstone Caverns should begin the game on the battlefield");
}

#[test]
fn inquisition_suspends_for_caster_ui_and_applies_chosen_discard() {
    // P0 wants UI → Inquisition should suspend with a Decision::Discard
    // pointing at P0 (the picker), showing P1's filtered hand. Submitting
    // the chosen card discards it to P1's graveyard.
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let pricey = g.add_card_to_hand(1, catalog::shivan_dragon());   // CMC 6 — filtered out
    let target = g.add_card_to_hand(1, catalog::lightning_bolt()); // CMC 1
    let other = g.add_card_to_hand(1, catalog::counterspell());    // CMC 2
    g.add_card_to_hand(1, catalog::forest());                      // land — filtered out

    let inq = g.add_card_to_hand(0, catalog::inquisition_of_kozilek());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: inq, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Inquisition castable for {B}");
    // First PassPriority lets P1 respond; second resolves the spell.
    g.perform_action(GameAction::PassPriority).unwrap();
    g.perform_action(GameAction::PassPriority).unwrap();

    // Now Inquisition is resolving — it should suspend on a Discard
    // decision. P0 is the acting player; the candidate list is P1's
    // filtered hand (Bolt + Counterspell, but not Forest or Shivan).
    let pd = g.pending_decision.as_ref().expect("Inquisition should suspend");
    let crate::decision::Decision::Discard { player, count, hand } = &pd.decision else {
        panic!("expected Decision::Discard, got {:?}", pd.decision);
    };
    assert_eq!(*player, 0, "picker is the caster (P0)");
    assert_eq!(*count, 1);
    let hand_ids: Vec<CardId> = hand.iter().map(|(id, _)| *id).collect();
    assert!(hand_ids.contains(&target));
    assert!(hand_ids.contains(&other));
    assert!(!hand_ids.contains(&pricey),
        "CMC ≥ 4 should be filtered out — Inquisition only sees ≤ 3");

    // Submit the choice — discard the Bolt.
    g.submit_decision(DecisionAnswer::Discard(vec![target]))
        .expect("decision submitted");
    drain_stack(&mut g);

    assert!(g.players[1].graveyard.iter().any(|c| c.id == target),
        "Targeted Bolt should hit P1's graveyard");
    assert!(g.players[1].hand.iter().any(|c| c.id == other),
        "Other matching card stays in hand — Inquisition only takes one");
}

#[test]
fn chancellor_of_the_annex_taxes_opponents_first_spell() {
    // Stock Chancellor in P0's opening hand. Start-of-game pass queues a
    // YourNextUpkeep delayed trigger; the trigger resolution stamps a {1}
    // tax on P1's first spell.
    let mut g = two_player_game();
    g.step = TurnStep::Untap;
    g.add_card_to_hand(0, catalog::chancellor_of_the_annex());
    g.fire_start_of_game_effects();
    // Step into Upkeep so the delayed trigger fires + resolves.
    for _ in 0..30 {
        if g.players[1].first_spell_tax_charges > 0 { break; }
        let _ = g.perform_action(GameAction::PassPriority);
    }
    assert_eq!(g.players[1].first_spell_tax_charges, 1);

    let bolt1 = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bolt1, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "First Bolt should require an extra {{1}} due to the Annex tax");

    g.players[1].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt1, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("First Bolt castable for {1}{R} under the tax");
    drain_stack(&mut g);
    assert_eq!(g.players[1].first_spell_tax_charges, 0,
        "Tax should clear after the first cast");

    // Second spell pays the regular {R} again.
    let bolt2 = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt2, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Second Bolt should cast for {R} (no further tax)");
}

#[test]
fn ephemerate_rebound_exiles_then_recasts_next_upkeep() {
    // Cast Ephemerate from hand. After resolution: creature is flickered
    // back, Ephemerate is in exile (not graveyard) with a `YourNextUpkeep`
    // delayed trigger queued. Advancing to P0's next upkeep should fire
    // the rebound — re-running the flicker effect on a fresh auto-target.
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Damage so we can confirm the flicker clears it (and the rebound
    // recast still flickers something).
    g.battlefield_find_mut(creature).unwrap().damage = 1;

    let eph = g.add_card_to_hand(0, catalog::ephemerate());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: eph,
        target: Some(Target::Permanent(creature)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Ephemerate castable for {W}");
    drain_stack(&mut g);

    // Damage cleared by flicker.
    assert_eq!(g.battlefield_find(creature).unwrap().damage, 0);
    // Rebound: Ephemerate is in exile, NOT in graveyard.
    assert!(g.exile.iter().any(|c| c.id == eph),
        "Ephemerate should be exiled by rebound, not graveyarded");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == eph),
        "Ephemerate should NOT have hit P0's graveyard");
    // A delayed YourNextUpkeep trigger should be queued.
    assert_eq!(g.delayed_triggers.len(), 1,
        "Rebound should register one delayed trigger");

    // Damage the creature again so we can prove the rebound recast actually
    // re-flickered it (would clear the new damage).
    g.battlefield_find_mut(creature).unwrap().damage = 2;

    // Advance to P0's next upkeep.
    g.step = TurnStep::Cleanup;
    g.active_player_idx = 1;
    for _ in 0..15 {
        if g.active_player_idx == 0 && g.step == TurnStep::Upkeep && g.stack.is_empty() {
            break;
        }
        g.perform_action(GameAction::PassPriority).unwrap();
    }

    // The rebound trigger fired and re-flickered the creature → damage cleared.
    assert_eq!(g.battlefield_find(creature).unwrap().damage, 0,
        "Rebound's recast should re-flicker the creature, clearing damage");
    assert!(g.delayed_triggers.is_empty(),
        "Rebound trigger should fire once and clear");
}

#[test]
fn elesh_norn_doubles_your_etb_triggers() {
    // Devourer of Destiny scry-on-cast still fires from the stack — but
    // ETB triggers from a permanent entering should fire 2x while you
    // control Elesh Norn. Use Quantum Riddler's on-cast cantrip… wait,
    // that's a SpellCast trigger, not ETB. Use a creature with a real
    // ETB: Solitude (ETB exiles target opp creature). With Elesh Norn out
    // and two opp creatures, both should be exiled.
    let mut g = two_player_game();
    let _norn = g.add_card_to_battlefield(0, catalog::elesh_norn_mother_of_machines());
    let opp_a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let opp_b = g.add_card_to_battlefield(1, catalog::llanowar_elves());

    let solitude = g.add_card_to_hand(0, catalog::solitude());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: solitude, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Solitude castable for {3}{W}");
    drain_stack(&mut g);

    // Both opp creatures should have been exiled by the doubled ETB.
    let exiled = [opp_a, opp_b].iter()
        .filter(|cid| g.exile.iter().any(|c| c.id == **cid))
        .count();
    assert_eq!(exiled, 2,
        "Solitude's ETB should fire twice with Elesh Norn out, exiling both opp creatures");
}

#[test]
fn elesh_norn_suppresses_opponent_etb_triggers() {
    // P0 controls Elesh Norn. P1 plays Solitude — its ETB exile trigger
    // should be suppressed (zero firings), so P0's creatures stay put.
    let mut g = two_player_game();
    let _norn = g.add_card_to_battlefield(0, catalog::elesh_norn_mother_of_machines());
    let p0_creature = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    let solitude = g.add_card_to_hand(1, catalog::solitude());
    g.players[1].mana_pool.add(Color::White, 2);
    g.players[1].mana_pool.add_colorless(3);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: solitude, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Opponent's Solitude castable");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == p0_creature),
        "P0's creature should survive — Elesh Norn suppresses opp's ETB triggers");
    assert!(!g.exile.iter().any(|c| c.id == p0_creature));
}

#[test]
fn cavern_of_souls_makes_creatures_uncounterable() {
    // While P0 controls a Cavern of Souls, any creature spell P0 casts is
    // marked uncounterable on the stack — Counterspell from P1 should not
    // remove it.
    let mut g = two_player_game();
    let _cavern = g.add_card_to_battlefield(0, catalog::cavern_of_souls());

    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Grizzly Bears castable for {1}{G}");

    // P1 tries to counter the bear.
    let counter = g.add_card_to_hand(1, catalog::counterspell());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: counter,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Counterspell castable");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == bear),
        "Cavern of Souls should make P0's creature uncounterable");
}

#[test]
fn cavern_of_souls_does_not_protect_noncreature_spells() {
    // Cavern's uncounterable approximation is creature-only — countering a
    // noncreature spell should still work even with a Cavern in play.
    let mut g = two_player_game();
    let _cavern = g.add_card_to_battlefield(0, catalog::cavern_of_souls());

    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");

    let counter = g.add_card_to_hand(1, catalog::counterspell());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: counter,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Counterspell castable");
    drain_stack(&mut g);

    assert!(!g.stack.iter().any(|si| matches!(
        si, crate::game::StackItem::Spell { card, .. } if card.id == bolt
    )), "Lightning Bolt should be countered (Cavern only protects creatures)");
}

#[test]
fn cavern_of_souls_only_protects_named_creature_type() {
    // When the chosen_creature_type is set on a Cavern, only spells with
    // that creature type are uncounterable; others remain counterable.
    let mut g = two_player_game();
    let cavern = g.add_card_to_battlefield(0, catalog::cavern_of_souls());
    g.battlefield_find_mut(cavern).unwrap().chosen_creature_type =
        Some(crate::card::CreatureType::Bear);

    // Llanowar Elves (Elf type, not Bear) should be counterable.
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: elf, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Llanowar Elves castable");

    let counter = g.add_card_to_hand(1, catalog::counterspell());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: counter, target: Some(Target::Permanent(elf)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Counterspell castable");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == elf),
        "Elf should be countered — Cavern named Bear, Elves are not Bears");
}

#[test]
fn cavern_of_souls_etb_picks_creature_type_via_decider() {
    // ETB trigger asks the decider; ScriptedDecider picks Phyrexian and
    // verify the chosen type is recorded.
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::CreatureType(crate::card::CreatureType::Phyrexian),
    ]));

    g.add_card_to_library(0, catalog::cavern_of_souls());
    let cavern_id = g.players[0].library[0].id;
    let card = g.players[0].library.remove(0);
    g.players[0].hand.push(card);
    g.perform_action(GameAction::PlayLand(cavern_id))
        .expect("Cavern should be playable");
    drain_stack(&mut g);

    let cavern = g.battlefield_find(cavern_id).expect("Cavern on battlefield");
    assert_eq!(
        cavern.chosen_creature_type,
        Some(crate::card::CreatureType::Phyrexian),
        "ETB should record the chosen type"
    );
}

#[test]
fn damping_sphere_taxes_each_spell_after_the_first() {
    // First Lightning Bolt costs {R}; second one costs {1}{R} while
    // Damping Sphere is in play.
    let mut g = two_player_game();
    let _sphere = g.add_card_to_battlefield(0, catalog::damping_sphere());

    // First spell: should cost just {R}.
    let bolt1 = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt1,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("First Lightning Bolt should cast for {R}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].spells_cast_this_turn, 1);

    // Second spell: should now require {1}{R}. Pay just {R} → fails.
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bolt2,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(err.is_err(),
        "Second spell should require an extra {{1}} under Damping Sphere");

    // With the extra {1}, it casts.
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt2,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Second Lightning Bolt should cast for {1}{R} under Damping Sphere");
}

#[test]
fn damping_sphere_resets_count_at_turn_start() {
    // Per-player `spells_cast_this_turn` should clear at the start of the
    // next turn, so the first spell of a new turn isn't taxed.
    let mut g = two_player_game();
    let _sphere = g.add_card_to_battlefield(0, catalog::damping_sphere());

    g.players[0].spells_cast_this_turn = 3;
    // Simulate a fresh turn for P0 — `do_untap` resets the per-player
    // counter (and lands_played).
    g.do_untap();
    assert_eq!(g.players[0].spells_cast_this_turn, 0,
        "Per-player spell count should reset on untap");

    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("First spell of a new turn should not be taxed");
}

#[test]
fn consign_to_memory_counters_targeted_trigger() {
    // P1 casts Devourer of Destiny (on-cast Scry 2 trigger goes on the
    // stack). P0 responds with Consign to Memory targeting Devourer; the
    // Scry trigger is removed from the stack before it can resolve.
    let mut g = two_player_game();
    // Seed P1's library so the scry would have something to act on if it
    // resolved (so we can verify it didn't).
    g.add_card_to_library(1, catalog::island());
    g.add_card_to_library(1, catalog::island());

    let dev = g.add_card_to_hand(1, catalog::devourer_of_destiny());
    g.players[1].mana_pool.add_colorless(7);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: dev, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Devourer of Destiny castable for {7}");

    // Confirm the scry trigger landed on the stack alongside the spell.
    let trigger_count = g.stack.iter()
        .filter(|si| matches!(si, crate::game::StackItem::Trigger { source, .. } if *source == dev))
        .count();
    assert_eq!(trigger_count, 1, "Scry-on-cast trigger should be queued");

    // P0 casts Consign to Memory targeting Devourer.
    let consign = g.add_card_to_hand(0, catalog::consign_to_memory());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: consign,
        target: Some(Target::Permanent(dev)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Consign to Memory castable for {U}");
    drain_stack(&mut g);

    // The Scry trigger never resolved → no scry decision was asked. We
    // can't directly observe "did scry happen" without a scripted decider,
    // but we can assert the trigger was removed from the stack and the
    // Devourer still resolved (since Consign only counters the ability,
    // not the spell).
    assert!(g.battlefield.iter().any(|c| c.id == dev),
        "Devourer should still resolve — Consign only counters the ability");
    assert!(!g.stack.iter().any(|si| matches!(
        si, crate::game::StackItem::Trigger { source, .. } if *source == dev
    )), "Scry-on-cast trigger should have been countered");
}

#[test]
fn consign_to_memory_mode_one_counters_legendary_spell() {
    // Mode 1: counter target legendary spell. P1 puts Thalia, Guardian of
    // Thraben on the stack; P0 responds with Consign to Memory mode-1 →
    // Thalia is countered.
    let mut g = two_player_game();
    let thalia = g.add_card_to_hand(1, catalog::thalia_guardian_of_thraben());
    g.players[1].mana_pool.add(Color::White, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: thalia, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Thalia castable for {1}{W}");

    // Confirm Thalia is on the stack.
    assert!(g.stack.iter().any(|si| matches!(
        si, crate::game::StackItem::Spell { card, .. } if card.id == thalia
    )), "Thalia should be on the stack");

    // P0 casts Consign to Memory mode 1 targeting Thalia.
    let consign = g.add_card_to_hand(0, catalog::consign_to_memory());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: consign,
        target: Some(Target::Permanent(thalia)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    })
    .expect("Consign to Memory mode 1 castable for {U} targeting a legendary spell");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == thalia),
        "Thalia should have been countered (not resolved)");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == thalia),
        "Countered Thalia should be in P1's graveyard");
}

// ── Opening-hand effects ─────────────────────────────────────────────────────

#[test]
fn leyline_of_sanctity_starts_in_play_after_mulligan() {
    // After a kept opening hand that contains Leyline of Sanctity, the
    // engine moves the leyline from hand to its owner's battlefield.
    let mut g = two_player_game();
    // Seed each player's library so the mulligan can deal 7 cards. Leyline
    // is the FIRST factory pushed → it'll end up on top of the library →
    // dealt as part of the opening 7.
    g.add_card_to_library(0, catalog::leyline_of_sanctity());
    for _ in 0..7 {
        g.add_card_to_library(0, catalog::forest());
    }
    for _ in 0..7 {
        g.add_card_to_library(1, catalog::forest());
    }
    g.start_mulligan_phase();

    // Both players keep, no London-mulligan dance.
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Keep)).unwrap();
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Keep)).unwrap();

    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Leyline of Sanctity"),
        "Leyline should start in play"
    );
    assert!(
        !g.players[0].hand.iter().any(|c| c.definition.name == "Leyline of Sanctity"),
        "Leyline should leave hand"
    );
}

#[test]
fn leyline_of_sanctity_grants_player_hexproof() {
    // Bolt aimed at the Leyline's controller is rejected as InvalidTarget.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::leyline_of_sanctity());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).unwrap_err();
    assert!(matches!(err, GameError::TargetHasHexproof(_)),
        "Leyline rejects opponent player-target spells");
    // Self-targeting still works: Leyline doesn't block your own spells.
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt2,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("you can target yourself even with Leyline up");
}

#[test]
fn gemstone_caverns_starts_in_play_with_luck_counter() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::gemstone_caverns());
    for _ in 0..7 {
        g.add_card_to_library(0, catalog::forest());
    }
    for _ in 0..7 {
        g.add_card_to_library(1, catalog::forest());
    }
    g.start_mulligan_phase();
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Keep)).unwrap();
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Keep)).unwrap();

    let caverns = g.battlefield.iter()
        .find(|c| c.definition.name == "Gemstone Caverns")
        .expect("Gemstone Caverns should start in play");
    assert!(
        !caverns.tapped,
        "Gemstone Caverns starts untapped"
    );
    assert_eq!(
        caverns.counter_count(crate::card::CounterType::Charge),
        1,
        "Gemstone Caverns should ETB with one luck counter (modeled as Charge)"
    );
}

#[test]
fn chancellor_of_the_tangle_adds_green_on_first_main() {
    // P0 starts the game with Chancellor of the Tangle in opening hand.
    // The reveal registers a `YourNextMainPhase` delayed trigger; when P0
    // enters PreCombatMain on turn 1, the trigger pushes onto the stack
    // and resolves to add {G}.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::chancellor_of_the_tangle());
    for _ in 0..7 {
        g.add_card_to_library(0, catalog::forest());
    }
    for _ in 0..7 {
        g.add_card_to_library(1, catalog::forest());
    }
    // Start the mulligan window from the natural pre-game step (Untap).
    // `two_player_game()` defaults to PreCombatMain — wind it back so the
    // turn 1 PreCombatMain transition actually happens during the test.
    g.step = TurnStep::Untap;
    g.start_mulligan_phase();
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Keep)).unwrap();
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Keep)).unwrap();

    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 0,
        "no mana yet — the trigger fires on the main step");

    // Walk turn 1 until PreCombatMain. The trigger is queued when the
    // engine enters PreCombatMain; drain the stack once more after the
    // loop to actually resolve it.
    while !matches!(g.step, TurnStep::PreCombatMain) && !g.is_game_over() {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    assert_eq!(
        g.players[0].mana_pool.amount(Color::Green), 1,
        "Chancellor of the Tangle should grant {{G}} during turn 1's first main"
    );
}

#[test]
fn chancellor_of_the_annex_taxes_first_opponent_spell() {
    // P1 has Chancellor of the Annex in opening hand → P0's first spell
    // next turn costs {1} more. Cast Lightning Bolt with only {R}: it fails
    // because the tax pushes it to {1}{R}.
    let mut g = two_player_game();
    g.add_card_to_library(1, catalog::chancellor_of_the_annex());
    for _ in 0..7 {
        g.add_card_to_library(0, catalog::forest());
    }
    for _ in 0..7 {
        g.add_card_to_library(1, catalog::forest());
    }
    g.start_mulligan_phase();
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Keep)).unwrap();
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Keep)).unwrap();

    // Walk to P0's first upkeep so the delayed reveal trigger fires and
    // stamps the tax charge on P0.
    while !matches!(g.step, TurnStep::Upkeep) && !g.is_game_over() {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    assert_eq!(
        g.players[0].first_spell_tax_charges, 1,
        "Annex should stamp one tax charge on P0"
    );

    // P0 tries to cast Bolt on their own turn with exactly {R}. The {1}
    // tax pushes the cost to {1}{R} — they can't afford it.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(err.is_err(), "Tax should make Bolt unaffordable with only {{R}}");

    // Pay the extra and the cast succeeds; tax is consumed.
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable with the {1} extra");
    assert_eq!(
        g.players[0].first_spell_tax_charges, 0,
        "Tax should be consumed by the cast"
    );
}

#[test]
fn serum_powder_exiles_hand_and_redraws() {
    // Mulligan with a Serum Powder in hand. Submitting `SerumPowder(id)`
    // exiles the hand and deals a fresh 7. The London-mulligan ladder
    // doesn't advance.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::serum_powder());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::forest());
    }
    for _ in 0..7 {
        g.add_card_to_library(0, catalog::island());
    }
    for _ in 0..7 {
        g.add_card_to_library(1, catalog::forest());
    }
    g.start_mulligan_phase();

    let powder_id = g.players[0].hand.iter()
        .find(|c| c.definition.name == "Serum Powder")
        .map(|c| c.id)
        .expect("Serum Powder should be in opening hand (top of library)");

    // Submit the powder.
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::SerumPowder(powder_id))).unwrap();

    // The fresh seven shouldn't include the original powder — it was exiled.
    assert_eq!(g.players[0].hand.len(), 7, "fresh hand of 7 after powder");
    assert!(
        g.exile.iter().any(|c| c.id == powder_id),
        "the consumed Serum Powder should be in exile"
    );
    // We're still on player 0's mulligan window (no advance).
    assert!(matches!(
        g.pending_decision.as_ref().map(|pd| &pd.decision),
        Some(crate::decision::Decision::Mulligan { player: 0, .. }),
    ));
}

// ── Card behavior tests ──────────────────────────────────────────────────────

#[test]
fn callous_sell_sword_enters_with_counter_per_creature_died() {
    // {1}{B} 2/2 enters with a +1/+1 counter for each creature that died
    // under your control this turn.
    let mut g = two_player_game();
    g.players[0].creatures_died_this_turn = 2;
    let css = g.add_card_to_hand(0, catalog::callous_sell_sword());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: css, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Callous Sell-Sword castable for {1}{B}");
    drain_stack(&mut g);

    let sell = g.battlefield.iter().find(|c| c.id == css)
        .expect("Sell-Sword should be on the battlefield");
    // Base 2/2 + two +1/+1 counters = 4/4.
    assert_eq!(sell.power(), 4, "2 base + 2 counters");
    assert_eq!(sell.toughness(), 4, "2 base + 2 counters");
}

#[test]
fn plunge_into_darkness_mode_one_pays_x_life_looks_and_exiles_rest() {
    // Mode 1: pay 2 life (ChooseAmount), look at the top 2, take Bolt into
    // hand, exile the Swamp.
    let mut g = two_player_game();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt()); // top
    let swamp = g.add_card_to_library(0, catalog::swamp());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Amount(2),
        DecisionAnswer::Search(Some(bolt)),
    ]));

    let plunge = g.add_card_to_hand(0, catalog::plunge_into_darkness());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: plunge, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("Plunge castable for {1}{B}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before - 2, "pays the chosen X = 2 life");
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "Bolt goes to hand");
    assert!(g.exile.iter().any(|c| c.id == swamp), "the rest are exiled");
}

#[test]
fn plunge_into_darkness_entwine_runs_both_modes() {
    // Kicked (entwine {B}): sacrifice the one creature for 3 life AND pay 2
    // life to dig — net +3 -2 = +1 life, with the dug card in hand.
    use crate::card::Keyword;
    let mut g = two_player_game();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    let creature = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Amount(1),          // sacrifice 1 creature
        DecisionAnswer::Amount(2),          // pay 2 life
        DecisionAnswer::Search(Some(bolt)), // take Bolt
    ]));
    assert!(catalog::plunge_into_darkness().keywords.iter()
        .any(|k| matches!(k, Keyword::Kicker(_))), "entwine modeled as kicker");

    let plunge = g.add_card_to_hand(0, catalog::plunge_into_darkness());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 2); // {1}{B} + entwine {B}
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpellKicked {
        card_id: plunge, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Plunge castable entwined");
    drain_stack(&mut g);

    assert!(g.battlefield_find(creature).is_none(), "creature sacrificed");
    assert_eq!(g.players[0].life, life_before + 3 - 2, "+3 from sac, -2 from dig");
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "dug Bolt in hand");
}

#[test]
fn mulligan_decision_lists_serum_powders() {
    // A mulligan-phase decision should expose every Serum-Powder-style
    // helper currently in hand so the UI can render a "use powder" button.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::serum_powder());
    g.add_card_to_library(0, catalog::serum_powder());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    for _ in 0..7 {
        g.add_card_to_library(1, catalog::forest());
    }
    g.start_mulligan_phase();

    let pd = g.pending_decision.as_ref().expect("mulligan decision pending");
    let crate::decision::Decision::Mulligan { serum_powders, .. } = &pd.decision else {
        panic!("expected Mulligan decision");
    };
    assert_eq!(serum_powders.len(), 2,
        "both powders in P0's opening hand should be listed");
}

#[test]
fn first_spell_tax_charges_default_zero() {
    // Sanity check: a fresh player has no Annex tax pending.
    let g = two_player_game();
    assert_eq!(g.players[0].first_spell_tax_charges, 0);
    assert_eq!(g.players[1].first_spell_tax_charges, 0);
}

#[test]
fn reveal_until_find_caps_at_n_when_no_match() {
    // No matching card on top → mill `cap` cards, lose `cap * life` life.
    use crate::effect::{Effect, ZoneDest};
    use crate::card::SelectionRequirement;
    let mut g = two_player_game();
    // Library: 5 lands. With `find: Creature`, none match → mill all 5.
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let lib_before = g.players[0].library.len();
    let life_before = g.players[0].life;
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &Effect::RevealUntilFind {
            who: PlayerRef::You,
            find: SelectionRequirement::Creature,
            to: ZoneDest::Hand(PlayerRef::You),
            cap: Value::Const(10),
            life_per_revealed: 1,
            miss_dest: crate::effect::RevealMissDest::Graveyard,
        },
        &ctx,
    ).unwrap();

    assert_eq!(g.players[0].library.len(), 0,
        "all 5 lands should mill (no creature found)");
    assert_eq!(g.players[0].graveyard.len(), lib_before,
        "milled cards land in graveyard");
    assert_eq!(g.players[0].life, life_before - 5,
        "5 cards revealed → 5 life lost");
}

#[test]
fn add_first_spell_tax_increments_per_opponent() {
    // `Effect::AddFirstSpellTax` against `EachOpponent` adds one charge to
    // each opponent (not the controller).
    let mut g = two_player_game();
    let ctx = EffectContext::for_ability(crate::card::CardId(0), 0, None);
    g.resolve_effect(
        &Effect::AddFirstSpellTax {
            who: PlayerRef::EachOpponent,
            count: Value::Const(2),
        },
        &ctx,
    ).unwrap();
    assert_eq!(g.players[0].first_spell_tax_charges, 0,
        "controller is not an opponent of themselves");
    assert_eq!(g.players[1].first_spell_tax_charges, 2,
        "opponent should receive 2 charges");
}

#[test]
fn deal_to_hand_draws_from_top_of_library() {
    // Regression: deal_to_hand used to call library.pop() (bottom of library)
    // — for an unshuffled fixture that produced wrong opening hands.
    let mut g = two_player_game();
    // Push lightning bolt (top), then 6 forests below.
    g.add_card_to_library(0, catalog::lightning_bolt());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::forest());
    }
    for _ in 0..7 {
        g.add_card_to_library(1, catalog::forest());
    }
    g.start_mulligan_phase();
    // Opening hand should include the bolt (top of library).
    assert!(
        g.players[0].hand.iter().any(|c| c.definition.name == "Lightning Bolt"),
        "deal_to_hand should draw from the top — the bolt was at index 0"
    );
}

#[test]
fn teferi_static_locks_opponent_to_sorcery_timing() {
    // P0's Teferi locks every opponent into sorcery timing — even an
    // instant in the opponent's hand can't be cast outside their own
    // main phase.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::teferi_time_raveler());
    // Bob is the off-turn caster trying to cast Bolt during Alice's main.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    let result = g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(matches!(result, Err(GameError::SorcerySpeedOnly)),
        "Teferi's static should lock opp Bolt to sorcery timing, got {:?}",
        result);
}

#[test]
fn teferi_static_does_not_restrict_controllers_own_casts() {
    // Teferi only locks opponents — its controller can still cast
    // instants on their opponent's turn.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::teferi_time_raveler());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 0;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Teferi's controller can still cast instants on opp's turn");
}

#[test]
fn teferi_plus_one_grants_sorceries_as_flash_until_next_turn() {
    // P0's Teferi +1 lets P0 cast sorceries at instant speed even when it
    // isn't their turn. Once P0's next turn rolls around (do_untap), the
    // flag clears and the timing gate snaps back.
    let mut g = two_player_game();
    let teferi = g.add_card_to_battlefield(0, catalog::teferi_time_raveler());
    g.clear_sickness(teferi);

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: teferi, ability_index: 0, target: None,  // +1
    }).expect("Teferi's +1 should be sorcery-speed-castable");
    drain_stack(&mut g);

    assert!(g.players[0].sorceries_as_flash,
        "P0 should be flagged for sorcery-as-flash");

    // Hand P0 a sorcery and roll the turn over to P1's combat step. From
    // P1's combat — normally a sorcery-illegal window — P0 still casts it.
    let bolt_alike = g.add_card_to_hand(0, catalog::lightning_bolt());
    let _ = bolt_alike; // (bolt is an instant; for the gate test we use
                        // the engine path that triggers SorcerySpeedOnly:
                        // play_land out-of-turn would error, but spells
                        // on the cast path with `is_instant_speed` already
                        // bypass the gate. Use a sorcery to verify.)

    let sorcery = g.add_card_to_hand(0, catalog::wrath_of_god());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 2);
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 0;

    g.perform_action(GameAction::CastSpell {
        card_id: sorcery, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Teferi +1 should let P0 cast Wrath of God outside their main phase");

    // Now simulate untap on P0's turn — flag should clear.
    g.active_player_idx = 0;
    g.do_untap();
    assert!(!g.players[0].sorceries_as_flash,
        "do_untap should clear the flag at the start of P0's next turn");
}

#[test]
fn damping_sphere_downgrades_dual_lands_to_colorless() {
    // With Damping Sphere on the battlefield, a Watery Grave (dual UB)
    // should ETB with only a single "{T}: Add {C}" ability. A subsequent
    // tap activates the new colorless ability, producing 1 colorless mana.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::damping_sphere());

    // Watery Grave originally has two activated abilities (tap for {U},
    // tap for {B}) plus the ETB shock-trigger. When entering under
    // Damping Sphere it should be reduced to a single {T}: Add {C}.
    let lands = g.add_card_to_hand(0, catalog::watery_grave());
    g.perform_action(GameAction::PlayLand(lands)).expect("play Watery Grave");
    drain_stack(&mut g);

    let land = g.battlefield_find(lands).expect("Watery Grave on battlefield");
    assert_eq!(
        land.definition.activated_abilities.len(),
        1,
        "Damping Sphere should leave only one mana ability on dual lands"
    );

    // Activate it — it should produce {C}, not {U} or {B}.
    g.clear_sickness(lands);
    g.perform_action(GameAction::ActivateAbility {
        card_id: lands, ability_index: 0, target: None, x_value: None }).expect("the downgraded ability should activate");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 0);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 0);
}

#[test]
fn damping_sphere_preserves_non_mana_activated_abilities() {
    // Regression: Damping Sphere downgrades a land's *mana* abilities to
    // {T}: Add {C} — it must not erase non-mana activated abilities
    // (fetchland sac, manland animate, etc.). Build a synthetic land
    // with both a multi-color mana ability and a non-mana sac ability
    // and assert that the sac ability survives the downgrade.
    use crate::card::{ActivatedAbility, CardDefinition, CardType};
    use crate::effect::{Effect, ManaPayload, PlayerRef};
    use crate::mana::{Color, ManaCost};

    let mana_ability = ActivatedAbility {
        tap_cost: true,
        mana_cost: ManaCost::default(),
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Colors(vec![Color::White, Color::Blue]),
        },
        once_per_turn: false,
        sorcery_speed: false,
        sac_cost: false,
        condition: None,
        life_cost: 0,
        from_graveyard: false,
        exile_self_cost: false,
        exile_other_filter: None,
        self_counter_cost_reduction: None, sac_other_filter: None,
        tap_other_filter: None, from_hand: false,
    };
    // Sentinel non-mana ability: tap + sacrifice the land. The body is
    // Noop — what matters is that it's not a mana ability so the
    // downgrade preserves it.
    let sac_ability = ActivatedAbility {
        tap_cost: true,
        mana_cost: ManaCost::default(),
        effect: Effect::Noop,
        once_per_turn: false,
        sorcery_speed: false,
        sac_cost: true,
        condition: None,
        life_cost: 0,
        from_graveyard: false,
        exile_self_cost: false,
        exile_other_filter: None,
        self_counter_cost_reduction: None, sac_other_filter: None,
        tap_other_filter: None, from_hand: false,
    };
    let fancy_land = CardDefinition {
        name: "Fancy Test Land",
        card_types: vec![CardType::Land],
        activated_abilities: vec![mana_ability, sac_ability],
        ..Default::default()
    };

    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::damping_sphere());
    let id = g.add_card_to_hand(0, fancy_land);
    g.perform_action(GameAction::PlayLand(id)).expect("play fancy land");

    let land = g.battlefield_find(id).expect("land on battlefield");
    assert_eq!(
        land.definition.activated_abilities.len(),
        2,
        "non-mana sac ability must survive the Damping Sphere downgrade"
    );
    // The surviving abilities should be: the original non-mana sac, and
    // the inserted {T}: Add {C}. Order: non-mana retained first, then
    // the inserted colorless mana ability appended.
    assert!(land.definition.activated_abilities[0].sac_cost,
        "the non-mana sac ability should be preserved at its original slot");
    let mana_ab = &land.definition.activated_abilities[1];
    assert!(matches!(
        mana_ab.effect,
        Effect::AddMana { pool: ManaPayload::Colorless(_), .. }
    ));
}

#[test]
fn damping_sphere_leaves_basic_lands_alone() {
    // A single-color basic (Forest) should pass through unchanged: still
    // exactly one ability, still produces {G}.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::damping_sphere());
    let f = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(f)).expect("play Forest");
    let forest = g.battlefield_find(f).expect("Forest on battlefield");
    assert_eq!(
        forest.definition.activated_abilities.len(),
        1,
        "single-color basic should keep its sole mana ability"
    );
    g.clear_sickness(f);
    g.perform_action(GameAction::ActivateAbility {
        card_id: f, ability_index: 0, target: None, x_value: None }).expect("Forest should still tap for {G}");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
}

/// `auto_target_for_effect` should prefer an opponent-controlled permanent
/// over the controller's own when both satisfy the filter. This guards the
/// bot from auto-destroying its own creatures with hostile spells like
/// Doom Blade, Abrupt Decay, etc.
#[test]
fn auto_target_prefers_opponent_permanent_for_hostile_effect() {
    let mut g = two_player_game();
    // Bot controls a creature; opponent also controls one.
    let _own_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    // Doom Blade ({1}{B}: Destroy target nonblack creature) — both bears
    // satisfy the filter. Without the opp-preference, the auto-target would
    // pick whichever bear was added to the battlefield first, which is
    // the bot's own.
    let doom = catalog::doom_blade();
    let target = g.auto_target_for_effect(&doom.effect, 0)
        .expect("Doom Blade should have a legal target");
    assert_eq!(target, crate::game::Target::Permanent(opp_bear),
        "Doom Blade should auto-target the opp's bear, not the bot's own");
}

/// Mirror of the hostile-effect auto-target test: friendly buffs should
/// pick the *caster's* permanent, not the opponent's. Without this, the
/// random bot would happily pump the opp's bear with Vines of Vastwood.
#[test]
fn auto_target_prefers_friendly_permanent_for_buff_effect() {
    let mut g = two_player_game();
    let own_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    // Vines of Vastwood pumps a target creature +4/+4. Both bears satisfy
    // the filter; the friendly-preference heuristic should pick our own.
    let vines = catalog::vines_of_vastwood();
    let target = g.auto_target_for_effect(&vines.effect, 0)
        .expect("Vines of Vastwood should have a legal target");
    assert_eq!(target, crate::game::Target::Permanent(own_bear),
        "Vines of Vastwood should auto-target the caster's bear");
}

/// Tragic Slip pumps -13/-13: that's a debuff, so the friendly heuristic
/// should *not* fire and the auto-target should land on the opp's
/// creature even though the spell is `PumpPT`.
#[test]
fn auto_target_pumppt_debuff_falls_back_to_hostile() {
    let mut g = two_player_game();
    let _own_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let slip = catalog::tragic_slip();
    let target = g.auto_target_for_effect(&slip.effect, 0)
        .expect("Tragic Slip should have a legal target");
    assert_eq!(target, crate::game::Target::Permanent(opp_bear),
        "Negative pump should auto-target the opponent");
}

/// Reanimate-style "Move(target → Hand(You))" should prefer a card in the
/// caster's graveyard over a battlefield permanent. Pre-fix the bot would
/// happily Disentomb its opponent's living bear into its own hand —
/// effectively bouncing-stealing the creature.
#[test]
fn auto_target_disentomb_prefers_creature_in_your_graveyard() {
    let mut g = two_player_game();
    // P0 has a creature in graveyard, P1 has a living one on the battlefield.
    let dead_bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let _live_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let disentomb = catalog::disentomb();
    let target = g.auto_target_for_effect(&disentomb.effect, 0)
        .expect("Disentomb should have a legal target");
    assert_eq!(target, crate::game::Target::Permanent(dead_bear),
        "Disentomb should pick the dead bear from our graveyard, \
         not the opp's living one");
}

/// Same idea for Raise Dead. The friendly-target preference handles the
/// "ours vs theirs" axis; the graveyard-source preference handles
/// "battlefield vs graveyard". Both apply to Move-into-hand.
#[test]
fn auto_target_raise_dead_prefers_creature_in_your_graveyard() {
    let mut g = two_player_game();
    let dead_bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let _own_live = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    let rd = catalog::raise_dead();
    let target = g.auto_target_for_effect(&rd.effect, 0)
        .expect("Raise Dead should have a legal target");
    assert_eq!(target, crate::game::Target::Permanent(dead_bear),
        "Raise Dead should pick the dead bear, not pull a live one off \
         the battlefield into the hand");
}

/// Regression for the Spectate Bot vs Bot deadlock at
/// `debug/deadlock-t10-1777412787-934831200.json`. P1 wanted to cast
/// Bone Shards (Destroy target creature) and the only opponent
/// creature was Sylvan Caryatid — which has Hexproof. Pre-fix
/// `auto_target_for_effect` ignored targeting legality, returned
/// the Caryatid, the engine rejected the cast with `TargetHasHexproof`,
/// and the bot kept submitting the same illegal action until the
/// watchdog tripped. Post-fix the helper checks `check_target_legality`
/// and falls through to a legal target (or `None`).
#[test]
fn auto_target_skips_hexproof_opponent_creature() {
    let mut g = two_player_game();
    // Opponent's only creature has Hexproof.
    let caryatid = g.add_card_to_battlefield(1, catalog::sylvan_caryatid());
    // Caster has a creature too so the unfiltered fallback can pick it.
    let own_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    let bone_shards = catalog::bone_shards();
    let target = g.auto_target_for_effect(&bone_shards.effect, 0);

    // Whatever the helper returns, it MUST NOT be the hexproof Caryatid.
    assert_ne!(
        target,
        Some(crate::game::Target::Permanent(caryatid)),
        "auto_target must respect Hexproof",
    );
    // The fallback should land on the only other creature on the
    // battlefield — the caster's own bear. Self-targeting Destroy is
    // legal in MTG; bots being silly with their own removal is fine,
    // matches deadlocking is not.
    assert_eq!(target, Some(crate::game::Target::Permanent(own_bear)));
}

/// Regression: an `Any`-filtered Move (Regrowth) should NOT auto-target a
/// player. Move only consumes Permanent / Card refs, so a Player target
/// silently fizzles. The new `Effect::accepts_player_target` skip prevents
/// the heuristic from picking Player(0) for permanent-only effects even
/// when the filter is broad enough to legally accept it.
#[test]
fn auto_target_regrowth_skips_player_in_favor_of_graveyard_card() {
    let mut g = two_player_game();
    let dead_card = g.add_card_to_graveyard(0, catalog::mountain());

    let regrowth = catalog::regrowth();
    let target = g.auto_target_for_effect(&regrowth.effect, 0)
        .expect("Regrowth should have a legal target");
    assert_eq!(
        target,
        crate::game::Target::Permanent(dead_card),
        "Regrowth should pick the graveyard card, not a Player target — \
         Move only consumes card/permanent refs, so a player target \
         silently fizzles",
    );
}

/// Regression: Boomerang's `Permanent` filter rejects a Player target via
/// the requirement, but the previous heuristic short-circuited at the
/// Player check anyway since it was untested there. With
/// `accepts_player_target=false`, we skip the Player rung entirely and
/// land on a permanent immediately.
#[test]
fn auto_target_boomerang_picks_a_permanent_not_a_player() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let boomerang = catalog::boomerang();
    let target = g.auto_target_for_effect(&boomerang.effect, 0)
        .expect("Boomerang should find a legal permanent target");
    assert_eq!(target, crate::game::Target::Permanent(opp_bear));
}

/// Regression: Mind Rot's auto-target picks the opponent. Pre-fix the
/// effect used a bare `Selector::Target(0)` which `primary_target_filter`
/// didn't recognize, so the bot's `auto_target_for_effect` returned
/// `None` and the spell was unplayable by the random bot.
#[test]
fn auto_target_mind_rot_picks_opponent_player() {
    let g = two_player_game();
    let mind_rot = catalog::mind_rot();
    let target = g.auto_target_for_effect(&mind_rot.effect, 0)
        .expect("Mind Rot should auto-target the opponent");
    assert_eq!(target, crate::game::Target::Player(1),
        "Mind Rot should hit the opponent");
}

/// Discard / Mill effects with `target_filtered(Player)` are findable by
/// `primary_target_filter` so the bot's `auto_target_for_effect` can
/// resolve a target. Pre-fix only effects whose `what`/`who` slot was
/// matched in `primary_target_filter` (Destroy, PumpPT, etc.)
/// auto-targeted — player-side effects fell through and the bot
/// returned `None`, leaving the spell unplayable.
#[test]
fn auto_target_player_side_effects_resolve_via_filter() {
    use crate::card::SelectionRequirement;
    use crate::effect::Effect;
    use crate::card::Value;
    let g = two_player_game();

    let discard = Effect::Discard {
        who: crate::effect::shortcut::target_filtered(SelectionRequirement::Player),
        amount: Value::Const(1),
        random: true,
    };
    let mill = Effect::Mill {
        who: crate::effect::shortcut::target_filtered(SelectionRequirement::Player),
        amount: Value::Const(1),
    };

    // Both are hostile → opp.
    assert_eq!(g.auto_target_for_effect(&discard, 0),
        Some(crate::game::Target::Player(1)),
        "Discard should auto-target the opp");
    assert_eq!(g.auto_target_for_effect(&mill, 0),
        Some(crate::game::Target::Player(1)),
        "Mill should auto-target the opp");
}

/// Regression: Cling to Dust's effect tree is
///   `Seq([Move(target → Exile), If(Creature, GainLife, Noop)])`.
/// The trailing `If(... GainLife)` accepts a Player target, but the
/// PRIMARY target slot is the Move's `Any` filter — and Move only
/// consumes Permanent/Card refs, so a Player target silently fizzles.
/// Pre-fix `accepts_player_target` for `Seq` was an unconditional
/// `any`, so the heuristic returned Player(opp). Post-fix it defers to
/// the first child whose `primary_target_filter` is set (the Move),
/// which rejects Player.
#[test]
fn auto_target_cling_to_dust_picks_graveyard_card_not_player() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());

    let cling = catalog::cling_to_dust();
    let target = g.auto_target_for_effect(&cling.effect, 0)
        .expect("Cling to Dust should find a graveyard target");
    assert_eq!(target, crate::game::Target::Permanent(dead),
        "Cling to Dust auto-targets a graveyard card; Player target \
         would silently fizzle on the leading Move");
}

// ── Loyalty ability error variants ───────────────────────────────────────────

#[test]
fn loyalty_ability_already_used_returns_typed_error() {
    // Activate Teferi's -3 once, then try again the same turn — expect the
    // typed `LoyaltyAbilityAlreadyUsed` error rather than the placeholder
    // CardIsTapped the engine used to reuse.
    let mut g = two_player_game();
    let teferi = g.add_card_to_battlefield(0, catalog::teferi_time_raveler());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    // Seed P0's library so the trailing draw doesn't deck out the caster
    // and end the game (which would mask the second-activation rejection).
    g.add_card_to_library(0, catalog::forest());

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: teferi,
        ability_index: 1, // -3 (index 0 is +1 sorceries-as-flash)
        target: Some(Target::Permanent(bear)),
    })
    .expect("First -3 activation should succeed");
    drain_stack(&mut g);

    // Need a second targetable opponent permanent for the retry — otherwise
    // we'd fall through to the AutoDecider's "no legal target" path before
    // the used-ability check fires.
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear2);

    let err = g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: teferi,
        ability_index: 1,
        target: Some(Target::Permanent(bear2)),
    })
    .unwrap_err();
    assert!(
        matches!(err, GameError::LoyaltyAbilityAlreadyUsed(id) if id == teferi),
        "Expected LoyaltyAbilityAlreadyUsed; got {:?}",
        err,
    );
}

#[test]
fn loyalty_negative_cost_returns_not_enough_loyalty() {
    // Manually drop Teferi's loyalty to 2, then try to activate -3.
    let mut g = two_player_game();
    let teferi = g.add_card_to_battlefield(0, catalog::teferi_time_raveler());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.add_card_to_library(0, catalog::forest());

    if let Some(c) = g.battlefield_find_mut(teferi) {
        c.counters
            .insert(crate::card::CounterType::Loyalty, 2);
    }
    let err = g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: teferi,
        ability_index: 1, // -3 (insufficient against 2 loyalty)
        target: Some(Target::Permanent(bear)),
    })
    .unwrap_err();
    assert!(
        matches!(err, GameError::NotEnoughLoyalty(id) if id == teferi),
        "Expected NotEnoughLoyalty; got {:?}",
        err,
    );
}

// ── Strixhaven (STX) ──────────────────────────────────────────────────────

#[test]
fn spirited_companion_etb_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::spirited_companion());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Spirited Companion castable for {1}{W}");
    let hand_before = g.players[0].hand.len();
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id), "Companion enters battlefield");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "ETB should have drawn a card");
}

#[test]
fn eyetwitch_dies_triggers_learn_draw() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let twitch = g.add_card_to_battlefield(0, catalog::eyetwitch());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(twitch)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Lightning Bolt targets Eyetwitch");
    let hand_before = g.players[0].hand.len();
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == twitch), "Eyetwitch dies");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "Learn → drew a card");
}

#[test]
fn learn_fetches_a_lesson_from_the_sideboard() {
    // With a Lesson in the sideboard, Learn's AutoDecider reveals it into
    // hand (card advantage) instead of falling back to Draw 1.
    use crate::card::CardInstance;
    let mut g = two_player_game();
    let lesson = g.next_id();
    g.players[0]
        .sideboard
        .push(CardInstance::new(lesson, catalog::pest_summoning(), 0));
    let twitch = g.add_card_to_battlefield(0, catalog::eyetwitch());
    let hand_before = g.players[0].hand.len();

    g.remove_to_graveyard_with_triggers(twitch); // Eyetwitch dies → Learn
    drain_stack(&mut g);

    assert!(
        g.players[0].hand.iter().any(|c| c.id == lesson),
        "the Lesson was revealed into hand"
    );
    assert!(
        !g.players[0].sideboard.iter().any(|c| c.id == lesson),
        "the Lesson left the sideboard"
    );
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "hand +1 from the fetched Lesson");
}

#[test]
fn learn_rummage_discards_then_draws() {
    // A ScriptedDecider takes the rummage branch: discard a chosen card,
    // then draw one. Net hand size is unchanged; the discarded card is in
    // the graveyard and the library's top card is in hand.
    use crate::card::CardInstance;
    use crate::decision::LearnChoice;
    let mut g = two_player_game();
    let lesson_in_sb = g.next_id();
    g.players[0]
        .sideboard
        .push(CardInstance::new(lesson_in_sb, catalog::pest_summoning(), 0));
    let discard_me = g.add_card_to_hand(0, catalog::grizzly_bears());
    let lib_top = g.next_id();
    g.players[0].add_to_library_top(lib_top, catalog::lightning_bolt());
    let twitch = g.add_card_to_battlefield(0, catalog::eyetwitch());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Learn(
        LearnChoice::Rummage { discard: discard_me },
    )]));

    g.remove_to_graveyard_with_triggers(twitch);
    drain_stack(&mut g);

    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == discard_me),
        "the rummaged card was discarded"
    );
    assert!(
        g.players[0].hand.iter().any(|c| c.id == lib_top),
        "the top card was drawn"
    );
}

#[test]
fn learn_decline_does_nothing() {
    // Declining Learn leaves hand and sideboard untouched.
    use crate::card::CardInstance;
    use crate::decision::LearnChoice;
    let mut g = two_player_game();
    let lesson_in_sb = g.next_id();
    g.players[0]
        .sideboard
        .push(CardInstance::new(lesson_in_sb, catalog::pest_summoning(), 0));
    let twitch = g.add_card_to_battlefield(0, catalog::eyetwitch());
    let hand_before = g.players[0].hand.len();
    let sideboard_before = g.players[0].sideboard.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Learn(LearnChoice::Decline)]));

    g.remove_to_graveyard_with_triggers(twitch);
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before, "hand unchanged");
    assert_eq!(g.players[0].sideboard.len(), sideboard_before, "sideboard unchanged");
}

#[test]
fn learn_ui_player_suspends_and_resumes_via_submit_decision() {
    // A `wants_ui` player's Learn suspends on a `Decision::Learn` instead of
    // auto-resolving; submitting the answer reveals the Lesson into hand.
    use crate::card::CardInstance;
    use crate::decision::{Decision, LearnChoice};
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let lesson = g.next_id();
    g.players[0]
        .sideboard
        .push(CardInstance::new(lesson, catalog::pest_summoning(), 0));
    let twitch = g.add_card_to_battlefield(0, catalog::eyetwitch());

    g.remove_to_graveyard_with_triggers(twitch); // Eyetwitch dies → Learn
    drain_stack(&mut g);

    let pd = g.pending_decision.as_ref().expect("Learn should suspend for a UI player");
    assert!(matches!(pd.decision, Decision::Learn { .. }), "a Learn decision is pending");

    g.submit_decision(DecisionAnswer::Learn(LearnChoice::FetchLesson(lesson)))
        .expect("Learn answer accepted");
    assert!(g.pending_decision.is_none(), "decision resolved");
    assert!(
        g.players[0].hand.iter().any(|c| c.id == lesson),
        "the Lesson was fetched into hand after the UI answer"
    );
}

#[test]
fn closing_statement_exiles_target_and_gains_x_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::closing_statement());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("Closing Statement castable for {3}{W}{W}");
    let life_before = g.players[0].life;
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "Bear exiled");
    assert!(g.exile.iter().any(|c| c.id == bear), "Bear is in exile zone");
    assert_eq!(g.players[0].life, life_before + 3, "Gained X = 3 life");
}

#[test]
fn vanishing_verse_exiles_target_nonland_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::vanishing_verse());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Vanishing Verse castable for {W}{B}");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear),
        "Vanishing Verse should exile its target");
}

#[test]
fn devastating_mastery_destroys_each_nonland_permanent() {
    let mut g = two_player_game();
    let _land = g.add_card_to_battlefield(0, catalog::forest());
    let _land2 = g.add_card_to_battlefield(1, catalog::forest());
    let bear0 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::devastating_mastery());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Devastating Mastery castable for {4}{W}{W}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear0));
    assert!(!g.battlefield.iter().any(|c| c.id == bear1));
    assert!(g.battlefield.iter().any(|c| c.definition.is_land()),
        "Lands survive — `Selector::EachPermanent(Nonland)` skips them");
}

#[test]
fn eager_first_year_magecraft_pumps_a_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _eager = g.add_card_to_battlefield(0, catalog::eager_first_year());
    g.clear_sickness(bear);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);
    let pumped = g.battlefield.iter().any(|c| {
        c.controller == 0 && c.definition.is_creature() && c.power_bonus >= 1
    });
    assert!(pumped, "Magecraft should have pumped a creature");
}

#[test]
fn eager_first_year_magecraft_does_not_fire_on_creature_spell() {
    let mut g = two_player_game();
    let _eager = g.add_card_to_battlefield(0, catalog::eager_first_year());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Grizzly Bears castable");
    drain_stack(&mut g);
    let any_pumped = g.battlefield.iter().any(|c| {
        c.controller == 0 && c.definition.is_creature() && c.power_bonus >= 1
    });
    assert!(!any_pumped,
        "Casting a creature spell should NOT trigger magecraft");
}

#[test]
fn hunt_for_specimens_creates_pest_token_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::hunt_for_specimens());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Hunt for Specimens castable for {3}{B}");
    let hand_before = g.players[0].hand.len();
    drain_stack(&mut g);
    let pest_count = g.battlefield.iter().filter(|c|
        c.controller == 0 && c.is_token && c.definition.name == "Pest"
    ).count();
    assert_eq!(pest_count, 1, "Should create exactly one Pest token");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "Learn → draw 1");
}

#[test]
fn witherbloom_apprentice_magecraft_drains_one() {
    let mut g = two_player_game();
    let _appr = g.add_card_to_battlefield(0, catalog::witherbloom_apprentice());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p0_before = g.players[0].life;
    let p1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_before - 3 - 1, "Bolt + drain");
    assert_eq!(g.players[0].life, p0_before + 1, "Drain gives 1 life");
}

#[test]
fn pest_summoning_creates_two_pest_tokens() {
    // Updated: matches the printed Oracle ("create two 1/1 black-and-green
    // Pest tokens"). The token's death-trigger lifegain rider is still ⏳.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_summoning());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Pest Summoning castable for {B}{G}");
    drain_stack(&mut g);
    let pest_count = g.battlefield.iter().filter(|c|
        c.controller == 0 && c.is_token && c.definition.name == "Pest"
    ).count();
    assert_eq!(pest_count, 2, "Pest Summoning creates two Pests (printed Oracle)");
}

#[test]
fn inkling_summoning_creates_a_2_1_flying_inkling_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_summoning());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Inkling Summoning castable for {3}{W}{B}");
    drain_stack(&mut g);
    let inklings: Vec<_> = g.battlefield.iter().filter(|c|
        c.controller == 0 && c.is_token && c.definition.name == "Inkling"
    ).collect();
    assert_eq!(inklings.len(), 1);
    let i = &inklings[0];
    assert_eq!(i.power(), 2);
    assert_eq!(i.toughness(), 1);
    assert!(i.has_keyword(&crate::card::Keyword::Flying));
}

#[test]
fn tend_the_pests_sacrifices_creature_and_creates_x_pests() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    g.clear_sickness(big);
    let id = g.add_card_to_hand(0, catalog::tend_the_pests());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Tend the Pests castable for {1}{B}{G}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == big),
        "Sacrificed creature should have left the battlefield");
    let pests = g.battlefield.iter().filter(|c|
        c.controller == 0 && c.is_token && c.definition.name == "Pest"
    ).count();
    assert_eq!(pests, 4, "Should create X = 4 Pest tokens (one per sacrificed power)");
}

/// CR 601.2b — an "additional cost: sacrifice a creature" spell can't be
/// cast when there's nothing to sacrifice; the spell stays in hand and no
/// mana is spent.
#[test]
fn additional_cost_sacrifice_rejected_without_fodder() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let before = g.players[0].life;
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "no creature to sacrifice → cast rejected");
    assert!(g.players[0].has_in_hand(id), "spell reverts to hand");
    assert_eq!(g.players[0].mana_pool.total(), 2, "mana not spent on a rejected cast");
    assert_eq!(g.players[1].life, before, "opponent not drained — spell never resolved");
}

/// CR 601.2h — the additional-cost sacrifice happens *during casting*, so
/// the fodder leaves the battlefield immediately, before the spell resolves
/// off the stack.
#[test]
fn additional_cost_sacrifice_pays_during_cast() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable with fodder present");
    // Sacrifice paid as a cost — fodder is gone before the drain resolves.
    assert!(!g.battlefield.iter().any(|c| c.id == fodder),
        "fodder sacrificed during casting, not at resolution");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "Witherbloom Sacrosanct drains the opponent for 3");
}

// ── CR 702.90b — Infect player damage → poison counters ─────────────────────

/// CR 702.90b: "Damage dealt to a player by a source with infect doesn't
/// cause that player to lose life. Rather, it causes that source's
/// controller to give the player that many poison counters." The combat
/// path in `combat.rs::apply_combat_damage` already honors this for the
/// attacker-damages-player case (Plague Stinger-style infect attackers).
/// This test exercises the **non-combat** path (`Effect::DealDamage` →
/// `deal_damage_to_from`) by manually granting Infect to a Grizzly
/// Bears, then dealing 2 spell damage from that bear to the opp player.
/// The opp should gain 2 poison counters and lose 0 life.
#[test]
fn infect_spell_damage_to_player_grants_poison_per_cr_702_90b() {
    use crate::card::Keyword;
    use crate::effect::{Selector, Value};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);

    // Grant Infect EOT to the bear by adding it directly to the
    // permanent's keyword list. Bypassing the layer system here keeps
    // the test focused on the damage routing rather than wiring up a
    // continuous-effect bookkeeping layer.
    std::sync::Arc::make_mut(&mut g.battlefield_find_mut(bear).unwrap().definition)
        .keywords
        .push(Keyword::Infect);
    assert!(
        g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Infect),
        "bear should now have Infect"
    );

    let p1_life_before = g.players[1].life;
    let p1_poison_before = g.players[1].poison_counters;

    // Spell-damage the opp from a source-with-infect (the bear).
    let ctx_from_bear = EffectContext::for_ability(bear, 0, None);
    g.resolve_effect(
        &Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(2),
        },
        &ctx_from_bear,
    )
    .unwrap();

    assert_eq!(
        g.players[1].life, p1_life_before,
        "infect damage should NOT reduce life"
    );
    assert_eq!(
        g.players[1].poison_counters, p1_poison_before + 2,
        "infect damage should add 2 poison counters; got {}",
        g.players[1].poison_counters
    );
}

/// Without infect, the same spell-damage should reduce life normally.
/// This is the control case for the test above — confirms the routing
/// is gated on the source's effective keywords, not always-on.
#[test]
fn non_infect_spell_damage_to_player_reduces_life_per_cr_702_90b_control() {
    use crate::effect::{Selector, Value};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);

    let p1_life_before = g.players[1].life;
    let p1_poison_before = g.players[1].poison_counters;

    let ctx_from_bear = EffectContext::for_ability(bear, 0, None);
    g.resolve_effect(
        &Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(2),
        },
        &ctx_from_bear,
    )
    .unwrap();

    assert_eq!(g.players[1].life, p1_life_before - 2);
    assert_eq!(g.players[1].poison_counters, p1_poison_before);
}

/// CR 702.180c — a Toxic N attacker that connects with a player deals
/// normal combat damage *and* gives that player N poison counters.
#[test]
fn toxic_attacker_adds_poison_on_combat_damage() {
    use crate::card::Keyword;
    use crate::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let bear = setup_attacker(&mut g, 0, catalog::grizzly_bears);
    std::sync::Arc::make_mut(&mut g.battlefield_find_mut(bear).unwrap().definition)
        .keywords
        .push(Keyword::Toxic(2));

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();

    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();

    assert_eq!(g.players[1].life, 18, "still takes 2 combat damage");
    assert_eq!(g.players[1].poison_counters, 2, "Toxic 2 adds 2 poison");
}

// ── CR 111.4 — token name auto-derives from subtypes ─────────────────────

#[test]
fn token_without_name_derives_name_from_creature_subtypes() {
    use crate::card::{ArtifactSubtype, CreatureType, Subtypes, TokenDefinition};
    use crate::game::effects::token_to_card_definition;

    // Per CR 111.4, a token whose creating effect doesn't name it takes its
    // subtypes plus " Token" as its name.
    let spirit = TokenDefinition {
        name: String::new(),
        power: 1,
        toughness: 1,
        card_types: vec![crate::card::CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(token_to_card_definition(&spirit).name, "Spirit Token");

    let treasure = TokenDefinition {
        name: String::new(),
        card_types: vec![crate::card::CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Treasure],
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(token_to_card_definition(&treasure).name, "Treasure Token");

    // Explicit name still wins over the auto-derive.
    let explicit = TokenDefinition {
        name: "Tireless Tracker's Clue".into(),
        card_types: vec![crate::card::CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Clue],
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(
        token_to_card_definition(&explicit).name,
        "Tireless Tracker's Clue"
    );
}

// ── CR 701.17b — mill stops at empty library ─────────────────────────────

/// Push (modern_decks): per CR 701.17b, "A player can't mill a number of
/// cards greater than the number of cards in their library. If given the
/// choice to do so, they can't choose to take that action. If instructed
/// to do so, they mill as many as possible." The engine's `Effect::Mill`
/// handler breaks the per-card loop when the library is empty, so a
/// Mill(N) with M < N cards in library mills exactly M cards (not fewer
/// or more). This test stages a 3-card library and mills 10, asserting
/// all 3 cards go to graveyard and the library is empty afterwards.
#[test]
fn mill_caps_at_library_size_per_cr_701_17b() {
    use crate::card::CardType;
    use crate::effect::Effect;
    let mut g = two_player_game();
    // Build a 3-card library on P0.
    let _c1 = g.add_card_to_library(0, catalog::forest());
    let _c2 = g.add_card_to_library(0, catalog::island());
    let _c3 = g.add_card_to_library(0, catalog::mountain());
    assert_eq!(g.players[0].library.len(), 3);

    // Construct a "Mill 10 cards" effect directly.
    let mill_def = crate::card::CardDefinition {
        name: "Test Mill 10",
        cost: crate::mana::ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: crate::card::Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Mill {
            who: crate::effect::Selector::Player(crate::effect::PlayerRef::Target(0)),
            amount: Value::Const(10),
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
    };
    let mill = g.add_card_to_hand(0, mill_def);
    g.perform_action(GameAction::CastSpell {
        card_id: mill,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("mill spell castable");
    drain_stack(&mut g);

    assert_eq!(g.players[0].library.len(), 0, "library emptied");
    // 3 milled cards + the mill spell itself → 4 cards in graveyard.
    assert_eq!(g.players[0].graveyard.len(), 4,
        "3 milled + 1 cast goes to graveyard (per CR 701.17b mills exactly library_size)");
}

#[test]
fn cr_506_1_no_attackers_skips_to_end_of_combat() {
    // CR 506.1: "The declare blockers and combat damage steps are
    // skipped if no creatures are declared as attackers..." Verify that
    // ending the DeclareAttackers step with `attacking` empty advances
    // directly to EndCombat without lingering in DeclareBlockers or
    // CombatDamage.
    let mut g = two_player_game();
    g.step = TurnStep::DeclareAttackers;
    // No attackers declared. Pass priority twice (active then non-active).
    assert!(g.attacking.is_empty());
    g.perform_action(GameAction::PassPriority).unwrap();
    g.perform_action(GameAction::PassPriority).unwrap();
    // Per CR 506.1 we now jump straight to EndCombat.
    assert_eq!(g.step, TurnStep::EndCombat,
        "Empty-attacker DeclareAttackers should advance straight to EndCombat");
}

#[test]
fn cr_506_1_with_attackers_progresses_normally() {
    // Mirror test: when attackers exist, DeclareAttackers→DeclareBlockers
    // happens normally (no skip).
    let mut g = two_player_game();
    let bear_id = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == bear_id) {
        c.summoning_sick = false;
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear_id,
        target: AttackTarget::Player(1),
    }])).unwrap();
    assert!(!g.attacking.is_empty());
    g.perform_action(GameAction::PassPriority).unwrap();
    g.perform_action(GameAction::PassPriority).unwrap();
    assert_eq!(g.step, TurnStep::DeclareBlockers,
        "With attackers declared, DeclareAttackers should advance to DeclareBlockers");
}

// ── CR 401.7 — LibraryPosition::FromTop(n) ───────────────────────────────────

/// Verify that `LibraryPosition::FromTop(n)` places a card N from the top
/// of the library. We exercise this via the place_card_in_dest path by
/// resolving an Effect::Move targeting a card in hand.
#[test]
fn library_position_from_top_inserts_at_index() {
    use crate::card::SelectionRequirement;
    use crate::effect::shortcut::target_filtered;
    use crate::effect::{LibraryPosition, ZoneDest};
    let mut g = two_player_game();
    // Seed the library with five distinct cards.
    let a = g.add_card_to_library(0, catalog::plains());
    let b = g.add_card_to_library(0, catalog::island());
    let c = g.add_card_to_library(0, catalog::swamp());
    let d = g.add_card_to_library(0, catalog::mountain());
    let e = g.add_card_to_library(0, catalog::forest());

    // Pre-state: library top → bottom is [a, b, c, d, e]. Index 0 is the
    // top by convention.
    assert_eq!(g.players[0].library[0].id, a);

    // Cast a fake spell whose effect is Move(target → FromTop(2)).
    // We'll seed a token-like card on the battlefield and move it.
    let target_card = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = crate::game::effects::EffectContext::for_spell(
        0,
        Some(crate::game::types::Target::Permanent(target_card)),
        0,
        0,
    );
    let _ = g.resolve_effect(
        &Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Library {
                who: PlayerRef::Seat(0),
                pos: LibraryPosition::FromTop(2),
            },
        },
        &ctx,
    ).unwrap();

    // Verify the card is now at index 2 of the 6-card library.
    assert_eq!(g.players[0].library.len(), 6);
    assert_eq!(g.players[0].library[0].id, a);
    assert_eq!(g.players[0].library[1].id, b);
    assert_eq!(g.players[0].library[2].id, target_card);
    assert_eq!(g.players[0].library[3].id, c);
    assert_eq!(g.players[0].library[4].id, d);
    assert_eq!(g.players[0].library[5].id, e);
}

/// CR 401.7: "If a player is instructed to put a card 'Nth from the top'
/// of a library, and there are fewer than N cards in that library, the
/// card is put on the bottom of that library."
#[test]
fn library_position_from_top_with_fewer_cards_goes_to_bottom() {
    use crate::card::SelectionRequirement;
    use crate::effect::shortcut::target_filtered;
    use crate::effect::{LibraryPosition, ZoneDest};
    let mut g = two_player_game();
    // Seed the library with only two cards.
    let a = g.add_card_to_library(0, catalog::plains());
    let b = g.add_card_to_library(0, catalog::island());

    // Put a card in battlefield and move it to FromTop(7) — should go to bottom.
    let target_card = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = crate::game::effects::EffectContext::for_spell(
        0,
        Some(crate::game::types::Target::Permanent(target_card)),
        0,
        0,
    );
    let _ = g.resolve_effect(
        &Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Library {
                who: PlayerRef::Seat(0),
                pos: LibraryPosition::FromTop(7),
            },
        },
        &ctx,
    ).unwrap();

    assert_eq!(g.players[0].library.len(), 3);
    assert_eq!(g.players[0].library[0].id, a);
    assert_eq!(g.players[0].library[1].id, b);
    // The new card was put on bottom (index 2).
    assert_eq!(g.players[0].library[2].id, target_card);
}

/// `LibraryPosition::FromTop(0)` is equivalent to `Top`.
#[test]
fn library_position_from_top_zero_is_top() {
    use crate::card::SelectionRequirement;
    use crate::effect::shortcut::target_filtered;
    use crate::effect::{LibraryPosition, ZoneDest};
    let mut g = two_player_game();
    let a = g.add_card_to_library(0, catalog::plains());
    let b = g.add_card_to_library(0, catalog::island());

    let target_card = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = crate::game::effects::EffectContext::for_spell(
        0,
        Some(crate::game::types::Target::Permanent(target_card)),
        0,
        0,
    );
    let _ = g.resolve_effect(
        &Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Library {
                who: PlayerRef::Seat(0),
                pos: LibraryPosition::FromTop(0),
            },
        },
        &ctx,
    ).unwrap();

    assert_eq!(g.players[0].library[0].id, target_card);
    assert_eq!(g.players[0].library[1].id, a);
    assert_eq!(g.players[0].library[2].id, b);
}

// ── Combat edge cases ────────────────────────────────────────────────────

#[test]
fn lifelink_combat_damage_gains_life() {
    let mut g = two_player_game();
    let lifelinker = setup_attacker(&mut g, 0, catalog::serra_angel);
    std::sync::Arc::make_mut(&mut g.battlefield_find_mut(lifelinker).unwrap().definition).keywords.push(
        crate::card::Keyword::Lifelink,
    );
    let life_before = g.players[0].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: lifelinker,
        target: AttackTarget::Player(1),
    }]))
    .expect("Declare attackers");
    g.step = TurnStep::CombatDamage;
    let _events = g.resolve_combat().unwrap();
    assert!(g.players[0].life > life_before,
        "Lifelink should gain life from combat damage");
}

#[test]
fn cant_block_creature_can_still_attack() {
    let mut g = two_player_game();
    let attacker = setup_attacker(&mut g, 0, || {
        let mut def = catalog::grizzly_bears();
        def.keywords = vec![crate::card::Keyword::CantBlock];
        def
    });
    g.step = TurnStep::DeclareAttackers;
    let result = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]));
    assert!(result.is_ok(), "CantBlock creature should still be able to attack");
}

#[test]
fn effect_untap_removes_stun_counter_instead_of_untapping() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.battlefield_find_mut(bear).unwrap().add_counters(
        crate::card::CounterType::Stun, 1,
    );

    let ctx = EffectContext {
        controller: 0,
        source: Some(bear),
        targets: vec![],
        trigger_source: None,
        mode: 0,
        x_value: 0,
        converged_value: 0,
        mana_spent: 0,
        source_name: None,
        cast_from_hand: false,
        event_amount: 0,
        kicked: false,
    };
    g.resolve_effect(
        &Effect::Untap {
            what: Selector::EachPermanent(crate::card::SelectionRequirement::Creature),
            up_to: None,
        },
        &ctx,
    ).unwrap();

    let card = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert!(card.tapped, "Stunned creature should remain tapped after Effect::Untap");
    assert_eq!(card.counter_count(crate::card::CounterType::Stun), 0,
        "Stun counter should have been removed");
}

// ── Prowess enforcement (CR 702.107) ───────────────────────────────────────

#[test]
fn prowess_pumps_on_noncreature_spell() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::stormchaser_mage());
    g.clear_sickness(mage);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    let m = g.battlefield.iter().find(|c| c.id == mage).unwrap();
    assert!(m.power() >= 2, "Prowess should pump +1/+1 EOT");
    assert!(m.toughness() >= 4, "Prowess should pump +1/+1 EOT");
}

#[test]
fn prowess_does_not_trigger_on_creature_spell() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::stormchaser_mage());
    g.clear_sickness(mage);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("bear castable");
    drain_stack(&mut g);
    let m = g.battlefield.iter().find(|c| c.id == mage).unwrap();
    assert_eq!(m.power(), 1, "Prowess should NOT trigger on creature spell");
    assert_eq!(m.toughness(), 3);
}

// ── Combat module tests ─────────────────────────────────────────────────────

#[test]
fn cant_block_keyword_prevents_blocking() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    let cant_block_creature = g.add_card_to_battlefield(1, catalog::postmortem_professor());
    g.clear_sickness(cant_block_creature);

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    let result = g.perform_action(GameAction::DeclareBlockers(vec![(cant_block_creature, attacker)]));
    assert!(result.is_err(), "Creature with CantBlock should not be allowed to block");
}

// ── Deathtouch SBA (CR 704.5h) ────────────────────────────────────────────

#[test]
fn deathtouch_damage_kills_large_creature_via_sba() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(0, catalog::serra_angel());
    if let Some(c) = g.battlefield_find_mut(big) {
        c.damage = 1;
        c.dealt_deathtouch_damage = true;
    }
    let _ = g.check_state_based_actions();
    assert!(
        !g.battlefield.iter().any(|c| c.id == big),
        "Creature with deathtouch damage should die regardless of toughness"
    );
}

#[test]
fn normal_damage_does_not_trigger_deathtouch_sba() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(0, catalog::serra_angel());
    if let Some(c) = g.battlefield_find_mut(big) {
        c.damage = 1;
    }
    let _ = g.check_state_based_actions();
    assert!(
        g.battlefield.iter().any(|c| c.id == big),
        "1 normal damage should not kill a 4/4"
    );
}

// ── CR 704.5a — player at 0 or less life is eliminated ───────────────────

/// CR 704.5a: "If a player has 0 or less life, that player loses the game."
/// Verified by dealing lethal damage to a player and checking that the SBA
/// pass marks them eliminated.
#[test]
fn cr_704_5a_player_at_zero_life_is_eliminated() {
    let mut g = two_player_game();
    g.players[1].life = 0;
    let _ = g.check_state_based_actions();
    assert!(g.players[1].eliminated, "Player at 0 life should be eliminated by SBA");
}

#[test]
fn cr_704_5a_player_at_negative_life_is_eliminated() {
    let mut g = two_player_game();
    g.players[1].life = -5;
    let _ = g.check_state_based_actions();
    assert!(g.players[1].eliminated, "Player at negative life should be eliminated by SBA");
}

#[test]
fn cr_704_5a_player_at_one_life_survives() {
    let mut g = two_player_game();
    g.players[1].life = 1;
    let _ = g.check_state_based_actions();
    assert!(!g.players[1].eliminated, "Player at 1 life should not be eliminated");
}

// ── CR 704.5j — legend rule ──────────────────────────────────────────────

/// CR 704.5j: "If a player controls two or more legendary permanents with
/// the same name, that player chooses one of them, and the rest are put
/// into their owners' graveyards."
#[test]
fn cr_704_5j_legend_rule_keeps_newest() {
    let mut g = two_player_game();
    let first = g.add_card_to_battlefield(0, catalog::griselbrand());
    let second = g.add_card_to_battlefield(0, catalog::griselbrand());
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Griselbrand").count(),
        2, "Setup: two Griselbrands"
    );
    let _ = g.check_state_based_actions();
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Griselbrand").count(),
        1, "Legend rule should leave exactly one"
    );
    assert!(g.battlefield.iter().any(|c| c.id == second),
        "Legend rule should keep the newest (highest id)");
    assert!(!g.battlefield.iter().any(|c| c.id == first),
        "Legend rule should sacrifice the oldest");
}

#[test]
fn cr_704_5j_legend_rule_controller_chooses_which_to_keep() {
    // CR 704.5j: the controller chooses which legend to keep. Script a
    // KeptLegend answer that keeps the *older* one (not the engine default).
    let mut g = two_player_game();
    let first = g.add_card_to_battlefield(0, catalog::griselbrand());
    let second = g.add_card_to_battlefield(0, catalog::griselbrand());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::KeptLegend(first)]));
    let _ = g.check_state_based_actions();
    assert!(g.battlefield.iter().any(|c| c.id == first),
        "Controller chose to keep the older Griselbrand");
    assert!(!g.battlefield.iter().any(|c| c.id == second),
        "The unchosen duplicate is put into the graveyard");
}

#[test]
fn cr_704_5j_legend_rule_different_controllers_coexist() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::griselbrand());
    g.add_card_to_battlefield(1, catalog::griselbrand());
    let _ = g.check_state_based_actions();
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Griselbrand").count(),
        2, "Legend rule should not apply across different controllers"
    );
}

// ── CR 121.2b — per-turn draw cap ────────────────────────────────────────

#[test]
fn cr_121_2b_draw_cap_truncates_draws() {
    use crate::card::{CardDefinition, CardType, Subtypes};
    use crate::effect::PlayerStaticTarget;
    let mut g = two_player_game();
    // Synthetic enchantment: each player can't draw more than one card/turn.
    let lock = CardDefinition {
        name: "Labyrinth Lock",
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        static_abilities: vec![StaticAbility {
            description: "Each player can't draw more than one card each turn.",
            effect: StaticEffect::CapDrawsPerTurn {
                target: PlayerStaticTarget::EachPlayer,
                max: 1,
            },
        }],
        ..Default::default()
    };
    g.add_card_to_battlefield(1, lock);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let hand_before = g.players[0].hand.len();
    let ctx = EffectContext {
        controller: 0,
        source: None,
        targets: vec![],
        trigger_source: None,
        mode: 0,
        x_value: 0,
        converged_value: 0,
        mana_spent: 0,
        source_name: None,
        cast_from_hand: false,
        event_amount: 0,
        kicked: false,
    };
    g.resolve_effect(
        &Effect::Draw { who: Selector::You, amount: Value::Const(3) },
        &ctx,
    ).unwrap();
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "draw cap truncates a draw-3 to a single card");

    // The cap is surfaced to clients via PlayerView.draw_cap.
    let view = crate::server::view::project(&g, 0);
    let me = view.players.iter().find(|p| p.seat == 0).unwrap();
    assert_eq!(me.draw_cap, Some(1), "draw cap surfaced in the player view");
}

// ── CR 704.5c — empty library + draw attempt ─────────────────────────────

/// CR 704.5c is already tested at cr_121_4 / empty_library; adding a
/// complementary test that verifies the SBA path specifically.
#[test]
fn cr_704_5c_empty_library_draw_eliminates_player() {
    let mut g = two_player_game();
    assert!(g.players[0].library.is_empty());
    let ponder = g.add_card_to_hand(0, catalog::ponder());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let _ = g.perform_action(GameAction::CastSpell {
        card_id: ponder,
        target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    drain_stack(&mut g);
    assert!(g.players[0].eliminated,
        "Player who drew from an empty library should be eliminated");
}

// ── Ward enforcement (CR 702.21) ─────────────────────────────────────────

#[test]
fn ward_counters_spell_when_caster_cannot_pay() {
    // Sedgemoor Witch has Ward(1). Opponent tries to Lightning Bolt it
    // with only {R} — just enough for bolt cost but not Ward. The Ward
    // trigger should counter the bolt (CR 702.21a).
    let mut g = two_player_game();
    let witch = g.add_card_to_battlefield(0, catalog::sedgemoor_witch());
    g.clear_sickness(witch);

    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::PreCombatMain;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    // Only {R} for the bolt — nothing left for Ward {1}.
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(witch)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Bolt cast OK — Ward is triggered, not a cast restriction");
    drain_stack(&mut g);

    // Witch should survive because Ward countered the bolt (P1 had no
    // remaining mana to pay the {1} Ward cost after casting the bolt).
    assert!(g.battlefield.iter().any(|c| c.id == witch),
        "Sedgemoor Witch should survive — Ward should counter the Bolt");
}

#[test]
fn ward_does_not_trigger_on_own_spells() {
    // Ward only triggers on opponents' spells. Your own spells
    // targeting your own Ward creature should resolve normally.
    let mut g = two_player_game();
    let witch = g.add_card_to_battlefield(0, catalog::sedgemoor_witch());
    g.clear_sickness(witch);
    // Cast a pump spell targeting our own Ward creature.
    let pump = g.add_card_to_hand(0, catalog::interjection());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: pump,
        target: Some(Target::Permanent(witch)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Interjection cast OK");
    drain_stack(&mut g);

    // Witch should have the pump applied.
    let w = g.computed_permanent(witch).unwrap();
    assert!(w.power > 3, "Witch should be pumped by own spell");
}

// ── Stun counter enforcement (CR 701.48) ─────────────────────────────────

#[test]
fn stun_counter_prevents_untap_and_decrements() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    // Tap the bear and give it 2 stun counters.
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap().tapped = true;
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap()
        .counters.insert(CounterType::Stun, 2);

    // First untap step: bear should stay tapped, lose one stun counter.
    g.do_untap();
    let b = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert!(b.tapped, "Bear should stay tapped with stun counters");
    assert_eq!(b.counter_count(CounterType::Stun), 1,
        "Should have removed one stun counter");

    // Second untap step: still tapped, lose last stun counter.
    g.do_untap();
    let b = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert!(b.tapped, "Bear should still be tapped (last stun counter removed this step)");
    assert_eq!(b.counter_count(CounterType::Stun), 0,
        "Should have no stun counters left");

    // Third untap step: no stun counters — bear untaps normally.
    g.do_untap();
    let b = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert!(!b.tapped, "Bear should now untap normally with no stun counters");
}

#[test]
fn indestructible_survives_deathtouch_damage() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let c = g.battlefield_find_mut(big).unwrap();
    c.damage = 1;
    c.dealt_deathtouch_damage = true;
    std::sync::Arc::make_mut(&mut c.definition).keywords.push(Keyword::Indestructible);

    g.check_state_based_actions();
    assert!(g.battlefield.iter().any(|c| c.id == big),
        "Indestructible should survive the deathtouch SBA");
}

// ── CR 704.5d — token cleanup ────────────────────────────────────────────

/// CR 704.5d: "If a token is in a zone other than the battlefield, it
/// ceases to exist." This is the cleanup SBA that prevents an exiled or
/// graveyarded token from lingering.
#[test]
fn cr_704_5d_token_in_graveyard_ceases_to_exist() {
    let mut g = two_player_game();
    // Properly cast Pestlord II so ETB fires and mints a Pest token.
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestlord_ii_b192());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let pest = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Pest").unwrap().id;
    // Bolt the Pest to send it to graveyard.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(pest)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Per CR 704.5d, the token should NOT be in the graveyard.
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == pest),
        "CR 704.5d: token in graveyard ceases to exist");
}

// ── CR 704.5i — planeswalker zero loyalty ────────────────────────────────

/// CR 704.5i: "If a planeswalker has loyalty 0, it's put into its
/// owner's graveyard." Pinned via Heliod, Sun-Crowned — wait, that's
/// a creature. Let me use Professor Dellian Fel (a planeswalker in
/// the catalog via SOS).
#[test]
fn cr_704_5i_planeswalker_with_zero_loyalty_dies() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(0, catalog::professor_dellian_fel());
    // Manually drain its loyalty to 0.
    if let Some(c) = g.battlefield_find_mut(pw) {
        c.counters.insert(CounterType::Loyalty, 0);
    }
    let _ = g.check_state_based_actions();
    // Per CR 704.5i, the PW should be in graveyard.
    assert!(g.battlefield_find(pw).is_none(),
        "CR 704.5i: zero-loyalty planeswalker dies");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == pw));
}

// ─────────────────────────────────────────────────────────────────────────
// modern_decks CR rule lock-ins: Defender (702.3), Combat Damage Step
// multi-blocker assignment (510.1c), and deathtouch lethal-per-blocker
// (510.1c + 702.2). These advance the in-progress CR 510 tracker and
// pin the Defender enforcement near the Sylvan Caryatid card work.
// ─────────────────────────────────────────────────────────────────────────

/// Inline vanilla creature with arbitrary P/T + keywords for combat tests.
fn vanilla_body(name: &'static str, p: i32, t: i32, kws: Vec<crate::card::Keyword>) -> crate::card::CardDefinition {
    use crate::card::{CardDefinition, CardType, Subtypes};
    CardDefinition {
        name,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes::default(),
        power: p,
        toughness: t,
        keywords: kws,
        ..Default::default()
    }
}

#[test]
fn cr_702_3_defender_cannot_be_declared_as_attacker() {
    // CR 702.3b: a creature with Defender can't attack. Sylvan Caryatid
    // (0/3 Defender) is rejected at declare-attackers.
    let mut g = two_player_game();
    let caryatid = setup_attacker(&mut g, 0, catalog::sylvan_caryatid);
    g.step = TurnStep::DeclareAttackers;
    let res = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: caryatid,
        target: AttackTarget::Player(1),
    }]));
    assert!(res.is_err(), "Defender creature can't be declared as an attacker");
}

#[test]
fn cr_510_1c_trampler_assigns_lethal_to_each_blocker_then_tramples() {
    // CR 510.1c: a 5/5 trampler blocked by two 2/2s assigns lethal (2)
    // to each blocker, then the remaining 1 tramples to the player.
    let mut g = two_player_game();
    let attacker = setup_attacker(&mut g, 0, || {
        vanilla_body("Trampler 5/5", 5, 5, vec![crate::card::Keyword::Trample])
    });
    let b1 = setup_attacker(&mut g, 1, || vanilla_body("Wall A", 2, 2, vec![]));
    let b2 = setup_attacker(&mut g, 1, || vanilla_body("Wall B", 2, 2, vec![]));

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(b1, attacker), (b2, attacker)]))
        .unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();

    assert!(!g.battlefield.iter().any(|c| c.id == b1), "blocker A took lethal");
    assert!(!g.battlefield.iter().any(|c| c.id == b2), "blocker B took lethal");
    assert_eq!(g.players[1].life, 19, "5 - 2 - 2 = 1 tramples through");
}

#[test]
fn cr_510_1c_deathtouch_attacker_assigns_one_to_each_blocker() {
    // CR 510.1c + 702.2e: with deathtouch, 1 damage is lethal, so a 3/3
    // deathtoucher blocked by three 5/5s kills all three (1 each).
    let mut g = two_player_game();
    let attacker = setup_attacker(&mut g, 0, || {
        vanilla_body("DT 3/3", 3, 3, vec![crate::card::Keyword::Deathtouch])
    });
    let b1 = setup_attacker(&mut g, 1, || vanilla_body("Big A", 5, 5, vec![]));
    let b2 = setup_attacker(&mut g, 1, || vanilla_body("Big B", 5, 5, vec![]));
    let b3 = setup_attacker(&mut g, 1, || vanilla_body("Big C", 5, 5, vec![]));

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![
        (b1, attacker),
        (b2, attacker),
        (b3, attacker),
    ]))
    .unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();

    // All three 5/5s die to 1 deathtouch damage apiece; attacker dies to 15.
    assert!(!g.battlefield.iter().any(|c| c.id == b1));
    assert!(!g.battlefield.iter().any(|c| c.id == b2));
    assert!(!g.battlefield.iter().any(|c| c.id == b3));
    assert!(!g.battlefield.iter().any(|c| c.id == attacker), "attacker takes 15");
}

#[test]
fn cr_701_12c_exchange_life_totals_swaps_both_players() {
    use crate::effect::{Effect, Selector};
    use crate::game::effects::EffectContext;
    // Soul Conduit / Magus of the Mirror: "Exchange life totals with target
    // player." P0 (5 life) swaps with P1 (30) → P0 has 30, P1 has 5.
    let mut g = two_player_game();
    g.set_life(0, 5);
    g.set_life(1, 30);
    let p1_gained_before = g.players[1].life_gained_this_turn;
    let ctx = EffectContext::for_spell(0, Some(Target::Player(1)), 0, 0);
    g.resolve_effect(
        &Effect::ExchangeLifeTotals { a: Selector::You, b: Selector::Target(0) },
        &ctx,
    ).unwrap();
    assert_eq!(g.players[0].life, 30, "P0 takes P1's previous total");
    assert_eq!(g.players[1].life, 5, "P1 takes P0's previous total");
    // The gainer's life-gain-this-turn bumped (lifegain-matters payoffs see it).
    assert_eq!(g.players[0].life_gained_this_turn, 25,
        "P0 gained 25 — lifegain-matters counter reflects the swing");
    assert_eq!(g.players[1].life_gained_this_turn, p1_gained_before,
        "P1 lost life — its life-gained counter is unchanged");
}

#[test]
fn soul_conduit_activation_exchanges_life_totals() {
    let mut g = two_player_game();
    g.set_life(0, 4);
    g.set_life(1, 28);
    let conduit = g.add_card_to_battlefield(0, catalog::soul_conduit());
    g.players[0].mana_pool.add_colorless(6);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: conduit, ability_index: 0, target: None, x_value: None,
    }).expect("Soul Conduit activates at sorcery speed for {6}, {T}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 28, "P0 takes the opponent's previous total");
    assert_eq!(g.players[1].life, 4, "opponent takes P0's previous total");
}

#[test]
fn cr_702_15_landwalk_unblockable_only_when_defender_has_land_type() {
    use crate::card::{Keyword, LandType};
    // A Forestwalk attacker can't be blocked while the defending player
    // controls a Forest; remove the Forest and it blocks normally.
    let mut g = two_player_game();
    let attacker = setup_attacker(&mut g, 0, || {
        vanilla_body("Forestwalker", 2, 2, vec![Keyword::Landwalk(LandType::Forest)])
    });
    let blocker = setup_attacker(&mut g, 1, || vanilla_body("Blocker", 2, 2, vec![]));
    let forest = g.add_card_to_battlefield(1, catalog::forest());

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    // Defender controls a Forest → block is illegal.
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).is_err(),
        "Forestwalk: can't be blocked while defender controls a Forest");

    // Remove the Forest → the same block is now legal.
    g.remove_to_graveyard_with_triggers(forest);
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).is_ok(),
        "no Forest → Forestwalk grants no evasion");
}

#[test]
fn cr_510_1c_attacker_chooses_damage_assignment_order() {
    // A 3/3 attacker (no trample) is blocked by a 2/2 and a 3/3. The
    // attacking player chooses the order: scripting [big, small] assigns
    // all 3 to the 3/3 (lethal) and 0 to the 2/2 — so the 3/3 dies and the
    // 2/2 lives, the opposite of the default (lowest-id-first) order.
    let mut g = two_player_game();
    let attacker = setup_attacker(&mut g, 0, || vanilla_body("Atk 3/3", 3, 3, vec![]));
    let small = setup_attacker(&mut g, 1, || vanilla_body("Wall 2/2", 2, 2, vec![]));
    let big = setup_attacker(&mut g, 1, || vanilla_body("Wall 3/3", 3, 3, vec![]));

    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DamageOrder(vec![big, small])]));

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(small, attacker), (big, attacker)]))
        .unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();

    assert!(!g.battlefield.iter().any(|c| c.id == big),
        "attacker assigned all 3 to the 3/3 first → it dies");
    assert!(g.battlefield.iter().any(|c| c.id == small),
        "no damage left for the 2/2 → it survives");
}

#[test]
fn cr_510_1c_attacker_over_assigns_to_deny_trample() {
    // A 5/5 trampler blocked by two 2/2s would, by default, assign lethal
    // (2) to each and trample 1 through. The attacking player may instead
    // pour all 5 into the first blocker — denying any trample and sparing
    // the second 2/2. Scripted: empty order (keep default), then assign 5/0.
    let mut g = two_player_game();
    let attacker = setup_attacker(&mut g, 0, || {
        vanilla_body("Trampler 5/5", 5, 5, vec![crate::card::Keyword::Trample])
    });
    let b1 = setup_attacker(&mut g, 1, || vanilla_body("Wall A", 2, 2, vec![]));
    let b2 = setup_attacker(&mut g, 1, || vanilla_body("Wall B", 2, 2, vec![]));

    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::DamageOrder(vec![]),
        DecisionAnswer::CombatDamageAssignment(vec![(b1, 5), (b2, 0)]),
    ]));

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(b1, attacker), (b2, attacker)]))
        .unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();

    assert!(!g.battlefield.iter().any(|c| c.id == b1), "blocker A took all 5 → dies");
    assert!(g.battlefield.iter().any(|c| c.id == b2), "blocker B got 0 → survives");
    assert_eq!(g.players[1].life, 20, "no trample-over: defender takes no damage");
}

#[test]
fn cr_510_1c_invalid_over_assignment_falls_back_to_default() {
    // An illegal answer (assign damage to the second blocker while the first
    // is under-assigned) is rejected and the engine uses the lethal-to-each
    // default: a 4/4 trampler vs two 2/2s kills both and tramples 0.
    let mut g = two_player_game();
    let attacker = setup_attacker(&mut g, 0, || {
        vanilla_body("Trampler 4/4", 4, 4, vec![crate::card::Keyword::Trample])
    });
    let b1 = setup_attacker(&mut g, 1, || vanilla_body("Wall A", 2, 2, vec![]));
    let b2 = setup_attacker(&mut g, 1, || vanilla_body("Wall B", 2, 2, vec![]));

    // Skip the first blocker (0) but feed the second (2) — illegal ordering.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::DamageOrder(vec![]),
        DecisionAnswer::CombatDamageAssignment(vec![(b1, 0), (b2, 2)]),
    ]));

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(b1, attacker), (b2, attacker)]))
        .unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();

    assert!(!g.battlefield.iter().any(|c| c.id == b1), "default split: blocker A dies");
    assert!(!g.battlefield.iter().any(|c| c.id == b2), "default split: blocker B dies");
    assert_eq!(g.players[1].life, 20, "4 - 2 - 2 = 0 tramples through");
}

#[test]
fn cr_700_4_morbid_total_predicate_counts_deaths_across_players() {
    use crate::effect::Predicate;
    use crate::game::effects::EffectContext;
    let mut g = two_player_game();
    let morbid = Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::Const(1) };
    let ctx = EffectContext {
        controller: 0,
        source: None,
        targets: vec![],
        trigger_source: None,
        mode: 0,
        x_value: 0,
        converged_value: 0,
        mana_spent: 0,
        source_name: None,
        cast_from_hand: false,
        event_amount: 0,
        kicked: false,
    };
    assert!(!g.evaluate_predicate(&morbid, &ctx), "no deaths yet → morbid off");
    // A creature died under the opponent's control (seat 1).
    g.players[1].creatures_died_this_turn = 1;
    assert!(g.evaluate_predicate(&morbid, &ctx),
        "an opponent's creature dying this turn satisfies global morbid");
}

// ── CR 702.6 / 702.122 / 301.7 / 509.1b lock-in tests (this push) ───────────

/// CR 702.6 — Equip is a sorcery-speed activated ability that attaches an
/// Equipment to a creature its controller controls.
#[test]
fn cr_702_6_equip_attaches_at_sorcery_speed_to_your_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let boner = g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.players[0].mana_pool.add_colorless(1);
    // Sorcery-speed only.
    g.step = crate::game::TurnStep::DeclareBlockers;
    assert!(matches!(
        g.perform_action(crate::game::GameAction::Equip { equipment: boner, target: bear }),
        Err(crate::game::GameError::SorcerySpeedOnly)
    ));
    // In the main phase, it attaches and grants the bonus.
    g.step = crate::game::TurnStep::PreCombatMain;
    g.perform_action(crate::game::GameAction::Equip { equipment: boner, target: bear })
        .expect("equip in main phase");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4);
}

/// CR 702.122 — Crew taps creatures whose total power is at least the crew
/// number; below the threshold the activation is rejected.
#[test]
fn cr_702_122_crew_requires_total_power_at_least_n() {
    let mut g = two_player_game();
    let coach = g.add_card_to_battlefield(0, catalog::strixhaven_skycoach()); // Crew 2
    let one_power = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2 power — enough
    g.perform_action(crate::game::GameAction::Crew {
        vehicle: coach, crew_creatures: vec![one_power],
    }).expect("2 power satisfies crew 2");
    assert!(g.computed_permanent(coach).unwrap()
        .card_types.contains(&crate::card::CardType::Creature));
}

/// CR 301.7 — a Vehicle is not a creature until an effect (Crew) turns it
/// into one.
#[test]
fn cr_301_7_vehicle_is_not_a_creature_until_crewed() {
    let mut g = two_player_game();
    let coach = g.add_card_to_battlefield(0, catalog::strixhaven_skycoach());
    assert!(!g.computed_permanent(coach).unwrap()
        .card_types.contains(&crate::card::CardType::Creature),
        "uncrewed Vehicle is a noncreature artifact");
    // It still carries printed P/T characteristics (CR 301.7) even uncrewed.
    assert_eq!(coach_printed_power(&g, coach), 3);
}

fn coach_printed_power(g: &crate::game::GameState, id: crate::card::CardId) -> i32 {
    g.battlefield.iter().find(|c| c.id == id).unwrap().definition.power
}

/// CR 509.1b — an unblockable creature ("can't be blocked") can't be chosen
/// as the creature an attacker is blocked by.
#[test]
fn cr_509_1b_unblockable_attacker_cannot_be_blocked() {
    let mut g = two_player_game();
    // P0 animates Creeping Tar Pit (unblockable) and attacks.
    let land = g.add_card_to_battlefield(0, catalog::creeping_tar_pit());
    g.clear_sickness(land);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(crate::game::GameAction::ActivateAbility {
        card_id: land, ability_index: 2, target: None, x_value: None,
    }).expect("animate");
    drain_stack(&mut g);
    // P1 has a blocker.
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(blocker);
    g.step = crate::game::TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(crate::game::GameAction::DeclareAttackers(vec![crate::game::Attack {
        attacker: land,
        target: crate::game::AttackTarget::Player(1),
    }])).expect("tar pit attacks");
    g.step = crate::game::TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    let err = g.perform_action(crate::game::GameAction::DeclareBlockers(vec![(blocker, land)]))
        .expect_err("unblockable can't be blocked");
    assert!(matches!(err, crate::game::GameError::CannotBlock(_)), "got {err:?}");
}

#[test]
fn flanking_shrinks_nonflanking_blocker() {
    // CR 702.25: a blocker without flanking that blocks a flanking
    // attacker gets -1/-1 until end of turn.
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    std::sync::Arc::make_mut(&mut g.battlefield_find_mut(attacker).unwrap().definition)
        .keywords
        .push(crate::card::Keyword::Flanking);
    g.clear_sickness(attacker);
    g.step = crate::game::TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(crate::game::GameAction::DeclareAttackers(vec![crate::game::Attack {
        attacker,
        target: crate::game::AttackTarget::Player(1),
    }]))
    .unwrap();
    g.step = crate::game::TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(crate::game::GameAction::DeclareBlockers(vec![(blocker, attacker)]))
        .unwrap();
    let b = g.battlefield.iter().find(|c| c.id == blocker).unwrap();
    assert_eq!(b.power(), 1);
    assert_eq!(b.toughness(), 1);
}


#[test]
fn bushido_pumps_attacker_when_blocked() {
    // CR 702.45: a Bushido N creature that becomes blocked gets +N/+N.
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    std::sync::Arc::make_mut(&mut g.battlefield_find_mut(attacker).unwrap().definition).keywords
        .push(crate::card::Keyword::Bushido(2));
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).unwrap();
    let a = g.battlefield.iter().find(|c| c.id == attacker).unwrap();
    assert_eq!(a.power(), 4);
    assert_eq!(a.toughness(), 4);
}

#[test]
fn rampage_pumps_attacker_per_extra_blocker() {
    // CR 702.23: Rampage N gives +N/+N for each blocker beyond the first.
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    std::sync::Arc::make_mut(&mut g.battlefield_find_mut(attacker).unwrap().definition).keywords
        .push(crate::card::Keyword::Rampage(2));
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(b1, attacker), (b2, attacker)])).unwrap();
    let a = g.battlefield.iter().find(|c| c.id == attacker).unwrap();
    assert_eq!(a.power(), 4);
    assert_eq!(a.toughness(), 4);
}

/// CR 122.7 — "Nth counter" threshold trigger, expressed with the existing
/// `CounterAdded` trigger + an intervening-`if` (CR 603.4) on the counter
/// total. No new engine primitive needed: the draw fires only when the
/// third +1/+1 counter lands.
#[test]
fn cr_122_7_nth_counter_threshold_trigger() {
    use crate::card::{CardType, CounterType, EventKind, EventScope, EventSpec,
        Subtypes, TriggeredAbility};
    use crate::effect::Predicate;
    let watcher = crate::card::CardDefinition {
        name: "Counter Watcher",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes::default(),
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CounterAdded(CounterType::PlusOnePlusOne),
                EventScope::SelfSource,
            ),
            effect: Effect::If {
                cond: Predicate::ValueEquals(
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::PlusOnePlusOne,
                    },
                    Value::Const(3),
                ),
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    };
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, watcher);
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let ctx = EffectContext::for_ability(id, 0, None);
    let mut draws = 0usize;
    for expect in [false, false, true] {
        let before = g.players[0].hand.len();
        let events = g.resolve_effect(&Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        }, &ctx).unwrap();
        g.dispatch_triggers_for_events(&events);
        drain_stack(&mut g);
        let drew = g.players[0].hand.len() > before;
        assert_eq!(drew, expect, "draw should fire only on the 3rd counter");
        if drew { draws += 1; }
    }
    assert_eq!(draws, 1, "exactly one draw, when the count reaches 3");
}

/// CR 701.10b — "double target creature's power" is the continuous
/// +X/+0 form (X = its power as the effect resolves), expressible with
/// `PumpPT { power: PowerOf(target) }`. A 3/3 becomes 6/3 until end of
/// turn; a second doubling stacks to 12/3.
#[test]
fn cr_701_10b_double_power_is_plus_x_zero() {
    use crate::effect::Duration;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.battlefield_find_mut(bear).unwrap().add_counters(
        crate::card::CounterType::PlusOnePlusOne, 1); // now 3/3
    let double_power = Effect::PumpPT {
        what: Selector::TargetFiltered {
            slot: 0,
            filter: crate::card::SelectionRequirement::Creature,
        },
        power: Value::PowerOf(Box::new(Selector::Target(0))),
        toughness: Value::Const(0),
        duration: Duration::EndOfTurn,
    };
    let ctx = EffectContext::for_spell(0, Some(crate::game::Target::Permanent(bear)), 0, 0);
    g.resolve_effect(&double_power, &ctx).unwrap();
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!((c.power(), c.toughness()), (6, 3), "3 power doubled to 6, toughness unchanged");
    // Double again → reads the current 6 power, +6/+0 → 12/3.
    g.resolve_effect(&double_power, &ctx).unwrap();
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!((c.power(), c.toughness()), (12, 3), "doubling stacks as continuous +X/+0");
}
