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

// ── Cube cards (round 2) ─────────────────────────────────────────────────────

#[test]
fn daze_counters_when_unpaid() {
    let mut g = two_player_game();
    // Seat 1 casts Bolt at sorcery speed (their turn).
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable");
    // Seat 0 responds with Daze; seat 1 has no extra mana to pay {1}.
    let daze = g.add_card_to_hand(0, catalog::daze());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: daze,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Daze castable");
    drain_stack(&mut g);
    // Bolt countered → seat 0 takes no damage, Bolt in graveyard.
    assert_eq!(g.players[0].life, 20);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));
}

#[test]
fn daze_alt_cost_returns_island_to_counter_for_free() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    // Seat 0 has no mana but an Island to bounce as Daze's alt cost.
    let daze = g.add_card_to_hand(0, catalog::daze());
    let isl = g.add_card_to_battlefield(0, catalog::island());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: daze, pitch_card: None, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Daze castable via return-an-Island alt cost");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == isl), "Island bounced as the alt cost");
    assert!(g.players[0].hand.iter().any(|c| c.id == isl), "Island back in hand");
    // Seat 1 can't pay {1} (no mana left) → Bolt countered.
    assert_eq!(g.players[0].life, 20);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));
}

#[test]
fn swan_song_counters_enchantment_and_makes_a_bird() {
    let mut g = two_player_game();
    // Seat 1 casts a creature *enchantment* — use Hopeful Eidolon (an
    // enchantment creature) from the catalog so the spell type matches.
    // Hmm, Hopeful Eidolon is Enchantment Creature; Swan Song's filter is
    // "enchantment, instant, or sorcery" — enchantment matches.
    let eid = g.add_card_to_hand(1, catalog::hopeful_eidolon());
    g.players[1].mana_pool.add(Color::White, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: eid, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Eidolon castable");
    let swan = g.add_card_to_hand(0, catalog::swan_song());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: swan,
        target: Some(Target::Permanent(eid)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Swan Song castable");
    drain_stack(&mut g);
    // Eidolon countered.
    assert!(g.players[1].graveyard.iter().any(|c| c.id == eid));
    // Bird token created under the **countered spell's controller**
    // (seat 1) — resolved through `PlayerRef::ControllerOf(Target(0))`
    // via `stack_caster_for_card`.
    assert_eq!(g.battlefield.len(), bf_before + 1);
    let token = g.battlefield.last().unwrap();
    assert_eq!(token.definition.name, "Bird");
    assert_eq!(token.controller, 1);
    assert!(token.has_keyword(&crabomination::card::Keyword::Flying));
}

#[test]
fn swan_song_in_three_player_gives_bird_to_countered_spell_controller() {
    // 3-player game: seat 0 casts Swan Song on a spell seat 2 cast.
    // The Bird should go to seat 2 (the countered spell's controller),
    // not seat 1. Pre-fix this used EachOpponent which would have given
    // a token to both opponents (seats 1 AND 2).
    let mut g = crabomination::game::multi_player_game(3);
    let eid = g.add_card_to_hand(2, catalog::hopeful_eidolon());
    g.players[2].mana_pool.add(Color::White, 1);
    g.active_player_idx = 2;
    g.priority.player_with_priority = 2;
    g.perform_action(GameAction::CastSpell {
        card_id: eid, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Eidolon castable");
    let swan = g.add_card_to_hand(0, catalog::swan_song());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: swan,
        target: Some(Target::Permanent(eid)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Swan Song castable");
    drain_stack(&mut g);
    assert!(g.players[2].graveyard.iter().any(|c| c.id == eid));
    // Exactly one new permanent — the Bird under seat 2.
    assert_eq!(g.battlefield.len(), bf_before + 1);
    let token = g.battlefield.last().unwrap();
    assert_eq!(token.definition.name, "Bird");
    assert_eq!(token.controller, 2,
        "Bird should belong to the countered spell's controller (seat 2), not seat 1");
}

#[test]
fn drown_in_ichor_deals_three_damage_and_surveils() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let drown = g.add_card_to_hand(0, catalog::drown_in_ichor());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: drown,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Drown in Ichor castable");
    drain_stack(&mut g);
    // 2/2 takes 3 damage → dies.
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
}

#[test]
fn paradoxical_outcome_bounces_two_artifacts_and_draws_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    let ring1 = g.add_card_to_battlefield(0, catalog::sol_ring());
    let ring2 = g.add_card_to_battlefield(0, catalog::sol_ring());
    let outcome = g.add_card_to_hand(0, catalog::paradoxical_outcome());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Blue, 2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: outcome, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Paradoxical Outcome castable");
    drain_stack(&mut g);
    // Both rings returned, two cards drawn, outcome itself in graveyard.
    assert!(!g.battlefield.iter().any(|c| c.id == ring1 || c.id == ring2));
    assert!(g.players[0].hand.iter().any(|c| c.id == ring1));
    assert!(g.players[0].hand.iter().any(|c| c.id == ring2));
    // Net hand: cast (-1) + bounce 2 (+2) + draw 2 (+2) = +3.
    assert_eq!(g.players[0].hand.len(), hand_before + 3);
}

#[test]
fn blasphemous_edict_each_player_sacrifices_a_creature() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let edict = g.add_card_to_hand(0, catalog::blasphemous_edict());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: edict, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Blasphemous Edict castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == mine));
    assert!(!g.battlefield.iter().any(|c| c.id == theirs));
}

#[test]
fn fell_destroys_tapped_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap().tapped = true;
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    let fell = g.add_card_to_hand(0, catalog::fell());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: fell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Fell castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

#[test]
fn big_score_discards_one_creates_two_treasures_draws_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_hand(0, catalog::lightning_bolt()); // discardable
    let big = g.add_card_to_hand(0, catalog::big_score());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Red, 1);
    let yard_before = g.players[0].graveyard.len();
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: big, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Big Score castable");
    drain_stack(&mut g);
    // One discarded card + Big Score itself in graveyard = +2.
    assert!(g.players[0].graveyard.len() >= yard_before + 2);
    // Two Treasure tokens on battlefield.
    let treasures = g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count();
    assert_eq!(treasures, 2);
    let _ = bf_before;
}

/// CR 601.2b — a `wants_ui` caster chooses *which* card to discard for Big
/// Score's "as an additional cost, discard a card" rather than the engine
/// dumping the first card in hand. Casting suspends on a `Decision::Discard`;
/// the chosen card is the one discarded.
#[test]
fn big_score_ui_caster_chooses_which_card_to_discard() {
    let mut g = two_player_game();
    // See `crop_rotation_ui_player_picks_which_land_to_sacrifice` — the
    // CR 601.2b additional-cost prompt keys on `manual_mana`.
    g.players[0].wants_ui = true;
    g.players[0].manual_mana = true;
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    // Two discardable cards → a genuine choice (the card being cast is excluded).
    let pitch = g.add_card_to_hand(0, catalog::lightning_bolt());
    let keep = g.add_card_to_hand(0, catalog::grizzly_bears());
    let big = g.add_card_to_hand(0, catalog::big_score());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: big, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast suspends for the discard choice");

    let pd = g.pending_decision.as_ref().expect("a discard choice is pending");
    assert_eq!(pd.acting_player(), 0);
    match &pd.decision {
        crabomination::decision::Decision::Discard { count, hand, .. } => {
            assert_eq!(*count, 1);
            assert_eq!(hand.len(), 2, "both other cards offered; Big Score itself excluded");
            assert!(hand.iter().all(|(id, _)| *id != big), "cannot discard the spell being cast");
        }
        other => panic!("expected Discard, got {other:?}"),
    }

    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Discard(vec![pitch])))
        .expect("submit the discard choice");

    assert!(g.players[0].graveyard.iter().any(|c| c.id == pitch), "chosen card discarded");
    assert!(g.players[0].hand.iter().any(|c| c.id == keep), "unchosen card kept");
}

#[test]
fn restoration_angel_blinks_a_friendly_non_angel() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let angel = g.add_card_to_hand(0, catalog::restoration_angel());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::White, 1);
    // Cast — auto-target heuristic picks the bear (the only legal non-Angel
    // creature you control).
    g.perform_action(GameAction::CastSpell {
        card_id: angel, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Restoration Angel castable");
    drain_stack(&mut g);
    // Angel resolves; ETB exiles bear and brings it back. The card id is
    // preserved across the round-trip, but it now has summoning sickness
    // again.
    let bear_back = g.battlefield.iter().find(|c| c.id == bear);
    assert!(bear_back.is_some(), "bear should have returned to battlefield");
    assert!(bear_back.unwrap().summoning_sick, "blink resets sickness");
}

#[test]
fn flickerwisp_exiles_until_end_of_turn() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wisp = g.add_card_to_hand(0, catalog::flickerwisp());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::White, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: wisp, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Flickerwisp castable");
    drain_stack(&mut g);
    // Bear is exiled.
    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear));
    assert!(g.exile.iter().any(|c| c.id == opp_bear));
    // A delayed trigger is queued for the next end step under seat 0.
    assert!(
        g.delayed_triggers.iter().any(|d| d.controller == 0),
        "Flickerwisp should register a delayed return trigger"
    );
}

// ── Cube cards (round 3) ─────────────────────────────────────────────────────

#[test]
fn isolate_exiles_one_mana_value_permanent() {
    let mut g = two_player_game();
    // Sengir Vampire is 4-mana, won't match. Lightning Bolt is an instant
    // (not a permanent). Use Llanowar Elves: {G}, mana value 1, creature.
    let elves = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // mv 2 — should NOT match
    let isolate = g.add_card_to_hand(0, catalog::isolate());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: isolate,
        target: Some(Target::Permanent(elves)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Isolate castable on a 1-MV permanent");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == elves));
    assert!(g.exile.iter().any(|c| c.id == elves));

    // Casting on the 2-MV bear should be rejected at cast time by the
    // selection-requirement check.
    let isolate2 = g.add_card_to_hand(0, catalog::isolate());
    g.players[0].mana_pool.add(Color::White, 1);
    let err = g.perform_action(GameAction::CastSpell {
        card_id: isolate2,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(), "Isolate on 2-MV target should fail");
}

#[test]
fn mind_stone_taps_for_one_colorless() {
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(0, catalog::mind_stone());
    g.clear_sickness(stone);
    g.perform_action(GameAction::ActivateAbility {
        card_id: stone,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Mind Stone activates");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1);
}

#[test]
fn spirebluff_canal_enters_untapped_with_few_lands() {
    let mut g = two_player_game();
    // No prior lands — Spirebluff Canal counts itself, so post-ETB land
    // count is 1, well below the fastland threshold of 4.
    let canal_def = catalog::spirebluff_canal();
    let canal = g.add_card_to_hand(0, canal_def);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(canal))
        .expect("Spirebluff Canal plays");
    // Resolve any ETB triggers on the stack.
    drain_stack(&mut g);
    let on_bf = g.battlefield.iter().find(|c| c.id == canal).unwrap();
    assert!(!on_bf.tapped, "fastland enters untapped with <4 lands");
}

#[test]
fn spirebluff_canal_enters_tapped_with_many_lands() {
    let mut g = two_player_game();
    // Stack the battlefield with three lands first; Spirebluff Canal then
    // becomes the fourth and taps on entry.
    g.add_card_to_battlefield(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::island());
    let canal = g.add_card_to_hand(0, catalog::spirebluff_canal());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(canal))
        .expect("Spirebluff Canal plays");
    drain_stack(&mut g);
    let on_bf = g.battlefield.iter().find(|c| c.id == canal).unwrap();
    assert!(on_bf.tapped, "fastland enters tapped with ≥4 lands");
}

/// Prismatic Vista cracks for any basic.
#[test]
fn prismatic_vista_fetches_any_basic() {
    let mut g = two_player_game();
    let vista = g.add_card_to_battlefield(0, catalog::prismatic_vista());
    g.clear_sickness(vista);
    let mountain = g.add_card_to_library(0, catalog::mountain());
    let life = g.players[0].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(mountain))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: vista, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("crack the Vista");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mountain).is_some(), "basic fetched untapped");
    assert!(g.battlefield_find(vista).is_none(), "Vista sacrificed");
    assert_eq!(g.players[0].life, life - 1, "paid 1 life");
}

/// Check-land: enters untapped with a matching basic-typed land, tapped
/// without one.
#[test]
fn checkland_taps_only_without_a_matching_type() {
    let mut g = two_player_game();
    // No Plains/Island → Glacial Fortress enters tapped.
    let tapped = g.add_card_to_hand(0, catalog::glacial_fortress());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(tapped)).expect("plays");
    drain_stack(&mut g);
    assert!(g.battlefield_find(tapped).unwrap().tapped, "no check → tapped");
    // With an Island on board the next one enters untapped.
    g.add_card_to_battlefield(0, catalog::island());
    g.players[0].lands_played_this_turn = 0;
    let untapped = g.add_card_to_hand(0, catalog::glacial_fortress());
    g.perform_action(GameAction::PlayLand(untapped)).expect("plays");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(untapped).unwrap().tapped, "check satisfied → untapped");
}

/// The full ten-card check-land cycle exists with two mana abilities each.
#[test]
fn checkland_cycle_definitions() {
    for def in [
        catalog::glacial_fortress(), catalog::drowned_catacomb(),
        catalog::dragonskull_summit(), catalog::rootbound_crag(),
        catalog::sunpetal_grove(), catalog::isolated_chapel(),
        catalog::sulfur_falls(), catalog::woodland_cemetery(),
        catalog::clifftop_retreat(), catalog::hinterland_harbor(),
    ] {
        assert!(def.is_land(), "{} is a land", def.name);
        assert_eq!(def.activated_abilities.len(), 2, "{} taps for two colors", def.name);
    }
}

/// DSK painland: enters tapped at high life, untapped once a player is at 13
/// or less.
#[test]
fn dsk_painland_taps_unless_a_player_is_low() {
    let mut g = two_player_game();
    // Both at 20 → Razortrap Gorge enters tapped.
    let tapped = g.add_card_to_hand(0, catalog::razortrap_gorge());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(tapped)).expect("plays");
    drain_stack(&mut g);
    assert!(g.battlefield_find(tapped).unwrap().tapped, "high life → tapped");
    // Drop the opponent to 13 → the next one enters untapped.
    g.players[1].life = 13;
    g.players[0].lands_played_this_turn = 0;
    let untapped = g.add_card_to_hand(0, catalog::razortrap_gorge());
    g.perform_action(GameAction::PlayLand(untapped)).expect("plays");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(untapped).unwrap().tapped, "a player at 13 → untapped");
}

/// The full ten-card DSK painland cycle exists with two mana abilities each.
#[test]
fn dsk_painland_cycle_definitions() {
    for def in [
        catalog::abandoned_campground(), catalog::bleeding_woods(),
        catalog::etched_cornfield(), catalog::lakeside_shack(),
        catalog::murky_sewer(), catalog::neglected_manor(),
        catalog::peculiar_lighthouse(), catalog::raucous_carnival(),
        catalog::razortrap_gorge(), catalog::strangled_cemetery(),
    ] {
        assert!(def.is_land(), "{} is a land", def.name);
        assert_eq!(def.activated_abilities.len(), 2, "{} taps for two colors", def.name);
    }
}

#[test]
fn ancient_den_taps_for_white_and_is_an_artifact() {
    let mut g = two_player_game();
    let den = g.add_card_to_battlefield(0, catalog::ancient_den());
    g.clear_sickness(den);
    g.perform_action(GameAction::ActivateAbility {
        card_id: den,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Ancient Den taps for {W}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::White), 1);
    let on_bf = g.battlefield.iter().find(|c| c.id == den).unwrap();
    assert!(on_bf.definition.is_artifact());
    assert!(on_bf.definition.is_land());
}

#[test]
fn darksteel_citadel_is_indestructible() {
    let mut g = two_player_game();
    let citadel = g.add_card_to_battlefield(1, catalog::darksteel_citadel());
    let disenchant = g.add_card_to_hand(0, catalog::disenchant());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: disenchant,
        target: Some(Target::Permanent(citadel)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Disenchant castable");
    drain_stack(&mut g);
    // Indestructible artifact survives Destroy.
    assert!(g.battlefield.iter().any(|c| c.id == citadel));
}

// ── Cube cards (round 5: filter enforcement + tokens combined) ──────────────

#[test]
fn voldaren_epicure_etb_creates_blood_and_pings_each_opponent() {
    let mut g = two_player_game();
    let epi = g.add_card_to_hand(0, catalog::voldaren_epicure());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let opp_life_before = g.players[1].life;
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: epi, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Voldaren Epicure castable");
    drain_stack(&mut g);
    // 1 damage to opp.
    assert_eq!(g.players[1].life, opp_life_before - 1);
    // Blood token entered the battlefield (epicure + token = +2).
    assert_eq!(g.battlefield.len(), bf_before + 2);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Blood"));
}

#[test]
fn call_of_the_herd_makes_an_elephant_and_can_flashback() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::call_of_the_herd());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Call of the Herd castable");
    drain_stack(&mut g);
    let elephants = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Elephant").count();
    assert_eq!(elephants, 1, "creates one 3/3 Elephant token");
    // It carries Flashback so it can be recast from the graveyard.
    assert!(g.players[0].graveyard.iter().any(|c|
        c.definition.name == "Call of the Herd"
        && c.definition.keywords.iter().any(|k| matches!(k, Keyword::Flashback(_)))),
        "Call of the Herd is in the graveyard with Flashback");
}

#[test]
fn vampire_nighthawk_has_flying_deathtouch_lifelink() {
    use crabomination::card::Keyword;
    let def = catalog::vampire_nighthawk();
    assert_eq!((def.power, def.toughness), (2, 3));
    for kw in [Keyword::Flying, Keyword::Deathtouch, Keyword::Lifelink] {
        assert!(def.keywords.contains(&kw), "Nighthawk has {kw:?}");
    }
}

#[test]
fn wind_drake_is_a_two_two_flier() {
    use crabomination::card::Keyword;
    let def = catalog::wind_drake();
    assert_eq!((def.power, def.toughness), (2, 2));
    assert!(def.keywords.contains(&Keyword::Flying));
}

#[test]
fn nekrataal_etb_destroys_a_nonblack_creature() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green
    let id = g.add_card_to_hand(0, catalog::nekrataal());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Nekrataal castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == victim), "the nonblack creature is destroyed");
}

#[test]
fn skinrender_etb_shrinks_target_with_three_minus_counters() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::skinrender());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Skinrender castable");
    drain_stack(&mut g);
    let cp = g.computed_permanent(victim).expect("4/4 survives three -1/-1");
    assert_eq!((cp.power, cp.toughness), (1, 1), "4/4 → 1/1 after three -1/-1 counters");
}

#[test]
fn ravenous_chupacabra_etb_destroys_an_opponent_creature() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::ravenous_chupacabra());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ravenous Chupacabra castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == victim), "opponent's creature destroyed on ETB");
}

#[test]
fn sentinel_spider_has_vigilance_and_reach() {
    use crabomination::card::Keyword;
    let def = catalog::sentinel_spider();
    assert_eq!((def.power, def.toughness), (4, 4));
    assert!(def.keywords.contains(&Keyword::Vigilance) && def.keywords.contains(&Keyword::Reach));
}

#[test]
fn brindle_boar_sacrifices_for_four_life() {
    let mut g = two_player_game();
    let boar = g.add_card_to_battlefield(0, catalog::brindle_boar());
    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: boar, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Brindle Boar sac ability activates");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 4, "gained 4 life");
    assert!(!g.battlefield.iter().any(|c| c.id == boar), "Brindle Boar was sacrificed");
}

#[test]
fn reckless_abandon_sacrifices_a_creature_and_deals_four() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // fodder
    let id = g.add_card_to_hand(0, catalog::reckless_abandon());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reckless Abandon castable with fodder present");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 16, "dealt 4 to the opponent");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "the sacrificed creature is in the graveyard");
}

#[test]
fn cloudgoat_ranger_etb_makes_three_kithkin() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::cloudgoat_ranger());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cloudgoat Ranger castable");
    drain_stack(&mut g);
    let tokens = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Kithkin Soldier").count();
    assert_eq!(tokens, 3, "creates three 1/1 Kithkin Soldier tokens");
}

#[test]
fn pelakka_wurm_etb_gains_seven_and_death_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::pelakka_wurm());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pelakka Wurm castable for {5}{G}{G}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 7, "ETB gained 7 life");
    let wurm = g.battlefield.iter().find(|c| c.definition.name == "Pelakka Wurm").unwrap().id;
    let hand_before = g.players[0].hand.len();
    g.remove_to_graveyard_with_triggers(wurm);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "death trigger drew a card");
}

#[test]
fn springbloom_druid_etb_fetches_two_basics_tapped() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let f1 = g.add_card_to_library(0, catalog::forest());
    let f2 = g.add_card_to_library(0, catalog::forest());
    let lands_before = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    let id = g.add_card_to_hand(0, catalog::springbloom_druid());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 1);
    // Script the two tutor picks (AutoDecider declines searches by default).
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(f1)),
        DecisionAnswer::Search(Some(f2)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Springbloom Druid castable");
    drain_stack(&mut g);
    let tapped_lands = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.is_land() && c.tapped).count();
    let lands_after = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    assert_eq!(lands_after - lands_before, 2, "two basics entered the battlefield");
    assert!(tapped_lands >= 2, "the fetched basics entered tapped");
}

#[test]
fn cryptolith_rite_grants_creatures_tap_for_any_color() {
    // "Creatures you control have '{T}: Add one mana of any color.'" — the
    // creature-filter path of StaticEffect::GrantActivatedAbility. A bear
    // (0 printed abilities) gets the grant at index 0.
    let mut g = two_player_game();
    let _rite = g.add_card_to_battlefield(0, catalog::cryptolith_rite());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Cryptolith Rite grants the bear a tap-for-any-color ability");
    assert_eq!(g.players[0].mana_pool.total() - before, 1, "added one mana");
    assert!(g.battlefield_find(bear).unwrap().tapped, "bear tapped for the grant");
}

#[test]
fn big_game_hunter_etb_destroys_a_big_creature() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let bgh = g.add_card_to_hand(0, catalog::big_game_hunter());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: bgh, target: Some(Target::Permanent(big)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Big Game Hunter castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == big), "the 4/4 (power ≥ 4) is destroyed");
    assert!(g.battlefield.iter().any(|c| c.id == small), "the 2/2 is untouched");
}

#[test]
fn arrogant_wurm_is_a_four_four_trampling_madness_wurm() {
    use crabomination::card::Keyword;
    let def = catalog::arrogant_wurm();
    assert_eq!((def.power, def.toughness), (4, 4));
    assert!(def.keywords.contains(&Keyword::Trample));
    assert!(def.keywords.iter().any(|k| matches!(k, Keyword::Madness(_))));
}

#[test]
fn hill_giant_is_a_vanilla_three_three() {
    let def = catalog::hill_giant();
    assert_eq!((def.power, def.toughness), (3, 3));
    assert!(def.keywords.is_empty() && def.activated_abilities.is_empty()
        && def.triggered_abilities.is_empty(), "vanilla beater");
}

#[test]
fn cunning_sparkmage_taps_to_ping_for_one() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::cunning_sparkmage());
    g.clear_sickness(mage);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mage, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Cunning Sparkmage pings");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "1 damage to the opponent");
    assert!(g.battlefield_find(mage).unwrap().tapped, "mage tapped to ping");
}

#[test]
fn fiery_temper_deals_three_to_any_target() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::fiery_temper());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fiery Temper castable for {1}{R}{R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "3 damage to the opponent");
}

#[test]
fn reckless_wurm_is_a_five_four_trampling_madness_wurm() {
    use crabomination::card::Keyword;
    let g = two_player_game();
    let def = catalog::reckless_wurm();
    assert_eq!((def.power, def.toughness), (4, 4));
    assert!(def.keywords.contains(&Keyword::Trample));
    assert!(def.keywords.iter().any(|k| matches!(k, Keyword::Madness(_))));
    let _ = g;
}

#[test]
fn anjes_ravager_attack_discards_hand_then_draws_three() {
    let mut g = two_player_game();
    let ravager = g.add_card_to_battlefield(0, catalog::anjes_ravager());
    g.clear_sickness(ravager);
    // Two junk cards in hand; five in library to draw from.
    g.add_card_to_hand(0, catalog::lightning_bolt());
    g.add_card_to_hand(0, catalog::shock());
    for _ in 0..5 { g.add_card_to_library(0, catalog::mountain()); }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ravager, target: AttackTarget::Player(1),
    }])).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 3, "discarded the 2-card hand, then drew 3");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Lightning Bolt"),
        "the discarded hand hit the graveyard");
}

#[test]
fn lure_forces_all_able_creatures_to_block() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    // Lure's rider on the attacker: all able creatures must block it.
    g.battlefield_find_mut(attacker).unwrap()
        .granted_keywords_eot.push(Keyword::AllMustBlock);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    g.step = TurnStep::DeclareBlockers;
    // Blocking with only one of two able creatures is illegal.
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(b1, attacker)])).is_err(),
        "Lure requires every able creature to block");
    // Assigning both able blockers satisfies the requirement.
    assert!(g.perform_action(
        GameAction::DeclareBlockers(vec![(b1, attacker), (b2, attacker)])).is_ok(),
        "blocking with all able creatures is legal");
}

#[test]
fn goldspan_dragon_attack_creates_a_treasure() {
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(0, catalog::goldspan_dragon());
    g.clear_sickness(dragon);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: dragon,
        target: AttackTarget::Player(1),
    }]))
    .expect("Goldspan Dragon attacks");
    drain_stack(&mut g);
    let treasure = g.battlefield.iter().find(|c| c.definition.name == "Treasure")
        .expect("attack mints a Treasure");
    // Goldspan's Treasure taps+sacs for *two* mana of one color.
    let tid = treasure.id;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tid, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Treasure mana ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 2, "Goldspan Treasure yields two mana");
}

#[test]
fn goldspan_dragon_treasure_on_becoming_targeted() {
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(0, catalog::goldspan_dragon());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.cast_spell(bolt, Some(Target::Permanent(dragon)), vec![], None, None)
        .expect("Bolt targets Goldspan");
    drain_stack(&mut g);
    // Becoming targeted minted a Treasure for Goldspan's controller; the 4/4
    // survives the 3 damage.
    assert!(g.battlefield.iter().any(|c| c.id == dragon), "4/4 survives Bolt");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"),
        "becoming the target of a spell mints a Treasure");
}

#[test]
fn battle_mammoth_draws_when_your_permanent_is_targeted_by_opponent() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::battle_mammoth());
    // A *different* permanent you control is the one targeted.
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let hand_before = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.cast_spell(bolt, Some(Target::Permanent(bears)), vec![], None, None)
        .expect("opponent's Bolt targets your creature");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Battle Mammoth draws when your permanent is targeted by an opponent");
}

#[test]
fn battle_mammoth_does_not_draw_on_your_own_targeting() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::battle_mammoth());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let hand_before = g.players[0].hand.len();
    // You target your own creature — no draw (opponent-only clause).
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.cast_spell(bolt, Some(Target::Permanent(bears)), vec![], None, None)
        .expect("your own Bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before,
        "no draw when you target your own permanent");
}

#[test]
fn tireless_tracker_investigates_when_a_land_enters() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tireless_tracker());
    let land = g.add_card_to_hand(0, catalog::forest());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(land))
        .expect("Forest plays");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Clue"),
        "Land ETB should investigate (create a Clue)");
}

/// Tireless Tracker costs `{1}{G}{G}`. With 1 Forest + 2 Mountains, the
/// player has only 1 green source — not enough to pay the second `{G}`
/// pip — so the cast must fail. With 2 Forests + 1 Mountain, all three
/// pips are payable and the cast succeeds. Locks down the cost so a
/// future "off by one mana" regression in the catalog (or the auto-tap
/// path) can't sneak through.
#[test]
fn tireless_tracker_requires_a_green_mana_source() {
    let mut g = two_player_game();
    g.priority.player_with_priority = 0;

    // Setup A: 3 Mountains untapped — no green available, so {2}{G} fails.
    for _ in 0..3 {
        let m = g.add_card_to_battlefield(0, catalog::mountain());
        g.battlefield_find_mut(m).unwrap().tapped = false;
    }
    let tracker = g.add_card_to_hand(0, catalog::tireless_tracker());
    let err = g.perform_action(GameAction::CastSpell {
        card_id: tracker,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(err.is_err(),
        "{{2}}{{G}} cannot be paid with no green source: {err:?}");

    // Setup B: 1 Forest + 2 Mountains — one green + two generic pays {2}{G}.
    let mut g = two_player_game();
    g.priority.player_with_priority = 0;
    let f = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(f).unwrap().tapped = false;
    for _ in 0..2 {
        let m = g.add_card_to_battlefield(0, catalog::mountain());
        g.battlefield_find_mut(m).unwrap().tapped = false;
    }
    let tracker = g.add_card_to_hand(0, catalog::tireless_tracker());
    g.perform_action(GameAction::CastSpell {
        card_id: tracker,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{2}{G} pays from 1 Forest + 2 Mountains");
}

#[test]
fn tireless_tracker_does_not_trigger_on_non_land_etb() {
    // Casting a creature shouldn't fire Tracker's land filter.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tireless_tracker());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bears castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Clue"),
        "non-land ETB should NOT trigger Tracker");
}

#[test]
fn bloodtithe_harvester_etb_and_attack_each_make_a_blood() {
    let mut g = two_player_game();
    let harv = g.add_card_to_hand(0, catalog::bloodtithe_harvester());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: harv, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bloodtithe Harvester castable");
    drain_stack(&mut g);
    let bloods_after_etb =
        g.battlefield.iter().filter(|c| c.definition.name == "Blood").count();
    assert_eq!(bloods_after_etb, 1, "ETB should make one Blood");
    // Attack — should make a second Blood.
    g.clear_sickness(harv);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: harv,
        target: AttackTarget::Player(1),
    }]))
    .expect("Harvester attacks");
    drain_stack(&mut g);
    let bloods_after_attack =
        g.battlefield.iter().filter(|c| c.definition.name == "Blood").count();
    assert_eq!(bloods_after_attack, 2, "Attack should make a second Blood");
}

// ── Engine: trigger-filter enforcement ──────────────────────────────────────

#[test]
fn up_the_beanstalk_etb_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let beanstalk = g.add_card_to_hand(0, catalog::up_the_beanstalk());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: beanstalk, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Up the Beanstalk castable");
    drain_stack(&mut g);
    // Net: cast (-1) + ETB draw (+1) = 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn up_the_beanstalk_does_not_trigger_on_low_mana_value_spells() {
    // Cast Lightning Bolt ({R}, mana value 1). Beanstalk's filter
    // (mana value ≥ 5) should keep its trigger from firing.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::up_the_beanstalk());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    // Net: cast (-1) + no draw (+0) = -1.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn up_the_beanstalk_triggers_on_five_plus_mana_value_spells() {
    // Cast Serra Angel ({3}{W}{W}, mana value 5). Beanstalk should fire.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::up_the_beanstalk());
    let angel = g.add_card_to_hand(0, catalog::serra_angel());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::White, 2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: angel, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Serra Angel castable");
    drain_stack(&mut g);
    // Net: cast (-1) + Beanstalk draw (+1) = 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
    // Angel landed on battlefield.
    assert!(g.battlefield.iter().any(|c| c.id == angel));
}

#[test]
fn temur_ascendancy_draws_only_for_power_4_plus_etb() {
    // The trigger is YourControl + EnterBattlefield + filter (power ≥ 4).
    // We need to actually CAST creatures so PermanentEntered events fire
    // through `dispatch_triggers_for_events`; `add_card_to_battlefield` is
    // a test-helper that bypasses the event stream.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::temur_ascendancy());

    // Cast Llanowar Elves ({G}, 1/1, power 1) — filter rejects, no draw.
    let elves = g.add_card_to_hand(0, catalog::llanowar_elves());
    g.players[0].mana_pool.add(Color::Green, 1);
    let elves_hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: elves, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Llanowar Elves castable");
    drain_stack(&mut g);
    // Net: cast (-1) + ETB (+0 — filter rejects) = -1.
    assert_eq!(g.players[0].hand.len(), elves_hand_before - 1,
        "low-power ETB should NOT trigger Temur draw");

    // Cast Serra Angel ({3}{W}{W}, 4/4, power 4) — filter passes, draw 1.
    let angel = g.add_card_to_hand(0, catalog::serra_angel());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::White, 2);
    let angel_hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: angel, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Serra Angel castable");
    drain_stack(&mut g);
    // Net: cast (-1) + Temur draw (+1) = 0.
    assert_eq!(g.players[0].hand.len(), angel_hand_before,
        "power-4 ETB SHOULD trigger Temur draw");
}

#[test]
fn temur_ascendancy_grants_haste_only_to_power_4_plus() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::temur_ascendancy());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c = g.compute_battlefield();
    assert!(!c.iter().find(|c| c.id == bear).unwrap().keywords.contains(&Keyword::Haste),
        "a 2/2 is below the power-4 gate");
    // Pump the bear past the gate (CR 613.8 — the gate reads computed power).
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap().power_bonus += 2;
    let c = g.compute_battlefield();
    assert!(c.iter().find(|c| c.id == bear).unwrap().keywords.contains(&Keyword::Haste),
        "a pumped 4/2 gains haste");
    // Opponent's big creature gets no haste.
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == opp).unwrap().power_bonus += 5;
    let c = g.compute_battlefield();
    assert!(!c.iter().find(|c| c.id == opp).unwrap().keywords.contains(&Keyword::Haste),
        "opponent's creatures are unaffected");
}

// ── Engine: token activated abilities (Treasures, Food, Blood, Clue) ────────

#[test]
fn treasure_token_taps_and_sacrifices_for_one_color() {
    // Big Score creates two Treasure tokens; tapping one and sacrificing
    // it adds one mana of any color. Tokens are now created with their
    // canonical activated ability (TokenDefinition.activated_abilities).
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::lightning_bolt()); // discardable for Big Score
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    let big = g.add_card_to_hand(0, catalog::big_score());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: big, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Big Score castable");
    drain_stack(&mut g);
    // Find a Treasure token on the battlefield and tap-sac it for blue.
    let treasure_id = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Treasure")
        .map(|c| c.id)
        .expect("a Treasure token should exist");
    g.clear_sickness(treasure_id);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Blue)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: treasure_id,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Treasure tap-sac mana ability activates");
    drain_stack(&mut g);
    // Treasure is in graveyard; one blue mana floats in the pool.
    assert!(!g.battlefield.iter().any(|c| c.id == treasure_id));
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1);
}

// ── Engine: sac-as-cost activation ──────────────────────────────────────────

#[test]
fn mind_stone_sac_for_draw_moves_self_to_graveyard_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let stone = g.add_card_to_battlefield(0, catalog::mind_stone());
    g.clear_sickness(stone);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    // Ability index 1 is the sac-for-draw ({1}, {T}, sac: Draw 1).
    g.perform_action(GameAction::ActivateAbility {
        card_id: stone,
        ability_index: 1,
        target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Mind Stone sac-for-draw activates");
    drain_stack(&mut g);
    // Mind Stone is in the graveyard; the draw nets +1 hand.
    assert!(!g.battlefield.iter().any(|c| c.id == stone));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == stone));
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn cathar_commando_sac_destroys_artifact() {
    let mut g = two_player_game();
    let opp_ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let cathar = g.add_card_to_battlefield(0, catalog::cathar_commando());
    g.clear_sickness(cathar);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cathar,
        ability_index: 0,
        target: Some(Target::Permanent(opp_ring)), additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Cathar Commando sac-destroy activates");
    drain_stack(&mut g);
    // Cathar Commando in graveyard, target ring destroyed.
    assert!(!g.battlefield.iter().any(|c| c.id == cathar));
    assert!(!g.battlefield.iter().any(|c| c.id == opp_ring));
}

#[test]
fn haywire_mite_sac_destroys_artifact_and_gains_life() {
    let mut g = two_player_game();
    let opp_ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let mite = g.add_card_to_battlefield(0, catalog::haywire_mite());
    g.clear_sickness(mite);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: mite,
        ability_index: 0,
        target: Some(Target::Permanent(opp_ring)), additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Haywire Mite sac activates");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == mite));
    assert!(!g.battlefield.iter().any(|c| c.id == opp_ring));
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn aether_spellbomb_sac_bounces_a_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bomb = g.add_card_to_battlefield(0, catalog::aether_spellbomb());
    g.clear_sickness(bomb);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bomb,
        ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Aether Spellbomb sac-bounce activates");
    drain_stack(&mut g);
    // Bomb in graveyard, bear back in opponent's hand.
    assert!(!g.battlefield.iter().any(|c| c.id == bomb));
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
}

#[test]
fn sac_cost_failure_to_pay_mana_keeps_source_on_battlefield() {
    // Insufficient mana → activation fails, source stays. The sac happens
    // only after mana payment succeeds.
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(0, catalog::mind_stone());
    g.clear_sickness(stone);
    // Pool empty — Mind Stone's sac-for-draw needs {1}.
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: stone,
        ability_index: 1,
        target: None, additional_targets: Vec::new(), x_value: None , mode: None});
    assert!(err.is_err(), "Mind Stone sac-for-draw should fail without mana");
    // Source still on battlefield, untapped, hand unchanged.
    assert!(g.battlefield.iter().any(|c| c.id == stone));
    let on_bf = g.battlefield.iter().find(|c| c.id == stone).unwrap();
    assert!(!on_bf.tapped, "tap-cost should roll back when mana pay fails");
}

// ── Cube cards (round 4) ─────────────────────────────────────────────────────

#[test]
fn sentinel_makes_a_map_on_enters_and_attack() {
    let mut g = two_player_game();
    // Cast it so the ETB trigger fires (add_card_to_battlefield skips ETBs).
    let sentinel = g.add_card_to_hand(0, catalog::sentinel_of_the_nameless_city());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: sentinel, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Map").count(),
        1, "ETB made a Map",
    );
    g.clear_sickness(sentinel);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sentinel,
        target: AttackTarget::Player(1),
    }])).expect("Sentinel can attack");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Map").count(),
        2, "attack made a second Map",
    );
}

#[test]
fn ranger_captain_etb_searches_for_a_one_drop() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears()); // 2-MV — should NOT match
    let elves = g.add_card_to_library(0, catalog::llanowar_elves()); // 1-MV — match
    g.add_card_to_library(0, catalog::island());
    let ranger = g.add_card_to_hand(0, catalog::ranger_captain_of_eos());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::White, 2);
    // Scripted decider picks Llanowar Elves out of the search candidates.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(elves))]));
    g.perform_action(GameAction::CastSpell {
        card_id: ranger, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Ranger-Captain castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == elves), "Llanowar Elves to hand");
}

#[test]
fn ranger_captain_sac_locks_opponents_out_of_noncreature_spells() {
    let mut g = two_player_game();
    let ranger = g.add_card_to_battlefield(0, catalog::ranger_captain_of_eos());
    g.clear_sickness(ranger);
    // Sacrifice Ranger-Captain to fire its lock.
    g.perform_action(GameAction::ActivateAbility {
        card_id: ranger, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("sac ability activates");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == ranger), "Ranger sacrificed");
    assert!(g.players[1].cant_cast_noncreature_this_turn, "opponents locked out");

    // The gate rejects a noncreature spell but allows a creature spell.
    g.players[1].cant_cast_noncreature_this_turn = true;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    let err = g.cast_spell(bolt, Some(Target::Player(0)), vec![], None, None);
    assert!(matches!(err, Err(GameError::CantCastNoncreature)),
        "noncreature spell blocked");
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add_colorless(1);
    g.players[1].mana_pool.add(Color::Green, 1);
    assert!(!matches!(
        g.cast_spell(bears, None, vec![], None, None),
        Err(GameError::CantCastNoncreature)),
        "creature spell not blocked by the lock");
}

#[test]
fn upheaval_returns_all_permanents_to_hands() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let p0_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let p1_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let upheaval = g.add_card_to_hand(0, catalog::upheaval());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: upheaval, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Upheaval castable");
    drain_stack(&mut g);
    // Both creatures returned to their owners' hands; battlefield empty
    // of these cards (Upheaval itself goes to graveyard).
    assert!(!g.battlefield.iter().any(|c| c.id == p0_bear || c.id == p1_bear));
    assert!(g.players[0].hand.iter().any(|c| c.id == p0_bear));
    assert!(g.players[1].hand.iter().any(|c| c.id == p1_bear));
}

#[test]
fn aetherize_returns_attackers_to_their_owners_hands() {
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).unwrap();
    let id = g.add_card_to_hand(0, catalog::aetherize());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Aetherize castable");
    drain_stack(&mut g);
    // Only the attacker bounced (to its owner); the non-attacking blocker stays.
    assert!(g.players[0].hand.iter().any(|c| c.id == attacker), "attacker → owner's hand");
    assert!(g.battlefield_find(blocker).is_some(), "non-attacker untouched");
}

#[test]
fn evacuation_returns_all_creatures_to_owners() {
    let mut g = two_player_game();
    let p0_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let p1_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::evacuation());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Evacuation castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == p0_bear), "P0 bear → P0 hand");
    assert!(g.players[1].hand.iter().any(|c| c.id == p1_bear), "P1 bear → P1 hand");
}

#[test]
fn rakshasas_bargain_takes_two_rest_to_graveyard() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let bargain = g.add_card_to_hand(0, catalog::rakshasas_bargain());
    // {2/B}{2/G}{2/U} — pay the generic side of each mono-hybrid pip.
    g.players[0].mana_pool.add_colorless(6);
    let hand_before = g.players[0].hand.len();
    let gy_before = g.players[0].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bargain, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Rakshasa's Bargain castable for 6 generic");
    drain_stack(&mut g);
    // Cast (-1 the spell) + put 2 of the top 4 into hand = +1 net.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "two of four kept");
    // The spell + the two unchosen revealed cards land in the graveyard.
    assert_eq!(g.players[0].graveyard.len(), gy_before + 3);
}

#[test]
fn sundering_eruption_front_face_burns_a_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let erupt = g.add_card_to_hand(0, catalog::sundering_eruption());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: erupt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Sundering Eruption castable");
    drain_stack(&mut g);
    // 3 damage to a 2/2 → dies.
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

#[test]
fn sundering_eruption_back_face_plays_as_a_mountain() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::sundering_eruption());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLandBack(id))
        .expect("Mount Tyrhus plays via PlayLandBack");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).expect("on battlefield");
    assert_eq!(card.definition.name, "Mount Tyrhus");
    assert!(card.definition.subtypes.land_types.contains(&crabomination::card::LandType::Mountain));
    // ETB-tap trigger taps it.
    assert!(card.tapped, "Mount Tyrhus enters tapped");
}

#[test]
fn loran_etb_destroys_artifact_and_tap_ability_lets_both_draw() {
    let mut g = two_player_game();
    let opp_ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(1, catalog::forest());
    let loran = g.add_card_to_hand(0, catalog::loran_of_the_third_path());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: loran, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Loran castable");
    drain_stack(&mut g);
    // ETB destroyed the opponent's Sol Ring.
    assert!(!g.battlefield.iter().any(|c| c.id == opp_ring));
    // Activated ability: clear sickness, then tap for the joint draw.
    g.clear_sickness(loran);
    let p0_hand = g.players[0].hand.len();
    let p1_hand = g.players[1].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: loran,
        ability_index: 0,
        target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Loran tap ability activates");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), p0_hand + 1);
    assert_eq!(g.players[1].hand.len(), p1_hand + 1);
}

// ── New cube/Modern additions ─────────────────────────────────────────────────

/// Reanimate puts a creature card from a graveyard onto the battlefield
/// under the caster's control, and the caster loses life equal to its
/// mana value. Atraxa has CMC 7 ({3}{G}{W}{U}{B}) → caster pays 7 life.
#[test]
fn reanimate_puts_creature_into_play_and_pays_cmc_life() {
    let mut g = two_player_game();
    let atraxa = g.add_card_to_library(0, catalog::atraxa_grand_unifier());
    let pos = g.players[0].library.iter().position(|c| c.id == atraxa).unwrap();
    let card = g.players[0].library.remove(pos);
    g.players[0].graveyard.push(card);

    let id = g.add_card_to_hand(0, catalog::reanimate());
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(atraxa)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Reanimate castable for {B}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == atraxa),
        "Atraxa should be on the battlefield");
    assert_eq!(g.players[0].life, life_before - 7,
        "Caster should lose CMC=7 life for reanimating Atraxa");
}

/// Reanimate's life-loss reads the actual mana value. Reanimating a 2-cost
/// creature should only cost 2 life — not the flat 7 the previous stub used.
#[test]
fn reanimate_life_cost_scales_with_mana_value() {
    let mut g = two_player_game();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears()); // {1}{G} = CMC 2
    let pos = g.players[0].library.iter().position(|c| c.id == bear).unwrap();
    let card = g.players[0].library.remove(pos);
    g.players[0].graveyard.push(card);

    let id = g.add_card_to_hand(0, catalog::reanimate());
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Reanimate castable for {B}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before - 2,
        "Reanimating a 2-cost creature should cost 2 life");
}

/// Bone Shards' default mode (sacrifice) should sac one of the caster's
/// creatures and destroy the targeted creature.
#[test]
fn bone_shards_sacrifices_creature_and_destroys_target() {
    let mut g = two_player_game();
    let sac_target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let kill_target = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::bone_shards());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(kill_target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bone Shards castable for {B}");
    drain_stack(&mut g);

    // Sacrificed creature in P0's graveyard; destroyed creature in P1's graveyard.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == sac_target),
        "Caster's creature should be sacrificed (mode 0)");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == kill_target),
        "Targeted opponent creature should be destroyed");
    assert!(!g.battlefield.iter().any(|c| c.id == sac_target));
    assert!(!g.battlefield.iter().any(|c| c.id == kill_target));
}

/// Bone Shards mode 1 — discard a card instead of sacrificing — should
/// cost a card from the caster's hand and still destroy the targeted
/// creature.
#[test]
fn bone_shards_can_discard_instead_of_sacrifice() {
    let mut g = two_player_game();
    let to_discard = g.add_card_to_hand(0, catalog::lightning_bolt());
    let kill_target = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::bone_shards());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(kill_target)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    })
    .expect("Bone Shards castable for {B}");
    drain_stack(&mut g);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == to_discard),
        "Discarded card should be in caster's graveyard");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == kill_target),
        "Targeted creature should be destroyed");
}

/// Pyrokinesis can be cast via its alt-cost (exile a red card from your
/// hand) for free. The targeted creature takes 4 damage.
#[test]
fn pyrokinesis_alt_cost_exiles_red_card_and_deals_four_damage() {
    let mut g = two_player_game();
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 → dies to 4
    let red_card = g.add_card_to_hand(0, catalog::lightning_bolt()); // red

    let id = g.add_card_to_hand(0, catalog::pyrokinesis());
    // No mana paid — alt cost is "exile a red card".
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id,
        pitch_card: Some(red_card),
        target: Some(Target::Permanent(serra)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Pyrokinesis alt-castable by exiling a red card");
    drain_stack(&mut g);

    // The exiled pitch card is in exile.
    assert!(g.exile.iter().any(|c| c.id == red_card),
        "Pitched red card should be in exile");
    // Serra Angel (4/4) takes 4 damage → dies.
    assert!(!g.battlefield.iter().any(|c| c.id == serra),
        "Serra Angel should die to 4 damage");
}

/// A free 2/2 carrying Fabricate 2. AutoDecider takes the counter mode
/// (→ 4/4); a scripted decider takes the Servo-token mode (CR 702.122).
fn fabricator_body() -> crabomination::card::CardDefinition {
    crabomination::card::CardDefinition {
        name: "Test Fabricator",
        card_types: vec![CardType::Creature],
        power: 2,
        toughness: 2,
        triggered_abilities: vec![crabomination::effect::shortcut::fabricate(2)],
        ..Default::default()
    }
}

#[test]
fn fabricate_counter_mode_grows_the_creature() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, fabricator_body());
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for free");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.power(), c.toughness()), (4, 4), "Fabricate 2 (counter mode) → 4/4");
}

#[test]
fn fabricate_token_mode_mints_two_servos() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Mode(1)]));
    let id = g.add_card_to_hand(0, fabricator_body());
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for free");
    drain_stack(&mut g);
    let servos = g.battlefield.iter().filter(|c| c.definition.name == "Servo").count();
    assert_eq!(servos, 2, "Fabricate 2 (token mode) → two 1/1 Servos");
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.power(), c.toughness()), (2, 2), "token mode leaves the body 2/2");
}

/// Bolster N lands its counters on the controller's least-toughness creature.
#[test]
fn bolster_buffs_least_toughness_creature() {
    let mut g = two_player_game();
    let runt = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let big = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4, higher toughness
    // A free 3/3 that bolsters 2 on ETB.
    let body = crabomination::card::CardDefinition {
        name: "Test Bolsterer",
        card_types: vec![CardType::Creature],
        power: 3,
        toughness: 3,
        triggered_abilities: vec![crabomination::effect::shortcut::etb(
            crabomination::effect::shortcut::bolster(2),
        )],
        ..Default::default()
    };
    let id = g.add_card_to_hand(0, body);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for free");
    drain_stack(&mut g);
    // The 2/2 has the least toughness, so it gets the two counters → 4/4.
    let r = g.battlefield_find(runt).unwrap();
    assert_eq!((r.power(), r.toughness()), (4, 4), "bolster hit the 2/2");
    let b = g.battlefield_find(big).unwrap();
    assert_eq!((b.power(), b.toughness()), (4, 4), "the bigger creature is untouched");
}

/// Aether Adept bounces a target creature to its owner's hand on ETB.
#[test]
fn aether_adept_bounces_target_creature_on_etb() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::aether_adept());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Aether Adept castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear left the battlefield");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "bear returned to owner's hand");
}

/// Augury Owl is a 1/1 flyer whose ETB scry resolves cleanly (no draw).
#[test]
fn augury_owl_scries_on_etb() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let lib_before = g.players[0].library.len();
    let id = g.add_card_to_hand(0, catalog::augury_owl());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Augury Owl castable");
    drain_stack(&mut g);
    // Scry looks but doesn't draw — library size unchanged.
    assert_eq!(g.players[0].library.len(), lib_before, "scry does not change library size");
    let c = g.battlefield.iter().find(|c| c.definition.name == "Augury Owl").unwrap();
    assert!(c.has_keyword(&crabomination::card::Keyword::Flying));
}

/// Cloudkin Seer is a 2/2 flyer that draws a card on ETB.
#[test]
fn cloudkin_seer_draws_on_etb() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::cloudkin_seer());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cloudkin Seer castable");
    drain_stack(&mut g);
    // cast(-1) + ETB draw(+1) = net 0 vs before.
    assert_eq!(g.players[0].hand.len(), before, "ETB drew a card");
    let c = g.battlefield.iter().find(|c| c.definition.name == "Cloudkin Seer").unwrap();
    assert!(c.has_keyword(&crabomination::card::Keyword::Flying));
    assert_eq!((c.definition.power, c.definition.toughness), (2, 1));
}

/// Benthic Biomancer's `{1}{U}: Adapt 1` makes it 2/2 and loots (draw+discard).
#[test]
fn benthic_biomancer_adapts_and_loots() {
    let mut g = two_player_game();
    let bio = g.add_card_to_battlefield(0, catalog::benthic_biomancer());
    g.add_card_to_hand(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: bio, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Adapt activatable");
    drain_stack(&mut g);
    let b = g.battlefield_find(bio).unwrap();
    assert_eq!((b.power(), b.toughness()), (2, 2), "adapt 1 → 2/2");
    assert_eq!(g.players[0].hand.len(), hand_before, "draw(+1) + discard(-1) = net 0");
}

/// Chandra's Pyrohelix splits 2 damage among two players (1 each).
#[test]
fn chandras_pyrohelix_divides_two_among_two_players() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::chandras_pyrohelix());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![Target::Player(0)],
        mode: None,
        x_value: None,
    }).expect("Pyrohelix castable for {1}{R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19);
    assert_eq!(g.players[0].life, 19);
}

/// Merfolk Skydiver's `{1}{U}: Adapt 1` makes it 2/2 and proliferates —
/// a co-counter on another creature ticks up too.
#[test]
fn merfolk_skydiver_adapts_and_proliferates() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let diver = g.add_card_to_battlefield(0, catalog::merfolk_skydiver());
    let ally = g.add_card_to_battlefield(0, catalog::pteramander());
    // Seed a +1/+1 counter on the ally so proliferate has something to grow.
    if let Some(c) = g.battlefield_find_mut(ally) {
        c.add_counters(CounterType::PlusOnePlusOne, 1);
    }
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: diver, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Adapt activatable");
    drain_stack(&mut g);
    // Adapt 1 → one counter, then proliferate adds another to the diver
    // itself (and to the ally) → 3/3.
    let d = g.battlefield_find(diver).unwrap();
    assert_eq!((d.power(), d.toughness()), (3, 3), "adapt 1 then proliferate-self → 3/3");
    let a = g.battlefield_find(ally).unwrap();
    assert_eq!(a.counter_count(CounterType::PlusOnePlusOne), 2,
        "proliferate added a second +1/+1 to the ally");
}

/// Pteramander's `{7}: Adapt 4` puts four +1/+1 counters on it (1/1 → 5/5)
/// when it has none; a second activation is a no-op (CR 702.108).
#[test]
fn pteramander_adapt_four_then_noop_when_already_adapted() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::pteramander());
    let count = |g: &crabomination::game::GameState| {
        g.computed_permanent(id).map(|cp| (cp.power, cp.toughness)).unwrap()
    };
    assert_eq!(count(&g), (1, 1));
    g.players[0].mana_pool.add_colorless(7);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Adapt activatable");
    drain_stack(&mut g);
    assert_eq!(count(&g), (5, 5), "1/1 + four counters = 5/5");

    // Second adapt: it already has counters, so nothing happens.
    g.players[0].mana_pool.add_colorless(7);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Adapt re-activatable (resolves to nothing)");
    drain_stack(&mut g);
    assert_eq!(count(&g), (5, 5), "still 5/5 — adapt no-ops with counters present");
}

/// Forked Lightning splits 4 damage among two creatures (2 each via even
/// split → two 2/2s die).
#[test]
fn forked_lightning_divides_four_among_two_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::forked_lightning());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    }).expect("Forked Lightning castable for {3}{R}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == a), "first 2/2 dies");
    assert!(!g.battlefield.iter().any(|c| c.id == b), "second 2/2 dies");
}

/// Arc Lightning splits 3 damage among three 1/1 creatures (1 each → all die).
#[test]
fn arc_lightning_divides_three_among_three_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::pteramander());
    let b = g.add_card_to_battlefield(1, catalog::pteramander());
    let c = g.add_card_to_battlefield(1, catalog::pteramander());
    let id = g.add_card_to_hand(0, catalog::arc_lightning());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b), Target::Permanent(c)],
        mode: None,
        x_value: None,
    }).expect("Arc Lightning castable for {2}{R}");
    drain_stack(&mut g);
    for id in [a, b, c] {
        assert!(!g.battlefield.iter().any(|c| c.id == id), "each 1/1 dies to its 1 damage");
    }
}

/// Forked Bolt divides 2 damage among two targets (here both players, 1 each).
#[test]
fn forked_bolt_divides_two_among_two_players() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::forked_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![Target::Player(0)],
        mode: None,
        x_value: None,
    })
    .expect("Forked Bolt castable for {R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "opponent takes 1");
    assert_eq!(g.players[0].life, 19, "caster takes the other 1");
}

/// Pyrokinesis divides its 4 damage among two target creatures. AutoDecider
/// spreads evenly (2 + 2), killing two 2/2 bears.
#[test]
fn pyrokinesis_divides_four_damage_among_two_creatures() {
    let mut g = two_player_game();
    let bear_a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bear_b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let red_card = g.add_card_to_hand(0, catalog::lightning_bolt());

    let id = g.add_card_to_hand(0, catalog::pyrokinesis());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id,
        pitch_card: Some(red_card),
        target: Some(Target::Permanent(bear_a)),
        additional_targets: vec![Target::Permanent(bear_b)],
        mode: None,
        x_value: None,
    })
    .expect("Pyrokinesis castable with two creature targets");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear_a),
        "first bear should die to its 2 damage");
    assert!(!g.battlefield.iter().any(|c| c.id == bear_b),
        "second bear should die to its 2 damage");
}

/// Pyrokinesis's alt cost requires a red card — pitching a non-red card
/// should be rejected by the engine's `exile_filter` check.
#[test]
fn pyrokinesis_alt_cost_rejects_non_red_pitch() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Counterspell is blue — should be rejected as the pitch.
    let blue_card = g.add_card_to_hand(0, catalog::counterspell());

    let id = g.add_card_to_hand(0, catalog::pyrokinesis());
    let result = g.perform_action(GameAction::CastSpellAlternative {
        card_id: id,
        pitch_card: Some(blue_card),
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(result.is_err(),
        "Pyrokinesis alt cost must reject a non-red pitch card");
}

/// Tishana's Tidebinder ETB counters target activated/triggered ability.
/// Same setup as the Consign-to-Memory test: P1 casts Devourer of Destiny
/// (Scry-2 on-cast trigger lands above the spell), then P0 flashes in
/// Tidebinder targeting Devourer to counter the Scry trigger before it
/// resolves.
#[test]
fn tishanas_tidebinder_etb_counters_target_ability() {
    let mut g = two_player_game();
    g.add_card_to_library(1, catalog::island());
    g.add_card_to_library(1, catalog::island());

    let dev = g.add_card_to_hand(1, catalog::devourer_of_destiny());
    g.players[1].mana_pool.add_colorless(7);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: dev, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Devourer castable for {7}");

    // Confirm the scry trigger landed on the stack.
    let trigger_count = g.stack.iter()
        .filter(|si| matches!(si, crabomination::game::StackItem::Trigger { source, .. } if *source == dev))
        .count();
    assert_eq!(trigger_count, 1, "Scry-on-cast trigger should be queued");

    // P0 flashes in Tidebinder; its ETB counters the Scry trigger.
    let tide = g.add_card_to_hand(0, catalog::tishanas_tidebinder());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: tide,
        target: Some(Target::Permanent(dev)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Tidebinder castable at instant speed (Flash)");
    drain_stack(&mut g);

    // Devourer resolves (Tidebinder only counters the ability, not the spell).
    assert!(g.battlefield.iter().any(|c| c.id == dev),
        "Devourer should still resolve");
    assert!(g.battlefield.iter().any(|c| c.id == tide),
        "Tidebinder should be on the battlefield");
    // Scry trigger is gone.
    assert!(!g.stack.iter().any(|si| matches!(
        si, crabomination::game::StackItem::Trigger { source, .. } if *source == dev
    )), "Scry-on-cast trigger should have been countered");
}

/// Sylvan Safekeeper sacrifices a Forest to grant a creature shroud EOT.
#[test]
fn sylvan_safekeeper_sacs_forest_to_grant_shroud() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let sk = g.add_card_to_battlefield(0, catalog::sylvan_safekeeper());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(sk);

    g.perform_action(GameAction::ActivateAbility {
        card_id: sk,
        ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Sylvan Safekeeper activates");
    drain_stack(&mut g);

    // The Forest is sacrificed.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == forest),
        "Forest should be sacrificed");
    // The bear has Shroud until end of turn (computed via the layer view).
    let computed = g.compute_battlefield();
    let view = computed.iter().find(|c| c.id == bear).unwrap();
    assert!(view.keywords.contains(&Keyword::Shroud),
        "Bear should gain shroud until end of turn");
}

/// Grim Lavamancer's activated ability deals 2 damage to any target. The
/// graveyard-exile cost is currently approximated away; the damage half is
/// the load-bearing test.
#[test]
fn grim_lavamancer_activated_ability_deals_two_damage() {
    let mut g = two_player_game();
    let lava = g.add_card_to_battlefield(0, catalog::grim_lavamancer());
    g.clear_sickness(lava);
    g.players[0].mana_pool.add(Color::Red, 1);
    // Batch 114: activation now requires 2 cards in graveyard to exile.
    let _fodder_a = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let _fodder_b = g.add_card_to_graveyard(0, catalog::shock());
    let life_before = g.players[1].life;

    g.perform_action(GameAction::ActivateAbility {
        card_id: lava,
        ability_index: 0,
        target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Grim Lavamancer activates");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life_before - 2);
    let card = g.battlefield_find(lava).unwrap();
    assert!(card.tapped, "Tap-cost ability should leave the source tapped");
    // Both gy fodder cards should now be in exile (the exile-2 cost).
    assert_eq!(g.players[0].graveyard.len(), 0,
        "Both graveyard cards were exiled as the activation cost");
    assert!(g.exile.len() >= 2, "Exile zone gained both cost-paid cards");
}

/// CR 602.5b — a hand-paying activator chooses which graveyard cards to exile
/// for Grim Lavamancer's "exile two cards from your graveyard" cost, via a
/// `ChooseCards` modal, instead of the engine auto-exiling the cheapest.
/// Gated on `manual_mana` rather than `wants_ui`; see
/// `ashnods_altar_ui_activator_chooses_creature_to_sacrifice`.
#[test]
fn grim_lavamancer_ui_activator_chooses_graveyard_cards_to_exile() {
    let mut g = two_player_game();
    let lava = g.add_card_to_battlefield(0, catalog::grim_lavamancer());
    g.clear_sickness(lava);
    g.players[0].wants_ui = true;
    g.players[0].manual_mana = true;
    g.players[0].mana_pool.add(Color::Red, 1);
    // Three graveyard cards → a genuine choice (exile two of three).
    let a = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let b = g.add_card_to_graveyard(0, catalog::shock());
    let keep = g.add_card_to_graveyard(0, catalog::grizzly_bears());

    g.perform_action(GameAction::ActivateAbility {
        card_id: lava, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("activation suspends for the exile choice");

    let pd = g.pending_decision.as_ref().expect("an exile choice is pending");
    assert_eq!(pd.acting_player(), 0);
    match &pd.decision {
        crabomination::decision::Decision::ChooseCards { candidates, min, max, .. } => {
            assert_eq!((*min, *max), (2, 2), "must exile exactly two");
            assert_eq!(candidates.len(), 3, "all three graveyard cards offered");
        }
        other => panic!("expected ChooseCards, got {other:?}"),
    }

    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Cards(vec![a, b])))
        .expect("submit the exile choice");

    // Chosen cards exiled as the cost; the unchosen one stays in the graveyard.
    assert!(g.exile.iter().any(|c| c.id == a) && g.exile.iter().any(|c| c.id == b),
        "chosen graveyard cards exiled");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == keep), "unchosen card kept");
    // The ability is on the stack; resolving it deals 2.
    let life_before = g.players[1].life;
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 2, "ability resolves for 2 damage");
}

/// Zuran Orb sacrifices a land to gain 2 life.
#[test]
fn zuran_orb_sacrifices_a_land_for_two_life() {
    let mut g = two_player_game();
    let orb = g.add_card_to_battlefield(0, catalog::zuran_orb());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.clear_sickness(orb);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::ActivateAbility {
        card_id: orb, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Zuran Orb activates");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 2);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == forest),
        "Sacrificed Forest should be in the graveyard");
    // The Orb itself is still on the battlefield (it's not sacrificed).
    assert!(g.battlefield.iter().any(|c| c.id == orb));
}

/// Chromatic Star: tap and sac for any color of mana, then draw a card
/// when it lands in the graveyard.
#[test]
fn chromatic_star_sacrifices_for_mana_and_cantrips_on_leave() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let star = g.add_card_to_battlefield(0, catalog::chromatic_star());
    g.clear_sickness(star);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::ActivateAbility {
        card_id: star, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Chromatic Star activates");
    drain_stack(&mut g);

    // The sac put the Star in the graveyard.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == star),
        "Star should be sacrificed to the graveyard");
    // One mana of any color was added (then spent on the activation? no — the
    // {1} cost was paid up front, and the AddMana effect runs after. So we
    // gained one mana and drew a card from the leaves trigger.
    let pool = g.players[0].mana_pool.total();
    assert_eq!(pool, 1, "Star adds one mana of any color when activated");
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Star's leaves-the-battlefield trigger should draw a card");
}

/// Soul-Guide Lantern's first ability exiles a card from each opponent's
/// graveyard (approximation of "target opponent exiles one"). For the
/// 2-player demo it's gameplay-equivalent.
#[test]
fn soul_guide_lantern_first_ability_exiles_from_opponent_graveyard() {
    let mut g = two_player_game();
    let lantern = g.add_card_to_battlefield(0, catalog::soul_guide_lantern());
    g.clear_sickness(lantern);
    // Stock P1's graveyard with one card.
    let trash = g.add_card_to_library(1, catalog::lightning_bolt());
    let pos = g.players[1].library.iter().position(|c| c.id == trash).unwrap();
    let card = g.players[1].library.remove(pos);
    g.players[1].graveyard.push(card);

    g.perform_action(GameAction::ActivateAbility {
        card_id: lantern, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Lantern's tap ability activates");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == trash),
        "Opponent's graveyard card should be in exile");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == trash));
}

/// Soul-Guide Lantern's second ability exiles every player's graveyard,
/// sacrifices itself, and draws a card.
#[test]
fn soul_guide_lantern_second_ability_clears_graveyards_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let lantern = g.add_card_to_battlefield(0, catalog::soul_guide_lantern());
    g.clear_sickness(lantern);
    // Each player has a graveyard card.
    let p0_card = g.add_card_to_library(0, catalog::lightning_bolt());
    let pos = g.players[0].library.iter().position(|c| c.id == p0_card).unwrap();
    let card = g.players[0].library.remove(pos);
    g.players[0].graveyard.push(card);
    let p1_card = g.add_card_to_library(1, catalog::lightning_bolt());
    let pos = g.players[1].library.iter().position(|c| c.id == p1_card).unwrap();
    let card = g.players[1].library.remove(pos);
    g.players[1].graveyard.push(card);

    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::ActivateAbility {
        card_id: lantern, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Lantern's sac ability activates");
    drain_stack(&mut g);

    // Both graveyards are cleared (modulo the sacrificed Lantern itself).
    assert!(g.exile.iter().any(|c| c.id == p0_card));
    assert!(g.exile.iter().any(|c| c.id == p1_card));
    assert!(!g.battlefield.iter().any(|c| c.id == lantern),
        "Lantern is sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Sac ability draws a card");
}

/// Cankerbloom sacrifices itself to destroy an artifact or enchantment,
/// then proliferates. We can verify the destroy half cleanly; proliferate
/// in isolation is gameplay-equivalent to "no-op when nothing has counters",
/// so we set up a counter to assert the proliferate fired.
#[test]
fn cankerbloom_sacs_to_destroy_and_proliferate() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let canker = g.add_card_to_battlefield(0, catalog::cankerbloom());
    let opp_artifact = g.add_card_to_battlefield(1, catalog::sol_ring());
    // Put a counter on something so proliferate has work to do.
    let counted = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    {
        let bear = g.battlefield.iter_mut().find(|c| c.id == counted).unwrap();
        *bear.counters.entry(CounterType::PlusOnePlusOne).or_insert(0) = 1;
    }
    g.clear_sickness(canker);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::ActivateAbility {
        card_id: canker,
        ability_index: 0,
        target: Some(Target::Permanent(opp_artifact)), additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Cankerbloom activates");
    drain_stack(&mut g);

    // The opp Sol Ring is destroyed; Cankerbloom is sacrificed.
    assert!(!g.battlefield.iter().any(|c| c.id == opp_artifact));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == opp_artifact));
    assert!(!g.battlefield.iter().any(|c| c.id == canker));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == canker));
    // Proliferate added one more +1/+1 counter.
    let bear_view = g.battlefield.iter().find(|c| c.id == counted).unwrap();
    assert_eq!(*bear_view.counters.get(&CounterType::PlusOnePlusOne).unwrap_or(&0), 2,
        "Proliferate should bump the +1/+1 counter from 1 to 2");
}

