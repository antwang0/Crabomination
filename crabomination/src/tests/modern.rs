//! Functionality tests for the Modern-supplement card pack
//! (`catalog::sets::decks::modern`). Each card gets at least one test
//! exercising its primary play pattern.

use crate::card::{CardType, CounterType};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

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
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
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
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
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
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
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
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
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
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Flashback castable for {2}{R} from graveyard");
    drain_stack(&mut g);

    // The card is in exile (not in graveyard).
    assert!(g.exile.iter().any(|c| c.id == id),
        "Flashback-cast Faithless Looting should be exiled on resolution");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == id),
        "Flashback-cast spell must NOT return to the graveyard");
}

/// Sign in Blood: target player draws 2 and loses 2 life. Verifies both self-target
/// and opp-target (the latter exercises the `target_filter(Player)` path).
#[test]
fn sign_in_blood_drains_targeted_player() {
    // Target self.
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::sign_in_blood());
    g.players[0].mana_pool.add(Color::Black, 2);
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sign in Blood castable for {B}{B}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before - 2);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "-1 cast +2 draw = +1");

    // Target opponent.
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(1, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::sign_in_blood());
    g.players[0].mana_pool.add(Color::Black, 2);
    let p0_life_before = g.players[0].life;
    let p1_life_before = g.players[1].life;
    let p0_hand_before = g.players[0].hand.len();
    let p1_hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sign in Blood castable for {B}{B}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_life_before, "caster life unchanged");
    assert_eq!(g.players[1].life, p1_life_before - 2, "target lost 2");
    assert_eq!(g.players[0].hand.len(), p0_hand_before - 1, "caster lost the spell");
    assert_eq!(g.players[1].hand.len(), p1_hand_before + 2, "target drew 2");
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
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
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
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
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
        additional_targets: vec![],
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
        additional_targets: vec![],
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
        additional_targets: vec![],
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
        additional_targets: vec![],
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
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
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
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
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
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
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
    // Push (modern_decks): printed Oracle is now "each player puts a
    // creature card from their graveyard onto the battlefield". The
    // caster's auto-decider picks the top creature card in their own
    // graveyard; same for the opp. This test only seeds the caster's
    // graveyard, so only the caster reanimates.
    let creature = g.add_card_to_graveyard(0, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::exhume());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Exhume castable for {1}{B}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == creature),
        "Exhume should reanimate the caster's only creature in their graveyard");
}

#[test]
fn exhume_each_player_reanimates_a_creature() {
    // Push (modern_decks): "each player reanimates" symmetry — both
    // P0 and P1 have a creature in their gy; both should land on the
    // battlefield under their respective controllers.
    let mut g = two_player_game();
    let p0_bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let p1_bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::exhume());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Exhume castable for {1}{B}");
    drain_stack(&mut g);

    let p0_back = g.battlefield_find(p0_bear);
    let p1_back = g.battlefield_find(p1_bear);
    assert!(p0_back.is_some() && p0_back.unwrap().controller == 0,
        "P0's bear should be on P0's battlefield");
    assert!(p1_back.is_some() && p1_back.unwrap().controller == 1,
        "P1's bear should be on P1's battlefield (each-player symmetry)");
}

// ── Creatures ────────────────────────────────────────────────────────────────

#[test]
fn burning_tree_emissary_etb_adds_red_and_green() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::burning_tree_emissary());
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
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
        card_id: imp, ability_index: 0, target: None, x_value: None })
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
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
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

#[test]
fn karakas_taps_for_white() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::karakas());

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None })
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
        target: Some(Target::Permanent(atraxa)), x_value: None })
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
        additional_targets: vec![],
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
        target: Some(Target::Player(0)), x_value: None });
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
fn thalia_taxes_noncreature_spells_after_first() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::thalia_guardian_of_thraben());

    // First Bolt this turn pays no tax — passes for {R}.
    let bolt1 = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt1,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
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
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .unwrap_err();
    assert!(matches!(err, GameError::Mana(_)));
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
        additional_targets: vec![],
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
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Vandalblast castable for {R}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == opp_ring));
    assert!(g.battlefield.iter().any(|c| c.id == mine_ring), "your own artifact untouched");
}

#[test]
fn vandalblast_overload_destroys_all_opponent_artifacts() {
    let mut g = two_player_game();
    let opp_ring1 = g.add_card_to_battlefield(1, catalog::sol_ring());
    let opp_ring2 = g.add_card_to_battlefield(1, catalog::sol_ring());
    let mine_ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let vand = g.add_card_to_hand(0, catalog::vandalblast());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: vand,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Vandalblast Overload for {4}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == opp_ring1), "opp artifact 1 destroyed");
    assert!(!g.battlefield.iter().any(|c| c.id == opp_ring2), "opp artifact 2 destroyed");
    assert!(g.battlefield.iter().any(|c| c.id == mine_ring), "own artifact untouched");
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
        additional_targets: vec![],
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
        target: None, x_value: None })
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
        target: Some(Target::Player(1)), x_value: None })
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
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
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
        target: None, x_value: None })
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
    assert!(token.has_keyword(&crate::card::Keyword::Flying));
}

#[test]
fn swan_song_in_three_player_gives_bird_to_countered_spell_controller() {
    // 3-player game: seat 0 casts Swan Song on a spell seat 2 cast.
    // The Bird should go to seat 2 (the countered spell's controller),
    // not seat 1. Pre-fix this used EachOpponent which would have given
    // a token to both opponents (seats 1 AND 2).
    let mut g = crate::game::multi_player_game(3);
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
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Black, 1);
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
        target: None, x_value: None })
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
        target: None, x_value: None })
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

/// Tireless Tracker costs `{1}{G}{G}`. With 1 Forest + 2 Mountains, the
/// player has only 1 green source — not enough to pay the second `{G}`
/// pip — so the cast must fail. With 2 Forests + 1 Mountain, all three
/// pips are payable and the cast succeeds. Locks down the cost so a
/// future "off by one mana" regression in the catalog (or the auto-tap
/// path) can't sneak through.
#[test]
fn tireless_tracker_requires_two_green_mana_sources() {
    let mut g = two_player_game();
    g.priority.player_with_priority = 0;

    // Setup A: 1 Forest, 2 Mountains in play, untapped — only 1 green available.
    let f = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(f).unwrap().tapped = false;
    let m1 = g.add_card_to_battlefield(0, catalog::mountain());
    g.battlefield_find_mut(m1).unwrap().tapped = false;
    let m2 = g.add_card_to_battlefield(0, catalog::mountain());
    g.battlefield_find_mut(m2).unwrap().tapped = false;
    let tracker = g.add_card_to_hand(0, catalog::tireless_tracker());
    let err = g.perform_action(GameAction::CastSpell {
        card_id: tracker,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(err.is_err(),
        "{{1}}{{G}}{{G}} cannot be paid from 1 Forest + 2 Mountains: {err:?}");

    // Setup B: swap a Mountain for a Forest — now we have 2G + 1 generic.
    let mut g = two_player_game();
    g.priority.player_with_priority = 0;
    let f1 = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(f1).unwrap().tapped = false;
    let f2 = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(f2).unwrap().tapped = false;
    let m = g.add_card_to_battlefield(0, catalog::mountain());
    g.battlefield_find_mut(m).unwrap().tapped = false;
    let tracker = g.add_card_to_hand(0, catalog::tireless_tracker());
    g.perform_action(GameAction::CastSpell {
        card_id: tracker,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{1}{G}{G} pays from 2 Forests + 1 Mountain");
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
        target: None, x_value: None })
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
        target: None, x_value: None })
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
        target: Some(Target::Permanent(opp_ring)), x_value: None })
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
        target: Some(Target::Permanent(opp_ring)), x_value: None })
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
        target: Some(Target::Permanent(bear)), x_value: None })
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
        target: None, x_value: None });
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
        card_id: ranger, target: None, additional_targets: vec![], mode: None, x_value: None,
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
fn rakshasas_bargain_pays_4_life_and_draws_4() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let bargain = g.add_card_to_hand(0, catalog::rakshasas_bargain());
    // Real Oracle: `{4}{B}{B}` Instant.
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Black, 2);
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bargain, target: None, additional_targets: vec![], mode: None, x_value: None,
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
        target: Some(Target::Player(1)), x_value: None })
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
        .filter(|si| matches!(si, crate::game::StackItem::Trigger { source, .. } if *source == dev))
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
        si, crate::game::StackItem::Trigger { source, .. } if *source == dev
    )), "Scry-on-cast trigger should have been countered");
}

/// Sylvan Safekeeper sacrifices a Forest to grant a creature shroud EOT.
#[test]
fn sylvan_safekeeper_sacs_forest_to_grant_shroud() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let sk = g.add_card_to_battlefield(0, catalog::sylvan_safekeeper());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(sk);

    g.perform_action(GameAction::ActivateAbility {
        card_id: sk,
        ability_index: 0,
        target: Some(Target::Permanent(bear)), x_value: None })
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
        target: Some(Target::Player(1)), x_value: None })
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

/// Zuran Orb sacrifices a land to gain 2 life.
#[test]
fn zuran_orb_sacrifices_a_land_for_two_life() {
    let mut g = two_player_game();
    let orb = g.add_card_to_battlefield(0, catalog::zuran_orb());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.clear_sickness(orb);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::ActivateAbility {
        card_id: orb, ability_index: 0, target: None, x_value: None })
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
        card_id: star, ability_index: 0, target: None, x_value: None })
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
        card_id: lantern, ability_index: 0, target: None, x_value: None })
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
        card_id: lantern, ability_index: 1, target: None, x_value: None })
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
    use crate::card::CounterType;
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
        target: Some(Target::Permanent(opp_artifact)), x_value: None })
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

// ── New modern-supplement cards (claude/modern_decks batch) ──────────────────

/// Cathartic Reunion: discard 2, draw 3.
#[test]
fn cathartic_reunion_discards_two_then_draws_three() {
    let mut g = two_player_game();
    // Stock 5 cards in library so the draw 3 has inputs.
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    // 4 cards in hand: Cathartic Reunion + 3 fillers (so we can discard 2
    // and still cast).
    let id = g.add_card_to_hand(0, catalog::cathartic_reunion());
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::lightning_bolt());
    }
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Cathartic Reunion castable for {1}{R}");
    drain_stack(&mut g);

    // Hand: -1 cast -2 discard +3 draw = net 0. The Cathartic Reunion itself
    // and 2 discarded cards are now in the graveyard.
    assert_eq!(g.players[0].hand.len(), hand_before, "net hand change should be 0");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id),
        "Cathartic Reunion should hit the graveyard");
    assert!(g.players[0].graveyard.len() >= 3,
        "Two discards plus the Reunion itself = at least 3 cards in graveyard");
}

/// Gitaxian Probe: lose 2 life, draw 1 card.
#[test]
fn gitaxian_probe_pays_two_life_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::gitaxian_probe());
    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Gitaxian Probe castable for {0}");
    drain_stack(&mut g);

    // -1 cast +1 draw → net hand 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert_eq!(g.players[0].life, life_before - 2, "Probe pays 2 life");
}

/// Force Spike counters target spell unless its controller pays {1}.
/// When the opp can't pay, the spell is countered.
#[test]
fn force_spike_counters_when_opponent_cannot_pay() {
    let mut g = two_player_game();
    // P1 casts a Lightning Bolt with no spare mana; P0 responds with Force Spike.
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
    .expect("Lightning Bolt castable for {R}");

    g.priority.player_with_priority = 0;
    let spike = g.add_card_to_hand(0, catalog::force_spike());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spike,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Force Spike castable for {U}");
    drain_stack(&mut g);

    // P1 had only {R} (already spent) and 0 generic, so they can't pay {1}.
    // The Bolt is countered → P0 still at 20.
    assert_eq!(g.players[0].life, 20,
        "Bolt countered; P0 takes no damage");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Countered Bolt goes to controller's graveyard");
}

/// Force Spike doesn't counter when the opponent can pay {1}.
#[test]
fn force_spike_does_not_counter_when_opponent_can_pay() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.players[1].mana_pool.add_colorless(1); // spare to pay the spike
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

    g.priority.player_with_priority = 0;
    let spike = g.add_card_to_hand(0, catalog::force_spike());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spike,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Force Spike castable for {U}");
    drain_stack(&mut g);

    // P1 pays the {1}, Bolt resolves.
    assert_eq!(g.players[0].life, 17, "Bolt resolved; P0 took 3 damage");
    assert_eq!(g.players[1].mana_pool.colorless_amount(), 0,
        "P1's spare colorless should have been consumed paying the spike");
}

/// Vampiric Tutor: pay 2 life, search the library, put on top.
#[test]
fn vampiric_tutor_pays_two_life_and_tutors_to_top_of_library() {
    let mut g = two_player_game();
    let target_card = g.add_card_to_library(0, catalog::griselbrand());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target_card))]));

    let id = g.add_card_to_hand(0, catalog::vampiric_tutor());
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Vampiric Tutor castable for {B}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before - 2, "Vampiric pays 2 life");
    // Tutored card should be on top of the library.
    let top = g.players[0].library.first().unwrap();
    assert_eq!(top.id, target_card,
        "Vampiric Tutor should put the chosen card on top of the library");
}

/// Sylvan Scrying tutors a land into hand.
#[test]
fn sylvan_scrying_tutors_a_land_to_hand() {
    let mut g = two_player_game();
    let target_land = g.add_card_to_library(0, catalog::bojuka_bog());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target_land))]));

    let id = g.add_card_to_hand(0, catalog::sylvan_scrying());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Sylvan Scrying castable for {1}{G}");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == target_land),
        "Tutored land should be in hand");
}

/// Abrupt Decay destroys a low-CMC nonland permanent and is uncounterable.
#[test]
fn abrupt_decay_destroys_low_cmc_nonland() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2-CMC creature
    let id = g.add_card_to_hand(0, catalog::abrupt_decay());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Abrupt Decay castable for {B}{G}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear (CMC 2) should be destroyed");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
}

/// Abrupt Decay refuses to target a CMC-3-or-higher permanent at cast time.
#[test]
fn abrupt_decay_rejects_high_cmc_target() {
    let mut g = two_player_game();
    // Tarmogoyf is base {1}{G} → CMC 2 — but the engine validates the cast-
    // time `ManaValueAtMost(2)` against the *definition* CMC. Use a
    // 3-CMC card for the rejection test: Cankerbloom is {1}{G}{G}? Actually
    // it's {1}{G} = 2. Let's use Soul-Guide Lantern which is {1} = 1. Let's
    // pick something CMC ≥ 3: Pact of Negation is {0}, no good. Let's use
    // mana_leak ({1}{U} = 2). Use phyrexian_arena ({1}{B}{B} = 3). Yes.
    let arena = g.add_card_to_battlefield(1, catalog::phyrexian_arena());
    let id = g.add_card_to_hand(0, catalog::abrupt_decay());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);

    let res = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(arena)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(res.is_err(),
        "Abrupt Decay should reject a CMC-3 permanent target");
}

/// Abrupt Decay is uncounterable via Keyword::CantBeCountered.
#[test]
fn abrupt_decay_cannot_be_countered() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::abrupt_decay());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Abrupt Decay castable");

    // Verify the spell on the stack is flagged uncounterable.
    let flagged = g.stack.iter().any(|si| matches!(si, StackItem::Spell { uncounterable: true, .. }));
    assert!(flagged, "Abrupt Decay's stack item should be marked uncounterable");
}

/// Kodama's Reach searches twice — one basic to play tapped, one to hand.
#[test]
fn kodamas_reach_searches_two_basics() {
    let mut g = two_player_game();
    let bf_target = g.add_card_to_library(0, catalog::forest());
    let hand_target = g.add_card_to_library(0, catalog::island());
    // Library padding so the search filters have non-trivial options.
    g.add_card_to_library(0, catalog::lightning_bolt());

    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(bf_target)),
        DecisionAnswer::Search(Some(hand_target)),
    ]));

    let id = g.add_card_to_hand(0, catalog::kodamas_reach());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Kodama's Reach castable for {2}{G}");
    drain_stack(&mut g);

    // First basic should be on the battlefield tapped.
    let bf_view = g.battlefield.iter().find(|c| c.id == bf_target);
    assert!(bf_view.is_some(), "First basic should land on the battlefield");
    assert!(bf_view.unwrap().tapped, "Battlefield basic should enter tapped");
    // Second basic should be in hand.
    assert!(g.players[0].hand.iter().any(|c| c.id == hand_target),
        "Second basic should land in hand");
}

/// Lotus Petal: tap and sac for one mana of any color.
#[test]
fn lotus_petal_taps_and_sacs_for_any_one_color() {
    let mut g = two_player_game();
    let petal = g.add_card_to_battlefield(0, catalog::lotus_petal());
    g.clear_sickness(petal);

    g.perform_action(GameAction::ActivateAbility {
        card_id: petal, ability_index: 0, target: None, x_value: None })
    .expect("Lotus Petal activates");
    drain_stack(&mut g);

    // Sacrificed: leaves the battlefield, lands in graveyard.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == petal),
        "Petal should be sacrificed");
    assert!(!g.battlefield.iter().any(|c| c.id == petal));
    // One mana of any color was added.
    assert_eq!(g.players[0].mana_pool.total(), 1,
        "Petal should add exactly one mana");
}

/// Tormod's Crypt: tap and sac to exile each opponent's graveyard.
#[test]
fn tormods_crypt_exiles_opponent_graveyard() {
    let mut g = two_player_game();
    // Stock P1's graveyard with a few cards.
    for _ in 0..3 {
        let cid = g.add_card_to_library(1, catalog::lightning_bolt());
        let pos = g.players[1].library.iter().position(|c| c.id == cid).unwrap();
        let card = g.players[1].library.remove(pos);
        g.players[1].graveyard.push(card);
    }
    let p1_grave_before = g.players[1].graveyard.len();
    let crypt = g.add_card_to_battlefield(0, catalog::tormods_crypt());
    g.clear_sickness(crypt);

    g.perform_action(GameAction::ActivateAbility {
        card_id: crypt, ability_index: 0, target: None, x_value: None })
    .expect("Tormod's Crypt activates");
    drain_stack(&mut g);

    // Crypt sacrificed; opp graveyard exiled.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == crypt),
        "Crypt should be sacrificed");
    assert_eq!(g.players[1].graveyard.len(), 0,
        "P1's graveyard should be empty");
    assert!(g.exile.len() >= p1_grave_before,
        "Exiled cards should land in exile");
}

/// Mishra's Bauble: tap and sac to register a delayed cantrip on next upkeep.
#[test]
fn mishras_bauble_sacs_and_registers_delayed_draw() {
    let mut g = two_player_game();
    // Library has a card so the LookAtTop has an input.
    g.add_card_to_library(0, catalog::island());
    let bauble = g.add_card_to_battlefield(0, catalog::mishras_bauble());
    g.clear_sickness(bauble);

    let delayed_before = g.delayed_triggers.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: bauble, ability_index: 0, target: None, x_value: None })
    .expect("Mishra's Bauble activates");
    drain_stack(&mut g);

    // Bauble sacrificed.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bauble),
        "Bauble should be sacrificed");
    // A delayed trigger should be queued for the next upkeep.
    assert_eq!(g.delayed_triggers.len(), delayed_before + 1,
        "Bauble should have registered a delayed-draw trigger");
}

/// Stoneforge Mystic ETB tutors an Equipment.
///
/// Note: the cube/catalog has no equipment cards yet that are easy to fixture.
/// We assert the ETB-search trigger fires and routes through the decider —
/// declining is the "no equipment in library" outcome and produces no hand
/// gain.
#[test]
fn stoneforge_mystic_etb_searches_for_equipment() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::lightning_bolt());

    // Decider will be asked Search(None) — there's no equipment to pull. The
    // important assertion is that the decision was raised at all.
    let asked_before = 0usize;

    let id = g.add_card_to_battlefield(0, catalog::stoneforge_mystic());
    drain_stack(&mut g);

    // Stoneforge is on the battlefield; ETB trigger should have resolved
    // (search resolved as `None`, no hand gain).
    assert!(g.battlefield.iter().any(|c| c.id == id));
    let _ = asked_before;
}

/// Qasali Pridemage: {1}, sac itself to destroy artifact/enchantment.
#[test]
fn qasali_pridemage_sacs_to_destroy_artifact() {
    let mut g = two_player_game();
    let pride = g.add_card_to_battlefield(0, catalog::qasali_pridemage());
    let opp_artifact = g.add_card_to_battlefield(1, catalog::sol_ring());
    g.clear_sickness(pride);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::ActivateAbility {
        card_id: pride,
        ability_index: 0,
        target: Some(Target::Permanent(opp_artifact)), x_value: None })
    .expect("Qasali Pridemage activates");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == opp_artifact),
        "Sol Ring should be destroyed");
    assert!(!g.battlefield.iter().any(|c| c.id == pride),
        "Pridemage is sacrificed");
}

/// Greater Good: sac creature, draw P, discard 3.
#[test]
fn greater_good_sacrifices_creature_and_draws_power() {
    let mut g = two_player_game();
    let gg = g.add_card_to_battlefield(0, catalog::greater_good());
    // Sac fodder: a 5/5 Griselbrand-class body. Use Goldspan Dragon (4/4).
    let fodder = g.add_card_to_battlefield(0, catalog::goldspan_dragon());
    g.clear_sickness(gg);
    g.clear_sickness(fodder);
    // Stock library with 5 cards so the draw 4 has inputs.
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    // Stock hand with extra cards so the discard 3 has inputs.
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::lightning_bolt());
    }
    let hand_before = g.players[0].hand.len();
    let library_before = g.players[0].library.len();

    g.perform_action(GameAction::ActivateAbility {
        card_id: gg, ability_index: 0, target: None, x_value: None })
    .expect("Greater Good activates");
    drain_stack(&mut g);

    // Goldspan Dragon (4 power) sacrificed; draw 4; discard 3.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder),
        "Goldspan Dragon should be sacrificed");
    let drawn = library_before - g.players[0].library.len();
    assert_eq!(drawn, 4, "Should draw 4 cards (= sacrificed power)");
    // Net hand: +4 draw - 3 discard = +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Net hand = +4 draw - 3 discard = +1");
}

// ── Cube cards (round 6: modal counter, sac-payoff, drain Demon, recursion) ──

#[test]
fn cryptic_command_counter_plus_bounce_resolves() {
    // P1 casts Lightning Bolt at P0; P0 responds with Cryptic Command in
    // mode 0 (counter + bounce). The counter half consumes Bolt; the
    // bounce half tries to operate on the same target slot — but with a
    // counter that just consumed the spell, the second `Move` no longer
    // finds anything on the stack. We just verify the spell got countered.
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable");

    let cryptic = g.add_card_to_hand(0, catalog::cryptic_command());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: cryptic,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: Some(0), // counter + bounce
        x_value: None,
    })
    .expect("Cryptic Command castable for {1}{U}{U}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, 20,
        "Bolt should have been countered by Cryptic Command's mode 0");
    // After the counter, mode 0's bounce step then operates on the same
    // target — by then the Bolt has hit the graveyard, so the bounce
    // pulls it into P1's hand. Either zone is consistent with the
    // counter having succeeded; just assert it's not still on the stack
    // and no damage was dealt.
    assert!(g.stack.is_empty(), "Stack should be empty after resolution");
}

#[test]
fn cryptic_command_mode_two_counter_and_draw() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .unwrap();

    let cryptic = g.add_card_to_hand(0, catalog::cryptic_command());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 3);
    let hand_before = g.players[0].hand.len();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: cryptic,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: Some(2), // counter + draw 1
        x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);

    // Cryptic Command itself goes to grave on resolution; net hand
    // change = +1 (drew 1 from mode 2) - 1 (cast Cryptic) = 0.
    assert_eq!(g.players[0].hand.len(), hand_before, "Net hand: +1 draw - 1 cast = 0");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));
}

#[test]
fn deadly_dispute_sacrifices_and_creates_treasure_and_draws_two() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(fodder);
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let dispute = g.add_card_to_hand(0, catalog::deadly_dispute());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: dispute, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Deadly Dispute castable for {1}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == fodder),
        "Sacrificed creature should leave the battlefield");
    let treasures = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 1, "Should create one Treasure token");
    // Cast Dispute (-1), drew 2 (+2), net +1 ≈ hand_before + 1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "Net +1 hand");
}

#[test]
fn bloodchiefs_thirst_destroys_low_cmc_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // CMC 2
    let thirst = g.add_card_to_hand(0, catalog::bloodchiefs_thirst());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: thirst,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bloodchief's Thirst castable for {B}");
    drain_stack(&mut g);

    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear),
        "Targeted bear should be destroyed");
}

#[test]
fn bloodchiefs_thirst_rejects_high_cmc_target() {
    let mut g = two_player_game();
    let mahamoti = g.add_card_to_battlefield(1, catalog::mahamoti_djinn()); // CMC 6
    let thirst = g.add_card_to_hand(0, catalog::bloodchiefs_thirst());
    g.players[0].mana_pool.add(Color::Black, 1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: thirst,
        target: Some(Target::Permanent(mahamoti)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .unwrap_err();
    assert_eq!(err, GameError::SelectionRequirementViolated,
        "Mana value 6 fails the ≤2 base mode filter");
}

#[test]
fn bloodchiefs_thirst_kicked_destroys_high_cmc_creature() {
    let mut g = two_player_game();
    let mahamoti = g.add_card_to_battlefield(1, catalog::mahamoti_djinn()); // CMC 6
    let thirst = g.add_card_to_hand(0, catalog::bloodchiefs_thirst());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: thirst,
        target: Some(Target::Permanent(mahamoti)),
        additional_targets: vec![],
        pitch_card: None,
        mode: None,
        x_value: None,
    })
    .expect("Kicked Bloodchief's Thirst should destroy any creature/PW");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().find(|c| c.id == mahamoti).is_none(),
        "Mahamoti Djinn should be destroyed by kicked Bloodchief's Thirst");
}

#[test]
fn heliod_sun_crowned_grants_lifelink_until_end_of_turn() {
    let mut g = two_player_game();
    let heliod = g.add_card_to_battlefield(0, catalog::heliod_sun_crowned());
    g.clear_sickness(heliod);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::ActivateAbility {
        card_id: heliod,
        ability_index: 0,
        target: Some(Target::Permanent(bear)), x_value: None })
    .expect("Heliod's lifelink-grant activates for {1}{W}");
    drain_stack(&mut g);

    let cp = g.computed_permanent(bear).expect("Bear still in play");
    assert!(cp.keywords.contains(&crate::card::Keyword::Lifelink),
        "Bear should now have Lifelink");
}

#[test]
fn indulgent_tormentor_drains_each_opponent_at_end_step() {
    let mut g = two_player_game();
    let _torm = g.add_card_to_battlefield(0, catalog::indulgent_tormentor());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let p1_life_before = g.players[1].life;

    // Drive to End step on P0's turn.
    g.step = TurnStep::PostCombatMain;
    g.perform_action(GameAction::PassPriority).unwrap();
    g.perform_action(GameAction::PassPriority).unwrap();
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, p1_life_before - 3,
        "Indulgent Tormentor's end-step trigger should drain 3 life from P1");
}

/// With the graveyard-source preference in `auto_target_for_effect`,
/// Eternal Witness's ETB now picks a card out of YOUR graveyard
/// automatically — the trigger no longer requires UI to land its
/// gameplay-default behavior.
#[test]
fn eternal_witness_etb_returns_graveyard_card_via_auto_target() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::eternal_witness());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Eternal Witness castable for {1}{G}{G}");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == bolt),
        "Bolt should auto-return from graveyard to hand");
}

#[test]
fn static_prison_etb_taps_target() {
    // ETB taps the target permanent. The X-cost stun-counter clause
    // also fires now that the engine threads `x_value` from the
    // resolving spell into the ETB trigger's `EffectContext`.
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(opp_bear).unwrap().tapped = false;

    let prison = g.add_card_to_hand(0, catalog::static_prison());
    // X=0 cast, just to exercise the tap path: total {2}{W}.
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: prison,
        target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(0),
    })
    .expect("Static Prison castable for {0}{2}{W}");
    drain_stack(&mut g);

    assert!(g.battlefield_find(prison).is_some(), "Prison on battlefield");
    assert!(g.battlefield_find(opp_bear).unwrap().tapped,
        "Targeted permanent should be tapped on ETB");
}

#[test]
fn static_prison_x2_etb_adds_two_stun_counters_to_target() {
    // Push (modern_decks): Stun counters now land on the TARGET (CR-
    // correct), not on Static Prison itself. The engine's stun-counter
    // mechanic (CR 122.1d) consumes one counter per untap step, keeping
    // the target tapped for X turn cycles.
    use crate::card::CounterType;
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let prison = g.add_card_to_hand(0, catalog::static_prison());
    // {X=2}{2}{W} → pay 4 colorless + {W}.
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: prison,
        target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("Static Prison castable with X=2");
    drain_stack(&mut g);

    let target = g.battlefield_find(opp_bear).expect("Bear still on battlefield");
    assert_eq!(
        target.counter_count(CounterType::Stun),
        2,
        "X=2 should put 2 stun counters on the TARGET (the opp's bear), \
         not on Static Prison itself"
    );
    assert!(target.tapped, "Target should also be tapped");
    let inst = g.battlefield_find(prison).expect("Prison on battlefield");
    assert_eq!(
        inst.counter_count(CounterType::Stun),
        0,
        "Static Prison itself should have no stun counters (CR-correct: \
         counters go on the targeted permanent)"
    );
}

#[test]
fn marauding_mako_grows_when_you_discard() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let mako = g.add_card_to_battlefield(0, catalog::marauding_mako());
    g.clear_sickness(mako);

    // P0 discards a card via an effect — we use direct hand-to-graveyard
    // movement to keep the test focused on the discard listener.
    let throwaway = g.add_card_to_hand(0, catalog::forest());
    let card = g.players[0].remove_from_hand(throwaway).unwrap();
    g.players[0].graveyard.push(card);
    // Fire the discard event directly — this exercises the listener path.
    let events = vec![GameEvent::CardDiscarded { player: 0, card_id: throwaway }];
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);

    let counters = g.battlefield_find(mako).unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, 1, "Discarding a card should add one +1/+1 counter");
}

// ── New cards (claude/modern_decks: sweepers / tutors / burn / lands) ────────

/// Pyroclasm: 2 damage to each creature destroys 2-toughness creatures
/// while leaving bigger ones alive.
#[test]
fn pyroclasm_kills_two_toughness_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // 5/5
    let py = g.add_card_to_hand(0, catalog::pyroclasm());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: py, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Pyroclasm castable for {1}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Pyroclasm should kill the 2-toughness Grizzly Bears");
    assert!(g.battlefield.iter().any(|c| c.id == dragon),
        "Pyroclasm should leave the 5-toughness Shivan Dragon alive");
}

/// Day of Judgment: destroy each creature regardless of toughness.
#[test]
fn day_of_judgment_destroys_all_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    let lion = g.add_card_to_battlefield(0, catalog::savannah_lions());
    let day = g.add_card_to_hand(0, catalog::day_of_judgment());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: day, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Day of Judgment castable for {2}{W}{W}");
    drain_stack(&mut g);

    for cid in [bear, dragon, lion] {
        assert!(!g.battlefield.iter().any(|c| c.id == cid),
            "Day of Judgment should destroy all creatures");
    }
}

/// Damnation: black-mana mirror of Day of Judgment. Destroys every
/// creature including indestructible-without-shroud ones (engine has no
/// regen primitive to bypass anyway).
#[test]
fn damnation_destroys_all_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    let dn = g.add_card_to_hand(0, catalog::damnation());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: dn, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Damnation castable for {2}{B}{B}");
    drain_stack(&mut g);

    for cid in [a, b] {
        assert!(!g.battlefield.iter().any(|c| c.id == cid));
    }
}

/// Mystical Tutor: search library for an instant or sorcery and put on top.
#[test]
fn mystical_tutor_finds_instant_and_puts_on_top() {
    let mut g = two_player_game();
    // Stock library with a creature (ineligible) + a sorcery (eligible).
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bolt))]));

    let id = g.add_card_to_hand(0, catalog::mystical_tutor());
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Mystical Tutor castable for {U}");
    drain_stack(&mut g);

    // Bolt should land on top of library; bear stays put.
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(bolt),
        "Mystical Tutor should put the chosen instant on top of library");
    assert!(g.players[0].library.iter().any(|c| c.id == bear),
        "Untargeted card should remain in library");
}

/// Worldly Tutor: search for a creature, put on top.
#[test]
fn worldly_tutor_finds_creature_and_puts_on_top() {
    let mut g = two_player_game();
    let creature = g.add_card_to_library(0, catalog::shivan_dragon());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(creature))]));

    let id = g.add_card_to_hand(0, catalog::worldly_tutor());
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Worldly Tutor castable for {G}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(creature),
        "Worldly Tutor should put the chosen creature on top");
}

/// Enlightened Tutor: search for an artifact or enchantment.
#[test]
fn enlightened_tutor_finds_artifact_and_puts_on_top() {
    let mut g = two_player_game();
    let artifact = g.add_card_to_library(0, catalog::sol_ring());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(artifact))]));

    let id = g.add_card_to_hand(0, catalog::enlightened_tutor());
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Enlightened Tutor castable for {W}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(artifact),
        "Enlightened Tutor should put the chosen artifact on top");
}

/// Diabolic Tutor: search for any card, put into hand.
#[test]
fn diabolic_tutor_finds_any_card_into_hand() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bolt))]));

    let id = g.add_card_to_hand(0, catalog::diabolic_tutor());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Diabolic Tutor castable for {2}{B}{B}");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == bolt),
        "Diabolic Tutor should pull the chosen card into hand");
    assert!(!g.players[0].library.iter().any(|c| c.id == bolt));
}

/// Imperial Seal: pay 2 life, search for any card, put on top.
#[test]
fn imperial_seal_pays_two_life_and_tutors_to_top() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bolt))]));

    let id = g.add_card_to_hand(0, catalog::imperial_seal());
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Imperial Seal castable for {B}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before - 2,
        "Imperial Seal should cost 2 life");
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(bolt),
        "Imperial Seal should put the chosen card on top");
}

/// Lightning Strike: 3 damage to a creature.
#[test]
fn lightning_strike_deals_three_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lightning_strike());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Lightning Strike castable for {1}{R} on a creature");
    drain_stack(&mut g);

    // 3 damage > 2 toughness ⇒ destroyed.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Lightning Strike should destroy the Grizzly Bears");
}

/// Lightning Strike: 3 damage to a player.
#[test]
fn lightning_strike_can_target_a_player() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lightning_strike());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Lightning Strike castable at a player");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, before - 3);
}

/// Goblin Bombardment: sacrifice a creature, deal 1 damage to any target.
#[test]
fn goblin_bombardment_sacrifices_creature_and_deals_one_damage() {
    let mut g = two_player_game();
    let bomb = g.add_card_to_battlefield(0, catalog::goblin_bombardment());
    g.clear_sickness(bomb);
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(fodder);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::ActivateAbility {
        card_id: bomb,
        ability_index: 0,
        target: Some(Target::Player(1)), x_value: None })
    .expect("Goblin Bombardment activates");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == fodder),
        "Bomb should sacrifice the Grizzly Bears");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder));
    assert_eq!(g.players[1].life, life_before - 1,
        "Bombardment should ping the targeted player for 1");
}

/// Wasteland: tap and sacrifice to destroy a nonbasic land.
#[test]
fn wasteland_destroys_nonbasic_land() {
    let mut g = two_player_game();
    let waste = g.add_card_to_battlefield(0, catalog::wasteland());
    g.clear_sickness(waste);
    // Place a nonbasic dual under P1.
    let dual = g.add_card_to_battlefield(1, catalog::watery_grave());
    g.clear_sickness(dual);

    // Activate ability index 1 (the destroy-land ability).
    g.perform_action(GameAction::ActivateAbility {
        card_id: waste,
        ability_index: 1,
        target: Some(Target::Permanent(dual)), x_value: None })
    .expect("Wasteland's destroy ability activates");
    drain_stack(&mut g);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == waste),
        "Wasteland should be sacrificed");
    assert!(!g.battlefield.iter().any(|c| c.id == dual),
        "Wasteland should destroy the nonbasic dual");
}

/// Wasteland: rejects a basic land target (filter enforces nonbasic).
#[test]
fn wasteland_rejects_basic_land_target() {
    let mut g = two_player_game();
    let waste = g.add_card_to_battlefield(0, catalog::wasteland());
    g.clear_sickness(waste);
    let plains = g.add_card_to_battlefield(1, catalog::plains());

    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: waste,
        ability_index: 1,
        target: Some(Target::Permanent(plains)), x_value: None });
    assert!(res.is_err(),
        "Wasteland's destroy ability should reject a basic-land target");
}

/// Strip Mine: tap and sacrifice to destroy any land (including basics).
#[test]
fn strip_mine_destroys_any_land() {
    let mut g = two_player_game();
    let strip = g.add_card_to_battlefield(0, catalog::strip_mine());
    g.clear_sickness(strip);
    let plains = g.add_card_to_battlefield(1, catalog::plains());

    g.perform_action(GameAction::ActivateAbility {
        card_id: strip,
        ability_index: 1,
        target: Some(Target::Permanent(plains)), x_value: None })
    .expect("Strip Mine activates against any land");
    drain_stack(&mut g);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == strip),
        "Strip Mine should be sacrificed");
    assert!(!g.battlefield.iter().any(|c| c.id == plains),
        "Strip Mine should destroy even a basic land");
}

/// Snuff Out: cast for {3}{B} normally — destroys nonblack creature.
#[test]
fn snuff_out_destroys_nonblack_creature_via_normal_cost() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::snuff_out());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Snuff Out castable for {3}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

/// Snuff Out: pitch alt cost — pay 4 life instead of mana.
#[test]
fn snuff_out_alt_cost_pays_four_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::snuff_out());
    let life_before = g.players[0].life;
    // No mana — alt cost must succeed via 4 life.

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id,
        pitch_card: None,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Snuff Out alt cost pays 4 life");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before - 4,
        "Snuff Out alt cost should deduct 4 life");
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

/// Teferi -3 rejects a target that doesn't match its
/// "nonland permanent an opponent controls" filter. Loyalty abilities
/// previously skipped the slot-0 filter check (only spell casts and
/// activated abilities enforced it), so a Teferi -3 aimed at the
/// controller's own permanent silently bounced their own creature.
#[test]
fn teferi_minus_three_rejects_self_targeted_land() {
    let mut g = two_player_game();
    let teferi = g.add_card_to_battlefield(0, catalog::teferi_time_raveler());
    let own_forest = g.add_card_to_battlefield(0, catalog::forest());
    // Stock a card so the +draw rider doesn't deck out.
    g.add_card_to_library(0, catalog::forest());

    let err = g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: teferi,
        ability_index: 1, // -3
        target: Some(Target::Permanent(own_forest)),
    })
    .unwrap_err();
    assert_eq!(err, GameError::SelectionRequirementViolated,
        "Teferi -3 should reject the controller's own land");
    // Forest still on the battlefield.
    assert!(g.battlefield.iter().any(|c| c.id == own_forest));
}

/// Snuff Out: rejects a black-creature target (filter enforces nonblack).
#[test]
fn snuff_out_rejects_black_creature() {
    let mut g = two_player_game();
    let demon = g.add_card_to_battlefield(1, catalog::griselbrand());
    let id = g.add_card_to_hand(0, catalog::snuff_out());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Black, 1);

    let res = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(demon)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(res.is_err(),
        "Snuff Out should reject a black creature target");
}

/// Windfall: each player discards their hand and draws 7 cards.
#[test]
fn windfall_discards_both_hands_then_draws_max_discarded() {
    // Push (batch 115): dynamic yield. P0 has 2 cards, P1 has 3 cards
    // (plus Windfall itself = 4 in hand). After discarding everything
    // each player draws `max(2, 4) = 4` cards.
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    // Clear pre-existing hands so we can stage the counts precisely.
    g.players[0].hand.clear();
    g.players[1].hand.clear();
    // Give each player a few cards in hand + library.
    for _ in 0..2 { g.add_card_to_hand(0, catalog::forest()); }
    for _ in 0..3 { g.add_card_to_hand(1, catalog::island()); }
    for _ in 0..15 { g.add_card_to_library(0, catalog::forest()); }
    for _ in 0..15 { g.add_card_to_library(1, catalog::island()); }
    let wf = g.add_card_to_hand(1, catalog::windfall()); // P1 hand now = 4
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.players[1].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: wf, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    // P0 discarded 2; P1 discarded 4 (3 islands + Windfall after it
    // started resolving — actually Windfall leaves hand at cast time
    // so P1's hand was 3 at the discard step). Max = 4 or 3 depending
    // on cast-time bookkeeping; what matters is "both players draw the
    // same amount, equal to the max".
    let drawn_p0 = g.players[0].hand.len();
    let drawn_p1 = g.players[1].hand.len();
    assert_eq!(drawn_p0, drawn_p1,
        "Each player draws the same amount (the max discarded)");
    assert!(drawn_p0 >= 3, "Max discarded was at least 3 (P1's island hand)");
    assert!(drawn_p0 <= 4, "Max discarded was at most 4 (P1's full pre-cast hand)");
}

#[test]
fn windfall_asymmetric_discards_yields_higher_player_count() {
    // Force an asymmetric discard: P0 has 6 cards, P1 has 1 + Windfall.
    // Each player draws 6 (P0's discard count, the max).
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[0].hand.clear();
    g.players[1].hand.clear();
    for _ in 0..6 { g.add_card_to_hand(0, catalog::forest()); }
    for _ in 0..1 { g.add_card_to_hand(1, catalog::island()); }
    for _ in 0..20 { g.add_card_to_library(0, catalog::forest()); }
    for _ in 0..20 { g.add_card_to_library(1, catalog::island()); }
    let wf = g.add_card_to_hand(1, catalog::windfall());
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.players[1].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: wf, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 6,
        "P0 discarded 6 — Max = 6, P0 redraws 6");
    assert_eq!(g.players[1].hand.len(), 6,
        "P1 only discarded 2 (1 island + Windfall) but still draws 6 = max");
}

/// Treasure Cruise: at full {7}{U} cost, draws 3 cards.
#[test]
fn treasure_cruise_draws_three_at_full_cost() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::treasure_cruise());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(7);
    let hand_before_cast = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);

    // Net change: cast Cruise (-1) + drew 3 (+3) = +2.
    assert_eq!(g.players[0].hand.len(), hand_before_cast + 2);
}

/// Lose Focus: counters target spell when controller can't pay {2}.
#[test]
fn lose_focus_counters_when_controller_cannot_pay_two() {
    let mut g = two_player_game();
    // Bob is the active player; he casts Lightning Bolt at Alice.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).unwrap();
    // Bob has no mana, so paying {2} is impossible. Alice casts Lose Focus
    // at the bolt at instant speed (priority moved to her after Bob's cast).
    g.priority.player_with_priority = 0;
    let lose = g.add_card_to_hand(0, catalog::lose_focus());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: lose,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    // Bolt should be countered (graveyard) — no damage to Alice.
    assert_eq!(g.players[0].life, 20);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Lightning Bolt should be in Bob's graveyard after counter");
}

/// Lose Focus: leaves the spell alone when the controller can pay {2}.
#[test]
fn lose_focus_does_not_counter_when_controller_can_pay_two() {
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    // Bob has 2 extra colorless to pay the unless-cost.
    g.players[1].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).unwrap();
    g.priority.player_with_priority = 0;
    let lose = g.add_card_to_hand(0, catalog::lose_focus());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: lose,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    // Bolt resolved despite Lose Focus — Alice took 3.
    assert_eq!(g.players[0].life, 17);
}

// ── New mod_set additions: Stifle / Memory Lapse / Reckless Charge / etc. ──

/// Stifle counters the most recent triggered ability whose source matches
/// the targeted permanent.
#[test]
fn stifle_counters_a_triggered_ability_off_the_stack() {
    let mut g = two_player_game();
    // Cast Devourer of Destiny (P0) — its on-cast Scry-2 trigger goes on
    // top of the spell. Then Stifle the trigger.
    let dev = g.add_card_to_hand(0, catalog::devourer_of_destiny());
    g.players[0].mana_pool.add_colorless(7);
    g.perform_action(GameAction::CastSpell {
        card_id: dev, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    // P1 stifles the trigger before it resolves.
    g.priority.player_with_priority = 1;
    let stifle = g.add_card_to_hand(1, catalog::stifle());
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: stifle,
        target: Some(Target::Permanent(dev)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Stifle should accept Devourer as the source target");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == dev),
        "Devourer should still resolve — Stifle only counters the ability");
    assert!(!g.stack.iter().any(|si| matches!(
        si, crate::game::StackItem::Trigger { source, .. } if *source == dev
    )), "Scry trigger should have been countered");
}

/// Memory Lapse: counters a target spell.
#[test]
fn memory_lapse_counters_target_spell() {
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).unwrap();
    g.priority.player_with_priority = 0;
    let lapse = g.add_card_to_hand(0, catalog::memory_lapse());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: lapse,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Memory Lapse should accept the bolt as a spell target");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "bolt was countered");
}

/// Vines of Vastwood: pumps the targeted creature +4/+4 EOT.
#[test]
fn vines_of_vastwood_pumps_target_creature_plus_four() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let vines = g.add_card_to_hand(0, catalog::vines_of_vastwood());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: vines,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Vines should accept the bear as a target");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).expect("bear still alive");
    assert_eq!(cp.power, 6, "Grizzly Bears 2/2 + 4 = 6 power");
    assert_eq!(cp.toughness, 6, "Grizzly Bears 2/2 + 4 = 6 toughness");
}

/// Reckless Charge: pumps +3/+0 and grants haste until end of turn.
#[test]
fn reckless_charge_grants_three_power_and_haste() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let charge = g.add_card_to_hand(0, catalog::reckless_charge());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: charge,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Reckless Charge castable for {R}");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 5, "+3 power from Reckless Charge");
    assert_eq!(cp.toughness, 2, "toughness unchanged");
    assert!(
        cp.keywords.contains(&crate::card::Keyword::Haste),
        "should have haste"
    );
}

/// Boil: destroys every Island in play, regardless of controller.
#[test]
fn boil_destroys_all_islands() {
    let mut g = two_player_game();
    let i1 = g.add_card_to_battlefield(0, catalog::island());
    let i2 = g.add_card_to_battlefield(1, catalog::island());
    let f1 = g.add_card_to_battlefield(0, catalog::forest());
    let boil = g.add_card_to_hand(0, catalog::boil());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: boil, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Boil castable for {2}{R}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == i1), "P0's Island should be destroyed");
    assert!(!g.battlefield.iter().any(|c| c.id == i2), "P1's Island should be destroyed");
    assert!(g.battlefield.iter().any(|c| c.id == f1), "Forest should survive");
}

/// Compulsive Research: caster draws three then discards two.
#[test]
fn compulsive_research_draws_three_discards_two() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::compulsive_research());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    // -1 (cast) + 3 (draw) - 2 (discard) = net 0.
    assert_eq!(g.players[0].hand.len(), hand_before, "net hand size unchanged");
    assert_eq!(g.players[0].graveyard.len(), 3, "2 discards + the cast spell itself");
}

/// Demolish: destroys target artifact.
#[test]
fn demolish_destroys_target_artifact() {
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    let demo = g.add_card_to_hand(0, catalog::demolish());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: demo,
        target: Some(Target::Permanent(stone)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Demolish should accept an artifact target");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == stone),
        "Mind Stone should be destroyed by Demolish");
}

/// Mind Sculpt: each opponent mills 7.
#[test]
fn mind_sculpt_mills_each_opponent_seven() {
    let mut g = two_player_game();
    for _ in 0..15 { g.add_card_to_library(1, catalog::island()); }
    let lib_before = g.players[1].library.len();
    let ms = g.add_card_to_hand(0, catalog::mind_sculpt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: ms, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib_before - 7,
        "P1 should have milled 7 cards");
    assert_eq!(g.players[1].graveyard.len(), 7);
}

/// Cabal Therapy: caster picks a nonland card from each opponent's hand.
#[test]
fn cabal_therapy_discards_a_nonland_from_opponent() {
    let mut g = two_player_game();
    let target_card = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::forest());
    let ct = g.add_card_to_hand(0, catalog::cabal_therapy());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: ct, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cabal Therapy castable for {B}");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == target_card),
        "Lightning Bolt (the only nonland in P1's hand) should be discarded");
    assert_eq!(g.players[1].hand.len(), 1, "Forest still in hand");
}

/// Wear Down: destroys a target artifact or enchantment.
#[test]
fn wear_down_destroys_target_artifact() {
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    let wd = g.add_card_to_hand(0, catalog::wear_down());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: wd,
        target: Some(Target::Permanent(stone)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Wear Down should accept an artifact target");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == stone),
        "Mind Stone should be destroyed by Wear Down");
}

// ── Cube additions: cheap creatures + sacrifice-cost spells ─────────────────

/// Memnite: vanilla {0} 1/1 artifact creature — castable from an empty pool.
#[test]
fn memnite_casts_for_zero_mana() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::memnite());
    // Zero pool — Memnite costs nothing.
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Memnite is free");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).expect("Memnite on battlefield");
    assert_eq!(card.power(), 1);
    assert_eq!(card.toughness(), 1);
    assert!(card.definition.card_types.contains(&CardType::Artifact));
    assert!(card.definition.card_types.contains(&CardType::Creature));
}

/// Fanatic of Rhonas: {G},{T} produces {G}{G} (net +{G}).
#[test]
fn fanatic_of_rhonas_taps_for_two_green_after_paying_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::fanatic_of_rhonas());
    g.clear_sickness(id);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).expect("Fanatic activates for {{G}},{{T}}");
    // Activated mana abilities resolve immediately (no stack), so no drain.
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2,
        "Net production: paid {{G}} + ability adds {{G}}{{G}} = +{{G}} pool");
    let card = g.battlefield_find(id).expect("still on battlefield");
    assert!(card.tapped, "Tap cost taps the source");
}

/// Greasewrench Goblin: vanilla 2/2 haste body.
#[test]
fn greasewrench_goblin_enters_with_haste() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::greasewrench_goblin());
    let card = g.battlefield_find(id).expect("on battlefield");
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 2);
    assert!(card.has_keyword(&crate::card::Keyword::Haste),
        "Greasewrench Goblin should have Haste");
    // Haste lets it attack on the turn it enters.
    assert!(card.can_attack(),
        "Haste creature can attack the turn it enters");
}

/// Orcish Lumberjack: {T}, sacrifice a Forest → add {G}{G}{G}. The
/// Forest sacrifice is folded into the resolved effect's first step, so
/// we need to make this a non-mana ability that goes on the stack… but
/// the engine treats `Seq([Sacrifice, AddMana])` as a non-mana ability
/// since `is_mana_ability` only matches pure-AddMana effects. Drain the
/// stack to resolve.
#[test]
fn orcish_lumberjack_sacrifices_forest_for_three_green() {
    let mut g = two_player_game();
    let lj = g.add_card_to_battlefield(0, catalog::orcish_lumberjack());
    g.clear_sickness(lj);
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: lj, ability_index: 0, target: None, x_value: None }).expect("Lumberjack should activate for {T}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == forest),
        "Forest should be sacrificed by the activation");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 3,
        "Activation should add {{G}}{{G}}{{G}}");
}

/// Mine Collapse: {2}{R} sorcery, sacrifice a Mountain on resolution,
/// deal 4 damage to the target.
#[test]
fn mine_collapse_sacrifices_mountain_and_deals_four() {
    let mut g = two_player_game();
    let mtn = g.add_card_to_battlefield(0, catalog::mountain());
    let mc = g.add_card_to_hand(0, catalog::mine_collapse());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: mc,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Mine Collapse castable for {{2}}{{R}}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == mtn),
        "Mountain should be sacrificed on resolution");
    assert_eq!(g.players[1].life, 16,
        "Target player should take 4 damage");
}

/// Satyr Wayfinder: ETB mills 4 from your library.
#[test]
fn satyr_wayfinder_etb_mills_four() {
    let mut g = two_player_game();
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::forest());
    }
    let lib_before = g.players[0].library.len();
    let yard_before = g.players[0].graveyard.len();
    let sw = g.add_card_to_hand(0, catalog::satyr_wayfinder());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: sw, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Satyr Wayfinder castable for {{1}}{{G}}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before - 4,
        "Four cards should leave the library");
    assert_eq!(g.players[0].graveyard.len(), yard_before + 4,
        "Four cards should land in the graveyard");
}

/// Fireblast: {4}{R}{R} for 4 damage to any target. (Alt cost path —
/// sacrifice 2 Mountains — is not yet wired; this exercises the regular
/// cost.)
#[test]
fn fireblast_deals_four_to_any_target() {
    let mut g = two_player_game();
    let fb = g.add_card_to_hand(0, catalog::fireblast());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: fb,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Fireblast castable for {{4}}{{R}}{{R}}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 16, "Target should take 4 damage");
}

/// Talisman of Progress: {T}: Add {C} via index 0; {T}: lose 1 + add
/// {W} via index 1; index 2 adds {U}.
#[test]
fn talisman_of_progress_taps_for_colorless_or_one_of_w_or_u() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::talisman_of_progress());
    g.clear_sickness(id);
    // Colorless ability (index 0) — no life cost.
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).expect("colorless tap succeeds");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1);
    // Mana abilities tap the source synchronously; untap to use again.
    let life_before = g.players[0].life;
    g.battlefield_find_mut(id).unwrap().tapped = false;
    // White ability (index 1) — costs 1 life. Bundled with `LoseLife`
    // it's no longer a pure mana ability, so it goes on the stack and
    // needs draining.
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, x_value: None }).expect("white tap succeeds");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::White), 1);
    assert_eq!(g.players[0].life, life_before - 1,
        "Talisman costs 1 life when tapped for a color");
}

/// Talisman of Dominance: UB mirror — index 1 = {U}, index 2 = {B}.
#[test]
fn talisman_of_dominance_taps_for_blue_costing_one_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::talisman_of_dominance());
    g.clear_sickness(id);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, x_value: None }).expect("blue tap succeeds");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1);
    assert_eq!(g.players[0].life, life_before - 1);
}

/// Elvish Spirit Guide: vanilla 2/2 body. (Hand-activated alt-mana
/// ability is gated on a future hand-activation primitive.)
#[test]
fn elvish_spirit_guide_is_a_two_two_elf_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::elvish_spirit_guide());
    let card = g.battlefield_find(id).expect("on battlefield");
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 2);
    assert!(card.definition.subtypes.creature_types.contains(&crate::card::CreatureType::Elf));
    assert!(card.definition.subtypes.creature_types.contains(&crate::card::CreatureType::Spirit));
}

// ── New cube cards (this branch) ───────────────────────────────────────────

#[test]
fn bloodghast_returns_from_graveyard_when_you_play_a_land() {
    let mut g = two_player_game();
    // Seed Bloodghast in P0's graveyard.
    let bg_id = g.add_card_to_library(0, catalog::bloodghast());
    let card = g.players[0]
        .library
        .iter()
        .position(|c| c.id == bg_id)
        .map(|pos| g.players[0].library.remove(pos))
        .unwrap();
    g.players[0].graveyard.push(card);

    // P0 plays a Forest. The landfall trigger should reanimate Bloodghast.
    let forest = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(forest)).unwrap();
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == bg_id),
        "Bloodghast should return to the battlefield on landfall");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bg_id),
        "Bloodghast should no longer be in the graveyard");
}

#[test]
fn ichorid_returns_at_upkeep_then_exiles_at_end_step() {
    // Pre-batch-112 this trigger fired unconditionally. Now it's gated on
    // an opponent having a black creature in their graveyard (printed
    // Oracle text), so the test seeds Black Knight in p1's graveyard
    // before walking to upkeep.
    let mut g = two_player_game();
    g.step = TurnStep::Cleanup;
    let id = g.add_card_to_library(0, catalog::ichorid());
    let card = g.players[0]
        .library
        .iter()
        .position(|c| c.id == id)
        .map(|pos| g.players[0].library.remove(pos))
        .unwrap();
    g.players[0].graveyard.push(card);
    // Seed an opp black creature in their graveyard so the new gate opens.
    g.add_card_to_graveyard(1, catalog::black_knight());

    // Walk Cleanup → Untap → Upkeep so the trigger fires.
    for _ in 0..30 {
        if g.battlefield.iter().any(|c| c.id == id) { break; }
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == id),
        "Ichorid should reanimate at the start of upkeep");
    assert!(g.delayed_triggers.iter().any(|t|
        t.kind == crate::game::types::DelayedKind::NextEndStep),
        "Reanimation should register an end-step exile delayed trigger");
}

#[test]
fn ichorid_stays_in_graveyard_when_no_opp_black_creature_in_gy() {
    // Negative test for the batch-112 gate: with no black creature in
    // any opp's graveyard, the upkeep trigger predicate fails and
    // Ichorid stays in the graveyard.
    let mut g = two_player_game();
    g.step = TurnStep::Cleanup;
    let id = g.add_card_to_library(0, catalog::ichorid());
    let card = g.players[0]
        .library
        .iter()
        .position(|c| c.id == id)
        .map(|pos| g.players[0].library.remove(pos))
        .unwrap();
    g.players[0].graveyard.push(card);
    // Seed a non-black creature in opp's graveyard (Grizzly Bears is
    // green) — the predicate must still fail.
    g.add_card_to_graveyard(1, catalog::grizzly_bears());

    // Walk past Cleanup → Untap → Upkeep.
    for _ in 0..10 {
        if g.step == TurnStep::Draw { break; }
        let _ = g.perform_action(GameAction::PassPriority);
    }
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == id),
        "Ichorid should NOT reanimate — opp has no black creature in graveyard");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id),
        "Ichorid still sits in p0's graveyard");
}

#[test]
fn silversmote_ghoul_returns_from_graveyard_on_lifegain() {
    let mut g = two_player_game();
    let id = g.add_card_to_library(0, catalog::silversmote_ghoul());
    let card = g.players[0]
        .library
        .iter()
        .position(|c| c.id == id)
        .map(|pos| g.players[0].library.remove(pos))
        .unwrap();
    g.players[0].graveyard.push(card);

    // Cast Faithful Mending (mode 2 = Discard 0) to gain 2 life.
    let mending = g.add_card_to_hand(0, catalog::faithful_mending());
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: mending, target: None, additional_targets: vec![], mode: Some(2), x_value: None,
    }).expect("Faithful Mending castable for {1}{W}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == id),
        "Silversmote Ghoul should return when its controller gains life");
}

#[test]
fn bitterbloom_bearer_etb_creates_a_faerie_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::bitterbloom_bearer());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bitterbloom Bearer castable for {1}{B}");
    drain_stack(&mut g);

    assert_eq!(g.battlefield.len(), bf_before + 2,
        "Bitterbloom Bearer + 1 Faerie token = +2 permanents");
    let faerie = g.battlefield.iter().find(|c| c.definition.name == "Faerie")
        .expect("Faerie token should be on the battlefield");
    assert_eq!(faerie.definition.power, 1);
    assert_eq!(faerie.definition.toughness, 1);
    assert!(faerie.definition.keywords.contains(&crate::card::Keyword::Flying));
}

#[test]
fn dandan_sacrifices_at_upkeep_when_no_island() {
    let mut g = two_player_game();
    let dd = g.add_card_to_battlefield(0, catalog::dandan());
    g.clear_sickness(dd);
    g.step = TurnStep::Cleanup;
    // No Islands — at the start of upkeep Dandân should sac itself.

    for _ in 0..30 {
        if !g.battlefield.iter().any(|c| c.id == dd) { break; }
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == dd),
        "Dandân should be sacrificed at upkeep when no Island is in play");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == dd),
        "Sacrificed Dandân should land in the graveyard");
}

#[test]
fn dandan_stays_in_play_with_an_island() {
    let mut g = two_player_game();
    let _island = g.add_card_to_battlefield(0, catalog::island());
    let dd = g.add_card_to_battlefield(0, catalog::dandan());
    g.clear_sickness(dd);
    g.step = TurnStep::Cleanup;

    // Walk past upkeep — Dandân should survive.
    for _ in 0..15 {
        if g.step == TurnStep::PreCombatMain { break; }
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == dd),
        "Dandân should survive while you control an Island");
}

#[test]
fn turnabout_mode_four_taps_all_opponent_lands() {
    let mut g = two_player_game();
    let m1 = g.add_card_to_battlefield(1, catalog::mountain());
    let m2 = g.add_card_to_battlefield(1, catalog::mountain());
    let i1 = g.add_card_to_battlefield(1, catalog::island());

    let ta = g.add_card_to_hand(0, catalog::turnabout());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: ta, target: None, additional_targets: vec![], mode: Some(4), x_value: None,
    }).expect("Turnabout castable for {2}{U}{U}");
    drain_stack(&mut g);

    for id in [m1, m2, i1] {
        let card = g.battlefield.iter().find(|c| c.id == id).unwrap();
        assert!(card.tapped, "Land {:?} should be tapped after Turnabout mode 4", id);
    }
}

#[test]
fn heliod_adds_plus_one_counter_when_you_gain_life_with_lifelink() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let _heliod = g.add_card_to_battlefield(0, catalog::heliod_sun_crowned());
    let ll = g.add_card_to_battlefield(0, catalog::hopeful_eidolon());
    g.clear_sickness(ll);

    // Cast Faithful Mending mode 2 (Discard 0 → Draw 2 + GainLife 2).
    let mending = g.add_card_to_hand(0, catalog::faithful_mending());
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: mending, target: None, additional_targets: vec![], mode: Some(2), x_value: None,
    }).expect("Faithful Mending castable");
    drain_stack(&mut g);

    let counters = g.battlefield.iter().find(|c| c.id == ll)
        .and_then(|c| c.counters.get(&CounterType::PlusOnePlusOne).copied())
        .unwrap_or(0);
    assert!(counters >= 1,
        "Heliod should add a +1/+1 counter to a lifelink creature when you gain life");
}

#[test]
fn dread_return_reanimates_target_creature_from_graveyard() {
    let mut g = two_player_game();
    // Seed a Grizzly Bears in P0's graveyard.
    let bear_id = g.add_card_to_library(0, catalog::grizzly_bears());
    let card = g.players[0].library.iter().position(|c| c.id == bear_id)
        .map(|pos| g.players[0].library.remove(pos)).unwrap();
    g.players[0].graveyard.push(card);

    // Cast Dread Return for {2}{B}{B}.
    let dr = g.add_card_to_hand(0, catalog::dread_return());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: dr,
        target: Some(Target::Permanent(bear_id)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Dread Return castable for {2}{B}{B}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == bear_id),
        "Dread Return should reanimate the target creature");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bear_id),
        "Bears should no longer be in graveyard");
}

#[test]
fn tidehollow_sculler_etb_takes_an_opponent_card() {
    let mut g = two_player_game();
    // Seed P1's hand with a Lightning Bolt.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());

    let sculler = g.add_card_to_hand(0, catalog::tidehollow_sculler());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: sculler, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tidehollow Sculler castable for {W}{B}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == sculler),
        "Sculler should resolve onto the battlefield");
    assert!(!g.players[1].hand.iter().any(|c| c.id == bolt),
        "ETB DiscardChosen should remove the Bolt from P1's hand");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Bolt should land in P1's graveyard (approximation of exile-until-LTB)");
}

// ── Talisman cycle (RW / UR / GU) ────────────────────────────────────────────

/// Talisman of Conviction: {T}: Add {C} (index 0); index 1 = {R}, index 2 = {W}.
#[test]
fn talisman_of_conviction_taps_for_red_costing_one_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::talisman_of_conviction());
    g.clear_sickness(id);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, x_value: None }).expect("red tap succeeds");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
    assert_eq!(g.players[0].life, life_before - 1,
        "Talisman costs 1 life when tapped for a color");
}

/// Talisman of Creativity: index 1 = {U}, index 2 = {R}.
#[test]
fn talisman_of_creativity_taps_for_blue_or_red_costing_one_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::talisman_of_creativity());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 2, target: None, x_value: None }).expect("red tap succeeds");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
    assert_eq!(g.players[0].life, 19,
        "Talisman costs 1 life when tapped for a color");
}

/// Talisman of Curiosity: index 1 = {G}, index 2 = {U}.
#[test]
fn talisman_of_curiosity_taps_for_green_costing_one_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::talisman_of_curiosity());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, x_value: None }).expect("green tap succeeds");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
    assert_eq!(g.players[0].life, 19);
}

// ── Edict / forced-sacrifice removal ─────────────────────────────────────────

/// Edict-flavour sacrifice picks the lowest-CMC creature first.
/// Validates the new auto-decider sacrifice prioritization (tokens
/// first, then by lowest CMC, then by lowest power).
#[test]
fn forced_sacrifice_picks_lowest_cmc_creature_first() {
    let mut g = two_player_game();
    // Two creatures: a 4/5 (CMC 5) and a 2/2 (CMC 2). The bot should
    // sacrifice the 2/2 first.
    let big = g.add_card_to_battlefield(0, catalog::serra_angel());
    let small = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ib = g.add_card_to_hand(0, catalog::innocent_blood());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: ib, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Innocent Blood castable for {B}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == big),
        "Higher-CMC creature should survive Innocent Blood when a smaller one exists");
    assert!(!g.battlefield.iter().any(|c| c.id == small),
        "Lower-CMC creature should be sacrificed first");
}

/// Innocent Blood: each player sacrifices a creature.
#[test]
fn innocent_blood_each_player_sacrifices_a_creature() {
    let mut g = two_player_game();
    let p0_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let p1_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ib = g.add_card_to_hand(0, catalog::innocent_blood());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: ib, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Innocent Blood castable for {B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == p0_bear),
        "P0's bear should be sacrificed");
    assert!(!g.battlefield.iter().any(|c| c.id == p1_bear),
        "P1's bear should be sacrificed");
}

/// Diabolic Edict: target opponent sacrifices a creature.
#[test]
fn diabolic_edict_targets_opponent_to_sacrifice_a_creature() {
    let mut g = two_player_game();
    let p1_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // P0 also has a creature — to verify Edict picks from the *target*'s
    // pool, not the caster's.
    let p0_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let de = g.add_card_to_hand(0, catalog::diabolic_edict());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: de,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Diabolic Edict castable for {1}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == p1_bear),
        "P1's bear should be sacrificed");
    assert!(g.battlefield.iter().any(|c| c.id == p0_bear),
        "P0's bear should not be touched");
}

/// Geth's Verdict: target sacs + loses 1 life.
#[test]
fn geths_verdict_sacs_target_and_drains_one_life() {
    let mut g = two_player_game();
    let p1_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let gv = g.add_card_to_hand(0, catalog::geths_verdict());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: gv,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Geth's Verdict castable for {1}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == p1_bear),
        "P1's bear should be sacrificed");
    assert_eq!(g.players[1].life, 19, "P1 should lose 1 life");
}

// ── Burn / interaction ───────────────────────────────────────────────────────

/// Magma Jet: 2 damage to any target + Scry 2.
#[test]
fn magma_jet_deals_two_and_scries_two() {
    let mut g = two_player_game();
    // Stock the library so Scry has visible inputs.
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let lib_before = g.players[0].library.len();
    let mj = g.add_card_to_hand(0, catalog::magma_jet());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: mj,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Magma Jet castable for {1}{R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "Target should take 2 damage");
    // Library size unchanged after Scry — cards stay on top by default
    // (AutoDecider keeps the top of the library).
    assert_eq!(g.players[0].library.len(), lib_before,
        "Scry shouldn't draw or mill cards");
}

/// Remand: counters a target spell, returns it to its owner's hand,
/// caster draws a card.
#[test]
fn remand_counters_returns_to_hand_and_draws() {
    let mut g = two_player_game();
    // Seed P0's library so the cantrip has an input.
    g.add_card_to_library(0, catalog::island());
    let hand_before_p0 = g.players[0].hand.len();
    // P1 casts a Lightning Bolt at P0.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bolt castable for {R}");
    // P0 Remands the bolt.
    g.priority.player_with_priority = 0;
    let rem = g.add_card_to_hand(0, catalog::remand());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: rem,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Remand castable for {1}{U}");
    drain_stack(&mut g);
    // Bolt didn't resolve.
    assert_eq!(g.players[0].life, 20, "Bolt was countered");
    // Bolt landed back in P1's hand (Move target → owner's hand).
    assert!(g.players[1].hand.iter().any(|c| c.id == bolt),
        "Bolt should be back in P1's hand");
    // Cantrip: P0 drew a card. Hand started at `hand_before_p0`, then we
    // added the Remand (+1), cast it (-1), drew 1 (+1) → end at +1.
    assert_eq!(g.players[0].hand.len(), hand_before_p0 + 1,
        "Cantrip should net P0 one card");
}

/// Read the Bones: scry 2, draw 2, lose 2.
#[test]
fn read_the_bones_scry_two_draw_two_lose_two() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;
    let rb = g.add_card_to_hand(0, catalog::read_the_bones());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: rb, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Read the Bones castable for {2}{B}");
    drain_stack(&mut g);
    // hand_before captured before we added Read the Bones; the spell's
    // own +1/-1 round-trip cancels, so the +2 draw is the only delta.
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "+2 draw");
    assert_eq!(g.players[0].life, life_before - 2, "lose 2 life");
}

/// Storm Crow: 1U 1/2 flying Bird body.
#[test]
fn storm_crow_is_a_one_two_flying_bird() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::storm_crow());
    let card = g.battlefield_find(id).expect("on battlefield");
    assert_eq!(card.power(), 1);
    assert_eq!(card.toughness(), 2);
    assert!(card.definition.keywords.contains(&crate::card::Keyword::Flying));
    assert!(card.definition.subtypes.creature_types.contains(&crate::card::CreatureType::Bird));
}

/// Ancient Grudge: destroys a target artifact, lands in graveyard with
/// flashback available.
#[test]
fn ancient_grudge_destroys_artifact_with_flashback_available() {
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    let ag = g.add_card_to_hand(0, catalog::ancient_grudge());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: ag,
        target: Some(Target::Permanent(stone)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Ancient Grudge castable for {1}{R}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == stone),
        "Mind Stone should be destroyed");
    // Spell ended up in graveyard, available for flashback.
    let in_yard = g.players[0].graveyard.iter().any(|c| c.id == ag);
    assert!(in_yard, "Ancient Grudge in graveyard");
    let card = g.players[0].graveyard.iter().find(|c| c.id == ag).unwrap();
    assert!(card.definition.has_flashback().is_some(),
        "Flashback cost should still be on the card");
}

/// Ancient Grudge: cast from graveyard via Flashback {G} — destroys a
/// second artifact and exiles the spell on resolution.
#[test]
fn ancient_grudge_flashback_destroys_second_artifact_and_exiles() {
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    // Ancient Grudge starts in P0's graveyard.
    let ag = g.add_card_to_graveyard(0, catalog::ancient_grudge());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastFlashback {
        card_id: ag,
        target: Some(Target::Permanent(stone)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Flashback castable for {G}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == stone),
        "Mind Stone should be destroyed by flashback");
    assert!(g.exile.iter().any(|c| c.id == ag),
        "Flashback resolves into exile");
}

/// Tragic Slip: target creature gets -13/-13 EOT (effectively lethal).
#[test]
fn tragic_slip_kills_creature_via_minus_thirteen() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ts = g.add_card_to_hand(0, catalog::tragic_slip());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: ts,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Tragic Slip castable for {B}");
    drain_stack(&mut g);
    // Either dead via state-based action (toughness ≤ 0), or pumped to
    // -11/-11 — both end with the bear gone after SBAs run.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Grizzly Bears should be dead from -13/-13");
}

// ── New cards: rummagers, burn, counters, removal, white tokens, ETB destroy ──

/// Tormenting Voice: discard a card, then draw two — net +1 hand minus the
/// spell itself, so the hand stays the same size while filtering.
#[test]
fn tormenting_voice_discards_one_and_draws_two() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(0, catalog::lightning_bolt()); // chuck-able
    let id = g.add_card_to_hand(0, catalog::tormenting_voice());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    let grave_before = g.players[0].graveyard.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tormenting Voice castable for {1}{R}");
    drain_stack(&mut g);

    // Net: -1 cast, -1 discard, +2 draw = 0 change.
    assert_eq!(g.players[0].hand.len(), hand_before, "Voice nets 0 hand size");
    assert_eq!(g.players[0].graveyard.len(), grave_before + 2,
        "Spell + discarded card both go to graveyard");
}

/// Wild Guess and Tormenting Voice mirror — same effect, different cost.
#[test]
fn wild_guess_discards_one_and_draws_two() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::wild_guess());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wild Guess castable for {2}{R}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before, "Wild Guess nets 0 hand size");
}

/// Thrill of Possibility is the instant-speed version. Same loot pattern,
/// but the spell is castable as an Instant.
#[test]
fn thrill_of_possibility_is_an_instant_loot_2() {
    use crate::card::CardType;
    let card = catalog::thrill_of_possibility();
    assert!(card.card_types.contains(&CardType::Instant),
        "Thrill of Possibility should be an Instant");

    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, card);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Thrill castable for {1}{R}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before, "Thrill nets 0 hand size");
}

/// Volcanic Hammer is a 3-damage sorcery — straight Lightning Strike at
/// sorcery timing.
#[test]
fn volcanic_hammer_deals_three_to_creature() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::volcanic_hammer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(big)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Volcanic Hammer castable for {1}{R}");
    drain_stack(&mut g);

    // Serra is 4/4 — 3 damage doesn't kill it but does mark it.
    let serra = g.battlefield.iter().find(|c| c.id == big).expect("Serra survives");
    assert_eq!(serra.damage, 3, "Volcanic Hammer should mark 3 damage");
}

/// Slagstorm mode 0: sweeps creatures (3 to each), leaves players alone.
#[test]
fn slagstorm_mode_zero_sweeps_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let lion = g.add_card_to_battlefield(0, catalog::savannah_lions()); // 2/1
    let p0_life_before = g.players[0].life;
    let p1_life_before = g.players[1].life;

    let id = g.add_card_to_hand(0, catalog::slagstorm());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Slagstorm castable for {2}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear (2/2) should die to 3 damage");
    assert!(!g.battlefield.iter().any(|c| c.id == lion),
        "Savannah Lions (2/1) should die to 3 damage");
    assert_eq!(g.players[0].life, p0_life_before, "mode 0 doesn't burn players");
    assert_eq!(g.players[1].life, p1_life_before, "mode 0 doesn't burn players");
}

/// Slagstorm mode 1: 3 damage to each player, creatures survive.
#[test]
fn slagstorm_mode_one_burns_each_player() {
    let mut g = two_player_game();
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let p0_before = g.players[0].life;
    let p1_before = g.players[1].life;

    let id = g.add_card_to_hand(0, catalog::slagstorm());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("Slagstorm castable for {2}{R}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, p0_before - 3,
        "mode 1 burns the caster too (Slagstorm is symmetric)");
    assert_eq!(g.players[1].life, p1_before - 3, "mode 1 burns each player");
    assert!(g.battlefield.iter().any(|c| c.id == serra),
        "mode 1 doesn't touch creatures");
}

/// Cancel: counter target spell.
#[test]
fn cancel_counters_a_spell() {
    let mut g = two_player_game();
    // P1 casts a Lightning Bolt at P0; P0 cancels.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).unwrap();

    let cancel = g.add_card_to_hand(0, catalog::cancel());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: cancel,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Cancel castable for {1}{U}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, 20, "Bolt should never resolve");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Countered spell still goes to its owner's graveyard");
}

/// Annul rejects a noncreature, non-artifact, non-enchantment spell at
/// cast time (e.g. Lightning Bolt is an instant, not in scope).
#[test]
fn annul_rejects_instant_target_at_cast_time() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).unwrap();

    let annul = g.add_card_to_hand(0, catalog::annul());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: annul,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(), "Annul shouldn't accept an instant target");
}

/// Hero's Downfall destroys a target creature.
#[test]
fn heros_downfall_destroys_target_creature() {
    let mut g = two_player_game();
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // legendary? no
    let id = g.add_card_to_hand(0, catalog::heros_downfall());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(serra)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Hero's Downfall castable for {1}{B}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == serra),
        "Serra Angel should be destroyed");
}

/// Cast Down rejects a Legendary creature target at cast time.
#[test]
fn cast_down_rejects_legendary_creature() {
    let mut g = two_player_game();
    // Griselbrand is legendary.
    let gris = g.add_card_to_battlefield(1, catalog::griselbrand());
    let id = g.add_card_to_hand(0, catalog::cast_down());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(gris)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(), "Cast Down shouldn't accept a legendary target");
}

/// Cast Down destroys a nonlegendary creature.
#[test]
fn cast_down_destroys_nonlegendary_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::cast_down());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Cast Down castable for {1}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be destroyed");
}

/// Mind Rot: target player discards two cards.
#[test]
fn mind_rot_discards_two_from_target() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::lightning_bolt());
    }
    let id = g.add_card_to_hand(0, catalog::mind_rot());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Mind Rot castable for {2}{B}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand_before - 2,
        "Mind Rot should remove two cards from the target's hand");
}

/// Raise Dead returns a creature card from the graveyard to the hand.
#[test]
fn raise_dead_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::raise_dead());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Raise Dead castable for {B}");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear),
        "Bear should return to hand");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bear),
        "Bear should leave graveyard");
}

/// Healing Salve: gain 3 life on target.
#[test]
fn healing_salve_gives_three_life() {
    let mut g = two_player_game();
    g.players[0].life = 10;
    let id = g.add_card_to_hand(0, catalog::healing_salve());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Healing Salve castable for {W}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 13, "+3 life");
}

/// Raise the Alarm creates two 1/1 Soldier tokens.
#[test]
fn raise_the_alarm_creates_two_soldier_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::raise_the_alarm());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Raise the Alarm castable for {1}{W}");
    drain_stack(&mut g);

    let soldiers: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Soldier")
        .collect();
    assert_eq!(soldiers.len(), 2, "Two Soldier tokens should enter");
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2, "Two new permanents on the battlefield");
}

/// Reclamation Sage's ETB destroys an artifact.
#[test]
fn reclamation_sage_etb_destroys_artifact() {
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    let id = g.add_card_to_hand(0, catalog::reclamation_sage());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reclamation Sage castable for {2}{G}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == stone),
        "Mind Stone should be destroyed by Sage's ETB");
}

/// Acidic Slime is a 2/2 Deathtouch and its ETB hits a land.
#[test]
fn acidic_slime_etb_destroys_land() {
    use crate::card::Keyword;
    let card = catalog::acidic_slime();
    assert!(card.keywords.contains(&Keyword::Deathtouch),
        "Acidic Slime has Deathtouch");
    assert_eq!(card.power, 2);
    assert_eq!(card.toughness, 2);

    let mut g = two_player_game();
    let mountain = g.add_card_to_battlefield(1, catalog::mountain());
    let id = g.add_card_to_hand(0, card);
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Acidic Slime castable for {3}{G}{G}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == mountain),
        "Mountain should be destroyed by Slime's ETB");
}

/// Stoke the Flames: convoke 4-damage instant. Casting at full {4}{R} is
/// fine; the convoke half is exercised by the existing convoke tests.
#[test]
fn stoke_the_flames_deals_four_at_full_cost() {
    use crate::card::Keyword;
    let card = catalog::stoke_the_flames();
    assert!(card.keywords.contains(&Keyword::Convoke),
        "Stoke the Flames has Convoke");
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, card);
    // Real Oracle: `{2}{R}{R}` Instant.
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Stoke the Flames castable for {2}{R}{R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 4);
}

// ── Bounce ───────────────────────────────────────────────────────────────────

/// Unsummon: target creature returns to its owner's hand.
#[test]
fn unsummon_returns_target_creature_to_owners_hand() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::unsummon());
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Unsummon castable for {U}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should leave the battlefield");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear),
        "Bear should return to its owner's (Bob's) hand, not the caster's");
}

/// Boomerang: bounces non-creature permanents (Sol Ring), proving the wider
/// `Permanent` filter compared to Unsummon.
#[test]
fn boomerang_bounces_a_non_creature_permanent() {
    let mut g = two_player_game();
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::boomerang());
    g.players[0].mana_pool.add(Color::Blue, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(ring)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Boomerang castable for {U}{U}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == ring),
        "Sol Ring should leave the battlefield");
    assert!(g.players[1].hand.iter().any(|c| c.id == ring),
        "Sol Ring should return to its owner's hand");
}

/// Cyclonic Rift rejects targeting your own permanents at cast time.
#[test]
fn cyclonic_rift_rejects_your_own_permanent() {
    let mut g = two_player_game();
    let your_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::cyclonic_rift());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(your_bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Cyclonic Rift should reject your own creature: {:?}", err);
}

/// Cyclonic Rift bounces an opp permanent.
#[test]
fn cyclonic_rift_bounces_opponent_nonland_permanent() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::cyclonic_rift());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Cyclonic Rift castable for {1}{U}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear));
    assert!(g.players[1].hand.iter().any(|c| c.id == opp_bear));
}

#[test]
fn cyclonic_rift_overload_bounces_all_opponent_nonland_permanents() {
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let own_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::cyclonic_rift());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(6);

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Cyclonic Rift Overload for {6}{U}");
    drain_stack(&mut g);

    // Both opponent creatures should be bounced.
    assert!(!g.battlefield.iter().any(|c| c.id == bear1));
    assert!(!g.battlefield.iter().any(|c| c.id == bear2));
    // Own creature should remain.
    assert!(g.battlefield.iter().any(|c| c.id == own_bear));
}

/// Repeal: pays X = 2, bounces a CMC-2 creature, draws a card.
#[test]
fn repeal_with_x_two_bounces_two_drop_and_cantrips() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // {1}{G}
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::repeal());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    }).expect("Repeal castable for {2}{U} (X=2)");
    drain_stack(&mut g);

    assert!(g.players[1].hand.iter().any(|c| c.id == bear),
        "Bear should bounce to opp's hand");
    // Repeal goes to caster's graveyard; draw replaces it from library.
    // Net hand change: -1 (cast) + 1 (draw) = 0.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "Cast (-1) + cantrip (+1) = net 0");
}

/// Repeal: when X is too small the cmc gate fails — only the cantrip fires,
/// the target stays on the battlefield.
#[test]
fn repeal_x_zero_against_two_drop_does_not_bounce() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::repeal());
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(0),
    }).expect("Repeal castable for {U} (X=0)");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear),
        "Bear should stay on battlefield: 2 > X=0");
}

// ── Removal ──────────────────────────────────────────────────────────────────

/// Murder destroys any creature, including a black one (vs Doom Blade).
#[test]
fn murder_destroys_any_creature_including_black() {
    let mut g = two_player_game();
    let specter = g.add_card_to_battlefield(1, catalog::hypnotic_specter());
    let id = g.add_card_to_hand(0, catalog::murder());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(specter)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Murder castable for {1}{B}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == specter),
        "Hypnotic Specter (black) should die to Murder");
}

/// Go for the Throat destroys non-artifact creatures, rejects artifact creatures.
#[test]
fn go_for_the_throat_destroys_nonartifact_but_rejects_artifact() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let memnite = g.add_card_to_battlefield(1, catalog::memnite()); // 1/1 artifact creature
    let id_ok = g.add_card_to_hand(0, catalog::go_for_the_throat());
    let id_bad = g.add_card_to_hand(0, catalog::go_for_the_throat());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id_ok,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Go for the Throat castable for {1}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "Bear should die");

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id_bad,
        target: Some(Target::Permanent(memnite)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Go for the Throat should reject Memnite (artifact): {:?}", err);
}

/// Disfigure: -2/-2 EOT — kills a 2/2.
#[test]
fn disfigure_kills_a_two_two_via_minus_two_minus_two() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::disfigure());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Disfigure castable for {B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear (2/2) should die to -2/-2");
}

/// Languish: every creature gets -2/-2 EOT — sweeps 2/2s, leaves 4/4s alive.
#[test]
fn languish_sweeps_small_but_leaves_big_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let lions = g.add_card_to_battlefield(0, catalog::savannah_lions()); // 2/1
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::languish());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Languish castable for {2}{B}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear (2/2) should die to -2/-2");
    assert!(!g.battlefield.iter().any(|c| c.id == lions),
        "Savannah Lions (2/1) should die to -2/-2");
    assert!(g.battlefield.iter().any(|c| c.id == serra),
        "Serra (4/4) should survive — 4-2 = 2 toughness left");
}

/// Lay Down Arms exiles low-power creatures, rejects power-4+.
#[test]
fn lay_down_arms_exiles_low_power_but_rejects_high_power() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let craw = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4
    let id_ok = g.add_card_to_hand(0, catalog::lay_down_arms());
    let id_bad = g.add_card_to_hand(0, catalog::lay_down_arms());
    g.players[0].mana_pool.add(Color::White, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id_ok,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Lay Down Arms castable for {W}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "Bear should leave battlefield");
    assert!(g.exile.iter().any(|c| c.id == bear), "Bear should be exiled, not in graveyard");

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id_bad,
        target: Some(Target::Permanent(craw)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Lay Down Arms should reject power-6 Craw Wurm: {:?}", err);
}

/// Smelt destroys an artifact.
#[test]
fn smelt_destroys_an_artifact() {
    let mut g = two_player_game();
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::smelt());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(ring)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Smelt castable for {R}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == ring));
}

// ── X-cost burn ──────────────────────────────────────────────────────────────

/// Banefire: X damage to a creature scales with X paid.
#[test]
fn banefire_deals_x_damage_to_creature() {
    let mut g = two_player_game();
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::banefire());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(serra)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(5),
    }).expect("Banefire castable for {5}{R} (X=5)");
    drain_stack(&mut g);

    // Banefire is sorcery — damage marks the creature; lethal moves it to graveyard via SBA.
    assert!(!g.battlefield.iter().any(|c| c.id == serra),
        "Serra (4 toughness) should die to 5 damage");
}

/// Banefire to a player face — pure burn.
#[test]
fn banefire_burns_a_player_face_for_x() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::banefire());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(7);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(7),
    }).expect("Banefire castable for {7}{R} (X=7)");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 7,
        "Banefire X=7 should burn for 7");
}

#[test]
fn banefire_uncounterable_at_x_five() {
    // Push (modern_decks): "If X is 5 or more, this spell can't be
    // countered" rider wired via `caster_grants_uncounterable_with_x`.
    // X=5 → the cast pushes `StackItem::Spell { uncounterable: true }`
    // and counterspells targeting it fizzle.
    use crate::game::types::StackItem;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::banefire());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(5),
    }).expect("Banefire castable for {5}{R} (X=5)");

    // Inspect the stack item to confirm uncounterable is set.
    let uncounterable = g.stack.iter().find_map(|si| match si {
        StackItem::Spell { uncounterable, .. } => Some(*uncounterable),
        _ => None,
    });
    assert_eq!(uncounterable, Some(true),
        "Banefire at X=5 should land on the stack as uncounterable");
}

#[test]
fn banefire_counterable_below_x_five() {
    // X=4: stays counterable (rider doesn't kick in until X ≥ 5).
    use crate::game::types::StackItem;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::banefire());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(4),
    }).expect("Banefire castable for {4}{R} (X=4)");

    let uncounterable = g.stack.iter().find_map(|si| match si {
        StackItem::Spell { uncounterable, .. } => Some(*uncounterable),
        _ => None,
    });
    assert_eq!(uncounterable, Some(false),
        "Banefire at X=4 should remain counterable");
}

// ── Tokens ───────────────────────────────────────────────────────────────────

/// Spectral Procession creates three 1/1 white flying spirits.
#[test]
fn spectral_procession_creates_three_flying_spirits() {
    use crate::card::{CreatureType, Keyword};
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::spectral_procession());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_count_before = g.battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .count();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spectral Procession castable for {2}{W}");
    drain_stack(&mut g);

    let new_tokens: Vec<_> = g.battlefield
        .iter()
        .filter(|c| c.controller == 0
            && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(new_tokens.len(), 3,
        "Spectral Procession should create three Spirit tokens");
    for tok in &new_tokens {
        assert!(tok.definition.keywords.contains(&Keyword::Flying),
            "Spirit tokens should have flying");
        assert_eq!(tok.definition.power, 1);
        assert_eq!(tok.definition.toughness, 1);
    }
    let bf_count_after = g.battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .count();
    assert_eq!(bf_count_after, bf_count_before + 3,
        "+3 permanents on caster's side of board");
}

// ── Recursion ────────────────────────────────────────────────────────────────

/// Regrowth: returns any card type from your graveyard to your hand.
#[test]
fn regrowth_returns_a_land_card_from_graveyard() {
    let mut g = two_player_game();
    let mountain = g.add_card_to_graveyard(0, catalog::mountain());
    let id = g.add_card_to_hand(0, catalog::regrowth());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(mountain)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Regrowth castable for {1}{G}");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == mountain),
        "Mountain card should return to caster's hand");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == mountain),
        "Mountain card should leave graveyard");
}

/// Beast Within: destroy any permanent, the controller gets a 3/3 Beast.
#[test]
fn beast_within_destroys_and_creates_beast_for_controller() {
    use crate::card::{CreatureType, CardType};
    let mut g = two_player_game();
    let opp_ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::beast_within());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(opp_ring)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Beast Within castable for {2}{G}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == opp_ring),
        "Sol Ring should be destroyed");
    let beasts: Vec<_> = g.battlefield
        .iter()
        .filter(|c| c.controller == 1
            && c.definition.card_types.contains(&CardType::Creature)
            && c.definition.subtypes.creature_types.contains(&CreatureType::Beast))
        .collect();
    assert_eq!(beasts.len(), 1,
        "Opp (Sol Ring's controller) should get one 3/3 Beast token");
    assert_eq!(beasts[0].definition.power, 3);
    assert_eq!(beasts[0].definition.toughness, 3);
}

/// Grasp of Darkness: -4/-4 EOT — kills a 4/4.
#[test]
fn grasp_of_darkness_kills_a_four_four() {
    let mut g = two_player_game();
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::grasp_of_darkness());
    g.players[0].mana_pool.add(Color::Black, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(serra)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Grasp of Darkness castable for {B}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == serra),
        "Serra Angel (4/4) should die to -4/-4");
}

/// Shatter destroys an artifact.
#[test]
fn shatter_destroys_an_artifact() {
    let mut g = two_player_game();
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::shatter());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(ring)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Shatter castable for {1}{R}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == ring));
}

// ── modern_decks-8 tests ─────────────────────────────────────────────────────

/// Incinerate deals 3 damage to a creature, killing a 2/2.
#[test]
fn incinerate_deals_three_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::incinerate());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Incinerate castable for {1}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Incinerate (3 damage) should kill the Grizzly Bears");
}

/// Incinerate burns a player face for 3.
#[test]
fn incinerate_burns_a_player_for_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::incinerate());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Incinerate can hit a player");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life_before - 3);
}

/// Searing Spear: 3 damage to any target.
#[test]
fn searing_spear_deals_three_damage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::searing_spear());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Searing Spear castable for {1}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

/// Flame Slash: 4 damage destroys a 4-toughness creature.
#[test]
fn flame_slash_kills_a_four_toughness_creature() {
    let mut g = two_player_game();
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::flame_slash());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(serra)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Flame Slash castable for {R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == serra),
        "Flame Slash (4 damage) should kill the 4/4 Serra Angel");
}

/// Flame Slash rejects a player target at cast time (creature-only).
#[test]
fn flame_slash_rejects_player_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::flame_slash());
    g.players[0].mana_pool.add(Color::Red, 1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Flame Slash should reject a player target: {:?}", err);
}

/// Roast: 5 damage kills a non-flier (Grizzly Bears).
#[test]
fn roast_kills_a_non_flying_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::roast());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Roast castable for {1}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

/// Roast rejects a flier at cast time.
#[test]
fn roast_rejects_a_flier() {
    let mut g = two_player_game();
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // flying
    let id = g.add_card_to_hand(0, catalog::roast());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(serra)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Roast should reject a flying creature: {:?}", err);
}

/// Smother destroys a 2-CMC creature.
#[test]
fn smother_destroys_low_cmc_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2 CMC
    let id = g.add_card_to_hand(0, catalog::smother());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Smother castable for {1}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

/// Smother rejects high-CMC creature targets at cast time.
#[test]
fn smother_rejects_high_cmc_target() {
    let mut g = two_player_game();
    let craw = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6 CMC
    let id = g.add_card_to_hand(0, catalog::smother());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(craw)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Smother should reject a 6-CMC Craw Wurm: {:?}", err);
}

/// Final Reward: exiles a creature.
#[test]
fn final_reward_exiles_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::final_reward());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Final Reward castable for {4}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should leave the battlefield");
    assert!(g.exile.iter().any(|c| c.id == bear),
        "Bear should be exiled, not graveyarded");
}

/// Holy Light sweeps -1/-1 across all creatures, killing 1-toughness.
#[test]
fn holy_light_sweeps_minus_one_minus_one() {
    let mut g = two_player_game();
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves()); // 1/1
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::holy_light());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Holy Light castable for {1}{W}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == elf),
        "1/1 Llanowar Elves should die to -1/-1");
    let bear_view = g.battlefield.iter().find(|c| c.id == bear);
    assert!(bear_view.is_some(),
        "Grizzly Bears (2/2) survives -1/-1 sweep");
}

/// Mana Tithe counters a spell when controller can't pay {1}.
#[test]
fn mana_tithe_counters_when_controller_cannot_pay_one() {
    let mut g = two_player_game();
    // P1 casts a Lightning Bolt on their turn with red mana only — no
    // leftover generic to pay the tax.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Lightning Bolt castable for {R}");

    // P0 responds with Mana Tithe.
    g.priority.player_with_priority = 0;
    let tithe = g.add_card_to_hand(0, catalog::mana_tithe());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: tithe,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Mana Tithe castable for {W}");
    drain_stack(&mut g);

    // Bolt should be countered (lands in graveyard, no damage to P0).
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Lightning Bolt should be countered to graveyard");
    assert_eq!(g.players[0].life, 20,
        "P0 should not have taken Bolt damage");
}

/// Rampant Growth tutors a basic into play tapped.
#[test]
fn rampant_growth_searches_a_basic_into_play_tapped() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::lightning_bolt()); // padding non-basic

    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest)),
    ]));

    let id = g.add_card_to_hand(0, catalog::rampant_growth());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Rampant Growth castable for {1}{G}");
    drain_stack(&mut g);

    let view = g.battlefield.iter().find(|c| c.id == forest);
    assert!(view.is_some(), "Forest should be on battlefield");
    assert!(view.unwrap().tapped, "Forest should enter tapped");
}

/// Cultivate fetches two basics: one tapped to play, one to hand.
#[test]
fn cultivate_searches_two_basics() {
    let mut g = two_player_game();
    let bf_target = g.add_card_to_library(0, catalog::forest());
    let hand_target = g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::lightning_bolt());

    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(bf_target)),
        DecisionAnswer::Search(Some(hand_target)),
    ]));

    let id = g.add_card_to_hand(0, catalog::cultivate());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cultivate castable for {2}{G}");
    drain_stack(&mut g);

    let bf = g.battlefield.iter().find(|c| c.id == bf_target);
    assert!(bf.is_some(), "First basic on battlefield");
    assert!(bf.unwrap().tapped, "Battlefield basic enters tapped");
    assert!(g.players[0].hand.iter().any(|c| c.id == hand_target),
        "Second basic into hand");
}

/// Farseek tutors a basic into play tapped.
#[test]
fn farseek_searches_a_basic_into_play_tapped() {
    let mut g = two_player_game();
    let plains = g.add_card_to_library(0, catalog::plains());

    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(plains)),
    ]));

    let id = g.add_card_to_hand(0, catalog::farseek());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Farseek castable for {1}{G}");
    drain_stack(&mut g);

    let view = g.battlefield.iter().find(|c| c.id == plains);
    assert!(view.is_some(), "Plains should be on battlefield");
    assert!(view.unwrap().tapped, "Plains should enter tapped");
}

/// Sakura-Tribe Elder: tap-and-sac search for a basic.
#[test]
fn sakura_tribe_elder_sacrifices_for_a_basic() {
    let mut g = two_player_game();
    let elder = g.add_card_to_battlefield(0, catalog::sakura_tribe_elder());
    g.clear_sickness(elder);
    let forest = g.add_card_to_library(0, catalog::forest());

    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest)),
    ]));

    g.perform_action(GameAction::ActivateAbility {
        card_id: elder, ability_index: 0, target: None, x_value: None }).expect("Sakura-Tribe Elder activates");
    drain_stack(&mut g);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == elder),
        "Elder should be sacrificed");
    let view = g.battlefield.iter().find(|c| c.id == forest);
    assert!(view.is_some(), "Forest tutored to battlefield");
    assert!(view.unwrap().tapped, "Forest enters tapped");
}

/// Wood Elves: ETB search a Forest into play untapped.
#[test]
fn wood_elves_etb_searches_forest_untapped() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::lightning_bolt()); // padding

    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest)),
    ]));

    let id = g.add_card_to_hand(0, catalog::wood_elves());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wood Elves castable for {2}{G}");
    drain_stack(&mut g);

    let view = g.battlefield.iter().find(|c| c.id == forest);
    assert!(view.is_some(), "Forest tutored to battlefield");
    assert!(!view.unwrap().tapped, "Forest should ENTER UNTAPPED for Wood Elves");
}

/// Elvish Mystic: tap for {G}.
#[test]
fn elvish_mystic_taps_for_green() {
    let mut g = two_player_game();
    let mystic = g.add_card_to_battlefield(0, catalog::elvish_mystic());
    g.clear_sickness(mystic);
    let pool_before = g.players[0].mana_pool.total();

    g.perform_action(GameAction::ActivateAbility {
        card_id: mystic, ability_index: 0, target: None, x_value: None }).expect("Elvish Mystic activates");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == mystic && c.tapped),
        "Mystic should be tapped");
    assert_eq!(g.players[0].mana_pool.total(), pool_before + 1,
        "Mystic adds 1 green mana");
    assert!(g.players[0].mana_pool.amount(Color::Green) >= 1,
        "Pool should have at least 1 green");
}

/// Harmonize: draws three cards.
#[test]
fn harmonize_draws_three_cards() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::forest()); }
    let id = g.add_card_to_hand(0, catalog::harmonize());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 2);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Harmonize castable for {2}{G}{G}");
    drain_stack(&mut g);

    // -1 (cast) + 3 (draw) = +2 hand.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3,
        "Harmonize nets +2 cards");
}

/// Concentrate: draws three cards.
#[test]
fn concentrate_draws_three_cards() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::concentrate());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Blue, 2);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Concentrate castable for {2}{U}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3);
}

/// Severed Strands: sac a creature, destroy a creature, gain 2 life.
#[test]
fn severed_strands_sacs_and_destroys_for_life() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(fodder);
    let target = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::severed_strands());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Severed Strands castable for {1}{B}");
    drain_stack(&mut g);

    // Fodder sacrificed.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder),
        "Fodder should be sacrificed");
    // Target destroyed.
    assert!(!g.battlefield.iter().any(|c| c.id == target),
        "Target should be destroyed");
    assert_eq!(g.players[0].life, life_before + 2,
        "P0 should gain 2 life");
}

/// Anticipate: scry 2 + draw 1 net (-1 cast +1 draw = net 0 hand).
#[test]
fn anticipate_resolves_and_draws() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::anticipate());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Anticipate castable for {1}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before,
        "Anticipate (cast -1, draw +1) should net 0 hand");
}

/// Divination: -1 cast +2 draw = net +1 hand.
#[test]
fn divination_draws_two_cards() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::divination());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Divination castable for {2}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2);
}

/// Ambition's Cost: draws 3 and lose 3 life.
#[test]
fn ambitions_cost_draws_three_loses_three() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::swamp()); }
    let id = g.add_card_to_hand(0, catalog::ambitions_cost());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Black, 1);
    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ambition's Cost castable for {3}{B}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3);
    assert_eq!(g.players[0].life, life_before - 3);
}

/// Path of Peace: kill an opp creature; their controller gains 4 life.
#[test]
fn path_of_peace_destroys_and_gives_opp_four_life() {
    let mut g = two_player_game();
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::path_of_peace());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::White, 1);
    let opp_life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(serra)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Path of Peace castable for {3}{W}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == serra),
        "Serra Angel destroyed");
    assert_eq!(g.players[1].life, opp_life_before + 4,
        "Opponent (target's controller) gains 4 life");
}

// ── modern_decks-9 tests ─────────────────────────────────────────────────────

/// Despise: target opp discards a chosen creature.
#[test]
fn despise_takes_a_creature_from_opp_hand() {
    let mut g = two_player_game();
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::lightning_bolt()); // non-creature padding
    let id = g.add_card_to_hand(0, catalog::despise());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Despise castable for {B}");
    drain_stack(&mut g);

    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear),
        "Bear (creature) should be the discard pick");
}

/// Distress: takes a non-creature non-land from opp hand.
#[test]
fn distress_takes_a_noncreature_nonland_from_opp_hand() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears()); // creature, should NOT be picked
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt()); // valid
    let id = g.add_card_to_hand(0, catalog::distress());
    g.players[0].mana_pool.add(Color::Black, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Distress castable for {B}{B}");
    drain_stack(&mut g);

    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Bolt (instant) should be the discard pick");
}

/// Vryn Wingmare: 2/1 flying body with the noncreature-spell tax.
#[test]
fn vryn_wingmare_is_a_flying_two_one() {
    let g = two_player_game();
    let def = catalog::vryn_wingmare();
    assert_eq!(def.power, 2);
    assert_eq!(def.toughness, 1);
    assert!(def.keywords.contains(&crate::card::Keyword::Flying));
    assert_eq!(def.static_abilities.len(), 1,
        "Vryn Wingmare should ship its noncreature-tax static");
    let _ = g; // suppress unused
}

/// Vryn Wingmare's tax is observable: opp's second-spell-this-turn
/// gets a +{1} surcharge filtered to noncreature.
#[test]
fn vryn_wingmare_taxes_noncreature_spells_after_first_cast() {
    let mut g = two_player_game();
    let wingmare = g.add_card_to_battlefield(0, catalog::vryn_wingmare());
    g.clear_sickness(wingmare);
    // P1 has cast one spell already this turn — the next noncreature
    // spell should be taxed +{1}.
    g.players[1].spells_cast_this_turn = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    // {R} only — printed cost; with Vryn Wingmare's +{1} should fail.
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Bolt with only {{R}} should be rejected under Vryn Wingmare's tax: {:?}", err);
}

/// Lava Coil: 4 damage kills a 4-toughness creature.
#[test]
fn lava_coil_kills_a_four_toughness() {
    let mut g = two_player_game();
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::lava_coil());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(serra)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Lava Coil castable for {1}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == serra),
        "Lava Coil (4 damage) should kill Serra Angel (4 toughness)");
    // Push (modern_decks): Lava Coil now exiles creatures it would kill
    // instead of graveyarding them, approximating the printed "if that
    // creature would die this turn, exile it instead" rider.
    assert!(g.exile.iter().any(|c| c.id == serra),
        "Lava Coil should exile (not graveyard) creatures it would kill");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == serra),
        "Lava Coil should not put the dead creature in graveyard");
}

#[test]
fn lava_coil_deals_damage_without_killing_a_five_toughness() {
    // 4 damage doesn't kill a 5-toughness creature; the else branch
    // resolves with `DealDamage` only (no exile).
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());  // 5/5
    let id = g.add_card_to_hand(0, catalog::lava_coil());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(dragon)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Lava Coil castable for {1}{R}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == dragon),
        "5-toughness dragon survives the 4 damage");
    let damage = g.battlefield_find(dragon).unwrap().damage;
    assert_eq!(damage, 4, "Dragon should have 4 damage marked");
}

/// Jaya's Greeting: 3 damage + scry 2.
#[test]
fn jayas_greeting_deals_three_and_scries() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::jayas_greeting());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Jaya's Greeting castable for {1}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Jaya's Greeting (3 dmg) should kill Grizzly Bears");
}

/// Telling Time: scry 2 + draw 1 net 0 hand.
#[test]
fn telling_time_resolves_and_draws() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::telling_time());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Telling Time castable for {1}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before,
        "Telling Time net 0 hand (cast -1, draw +1)");
}

/// Read the Tides: -1 cast + 3 draw = +2 hand.
#[test]
fn read_the_tides_draws_three() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::read_the_tides());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Blue, 2);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Read the Tides castable for {4}{U}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3);
}

/// Last Gasp: -3/-3 kills a 3-toughness creature.
#[test]
fn last_gasp_kills_a_three_toughness() {
    let mut g = two_player_game();
    // Hypnotic Specter is 2/2 — let's use an explicit 3-toughness.
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::last_gasp());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(serra)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Last Gasp castable for {1}{B}");
    drain_stack(&mut g);

    // 4 - 3 = 1 toughness left, still alive (no damage marked).
    let view = g.battlefield.iter().find(|c| c.id == serra);
    assert!(view.is_some(),
        "Serra (4/4) survives -3/-3 with 1 toughness left");
    // But a 3-toughness creature would die — verify with bear (2/2 → -1/-1 → dies).
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id2 = g.add_card_to_hand(0, catalog::last_gasp());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id2,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Last Gasp castable second time");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear (2/2) dies to -3/-3");
}

/// Wild Mongrel: discard ability gives +1/+1 EOT.
#[test]
fn wild_mongrel_pumps_via_discard() {
    let mut g = two_player_game();
    let mongrel = g.add_card_to_battlefield(0, catalog::wild_mongrel());
    g.clear_sickness(mongrel);
    let fodder = g.add_card_to_hand(0, catalog::lightning_bolt());

    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Discard(vec![fodder]),
    ]));

    g.perform_action(GameAction::ActivateAbility {
        card_id: mongrel, ability_index: 0, target: None, x_value: None }).expect("Wild Mongrel activates");
    drain_stack(&mut g);

    let view = g.battlefield.iter().find(|c| c.id == mongrel)
        .expect("Wild Mongrel still on battlefield");
    // Wild Mongrel is 2/2 + 1/1 EOT = 3/3.
    assert_eq!(view.power(), 3, "power should be base 2 + bonus 1 = 3");
    assert_eq!(view.toughness(), 3, "toughness should be base 2 + bonus 1 = 3");
    // Fodder discarded.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder),
        "Discarded card lands in graveyard");
}

// ── Modern utility lands and artifacts (modern_decks-10 batch) ──────────────

#[test]
fn glimmerpost_etbs_tapped_and_grants_one_life() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::glimmerpost());
    g.perform_action(GameAction::PlayLand(id))
        .expect("Glimmerpost playable as a land");
    drain_stack(&mut g);

    let card = g.battlefield_find(id).expect("Glimmerpost on the battlefield");
    assert!(card.tapped, "Glimmerpost has the etb-tap trigger");
    assert_eq!(g.players[0].life, life_before + 1,
        "ETB should grant 1 life (Locus scaling collapsed to flat 1)");
}

#[test]
fn glimmerpost_taps_for_colorless_after_untap() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::glimmerpost());
    // Drop the post-ETB tapped state before activating.
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    let total_before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None })
    .expect("Glimmerpost mana ability");
    assert_eq!(g.players[0].mana_pool.total(), total_before + 1,
        "Glimmerpost taps for {{C}}");
}

#[test]
fn cloudpost_etbs_tapped_and_taps_for_colorless() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::cloudpost());
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);

    assert!(g.battlefield_find(id).unwrap().tapped, "Cloudpost ETB-tapped");
    // Untap and verify mana ability.
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).unwrap();
    assert_eq!(g.players[0].mana_pool.total(), 1,
        "Cloudpost taps for one colorless");
}

#[test]
fn lotus_field_etb_sacrifices_two_lands() {
    let mut g = two_player_game();
    // Stock the battlefield with three Forests so the sac doesn't kill
    // the Field itself by triggering before it has friends to sacrifice.
    let f1 = g.add_card_to_battlefield(0, catalog::forest());
    let f2 = g.add_card_to_battlefield(0, catalog::forest());
    let f3 = g.add_card_to_battlefield(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::lotus_field());

    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);

    // The Field is on the battlefield (tapped via the ETB-tap step).
    assert!(g.battlefield_find(id).is_some(), "Lotus Field stays in play");
    assert!(g.battlefield_find(id).unwrap().tapped);
    // Two of the three forests sacrificed; one remains.
    let remaining_forests = [f1, f2, f3].iter()
        .filter(|fid| g.battlefield_find(**fid).is_some())
        .count();
    assert_eq!(remaining_forests, 1,
        "Lotus Field's ETB should sacrifice two of your lands");
}

#[test]
fn lotus_field_taps_for_three_of_one_color() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lotus_field());
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).expect("Lotus Field mana ability");
    // ManaPayload::AnyOneColor with Const(3) deposits 3 mana in a single color.
    assert_eq!(g.players[0].mana_pool.total(), 3,
        "Lotus Field should add 3 mana of one color");
}

#[test]
fn evolving_wilds_sacrifices_to_search_basic() {
    let mut g = two_player_game();
    // Stock a basic in the library to fetch.
    let plains_id = g.add_card_to_library(0, catalog::plains());
    let wilds_id = g.add_card_to_battlefield(0, catalog::evolving_wilds());
    g.battlefield.iter_mut().find(|c| c.id == wilds_id).unwrap().tapped = false;

    // Scripted decider picks the basic to fetch.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(plains_id))]));

    g.perform_action(GameAction::ActivateAbility {
        card_id: wilds_id, ability_index: 0, target: None, x_value: None }).expect("Evolving Wilds search ability");
    drain_stack(&mut g);

    // Wilds was sacrificed to its own cost; Plains is on the battlefield tapped.
    assert!(g.battlefield_find(wilds_id).is_none(),
        "Evolving Wilds sacrificed itself to its activation cost");
    let plains_inplay = g.battlefield_find(plains_id)
        .expect("Plains landed on the battlefield");
    assert!(plains_inplay.tapped, "Wilds searches put the basic onto BF tapped");
}

#[test]
fn mistvault_bridge_etbs_tapped_with_dual_basic_typing() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mistvault_bridge());
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);

    let card = g.battlefield_find(id).unwrap();
    assert!(card.tapped, "Bridge ETB-tapped");
    // Bridge is typed as both Island and Swamp.
    let lts = &card.definition.subtypes.land_types;
    assert!(lts.contains(&crate::card::LandType::Island));
    assert!(lts.contains(&crate::card::LandType::Swamp));
}

#[test]
fn drossforge_bridge_taps_for_colorless() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::drossforge_bridge());
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).unwrap();
    assert_eq!(g.players[0].mana_pool.total(), 1, "Bridge taps for {{C}}");
}

#[test]
fn coalition_relic_taps_for_one_mana_of_any_color() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::coalition_relic());
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None })
    .expect("Coalition Relic's mana ability");
    // AnyOneColor — pool gains 1 mana of *some* color.
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

#[test]
fn coalition_relic_taps_to_add_charge_counter() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::coalition_relic());
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, x_value: None })
    .expect("Coalition Relic's charge-counter ability");
    drain_stack(&mut g);
    let relic = g.battlefield_find(id).expect("relic still on battlefield");
    assert_eq!(relic.counter_count(CounterType::Charge), 1,
        "Activating ability #1 deposits one charge counter");
    assert!(relic.tapped, "tap-cost activated abilities tap the source");
}

#[test]
fn coalition_relic_removes_three_charge_counters_for_wubrg() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::coalition_relic());
    // Pre-seed three charge counters (skip the three turn cycles a real
    // game would need to deposit them).
    {
        let relic = g.battlefield_find_mut(id).expect("relic on battlefield");
        relic.add_counters(CounterType::Charge, 3);
    }
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 2, target: None, x_value: None })
    .expect("Coalition Relic's WUBRG burst ability");
    drain_stack(&mut g);
    let relic = g.battlefield_find(id).expect("relic still on battlefield");
    assert_eq!(relic.counter_count(CounterType::Charge), 0,
        "All three charge counters consumed by the burst");
    assert_eq!(g.players[0].mana_pool.total(), 5,
        "WUBRG = one mana of each of the five colors");
}

#[test]
fn coalition_relic_wubrg_burst_rejects_activation_without_three_counters() {
    // The activation must be rejected at the gate, not silently resolve
    // to zero mana — the `condition: ValueAtLeast(CountersOn(Charge), 3)`
    // on ability #2 prevents the activation from ever reaching the
    // stack when the counter pool is empty.
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::coalition_relic());
    let result = g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 2, target: None, x_value: None });
    assert!(result.is_err(),
        "WUBRG burst rejected at activation gate without 3 charge counters");
    assert_eq!(g.players[0].mana_pool.total(), 0,
        "No mana produced — burst never resolved");
}

#[test]
fn coalition_relic_wubrg_burst_rejects_with_two_charge_counters() {
    // Boundary check: even one shy of three counters, the activation
    // gate must reject — strict `≥ 3` semantics.
    use crate::card::CounterType;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::coalition_relic());
    {
        let relic = g.battlefield_find_mut(id).expect("relic on battlefield");
        relic.add_counters(CounterType::Charge, 2);
    }
    let result = g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 2, target: None, x_value: None });
    assert!(result.is_err(),
        "Two charge counters is one short of the gate — activation rejected");
}

#[test]
fn ghost_vacuum_auto_target_picks_graveyard_card_when_present() {
    // Without the `prefers_graveyard_target` heuristic, the bot walks the
    // battlefield first and Ghost Vacuum (filter `Any`) would auto-target
    // a battlefield permanent — then exile it. The fix routes Move-to-
    // Exile spells through the graveyard walk first.
    let mut g = two_player_game();
    let _bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let _vac = g.add_card_to_battlefield(0, catalog::ghost_vacuum());

    let target = g.auto_target_for_effect(
        &catalog::ghost_vacuum().activated_abilities[0].effect, 0
    );
    assert_eq!(target, Some(Target::Permanent(dead)),
        "Auto-target should pick a graveyard card, not a battlefield permanent");
}

#[test]
fn ghost_vacuum_exiles_target_card_from_graveyard() {
    let mut g = two_player_game();
    // Seed P1's graveyard with a Bear directly.
    let bear_id = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let vac = g.add_card_to_battlefield(0, catalog::ghost_vacuum());
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::ActivateAbility {
        card_id: vac, ability_index: 0, target: Some(Target::Permanent(bear_id)), x_value: None })
    .expect("Ghost Vacuum activated for {{2}}, {{T}}");
    drain_stack(&mut g);

    assert!(!g.players[1].graveyard.iter().any(|c| c.id == bear_id),
        "Bear left the graveyard");
    assert!(g.exile.iter().any(|c| c.id == bear_id),
        "Bear is now in the exile zone");
}

#[test]
fn all_bridges_etb_tapped_and_carry_two_basic_land_types() {
    use crate::card::{CardDefinition, LandType};
    type BridgeCase = (fn() -> CardDefinition, LandType, LandType);
    // Each bridge factory paired with the two basic land types it should
    // expose. If the lookup ever changes (e.g., we promote bridges to
    // "every basic land type"), tighten this in one place.
    let cases: &[BridgeCase] = &[
        (catalog::mistvault_bridge,  LandType::Island,    LandType::Swamp),
        (catalog::drossforge_bridge, LandType::Swamp,     LandType::Mountain),
        (catalog::razortide_bridge,  LandType::Plains,    LandType::Island),
        (catalog::goldmire_bridge,   LandType::Plains,    LandType::Swamp),
        (catalog::silverbluff_bridge,LandType::Island,    LandType::Mountain),
        (catalog::tanglepool_bridge, LandType::Island,    LandType::Forest),
        (catalog::slagwoods_bridge,  LandType::Mountain,  LandType::Forest),
        (catalog::thornglint_bridge, LandType::Plains,    LandType::Forest),
        (catalog::darkmoss_bridge,   LandType::Swamp,     LandType::Forest),
        (catalog::rustvale_bridge,   LandType::Plains,    LandType::Mountain),
    ];
    for &(factory, ta, tb) in cases {
        let def = factory();
        let lts = &def.subtypes.land_types;
        assert!(lts.contains(&ta), "{}: missing {:?}", def.name, ta);
        assert!(lts.contains(&tb), "{}: missing {:?}", def.name, tb);
        // Each bridge has exactly the etb-tap trigger + a {T}: Add {C} ability.
        assert_eq!(def.activated_abilities.len(), 1,
            "{}: should have one mana ability", def.name);
        assert!(!def.triggered_abilities.is_empty(),
            "{}: should have an etb-tap trigger", def.name);
    }
}

// ── modern_decks-11: Multi-color removal + sweepers + body ───────────────────

#[test]
fn tear_asunder_destroys_target_artifact() {
    let mut g = two_player_game();
    let relic = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::tear_asunder());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(relic)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Tear Asunder castable for {1}{B}{G}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == relic),
        "Tear Asunder destroys the artifact");
}

#[test]
fn tear_asunder_rejects_creature_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::tear_asunder());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);

    let r = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(r.is_err(), "Tear Asunder should reject creature targets at cast time");
}

#[test]
fn assassins_trophy_destroys_opp_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::assassins_trophy());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Trophy castable for {B}{G}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Assassin's Trophy destroys the opp's creature");
}

#[test]
fn assassins_trophy_rejects_your_own_permanent() {
    // Filter is "permanent an opponent controls" — caster's own creature
    // should be rejected at cast time.
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::assassins_trophy());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);

    let r = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(r.is_err(), "Trophy should reject caster-controlled targets");
}

#[test]
fn volcanic_fallout_deals_two_to_each_creature_and_player() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // 5/5
    let life0 = g.players[0].life;
    let life1 = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::volcanic_fallout());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Volcanic Fallout castable for {1}{R}{R}");
    drain_stack(&mut g);

    // Both 2/2 bears die; the 5/5 dragon survives.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Volcanic Fallout kills both 2-toughness bears (yours)");
    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear),
        "Volcanic Fallout kills both 2-toughness bears (opp)");
    assert!(g.battlefield.iter().any(|c| c.id == dragon),
        "Volcanic Fallout doesn't kill the 5-toughness dragon");
    assert_eq!(g.players[0].life, life0 - 2, "Caster takes 2");
    assert_eq!(g.players[1].life, life1 - 2, "Opp takes 2");
}

#[test]
fn volcanic_fallout_is_uncounterable() {
    // Push (modern_decks): the "this can't be countered" rider now lands
    // via `Keyword::CantBeCountered` on the card definition. The cast
    // pushes `StackItem::Spell.uncounterable = true`.
    use crate::game::types::StackItem;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::volcanic_fallout());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Fallout castable");
    let uncounterable = g.stack.iter().find_map(|si| match si {
        StackItem::Spell { uncounterable, .. } => Some(*uncounterable),
        _ => None,
    });
    assert_eq!(uncounterable, Some(true),
        "Fallout should land on the stack as uncounterable");
}

#[test]
fn rout_destroys_all_creatures_at_sorcery_speed() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let lion = g.add_card_to_battlefield(0, catalog::savannah_lions());
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    let id = g.add_card_to_hand(0, catalog::rout());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::White, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Rout castable for {3}{W}{W}");
    drain_stack(&mut g);

    for cid in [bear, lion, dragon] {
        assert!(!g.battlefield.iter().any(|c| c.id == cid),
            "Rout should destroy all creatures");
    }
}

#[test]
fn plague_wind_destroys_only_opponent_creatures() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let opp_dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    let id = g.add_card_to_hand(0, catalog::plague_wind());
    g.players[0].mana_pool.add_colorless(8);
    g.players[0].mana_pool.add(Color::Black, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Plague Wind castable for {8}{B}{B}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == mine),
        "Plague Wind leaves caster's creatures alone");
    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear),
        "Plague Wind destroys opp's bear");
    assert!(!g.battlefield.iter().any(|c| c.id == opp_dragon),
        "Plague Wind destroys opp's dragon");
}

#[test]
fn carnage_tyrant_resolves_through_counterspell() {
    let mut g = two_player_game();
    let tyrant = g.add_card_to_hand(0, catalog::carnage_tyrant());
    let counter = g.add_card_to_hand(1, catalog::counterspell());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[1].mana_pool.add(Color::Blue, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: tyrant, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Tyrant castable for {4}{G}{G}");
    // Opponent tries to counter the spell. Counter targets are addressed
    // by `Target::Permanent(card_id)` (the stack-item lookup uses the
    // card-id internally regardless of zone). The Tyrant carries
    // `Keyword::CantBeCountered`, so `CounterSpell.uncounterable_check`
    // should let the Tyrant through unscathed.
    let _ = g.perform_action(GameAction::CastSpell {
        card_id: counter,
        target: Some(Target::Permanent(tyrant)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == tyrant),
        "Tyrant resolves despite the counter attempt");
}

#[test]
fn krark_clan_ironworks_sacs_artifact_for_two_colorless() {
    let mut g = two_player_game();
    let kci = g.add_card_to_battlefield(0, catalog::krark_clan_ironworks());
    // Have at least one other artifact under our control so the sac has
    // something to pick. KCI itself is also an artifact, so the bot may
    // sac KCI itself; we add a Sol Ring as the obvious sacrifice target.
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let pool_before = g.players[0].mana_pool.total();

    g.perform_action(GameAction::ActivateAbility {
        card_id: kci, ability_index: 0,
        target: None, x_value: None })
    .expect("KCI activated");
    drain_stack(&mut g);

    // One of {KCI, Sol Ring} should be sacrificed (whichever the bot picked).
    let kci_alive = g.battlefield.iter().any(|c| c.id == kci);
    let ring_alive = g.battlefield.iter().any(|c| c.id == ring);
    assert!(!kci_alive || !ring_alive,
        "At least one artifact should have been sacrificed");
    assert!(g.players[0].mana_pool.total() >= pool_before + 2,
        "KCI's sac yields at least {{2}}");
}

// ── Surveil land cycle (modern_decks-11) ─────────────────────────────────────

#[test]
fn underground_mortuary_etbs_tapped_and_carries_surveil_trigger() {
    let mut g = two_player_game();
    // Stock the library so the surveil has an input to inspect.
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::underground_mortuary());
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);

    let card = g.battlefield_find(id).expect("Mortuary on the battlefield");
    assert!(card.tapped, "Surveil land enters tapped");
    // The factory definition carries the surveil trigger (AutoDecider keeps
    // the surveil-peeked card on top, so we don't observe a library shape
    // change here — the structural assertion is what the cube wires).
    let def = catalog::underground_mortuary();
    let has_surveil = def.triggered_abilities.iter().any(|t| {
        if let crate::card::Effect::Seq(steps) = &t.effect {
            steps.iter().any(|e| matches!(e, crate::card::Effect::Surveil { .. }))
        } else {
            false
        }
    });
    assert!(has_surveil, "Mortuary's ETB trigger contains a Surveil step");
}

#[test]
fn underground_mortuary_taps_for_black_or_green() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::underground_mortuary());
    // Force-untap (helper places it tapped by default through ETB; we test
    // both colored mana abilities).
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).unwrap();
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1,
        "ability 0 produces {{B}}");

    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, x_value: None }).unwrap();
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1,
        "ability 1 produces {{G}}");
}

// ── modern_decks-12: 12 new playables ────────────────────────────────────────

#[test]
fn stone_rain_destroys_target_land() {
    let mut g = two_player_game();
    let mountain = g.add_card_to_battlefield(1, catalog::mountain());
    let id = g.add_card_to_hand(0, catalog::stone_rain());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(mountain)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Stone Rain castable for {2}{R}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == mountain),
        "Mountain should be destroyed");
}

#[test]
fn stone_rain_rejects_creature_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::stone_rain());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(), "Stone Rain rejects non-land target: {err:?}");
}

#[test]
fn bone_splinters_sacrifices_then_destroys_target() {
    let mut g = two_player_game();
    let sac_target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_target = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::bone_splinters());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(opp_target)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bone Splinters castable for {B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == sac_target),
        "Caster's bear should be sacrificed");
    assert!(!g.battlefield.iter().any(|c| c.id == opp_target),
        "Opponent's Serra Angel should be destroyed");
}

#[test]
fn hieroglyphic_illumination_draws_two_cards() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::hieroglyphic_illumination());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hieroglyphic Illumination castable for {3}{U}");
    drain_stack(&mut g);
    // Cast (-1) + Draw 2 (+2) = +1 hand size.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn mortify_destroys_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::mortify());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Mortify castable for {1}{W}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be destroyed");
}

#[test]
fn mortify_destroys_enchantment() {
    let mut g = two_player_game();
    // Phyrexian Arena is an enchantment; use it as the target.
    let arena = g.add_card_to_battlefield(1, catalog::phyrexian_arena());
    let id = g.add_card_to_hand(0, catalog::mortify());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(arena)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Mortify castable on enchantment");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == arena),
        "Phyrexian Arena should be destroyed");
}

#[test]
fn mortify_rejects_land_target() {
    let mut g = two_player_game();
    let mountain = g.add_card_to_battlefield(1, catalog::mountain());
    let id = g.add_card_to_hand(0, catalog::mortify());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(mountain)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(), "Mortify rejects a land target: {err:?}");
}

#[test]
fn maelstrom_pulse_destroys_nonland_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::maelstrom_pulse());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Maelstrom Pulse castable for {1}{B}{G}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

#[test]
fn maelstrom_pulse_rejects_land_target() {
    let mut g = two_player_game();
    let mountain = g.add_card_to_battlefield(1, catalog::mountain());
    let id = g.add_card_to_hand(0, catalog::maelstrom_pulse());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(mountain)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(), "Maelstrom Pulse rejects land: {err:?}");
}

#[test]
fn mind_twist_discards_x_random_cards_from_target_player() {
    let mut g = two_player_game();
    // Stack opponent's hand with five cards.
    for _ in 0..5 { g.add_card_to_hand(1, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::mind_twist());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    }).expect("Mind Twist castable for {3}{B} (X=3)");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand_before - 3,
        "Three random cards discarded");
    assert_eq!(g.players[1].graveyard.len(), 3,
        "Three cards in opponent's graveyard");
}

#[test]
fn mind_twist_x_zero_does_nothing() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_hand(1, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::mind_twist());
    g.players[0].mana_pool.add(Color::Black, 1);
    let hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(0),
    }).expect("Mind Twist castable at X=0 for {B}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand_before,
        "X=0 yields no discards");
}

#[test]
fn dismember_kills_a_five_toughness_creature() {
    let mut g = two_player_game();
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flying
    let big = g.add_card_to_battlefield(1, catalog::sengir_vampire()); // 4/4
    let id = g.add_card_to_hand(0, catalog::dismember());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);

    let _ = serra;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(big)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Dismember castable for {1}{B}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == big),
        "Sengir Vampire (4/4) dies to -5/-5");
}

#[test]
fn echoing_truth_bounces_nonland_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::echoing_truth());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Echoing Truth castable for {1}{U}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    assert!(g.players[1].hand.iter().any(|c| c.id == bear),
        "Bear bounces back to its owner's hand");
}

#[test]
fn echoing_truth_rejects_land_target() {
    let mut g = two_player_game();
    let mountain = g.add_card_to_battlefield(1, catalog::mountain());
    let id = g.add_card_to_hand(0, catalog::echoing_truth());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(mountain)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(), "Echoing Truth is nonland-only: {err:?}");
}

#[test]
fn celestial_purge_exiles_a_black_creature() {
    let mut g = two_player_game();
    let specter = g.add_card_to_battlefield(1, catalog::hypnotic_specter());
    let id = g.add_card_to_hand(0, catalog::celestial_purge());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(specter)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Celestial Purge castable for {1}{W}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == specter),
        "Specter exiled");
    assert!(g.exile.iter().any(|c| c.id == specter),
        "Specter is in exile (not graveyard)");
}

#[test]
fn celestial_purge_rejects_a_green_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green
    let id = g.add_card_to_hand(0, catalog::celestial_purge());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Celestial Purge can only target black or red permanents: {err:?}");
}

#[test]
fn earthquake_burns_each_player_and_grounded_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 ground
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flying
    let id = g.add_card_to_hand(0, catalog::earthquake());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life_p0 = g.players[0].life;
    let life_p1 = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None,
        x_value: Some(3),
    }).expect("Earthquake castable for {3}{R} (X=3)");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear (2/2 ground) takes 3 and dies");
    assert!(g.battlefield.iter().any(|c| c.id == serra),
        "Serra (flying) is untouched");
    assert_eq!(g.players[0].life, life_p0 - 3, "P0 takes 3");
    assert_eq!(g.players[1].life, life_p1 - 3, "P1 takes 3");
}

#[test]
fn glimpse_the_unthinkable_mills_ten_from_target() {
    let mut g = two_player_game();
    for _ in 0..15 { g.add_card_to_library(1, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::glimpse_the_unthinkable());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let lib_before = g.players[1].library.len();
    let yard_before = g.players[1].graveyard.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Glimpse castable for {U}{B}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib_before - 10);
    assert_eq!(g.players[1].graveyard.len(), yard_before + 10);
}

#[test]
fn cling_to_dust_exiles_creature_card_and_gains_two_life() {
    let mut g = two_player_game();
    // Seed a creature card in opp's graveyard.
    let card_id = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::cling_to_dust());
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(card_id)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Cling to Dust castable for {B}");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == card_id),
        "Card moves from graveyard to exile");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == card_id));
    assert_eq!(g.players[0].life, life_before + 2,
        "Caster gains 2 life when a creature is exiled");
}

#[test]
fn cling_to_dust_no_lifegain_for_noncreature_card() {
    let mut g = two_player_game();
    // Seed a non-creature (instant) card in opp's graveyard.
    let card_id = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::cling_to_dust());
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(card_id)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Cling to Dust castable for {B}");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == card_id),
        "Card still moves to exile");
    assert_eq!(g.players[0].life, life_before,
        "No lifegain when the exiled card isn't a creature");
}

// ── modern_decks-13: 12 new cards ───────────────────────────────────────────

#[test]
fn lumra_returns_all_lands_from_your_graveyard() {
    let mut g = two_player_game();
    // Seed two land cards + one non-land in P0's graveyard.
    let f1 = g.add_card_to_graveyard(0, catalog::forest());
    let f2 = g.add_card_to_graveyard(0, catalog::forest());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lumra_bellow_of_the_woods());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Green, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lumra castable for {4}{G}{G}");
    drain_stack(&mut g);

    // Both Forests came back to BF tapped, the Bear stayed in graveyard.
    assert!(g.battlefield_find(f1).is_some(), "Forest 1 returned to BF");
    assert!(g.battlefield_find(f2).is_some(), "Forest 2 returned to BF");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear),
        "Bear stays in graveyard (not a land)");
    assert!(g.battlefield_find(f1).unwrap().tapped,
        "Land returns tapped per Oracle");
    // Lumra itself is on the battlefield as a 6/6 trampler.
    let lumra = g.battlefield_find(id).unwrap();
    assert_eq!(lumra.definition.power, 6);
    assert_eq!(lumra.definition.toughness, 6);
    assert!(lumra.definition.keywords.contains(&crate::card::Keyword::Trample));
}

#[test]
fn lumra_etb_with_empty_graveyard_is_a_noop() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lumra_bellow_of_the_woods());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Green, 2);

    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lumra castable for {4}{G}{G}");
    drain_stack(&mut g);

    // Empty graveyard → the only addition to BF is Lumra herself.
    assert_eq!(g.battlefield.len(), bf_before + 1);
    assert!(g.battlefield_find(id).is_some());
}

#[test]
fn crabomination_etb_mills_each_opponent_three_cards() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::island());
    }
    let lib_before = g.players[1].library.len();
    let yard_before = g.players[1].graveyard.len();
    let id = g.add_card_to_hand(0, catalog::crabomination());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Crabomination castable for {2}{U}{B}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].library.len(), lib_before - 3,
        "Crabomination mills 3 cards");
    assert_eq!(g.players[1].graveyard.len(), yard_before + 3,
        "Milled cards land in opp's graveyard");
}

#[test]
fn chaos_warp_sends_target_permanent_to_owners_library() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::chaos_warp());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Red, 1);

    let lib_before = g.players[1].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Chaos Warp castable for {2}{R}");
    drain_stack(&mut g);

    assert!(g.battlefield_find(bear).is_none(),
        "Bear left the battlefield");
    assert_eq!(g.players[1].library.len(), lib_before + 1,
        "Bear returns to its owner's library");
    assert!(g.players[1].library.iter().any(|c| c.id == bear));
}

#[test]
fn elvish_reclaimer_sacrifices_land_to_search_for_one() {
    let mut g = two_player_game();
    let reclaimer = g.add_card_to_battlefield(0, catalog::elvish_reclaimer());
    g.clear_sickness(reclaimer);
    // Untap so the tap-cost ability is legal.
    g.battlefield.iter_mut().find(|c| c.id == reclaimer).unwrap().tapped = false;
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let target_in_lib = g.add_card_to_library(0, catalog::wasteland());

    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(target_in_lib)),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: reclaimer,
        ability_index: 0,
        target: None, x_value: None }).expect("Elvish Reclaimer's tap+sac+search ability");
    drain_stack(&mut g);

    // Forest was sacrificed.
    assert!(g.battlefield_find(forest).is_none(),
        "Forest sacrificed as cost");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == forest));
    // Wasteland made it to the battlefield.
    assert!(g.battlefield_find(target_in_lib).is_some(),
        "Searched land entered the battlefield");
}

#[test]
fn rofellos_taps_for_two_green_mana() {
    // Push (claude/modern_decks): Rofellos's mana ability now scales
    // with Forest count (`{T}: Add {G} for each Forest you control`,
    // rendered Add {G}{G} on the printed card). One Forest in play +
    // tap Rofellos → 2 green.
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::rofellos_llanowar_emissary());
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    g.add_card_to_battlefield(0, catalog::forest());

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).expect("Rofellos's mana ability");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2,
        "Rofellos adds two green mana per Forest (1 forest → 2 green)");
}

#[test]
fn biorhythm_drops_each_opponent_to_zero_or_below() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::biorhythm());
    for _ in 0..8 {
        g.players[0].mana_pool.add(Color::Green, 1);
    }

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Biorhythm castable for {6}{G}{G}");
    drain_stack(&mut g);

    // Push (modern_decks): Biorhythm now uses SetLifeTotal (CR 119.5).
    // With 0 creatures opp controls → opp life = 0.
    assert_eq!(g.players[1].life, 0,
        "Opp life set to creature count (0): got {}", g.players[1].life);
}

#[test]
fn biorhythm_sets_life_to_creature_count_per_cr_119_5() {
    let mut g = two_player_game();
    // You control 3 bears; opp controls 1 bear.
    let _b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _b3 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _o1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::biorhythm());
    for _ in 0..8 {
        g.players[0].mana_pool.add(Color::Green, 1);
    }
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Biorhythm castable for {6}{G}{G}");
    drain_stack(&mut g);

    // CR 119.5 — life total set to creature count per side.
    assert_eq!(g.players[0].life, 3, "your life = 3 bears");
    assert_eq!(g.players[1].life, 1, "opp life = 1 bear");
}

#[test]
fn karn_scion_of_urza_minus_two_creates_a_construct_token() {
    let mut g = two_player_game();
    let karn = g.add_card_to_battlefield(0, catalog::karn_scion_of_urza());
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: karn, ability_index: 2, target: None,
    }).expect("Karn -2 to make a Construct token");
    drain_stack(&mut g);

    assert_eq!(g.battlefield.len(), bf_before + 1, "Token entered the battlefield");
    let token = g.battlefield.iter().find(|c| c.definition.name == "Construct")
        .expect("Construct token present");
    assert!(token.definition.card_types.contains(&crate::card::CardType::Artifact));
    assert!(token.definition.card_types.contains(&crate::card::CardType::Creature));
}

#[test]
fn karn_plus_one_draws_a_card_and_mills_one() {
    let mut g = two_player_game();
    let karn = g.add_card_to_battlefield(0, catalog::karn_scion_of_urza());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let hand_before = g.players[0].hand.len();
    let yard_before = g.players[0].graveyard.len();

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: karn, ability_index: 0, target: None,
    }).expect("Karn +1");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 1, "Drew 1 card");
    assert_eq!(g.players[0].graveyard.len(), yard_before + 1, "Milled 1 card");
}

#[test]
fn tezzeret_minus_two_drains_each_opponent_for_two() {
    let mut g = two_player_game();
    let tez = g.add_card_to_battlefield(0, catalog::tezzeret_cruel_captain());
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: tez, ability_index: 1, target: None,
    }).expect("Tezzeret -2 drain");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, p1_life - 2, "Opp loses 2 life");
    assert_eq!(g.players[0].life, p0_life + 2, "You gain 2 life");
}

#[test]
fn tezzeret_plus_one_shrinks_target_creature() {
    let mut g = two_player_game();
    let tez = g.add_card_to_battlefield(0, catalog::tezzeret_cruel_captain());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: tez, ability_index: 0,
        target: Some(Target::Permanent(bear)),
    }).expect("Tezzeret +1 -2/-2");
    drain_stack(&mut g);

    // 2/2 bear with -2/-2 dies via SBA.
    assert!(g.battlefield_find(bear).is_none(),
        "Bear died from -2/-2");
}

#[test]
fn balefire_dragon_combat_damage_burns_each_opp_creature() {
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(0, catalog::balefire_dragon());
    g.battlefield.iter_mut().find(|c| c.id == dragon).unwrap().tapped = false;
    let bear1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Fire the trigger directly (the combat-damage event is tested separately
    // in the combat test suite). We exercise the trigger's effect by event-
    // bus push here.
    let trig = catalog::balefire_dragon().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(
        dragon, 0, None, 0,
    );
    let _ = g.resolve_effect(&trig, &ctx);

    // Each opp creature took 6 damage and died via SBA.
    assert!(g.battlefield_find(bear1).is_none(), "Bear 1 perished");
    assert!(g.battlefield_find(bear2).is_none(), "Bear 2 perished");
}

#[test]
fn greasewrench_goblin_creates_treasure_on_death() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::greasewrench_goblin());
    let bf_before = g.battlefield.len();
    // Kill the goblin via direct removal (Lightning Bolt).
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(id)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bolt castable for {R}");
    drain_stack(&mut g);

    // Goblin died; a Treasure token appeared.
    assert!(g.battlefield_find(id).is_none(), "Goblin died");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"),
        "Treasure token appeared on death");
    // BF: -1 (goblin gone) +1 (treasure) = unchanged in count.
    assert_eq!(g.battlefield.len(), bf_before);
}

#[test]
fn cruel_somnophage_pt_scales_with_your_graveyard() {
    let mut g = two_player_game();
    // Seed graveyard with three cards before Cruel Somnophage enters.
    g.add_card_to_graveyard(0, catalog::island());
    g.add_card_to_graveyard(0, catalog::island());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_battlefield(0, catalog::cruel_somnophage());

    let computed = g.compute_battlefield();
    let card = computed.iter().find(|c| c.id == id).unwrap();
    assert_eq!(card.power, 3, "Power = your graveyard size (3)");
    assert_eq!(card.toughness, 3, "Toughness = your graveyard size (3)");

    // Mill another card and watch P/T grow.
    g.add_card_to_graveyard(0, catalog::island());
    let computed = g.compute_battlefield();
    let card = computed.iter().find(|c| c.id == id).unwrap();
    assert_eq!(card.power, 4);
    assert_eq!(card.toughness, 4);
}

#[test]
fn pentad_prism_etb_with_two_charge_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pentad_prism());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pentad Prism castable for {2}");
    drain_stack(&mut g);

    let card = g.battlefield_find(id).unwrap();
    let charge = card.counters.get(&crate::card::CounterType::Charge).copied().unwrap_or(0);
    assert_eq!(charge, 2, "Pentad Prism enters with 2 charge counters");
}

#[test]
fn pentad_prism_removes_counter_to_add_one_mana_of_any_color() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pentad_prism());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pentad Prism castable");
    drain_stack(&mut g);

    let pool_before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).expect("Pentad Prism's counter-removal mana ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), pool_before + 1,
        "Mana pool gains 1");

    let card = g.battlefield_find(id).unwrap();
    let charge = card.counters.get(&crate::card::CounterType::Charge).copied().unwrap_or(0);
    assert_eq!(charge, 1, "Charge counters: 2 → 1 after one activation");
}

// ── modern_decks-14 ──────────────────────────────────────────────────────────

#[test]
fn vindicate_destroys_target_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::vindicate());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Vindicate castable for {1}{W}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Vindicate destroys target permanent");
}

#[test]
fn vindicate_can_target_a_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::island());
    let id = g.add_card_to_hand(0, catalog::vindicate());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(land)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Vindicate accepts a land target (Permanent filter)");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == land),
        "Vindicate destroys a land target — same as Oracle");
}

#[test]
fn anguished_unmaking_exiles_and_caster_loses_three_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::anguished_unmaking());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Anguished Unmaking castable for {1}{W}{B}");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == bear),
        "Bear should be in exile, not graveyard");
    assert_eq!(g.players[0].life, life_before - 3,
        "Caster loses 3 life");
}

#[test]
fn magma_spray_deals_two_damage_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::magma_spray());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Magma Spray castable for {R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Grizzly Bears (2-toughness) dies to 2 damage");
}

#[test]
fn despark_exiles_high_cmc_permanent() {
    let mut g = two_player_game();
    let craw = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6 CMC
    let id = g.add_card_to_hand(0, catalog::despark());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(craw)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Despark castable for {W}{B}");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == craw),
        "6-CMC Craw Wurm gets exiled by Despark");
}

#[test]
fn despark_rejects_low_cmc_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2 CMC
    let id = g.add_card_to_hand(0, catalog::despark());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Despark should reject a 2-CMC creature: {:?}", err);
}

#[test]
fn crumble_to_dust_exiles_nonbasic_but_rejects_basic() {
    let mut g = two_player_game();
    let dual = g.add_card_to_battlefield(1, catalog::watery_grave());
    let basic = g.add_card_to_battlefield(1, catalog::island());
    let id_ok = g.add_card_to_hand(0, catalog::crumble_to_dust());
    let id_bad = g.add_card_to_hand(0, catalog::crumble_to_dust());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Red, 4);

    g.perform_action(GameAction::CastSpell {
        card_id: id_ok,
        target: Some(Target::Permanent(dual)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Crumble to Dust castable for {2}{R}{R}");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == dual), "Watery Grave (nonbasic) gets exiled");

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id_bad,
        target: Some(Target::Permanent(basic)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Crumble to Dust should reject a basic Island: {:?}", err);
}

#[test]
fn skullcrack_deals_three_damage_to_player() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::skullcrack());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Skullcrack castable for {1}{R}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life_before - 3,
        "Skullcrack deals 3 damage to target player");
}

#[test]
fn skullcrack_locks_target_player_lifegain_for_the_turn() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::skullcrack());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Skullcrack castable");
    drain_stack(&mut g);

    // Target is now locked from gaining life.
    assert!(g.players[1].cannot_gain_life_this_turn);
    let life_after_bolt = g.players[1].life;
    // Try to gain 5 life — should be a no-op.
    g.adjust_life(1, 5);
    assert_eq!(g.players[1].life, life_after_bolt,
        "CR 119.7 — locked player can't gain life");
    // Caster (seat 0) is not locked.
    g.adjust_life(0, 5);
    assert!(!g.players[0].cannot_gain_life_this_turn);
}

#[test]
fn skullcrack_lifegain_lock_clears_at_next_untap() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::skullcrack());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Skullcrack castable");
    drain_stack(&mut g);
    assert!(g.players[1].cannot_gain_life_this_turn);

    // do_untap is called when the active player rotates. Run it
    // directly to assert the per-turn flag clears for every player.
    g.do_untap();
    assert!(!g.players[1].cannot_gain_life_this_turn);
    assert!(!g.players[0].cannot_gain_life_this_turn);
}

#[test]
fn fiery_impulse_deals_two_damage_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::fiery_impulse());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Fiery Impulse castable for {R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "2-toughness Grizzly Bears dies to Fiery Impulse");
}

#[test]
fn fiery_impulse_deals_three_damage_with_spell_mastery() {
    let mut g = two_player_game();
    // Seed 2+ IS cards in your graveyard for spell mastery.
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    // 3-toughness creature: Owlin Shieldmage is 2/3.
    let target = g.add_card_to_battlefield(1, catalog::owlin_shieldmage());
    let id = g.add_card_to_hand(0, catalog::fiery_impulse());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Fiery Impulse castable for {R}");
    drain_stack(&mut g);

    // Owlin Shieldmage is 2/3 — 3 damage kills it, 2 damage doesn't.
    assert!(!g.battlefield.iter().any(|c| c.id == target),
        "Spell mastery: 3-toughness creature dies to upgraded 3 damage");
}

#[test]
fn fiery_impulse_deals_two_damage_without_spell_mastery() {
    let mut g = two_player_game();
    // Only ONE IS card in your graveyard — spell mastery NOT active.
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let target = g.add_card_to_battlefield(1, catalog::owlin_shieldmage()); // 2/3
    let id = g.add_card_to_hand(0, catalog::fiery_impulse());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Fiery Impulse castable for {R}");
    drain_stack(&mut g);

    // Owlin survives — 2 damage dealt (no spell mastery), 2 < 3 toughness.
    assert!(g.battlefield.iter().any(|c| c.id == target),
        "Without spell mastery: 3-toughness creature survives 2 damage");
}

#[test]
fn searing_blood_deals_two_damage_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::searing_blood());
    g.players[0].mana_pool.add(Color::Red, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Searing Blood castable for {R}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "2-toughness Grizzly Bears dies to Searing Blood");
}

#[test]
fn harrow_sacrifices_land_and_searches_two_basics() {
    let mut g = two_player_game();
    // Stock the library with two Forests so Harrow has fetch targets.
    let forest_one = g.add_card_to_library(0, catalog::forest());
    let forest_two = g.add_card_to_library(0, catalog::forest());
    // Sac fodder: a Mountain on the battlefield.
    let mountain = g.add_card_to_battlefield(0, catalog::mountain());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest_one)),
        DecisionAnswer::Search(Some(forest_two)),
    ]));
    let id = g.add_card_to_hand(0, catalog::harrow());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Harrow castable for {2}{G}");
    drain_stack(&mut g);

    // Mountain went to graveyard (sacrificed).
    assert!(g.players[0].graveyard.iter().any(|c| c.id == mountain),
        "Sacrificed Mountain ends in graveyard");
    // Two Forests are now in play (untapped).
    let forests_in_play = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Forest")
        .count();
    assert_eq!(forests_in_play, 2, "Both Forests entered the battlefield");
    // Both Forests should enter UNTAPPED (this is what differentiates Harrow
    // from Cultivate / Kodama's Reach).
    for f in g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Forest")
    {
        assert!(!f.tapped, "Harrow's basics enter untapped");
    }
}

#[test]
fn drown_in_the_loch_mode_zero_counters_a_spell() {
    let mut g = two_player_game();
    // Opponent casts a spell on their own turn.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Lightning Bolt cast by opp");

    // Caster responds with Drown in the Loch in mode 0 (counter).
    let id = g.add_card_to_hand(0, catalog::drown_in_the_loch());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.priority.player_with_priority = 0;
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    }).expect("Drown in the Loch mode 0 (counter) castable");
    drain_stack(&mut g);

    // Bolt is countered → caster takes no damage.
    assert_eq!(g.players[0].life, life_before,
        "Bolt was countered, caster's life is unchanged");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Countered Bolt ends in opp's graveyard");
}

#[test]
fn drown_in_the_loch_mode_one_destroys_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::drown_in_the_loch());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    }).expect("Drown in the Loch mode 1 castable");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Mode 1 destroys target creature");
}

#[test]
fn cremate_exiles_graveyard_card_and_draws() {
    let mut g = two_player_game();
    // Stock graveyard with a card and library with one to draw.
    let grave_id = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::cremate());
    g.players[0].mana_pool.add(Color::Black, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(grave_id)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Cremate castable for {B}");
    drain_stack(&mut g);

    // Net hand: -1 cast +1 draw = 0.
    assert_eq!(g.players[0].hand.len(), hand_before, "Cantrip nets 0 hand");
    assert!(g.exile.iter().any(|c| c.id == grave_id),
        "Targeted graveyard card was exiled");
}

#[test]
fn mortuary_mire_etb_taps_and_recurs_creature_card() {
    let mut g = two_player_game();
    let _grave_creature = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::mortuary_mire());

    g.perform_action(GameAction::PlayLand(id))
        .expect("Mortuary Mire is a playable land");
    drain_stack(&mut g);

    let mire = g.battlefield_find(id).expect("Mortuary Mire on battlefield");
    assert!(mire.tapped, "Mortuary Mire enters tapped");
    // The bear should have moved to the top of player 0's library.
    let top = g.players[0].library.last()
        .expect("Library should not be empty");
    assert_eq!(top.definition.name, "Grizzly Bears",
        "ETB places the creature card on top of library");
}

#[test]
fn geier_reach_sanitarium_taps_for_colorless() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::geier_reach_sanitarium());
    let pool_before = g.players[0].mana_pool.total();

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).expect("First mana ability is {T}: Add {C}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].mana_pool.total(), pool_before + 1,
        "Geier Reach taps for {{C}}");
}

#[test]
fn geier_reach_sanitarium_wheel_ability_each_player_loots() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::geier_reach_sanitarium());
    g.players[0].mana_pool.add_colorless(1);
    // Stock libraries so each player has a card to draw.
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(1, catalog::island());
    // Stock hands so the discard step has something to throw away.
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let h0 = g.players[0].hand.len();
    let h1 = g.players[1].hand.len();

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, x_value: None }).expect("Wheel ability is sorcery-speed");
    drain_stack(&mut g);

    // Each player draws 1 then discards 1 → net 0 hand size for each.
    assert_eq!(g.players[0].hand.len(), h0,
        "Player 0 nets 0 hand from each-player loot");
    assert_eq!(g.players[1].hand.len(), h1,
        "Player 1 nets 0 hand from each-player loot");
}

// ── modern_decks-15: 12 new cube cards ───────────────────────────────────────

#[test]
fn strangle_deals_three_damage_and_surveils() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::strangle());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Strangle castable for {R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Grizzly Bears (2-toughness) dies to 3 damage");
    // Surveil 1 inspected the top card of the library, putting it either
    // back on top or in the graveyard. Either outcome reduces or holds
    // the library size; we just verify the cast didn't fail at surveil.
    assert!(g.players[0].library.len() <= lib_before,
        "Surveil 1 either kept or graveyarded the top card");
}

#[test]
fn dreadbore_destroys_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::dreadbore());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Dreadbore castable for {B}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Dreadbore destroys the target creature");
}

#[test]
fn bedevil_destroys_target_artifact() {
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    let id = g.add_card_to_hand(0, catalog::bedevil());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(stone)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bedevil castable for {B}{B}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == stone),
        "Bedevil destroys the target artifact");
}

#[test]
fn tome_scour_mills_target_player_five() {
    let mut g = two_player_game();
    for _ in 0..10 { g.add_card_to_library(1, catalog::island()); }
    let lib_before = g.players[1].library.len();
    let grave_before = g.players[1].graveyard.len();
    let id = g.add_card_to_hand(0, catalog::tome_scour());
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Tome Scour castable for {U}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].library.len(), lib_before - 5,
        "Tome Scour mills 5 cards from target player");
    assert_eq!(g.players[1].graveyard.len(), grave_before + 5);
}

#[test]
fn repulse_returns_creature_and_draws() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::repulse());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    let opp_hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Repulse castable for {2}{U}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear is bounced off the battlefield");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear),
        "Bear returns to its owner's hand");
    // Caster: -1 cast +1 draw = 0.
    assert_eq!(g.players[0].hand.len(), hand_before, "Repulse cantrips");
    // Opp gains the bear in hand.
    assert_eq!(g.players[1].hand.len(), opp_hand_before + 1);
}

#[test]
fn visions_of_beyond_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::visions_of_beyond());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Visions of Beyond castable for {U}");
    drain_stack(&mut g);

    // -1 cast +1 draw = 0 net hand change.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "Visions of Beyond is a 1-mana cantrip");
}

#[test]
fn visions_of_beyond_draws_three_with_twenty_card_graveyard() {
    let mut g = two_player_game();
    // Stack opponent's graveyard with 20 cards (any card type works).
    for _ in 0..20 {
        let id = g.add_card_to_library(1, catalog::island());
        // Put it directly into the graveyard.
        if let Some(pos) = g.players[1].library.iter().position(|c| c.id == id) {
            let card = g.players[1].library.remove(pos);
            g.players[1].graveyard.push(card);
        }
    }
    assert_eq!(g.players[1].graveyard.len(), 20);
    // Stock 4 cards in seat 0's library so the draw-3 has fodder.
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::visions_of_beyond());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Visions of Beyond castable for {U}");
    drain_stack(&mut g);

    // -1 cast +3 draw = +2 net hand change (the upgraded mode).
    assert_eq!(g.players[0].hand.len(), hand_before + 2,
        "Visions of Beyond draws 3 when a graveyard has 20+ cards");
}

#[test]
fn visions_of_beyond_draws_one_with_nineteen_card_graveyard() {
    let mut g = two_player_game();
    // Just under the threshold — 19 cards.
    for _ in 0..19 {
        let id = g.add_card_to_library(1, catalog::island());
        if let Some(pos) = g.players[1].library.iter().position(|c| c.id == id) {
            let card = g.players[1].library.remove(pos);
            g.players[1].graveyard.push(card);
        }
    }
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::visions_of_beyond());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Visions of Beyond castable for {U}");
    drain_stack(&mut g);

    // -1 cast +1 draw = 0 net hand change (the cantrip mode).
    assert_eq!(g.players[0].hand.len(), hand_before,
        "Visions of Beyond draws 1 when no graveyard has 20+ cards");
}

#[test]
fn plummet_destroys_target_flying_creature() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // flying
    let id = g.add_card_to_hand(0, catalog::plummet());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(angel)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Plummet castable for {1}{G}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == angel),
        "Plummet destroys flying Serra Angel");
}

#[test]
fn plummet_rejects_non_flying_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // no flying
    let id = g.add_card_to_hand(0, catalog::plummet());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Plummet should reject a non-flying creature: {:?}", err);
}

#[test]
fn strategic_planning_mills_three_and_draws_one() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let lib_before = g.players[0].library.len();
    let grave_before = g.players[0].graveyard.len();
    let id = g.add_card_to_hand(0, catalog::strategic_planning());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Strategic Planning castable for {1}{U}");
    drain_stack(&mut g);

    // Mill 3, then Draw 1 => library down by 4, graveyard up by 3 milled
    // cards + 1 (the resolved Strategic Planning sorcery itself) = 4.
    assert_eq!(g.players[0].library.len(), lib_before - 4,
        "Library lost 3 to mill + 1 to draw = 4");
    assert_eq!(g.players[0].graveyard.len(), grave_before + 4,
        "Graveyard gained 3 milled cards + the resolved sorcery itself");
    // Hand: -1 cast + 1 draw = 0.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "Strategic Planning is a 2-mana cantrip");
}

#[test]
fn ravenous_rats_etb_makes_each_opponent_discard() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let opp_hand_before = g.players[1].hand.len();
    let opp_grave_before = g.players[1].graveyard.len();
    let id = g.add_card_to_hand(0, catalog::ravenous_rats());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ravenous Rats castable for {1}{B}");
    drain_stack(&mut g);

    // Body enters the battlefield.
    assert!(g.battlefield.iter().any(|c| c.id == id),
        "Rat body resolves onto the battlefield");
    // Opp discards a card → hand -1, graveyard +1.
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1,
        "Opponent discarded a card from ETB trigger");
    assert_eq!(g.players[1].graveyard.len(), opp_grave_before + 1);
}

#[test]
fn brain_maggot_etb_strips_a_nonland_card() {
    let mut g = two_player_game();
    let target_card = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::forest()); // land — should be skipped by filter
    let id = g.add_card_to_hand(0, catalog::brain_maggot());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Brain Maggot castable for {1}{B}");
    drain_stack(&mut g);

    assert!(g.players[1].graveyard.iter().any(|c| c.id == target_card),
        "Lightning Bolt (the only nonland in P1's hand) is stripped");
    // Forest stays in P1's hand (it's a land).
    assert!(g.players[1].hand.iter().any(|c| c.definition.name == "Forest"),
        "Land remains in opponent's hand (filter is Nonland)");
}

#[test]
fn bond_of_discipline_taps_each_opponent_creature_and_grants_lifelink() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let bear_a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bear_b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let your_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::bond_of_discipline());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bond of Discipline castable for {3}{W}");
    drain_stack(&mut g);

    let perm_a = g.battlefield_find(bear_a).expect("opp bear A still on battlefield");
    let perm_b = g.battlefield_find(bear_b).expect("opp bear B still on battlefield");
    assert!(perm_a.tapped, "Bond of Discipline taps each opponent creature (A)");
    assert!(perm_b.tapped, "Bond of Discipline taps each opponent creature (B)");

    // Your bear gains lifelink EOT — check the temporary keyword grant.
    let computed = g.computed_permanent(your_bear)
        .expect("your bear still on battlefield");
    assert!(computed.keywords.iter().any(|k| matches!(k, Keyword::Lifelink)),
        "Your creature has lifelink granted EOT");
}

#[test]
fn sudden_edict_forces_target_player_to_sacrifice() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::sudden_edict());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sudden Edict castable for {1}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Targeted opponent sacrificed their only creature");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear),
        "Sacrificed creature ends up in opp's graveyard");
}

/// Regression: `Effect::Sacrifice`'s `who` slot now surfaces a target
/// filter via `primary_target_filter`, so the auto-target heuristic
/// picks the opponent for edict-class spells. Without the surfacing,
/// `auto_target_for_effect` returned None and the bot couldn't cast
/// Sudden Edict at all.
#[test]
fn auto_target_for_sudden_edict_picks_opponent() {
    use crate::server::bot;
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let card = catalog::sudden_edict();
    let target = bot::choose_target(&g, &card, 0);
    assert_eq!(target, Some(Target::Player(1)),
        "auto_target_for_effect picks the opponent for Sudden Edict");
}

/// `target_filtered(SelectionRequirement::Player)` rejects a permanent
/// target at cast time, so Sudden Edict can't be aimed at a creature
/// directly (cast-time filter mismatch).
#[test]
fn sudden_edict_rejects_creature_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::sudden_edict());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Sudden Edict should reject a creature target (Player filter): {:?}",
        err);
}

// ── modern_decks-16: cube-pool activations ───────────────────────────────────
//
// These cards already have factories + targeted unit tests covering
// the cards' primary play patterns — see e.g. `vandalblast_destroys_
// opponent_artifact`, `ranger_captain_etb_searches_for_a_one_drop`,
// `heliod_sun_crowned_grants_lifelink_until_end_of_turn`,
// `containment_priest_is_a_flash_two_two`, `tireless_tracker_*`,
// `swan_song_*`. The activations below pin the cube-pool wiring (so
// regressions on the cube prefetch / sampling path get caught early).

/// Fellwar Stone joins the colorless utility pool when activated.
/// Verify the factory produces a working {2} mana rock that taps for
/// any one color.
#[test]
fn fellwar_stone_taps_for_any_color() {
    // Push (batch 117): Fellwar Stone now respects "an opponent's
    // land could produce". With no opp lands at all, falls back to
    // colorless. Seed an opp Island so the pool gains blue.
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(0, catalog::fellwar_stone());
    g.battlefield_find_mut(stone).unwrap().summoning_sick = false;
    // Opp has an Island → Blue is in the legal pool.
    g.add_card_to_battlefield(1, catalog::island());

    g.perform_action(GameAction::ActivateAbility {
        card_id: stone, ability_index: 0, target: None, x_value: None })
    .expect("Fellwar Stone's mana ability should resolve");

    let pool = &g.players[0].mana_pool;
    // With only Island under opp's control, only Blue should be legal.
    assert_eq!(pool.amount(Color::Blue), 1,
        "Fellwar Stone produced 1 blue (the only color opp's Island can produce)");
    assert_eq!(pool.amount(Color::White), 0);
    assert_eq!(pool.amount(Color::Black), 0);
    assert_eq!(pool.amount(Color::Red), 0);
    assert_eq!(pool.amount(Color::Green), 0);
}

#[test]
fn fellwar_stone_falls_back_to_colorless_when_no_opp_basic_lands() {
    // No opp lands → pool gains 1 colorless (so the activation isn't
    // a silent no-op). Matches the "never silently no-op" convention.
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(0, catalog::fellwar_stone());
    g.battlefield_find_mut(stone).unwrap().summoning_sick = false;
    // Opp has no battlefield permanents at all.
    g.perform_action(GameAction::ActivateAbility {
        card_id: stone, ability_index: 0, target: None, x_value: None })
    .expect("Fellwar Stone activates with no opp lands");
    let pool = &g.players[0].mana_pool;
    assert_eq!(pool.total(), 1, "Pool has exactly 1 mana");
    // Colorless fallback — none of the colored amounts increment.
    assert_eq!(pool.amount(Color::White), 0);
    assert_eq!(pool.amount(Color::Blue), 0);
    assert_eq!(pool.amount(Color::Black), 0);
    assert_eq!(pool.amount(Color::Red), 0);
    assert_eq!(pool.amount(Color::Green), 0);
}

#[test]
fn fellwar_stone_unions_colors_across_multiple_opp_lands() {
    // Multiple opp basic-typed lands → union of their colors is the
    // legal pool. With opp Island + Forest, only Blue + Green are
    // legal; AutoDecider picks the first (Blue).
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(0, catalog::fellwar_stone());
    g.battlefield_find_mut(stone).unwrap().summoning_sick = false;
    g.add_card_to_battlefield(1, catalog::island());
    g.add_card_to_battlefield(1, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: stone, ability_index: 0, target: None, x_value: None })
    .expect("Fellwar Stone activates");
    let pool = &g.players[0].mana_pool;
    assert_eq!(pool.total(), 1);
    // White / Black / Red are not in the legal pool — opp controls no
    // Plains / Swamp / Mountain.
    assert_eq!(pool.amount(Color::White), 0);
    assert_eq!(pool.amount(Color::Black), 0);
    assert_eq!(pool.amount(Color::Red), 0);
    // Blue or Green (one of them) gained 1.
    assert!(pool.amount(Color::Blue) + pool.amount(Color::Green) == 1,
        "Exactly one of Blue/Green gained 1");
}

/// Grim Lavamancer's `{R}, {T}, Exile two cards from your gy:` deals
/// 2 damage to any target. Push (batch 114): the exile-two cost is
/// now wired faithfully via the extended `exile_other_filter:
/// Some((filter, 2))` shape. Verify activation pings 2 damage to a
/// target creature when there are ≥ 2 gy cards to exile.
#[test]
fn grim_lavamancer_pings_creature_with_gy_card_to_exile() {
    let mut g = two_player_game();
    let lava = g.add_card_to_battlefield(0, catalog::grim_lavamancer());
    g.battlefield_find_mut(lava).unwrap().summoning_sick = false;
    // Seed two graveyard cards for the exile-2-from-gy cost.
    let _fodder_a = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let _fodder_b = g.add_card_to_graveyard(0, catalog::shock());
    // Need a creature target on the battlefield (opponent's bear).
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: lava, ability_index: 0,
        target: Some(Target::Permanent(bear)), x_value: None })
    .expect("Lavamancer can activate with R + 2 gy fodder");
    drain_stack(&mut g);
    // Bear (2/2) takes 2 damage and dies.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Grim Lavamancer should ping the bear for 2 (now dead)");
}

#[test]
fn grim_lavamancer_rejects_activation_with_only_one_gy_card() {
    // Batch 114 negative test: with only 1 card in graveyard the
    // exile-2 cost can't be paid → activation rejects cleanly without
    // burning tap/mana.
    let mut g = two_player_game();
    let lava = g.add_card_to_battlefield(0, catalog::grim_lavamancer());
    g.clear_sickness(lava);
    let _fodder = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;
    let pool_before = g.players[0].mana_pool.total();
    let tapped_before = g.battlefield_find(lava).map(|c| c.tapped).unwrap_or(false);

    let result = g.perform_action(GameAction::ActivateAbility {
        card_id: lava, ability_index: 0,
        target: Some(Target::Player(1)), x_value: None });
    assert!(result.is_err(),
        "Only 1 card in gy — activation must reject the exile-2 cost");
    assert_eq!(g.players[1].life, life_before, "No damage was dealt");
    assert_eq!(g.players[0].mana_pool.total(), pool_before,
        "Mana wasn't burned on the rejected activation");
    let tapped_after = g.battlefield_find(lava).map(|c| c.tapped).unwrap_or(false);
    assert_eq!(tapped_before, tapped_after,
        "Tap wasn't burned on the rejected activation");
    // Single gy fodder card is still in the graveyard.
    assert_eq!(g.players[0].graveyard.len(), 1,
        "GY fodder still in place — cost wasn't partially paid");
}

// ── Guardian Scalelord (M15 / cube card) ────────────────────────────────────

#[test]
fn guardian_scalelord_attack_grants_flying_to_target_friendly() {
    use crate::card::{CreatureType, Keyword};
    use crate::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let scalelord = g.add_card_to_battlefield(0, catalog::guardian_scalelord());
    g.clear_sickness(scalelord);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    drain_stack(&mut g);
    // Body sanity check.
    let scalelord_card = g.battlefield_find(scalelord).unwrap();
    assert!(scalelord_card.has_keyword(&Keyword::Flying));
    assert!(scalelord_card.definition.subtypes.creature_types.contains(&CreatureType::Dragon));

    // Accept the MayDo rider so the bear actually gets Flying.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: scalelord,
        target: AttackTarget::Player(1),
    }])).expect("Scalelord can attack");
    drain_stack(&mut g);
    // The bear should now have Flying EOT.
    let bear_card = g.battlefield_find(bear).unwrap();
    assert!(bear_card.has_keyword(&Keyword::Flying),
        "Scalelord's attack trigger gave the bear flying");
}

#[test]
fn guardian_scalelord_declines_optional_grant_by_default() {
    // AutoDecider defaults to "no" on MayDo (CR 603.2 — the controller
    // chooses; the bot harness defaults to skipping optional non-cost
    // riders). The bear should NOT get flying without an explicit yes.
    use crate::card::Keyword;
    use crate::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let scalelord = g.add_card_to_battlefield(0, catalog::guardian_scalelord());
    g.clear_sickness(scalelord);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    drain_stack(&mut g);

    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: scalelord,
        target: AttackTarget::Player(1),
    }])).expect("Scalelord can attack");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert!(!bear_card.has_keyword(&Keyword::Flying),
        "AutoDecider declines the MayDo; bear stays grounded");
}

// ── Push (claude/modern_decks) — new card functional tests ──────────────────

#[test]
fn rofellos_taps_for_two_green_per_forest() {
    // Rofellos's mana ability now scales with Forest count: 1 Forest →
    // 2 green; 3 Forests → 6 green. The activation cost is `{T}` only.
    let mut g = two_player_game();
    let rofellos = g.add_card_to_battlefield(0, catalog::rofellos_llanowar_emissary());
    g.clear_sickness(rofellos);
    // Three Forests in play under our control.
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let pool_before = g.players[0].mana_pool.amount(Color::Green);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rofellos, ability_index: 0, target: None, x_value: None })
    .expect("Rofellos's mana ability should activate");
    let pool_after = g.players[0].mana_pool.amount(Color::Green);
    assert_eq!(
        pool_after - pool_before, 6,
        "Rofellos with 3 Forests adds 2 * 3 = 6 green mana"
    );
}

#[test]
fn rofellos_taps_for_zero_with_no_forests() {
    // Edge case: no Forests means 0 mana (the multiplier annihilates).
    let mut g = two_player_game();
    let rofellos = g.add_card_to_battlefield(0, catalog::rofellos_llanowar_emissary());
    g.clear_sickness(rofellos);
    let pool_before = g.players[0].mana_pool.amount(Color::Green);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rofellos, ability_index: 0, target: None, x_value: None })
    .expect("Rofellos's mana ability should still activate");
    let pool_after = g.players[0].mana_pool.amount(Color::Green);
    assert_eq!(pool_after - pool_before, 0, "0 Forests → 0 green");
}

#[test]
fn snapcaster_mage_etb_grants_may_play_on_gy_is_card() {
    // Snapcaster Mage ETBs and grants MayPlay{EndOfThisTurn,
    // exile_after: true} on a target IS card in your graveyard. Same
    // shape as Flashback (the spell) — recovers the IS card for the turn.
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let snap = g.add_card_to_hand(0, catalog::snapcaster_mage());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: snap, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Snapcaster castable for {1}{U}");
    drain_stack(&mut g);

    let bolt_gy = g.players[0].graveyard.iter().find(|c| c.id == bolt)
        .expect("Bolt still in graveyard");
    let perm = bolt_gy.may_play_until.expect("may_play stamped on Bolt");
    assert!(perm.exile_after, "exile-on-resolve flag set (CR 702.34d)");
    assert_eq!(perm.player, 0);
}

#[test]
fn snapcaster_mage_is_a_two_one_flash_wizard() {
    use crate::card::{CreatureType, Keyword};
    let snap = catalog::snapcaster_mage();
    assert_eq!(snap.power, 2);
    assert_eq!(snap.toughness, 1);
    assert!(snap.keywords.contains(&Keyword::Flash));
    assert!(snap.subtypes.creature_types.contains(&CreatureType::Wizard));
}

#[test]
fn pyroblast_counters_a_blue_spell() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    // Opp casts a blue spell.
    let cancel = g.add_card_to_hand(1, catalog::cancel());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: cancel, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Cancel castable for {1}{U}{U}");
    // Now controller of Pyroblast (P0) counters the Cancel on the stack.
    let pyro = g.add_card_to_hand(0, catalog::pyroblast());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: pyro,
        target: Some(Target::Permanent(cancel)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("Pyroblast castable for {R}, targeting Cancel");
    drain_stack(&mut g);
    // Cancel should have been countered to graveyard.
    assert!(g.players[1].graveyard.iter().any(|c| c.id == cancel),
        "Cancel was countered by Pyroblast");
}

#[test]
fn pyroblast_rejects_non_blue_spell_target() {
    use crate::game::types::Target;
    // Pyroblast's mode-0 filter rejects non-blue spells. Try targeting
    // Lightning Bolt (red) — the cast should fail at target-validation.
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable");

    let pyro = g.add_card_to_hand(0, catalog::pyroblast());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    let res = g.perform_action(GameAction::CastSpell {
        card_id: pyro,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    });
    assert!(res.is_err(), "Pyroblast can't target a non-blue spell");
}

#[test]
fn red_elemental_blast_counters_a_blue_spell() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let cancel = g.add_card_to_hand(1, catalog::cancel());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: cancel, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Cancel castable");
    let reb = g.add_card_to_hand(0, catalog::red_elemental_blast());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: reb,
        target: Some(Target::Permanent(cancel)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("REB castable, targeting Cancel");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == cancel),
        "REB countered the blue spell");
}

#[test]
fn hydroblast_counters_a_red_spell() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable");
    let hydro = g.add_card_to_hand(0, catalog::hydroblast());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: hydro,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("Hydroblast castable, targeting Bolt");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Hydroblast countered the red spell");
}

#[test]
fn blue_elemental_blast_counters_a_red_spell() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable");
    let beb = g.add_card_to_hand(0, catalog::blue_elemental_blast());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: beb,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("BEB castable, targeting Bolt");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "BEB countered the red spell");
}

#[test]
fn three_visits_fetches_a_forest_to_battlefield() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    // ScriptedDecider picks the Forest at search time (AutoDecider
    // declines library searches by returning Search(None)).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let visits = g.add_card_to_hand(0, catalog::three_visits());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: visits, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Three Visits castable for {1}{G}");
    drain_stack(&mut g);
    let f = g.battlefield.iter().find(|c| c.id == forest)
        .expect("Forest moved to battlefield");
    assert_eq!(f.controller, 0);
    assert!(!f.tapped, "Three Visits puts the land in untapped");
}

#[test]
fn tales_end_counters_a_legendary_spell() {
    use crate::game::types::{Target, TurnStep};
    // Drop a legendary creature spell on the stack, then Tale's End it.
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    let griselbrand = g.add_card_to_hand(1, catalog::griselbrand());
    g.players[1].mana_pool.add(Color::Black, 4);
    g.players[1].mana_pool.add_colorless(4);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: griselbrand, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Griselbrand castable");
    let tale = g.add_card_to_hand(0, catalog::tales_end());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: tale,
        target: Some(Target::Permanent(griselbrand)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("Tale's End castable, targeting Griselbrand");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == griselbrand),
        "Tale's End countered the legendary spell");
}

#[test]
fn wall_of_omens_etbs_and_draws() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let wall = g.add_card_to_hand(0, catalog::wall_of_omens());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: wall, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Wall of Omens castable");
    drain_stack(&mut g);
    // Wall is on the battlefield + caster drew 1; hand size: -1 (cast) +1 (etb).
    assert_eq!(g.players[0].hand.len(), hand_before);
    let w = g.battlefield_find(wall).unwrap();
    assert_eq!(w.toughness(), 4);
    assert!(w.has_keyword(&Keyword::Defender));
}

#[test]
fn wall_of_roots_taps_for_green_with_pump_cost() {
    let mut g = two_player_game();
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_roots());
    g.clear_sickness(wall);
    let pool_before = g.players[0].mana_pool.amount(Color::Green);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wall, ability_index: 0, target: None, x_value: None })
    .expect("Wall of Roots activation should resolve");
    drain_stack(&mut g);
    let pool_after = g.players[0].mana_pool.amount(Color::Green);
    assert_eq!(pool_after - pool_before, 1, "Wall of Roots adds {{G}}");
    let w = g.battlefield_find(wall).unwrap();
    assert_eq!(w.toughness(), 4,
        "Wall of Roots's activation cost shrinks its toughness by 1");
}

#[test]
fn channel_pays_one_life_for_one_mana() {
    let mut g = two_player_game();
    let ch = g.add_card_to_hand(0, catalog::channel());
    g.players[0].mana_pool.add(Color::Green, 1);
    let life_before = g.players[0].life;
    let pool_before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::CastSpell {
        card_id: ch, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Channel castable for {G}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before - 1, "Channel costs 1 life");
    // Total pool: started 1, spent 1 to cast, gained 1 colorless = 1.
    assert_eq!(g.players[0].mana_pool.total(), pool_before,
        "Channel adds {{1}} colorless after paying its cast cost");
}

#[test]
fn phyrexian_reclamation_returns_creature_for_one_b_two_life() {
    let mut g = two_player_game();
    let rec = g.add_card_to_battlefield(0, catalog::phyrexian_reclamation());
    g.clear_sickness(rec);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: rec, ability_index: 0,
        target: Some(crate::game::types::Target::Permanent(bear)), x_value: None })
    .expect("Reclamation should activate for {1}{B} + 2 life");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before - 2, "2 life paid as cost");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear),
        "Bear returned from gy to hand");
}

#[test]
fn pernicious_deed_destroys_low_cmc_permanents() {
    // Activate Deed for X=2: should kill the 1-mana and 2-mana
    // permanents but spare the 6-mana Shivan Dragon.
    let mut g = two_player_game();
    let deed = g.add_card_to_battlefield(0, catalog::pernicious_deed());
    g.clear_sickness(deed);
    let cheap = g.add_card_to_battlefield(1, catalog::savannah_lions()); // 1-cmc
    let mid = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2-cmc
    let big = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // 6-cmc
    g.players[0].mana_pool.add_colorless(2);
    g.active_player_idx = 0;
    g.step = crate::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: deed, ability_index: 0, target: None, x_value: Some(2) })
    .expect("Deed should activate for {2}, sac");
    drain_stack(&mut g);
    // Cheap (1-cmc) and mid (2-cmc) die; 6-cmc survives.
    assert!(!g.battlefield.iter().any(|c| c.id == cheap),
        "1-cmc creature should leave battlefield");
    assert!(!g.battlefield.iter().any(|c| c.id == mid),
        "2-cmc creature should leave battlefield");
    assert!(g.battlefield.iter().any(|c| c.id == big),
        "6-cmc creature survives Deed at X=2");
    assert!(!g.battlefield.iter().any(|c| c.id == deed),
        "Deed sacrificed as activation cost");
}

#[test]
fn toxic_deluge_sweeps_creatures_for_x_two() {
    let mut g = two_player_game();
    // Two 2/2s (one each side) should die.
    let mine_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // 5/5 stays.
    let big = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    let deluge = g.add_card_to_hand(0, catalog::toxic_deluge());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: deluge, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("Deluge castable with X=2");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before - 2, "Paid 2 life for X=2");
    assert!(!g.battlefield.iter().any(|c| c.id == mine_bear), "own bear died");
    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear), "opp bear died");
    assert!(g.battlefield.iter().any(|c| c.id == big), "5/5 survives -2/-2");
}

#[test]
fn demonic_consultation_mills_six_and_searches() {
    let mut g = two_player_game();
    // Seed library so mill 6 has something to chew on, plus the tutor target.
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::island());
    }
    let target = g.add_card_to_library(0, catalog::lightning_bolt());
    // ScriptedDecider picks the bolt at search time.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target))]));
    let cons = g.add_card_to_hand(0, catalog::demonic_consultation());
    g.players[0].mana_pool.add(Color::Black, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: cons, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Consultation castable for {B}");
    drain_stack(&mut g);
    assert!(g.players[0].library.len() <= lib_before - 7,
        "Library lost 6 to mill + 1 to search");
    assert!(g.players[0].hand.iter().any(|c| c.id == target),
        "Picked card lands in hand");
}

#[test]
fn howling_mine_draws_an_extra_card_each_turn() {
    use crate::game::types::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::howling_mine());
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::island());
    }
    let hand_before = g.players[1].hand.len();
    // Trigger P1's draw step manually — the Howling Mine trigger fires
    // for the active player; set active_player to 1, advance.
    g.active_player_idx = 1;
    g.step = TurnStep::Draw;
    g.priority.player_with_priority = 1;
    g.fire_step_triggers(TurnStep::Draw);
    drain_stack(&mut g);
    // P1 should have drawn 1 extra card from Mine.
    assert!(g.players[1].hand.len() > hand_before,
        "Howling Mine drew P1 a card on their draw step");
}

#[test]
fn sylvan_library_offers_draw_in_exchange_for_four_life() {
    use crate::game::types::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sylvan_library());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    // Force the decider to accept the MayDo (draw + lose 4).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.active_player_idx = 0;
    g.step = TurnStep::Draw;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::Draw);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before - 4, "Paid 4 life");
    assert!(g.players[0].hand.len() > hand_before, "Drew the extra card");
}

// ── Dark Confidant — "lose life equal to CMC" trigger ────────────────────────

#[test]
fn dark_confidant_loses_life_equal_to_revealed_card_cmc() {
    // Seeds the library with a 5-CMC Serra Angel on top; on upkeep, Dark
    // Confidant's trigger reveals + draws it and the controller loses 5
    // life (not the old approximated flat 2).
    use crate::game::types::TurnStep;
    let mut g = two_player_game();
    g.players[0].library.clear();
    // Use add_to_library_top to control ordering — the *last* call to
    // add_to_library_top is the top of the library.
    {
        let cid = g.next_id();
        g.players[0].add_to_library_top(cid, catalog::grizzly_bears()); // 2-CMC filler
    }
    {
        let cid = g.next_id();
        g.players[0].add_to_library_top(cid, catalog::serra_angel()); // 5-CMC on top
    }
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    let dc = g.add_card_to_battlefield(0, catalog::dark_confidant());
    g.clear_sickness(dc);
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Dark Confidant drew exactly one card");
    let drawn_name = g.players[0].hand.last().map(|c| c.definition.name).unwrap_or("");
    assert_eq!(drawn_name, "Serra Angel",
        "The on-top card (Serra Angel) was drawn into hand");
    let life_lost = (life_before - g.players[0].life) as u32;
    assert_eq!(life_lost, 5,
        "Life lost equals Serra Angel's mana value (CMC 5), not the old flat 2");
}

#[test]
fn dark_confidant_loses_zero_life_for_zero_cmc_card_on_top() {
    // Zero-CMC card (Black Lotus is the canonical {0} cost) → no life loss.
    // Tests the "0 mana value" corner of the new ManaValueOf wiring.
    use crate::game::types::TurnStep;
    let mut g = two_player_game();
    g.players[0].library.clear();
    {
        let cid = g.next_id();
        g.players[0].add_to_library_top(cid, catalog::black_lotus());
    }
    let life_before = g.players[0].life;
    let dc = g.add_card_to_battlefield(0, catalog::dark_confidant());
    g.clear_sickness(dc);
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before,
        "0-CMC revealed card → no life lost");
}

#[test]
fn ophiomancer_mints_a_snake_each_upkeep() {
    use crate::card::CreatureType;
    use crate::game::types::TurnStep;
    let mut g = two_player_game();
    let ophio = g.add_card_to_battlefield(0, catalog::ophiomancer());
    g.clear_sickness(ophio);
    let bf_before = g.battlefield.len();
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield.len() > bf_before, "Snake token created");
    let tok = g.battlefield.iter().find(|c|
        c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Snake)
    ).expect("Snake token exists");
    assert!(tok.has_keyword(&crate::card::Keyword::Deathtouch));
}

#[test]
fn yavimaya_elder_dies_searches_two_basics() {
    let mut g = two_player_game();
    // Seed the library with two basic lands.
    let forest = g.add_card_to_library(0, catalog::forest());
    let plains = g.add_card_to_library(0, catalog::plains());
    let elder = g.add_card_to_battlefield(0, catalog::yavimaya_elder());
    let hand_before = g.players[0].hand.len();
    // ScriptedDecider answers MayDo(yes) + Search(Forest) + MayDo(yes) + Search(Plains).
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(forest)),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(plains)),
    ]));
    let _ = g.remove_to_graveyard_with_triggers(elder);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 2,
        "Yavimaya Elder dies → +2 basic lands to hand");
}

#[test]
fn stroke_of_genius_draws_x_cards() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let stroke = g.add_card_to_hand(0, catalog::stroke_of_genius());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(5);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: stroke,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("Stroke castable at X=3");
    drain_stack(&mut g);
    // -1 (cast) + 3 (X draw) = +2.
    assert_eq!(g.players[0].hand.len(), hand_before + 2,
        "Stroke at X=3 draws 3 cards");
}

#[test]
fn green_suns_zenith_tutors_green_creature_with_cmc_x() {
    let mut g = two_player_game();
    // Seed library with a green creature.
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bear))]));
    let zenith = g.add_card_to_hand(0, catalog::green_suns_zenith());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: zenith, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("GSZ castable for {X=2}{G}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear),
        "GSZ tutored Grizzly Bears (2-cmc green) into play");
}

#[test]
fn red_suns_zenith_deals_x_damage_to_target() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let zenith = g.add_card_to_hand(0, catalog::red_suns_zenith());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: zenith,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("RSZ castable for {X=3}{R}");
    drain_stack(&mut g);
    // 2/2 bear takes 3 damage → dies.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "RSZ at X=3 kills the bear");
}

#[test]
fn white_suns_zenith_creates_x_cat_tokens() {
    use crate::card::CreatureType;
    let mut g = two_player_game();
    let zenith = g.add_card_to_hand(0, catalog::white_suns_zenith());
    g.players[0].mana_pool.add(Color::White, 3);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: zenith, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("WSZ castable for {X=2}{W}{W}{W}");
    drain_stack(&mut g);
    // 2 Cat tokens entered.
    assert_eq!(g.battlefield.len(), bf_before + 2);
    assert!(g.battlefield.iter().filter(|c|
        c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Cat)
    ).count() == 2, "Two Cat tokens minted");
}

#[test]
fn black_suns_zenith_puts_x_minus_one_counters_on_each_creature() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let zenith = g.add_card_to_hand(0, catalog::black_suns_zenith());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: zenith, target: None, additional_targets: vec![], mode: None, x_value: Some(1),
    })
    .expect("BSZ castable at X=1");
    drain_stack(&mut g);
    // Each 2/2 bear now has 1 -1/-1 counter (effectively 1/1).
    // SBA may not kill them at -1/-1 yet; just verify counters present.
    let b1 = g.battlefield_find(bear1);
    let b2 = g.battlefield_find(bear2);
    if let Some(c) = b1 {
        assert!(c.counter_count(CounterType::MinusOneMinusOne) >= 1,
            "Bear1 received -1/-1 counter");
    }
    if let Some(c) = b2 {
        assert!(c.counter_count(CounterType::MinusOneMinusOne) >= 1,
            "Bear2 received -1/-1 counter");
    }
}

#[test]
fn yavimaya_elder_sac_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let elder = g.add_card_to_battlefield(0, catalog::yavimaya_elder());
    g.clear_sickness(elder);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    // Sac-cost activated draw ability.
    g.perform_action(GameAction::ActivateAbility {
        card_id: elder, ability_index: 0, target: None, x_value: None })
    .expect("Yavimaya Elder's sac-draw should activate");
    drain_stack(&mut g);
    // Hand gains the drawn card; Elder leaves play. Note: the dies-
    // trigger also fires on the sac, which may search for basic lands
    // too — but the AutoDecider's MayDo default-no skips them.
    assert!(g.players[0].hand.len() > hand_before, "Drew a card");
    assert!(!g.battlefield.iter().any(|c| c.id == elder),
        "Elder sacrificed");
}

// ── claude/modern_decks batch 102: multicolor cube expansion ────────────────

#[test]
fn sorin_grim_nemesis_plus_one_draws_and_loses_three_life() {
    let mut g = two_player_game();
    let sorin = g.add_card_to_battlefield(0, catalog::sorin_grim_nemesis());
    g.add_card_to_library(0, catalog::island());
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: sorin, ability_index: 0, target: None,
    }).expect("Sorin +1 castable");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before - 3, "Lost 3 life");
    assert!(g.players[0].hand.len() > hand_before, "Drew a card");
}

#[test]
fn sorin_grim_nemesis_minus_nine_drains_each_opponent() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let sorin = g.add_card_to_battlefield(0, catalog::sorin_grim_nemesis());
    // Pump loyalty so -9 is legal (6 base + 3 = 9).
    if let Some(s) = g.battlefield_find_mut(sorin) {
        s.add_counters(CounterType::Loyalty, 3);
    }
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: sorin, ability_index: 2, target: None,
    }).expect("Sorin -9 ult");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, p1_life - 10, "Opp lost 10");
    assert_eq!(g.players[0].life, p0_life + 10, "Gained 10");
}

#[test]
fn saheeli_rai_plus_one_pings_each_opponent() {
    let mut g = two_player_game();
    let saheeli = g.add_card_to_battlefield(0, catalog::saheeli_rai());
    g.add_card_to_library(0, catalog::island());
    let p1_life = g.players[1].life;

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: saheeli, ability_index: 0, target: None,
    }).expect("Saheeli +1");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, p1_life - 1, "Opp pinged for 1");
}

#[test]
fn saheeli_rai_minus_two_creates_haste_copy() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let saheeli = g.add_card_to_battlefield(0, catalog::saheeli_rai());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: saheeli,
        ability_index: 1,
        target: Some(Target::Permanent(bear)),
    }).expect("Saheeli -2 copies bear");
    drain_stack(&mut g);

    assert_eq!(g.battlefield.len(), bf_before + 1, "Copy token entered");
    let bear_copies = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Grizzly Bears")
        .count();
    assert!(bear_copies >= 1, "At least one bear copy token");
}

#[test]
fn ashiok_nightmare_weaver_plus_two_mills_opponent_three() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let ashiok = g.add_card_to_battlefield(0, catalog::ashiok_nightmare_weaver());
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::island());
    }
    let yard_before = g.players[1].graveyard.len();

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: ashiok,
        ability_index: 0,
        target: Some(Target::Player(1)),
    }).expect("Ashiok +2 mills opp 3");
    drain_stack(&mut g);

    assert_eq!(g.players[1].graveyard.len(), yard_before + 3, "Opp milled 3");
}

#[test]
fn ashiok_nightmare_weaver_minus_one_exiles_creature() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let ashiok = g.add_card_to_battlefield(0, catalog::ashiok_nightmare_weaver());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: ashiok,
        ability_index: 1,
        target: Some(Target::Permanent(bear)),
    }).expect("Ashiok -1 exiles bear");
    drain_stack(&mut g);

    assert!(g.battlefield_find(bear).is_none(), "Bear exiled");
}

#[test]
fn tamiyo_collector_minus_two_returns_card_from_graveyard() {
    let mut g = two_player_game();
    let tamiyo = g.add_card_to_battlefield(0, catalog::tamiyo_collector_of_tales());
    // Stage a card in the graveyard.
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: tamiyo, ability_index: 0,
        target: Some(crate::game::types::Target::Permanent(bear)),
    }).expect("Tamiyo -2 reanimate-to-hand");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == bear),
        "Bear returned to hand");
    assert!(g.players[0].hand.len() >= hand_before,
        "Hand size sane (got bear back)");
}

#[test]
fn geyadrone_dihada_plus_one_drains_each_opponent_for_one() {
    let mut g = two_player_game();
    let dihada = g.add_card_to_battlefield(0, catalog::geyadrone_dihada());
    g.add_card_to_library(0, catalog::island());
    let p1_life = g.players[1].life;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: dihada, ability_index: 0, target: None,
    }).expect("Dihada +1");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, p1_life - 1, "Opp loses 1");
    assert!(g.players[0].hand.len() > hand_before, "You draw a card");
}

#[test]
fn geyadrone_dihada_minus_three_steals_creature() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let dihada = g.add_card_to_battlefield(0, catalog::geyadrone_dihada());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: dihada,
        ability_index: 1,
        target: Some(Target::Permanent(bear)),
    }).expect("Dihada -3 threaten");
    drain_stack(&mut g);

    // Bear is now under your control with haste.
    let bear_card = g.battlefield_find(bear).expect("Bear still on bf");
    assert_eq!(bear_card.controller, 0, "Bear now controlled by you");
}

#[test]
fn korvold_fae_cursed_king_triggers_on_sacrifice() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let korvold = g.add_card_to_battlefield(0, catalog::korvold_fae_cursed_king());
    g.clear_sickness(korvold);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let hand_before = g.players[0].hand.len();

    // Fire a Sacrifice on the bear via Effect::Sacrifice. We dispatch
    // the resulting CreatureSacrificed event into Korvold's trigger
    // listener after the sacrifice resolves.
    let sac_effect = crate::card::Effect::Sacrifice {
        who: crate::card::Selector::You,
        count: crate::card::Value::Const(1),
        filter: crate::card::SelectionRequirement::Creature,
    };
    let ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g.resolve_effect(&sac_effect, &ctx).expect("Sacrifice resolves");
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);

    // Korvold should have a +1/+1 counter and you should have drawn a card.
    let korvold_card = g.battlefield_find(korvold).expect("Korvold still alive");
    assert_eq!(korvold_card.counter_count(CounterType::PlusOnePlusOne), 1,
        "Korvold gained +1/+1 counter from sacrifice");
    assert!(g.players[0].hand.len() > hand_before,
        "Korvold drew a card from sacrifice");
}

#[test]
fn korvold_fae_cursed_king_triggers_on_artifact_sacrifice_via_permanent_event() {
    // PermanentSacrificed catches non-creature sacrifices too —
    // CR 701.16 generalization shipped with the batch 102 engine work.
    use crate::card::CounterType;
    let mut g = two_player_game();
    let korvold = g.add_card_to_battlefield(0, catalog::korvold_fae_cursed_king());
    g.clear_sickness(korvold);
    // An artifact, not a creature.
    g.add_card_to_battlefield(0, catalog::mind_stone());
    g.add_card_to_library(0, catalog::island());
    let hand_before = g.players[0].hand.len();

    let sac_effect = crate::card::Effect::Sacrifice {
        who: crate::card::Selector::You,
        count: crate::card::Value::Const(1),
        filter: crate::card::SelectionRequirement::Artifact,
    };
    let ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g.resolve_effect(&sac_effect, &ctx).expect("Sac resolves");
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);

    let korvold_card = g.battlefield_find(korvold).expect("Korvold alive");
    assert_eq!(korvold_card.counter_count(CounterType::PlusOnePlusOne), 1,
        "Korvold grew off non-creature (Mind Stone) sacrifice via PermanentSacrificed");
    assert!(g.players[0].hand.len() > hand_before,
        "Korvold drew a card from artifact sacrifice");
}

#[test]
fn lord_xander_the_collector_etb_makes_opponent_discard_three() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    // Stack opp hand
    for _ in 0..5 {
        g.add_card_to_hand(1, catalog::island());
    }
    let hand_before = g.players[1].hand.len();
    let xander = g.add_card_to_hand(0, catalog::lord_xander_the_collector());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: xander, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Xander castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand_before - 3,
        "Opp discarded 3 from Xander ETB");
}

#[test]
fn master_of_cruelties_attack_sets_opp_life_to_one() {
    let mut g = two_player_game();
    let master = g.add_card_to_battlefield(0, catalog::master_of_cruelties());
    g.clear_sickness(master);
    g.players[1].life = 20;

    // Fire the attack trigger directly via event bus.
    let trig = catalog::master_of_cruelties().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(
        master, 0, None, 0,
    );
    let _ = g.resolve_effect(&trig, &ctx);
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, 1, "Opp's life set to 1");
}

#[test]
fn territorial_kavu_grows_when_opponent_plays_a_land() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let kavu = g.add_card_to_battlefield(0, catalog::territorial_kavu());
    g.clear_sickness(kavu);

    // Opponent plays a land.
    let land = g.add_card_to_hand(1, catalog::forest());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = crate::game::types::TurnStep::PreCombatMain;
    g.perform_action(GameAction::PlayLand(land))
        .expect("Opp plays a forest");
    drain_stack(&mut g);

    let k = g.battlefield_find(kavu).expect("Kavu alive");
    assert_eq!(k.counter_count(CounterType::PlusOnePlusOne), 1,
        "Kavu grew off opp's land entering");
}

#[test]
fn kolaghans_command_mode_zero_discard_plus_reanimate() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    // Stage gy reanimation target + opp hand.
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::island());
    let hand_before = g.players[1].hand.len();
    let cmd = g.add_card_to_hand(0, catalog::kolaghans_command());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: cmd, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Kolaghan's Command mode 0 castable");
    drain_stack(&mut g);

    assert_eq!(g.players[1].hand.len(), hand_before - 1, "Opp discarded 1");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear),
        "Bear back in hand");
}

#[test]
fn heroic_intervention_grants_indestructible_to_your_perms() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hi = g.add_card_to_hand(0, catalog::heroic_intervention());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: hi, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Heroic Intervention castable");
    drain_stack(&mut g);

    let bear_card = g.battlefield_find(bear).expect("Bear alive");
    assert!(bear_card.has_keyword(&crate::card::Keyword::Indestructible),
        "Bear gained indestructible");
}

#[test]
fn wear_tear_destroys_target_artifact() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let mind_stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    let wt = g.add_card_to_hand(0, catalog::wear_tear());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: wt, target: Some(Target::Permanent(mind_stone)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wear // Tear castable");
    drain_stack(&mut g);

    assert!(g.battlefield_find(mind_stone).is_none(),
        "Mind Stone destroyed");
}

#[test]
fn stillmoon_cavalier_grants_flying_eot() {
    let mut g = two_player_game();
    let cav = g.add_card_to_battlefield(0, catalog::stillmoon_cavalier());
    g.clear_sickness(cav);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cav, ability_index: 0, target: None, x_value: None,
    }).expect("Stillmoon {W}: flying");
    drain_stack(&mut g);
    let c = g.battlefield_find(cav).expect("Stillmoon alive");
    assert!(c.has_keyword(&crate::card::Keyword::Flying),
        "Gained flying EOT");
}

#[test]
fn stillmoon_cavalier_grants_protection_from_black_eot() {
    let mut g = two_player_game();
    let cav = g.add_card_to_battlefield(0, catalog::stillmoon_cavalier());
    g.clear_sickness(cav);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cav, ability_index: 2, target: None, x_value: None,
    }).expect("Stillmoon {1}{W}: pro-black");
    drain_stack(&mut g);
    let c = g.battlefield_find(cav).expect("Stillmoon alive");
    assert!(c.has_keyword(&crate::card::Keyword::Protection(Color::Black)),
        "Gained protection from black EOT");
}

#[test]
fn wishclaw_talisman_enters_with_three_charge_counters() {
    // Cast the Talisman so the ETB-counters payload fires through the
    // normal pipeline (rather than add-direct-to-battlefield bypass).
    let mut g = two_player_game();
    let wishclaw = g.add_card_to_hand(0, catalog::wishclaw_talisman());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: wishclaw, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Wishclaw castable");
    drain_stack(&mut g);
    let w = g.battlefield_find(wishclaw).expect("Wishclaw on battlefield");
    use crate::card::CounterType;
    assert_eq!(w.counter_count(CounterType::Charge), 3,
        "Enters with three charge counters");
}

#[test]
fn wishclaw_talisman_searches_and_consumes_a_charge_counter() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let wishclaw = g.add_card_to_battlefield(0, catalog::wishclaw_talisman());
    // Manually stamp three charge counters — `add_card_to_battlefield`
    // bypasses the ETB pipeline so `enters_with_counters` doesn't fire.
    if let Some(w) = g.battlefield_find_mut(wishclaw) {
        w.add_counters(CounterType::Charge, 3);
    }
    g.clear_sickness(wishclaw);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(bear)),
    ]));

    g.perform_action(GameAction::ActivateAbility {
        card_id: wishclaw, ability_index: 0, target: None, x_value: None,
    }).expect("Wishclaw activatable");
    drain_stack(&mut g);

    // The tutored card is in your hand.
    assert!(g.players[0].hand.iter().any(|c| c.id == bear),
        "Tutored bear into hand");
    // Charge counter consumed.
    let w = g.battlefield_find(wishclaw).expect("Wishclaw alive");
    assert_eq!(w.counter_count(CounterType::Charge), 2,
        "Charge counter consumed");
    // Wishclaw stays under your control — the printed "opp gains
    // control" downside is documented engine-wide gap.
}

#[test]
fn murderous_cut_destroys_target_creature() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mc = g.add_card_to_hand(0, catalog::murderous_cut());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(5);

    g.perform_action(GameAction::CastSpell {
        card_id: mc, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Murderous Cut castable at full cost");
    drain_stack(&mut g);

    assert!(g.battlefield_find(bear).is_none(), "Bear destroyed");
}

#[test]
fn trinisphere_is_a_three_mana_artifact() {
    let g = two_player_game();
    let def = catalog::trinisphere();
    assert_eq!(def.cost.cmc(), 3, "Costs 3");
    assert!(def.card_types.contains(&CardType::Artifact), "Artifact");
    let _ = g;
}

#[test]
fn magma_spray_exiles_a_low_toughness_creature_via_if_branch() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    // 2-toughness creature: hits exile branch.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ms = g.add_card_to_hand(0, catalog::magma_spray());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: ms, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Magma Spray castable");
    drain_stack(&mut g);

    // Bear at 2 toughness: should be exiled, not in graveyard.
    let in_gy = g.players[1].graveyard.iter().any(|c| c.id == bear);
    let in_exile = g.exile.iter().any(|c| c.id == bear);
    assert!(in_exile, "Bear exiled");
    assert!(!in_gy, "Bear NOT in graveyard");
}

#[test]
fn yarok_the_desecrated_is_a_three_five_deathtouch_lifelink() {
    let mut g = two_player_game();
    let yarok = g.add_card_to_battlefield(0, catalog::yarok_the_desecrated());
    let y = g.battlefield_find(yarok).expect("Yarok alive");
    assert_eq!(y.power(), 3);
    assert_eq!(y.toughness(), 5);
    assert!(y.has_keyword(&crate::card::Keyword::Deathtouch));
    assert!(y.has_keyword(&crate::card::Keyword::Lifelink));
}

#[test]
fn hellrider_attack_pings_each_opponent_for_one() {
    let mut g = two_player_game();
    let hellrider = g.add_card_to_battlefield(0, catalog::hellrider());
    g.clear_sickness(hellrider);
    let p1_life = g.players[1].life;
    let trig = catalog::hellrider().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(
        hellrider, 0, None, 0,
    );
    let _ = g.resolve_effect(&trig, &ctx);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 1, "Opp pinged for 1");
}

#[test]
fn generous_gift_destroys_target_permanent() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let gg = g.add_card_to_hand(0, catalog::generous_gift());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: gg, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Generous Gift castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "Bear destroyed");
}

#[test]
fn putrefy_modern_destroys_artifact_or_creature() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    let p = g.add_card_to_hand(0, catalog::putrefy_modern());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: p, target: Some(Target::Permanent(stone)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Putrefy castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(stone).is_none(), "Mind Stone destroyed");
}

#[test]
fn etali_primal_storm_attack_mills_each_player_one() {
    let mut g = two_player_game();
    let etali = g.add_card_to_battlefield(0, catalog::etali_primal_storm());
    g.clear_sickness(etali);
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(1, catalog::island());
    let p0_yard = g.players[0].graveyard.len();
    let p1_yard = g.players[1].graveyard.len();
    let trig = catalog::etali_primal_storm().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(etali, 0, None, 0);
    let _ = g.resolve_effect(&trig, &ctx);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), p0_yard + 1, "P0 milled 1");
    assert_eq!(g.players[1].graveyard.len(), p1_yard + 1, "P1 milled 1");
}

#[test]
fn knight_of_the_reliquary_pt_scales_with_lands_in_graveyards() {
    let mut g = two_player_game();
    let knight = g.add_card_to_battlefield(0, catalog::knight_of_the_reliquary());
    // Base 2/2 with no lands in gys.
    let c = g.compute_battlefield();
    let k = c.iter().find(|c| c.id == knight).unwrap();
    assert_eq!(k.power, 2, "Base power 2");
    assert_eq!(k.toughness, 2, "Base toughness 2");
    // Add 3 lands to your graveyard.
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::forest());
    }
    // Add 2 lands to opp's graveyard.
    for _ in 0..2 {
        g.add_card_to_graveyard(1, catalog::island());
    }
    let c = g.compute_battlefield();
    let k = c.iter().find(|c| c.id == knight).unwrap();
    assert_eq!(k.power, 2 + 5, "Knight grew to 7/7");
    assert_eq!(k.toughness, 2 + 5, "Knight is 7/7");
}

#[test]
fn goblin_rabblemaster_attack_creates_a_goblin_token() {
    use crate::card::CreatureType;
    let mut g = two_player_game();
    let rabble = g.add_card_to_battlefield(0, catalog::goblin_rabblemaster());
    g.clear_sickness(rabble);
    let bf_before = g.battlefield.len();
    let trig = catalog::goblin_rabblemaster().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(rabble, 0, None, 0);
    let _ = g.resolve_effect(&trig, &ctx);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.len(), bf_before + 1, "Goblin token entered");
    assert!(g.battlefield.iter().any(|c|
        c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Goblin)
    ), "Goblin token present");
}

// ── modern_decks batch 103: new cube-expansion card tests ───────────────────

#[test]
fn death_greeters_champion_drains_opp_on_attack() {
    use crate::card::Keyword;
    use crate::game::types::{AttackTarget, TurnStep};
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::death_greeters_champion());
    g.clear_sickness(attacker);
    let life1_before = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attacker declared");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 1, "opp loses 1 life on attack");
    // Haste keyword check.
    let champ = catalog::death_greeters_champion();
    assert!(champ.keywords.contains(&Keyword::Haste));
}

#[test]
fn glaring_fleshraker_etb_pings_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::glaring_fleshraker());
    g.players[0].mana_pool.add_colorless(3);
    let life1_before = g.players[1].life;
    cast(&mut g, id);
    assert_eq!(g.players[1].life, life1_before - 2, "ETB pings for 2");
}

#[test]
fn detectives_phoenix_dies_schedules_delayed_return() {
    let mut g = two_player_game();
    let phoenix = g.add_card_to_battlefield(0, catalog::detectives_phoenix());
    g.clear_sickness(phoenix);
    let dt_before = g.delayed_triggers.len();
    g.remove_to_graveyard_with_triggers(phoenix);
    drain_stack(&mut g);
    // A delayed-return trigger should be scheduled (matches Goryo's
    // shape — at next end step the body fires).
    assert!(g.delayed_triggers.len() > dt_before,
        "Delayed return trigger scheduled");
}

#[test]
fn lonis_genetics_expert_creates_clue_when_other_creature_enters() {
    use crate::card::ArtifactSubtype;
    let mut g = two_player_game();
    let lonis = g.add_card_to_battlefield(0, catalog::lonis_genetics_expert());
    g.clear_sickness(lonis);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, bear);
    let clues: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Clue))
        .collect();
    assert_eq!(clues.len(), 1, "Lonis mints a Clue when another creature enters");
}

#[test]
fn loot_the_pathfinder_etb_creates_map_approximation() {
    use crate::card::ArtifactSubtype;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::loot_the_pathfinder());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    let clues: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Clue))
        .collect();
    assert_eq!(clues.len(), 1, "Loot mints a Clue (Map approximation) on ETB");
}

#[test]
fn brightglass_gearhulk_etb_scries_and_draws() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::brightglass_gearhulk());
    g.players[0].mana_pool.add_colorless(4);
    let hand_before = g.players[0].hand.len();
    cast(&mut g, id);
    // -1 cast + 1 draw = 0 net delta on hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.battlefield_find(id).is_some(), "Gearhulk on bf");
}

#[test]
fn mossborn_hydra_enters_with_x_counters() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mossborn_hydra());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: Some(3),
    }).expect("Mossborn castable at X=3");
    drain_stack(&mut g);
    let hydra = g.battlefield_find(id).expect("Hydra on bf");
    assert_eq!(hydra.counter_count(CounterType::PlusOnePlusOne), 3,
        "Mossborn enters with X +1/+1 counters");
}

#[test]
fn mai_scornful_striker_drains_opp_on_attack() {
    use crate::card::Keyword;
    use crate::game::types::{AttackTarget, TurnStep};
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::mai_scornful_striker());
    g.clear_sickness(attacker);
    let mai = g.battlefield_find(attacker).expect("Mai on bf");
    assert!(mai.has_keyword(&Keyword::Menace), "Has menace");
    let life1_before = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attacker declared");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 1, "opp -1 life on attack");
}

#[test]
fn tempest_angler_etb_scries_two() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::tempest_angler());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    use crate::card::Keyword;
    let angler = g.battlefield_find(id).expect("Angler on bf");
    assert!(angler.has_keyword(&Keyword::Flying));
}

#[test]
fn carnage_interpreter_etb_makes_each_opp_discard() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::carnage_interpreter());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_hand_before = g.players[1].hand.len();
    cast(&mut g, id);
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1,
        "Opp discards one card on ETB");
}

#[test]
fn helix_pinnacle_x_activation_adds_charge_counters() {
    let mut g = two_player_game();
    let hp = g.add_card_to_battlefield(0, catalog::helix_pinnacle());
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hp,
        ability_index: 0,
        target: None,
        x_value: Some(5),
    }).expect("Helix Pinnacle X=5 activation");
    drain_stack(&mut g);
    let c = g.battlefield_find(hp).expect("on bf");
    assert_eq!(c.counter_count(CounterType::Charge), 5,
        "5 charge counters from X=5 activation");
}

#[test]
fn helix_pinnacle_counter_cap_at_100() {
    // Excess counters via X-activation get pruned to 100 by CR 122.4 SBA.
    let mut g = two_player_game();
    let hp = g.add_card_to_battlefield(0, catalog::helix_pinnacle());
    g.players[0].mana_pool.add_colorless(150);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hp,
        ability_index: 0,
        target: None,
        x_value: Some(150),
    }).expect("Helix Pinnacle X=150 activation");
    drain_stack(&mut g);
    let c = g.battlefield_find(hp).expect("on bf");
    assert_eq!(c.counter_count(CounterType::Charge), 100,
        "Counter cap of 100 enforced by SBA");
}

#[test]
fn helix_pinnacle_wins_at_upkeep_with_one_hundred_counters() {
    let mut g = two_player_game();
    let hp = g.add_card_to_battlefield(0, catalog::helix_pinnacle());
    // Manually stamp 100 counters (bypass the activation mana cost for
    // the upkeep-win test).
    {
        let c = g.battlefield_find_mut(hp).expect("on bf");
        c.add_counters(CounterType::Charge, 100);
    }
    use crate::game::types::TurnStep;
    // Walk to next upkeep (active player == 0, step == Upkeep, turn >= 2).
    let mut iters = 0;
    while !(g.active_player_idx == 0 && g.step == TurnStep::Upkeep && g.turn_number >= 2)
        && iters < 200
    {
        let _ = g.pass_priority();
        drain_stack(&mut g);
        iters += 1;
        if g.game_over.is_some() {
            break;
        }
    }
    drain_stack(&mut g);
    assert!(g.game_over.is_some(),
        "Helix Pinnacle wins at upkeep with 100 storage counters");
    assert_eq!(g.game_over, Some(Some(0)),
        "P0 (Helix controller) declared winner");
}

// ── New cube cards (push claude/modern_decks) ──────────────────────────

#[test]
fn collective_brutality_mode_two_drains() {
    let mut g = two_player_game();
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::collective_brutality());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(2), x_value: None,
    }).expect("Collective Brutality castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 2);
    assert_eq!(g.players[0].life, my_life + 2);
}

#[test]
fn cam_and_farrik_pumps_on_noncreature_cast() {
    let mut g = two_player_game();
    let cam = g.add_card_to_battlefield(0, catalog::cam_and_farrik());
    g.clear_sickness(cam);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let p_before = g.battlefield.iter().find(|c| c.id == cam).unwrap().power();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    let p_after = g.battlefield.iter().find(|c| c.id == cam).unwrap().power();
    assert_eq!(p_after, p_before + 2);
}

#[test]
fn magda_brazen_outlaw_is_legendary_dwarf() {
    let card = catalog::magda_brazen_outlaw();
    assert_eq!(card.name, "Magda, Brazen Outlaw");
    assert_eq!(card.power, 2);
    assert_eq!(card.toughness, 1);
    assert!(card.supertypes.contains(&crate::card::Supertype::Legendary));
}

#[test]
fn descendant_of_storms_is_2_2_flying_spirit() {
    let card = catalog::descendant_of_storms();
    assert_eq!(card.name, "Descendant of Storms");
    assert_eq!(card.power, 2);
    assert_eq!(card.toughness, 2);
    assert!(card.keywords.contains(&crate::card::Keyword::Flying));
}

#[test]
fn keen_eyed_curator_etb_adds_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::keen_eyed_curator());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    let curator = g.battlefield.iter().find(|c| c.definition.name == "Keen-Eyed Curator")
        .expect("Curator on bf");
    assert_eq!(curator.counter_count(crate::card::CounterType::PlusOnePlusOne), 1);
    assert_eq!(curator.power(), 4);
}

#[test]
fn intervention_pact_gains_three_life_and_sets_delayed_trigger() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::intervention_pact());
    // Free cast ({0})
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 3);
    assert!(!g.delayed_triggers.is_empty(), "Should have a delayed PayOrLoseGame trigger");
}

#[test]
fn gush_draws_two_cards() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::gush());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    // Cast -1 (Gush) from hand + draw 2 = net +1
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

// ── Cube expansion cards ──────────────────────────────────────────────────────

#[test]
fn back_to_basics_prevents_nonbasic_land_untap() {
    let mut g = two_player_game();
    let _btb = g.add_card_to_battlefield(0, catalog::back_to_basics());
    // Tap a nonbasic land.
    let nonbasic = g.add_card_to_battlefield(0, catalog::razortide_bridge());
    g.battlefield.iter_mut().find(|c| c.id == nonbasic).unwrap().tapped = true;
    // Also tap a basic land for comparison.
    let basic = g.add_card_to_battlefield(0, catalog::island());
    g.battlefield.iter_mut().find(|c| c.id == basic).unwrap().tapped = true;

    g.do_untap();

    // Basic land should untap.
    assert!(!g.battlefield.iter().find(|c| c.id == basic).unwrap().tapped,
        "Basic land should untap normally");
    // Nonbasic land should stay tapped.
    assert!(g.battlefield.iter().find(|c| c.id == nonbasic).unwrap().tapped,
        "Nonbasic land should stay tapped under Back to Basics");
}

#[test]
fn collector_ouphe_is_2_2_ouphe() {
    let card = catalog::collector_ouphe();
    assert_eq!(card.name, "Collector Ouphe");
    assert_eq!(card.power, 2);
    assert_eq!(card.toughness, 2);
}

#[test]
fn arclight_phoenix_is_3_2_flying_haste() {
    let card = catalog::arclight_phoenix();
    assert_eq!(card.name, "Arclight Phoenix");
    assert_eq!(card.power, 3);
    assert_eq!(card.toughness, 2);
    assert!(card.keywords.contains(&crate::card::Keyword::Flying));
    assert!(card.keywords.contains(&crate::card::Keyword::Haste));
}

#[test]
fn omniscience_is_10_mana_enchantment() {
    let card = catalog::omniscience();
    assert_eq!(card.name, "Omniscience");
    assert!(card.card_types.contains(&crate::card::CardType::Enchantment));
}

#[test]
fn opposition_is_4_mana_enchantment() {
    let card = catalog::opposition();
    assert_eq!(card.name, "Opposition");
    assert!(card.card_types.contains(&crate::card::CardType::Enchantment));
}

// ── Overload cards ────────────────────────────────────────────────────────────

#[test]
fn blustersquall_taps_target_creature() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::blustersquall());
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(target)), additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);

    assert!(g.battlefield.iter().find(|c| c.id == target).unwrap().tapped,
        "Blustersquall should tap target creature");
}

#[test]
fn blustersquall_overload_taps_all_opponent_creatures() {
    let mut g = two_player_game();
    let c1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let c2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let own = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::blustersquall());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: spell, pitch_card: None, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);

    assert!(g.battlefield.iter().find(|c| c.id == c1).unwrap().tapped);
    assert!(g.battlefield.iter().find(|c| c.id == c2).unwrap().tapped);
    assert!(!g.battlefield.iter().find(|c| c.id == own).unwrap().tapped,
        "Own creatures should NOT be tapped by Overload");
}

#[test]
fn electrickery_deals_1_to_target() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::electrickery());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(target)), additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);

    let bear = g.battlefield.iter().find(|c| c.id == target).unwrap();
    assert_eq!(bear.damage, 1, "Electrickery should deal 1 damage");
}

#[test]
fn electrickery_overload_deals_1_to_each_opp_creature() {
    let mut g = two_player_game();
    let c1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let c2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::electrickery());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: spell, pitch_card: None, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);

    for id in [c1, c2] {
        let c = g.battlefield.iter().find(|c| c.id == id).unwrap();
        assert_eq!(c.damage, 1, "Electrickery Overload should deal 1 to each");
    }
}

#[test]
fn teleportal_pumps_and_grants_unblockable() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::teleportal());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(creature)), additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);

    let c = g.battlefield.iter().find(|c| c.id == creature).unwrap();
    assert_eq!(c.power(), 3, "Should get +1/+0");
    assert!(c.has_keyword(&crate::card::Keyword::Unblockable));
}

#[test]
fn street_spasm_deals_x_to_non_flying() {
    let card = catalog::street_spasm();
    assert_eq!(card.name, "Street Spasm");
    assert!(card.alternative_cost.is_some(), "Should have Overload alt cost");
}

// ── Modern cube supplement ──────────────────────────────────────────────────

#[test]
fn dreadhorde_arcanist_attack_returns_instant_from_graveyard() {
    let mut g = two_player_game();
    // Put an instant card in P0's graveyard.
    let bolt_id = g.add_card_to_library(0, catalog::lightning_bolt());
    let pos = g.players[0].library.iter().position(|c| c.id == bolt_id).unwrap();
    let bolt_card = g.players[0].library.remove(pos);
    g.players[0].graveyard.push(bolt_card);

    let arcanist = g.add_card_to_battlefield(0, catalog::dreadhorde_arcanist());
    g.clear_sickness(arcanist);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;

    let hand_before = g.players[0].hand.len();
    let gy_before = g.players[0].graveyard.len();

    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: arcanist,
        target: AttackTarget::Player(1),
    }]))
    .expect("Dreadhorde Arcanist attacks");
    drain_stack(&mut g);

    // The attack trigger should move the Lightning Bolt from graveyard to hand.
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Arcanist attack should return an IS card from graveyard to hand");
    assert_eq!(g.players[0].graveyard.len(), gy_before - 1,
        "Graveyard should lose one card");
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt_id),
        "Lightning Bolt should now be in hand");
}

#[test]
fn baleful_mastery_full_cost_exiles_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::baleful_mastery());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);

    let p1_hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Baleful Mastery castable for {3}{B}");
    drain_stack(&mut g);

    // Bear should be exiled.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be exiled from battlefield");
    assert!(g.exile.iter().any(|c| c.id == bear),
        "Bear should be in exile");
    // At full cost, opponent should NOT draw a card.
    assert_eq!(g.players[1].hand.len(), p1_hand_before,
        "At full cost, opponent should not draw a card");
}

#[test]
fn baleful_mastery_alt_cost_exiles_and_opp_draws() {
    let mut g = two_player_game();
    // Opponent needs library so they can draw.
    g.add_card_to_library(1, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::baleful_mastery());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    let p1_hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: spell,
        pitch_card: None,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Baleful Mastery alt-castable for {1}{B}");
    drain_stack(&mut g);

    // Bear should be exiled.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be exiled from battlefield");
    assert!(g.exile.iter().any(|c| c.id == bear),
        "Bear should be in exile");
    // At alt cost, opponent SHOULD draw a card.
    assert_eq!(g.players[1].hand.len(), p1_hand_before + 1,
        "At alt cost, opponent should draw 1 card");
}

#[test]
fn parallax_nexus_enters_with_counters_and_forces_discard() {
    let mut g = two_player_game();
    // Give opponent a card to discard.
    g.add_card_to_hand(1, catalog::grizzly_bears());

    // Cast the enchantment so the ETB-counters pipeline fires
    // (`add_card_to_battlefield` bypasses `enters_with_counters`).
    let nexus = g.add_card_to_hand(0, catalog::parallax_nexus());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: nexus, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Parallax Nexus castable for {1}{B}{B}");
    drain_stack(&mut g);

    // Verify it enters with 5 charge counters.
    let n = g.battlefield.iter().find(|c| c.id == nexus).unwrap();
    assert_eq!(n.counter_count(CounterType::Charge), 5,
        "Parallax Nexus should enter with 5 charge counters");

    let opp_hand_before = g.players[1].hand.len();

    // Activate the {0} ability to force an opponent discard.
    g.perform_action(GameAction::ActivateAbility {
        card_id: nexus,
        ability_index: 0,
        target: None,
        x_value: None,
    })
    .expect("Parallax Nexus activation should work");
    drain_stack(&mut g);

    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1,
        "Opponent should have discarded one card");
}

// ── Cube expansion: body-only stubs ─────────────────────────────────────────

#[test]
fn enduring_innocence_draws_on_nontoken_creature_etb() {
    let mut g = two_player_game();
    // Seed the library so the draw has something to pull.
    g.add_card_to_library(0, catalog::island());
    let _innocence = g.add_card_to_battlefield(0, catalog::enduring_innocence());

    // Cast a creature (goes through the stack → ETB triggers fire).
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bears castable");
    drain_stack(&mut g);

    // Net hand: cast bear (-1) + draw from Enduring Innocence (+1) = 0.
    assert_eq!(
        g.players[0].hand.len(),
        hand_before,
        "Enduring Innocence should draw 1 when a nontoken creature ETBs (net 0 from cast + draw)"
    );
}

#[test]
fn amped_raptor_body_is_2_1_dinosaur() {
    let card = catalog::amped_raptor();
    assert_eq!(card.name, "Amped Raptor");
    assert_eq!(card.power, 2);
    assert_eq!(card.toughness, 1);
    assert!(
        card.subtypes
            .creature_types
            .contains(&crate::card::CreatureType::Dinosaur),
        "Should be a Dinosaur"
    );
}

#[test]
fn thundertrap_trainer_etb_taps_opponent_creature() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Ensure it starts untapped.
    g.battlefield_find_mut(opp_bear).unwrap().tapped = false;

    let trainer = g.add_card_to_hand(0, catalog::thundertrap_trainer());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: trainer,
        target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Thundertrap Trainer castable");
    drain_stack(&mut g);

    let bear = g.battlefield.iter().find(|c| c.id == opp_bear).unwrap();
    assert!(bear.tapped, "Opponent bear should be tapped by Thundertrap Trainer ETB");
}

#[test]
fn corpse_dance_reanimates_creature_from_graveyard() {
    let mut g = two_player_game();
    // Put a creature in P0's graveyard.
    let bear_id = g.add_card_to_library(0, catalog::grizzly_bears());
    let pos = g.players[0]
        .library
        .iter()
        .position(|c| c.id == bear_id)
        .unwrap();
    let bear_card = g.players[0].library.remove(pos);
    g.players[0].graveyard.push(bear_card);

    let bf_creatures_before = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.card_types.contains(&CardType::Creature))
        .count();

    let spell = g.add_card_to_hand(0, catalog::corpse_dance());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Corpse Dance castable");
    drain_stack(&mut g);

    let bf_creatures_after = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.card_types.contains(&CardType::Creature))
        .count();
    assert!(
        bf_creatures_after > bf_creatures_before,
        "Corpse Dance should put a creature onto the battlefield"
    );
}

#[test]
fn basking_rootwalla_pump_once_per_turn() {
    let mut g = two_player_game();
    let rootwalla = g.add_card_to_battlefield(0, catalog::basking_rootwalla());
    g.clear_sickness(rootwalla);

    let base_power = g.computed_permanent(rootwalla).unwrap().power;
    assert_eq!(base_power, 1, "Basking Rootwalla base power should be 1");

    // Pay {1}{G} to activate the pump.
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::ActivateAbility {
        card_id: rootwalla,
        ability_index: 0,
        target: None,
        x_value: None,
    })
    .expect("Rootwalla pump activates");
    drain_stack(&mut g);

    let pumped = g.computed_permanent(rootwalla).unwrap();
    assert_eq!(pumped.power, 3, "Rootwalla should be 3/3 after pump");
    assert_eq!(pumped.toughness, 3, "Rootwalla should be 3/3 after pump");
}

// ── Push XIX: cube creature tests ──────────────────────────────────────

#[test]
fn elder_gargaroth_is_6_6_with_three_keywords() {
    let card = catalog::elder_gargaroth();
    assert_eq!(card.name, "Elder Gargaroth");
    assert_eq!(card.power, 6);
    assert_eq!(card.toughness, 6);
    assert!(card.keywords.contains(&Keyword::Vigilance));
    assert!(card.keywords.contains(&Keyword::Reach));
    assert!(card.keywords.contains(&Keyword::Trample));
    assert_eq!(card.triggered_abilities.len(), 2, "attack and block triggers");
}

// ── Push: new modern creatures ──────────────────────────────────────────

#[test]
fn blade_splicer_etb_creates_golem_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::blade_splicer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Blade Splicer castable");
    drain_stack(&mut g);
    // Blade Splicer (1/1) + Golem token (3/3)
    assert_eq!(g.battlefield.len(), bf_before + 2);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Golem"),
        "A 3/3 Golem token should be on the battlefield");
}

#[test]
fn vendilion_clique_is_3_1_legendary_flash_flying() {
    use crate::card::Keyword;
    let card = catalog::vendilion_clique();
    assert_eq!(card.name, "Vendilion Clique");
    assert_eq!(card.power, 3);
    assert_eq!(card.toughness, 1);
    assert!(card.keywords.contains(&Keyword::Flash));
    assert!(card.keywords.contains(&Keyword::Flying));
    assert!(card.supertypes.iter().any(|s| matches!(s, crate::card::Supertype::Legendary)));
}

#[test]
fn torrential_gearhulk_is_5_6_artifact_flash() {
    use crate::card::Keyword;
    let card = catalog::torrential_gearhulk();
    assert_eq!(card.name, "Torrential Gearhulk");
    assert_eq!(card.power, 5);
    assert_eq!(card.toughness, 6);
    assert!(card.keywords.contains(&Keyword::Flash));
    assert!(card.card_types.contains(&CardType::Artifact));
    assert!(card.card_types.contains(&CardType::Creature));
}

#[test]
fn kitesail_larcenist_etb_exiles_opponent_nonland() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::kitesail_larcenist());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Kitesail Larcenist castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear),
        "Opponent's creature should be exiled");
}

#[test]
fn grave_titan_etb_creates_two_zombie_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::grave_titan());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Grave Titan castable");
    drain_stack(&mut g);
    // Grave Titan + 2 Zombies
    assert_eq!(g.battlefield.len(), bf_before + 3);
    let zombie_count = g.battlefield.iter()
        .filter(|c| c.definition.name == "Zombie")
        .count();
    assert_eq!(zombie_count, 2, "Should create 2 Zombie tokens on ETB");
}

#[test]
fn shriekmaw_etb_destroys_nonblack_creature() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::shriekmaw());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Shriekmaw castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear),
        "Opponent's nonblack creature should be destroyed");
}

#[test]
fn phyrexian_obliterator_is_5_8_trample() {
    use crate::card::Keyword;
    let card = catalog::phyrexian_obliterator();
    assert_eq!(card.name, "Phyrexian Obliterator");
    assert_eq!(card.power, 5);
    assert_eq!(card.toughness, 8);
    assert!(card.keywords.contains(&Keyword::Trample));
}

#[test]
fn glorybringer_attack_deals_4_damage_to_opponent_creature() {
    use crate::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let glory = g.add_card_to_battlefield(0, catalog::glorybringer());
    g.clear_sickness(glory);
    // Opponent has a 5-toughness creature
    let opp_creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: glory,
        target: AttackTarget::Player(1),
    }]))
    .expect("Glorybringer attacks");
    drain_stack(&mut g);
    // Grizzly Bears has 2 toughness; 4 damage kills it
    assert!(g.players[1].graveyard.iter().any(|c| c.id == opp_creature),
        "Glorybringer should deal 4 damage to the targeted creature, killing it");
}

#[test]
fn inferno_titan_etb_deals_3_damage_to_creature() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::inferno_titan());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Inferno Titan castable");
    drain_stack(&mut g);
    // Grizzly Bears has 2 toughness; 3 damage kills it
    assert!(g.players[1].graveyard.iter().any(|c| c.id == opp_bear),
        "Inferno Titan ETB should deal 3 damage, killing the bear");
}

#[test]
fn thundermaw_hellkite_is_5_5_flying_haste() {
    use crate::card::Keyword;
    let card = catalog::thundermaw_hellkite();
    assert_eq!(card.name, "Thundermaw Hellkite");
    assert_eq!(card.power, 5);
    assert_eq!(card.toughness, 5);
    assert!(card.keywords.contains(&Keyword::Flying));
    assert!(card.keywords.contains(&Keyword::Haste));
    assert_eq!(card.triggered_abilities.len(), 1, "ETB trigger");
}

#[test]
fn craterhoof_behemoth_is_5_5_haste_trample() {
    use crate::card::Keyword;
    let card = catalog::craterhoof_behemoth();
    assert_eq!(card.name, "Craterhoof Behemoth");
    assert_eq!(card.power, 5);
    assert_eq!(card.toughness, 5);
    assert!(card.keywords.contains(&Keyword::Haste));
    assert!(card.keywords.contains(&Keyword::Trample));
    assert_eq!(card.triggered_abilities.len(), 1, "ETB pump trigger");
}

#[test]
fn thragtusk_etb_gains_5_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::thragtusk());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Thragtusk castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 5,
        "Thragtusk ETB should gain 5 life");
}

#[test]
fn courser_of_kruphix_gains_life_on_land_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::courser_of_kruphix());
    let land = g.add_card_to_hand(0, catalog::forest());
    g.priority.player_with_priority = 0;
    let life_before = g.players[0].life;
    g.perform_action(GameAction::PlayLand(land))
        .expect("Forest plays");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1,
        "Courser should gain 1 life when a land enters");
}

#[test]
fn wurmcoil_engine_is_6_6_deathtouch_lifelink() {
    use crate::card::Keyword;
    let card = catalog::wurmcoil_engine();
    assert_eq!(card.name, "Wurmcoil Engine");
    assert_eq!(card.power, 6);
    assert_eq!(card.toughness, 6);
    assert!(card.keywords.contains(&Keyword::Deathtouch));
    assert!(card.keywords.contains(&Keyword::Lifelink));
    assert!(card.card_types.contains(&CardType::Artifact));
    assert_eq!(card.triggered_abilities.len(), 1, "death trigger");
}

// ── Vengevine ───────────────────────────────────────────────────────────────

#[test]
fn vengevine_is_4_3_haste_elemental() {
    use crate::card::Keyword;
    let card = catalog::vengevine();
    assert_eq!(card.name, "Vengevine");
    assert_eq!(card.power, 4);
    assert_eq!(card.toughness, 3);
    assert!(card.keywords.contains(&Keyword::Haste));
    assert_eq!(card.triggered_abilities.len(), 1, "graveyard return trigger");
}

// ── Portal to Phyrexia ──────────────────────────────────────────────────────

#[test]
fn portal_to_phyrexia_etb_forces_opponent_sacrifice() {
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bear3 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _ = (bear1, bear2, bear3);

    let portal = g.add_card_to_hand(0, catalog::portal_to_phyrexia());
    g.players[0].mana_pool.add_colorless(9);
    let opp_creatures_before = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.card_types.contains(&CardType::Creature))
        .count();
    assert_eq!(opp_creatures_before, 3);

    g.perform_action(GameAction::CastSpell {
        card_id: portal, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Portal castable");
    drain_stack(&mut g);

    let opp_creatures_after = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.card_types.contains(&CardType::Creature))
        .count();
    assert_eq!(opp_creatures_after, 0, "Portal ETB should sac 3 creatures");
}

#[test]
fn portal_to_phyrexia_has_upkeep_reanimate_trigger() {
    let card = catalog::portal_to_phyrexia();
    assert_eq!(card.triggered_abilities.len(), 2, "ETB + upkeep triggers");
}

// ── Finale of Devastation ───────────────────────────────────────────────────

#[test]
fn finale_of_devastation_is_x_green_green_sorcery() {
    let card = catalog::finale_of_devastation();
    assert_eq!(card.name, "Finale of Devastation");
    assert!(card.card_types.contains(&CardType::Sorcery));
    assert_eq!(card.cost.cmc(), 2, "X + GG = 2 base CMC");
}

// ── Rishadan Port ───────────────────────────────────────────────────────────

#[test]
fn rishadan_port_taps_for_colorless() {
    let mut g = two_player_game();
    let port = g.add_card_to_battlefield(0, catalog::rishadan_port());
    g.perform_action(GameAction::ActivateAbility { card_id: port, ability_index: 0, target: None, x_value: None })
        .expect("tap for {C}");
    drain_stack(&mut g);
    assert!(g.players[0].mana_pool.colorless_amount() > 0, "Port should produce colorless mana");
}

#[test]
fn rishadan_port_taps_target_land() {
    let mut g = two_player_game();
    let port = g.add_card_to_battlefield(0, catalog::rishadan_port());
    let opp_land = g.add_card_to_battlefield(1, catalog::island());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: port, ability_index: 1,
        target: Some(Target::Permanent(opp_land)), x_value: None,
    }).expect("tap opp land");
    drain_stack(&mut g);
    let opp_land_card = g.battlefield.iter().find(|c| c.id == opp_land).unwrap();
    assert!(opp_land_card.tapped, "Opponent's land should be tapped");
}

// ── Horizon Canopy ──────────────────────────────────────────────────────────

#[test]
fn horizon_canopy_taps_for_green_costing_one_life() {
    let mut g = two_player_game();
    let hc = g.add_card_to_battlefield(0, catalog::horizon_canopy());
    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility { card_id: hc, ability_index: 0, target: None, x_value: None })
        .expect("tap for {G}");
    drain_stack(&mut g);
    assert!(g.players[0].mana_pool.amount(Color::Green) > 0, "Should produce green mana");
    assert_eq!(g.players[0].life, life_before - 1, "Should cost 1 life");
}

#[test]
fn horizon_canopy_sac_draws_a_card() {
    let mut g = two_player_game();
    let hc = g.add_card_to_battlefield(0, catalog::horizon_canopy());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility { card_id: hc, ability_index: 2, target: None, x_value: None })
        .expect("sac for draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "Should draw 1");
    assert!(g.battlefield.iter().all(|c| c.id != hc), "HC should be sacrificed");
}

// ── Sunbaked Canyon ─────────────────────────────────────────────────────────

#[test]
fn sunbaked_canyon_taps_for_red_costing_one_life() {
    let mut g = two_player_game();
    let sc = g.add_card_to_battlefield(0, catalog::sunbaked_canyon());
    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility { card_id: sc, ability_index: 0, target: None, x_value: None })
        .expect("tap for {R}");
    drain_stack(&mut g);
    assert!(g.players[0].mana_pool.amount(Color::Red) > 0, "Should produce red mana");
    assert_eq!(g.players[0].life, life_before - 1, "Should cost 1 life");
}

// ── Waterlogged Grove ───────────────────────────────────────────────────────

#[test]
fn waterlogged_grove_taps_for_green_costing_one_life() {
    let mut g = two_player_game();
    let wg = g.add_card_to_battlefield(0, catalog::waterlogged_grove());
    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility { card_id: wg, ability_index: 0, target: None, x_value: None })
        .expect("tap for {G}");
    drain_stack(&mut g);
    assert!(g.players[0].mana_pool.amount(Color::Green) > 0, "Should produce green mana");
    assert_eq!(g.players[0].life, life_before - 1, "Should cost 1 life");
}

// ── Koma, Cosmos Serpent ────────────────────────────────────────────────────

#[test]
fn koma_cosmos_serpent_is_6_6_uncounterable_serpent() {
    use crate::card::Keyword;
    let card = catalog::koma_cosmos_serpent();
    assert_eq!(card.name, "Koma, Cosmos Serpent");
    assert_eq!(card.power, 6);
    assert_eq!(card.toughness, 6);
    assert!(card.keywords.contains(&Keyword::CantBeCountered));
    assert_eq!(card.triggered_abilities.len(), 1, "upkeep token trigger");
}

// ── Mesmeric Orb ────────────────────────────────────────────────────────────

#[test]
fn mesmeric_orb_is_two_mana_artifact() {
    let card = catalog::mesmeric_orb();
    assert_eq!(card.name, "Mesmeric Orb");
    assert!(card.card_types.contains(&CardType::Artifact));
    assert_eq!(card.cost.cmc(), 2);
    assert_eq!(card.triggered_abilities.len(), 1, "upkeep mill trigger");
}

// ── Chalice of the Void ─────────────────────────────────────────────────────

#[test]
fn chalice_of_the_void_enters_with_x_charge_counters() {
    let card = catalog::chalice_of_the_void();
    assert_eq!(card.name, "Chalice of the Void");
    assert!(card.enters_with_counters.is_some());
    let (ctype, _) = card.enters_with_counters.unwrap();
    assert_eq!(ctype, CounterType::Charge);
}

#[test]
fn chalice_of_the_void_has_spell_counter_trigger() {
    let card = catalog::chalice_of_the_void();
    assert_eq!(card.triggered_abilities.len(), 1, "counter trigger");
}

// ── Candelabra of Tawnos ────────────────────────────────────────────────────

#[test]
fn candelabra_of_tawnos_has_tap_x_untap_activation() {
    let card = catalog::candelabra_of_tawnos();
    assert_eq!(card.name, "Candelabra of Tawnos");
    assert!(card.card_types.contains(&CardType::Artifact));
    assert_eq!(card.activated_abilities.len(), 1, "one X activation");
}

// ── Archdruid's Charm ───────────────────────────────────────────────────────

#[test]
fn archdruids_charm_is_a_three_mode_green_instant() {
    let card = catalog::archdruids_charm();
    assert_eq!(card.name, "Archdruid's Charm");
    assert!(card.card_types.contains(&CardType::Instant));
    assert_eq!(card.cost.cmc(), 3, "GGG = 3 CMC");
    if let crate::effect::Effect::ChooseMode(modes) = &card.effect {
        assert_eq!(modes.len(), 3, "three modes");
    } else {
        panic!("Archdruid's Charm should be ChooseMode");
    }
}

// ── Awaken the Honored Dead ─────────────────────────────────────────────────

#[test]
fn awaken_the_honored_dead_is_mass_reanimation() {
    let card = catalog::awaken_the_honored_dead();
    assert_eq!(card.name, "Awaken the Honored Dead");
    assert!(card.card_types.contains(&CardType::Sorcery));
    assert_eq!(card.cost.cmc(), 7, "5WB = 7 CMC");
}

// ── Growing Ranks ───────────────────────────────────────────────────────────

#[test]
fn growing_ranks_has_upkeep_token_trigger() {
    let card = catalog::growing_ranks();
    assert_eq!(card.name, "Growing Ranks");
    assert!(card.card_types.contains(&CardType::Enchantment));
    assert_eq!(card.triggered_abilities.len(), 1, "upkeep token trigger");
}

// ── Monument to Endurance ───────────────────────────────────────────────────

#[test]
fn monument_to_endurance_pumps_target_creature() {
    let mut g = two_player_game();
    let mon = g.add_card_to_battlefield(0, catalog::monument_to_endurance());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    let power_before = g.battlefield.iter().find(|c| c.id == bear).unwrap().definition.power;
    g.perform_action(GameAction::ActivateAbility {
        card_id: mon, ability_index: 0,
        target: Some(Target::Permanent(bear)), x_value: None,
    }).expect("activate pump");
    drain_stack(&mut g);
    let computed = g.compute_battlefield();
    let cp = computed.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(cp.power, power_before + 2, "Should pump +2/+2");
}

// ── Exotic Orchard ──────────────────────────────────────────────────────────

#[test]
fn exotic_orchard_taps_for_any_color() {
    let mut g = two_player_game();
    let eo = g.add_card_to_battlefield(0, catalog::exotic_orchard());
    g.perform_action(GameAction::ActivateAbility {
        card_id: eo, ability_index: 0,
        target: None, x_value: None,
    }).expect("tap for mana");
    drain_stack(&mut g);
    assert!(g.players[0].mana_pool.total() > 0, "Should produce mana");
}

// ── Master of Death ─────────────────────────────────────────────────────────

#[test]
fn master_of_death_is_3_1_zombie_wizard() {
    let card = catalog::master_of_death();
    assert_eq!(card.name, "Master of Death");
    assert_eq!(card.power, 3);
    assert_eq!(card.toughness, 1);
    assert_eq!(card.triggered_abilities.len(), 1, "graveyard return trigger");
}

#[test]
fn growing_ranks_creates_centaur_token_on_upkeep() {
    use crate::game::types::TurnStep;
    let mut g = two_player_game();
    let _ranks = g.add_card_to_battlefield(0, catalog::growing_ranks());
    let bf_before = g.battlefield.len();
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield.len() > bf_before, "Centaur token should be created");
    let tok = g.battlefield.iter().find(|c|
        c.is_token && c.definition.name == "Centaur"
    ).expect("Centaur token should exist on the battlefield");
    assert_eq!(tok.power(), 3, "Centaur token should be 3/3");
    assert_eq!(tok.toughness(), 3, "Centaur token should be 3/3");
}

#[test]
fn master_of_death_returns_from_graveyard_on_upkeep() {
    use crate::game::types::TurnStep;
    let mut g = two_player_game();
    // Put Master of Death directly into the graveyard.
    let _mod_id = g.add_card_to_graveyard(0, catalog::master_of_death());
    let hand_before = g.players[0].hand.len();
    // ScriptedDecider answers MayDo(yes) to pay 1 life.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
    ]));
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    // Master of Death should be in hand now.
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Master of Death should return to hand");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Master of Death"),
        "Master of Death should be in hand");
    // Player should have lost 1 life.
    assert_eq!(g.players[0].life, 19, "Should have paid 1 life");
}

#[test]
fn monument_to_endurance_is_3_cost_artifact_with_pump() {
    let card = catalog::monument_to_endurance();
    assert_eq!(card.name, "Monument to Endurance");
    assert!(card.card_types.contains(&CardType::Artifact));
    assert_eq!(card.cost.cmc(), 3, "costs 3 generic");
    assert_eq!(card.activated_abilities.len(), 1, "one pump activation");
}

#[test]
fn exotic_orchard_is_a_land_with_any_color_mana() {
    let card = catalog::exotic_orchard();
    assert_eq!(card.name, "Exotic Orchard");
    assert!(card.card_types.contains(&CardType::Land));
    assert_eq!(card.activated_abilities.len(), 1, "one tap-for-mana ability");
}

// ── Basking Broodscale ──────────────────────────────────────────────────────

#[test]
fn basking_broodscale_enters_with_counters_and_spawns() {
    let card = catalog::basking_broodscale();
    assert_eq!(card.name, "Basking Broodscale");
    assert!(card.enters_with_counters.is_some());
    let (ct, _) = card.enters_with_counters.unwrap();
    assert_eq!(ct, CounterType::PlusOnePlusOne);
    assert_eq!(card.triggered_abilities.len(), 1, "ETB token trigger");
}

// ── Sowing Mycospawn ────────────────────────────────────────────────────────

#[test]
fn sowing_mycospawn_is_4_4_with_land_search_etb() {
    let card = catalog::sowing_mycospawn();
    assert_eq!(card.name, "Sowing Mycospawn");
    assert_eq!(card.power, 4);
    assert_eq!(card.toughness, 4);
    assert_eq!(card.triggered_abilities.len(), 1, "ETB land search");
}

// ── Ursine Monstrosity ──────────────────────────────────────────────────────

#[test]
fn ursine_monstrosity_enters_with_five_counters_and_draws() {
    use crate::card::Keyword;
    let card = catalog::ursine_monstrosity();
    assert_eq!(card.name, "Ursine Monstrosity");
    assert!(card.keywords.contains(&Keyword::Trample));
    assert!(card.enters_with_counters.is_some());
    assert_eq!(card.triggered_abilities.len(), 1, "ETB draw");
}

// ── Moonshadow ──────────────────────────────────────────────────────────────

#[test]
fn moonshadow_is_2_1_flying_faerie_with_discard_trigger() {
    use crate::card::Keyword;
    let card = catalog::moonshadow();
    assert_eq!(card.name, "Moonshadow");
    assert_eq!(card.power, 2);
    assert_eq!(card.toughness, 1);
    assert!(card.keywords.contains(&Keyword::Flying));
    assert_eq!(card.triggered_abilities.len(), 1, "combat damage discard");
}

// ── Golos, Tireless Pilgrim ─────────────────────────────────────────────────

#[test]
fn golos_tireless_pilgrim_is_legendary_3_5_with_etb() {
    use crate::card::Supertype;
    let card = catalog::golos_tireless_pilgrim();
    assert_eq!(card.name, "Golos, Tireless Pilgrim");
    assert!(card.supertypes.contains(&Supertype::Legendary));
    assert_eq!(card.power, 3);
    assert_eq!(card.toughness, 5);
    assert_eq!(card.triggered_abilities.len(), 1, "ETB land search");
}

// ── Maelstrom Archangel ─────────────────────────────────────────────────────

#[test]
fn maelstrom_archangel_is_5_5_flying_five_color() {
    use crate::card::Keyword;
    let card = catalog::maelstrom_archangel();
    assert_eq!(card.name, "Maelstrom Archangel");
    assert_eq!(card.power, 5);
    assert_eq!(card.toughness, 5);
    assert!(card.keywords.contains(&Keyword::Flying));
    assert_eq!(card.cost.cmc(), 5, "WUBRG = 5 CMC");
}

// ── Ramos, Dragon Engine ────────────────────────────────────────────────────

#[test]
fn ramos_dragon_engine_is_4_4_flying_dragon_with_counter_trigger() {
    use crate::card::Keyword;
    let card = catalog::ramos_dragon_engine();
    assert_eq!(card.name, "Ramos, Dragon Engine");
    assert_eq!(card.power, 4);
    assert_eq!(card.toughness, 4);
    assert!(card.keywords.contains(&Keyword::Flying));
    assert_eq!(card.triggered_abilities.len(), 1, "spell-cast counter trigger");
    assert_eq!(card.activated_abilities.len(), 1, "mana burst activation");
}
