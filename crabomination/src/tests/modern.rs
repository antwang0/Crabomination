//! Functionality tests for the Modern-supplement card pack
//! (`catalog::sets::decks::modern`). Each card gets at least one test
//! exercising its primary play pattern; helpers from `tests/game.rs`
//! (`two_player_game`, `drain_stack`) are reused via `super::*` once this
//! file is registered alongside the existing test modules.

use crate::card::CardType;
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::*;
use crate::mana::Color;
use crate::player::Player;

fn two_player_game() -> GameState {
    let players = vec![Player::new(0, "Alice"), Player::new(1, "Bob")];
    let mut g = GameState::new(players);
    g.step = TurnStep::PreCombatMain;
    g
}

fn drain_stack(g: &mut GameState) {
    while !g.stack.is_empty() {
        g.perform_action(GameAction::PassPriority).unwrap();
        g.perform_action(GameAction::PassPriority).unwrap();
    }
}

// ── Cantrips ─────────────────────────────────────────────────────────────────

#[test]
fn ponder_resolves_and_draws_a_card() {
    let mut g = two_player_game();
    // Stock the library so the scry + draw both have inputs.
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::ponder());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Ponder should be castable for {U}");
    drain_stack(&mut g);

    // Ponder leaves the hand and ends in graveyard; the draw nets +1 hand.
    assert_eq!(g.players[0].hand.len(), hand_before, "cast (-1) + draw (+1) = net 0");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Ponder"));
}

#[test]
fn manamorphose_adds_two_mana_and_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::manamorphose());
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Manamorphose castable for {2}");
    drain_stack(&mut g);

    // Hand: -1 (cast) +1 (draw) → unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before);
    // Mana pool gained 2 mana of any colors. We don't constrain which colors
    // the bot picks; just that the total mana count went up by 2.
    let pool_total = g.players[0].mana_pool.total();
    assert_eq!(pool_total, 2, "Manamorphose should add 2 mana after spending {{2}} on its own cost");
}

#[test]
fn sleight_of_hand_cantrip_resolves() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::sleight_of_hand());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Sleight of Hand castable for {U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before, "cast + draw = net 0");
}

// ── Discard / draw-for-life / mill ───────────────────────────────────────────

#[test]
fn faithless_looting_draws_two_then_discards_two() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::faithless_looting());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    let grave_before = g.players[0].graveyard.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Faithless Looting castable for {R}");
    drain_stack(&mut g);

    // Net hand change: -1 cast, +2 draw, -2 discard = -1.
    assert_eq!(g.players[0].hand.len(), hand_before - 1,
        "Faithless Looting nets -1 hand size after cast/draw/discard");
    // Graveyard gains the spell + 2 discarded cards.
    assert_eq!(g.players[0].graveyard.len(), grave_before + 3);
}

#[test]
fn faithless_looting_has_flashback_keyword() {
    use crate::card::Keyword;
    let card = catalog::faithless_looting();
    assert!(
        card.keywords
            .iter()
            .any(|k| matches!(k, Keyword::Flashback(_))),
        "Faithless Looting should have Flashback"
    );
}

#[test]
fn flashback_cast_exiles_spell_on_resolution() {
    // A flashback-cast spell is exiled on resolution (not sent to the
    // graveyard). Faithless Looting flashback {2}{R}: discard a card from
    // graveyard via the path; the card should end up in exile, not
    // graveyard.
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    // Put Faithless Looting straight into graveyard.
    let id = g.add_card_to_library(0, catalog::faithless_looting());
    let pos = g.players[0].library.iter().position(|c| c.id == id).unwrap();
    let card = g.players[0].library.remove(pos);
    g.players[0].graveyard.push(card);

    // Pay the flashback cost {2}{R}.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastFlashback {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Flashback castable for {2}{R} from graveyard");
    drain_stack(&mut g);

    // The card is in exile (not in graveyard).
    assert!(g.exile.iter().any(|c| c.id == id),
        "Flashback-cast Faithless Looting should be exiled on resolution");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == id),
        "Flashback-cast spell must NOT return to the graveyard");
}

#[test]
fn sign_in_blood_draws_two_loses_two_life() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::sign_in_blood());
    g.players[0].mana_pool.add(Color::Black, 2);
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(0)),
        mode: None,
        x_value: None,
    })
    .expect("Sign in Blood castable for {B}{B}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before - 2);
    // Hand: -1 cast +2 draw = +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

/// Sign in Blood is now real-Oracle "target player". Targeting the opponent
/// makes them draw the cards and pay the life — a powerful sideboard line
/// against a low-life opponent. Verifies the new `target_filter(Player)`
/// path threads the targeted player into both `Draw` and `LoseLife`.
#[test]
fn sign_in_blood_can_target_opponent() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::sign_in_blood());
    g.players[0].mana_pool.add(Color::Black, 2);
    let p0_life_before = g.players[0].life;
    let p1_life_before = g.players[1].life;
    let p0_hand_before = g.players[0].hand.len();
    let p1_hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        mode: None,
        x_value: None,
    })
    .expect("Sign in Blood castable for {B}{B}");
    drain_stack(&mut g);

    // Caster (P0) life unchanged; target (P1) lost 2.
    assert_eq!(g.players[0].life, p0_life_before);
    assert_eq!(g.players[1].life, p1_life_before - 2);
    // P0 lost the cast card from hand; P1 drew 2.
    assert_eq!(g.players[0].hand.len(), p0_hand_before - 1);
    assert_eq!(g.players[1].hand.len(), p1_hand_before + 2);
}

#[test]
fn nights_whisper_draws_two_loses_two_life() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::nights_whisper());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Night's Whisper castable for {1}{B}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before - 2);
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn duress_picks_noncreature_nonland() {
    let mut g = two_player_game();
    // P1 hand: a creature, a land, and a noncreature/nonland sorcery.
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::forest());
    let target_card = g.add_card_to_hand(1, catalog::lightning_bolt());

    let id = g.add_card_to_hand(0, catalog::duress());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Duress castable for {B}");
    drain_stack(&mut g);

    // The auto-discarder should have plucked the noncreature/nonland (the
    // Lightning Bolt, since the creature and land are filtered out).
    assert!(g.players[1].graveyard.iter().any(|c| c.id == target_card),
        "Duress should discard the noncreature/nonland card");
}

// ── Burn ─────────────────────────────────────────────────────────────────────

#[test]
fn lava_spike_deals_three_damage_to_player() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lava_spike());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        mode: None,
        x_value: None,
    })
    .expect("Lava Spike castable for {R}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life_before - 3,
        "Lava Spike should deal 3 damage to the targeted player");
}

#[test]
fn shock_deals_two_damage_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::shock());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: None,
    })
    .expect("Shock castable for {R}");
    drain_stack(&mut g);

    // Bear has 2 toughness; 2 damage kills it (state-based actions move it
    // to graveyard).
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be destroyed by Shock");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
}

#[test]
fn lava_dart_deals_one_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lava_dart());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        mode: None,
        x_value: None,
    })
    .expect("Lava Dart castable for {R}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life_before - 1);
}

// ── Reanimation / graveyard ──────────────────────────────────────────────────

#[test]
fn unburial_rites_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    // Put Atraxa in P0's graveyard for the rites to grab.
    let atraxa = g.add_card_to_library(0, catalog::atraxa_grand_unifier());
    let pos = g.players[0].library.iter().position(|c| c.id == atraxa).unwrap();
    let card = g.players[0].library.remove(pos);
    g.players[0].graveyard.push(card);

    let rites = g.add_card_to_hand(0, catalog::unburial_rites());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: rites,
        target: Some(Target::Permanent(atraxa)),
        mode: None,
        x_value: None,
    })
    .expect("Unburial Rites castable for {3}{B}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == atraxa),
        "Atraxa should be reanimated onto the battlefield");
}

#[test]
fn entomb_searches_library_into_graveyard() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bolt))]));

    let id = g.add_card_to_hand(0, catalog::entomb());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Entomb castable for {B}");
    drain_stack(&mut g);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt),
        "Entomb should pull a card from library to graveyard");
    assert!(!g.players[0].library.iter().any(|c| c.id == bolt));
}

#[test]
fn buried_alive_searches_creature_into_graveyard() {
    let mut g = two_player_game();
    let creature = g.add_card_to_library(0, catalog::grizzly_bears());
    // Buried Alive now repeats the search up to 3 times. Answer the first
    // pull with the creature, then `Search(None)` to opt out of the
    // remaining iterations.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(creature)),
        DecisionAnswer::Search(None),
        DecisionAnswer::Search(None),
    ]));

    let id = g.add_card_to_hand(0, catalog::buried_alive());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Buried Alive castable for {2}{B}");
    drain_stack(&mut g);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == creature),
        "Buried Alive should pull a creature card into the graveyard");
}

/// Buried Alive's full Oracle is "search for up to three creature cards" —
/// the engine wires that as `Repeat(3, Search(...))`. Stocking three
/// creatures in the library and answering each pull with a different one
/// should land all three in the graveyard.
#[test]
fn buried_alive_pulls_up_to_three_creatures() {
    let mut g = two_player_game();
    let c1 = g.add_card_to_library(0, catalog::grizzly_bears());
    let c2 = g.add_card_to_library(0, catalog::grizzly_bears());
    let c3 = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(c1)),
        DecisionAnswer::Search(Some(c2)),
        DecisionAnswer::Search(Some(c3)),
    ]));

    let id = g.add_card_to_hand(0, catalog::buried_alive());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Buried Alive castable for {2}{B}");
    drain_stack(&mut g);

    for cid in [c1, c2, c3] {
        assert!(g.players[0].graveyard.iter().any(|c| c.id == cid),
            "All three searched creatures should be in the graveyard");
    }
}

#[test]
fn exhume_reanimates_creature() {
    let mut g = two_player_game();
    let creature = g.add_card_to_library(0, catalog::grizzly_bears());
    let pos = g.players[0].library.iter().position(|c| c.id == creature).unwrap();
    let card = g.players[0].library.remove(pos);
    g.players[0].graveyard.push(card);

    let id = g.add_card_to_hand(0, catalog::exhume());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(creature)),
        mode: None,
        x_value: None,
    })
    .expect("Exhume castable for {1}{B}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == creature),
        "Exhume should reanimate the targeted creature");
}

// ── Creatures ────────────────────────────────────────────────────────────────

#[test]
fn burning_tree_emissary_etb_adds_red_and_green() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::burning_tree_emissary());
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Burning-Tree Emissary castable for {2}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == id));
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
}

#[test]
fn putrid_imp_discard_grants_menace_eot() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let imp = g.add_card_to_battlefield(0, catalog::putrid_imp());
    g.clear_sickness(imp);
    let to_pitch = g.add_card_to_hand(0, catalog::lightning_bolt());

    g.perform_action(GameAction::ActivateAbility {
        card_id: imp, ability_index: 0, target: None,
    })
    .expect("Putrid Imp discard ability activates");
    drain_stack(&mut g);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == to_pitch),
        "Discarded card should hit graveyard");
    let computed = g.compute_battlefield();
    let imp_view = computed.iter().find(|c| c.id == imp).unwrap();
    assert!(imp_view.keywords.contains(&Keyword::Menace),
        "Putrid Imp should have menace until end of turn");
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
fn veil_of_summer_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::veil_of_summer());
    g.players[0].mana_pool.add(Color::Green, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Veil of Summer castable for {G}");
    drain_stack(&mut g);

    // Net hand: -1 cast +1 draw = 0
    assert_eq!(g.players[0].hand.len(), hand_before);
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
        card_id: id, target: None, mode: None, x_value: None,
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

#[test]
fn karakas_taps_for_white() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::karakas());

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None,
    })
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
        target: Some(Target::Permanent(atraxa)),
    })
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

#[test]
fn modern_card_factories_produce_valid_definitions() {
    // Smoke test: every modern card should have at least one card type and
    // a non-empty name.
    let cards: Vec<crate::card::CardDefinition> = vec![
        catalog::ponder(), catalog::manamorphose(), catalog::sleight_of_hand(),
        catalog::faithless_looting(), catalog::sign_in_blood(),
        catalog::nights_whisper(), catalog::duress(), catalog::lava_spike(),
        catalog::lava_dart(), catalog::shock(), catalog::unburial_rites(),
        catalog::exhume(), catalog::buried_alive(), catalog::entomb(),
        catalog::burning_tree_emissary(), catalog::putrid_imp(),
        catalog::tarmogoyf(), catalog::veil_of_summer(),
        catalog::crop_rotation(), catalog::karakas(), catalog::bojuka_bog(),
    ];

    for card in &cards {
        assert!(!card.name.is_empty(), "card name empty");
        assert!(!card.card_types.is_empty(), "{} has no card types", card.name);
        // Lands have CardType::Land; everything else has a cast cost or an
        // alt cost.
        if !card.card_types.contains(&CardType::Land) {
            let has_cost = !card.cost.symbols.is_empty() || card.alternative_cost.is_some();
            assert!(has_cost, "{} should have a cast or alt cost", card.name);
        }
    }
}

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
        mode: None,
        x_value: None,
    })
    .expect("Blossoming Defense castable for {G}");
    drain_stack(&mut g);

    let computed = g.computed_permanent(bear).unwrap();
    assert_eq!(computed.power, 4);
    assert_eq!(computed.toughness, 4);
    assert!(computed.keywords.contains(&crate::card::Keyword::Hexproof));
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
        mode: None,
        x_value: None,
    })
    .expect("Anger castable for {1}{R}{R}");
    drain_stack(&mut g);
    for cid in [b0, b1, lion] {
        assert!(!g.battlefield.iter().any(|c| c.id == cid));
    }
}

#[test]
fn blasphemous_act_kills_each_creature() {
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(0, catalog::shivan_dragon());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let act = g.add_card_to_hand(0, catalog::blasphemous_act());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: act,
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Blasphemous Act castable for {4}{R}");
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
        target: Some(Target::Player(0)),
    });
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
fn breeding_pool_is_a_forest_island_dual() {
    let def = catalog::breeding_pool();
    let lts = &def.subtypes.land_types;
    assert!(lts.contains(&crate::card::LandType::Forest));
    assert!(lts.contains(&crate::card::LandType::Island));
}

#[test]
fn steam_vents_carries_island_and_mountain_typing() {
    let def = catalog::steam_vents();
    let lts = &def.subtypes.land_types;
    assert!(lts.contains(&crate::card::LandType::Island));
    assert!(lts.contains(&crate::card::LandType::Mountain));
}

#[test]
fn stomping_ground_carries_mountain_and_forest_typing() {
    let def = catalog::stomping_ground();
    let lts = &def.subtypes.land_types;
    assert!(lts.contains(&crate::card::LandType::Mountain));
    assert!(lts.contains(&crate::card::LandType::Forest));
}

#[test]
fn temple_garden_carries_forest_and_plains_typing() {
    let def = catalog::temple_garden();
    let lts = &def.subtypes.land_types;
    assert!(lts.contains(&crate::card::LandType::Forest));
    assert!(lts.contains(&crate::card::LandType::Plains));
}

#[test]
fn blood_crypt_carries_swamp_and_mountain_typing() {
    let def = catalog::blood_crypt();
    let lts = &def.subtypes.land_types;
    assert!(lts.contains(&crate::card::LandType::Swamp));
    assert!(lts.contains(&crate::card::LandType::Mountain));
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
        card_id: wrath, target: None, mode: None, x_value: None,
    })
    .unwrap();

    let dispel = g.add_card_to_hand(0, catalog::dispel());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    let err = g
        .perform_action(GameAction::CastSpell {
            card_id: dispel,
            target: Some(Target::Permanent(wrath)),
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
fn thalia_taxes_noncreature_spells_after_first() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::thalia_guardian_of_thraben());

    // First Bolt this turn pays no tax — passes for {R}.
    let bolt1 = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt1,
        target: Some(Target::Player(1)),
        mode: None,
        x_value: None,
    })
    .expect("first noncreature spell shouldn't pay Thalia tax");
    drain_stack(&mut g);

    // Second Bolt now requires {1}{R}; only {R} fails.
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let err = g
        .perform_action(GameAction::CastSpell {
            card_id: bolt2,
            target: Some(Target::Player(1)),
            mode: None,
            x_value: None,
        })
        .unwrap_err();
    assert!(matches!(err, GameError::Mana(_)));
}

#[test]
fn dark_confidant_definition_has_upkeep_trigger() {
    use crate::card::{EventKind, EventScope};
    let def = catalog::dark_confidant();
    assert!(def.triggered_abilities.iter().any(|t| {
        matches!(t.event.kind, EventKind::StepBegins(crate::game::TurnStep::Upkeep))
            && matches!(t.event.scope, EventScope::YourControl)
    }));
}

#[test]
fn bloodghast_ships_with_haste() {
    use crate::card::Keyword;
    let def = catalog::bloodghast();
    assert!(def.keywords.contains(&Keyword::Haste));
}

#[test]
fn phyrexian_arena_draws_card_and_loses_life_at_upkeep() {
    let mut g = two_player_game();
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
fn tarfire_deals_two_damage_to_player() {
    let mut g = two_player_game();
    let tarfire = g.add_card_to_hand(0, catalog::tarfire());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: tarfire,
        target: Some(Target::Player(1)),
        mode: None,
        x_value: None,
    })
    .expect("Tarfire castable for {R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
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
        mode: None,
        x_value: None,
    })
    .expect("Frantic Search castable for {2}{U}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().find(|c| c.id == l1).unwrap().tapped);
    assert!(!g.battlefield.iter().find(|c| c.id == l2).unwrap().tapped);
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
        mode: None,
        x_value: None,
    })
    .expect("Spell Snare castable for {U}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bears));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bears));
}

#[test]
fn disentomb_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    // Drop the bear directly into the graveyard.
    let card = g.players[0].hand.pop().unwrap();
    g.players[0].graveyard.push(card);
    let _ = bear;
    let disentomb = g.add_card_to_hand(0, catalog::disentomb());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: disentomb,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: None,
    })
    .expect("Disentomb castable for {B}");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bear));
}

#[test]
fn vandalblast_destroys_opponent_artifact() {
    let mut g = two_player_game();
    let opp_ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let mine_ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let vand = g.add_card_to_hand(0, catalog::vandalblast());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: vand,
        target: Some(Target::Permanent(opp_ring)),
        mode: None,
        x_value: None,
    })
    .expect("Vandalblast castable for {R}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == opp_ring));
    assert!(g.battlefield.iter().any(|c| c.id == mine_ring), "your own artifact untouched");
}

#[test]
fn natures_lore_fetches_a_forest_to_battlefield_untapped() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    // AutoDecider declines `SearchLibrary` (returns `Search(None)`); a
    // scripted decider picks the matching land so Nature's Lore actually
    // resolves end-to-end.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let lore = g.add_card_to_hand(0, catalog::natures_lore());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: lore,
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Nature's Lore castable for {1}{G}");
    drain_stack(&mut g);
    let on_bf = g.battlefield.iter().find(|c| c.id == forest);
    assert!(on_bf.is_some(), "Forest should land on battlefield");
    assert!(!on_bf.unwrap().tapped, "Nature's Lore enters untapped");
}

#[test]
fn sylvan_caryatid_taps_for_one_mana_of_chosen_color() {
    let mut g = two_player_game();
    let caryatid = g.add_card_to_battlefield(0, catalog::sylvan_caryatid());
    g.clear_sickness(caryatid);
    // Scripted decider answers ChooseColor with Black so we can assert
    // the chosen pip lands in the right pool slot.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Black)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: caryatid,
        ability_index: 0,
        target: None,
    })
    .expect("Caryatid mana ability activates");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1);
}

#[test]
fn millstone_mills_target_for_two() {
    let mut g = two_player_game();
    g.add_card_to_library(1, catalog::forest());
    g.add_card_to_library(1, catalog::mountain());
    g.add_card_to_library(1, catalog::island());
    let stone = g.add_card_to_battlefield(0, catalog::millstone());
    g.clear_sickness(stone);
    g.players[0].mana_pool.add_colorless(2);
    let opp_lib_before = g.players[1].library.len();
    let opp_yard_before = g.players[1].graveyard.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: stone,
        ability_index: 0,
        target: Some(Target::Player(1)),
    })
    .expect("Millstone activates for {2}{T}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), opp_lib_before - 2);
    assert_eq!(g.players[1].graveyard.len(), opp_yard_before + 2);
}

#[test]
fn ornithopter_is_a_zero_cost_flying_creature() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::ornithopter());
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Ornithopter castable for free");
    drain_stack(&mut g);
    let bf = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!(bf.power(), 0);
    assert_eq!(bf.toughness(), 2);
    assert!(bf.has_keyword(&crate::card::Keyword::Flying));
}

#[test]
fn ornithopter_of_paradise_taps_for_any_one_color() {
    let mut g = two_player_game();
    let bird = g.add_card_to_battlefield(0, catalog::ornithopter_of_paradise());
    g.clear_sickness(bird);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: bird,
        ability_index: 0,
        target: None,
    })
    .expect("Ornithopter of Paradise mana ability activates");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
}

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
        mode: None, x_value: None,
    })
    .expect("Daze castable");
    drain_stack(&mut g);
    // Bolt countered → seat 0 takes no damage, Bolt in graveyard.
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
        card_id: eid, target: None, mode: None, x_value: None,
    })
    .expect("Eidolon castable");
    let swan = g.add_card_to_hand(0, catalog::swan_song());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: swan,
        target: Some(Target::Permanent(eid)),
        mode: None, x_value: None,
    })
    .expect("Swan Song castable");
    drain_stack(&mut g);
    // Eidolon countered.
    assert!(g.players[1].graveyard.iter().any(|c| c.id == eid));
    // Bird token created (under EachOpponent of caster — i.e. seat 1).
    assert_eq!(g.battlefield.len(), bf_before + 1);
    let token = g.battlefield.last().unwrap();
    assert_eq!(token.definition.name, "Bird");
    assert_eq!(token.controller, 1);
    assert!(token.has_keyword(&crate::card::Keyword::Flying));
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
        card_id: outcome, target: None, mode: None, x_value: None,
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
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: edict, target: None, mode: None, x_value: None,
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
        card_id: big, target: None, mode: None, x_value: None,
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
        card_id: angel, target: None, mode: None, x_value: None,
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
        card_id: wisp, target: None, mode: None, x_value: None,
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
        target: None,
    })
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

#[test]
fn ancient_den_taps_for_white_and_is_an_artifact() {
    let mut g = two_player_game();
    let den = g.add_card_to_battlefield(0, catalog::ancient_den());
    g.clear_sickness(den);
    g.perform_action(GameAction::ActivateAbility {
        card_id: den,
        ability_index: 0,
        target: None,
    })
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
        card_id: epi, target: None, mode: None, x_value: None,
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
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"));
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

#[test]
fn tireless_tracker_does_not_trigger_on_non_land_etb() {
    // Casting a creature shouldn't fire Tracker's land filter.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tireless_tracker());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, mode: None, x_value: None,
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
        card_id: harv, target: None, mode: None, x_value: None,
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
        card_id: beanstalk, target: None, mode: None, x_value: None,
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
        card_id: angel, target: None, mode: None, x_value: None,
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
        card_id: elves, target: None, mode: None, x_value: None,
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
        card_id: angel, target: None, mode: None, x_value: None,
    })
    .expect("Serra Angel castable");
    drain_stack(&mut g);
    // Net: cast (-1) + Temur draw (+1) = 0.
    assert_eq!(g.players[0].hand.len(), angel_hand_before,
        "power-4 ETB SHOULD trigger Temur draw");
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
        card_id: big, target: None, mode: None, x_value: None,
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
        target: None,
    })
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
        target: None,
    })
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
        target: Some(Target::Permanent(opp_ring)),
    })
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
        target: Some(Target::Permanent(opp_ring)),
    })
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
        target: Some(Target::Permanent(bear)),
    })
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
        target: None,
    });
    assert!(err.is_err(), "Mind Stone sac-for-draw should fail without mana");
    // Source still on battlefield, untapped, hand unchanged.
    assert!(g.battlefield.iter().any(|c| c.id == stone));
    let on_bf = g.battlefield.iter().find(|c| c.id == stone).unwrap();
    assert!(!on_bf.tapped, "tap-cost should roll back when mana pay fails");
}

// ── Cube cards (round 4) ─────────────────────────────────────────────────────

#[test]
fn sentinel_attack_creates_a_citizen_token() {
    let mut g = two_player_game();
    let sentinel = g.add_card_to_battlefield(0, catalog::sentinel_of_the_nameless_city());
    g.clear_sickness(sentinel);
    // Move to declare-attackers and declare an attack.
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sentinel,
        target: AttackTarget::Player(1),
    }]))
    .expect("Sentinel can attack");
    drain_stack(&mut g);
    // Token created.
    assert_eq!(g.battlefield.len(), bf_before + 1);
    let token = g.battlefield.iter().find(|c| c.definition.name == "Citizen");
    assert!(token.is_some(), "Citizen token created from attack trigger");
    let token = token.unwrap();
    assert!(token.is_token);
    assert_eq!(token.power(), 1);
    assert_eq!(token.toughness(), 1);
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
        card_id: ranger, target: None, mode: None, x_value: None,
    })
    .expect("Ranger-Captain castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == elves), "Llanowar Elves to hand");
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
        card_id: upheaval, target: None, mode: None, x_value: None,
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
fn rakshasas_bargain_pays_4_life_and_draws_4() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let bargain = g.add_card_to_hand(0, catalog::rakshasas_bargain());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bargain, target: None, mode: None, x_value: None,
    })
    .expect("Rakshasa's Bargain castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before - 4);
    // Net hand: cast (-1) + draw 4 (+4) = +3.
    assert_eq!(g.players[0].hand.len(), hand_before + 3);
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
    assert!(card.definition.subtypes.land_types.contains(&crate::card::LandType::Mountain));
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
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: loran, target: None, mode: None, x_value: None,
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
        target: Some(Target::Player(1)),
    })
    .expect("Loran tap ability activates");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), p0_hand + 1);
    assert_eq!(g.players[1].hand.len(), p1_hand + 1);
}
