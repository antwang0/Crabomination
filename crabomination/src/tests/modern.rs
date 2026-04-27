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
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Sign in Blood castable for {B}{B}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before - 2);
    // Hand: -1 cast +2 draw = +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
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
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(creature))]));

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
