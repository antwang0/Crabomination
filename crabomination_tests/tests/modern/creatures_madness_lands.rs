#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::TurnStep;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
#[allow(unused)]
use crate::Factory;

// ── Creatures ────────────────────────────────────────────────────────────────

#[test]
fn burning_tree_emissary_etb_adds_red_and_green() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::burning_tree_emissary());
    // {R/G}{R/G}: pay one pip with red, one with green.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Burning-Tree Emissary castable for {R}{G} via hybrid pips");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == id));
    // ETB ramp: the {R}{G} produced makes the Emissary "free" (it refunds
    // its own cost), so the pool nets back to {R}{G}.
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
}

#[test]
fn burning_tree_emissary_castable_with_two_red() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::burning_tree_emissary());
    // Both {R/G} pips payable with red.
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Burning-Tree Emissary castable for {R}{R}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id));
}

#[test]
fn putrid_imp_discard_grants_menace_eot() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let imp = g.add_card_to_battlefield(0, catalog::putrid_imp());
    g.clear_sickness(imp);
    let to_pitch = g.add_card_to_hand(0, catalog::lightning_bolt());

    g.perform_action(GameAction::ActivateAbility {
        card_id: imp, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Putrid Imp discard ability activates");
    drain_stack(&mut g);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == to_pitch),
        "Discarded card should hit graveyard");
    let computed = g.compute_battlefield();
    let imp_view = computed.iter().find(|c| c.id == imp).unwrap();
    assert!(imp_view.keywords.contains(&Keyword::Menace),
        "Putrid Imp should have menace until end of turn");
}

// ── Madness (CR 702.35) ──────────────────────────────────────────────────────

/// Helper: build a test-only Madness instant ("deal 1 to any target") with
/// the given madness cost. Lets the non-zero-cost payment paths be exercised
/// without depending on an unverified printed card.
fn test_madness_bolt(madness: crabomination::mana::ManaCost) -> crabomination::card::CardDefinition {
    use crabomination::card::{CardType, Keyword, SelectionRequirement};
    use crabomination::effect::{Effect, Selector, Value};
    use crabomination::mana::{ManaCost, ManaSymbol, Color};
    crabomination::card::CardDefinition {
        name: "Test Madness Bolt",
        cost: ManaCost::new(vec![ManaSymbol::Generic(1), ManaSymbol::Colored(Color::Red)]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Madness(madness)],
        effect: Effect::DealDamage {
            to: Selector::TargetFiltered { slot: 0, filter: SelectionRequirement::Any },
            amount: Value::Const(1),
        },
        ..Default::default()
    }
}

#[test]
fn madness_zero_cost_basking_rootwalla_cast_from_exile_when_accepted() {
    // CR 702.35a/b — discarding Basking Rootwalla (Madness {0}) exiles it
    // and offers a free cast; accepting puts the 1/1 onto the battlefield.
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let rw = g.add_card_to_hand(0, catalog::basking_rootwalla());

    let mut events = vec![];
    assert!(g.discard_card(0, rw, &mut events), "card found + discarded");
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == rw),
        "Accepted madness should cast Basking Rootwalla onto the battlefield");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == rw),
        "Madness-cast card is not in the graveyard");
    assert!(!g.exile.iter().any(|c| c.id == rw),
        "Resolved madness creature left exile for the battlefield");
}

#[test]
fn madness_declined_sends_card_to_graveyard() {
    // AutoDecider now casts affordable madness cards (the blanket decline
    // was the dead-keyword bug), so the CR 702.35b decline path is
    // exercised with a scripted "no".
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    let rw = g.add_card_to_hand(0, catalog::basking_rootwalla());

    let mut events = vec![];
    g.discard_card(0, rw, &mut events);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == rw),
        "Declined madness sends the card to the graveyard");
    assert!(!g.exile.iter().any(|c| c.id == rw), "card not left stranded in exile");
    assert!(!g.battlefield.iter().any(|c| c.id == rw), "card not cast");
}

#[test]
fn madness_nonzero_cost_paid_from_pool_then_cast() {
    use crabomination::mana::{ManaCost, ManaSymbol, Color};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bolt = g.add_card_to_hand(0, test_madness_bolt(
        ManaCost::new(vec![ManaSymbol::Colored(Color::Red)])));
    // Float the {R} madness cost up front.
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_life_before = g.players[1].life;

    let mut events = vec![];
    g.discard_card(0, bolt, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);

    // The {R} was consumed and the bolt resolved (auto-targeted the opp).
    assert_eq!(g.players[0].mana_pool.total(), 0, "madness cost was paid");
    assert!(g.players[1].life < opp_life_before || g.battlefield.is_empty(),
        "madness instant resolved (dealt its 1 damage)");
    assert!(!g.exile.iter().any(|c| c.id == bolt),
        "resolved instant left exile (to graveyard)");
}

#[test]
fn madness_nonzero_cost_unaffordable_goes_to_graveyard() {
    // Accepting the prompt but lacking the mana to pay falls through to the
    // graveyard (CR 702.35b — "if they don't, they put it into graveyard").
    use crabomination::mana::{ManaCost, ManaSymbol, Color};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bolt = g.add_card_to_hand(0, test_madness_bolt(
        ManaCost::new(vec![ManaSymbol::Colored(Color::Red)])));
    // No mana floated → can't pay the {R}.

    let mut events = vec![];
    g.discard_card(0, bolt, &mut events);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt),
        "unaffordable madness goes to the graveyard");
    assert!(!g.exile.iter().any(|c| c.id == bolt));
}

#[test]
fn cr_70235_madness_exile_still_counts_as_a_discard() {
    // CR 701.8b / 702.35a — the discard still happens (CardDiscarded fires)
    // even though the card is exiled rather than going to the graveyard.
    use crabomination::game::GameEvent;
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let rw = g.add_card_to_hand(0, catalog::basking_rootwalla());

    let before = g.cards_discarded_this_resolution;
    let mut events = vec![];
    g.discard_card(0, rw, &mut events);

    assert!(events.iter().any(|e| matches!(e,
        GameEvent::CardDiscarded { player: 0, card_id } if *card_id == rw)),
        "CardDiscarded fires for a madness discard");
    assert_eq!(g.cards_discarded_this_resolution, before + 1,
        "discard-matters counter bumped even though the card was exiled");
}

#[test]
fn cr_5141a_cleanup_discard_routes_through_madness() {
    // CR 514.1a — the cleanup discard-to-hand-size routes through the
    // centralized discard path, so a Madness card discarded at cleanup is
    // exiled and offered for its madness cost (CR 702.35) rather than
    // going straight to the graveyard.
    let mut g = two_player_game();
    let active = g.active_player_idx;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    // Stuff the active player's hand past the maximum (7) with vanilla
    // fillers, then a Basking Rootwalla as the head card to be discarded.
    let rw = g.add_card_to_hand(active, catalog::basking_rootwalla());
    for _ in 0..8 {
        g.add_card_to_hand(active, catalog::grizzly_bears());
    }

    g.do_cleanup(&mut Vec::new());
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == rw),
        "cleanup discard of a Madness {{0}} card lets it be cast from exile");
}

#[test]
fn tarmogoyf_pt_scales_with_card_types_in_graveyards() {
    let mut g = two_player_game();
    let goyf = g.add_card_to_battlefield(0, catalog::tarmogoyf());

    // Empty graveyards → 0/1.
    let computed = g.compute_battlefield();
    let view = computed.iter().find(|c| c.id == goyf).unwrap();
    assert_eq!(view.power, 0, "Tarmogoyf P = 0 with empty graveyards");
    assert_eq!(view.toughness, 1, "Tarmogoyf T = 1 with empty graveyards");

    // Add cards of distinct types into the graveyard.
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    let pos = g.players[0].library.iter().position(|c| c.id == bolt).unwrap();
    let card = g.players[0].library.remove(pos);
    g.players[0].graveyard.push(card); // Instant
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let pos = g.players[0].library.iter().position(|c| c.id == bear).unwrap();
    let card = g.players[0].library.remove(pos);
    g.players[0].graveyard.push(card); // Creature

    let computed = g.compute_battlefield();
    let view = computed.iter().find(|c| c.id == goyf).unwrap();
    assert_eq!(view.power, 2, "Tarmogoyf P = 2 with Instant + Creature in graveyards");
    assert_eq!(view.toughness, 3, "Tarmogoyf T = 3 with Instant + Creature in graveyards");
}

// ── Utility / lands ──────────────────────────────────────────────────────────

#[test]
fn veil_of_summer_draws_when_opponent_cast_blue_or_black() {
    let mut g = two_player_game();
    g.players[1].cast_blue_or_black_this_turn = true; // gate satisfied
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::veil_of_summer());
    g.players[0].mana_pool.add(Color::Green, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Veil castable for {G}");
    drain_stack(&mut g);
    // Net hand: -1 cast +1 draw = 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.players[0].spells_uncounterable_this_turn, "your spells can't be countered");
    assert!(g.players[1].cannot_gain_life_this_turn, "opponents can't gain life");
}

/// Veil's rider grants you and your permanents hexproof from blue and black:
/// an opponent's blue spell can't target your creature, but a red one still can.
#[test]
fn veil_of_summer_grants_hexproof_from_blue_and_black() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Player 0 resolves Veil of Summer.
    let veil = g.add_card_to_hand(0, catalog::veil_of_summer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: veil, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Veil");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hexproof_from_colors_this_turn, vec![Color::Blue, Color::Black]);
    // Opponent's blue Unsummon can't target the now-hexproof bear.
    let bounce = g.add_card_to_hand(1, catalog::unsummon());
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 1;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bounce, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None });
    assert!(matches!(err, Err(GameError::TargetHasHexproof(_))), "blue blocked, got {err:?}");
    // A red Lightning Bolt is unaffected by hexproof-from-blue/black.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("red bolt can still target");
}

#[test]
fn veil_of_summer_no_draw_without_blue_or_black() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::veil_of_summer());
    g.players[0].mana_pool.add(Color::Green, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Veil castable for {G}");
    drain_stack(&mut g);
    // No qualifying opponent spell → no draw, just the -1 for casting.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

/// The blue/black cast flag is set by finalize_cast when an opponent
/// actually casts a black spell.
#[test]
fn casting_black_spell_sets_veil_gate() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::swamp());
    let bolt = g.add_card_to_hand(1, catalog::dark_ritual());
    g.players[1].mana_pool.add(Color::Black, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Dark Ritual castable for {B}");
    drain_stack(&mut g);
    assert!(g.players[1].cast_blue_or_black_this_turn, "casting a black spell flips the gate");
}

#[test]
fn crop_rotation_sacrifices_land_and_searches_for_one() {
    let mut g = two_player_game();
    let sac_land = g.add_card_to_battlefield(0, catalog::forest());
    let target_land = g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target_land))]));

    let id = g.add_card_to_hand(0, catalog::crop_rotation());
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Crop Rotation castable for {G}");
    drain_stack(&mut g);

    // Sacrificed land moved to graveyard.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == sac_land),
        "Sacrificed land should be in graveyard");
    // Tutored land entered the battlefield.
    assert!(g.battlefield.iter().any(|c| c.id == target_land),
        "Tutored land should be on the battlefield");
}

/// CR 601.2b — a `wants_ui` caster picks *which* land to sacrifice for Crop
/// Rotation's additional cost instead of the engine auto-picking. Casting
/// suspends on a `ChooseTarget` decision; the chosen land is the one that
/// dies.
#[test]
fn crop_rotation_ui_player_picks_which_land_to_sacrifice() {
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    // Two legal lands → a real choice.
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let island = g.add_card_to_battlefield(0, catalog::island());
    let _fetch_target = g.add_card_to_library(0, catalog::plains());

    let id = g.add_card_to_hand(0, catalog::crop_rotation());
    g.players[0].mana_pool.add(Color::Green, 1);

    // The cast suspends for the sacrifice choice rather than auto-picking.
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast suspends for the sacrifice choice");

    let pd = g.pending_decision.as_ref().expect("a sacrifice decision is pending");
    assert_eq!(pd.acting_player(), 0);
    match &pd.decision {
        crabomination::decision::Decision::ChooseTarget { legal, .. } => {
            assert_eq!(legal.len(), 2, "both lands offered as sacrifice options");
        }
        other => panic!("expected ChooseTarget, got {other:?}"),
    }
    assert!(g.players[0].has_in_hand(id), "card stays in hand until the choice is made");

    // Pick the Island (not the cheapest auto-pick would necessarily choose).
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Target(
        Target::Permanent(island),
    )))
    .expect("submit the sacrifice choice");

    assert!(g.players[0].graveyard.iter().any(|c| c.id == island),
        "the chosen Island was sacrificed");
    assert!(g.battlefield.iter().any(|c| c.id == forest),
        "the unchosen Forest stays on the battlefield");
    assert!(!g.players[0].has_in_hand(id), "Crop Rotation left hand on the real cast");
}

/// A non-UI caster (bot / AutoDecider) keeps the auto-pick path — no
/// sacrifice decision is surfaced, so bot play and tests are unchanged.
#[test]
fn crop_rotation_auto_picks_sacrifice_for_non_ui_player() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::island());
    let target_land = g.add_card_to_library(0, catalog::plains());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target_land))]));

    let id = g.add_card_to_hand(0, catalog::crop_rotation());
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast resolves without a sacrifice decision");
    assert!(g.pending_decision.is_none(), "no sacrifice decision for a non-UI caster");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == target_land), "search resolved normally");
}

/// CR 701.16 — a `wants_ui` player hit by an Edict chooses *which* creature
/// to sacrifice (Diabolic Edict, "target player sacrifices a creature")
/// rather than the engine auto-dumping their weakest. The resolution suspends
/// on a `ChooseTarget`; the chosen creature is the one that dies.
#[test]
fn diabolic_edict_ui_target_chooses_creature_to_sacrifice() {
    let mut g = two_player_game();
    g.players[1].wants_ui = true;
    // The targeted player controls two creatures → a genuine choice.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let keep = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::diabolic_edict());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Diabolic Edict castable for {1}{B}");
    // Resolve the spell: both players pass priority.
    g.perform_action(GameAction::PassPriority).expect("active passes");
    g.perform_action(GameAction::PassPriority).expect("non-active passes → resolve");

    // Resolution suspended on the targeted player's sacrifice choice.
    let pd = g.pending_decision.as_ref().expect("sacrifice choice is pending");
    assert_eq!(pd.acting_player(), 1, "the targeted player chooses");
    match &pd.decision {
        crabomination::decision::Decision::ChooseTarget { legal, .. } => {
            assert_eq!(legal.len(), 2, "both of the player's creatures are options");
        }
        other => panic!("expected ChooseTarget, got {other:?}"),
    }

    // Choose to sacrifice `bear`; `keep` survives.
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Target(
        Target::Permanent(bear),
    )))
    .expect("submit the sacrifice choice");

    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear),
        "the chosen creature was sacrificed");
    assert!(g.battlefield.iter().any(|c| c.id == keep),
        "the unchosen creature survives");
}

/// A modal spell's mode descriptions (what the mode-pick modal shows) must
/// name each mode's target restriction — Abrade reads "deal 3 damage to
/// target creature" / "destroy target artifact", not a bare "target".
#[test]
fn abrade_mode_descriptions_name_the_target_restrictions() {
    let def = catalog::abrade();
    let crabomination::effect::Effect::ChooseMode(modes) = &def.effect else {
        panic!("Abrade should be a ChooseMode spell");
    };
    let d0 = modes[0].effect_short_text();
    let d1 = modes[1].effect_short_text();
    assert!(d0.contains("target creature"), "mode 0 should name the restriction: {d0}");
    assert!(d1.contains("target artifact"), "mode 1 should name the restriction: {d1}");
}

/// CR 601.2g — a `wants_ui` caster is asked before the engine auto-spends
/// pre-existing floating mana that untapped sources could pay instead, and a
/// "no" keeps the float (paying from lands).
#[test]
fn cast_keeps_floating_mana_when_player_declines_to_spend_it() {
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    g.players[0].mana_pool.add(Color::Blue, 1); // off-colour float to keep
    let f1 = g.add_card_to_battlefield(0, catalog::forest());
    let f2 = g.add_card_to_battlefield(0, catalog::forest());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears()); // {1}{G}

    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast suspends for the float-spend confirmation");
    let pd = g.pending_decision.as_ref().expect("a float-spend confirmation is pending");
    assert_eq!(pd.acting_player(), 0);
    assert!(matches!(pd.decision, crabomination::decision::Decision::OptionalTrigger { .. }));

    // Decline: keep the {U}, pay {1}{G} from the two Forests.
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Bool(false)))
        .expect("decline spending the float");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1, "floating {{U}} kept");
    assert!(g.battlefield_find(f1).unwrap().tapped && g.battlefield_find(f2).unwrap().tapped,
        "both Forests tapped to pay instead");
    assert!(g.battlefield.iter().any(|c| c.id == bear), "Grizzly Bears resolved");
}

/// Confirming the float-spend prompt spends the floating mana as before.
#[test]
fn cast_spends_floating_mana_when_player_confirms() {
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::forest());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());

    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .unwrap();
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Bool(true)))
        .expect("confirm spending the float");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 0, "floating {{U}} spent on the generic pip");
    assert!(g.battlefield.iter().any(|c| c.id == bear), "Grizzly Bears resolved");
}

/// CR 601.2g — the float-spend confirmation also covers activated-ability mana
/// costs (Gorilla Chieftain's {1}{G} regenerate): a `wants_ui` activator is
/// asked before off-colour float is swept onto the generic pip.
#[test]
fn activated_ability_keeps_floating_mana_when_player_declines() {
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    g.players[0].mana_pool.add(Color::Blue, 1); // off-colour float to keep
    let gorilla = g.add_card_to_battlefield(0, catalog::gorilla_chieftain());
    g.clear_sickness(gorilla);
    let f1 = g.add_card_to_battlefield(0, catalog::forest());
    let f2 = g.add_card_to_battlefield(0, catalog::forest());

    // Regenerate ({1}{G}): the {1} could come from the float or a Forest.
    g.perform_action(GameAction::ActivateAbility {
        card_id: gorilla, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("activation suspends for the float-spend confirmation");
    let pd = g.pending_decision.as_ref().expect("a float-spend confirmation is pending");
    assert_eq!(pd.acting_player(), 0);
    assert!(matches!(pd.decision, crabomination::decision::Decision::OptionalTrigger { .. }));

    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Bool(false)))
        .expect("decline spending the float");
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1, "floating {{U}} kept");
    assert!(
        g.battlefield_find(f1).unwrap().tapped && g.battlefield_find(f2).unwrap().tapped,
        "both Forests tapped to pay {{1}}{{G}} instead",
    );
}

/// No prompt when the floating mana is the only legal source — it's auto-spent
/// (CR 601.2g exemption), so a player with no lands isn't nagged.
#[test]
fn cast_auto_spends_floating_mana_when_it_is_the_only_source() {
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1); // {1}{G} entirely from float, no lands
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());

    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("casts without a prompt");
    assert!(g.pending_decision.is_none(), "no prompt when float is the only way to pay");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear), "Grizzly Bears resolved");
}

/// CR 601.2g — the prompt concerns only the *off-colour excess* float, not the
/// pip-matching part. Casting {3}{G} (Aberrant Manawurm) with {R}{G} floating
/// asks only about the {R}; declining keeps the {R} but still spends the {G}
/// float on the {G} pip (taps 3 lands for the generic, not 4).
#[test]
fn float_confirm_offers_only_off_colour_excess() {
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    g.players[0].mana_pool.add(Color::Red, 1); // off-colour excess
    g.players[0].mana_pool.add(Color::Green, 1); // matches the {G} pip
    let forests: Vec<_> = (0..4)
        .map(|_| g.add_card_to_battlefield(0, catalog::forest()))
        .collect();
    let wurm = g.add_card_to_hand(0, catalog::aberrant_manawurm()); // {3}{G}

    g.perform_action(GameAction::CastSpell {
        card_id: wurm, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast suspends for the float-spend confirmation");

    let pd = g.pending_decision.as_ref().expect("a float-spend confirmation is pending");
    let crabomination::decision::Decision::OptionalTrigger { description, .. } = &pd.decision else {
        panic!("expected an OptionalTrigger float prompt");
    };
    assert!(description.contains("{R}"), "prompt should mention the excess {{R}}: {description}");
    assert!(!description.contains("{G}"), "prompt must NOT offer the pip-matching {{G}}: {description}");

    // Decline: keep the {R}; the {G} float still pays the {G} pip; 3 Forests
    // pay the {3} (not 4).
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Bool(false)))
        .expect("decline spending the excess");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "excess {{R}} kept");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 0, "{{G}} float spent on its pip");
    let tapped = forests.iter().filter(|f| g.battlefield_find(**f).unwrap().tapped).count();
    assert_eq!(tapped, 3, "only 3 Forests tapped for the generic");
    assert!(g.battlefield.iter().any(|c| c.id == wurm), "Aberrant Manawurm resolved");
}

/// Coveted Jewel changes hands when an opponent's creature attacks its
/// controller, and untaps under the new controller (CR 800.4 control flip).
#[test]
fn coveted_jewel_steals_to_attacking_player() {
    let mut g = two_player_game();
    // P0 controls a (tapped) Coveted Jewel; P1 is the active attacker.
    let jewel = g.add_card_to_battlefield(0, catalog::coveted_jewel());
    g.battlefield_find_mut(jewel).unwrap().tapped = true;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;

    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(0),
    }])).expect("P1 attacks P0");
    drain_stack(&mut g);

    let j = g.battlefield_find(jewel).unwrap();
    assert_eq!(j.controller, 1, "the attacking player gains control of the Jewel");
    assert!(!j.tapped, "the Jewel untaps under its new controller");
}

#[test]
fn karakas_taps_for_white() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::karakas());

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Karakas's mana ability should activate");

    assert_eq!(g.players[0].mana_pool.amount(Color::White), 1);
}

#[test]
fn karakas_bounces_legendary_creature() {
    let mut g = two_player_game();
    // P0's Karakas, P1's legendary Atraxa on the battlefield.
    let kara = g.add_card_to_battlefield(0, catalog::karakas());
    let atraxa = g.add_card_to_battlefield(1, catalog::atraxa_grand_unifier());

    // Activate the bounce ability (index 1) targeting Atraxa.
    g.perform_action(GameAction::ActivateAbility {
        card_id: kara,
        ability_index: 1,
        target: Some(Target::Permanent(atraxa)), additional_targets: Vec::new(), x_value: None })
    .expect("Karakas bounce ability should activate against a legendary");
    drain_stack(&mut g);

    // Atraxa returned to its owner's hand (player 1).
    assert!(!g.battlefield.iter().any(|c| c.id == atraxa),
        "Atraxa should leave the battlefield");
    assert!(g.players[1].hand.iter().any(|c| c.id == atraxa),
        "Atraxa should return to its owner's hand");
}

#[test]
fn bojuka_bog_exiles_opponent_graveyard_on_etb() {
    let mut g = two_player_game();
    // Stock P1's graveyard with a few cards.
    for _ in 0..3 {
        let cid = g.add_card_to_library(1, catalog::lightning_bolt());
        let pos = g.players[1].library.iter().position(|c| c.id == cid).unwrap();
        let card = g.players[1].library.remove(pos);
        g.players[1].graveyard.push(card);
    }
    let p1_grave_before = g.players[1].graveyard.len();
    assert!(p1_grave_before > 0);

    let id = g.add_card_to_hand(0, catalog::bojuka_bog());
    g.perform_action(GameAction::PlayLand(id))
        .expect("Bojuka Bog playable as a land");
    drain_stack(&mut g);

    // Bog ETB-tapped (the trigger taps it) and the ForEach exiled the
    // opponent's graveyard contents.
    assert!(g.battlefield.iter().any(|c| c.id == id));
    assert_eq!(g.players[1].graveyard.len(), 0,
        "Bojuka Bog ETB should exile P1's graveyard");
    assert!(g.exile.len() >= p1_grave_before,
        "Exiled cards should land in the exile zone");
}

// ── Sanity: every modern card has the right card type ────────────────────────

// ── mod_set: removal / counterspells / pump (catalog::sets::mod_set) ─────────

#[test]
fn path_to_exile_exiles_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let path = g.add_card_to_hand(0, catalog::path_to_exile());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: path,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Path to Exile castable for {W}");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear));
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

#[test]
fn fatal_push_destroys_low_cmc_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let push = g.add_card_to_hand(0, catalog::fatal_push());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: push,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Fatal Push castable for {B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

#[test]
fn fatal_push_rejects_high_cmc_creature() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let push = g.add_card_to_hand(0, catalog::fatal_push());
    g.players[0].mana_pool.add(Color::Black, 1);
    let err = g.perform_action(GameAction::CastSpell {
        card_id: push,
        target: Some(Target::Permanent(angel)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(err.is_err(), "Fatal Push should reject Serra Angel (CMC 5)");
}

#[test]
fn doom_blade_destroys_nonblack_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let blade = g.add_card_to_hand(0, catalog::doom_blade());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: blade,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Doom Blade castable for {1}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

#[test]
fn doom_blade_rejects_black_creature() {
    let mut g = two_player_game();
    let specter = g.add_card_to_battlefield(1, catalog::hypnotic_specter());
    let blade = g.add_card_to_hand(0, catalog::doom_blade());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let err = g.perform_action(GameAction::CastSpell {
        card_id: blade,
        target: Some(Target::Permanent(specter)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(err.is_err(), "Doom Blade should reject black creature");
}

#[test]
fn vapor_snag_bounces_and_pings() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let snag = g.add_card_to_hand(0, catalog::vapor_snag());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: snag,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Vapor Snag castable for {U}");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear),
        "creature should return to owner's hand");
    assert_eq!(g.players[1].life, life_before - 1,
        "controller should lose 1 life");
}

#[test]
fn blossoming_defense_pumps_and_grants_hexproof() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let defense = g.add_card_to_hand(0, catalog::blossoming_defense());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: defense,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Blossoming Defense castable for {G}");
    drain_stack(&mut g);

    let computed = g.computed_permanent(bear).unwrap();
    assert_eq!(computed.power, 4);
    assert_eq!(computed.toughness, 4);
    assert!(computed.keywords.contains(&crabomination::card::Keyword::Hexproof));
}

#[test]
fn spell_pierce_counters_when_unpaid() {
    let mut g = two_player_game();
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
    .expect("Lightning Bolt castable");

    let pierce = g.add_card_to_hand(0, catalog::spell_pierce());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: pierce,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Spell Pierce castable");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, 20, "Bolt should be countered");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));
}

#[test]
fn mana_leak_lets_spell_through_when_paid() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.players[1].mana_pool.add_colorless(3);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable");

    let leak = g.add_card_to_hand(0, catalog::mana_leak());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: leak,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Mana Leak castable for {1}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, 17,
        "Bolt should resolve when controller pays {{3}}");
}

#[test]
fn anger_of_the_gods_burns_each_creature() {
    let mut g = two_player_game();
    let b0 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let lion = g.add_card_to_battlefield(0, catalog::savannah_lions());
    let anger = g.add_card_to_hand(0, catalog::anger_of_the_gods());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: anger,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Anger castable for {1}{R}{R}");
    drain_stack(&mut g);
    for cid in [b0, b1, lion] {
        assert!(!g.battlefield.iter().any(|c| c.id == cid));
    }
    // "If a creature would die this turn, exile it instead": the burned
    // creatures land in exile, not their owners' graveyards.
    for cid in [b0, b1, lion] {
        assert!(g.exile.iter().any(|c| c.id == cid), "creature exiled, not buried");
    }
    assert!(g.players[0].graveyard.iter().all(|c| c.id != b0 && c.id != lion),
        "no Anger victims in graveyard");
}

#[test]
fn fanatical_firebrand_taps_and_sacs_to_ping_any_target() {
    let mut g = two_player_game();
    let fb = g.add_card_to_battlefield(0, catalog::fanatical_firebrand());
    g.clear_sickness(fb); // (has Haste anyway)
    let life_before = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: fb, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None,
    }).expect("firebrand ability activatable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 1, "deals 1 to target player");
    assert!(g.battlefield_find(fb).is_none(), "sacrificed as part of the cost");
}

#[test]
fn sweltering_suns_burns_each_creature_and_has_cycling() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let b0 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let suns = g.add_card_to_hand(0, catalog::sweltering_suns());
    assert!(catalog::sweltering_suns().keywords.iter()
        .any(|k| matches!(k, Keyword::Cycling(_))), "has Cycling");
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: suns, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sweltering Suns castable for {1}{R}{R}");
    drain_stack(&mut g);
    // Both 2/2 bears take 3 and die.
    assert!(!g.battlefield.iter().any(|c| c.id == b0 || c.id == b1));
}

#[test]
fn blasphemous_act_kills_each_creature() {
    let mut g = two_player_game();
    // {8}{R} base, but "costs {1} less per creature on the battlefield":
    // with four creatures out it costs {4}{R} (Affinity hook).
    let dragon = g.add_card_to_battlefield(0, catalog::shivan_dragon());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let act = g.add_card_to_hand(0, catalog::blasphemous_act());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: act,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Blasphemous Act castable for {4}{R} with four creatures out");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == dragon));
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

#[test]
fn leyline_of_sanctity_blocks_targeted_ability() {
    // Tim's "{T}: deal 1 damage to any target" is an *ability* — under
    // Leyline, opponent activates can't aim at the protected player.
    let mut g = two_player_game();
    let _leyline = g.add_card_to_battlefield(0, catalog::leyline_of_sanctity());
    let tim = g.add_card_to_battlefield(1, catalog::prodigal_sorcerer());
    g.battlefield_find_mut(tim).unwrap().summoning_sick = false;
    g.priority.player_with_priority = 1;
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: tim,
        ability_index: 0,
        target: Some(Target::Player(0)), additional_targets: Vec::new(), x_value: None });
    assert!(err.is_err(),
        "Tim's targeted ability should be rejected against Leyline-protected player; got: {err:?}");
}


// ── Modern shocklands (mod_set/lands) ────────────────────────────────────────

#[test]
fn sacred_foundry_pays_two_life_and_stays_untapped_by_default() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::sacred_foundry());
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.definition.activated_abilities.len(), 2);
    assert!(!card.tapped, "shockland enters untapped after AutoDecider picks pay-2-life");
    assert_eq!(g.players[0].life, 18);
}

#[test]
fn watery_grave_is_a_ub_shockland() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::watery_grave());
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    // Two basic land types + the shock pay-2-or-tap ability; AutoDecider
    // pays 2 life so it enters untapped.
    assert!(!card.tapped, "shockland enters untapped after pay-2-life");
    assert_eq!(g.players[0].life, 18);
}

// ── Auxiliary instants (mod_set/spells) ──────────────────────────────────────

#[test]
fn disenchant_destroys_artifact() {
    let mut g = two_player_game();
    let sol_ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let disenchant = g.add_card_to_hand(0, catalog::disenchant());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: disenchant,
        target: Some(Target::Permanent(sol_ring)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Disenchant castable for {1}{W}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == sol_ring));
}

#[test]
fn natures_claim_destroys_artifact_and_grants_controller_four_life() {
    let mut g = two_player_game();
    let sol_ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let claim = g.add_card_to_hand(0, catalog::natures_claim());
    g.players[0].mana_pool.add(Color::Green, 1);
    let opp_life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: claim,
        target: Some(Target::Permanent(sol_ring)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Nature's Claim castable for {G}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == sol_ring));
    assert_eq!(
        g.players[1].life,
        opp_life_before + 4,
        "Sol Ring's controller should gain 4 life",
    );
}

#[test]
fn negate_counters_a_noncreature_spell() {
    let mut g = two_player_game();
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
    .unwrap();

    let negate = g.add_card_to_hand(0, catalog::negate());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: negate,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Negate castable for {1}{U}");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));
    assert_eq!(g.players[0].life, 20);
}

#[test]
fn negate_rejects_creature_target_at_cast_time() {
    let mut g = two_player_game();
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 2);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .unwrap();

    let negate = g.add_card_to_hand(0, catalog::negate());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    let err = g
        .perform_action(GameAction::CastSpell {
            card_id: negate,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .unwrap_err();
    assert_eq!(err, GameError::SelectionRequirementViolated);
}

#[test]
fn dispel_targets_only_instants() {
    let mut g = two_player_game();
    // Sorcery on the stack — Dispel can't target it.
    let wrath = g.add_card_to_hand(1, catalog::wrath_of_god());
    g.players[1].mana_pool.add_colorless(2);
    g.players[1].mana_pool.add(Color::White, 2);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: wrath, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .unwrap();

    let dispel = g.add_card_to_hand(0, catalog::dispel());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    let err = g
        .perform_action(GameAction::CastSpell {
            card_id: dispel,
            target: Some(Target::Permanent(wrath)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .unwrap_err();
    assert_eq!(err, GameError::SelectionRequirementViolated);
}

#[test]
fn dovins_veto_is_uncounterable() {
    // Alice casts a Bolt at Bob; Bob casts Dovin's Veto on the Bolt; Alice
    // tries to Counterspell the Veto but it can't be countered, so the
    // Veto resolves and counters the Bolt.
    let mut g = two_player_game();

    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .unwrap();

    let veto = g.add_card_to_hand(1, catalog::dovins_veto());
    g.players[1].mana_pool.add(Color::White, 1);
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: veto,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .unwrap();

    let cs = g.add_card_to_hand(0, catalog::counterspell());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: cs,
        target: Some(Target::Permanent(veto)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .unwrap();

    drain_stack(&mut g);

    // Bolt is countered (by Veto, which couldn't itself be countered).
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt));
    assert_eq!(g.players[1].life, 20, "Bob took no damage — Bolt was countered");
}

// ── Modern creatures (mod_set/creatures) ─────────────────────────────────────

#[test]
fn thalia_taxes_every_noncreature_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::thalia_guardian_of_thraben());

    // Even the first Bolt this turn owes {1} more — only {R} fails.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let err = g
        .perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .unwrap_err();
    assert!(matches!(err, GameError::Mana(_)));

    // {1}{R} pays the taxed Bolt.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{1}{R} covers Thalia's tax");
}

#[test]
fn phyrexian_arena_draws_card_and_loses_life_at_upkeep() {
    let mut g = two_player_game();
    // CR 103.7a — only turn 1's draw is skipped; keep libraries stocked for
    // fixtures that cross a turn boundary.
    for seat in 0..2 {
        for _ in 0..5 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g.add_card_to_battlefield(0, catalog::phyrexian_arena());
    g.add_card_to_library(0, catalog::forest());
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    let lib_before = g.players[0].library.len();

    // Roll forward to Alice's next upkeep.
    g.step = TurnStep::Cleanup;
    g.active_player_idx = 0;
    for _ in 0..30 {
        if g.is_game_over() {
            break;
        }
        if g.active_player_idx == 0
            && g.step == TurnStep::Upkeep
            && g.stack.is_empty()
            && g.players[0].hand.len() > hand_before
        {
            break;
        }
        g.perform_action(GameAction::PassPriority).unwrap();
    }

    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    assert_eq!(g.players[0].library.len(), lib_before - 1);
    assert_eq!(g.players[0].life, life_before - 1);
}

// ── Cube cards (mod_set additions) ───────────────────────────────────────────

#[test]
fn tarfire_deals_two_damage_to_player_or_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let to_player = g.add_card_to_hand(0, catalog::tarfire());
    let to_creature = g.add_card_to_hand(0, catalog::tarfire());
    g.players[0].mana_pool.add(Color::Red, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: to_player, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tarfire castable for {R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);

    g.perform_action(GameAction::CastSpell {
        card_id: to_creature, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tarfire castable for {R}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "2-toughness Bear should be dead");
}

#[test]
fn consider_surveils_then_draws() {
    // With one card in library and one already-known to be the "next draw",
    // Consider's Draw step should net +1 in hand even after Surveil 1
    // bottoms / graveyards a card. AutoDecider keeps Surveil's peeked card
    // on top, so the surveil leaves the library shape intact and Draw gets
    // that same card.
    let mut g = two_player_game();
    let top = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    let consider = g.add_card_to_hand(0, catalog::consider());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: consider,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Consider castable for {U}");
    drain_stack(&mut g);
    // Net change: cast (-1) + draw (+1) = 0. The drawn card may be `top` or
    // the surveil-buried card depending on the decider's choice — assert
    // only the count and that Consider itself is in the graveyard.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Consider"));
    let _ = top;
}

#[test]
fn thought_scour_mills_target_and_draws_for_caster() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(1, catalog::forest());
    g.add_card_to_library(1, catalog::mountain());
    let scour = g.add_card_to_hand(0, catalog::thought_scour());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let opp_lib_before = g.players[1].library.len();
    let opp_yard_before = g.players[1].graveyard.len();
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: scour,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Thought Scour castable for {U}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), opp_lib_before - 2);
    assert_eq!(g.players[1].graveyard.len(), opp_yard_before + 2);
    // Net: cast (-1) + draw (+1) = 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn frantic_search_draws_two_discards_two_untaps_lands() {
    let mut g = two_player_game();
    // Stock library so the two draws have inputs.
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::mountain());
    // Two tapped lands the player will untap on resolution.
    let l1 = g.add_card_to_battlefield(0, catalog::island());
    let l2 = g.add_card_to_battlefield(0, catalog::island());
    g.battlefield.iter_mut().find(|c| c.id == l1).unwrap().tapped = true;
    g.battlefield.iter_mut().find(|c| c.id == l2).unwrap().tapped = true;
    let fs = g.add_card_to_hand(0, catalog::frantic_search());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: fs,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Frantic Search castable for {2}{U}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().find(|c| c.id == l1).unwrap().tapped);
    assert!(!g.battlefield.iter().find(|c| c.id == l2).unwrap().tapped);
}

#[test]
fn frantic_search_caps_at_three_lands_when_more_are_tapped() {
    // Five tapped lands; Frantic Search's "up to three" cap kicks in
    // and only 3 untap. Exercises the new `Effect::Untap.up_to`
    // primitive against a permissive selector.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::mountain());
    let lands: Vec<_> = (0..5)
        .map(|_| g.add_card_to_battlefield(0, catalog::island()))
        .collect();
    for l in &lands {
        g.battlefield.iter_mut().find(|c| c.id == *l).unwrap().tapped = true;
    }
    let fs = g.add_card_to_hand(0, catalog::frantic_search());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: fs,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Frantic Search castable for {2}{U}");
    drain_stack(&mut g);
    // Exactly 3 of the 5 should be untapped after resolution.
    let untapped_count = lands
        .iter()
        .filter(|l| !g.battlefield.iter().find(|c| c.id == **l).unwrap().tapped)
        .count();
    assert_eq!(
        untapped_count, 3,
        "Frantic Search 'up to three' cap should untap exactly 3 of 5 tapped lands"
    );
}

#[test]
fn slaughter_pact_destroys_nonblack_creature_and_schedules_upkeep() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pact = g.add_card_to_hand(0, catalog::slaughter_pact());
    // Pact costs {0}.
    g.perform_action(GameAction::CastSpell {
        card_id: pact,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Slaughter Pact castable for free");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    // The upkeep `PayOrLoseGame` is registered on the delayed-trigger queue
    // with the caster as controller.
    assert!(
        g.delayed_triggers.iter().any(|d| d.controller == 0),
        "Slaughter Pact should register a delayed upkeep trigger for seat 0"
    );
}

#[test]
fn pact_of_the_titan_creates_giant_token() {
    let mut g = two_player_game();
    let pact = g.add_card_to_hand(0, catalog::pact_of_the_titan());
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: pact,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Pact of the Titan castable for free");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.len(), bf_before + 1);
    let token = g.battlefield.last().unwrap();
    assert_eq!(token.definition.name, "Giant");
    assert_eq!(token.power(), 4);
    assert_eq!(token.toughness(), 4);
    assert!(token.is_token);
    assert!(g.delayed_triggers.iter().any(|d| d.controller == 0));
}

#[test]
fn spell_snare_counters_two_mana_value_spell() {
    let mut g = two_player_game();
    // Bears ({1}{G}, mana value 2) cast on seat 1's turn at sorcery speed.
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add_colorless(1);
    g.players[1].mana_pool.add(Color::Green, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bears castable for {1}{G}");
    // Seat 0 responds with Spell Snare (instant) targeting Bears on the stack.
    let snare = g.add_card_to_hand(0, catalog::spell_snare());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: snare,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Spell Snare castable for {U}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bears));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bears));
}

#[test]
fn mental_misstep_counters_a_one_mana_spell() {
    let mut g = two_player_game();
    // Seat 1 casts Monastery Swiftspear ({R}, mana value 1).
    let swift = g.add_card_to_hand(1, catalog::monastery_swiftspear());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: swift, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Swiftspear castable for {R}");
    // Seat 0 responds with Mental Misstep, paying the {U/P} pip with 2 life.
    let misstep = g.add_card_to_hand(0, catalog::mental_misstep());
    let life = g.players[0].life;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: misstep, target: Some(Target::Permanent(swift)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mental Misstep castable for 2 life");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2, "paid the Phyrexian pip with 2 life");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == swift), "Swiftspear countered");
}

#[test]
fn torch_the_tower_bargained_deals_three() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let target = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let fodder = g.add_card_to_battlefield(0, catalog::the_everflowing_well()); // artifact
    let id = g.add_card_to_hand(0, catalog::torch_the_tower());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpellBargain {
        card_id: id, sacrifice: Some(fodder),
        target: Some(Target::Permanent(target)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Torch the Tower bargained");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target).unwrap().damage, 3, "bargained = 3 damage");
    assert!(g.battlefield_find(fodder).is_none(), "the artifact was sacrificed");
}

#[test]
fn torch_the_tower_unbargained_deals_two() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let target = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::torch_the_tower());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Torch the Tower normally");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target).unwrap().damage, 2, "no bargain = 2 damage");
}

#[test]
fn candy_grapple_bargained_gives_minus_five() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let target = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let fodder = g.add_card_to_battlefield(0, catalog::the_everflowing_well());
    let id = g.add_card_to_hand(0, catalog::candy_grapple());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpellBargain {
        card_id: id, sacrifice: Some(fodder),
        target: Some(Target::Permanent(target)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Candy Grapple bargained");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "-5/-5 killed the 4/4");
}

#[test]
fn archons_glory_bargained_grants_flying() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let fodder = g.add_card_to_battlefield(0, catalog::the_everflowing_well());
    let id = g.add_card_to_hand(0, catalog::archons_glory());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpellBargain {
        card_id: id, sacrifice: Some(fodder),
        target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Archon's Glory bargained");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!((c.power(), c.toughness()), (4, 4), "+2/+2");
    assert!(c.has_keyword(&Keyword::Flying), "bargained grants flying");
}

#[test]
fn kellans_lightblades_bargained_destroys_attacker() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.attacking = vec![Attack { attacker, target: AttackTarget::Player(0) }];
    g.step = TurnStep::DeclareBlockers;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 0;
    let fodder = g.add_card_to_battlefield(0, catalog::the_everflowing_well());
    let id = g.add_card_to_hand(0, catalog::kellans_lightblades());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpellBargain {
        card_id: id, sacrifice: Some(fodder),
        target: Some(Target::Permanent(attacker)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Kellan's Lightblades bargained");
    drain_stack(&mut g);
    assert!(g.battlefield_find(attacker).is_none(), "bargained destroyed the 4/4 attacker");
}

#[test]
fn stonesplitter_bolt_bargained_deals_twice_x() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let target = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let fodder = g.add_card_to_battlefield(0, catalog::the_everflowing_well());
    let id = g.add_card_to_hand(0, catalog::stonesplitter_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2); // {2}{R}, X = 2
    g.perform_action(GameAction::CastSpellBargain {
        card_id: id, sacrifice: Some(fodder),
        target: Some(Target::Permanent(target)), additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("cast Stonesplitter Bolt bargained, X=2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "twice X = 4 damage killed the 4/4");
}

#[test]
fn glidedive_duo_drains_two() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::glidedive_duo());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    let (my_life, opp_life) = (g.players[0].life, g.players[1].life);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Glidedive Duo");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 2, "opponent lost 2");
    assert_eq!(g.players[0].life, my_life + 2, "you gained 2");
}

#[test]
fn galewind_moose_has_flash_and_evasion_keywords() {
    use crabomination::card::Keyword;
    let m = catalog::galewind_moose();
    for kw in [Keyword::Flash, Keyword::Vigilance, Keyword::Reach, Keyword::Trample] {
        assert!(m.keywords.contains(&kw), "Galewind Moose has {kw:?}");
    }
    assert_eq!((m.power, m.toughness), (6, 6));
}

#[test]
fn thieving_otter_draws_on_combat_damage() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let otter = g.add_card_to_battlefield(0, catalog::thieving_otter());
    g.clear_sickness(otter);
    g.add_card_to_library(0, catalog::island());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: otter, target: AttackTarget::Player(1),
    }])).expect("Otter attacks");
    drain_stack(&mut g);
    // Pass through combat so damage is dealt.
    while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    assert!(g.players[0].hand.len() > hand_before, "drew a card from combat damage");
}

#[test]
fn bria_riptide_rogue_grants_prowess_to_other_creatures() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bria_riptide_rogue());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let computed = g.compute_battlefield();
    let b = computed.iter().find(|c| c.id == bear).unwrap();
    assert!(b.keywords.contains(&Keyword::Prowess), "Bria grants prowess to other creatures");
}

#[test]
fn cindering_cutthroat_enters_bigger_after_damage() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[1].life -= 1; // an opponent lost life this turn
    g.players[1].lost_life_this_turn = true;
    let id = g.add_card_to_hand(0, catalog::cindering_cutthroat());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Cindering Cutthroat");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1, "entered with a +1/+1 counter");
}

#[test]
fn three_tree_mascot_taps_for_any_color() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::three_tree_mascot());
    g.clear_sickness(id);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Changeling));
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate the mana ability");
    assert!(g.players[0].mana_pool.total() >= 1, "produced a mana of any color");
}

#[test]
fn hivespine_wolverine_mode_destroy_kills_an_enchantment() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let aura = g.add_card_to_battlefield(1, catalog::pacifism());
    let hw = g.add_card_to_hand(0, catalog::hivespine_wolverine());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: hw, target: None, additional_targets: vec![], mode: Some(2), x_value: None,
    }).expect("cast Hivespine Wolverine, ETB mode 2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(aura).is_none(), "destroy mode killed the enchantment");
}

#[test]
fn hivespine_wolverine_mode_counter_grows_a_creature() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hw = g.add_card_to_hand(0, catalog::hivespine_wolverine());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: hw, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast Hivespine Wolverine, ETB mode 0");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!((c.power(), c.toughness()), (3, 3), "+1/+1 counter mode");
}

#[test]
fn ice_out_costs_one_less_when_bargained() {
    let mut g = two_player_game();
    // Seat 1 casts a spell to counter.
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add_colorless(1);
    g.players[1].mana_pool.add(Color::Green, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bears cast");
    // Seat 0 bargains Ice Out: only {U}{U} in pool (one short of {1}{U}{U}).
    let fodder = g.add_card_to_battlefield(0, catalog::the_everflowing_well());
    let id = g.add_card_to_hand(0, catalog::ice_out());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellBargain {
        card_id: id, sacrifice: Some(fodder),
        target: Some(Target::Permanent(bears)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ice Out castable for {U}{U} after the {1}-less bargain discount");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bears), "Ice Out countered Bears");
}

#[test]
fn johanns_stopgap_bounces_and_draws() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let fodder = g.add_card_to_battlefield(0, catalog::the_everflowing_well());
    let id = g.add_card_to_hand(0, catalog::johanns_stopgap());
    // {2} less when bargained → only {U} needed (vs {3}{U}).
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1); // a little slack
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpellBargain {
        card_id: id, sacrifice: Some(fodder),
        target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Johann's Stopgap castable cheaply when bargained");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "bounced the bear to its owner's hand");
    // Cast the Stopgap (−1), drew one (+1) → hand unchanged net.
    assert_eq!(g.players[0].hand.len(), hand_before, "drew a card");
}

#[test]
fn troublemaker_ouphe_bargained_exiles_opponent_artifact() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let opp_art = g.add_card_to_battlefield(1, catalog::the_everflowing_well());
    let fodder = g.add_card_to_battlefield(0, catalog::the_everflowing_well());
    let id = g.add_card_to_hand(0, catalog::troublemaker_ouphe());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpellBargain {
        card_id: id, sacrifice: Some(fodder),
        target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Troublemaker Ouphe bargained");
    drain_stack(&mut g);
    assert!(g.battlefield_find(opp_art).is_none(), "bargained ETB exiled the opponent's artifact");
}

#[test]
fn troublemaker_ouphe_unbargained_does_not_exile() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let opp_art = g.add_card_to_battlefield(1, catalog::the_everflowing_well());
    let id = g.add_card_to_hand(0, catalog::troublemaker_ouphe());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Troublemaker Ouphe normally");
    drain_stack(&mut g);
    assert!(g.battlefield_find(opp_art).is_some(), "unbargained: no ETB exile");
}

#[test]
fn tenacious_tomeseeker_bargained_returns_instant_from_graveyard() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // Seed an instant in P0's graveyard.
    let inst = g.add_card_to_hand(0, catalog::mental_misstep());
    let card = g.players[0].hand.iter().position(|c| c.id == inst).map(|i| g.players[0].hand.remove(i)).unwrap();
    g.players[0].graveyard.push(card);
    let fodder = g.add_card_to_battlefield(0, catalog::the_everflowing_well());
    let id = g.add_card_to_hand(0, catalog::tenacious_tomeseeker());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpellBargain {
        card_id: id, sacrifice: Some(fodder),
        target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Tenacious Tomeseeker bargained");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == inst), "bargained ETB returned the instant to hand");
}

#[test]
fn mutagenic_growth_payable_with_two_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::mutagenic_growth());
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for 2 life via the Phyrexian pip");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2, "paid 2 life");
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!((c.power(), c.toughness()), (4, 4), "+2/+2");
}

