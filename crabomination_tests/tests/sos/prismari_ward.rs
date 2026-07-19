#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use crate::prepared_on_battlefield;
use crate::push15_17::place_creature;

// ── Prismari ⏳ closer + Ward-tagged MDFCs + ⏳ utility ─────────────────────

#[test]
fn colorstorm_stallion_is_three_three_ward_one_haste_elemental_horse() {
    use crabomination::card::CreatureType;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::colorstorm_stallion());
    let c = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!(c.power(), 3);
    assert_eq!(c.toughness(), 3);
    assert!(c.has_keyword(&Keyword::Haste));
    assert!(c.has_keyword(&Keyword::Ward(crabomination::card::WardCost::generic(1))));
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Elemental));
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Horse));
}

#[test]
fn elemental_mascot_is_one_four_flying_vigilance_elemental_bird() {
    use crabomination::card::CreatureType;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::elemental_mascot());
    let c = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!(c.power(), 1);
    assert_eq!(c.toughness(), 4);
    assert!(c.has_keyword(&Keyword::Flying));
    assert!(c.has_keyword(&Keyword::Vigilance));
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Elemental));
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Bird));
}

#[test]
fn prismari_the_inspiration_is_seven_seven_legendary_dragon_with_ward_five() {
    use crabomination::card::{CreatureType, Supertype};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::prismari_the_inspiration());
    let c = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!(c.power(), 7);
    assert_eq!(c.toughness(), 7);
    assert!(c.has_keyword(&Keyword::Flying));
    assert!(c.has_keyword(&Keyword::Ward(crabomination::card::WardCost::Life(5))));
    assert!(c.definition.supertypes.contains(&Supertype::Legendary));
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Dragon));
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Elder));
}

#[test]
fn campus_composer_is_three_four_ward_two_merfolk_bard() {
    use crabomination::card::CreatureType;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::campus_composer());
    let c = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!(c.power(), 3);
    assert_eq!(c.toughness(), 4);
    assert!(c.has_keyword(&Keyword::Ward(crabomination::card::WardCost::generic(2))));
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Merfolk));
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Bard));
    // Inset prepare spell: Aqueous Aria.
    let prep = c.definition.prepare_spell.as_ref().expect("prepare spell");
    assert_eq!(prep.name, "Aqueous Aria");
}

// Aqueous Aria — create a 3/3 blue-and-red Elemental token with flying.
#[test]
fn campus_composer_aqueous_aria_creates_elemental_token() {
    let mut g = two_player_game();
    let id = prepared_on_battlefield(&mut g, 0, catalog::campus_composer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Aqueous Aria castable for {4}{U}");
    drain_stack(&mut g);

    assert_eq!(g.battlefield.len(), bf_before + 1, "one token minted");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Elemental"
            && c.controller == 0),
        "Aqueous Aria mints an Elemental token");
}

#[test]
fn emeritus_of_ideation_prepare_spell_draws_three() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let id = prepared_on_battlefield(&mut g, 0, catalog::emeritus_of_ideation());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();
    let lib_before = g.players[0].library.len();

    // Cast the inset Ancestral Recall — costs {U}. Target self to draw 3.
    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Ancestral Recall castable for {U}");
    drain_stack(&mut g);

    // The copy never lived in hand: +3 hand. Library -3.
    assert_eq!(g.players[0].hand.len(), hand_before + 3);
    assert_eq!(g.players[0].library.len(), lib_before - 3);
}

// Ancestral Recall draws for the *targeted* player, not the caster.
// Aiming at the opponent makes them draw 3 (rarely the right play, but
// exercises the target_filtered(Player) wiring).
#[test]
fn emeritus_of_ideation_ancestral_recall_targets_opponent() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(1, catalog::island()); }
    let id = prepared_on_battlefield(&mut g, 0, catalog::emeritus_of_ideation());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let opp_hand_before = g.players[1].hand.len();
    let opp_lib_before = g.players[1].library.len();
    let caster_hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Ancestral Recall castable for {U}");
    drain_stack(&mut g);

    // Opp drew 3; caster's hand untouched (the copy never lived there).
    assert_eq!(g.players[1].hand.len(), opp_hand_before + 3);
    assert_eq!(g.players[1].library.len(), opp_lib_before - 3);
    assert_eq!(g.players[0].hand.len(), caster_hand_before);
}

#[test]
fn grave_researcher_front_etb_surveils_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::grave_researcher());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let lib_before = g.players[0].library.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Grave Researcher castable for {2}{B}");
    drain_stack(&mut g);

    // ETB Surveil 1: top card either stays or hits graveyard.
    let after_lib = g.players[0].library.len();
    let after_gy = g.players[0].graveyard.len();
    assert!(
        after_lib == lib_before || (after_lib == lib_before - 1 && after_gy >= 1),
        "Surveil 1 either kept or graveyarded the top card",
    );
}

#[test]
fn grave_researcher_prepare_spell_reanimates_creature_from_graveyard() {
    let mut g = two_player_game();
    // Seed a creature in P0's graveyard.
    let bear_id = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let researcher = prepared_on_battlefield(&mut g, 0, catalog::grave_researcher());
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;

    // Cast the inset Reanimate for {B}, targeting the bear.
    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: researcher,
        target: Some(Target::Permanent(bear_id)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Reanimate castable for {B}");
    drain_stack(&mut g);

    // Bear should now be on P0's battlefield.
    assert!(
        g.battlefield.iter().any(|c| c.id == bear_id),
        "Grizzly Bears returned to battlefield via Reanimate"
    );
    assert!(
        !g.players[0].graveyard.iter().any(|c| c.id == bear_id),
        "Bear left the graveyard"
    );
    // Real Reanimate: lose life equal to the creature's CMC.
    // Grizzly Bears is {1}{G} = CMC 2 → P0 loses 2 life.
    assert_eq!(g.players[0].life, life_before - 2);
}

#[test]
fn strife_scholar_is_three_two_ward_pay_two_life_orc_sorcerer() {
    use crabomination::card::CreatureType;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::strife_scholar());
    let c = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!(c.power(), 3);
    assert_eq!(c.toughness(), 2);
    assert!(c.has_keyword(&Keyword::Ward(crabomination::card::WardCost::Life(2))),
        "Strife Scholar has Ward—Pay 2 life");
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Orc));
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Sorcerer));
}

#[test]
fn strife_scholar_prepare_spell_creates_two_spirits() {
    let mut g = two_player_game();
    let id = prepared_on_battlefield(&mut g, 0, catalog::strife_scholar());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Awaken the Ages castable for {5}{R}");
    drain_stack(&mut g);

    assert_eq!(g.battlefield.len(), bf_before + 2, "two tokens minted");
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Spirit" && c.controller == 0)
        .collect();
    assert_eq!(spirits.len(), 2, "Awaken the Ages mints two Spirit tokens");
}

#[test]
fn awaken_the_ages_copy_ceases_to_exist_after_resolve() {
    // The cast prepare-spell copy never lingers in any zone after it
    // resolves (CR 707.10a): not graveyard, not exile, not hand. The
    // old self-exile rider is gone — it just makes the Spirits.
    let mut g = two_player_game();
    let id = prepared_on_battlefield(&mut g, 0, catalog::strife_scholar());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    let exile_before = g.exile.len();

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Awaken the Ages castable for {5}{R}");
    drain_stack(&mut g);

    // Two Spirits arrived; the copy itself exists in no zone.
    assert_eq!(g.battlefield.iter()
        .filter(|c| c.definition.name == "Spirit").count(), 2);
    let in_some_zone = g.exile.iter().any(|c| c.definition.name == "Awaken the Ages")
        || g.players[0].graveyard.iter().any(|c| c.definition.name == "Awaken the Ages")
        || g.players[0].hand.iter().any(|c| c.definition.name == "Awaken the Ages")
        || g.players[0].library.iter().any(|c| c.definition.name == "Awaken the Ages")
        || g.battlefield.iter().any(|c| c.definition.name == "Awaken the Ages");
    assert!(!in_some_zone, "the cast copy ceases to exist after resolution");
    assert_eq!(g.exile.len(), exile_before, "no self-exile rider anymore");
}

#[test]
fn strixhaven_skycoach_etb_searches_for_a_basic_land() {
    use crabomination::card::ArtifactSubtype;
    let mut g = two_player_game();
    // Seed a Forest into P0's library to tutor for.
    let forest_id = g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::strixhaven_skycoach());
    g.players[0].mana_pool.add_colorless(3);
    // Script the Search decision to actually pick the Forest.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest_id)),
    ]));

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Skycoach castable for {3}");
    drain_stack(&mut g);

    // Forest tutored to hand by the Skycoach ETB.
    assert!(g.players[0].hand.iter().any(|c| c.id == forest_id),
        "Forest tutored to hand by Skycoach ETB");
    // Skycoach is on battlefield with Vehicle subtype.
    let sc = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert!(sc.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Vehicle));
}

#[test]
fn choreographed_sparks_copies_target_instant_you_control() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    // Cast a Bolt first to put it on the stack.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let sparks = g.add_card_to_hand(0, catalog::choreographed_sparks());
    g.players[0].mana_pool.add(Color::Red, 3);

    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    // Bolt is on the stack now. Cast Sparks targeting the Bolt stack item by
    // its original CardId (the engine uses Target::Permanent(card_id) for
    // stack targets, see Test of Talents). Mode 0 = IS-spell copy.
    g.perform_action(GameAction::CastSpell {
        card_id: sparks, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: Some(0), x_value: None,
    })
    .expect("Choreographed Sparks castable for {R}{R}");
    drain_stack(&mut g);

    // Both the original Bolt and the copy hit the bear → 6 damage total →
    // bear dies (2 toughness).
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should die to original Bolt + Sparks copy (6 total damage)");
}

#[test]
fn choreographed_sparks_cant_be_copied() {
    // "This spell can't be copied." A second Choreographed Sparks aimed at
    // the first is skipped by the CopySpell resolver, so only the first
    // Sparks copies the Bolt: original Bolt + one copy = 6 damage to the
    // opponent (a successful copy of the first Sparks would add a third
    // Bolt for 9).
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let sparks1 = g.add_card_to_hand(0, catalog::choreographed_sparks());
    let sparks2 = g.add_card_to_hand(0, catalog::choreographed_sparks());
    g.players[0].mana_pool.add(Color::Red, 5);

    // Bolt → P1, on the stack.
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable for {R}");
    // Sparks #1 copies the Bolt (mode 0).
    g.perform_action(GameAction::CastSpell {
        card_id: sparks1, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Sparks #1 castable");
    // Sparks #2 tries to copy Sparks #1 (an instant on the stack).
    g.perform_action(GameAction::CastSpell {
        card_id: sparks2, target: Some(Target::Permanent(sparks1)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Sparks #2 castable");
    drain_stack(&mut g);

    assert_eq!(
        g.players[1].life, 14,
        "Sparks #1 wasn't copied: original Bolt + one copy = 6 damage (not 9)",
    );
}

#[test]
fn choreographed_sparks_mode_one_copies_target_creature_spell() {
    // Push (modern_decks): mode 1 copies a creature spell on the stack;
    // the copy resolves as a token (CR 608.3f), so two bears land
    // simultaneously.
    let mut g = two_player_game();
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    let sparks = g.add_card_to_hand(0, catalog::choreographed_sparks());
    // Bears: {1}{G}; Sparks: {R}{R}; total {1}{G}{R}{R}.
    g.players[0].mana_pool.add(Color::Red, 5);
    g.players[0].mana_pool.add(Color::Green, 5);
    g.players[0].mana_pool.add_colorless(5);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None,
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Grizzly Bears castable for {1}{G}");
    // Now cast Choreographed Sparks targeting the bear spell with
    // mode 1 (creature spell copy).
    g.perform_action(GameAction::CastSpell {
        card_id: sparks, target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: Some(1), x_value: None,
    })
    .expect("Choreographed Sparks castable for {R}{R} on creature spell");
    drain_stack(&mut g);

    // Original bears + token copy = 2 new permanents.
    let new_bears: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Grizzly Bears")
        .collect();
    assert!(new_bears.len() >= 2, "Original + copy should both be on battlefield");
    assert!(new_bears.iter().any(|c| c.is_token),
        "The copy should be a token");
    assert_eq!(g.battlefield.len(), bf_before + 2,
        "Bf grew by 2 (original + token copy)");

    // Printed riders (`Effect::CopySpellWithRiders`): the copy has haste,
    // the original doesn't.
    let copy_id = g.battlefield.iter()
        .find(|c| c.definition.name == "Grizzly Bears" && c.is_token)
        .map(|c| c.id)
        .expect("token copy on battlefield");
    let copy = g.computed_permanent(copy_id).unwrap();
    assert!(copy.keywords.contains(&Keyword::Haste),
        "the copy gains haste");
    let orig = g.computed_permanent(bears).unwrap();
    assert!(!orig.keywords.contains(&Keyword::Haste),
        "the original bears doesn't gain haste");

    // ...and the copy is sacrificed at the beginning of the next end step.
    g.fire_step_triggers(crabomination::game::types::TurnStep::End);
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == copy_id),
        "the copy is sacrificed at the next end step");
    assert!(g.battlefield.iter().any(|c| c.id == bears),
        "the original bears survives the end step");
}

#[test]
fn flashback_instant_grants_flashback_on_gy_is_card() {
    // The SOS "Flashback" instant grants until-end-of-turn flashback
    // (cost = the card's own mana cost) to a target IS card in your
    // graveyard. The card is then recastable via the normal flashback
    // path — paying its mana cost — and is exiled on resolve.
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let fb = g.add_card_to_hand(0, catalog::sos_flashback_instant());
    g.players[0].mana_pool.add(Color::Red, 1);

    // "Target instant or sorcery card in your graveyard" — the card is a
    // real target now, so the caster picks it explicitly.
    g.perform_action(GameAction::CastSpell {
        card_id: fb, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Flashback (instant) castable for {R}");
    drain_stack(&mut g);

    // Bolt stays in gy, now carrying an EOT flashback grant = its own cost.
    let bolt_gy = g.players[0].graveyard.iter().find(|c| c.id == bolt)
        .expect("Bolt still in graveyard");
    assert_eq!(
        bolt_gy.granted_flashback_eot.as_ref().map(|c| c.summary()),
        Some(catalog::lightning_bolt().cost.summary()),
        "flashback cost equals Bolt's own mana cost",
    );

    // Recast Bolt from the graveyard via flashback, paying {R}, at player 1.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastFlashback {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt recastable via granted flashback for {R}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, 17, "recast Bolt dealt 3 to player 1");
    assert!(
        g.exile.iter().any(|c| c.id == bolt),
        "flashback cast exiles Bolt on resolve (CR 702.34d)",
    );
    assert!(
        !g.players[0].graveyard.iter().any(|c| c.id == bolt),
        "Bolt left the graveyard",
    );
}

#[test]
fn granted_flashback_expires_at_end_of_turn() {
    // The grant is "until end of turn" — a graveyard card's
    // `granted_flashback_eot` is cleared at cleanup.
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.players[0]
        .graveyard
        .iter_mut()
        .find(|c| c.id == bolt)
        .unwrap()
        .granted_flashback_eot = Some(catalog::lightning_bolt().cost);

    g.do_cleanup(&mut Vec::new());

    assert!(
        g.players[0]
            .graveyard
            .iter()
            .find(|c| c.id == bolt)
            .unwrap()
            .granted_flashback_eot
            .is_none(),
        "EOT flashback grant cleared at cleanup",
    );
}

#[test]
fn echocasting_symposium_creates_a_copy_of_target_creature() {
    use crabomination::game::types::Target;
    // Push (modern_decks, batch 81): Echocasting Symposium now uses
    // CreateTokenCopyOf — the token inherits the target creature's
    // printed name + types + P/T (Grizzly Bears here, so a 2/2 Bear
    // token enters).
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::echocasting_symposium());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        // Slot 0: the creature to copy; slot 1: the player who creates.
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Player(0)],
        mode: None,
        x_value: None,
    })
    .expect("Echocasting Symposium castable for {4}{U}{U}");
    drain_stack(&mut g);

    assert_eq!(g.battlefield.len(), bf_before + 1, "One new token entered");
    let tok = g.battlefield.iter().find(|c|
        c.is_token && c.definition.name == "Grizzly Bears"
    ).expect("token is a copy of Grizzly Bears");
    assert_eq!(tok.power(), 2);
    assert_eq!(tok.toughness(), 2);
}

#[test]
fn applied_geometry_mints_a_six_six_fractal() {
    use crabomination::card::CreatureType;
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    // Seed a target permanent for the copy. Applied Geometry's printed
    // body copies a non-Aura permanent you control — the token inherits
    // the source's name + types + abilities, with P/T overridden to 0/0
    // and Fractal added to its creature types. Six +1/+1 counters
    // then ride on the token = a 6/6 Fractal-plus-bear.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::applied_geometry());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Applied Geometry castable for {2}{G}{U}");
    drain_stack(&mut g);

    // Find the freshly-minted token (is_token = true).
    let frac = g.battlefield.iter().find(|c|
        c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal)
    ).expect("Applied Geometry mints a Fractal-typed copy token");
    // 0/0 base override + 6 +1/+1 counters = 6/6.
    assert_eq!(frac.power(), 6, "token should be 6/6 from counters");
    assert_eq!(frac.toughness(), 6);
    assert!(
        frac.definition.subtypes.creature_types.contains(&CreatureType::Fractal),
        "token has Fractal type added",
    );
}

// ── Prismari Opus rider promotions ──────────────────────────────────────────
//
// Spectacular Skywhale fully wires its Opus rider (small: +3/+0 EOT;
// big: 3 +1/+1 counters instead). Colorstorm Stallion (copy-token) and
// Elemental Mascot (exile-top + may-play) now wire their big-body
// conditional clauses too via CreateTokenCopyOf / ExileTopAndGrantMayPlay
// — see the dedicated `*_opus_*` tests later in this file.

#[test]
fn spectacular_skywhale_opus_small_body_pumps_three_zero_eot() {
    // Cast Bolt ({R} = 1 mana). Small body fires: +3/+0 EOT → 4/4 power-only.
    let mut g = two_player_game();
    let sw = place_creature(&mut g, 0, catalog::spectacular_skywhale());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let c = g.battlefield_find(sw).expect("Skywhale alive");
    assert_eq!(c.power(), 1 + 3, "Small body adds +3 power EOT");
    assert_eq!(c.toughness(), 4, "Small body adds +0 toughness");
}

#[test]
fn spectacular_skywhale_opus_big_body_adds_three_counters() {
    // Cast Divergent Equation with X=2 → 5 mana spent, big body fires
    // and lands three +1/+1 counters instead of the temporary pump.
    let mut g = two_player_game();
    let sw = place_creature(&mut g, 0, catalog::spectacular_skywhale());
    let big = g.add_card_to_hand(0, catalog::divergent_equation());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: big, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("Divergent Equation castable with X=2");
    drain_stack(&mut g);
    let c = g.battlefield_find(sw).expect("Skywhale alive");
    assert_eq!(
        c.counter_count(CounterType::PlusOnePlusOne),
        3,
        "Big body (≥5 mana) lands three +1/+1 counters instead of the EOT pump",
    );
}

#[test]
fn colorstorm_stallion_opus_small_body_pumps_one_one_eot() {
    // Cast Bolt ({R}, 1 mana). Small body fires: +1/+1 EOT.
    let mut g = two_player_game();
    let cs = place_creature(&mut g, 0, catalog::colorstorm_stallion());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let c = g.battlefield_find(cs).expect("Stallion alive");
    assert_eq!(c.power(), 4, "Small body adds +1 power EOT (3 → 4)");
    assert_eq!(c.toughness(), 4, "Small body adds +1 toughness EOT");
}

// ── CR 506.4 — Removed from combat on zone change ───────────────────────────
//
// "A permanent is removed from combat if it leaves the battlefield."
// When an attacker is destroyed mid-combat, the engine must prune
// `self.attacking` so downstream consumers see consistent state.

#[test]
fn destroying_attacker_mid_combat_prunes_attacking_per_cr_506_4() {
    use crabomination::game::types::AttackTarget;
    let mut g = two_player_game();
    let bear = place_creature(&mut g, 0, catalog::grizzly_bears());

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("Bear can attack");
    drain_stack(&mut g);
    assert_eq!(g.attacking().len(), 1, "Bear is the lone attacker");

    // Destroy the bear mid-combat via Lightning Bolt at instant speed.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    // CR 506.4: the bear leaves the battlefield → it's removed from combat.
    // The `self.attacking` vector should no longer carry the bear's entry.
    assert!(
        g.attacking().iter().all(|a| a.attacker != bear),
        "CR 506.4: attacker removed from combat on zone change",
    );
    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "Bear is off the battlefield (destroyed by Bolt)",
    );
}

#[test]
fn elemental_mascot_opus_small_body_pumps_one_zero_eot() {
    let mut g = two_player_game();
    let em = place_creature(&mut g, 0, catalog::elemental_mascot());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let c = g.battlefield_find(em).expect("Mascot alive");
    assert_eq!(c.power(), 2, "Small body adds +1 power EOT (1 → 2)");
    assert_eq!(c.toughness(), 4, "Toughness unchanged (+0)");
}

// ── Push XXXVIII: Increment / CounterAdded promotions ──────────────────────

/// Pensive Professor's secondary rider: "Whenever one or more +1/+1
/// counters are put on this creature, you may draw a card." Cast a
/// 2-mana spell with the 0/2 Professor on the battlefield — Increment
/// drops a +1/+1 counter (mana_spent 2 > 0 power) → CounterAdded
/// trigger fires → controller draws (scripted Yes on the `may`).
#[test]
fn pensive_professor_secondary_counter_trigger_draws_a_card() {
    let mut g = two_player_game();
    // Pre-install a scripted decider that says Yes to the "may draw" prompt.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    let prof = place_creature(&mut g, 0, catalog::pensive_professor());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());

    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bears castable for {1}{G}");
    drain_stack(&mut g);

    // Increment should land a counter (2 > 0 power).
    let c = g.battlefield_find(prof).expect("Professor alive");
    assert_eq!(
        c.counter_count(CounterType::PlusOnePlusOne),
        1,
        "Increment should land a +1/+1 counter on Pensive Professor"
    );
    // hand_before counts Bears in hand; after cast Bears becomes a
    // permanent (no longer in hand). CounterAdded MayDo trigger fires
    // and draws 1 → net hand size = hand_before.
    assert_eq!(
        g.players[0].hand.len(),
        hand_before,
        "Hand should be net 0 (cast Bears -1, drew +1 from CounterAdded)"
    );
}

/// Pensive Professor's secondary rider defaults to no-draw under the
/// auto-decider (the printed "you may" makes the draw opt-in). The
/// counter still lands.
#[test]
fn pensive_professor_secondary_counter_trigger_skips_under_auto_decider() {
    let mut g = two_player_game();
    let prof = place_creature(&mut g, 0, catalog::pensive_professor());
    g.add_card_to_library(0, catalog::island());

    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bears castable for {1}{G}");
    drain_stack(&mut g);

    let c = g.battlefield_find(prof).expect("Professor alive");
    assert_eq!(
        c.counter_count(CounterType::PlusOnePlusOne),
        1,
        "Increment counter still lands without the optional draw"
    );
    // Bears moved from hand to battlefield (- 1). No draw under auto.
    assert_eq!(
        g.players[0].hand.len(),
        hand_before - 1,
        "Default auto-decider declines the optional draw"
    );
}

/// Textbook Tabulator's Increment: cast a 4-mana spell with the 0/3
/// Frog Wizard on the battlefield. mana_spent 4 > toughness 3 → +1/+1
/// counter lands.
#[test]
fn textbook_tabulator_increment_buffs_self_on_big_spell() {
    let mut g = two_player_game();
    let tab = place_creature(&mut g, 0, catalog::textbook_tabulator());
    // 7-mana spell to definitely beat the 0/3.
    let mascot = g.add_card_to_hand(0, catalog::mascot_exhibition());
    g.players[0].mana_pool.add_colorless(7);

    g.perform_action(GameAction::CastSpell {
        card_id: mascot, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Mascot Exhibition castable for {7}");
    drain_stack(&mut g);

    let c = g.battlefield_find(tab).expect("Tabulator alive");
    assert_eq!(
        c.counter_count(CounterType::PlusOnePlusOne),
        1,
        "Increment should fire on 7-mana spell vs 0/3 Tabulator"
    );
}

/// Potioner's Trove — `{T}: You gain 2 life. Activate only if you've
/// cast an instant or sorcery spell this turn.` The conditional gate
/// rejects the activation before tap is paid when the tally is 0;
/// after casting any IS spell this turn it succeeds.
#[test]
fn potioners_trove_lifegain_requires_is_cast_this_turn() {
    let mut g = two_player_game();
    let trove = g.add_card_to_battlefield(0, catalog::potioners_trove());
    g.clear_sickness(trove);
    let life_before = g.players[0].life;

    // Ability index 1 = lifegain activation. With 0 IS casts, the
    // condition rejects.
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: trove, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None });
    assert!(err.is_err(), "Lifegain activation should be rejected without an IS cast this turn");
    assert_eq!(g.players[0].life, life_before, "No life gained on rejected activation");

    // Cast a Bolt to bump the IS-cast tally to 1.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    // Now the lifegain activation succeeds.
    g.perform_action(GameAction::ActivateAbility {
        card_id: trove, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Lifegain activation should succeed after casting an IS spell");
    drain_stack(&mut g);
    assert_eq!(
        g.players[0].life, life_before + 2,
        "Lifegain activation grants 2 life when the gate is open"
    );
}

/// Ulna Alley Shopkeep — base body without lifegain is 2/3.
#[test]
fn ulna_alley_shopkeep_no_lifegain_is_two_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::ulna_alley_shopkeep());
    let c = g.battlefield_find(id).expect("Shopkeep on battlefield");
    assert_eq!(c.power(), 2, "Base power 2 without lifegain");
    assert_eq!(c.toughness(), 3, "Base toughness 3");
    assert!(c.has_keyword(&Keyword::Menace), "Menace keyword");
}

/// Ulna Alley Shopkeep — with lifegain this turn, the Infusion +2/+0
/// rider injects via the compute-time gate, making the Shopkeep a 4/3.
#[test]
fn ulna_alley_shopkeep_with_lifegain_is_four_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::ulna_alley_shopkeep());
    // Simulate lifegain this turn by bumping the tally directly.
    g.players[0].life_gained_this_turn = 1;
    let c = g.battlefield_find(id).expect("Shopkeep on battlefield");
    let computed = g.computed_permanent(id).expect("Shopkeep computed");
    assert_eq!(
        computed.power, 4,
        "+2 from Infusion gate (2 base + 2 = 4 computed power), card.power() = {}",
        c.power()
    );
    assert_eq!(computed.toughness, 3, "Toughness unchanged (+0)");
    assert!(
        computed.keywords.contains(&Keyword::Menace),
        "Menace persists"
    );
}

// ── Transcendent Archaic — MayDo around the ETB Converge draw ──────────────
//
// Push (modern_decks): wraps the ETB Converge draw + conditional discard 2
// in `Effect::MayDo` so the printed "you may draw X cards" optionality is
// honored. AutoDecider declines by default (no draw, no discard); a
// ScriptedDecider can opt in (draw X, discard 2 if X≥1).

#[test]
fn transcendent_archaic_etb_may_draw_declines_by_default() {
    // Default AutoDecider says "no" to MayDo prompts — so the ETB draw
    // and the conditional discard 2 are both skipped. We test by placing
    // the Archaic directly onto the battlefield (firing its ETB) instead
    // of routing through the cast path (which is awkward to set up at
    // {7} mana cost).
    let mut g = two_player_game();
    // Stuff the library with 5 known cards so we'd see the draw if it ran.
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let hand_before = g.players[0].hand.len();
    let gy_before = g.players[0].graveyard.len();
    // Place Transcendent Archaic on the battlefield (triggers its ETB
    // via the universal ETB trigger fire). With no cast context the
    // ConvergedValue defaults to 0 — so even if the MayDo were accepted
    // the draw would be 0 and the discard gate would fail.
    g.add_card_to_battlefield(0, catalog::transcendent_archaic());
    drain_stack(&mut g);

    // AutoDecider declined the MayDo — no cards drawn, no discard.
    assert_eq!(
        g.players[0].hand.len(),
        hand_before,
        "AutoDecider should decline the ETB MayDo (no draw)"
    );
    assert_eq!(
        g.players[0].graveyard.len(),
        gy_before,
        "AutoDecider should decline the discard 2 follow-up"
    );
}

#[test]
fn transcendent_archaic_etb_may_draw_accepts_via_scripted_decider() {
    // ScriptedDecider says Bool(true) to the MayDo prompt. With
    // ConvergedValue = 0 (no cast context) the draw is 0 and the
    // discard branch doesn't fire (gated on ConvergedValue ≥ 1).
    let mut g = two_player_game();
    // Configure scripted decider to accept the MayDo prompt.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let hand_before = g.players[0].hand.len();
    let gy_before = g.players[0].graveyard.len();
    g.add_card_to_battlefield(0, catalog::transcendent_archaic());
    drain_stack(&mut g);

    // Even with "yes", ConvergedValue is 0, so Draw 0 happens — no
    // observable change to hand, no discard either (gate fails).
    assert_eq!(
        g.players[0].hand.len(),
        hand_before,
        "Even with MayDo accepted, ConvergedValue=0 means no draw"
    );
    assert_eq!(
        g.players[0].graveyard.len(),
        gy_before,
        "No discard fires when ConvergedValue=0 (gate fails)"
    );
}

// ── Steal the Show — DiscardAnyNumber promotion ─────────────────────────────

#[test]
fn steal_the_show_mode_zero_discard_any_number_drops_zero_by_default() {
    // Mode 0 — target player discards any number, draws that many.
    // AutoDecider runs both default modes; with no creature target at
    // slot 1, mode 1 fizzles, and mode 0's discard auto-picks 0 (no draw).
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    // Give P0 a few hand cards so a non-zero pick would be observable.
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::lightning_bolt());
    }
    let id = g.add_card_to_hand(0, catalog::steal_the_show());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("Steal the Show castable for {2}{R} mode 0");
    drain_stack(&mut g);

    // AutoDecider picked 0 to discard → 0 to draw. Net hand: -1 (cast).
    assert_eq!(
        g.players[0].hand.len(),
        hand_before - 1,
        "AutoDecider picks 0 cards to discard → no draw → hand only loses the cast Steal the Show"
    );
}

#[test]
fn steal_the_show_runs_both_modes_with_per_mode_targets() {
    // "Choose one or both" — default picks [0, 1] run both modes, each
    // reading its own target slot: mode 0 the player (slot 0), mode 1 the
    // creature (slot 1). 3 IS cards in P0's graveyard → 3 damage kills the
    // bear, proving mode 1 read the slot-1 creature rather than the slot-0
    // player.
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
    }
    let id = g.add_card_to_hand(0, catalog::steal_the_show());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),                    // slot 0 → mode 0 (player)
        additional_targets: vec![Target::Permanent(bear)],  // slot 1 → mode 1 (creature)
        mode: None,
        x_value: None,
    })
    .expect("Steal the Show castable for {2}{R} with both modes' targets");
    drain_stack(&mut g);

    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "mode 1 read the slot-1 creature target → bear takes 3 and dies"
    );
}

#[test]
fn steal_the_show_scripted_pick_runs_only_the_chosen_mode() {
    // A ScriptedDecider picks only mode 1; mode 0 is skipped. The stable
    // slot mapping still routes the creature (slot 1) to mode 1, so the bear
    // dies while the slot-0 player is left alone (no discard/draw).
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
    }
    g.add_card_to_hand(1, catalog::lightning_bolt()); // P1 hand, would shrink if mode 0 ran
    let p1_hand_before = g.players[1].hand.len();
    let id = g.add_card_to_hand(0, catalog::steal_the_show());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Modes(vec![1])]));

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),                    // slot 0 (validated, unused)
        additional_targets: vec![Target::Permanent(bear)],  // slot 1 → mode 1
        mode: None,
        x_value: None,
    })
    .expect("castable with both slots filled (validated against default picks)");
    drain_stack(&mut g);

    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "only-mode-1 pick still routes the slot-1 creature → bear dies"
    );
    assert_eq!(
        g.players[1].hand.len(), p1_hand_before,
        "mode 0 was not chosen → target player did not discard/draw"
    );
}

#[test]
fn witherbloom_balancer_affinity_for_creatures_reduces_cost() {
    // Affinity for creatures: "{1} less for each creature you control."
    // With 4 of your creatures, Witherbloom, the Balancer should cost
    // {2}{B}{G} instead of {6}{B}{G}.
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    // Opp creatures should NOT count (ControlledByYou narrows).
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_the_balancer());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Witherbloom Balancer castable at {2}{B}{G} via affinity discount");
    drain_stack(&mut g);
    let drag = g.battlefield.iter().find(|c| c.definition.name == "Witherbloom, the Balancer");
    assert!(drag.is_some(), "Witherbloom Balancer on battlefield");
}

#[test]
fn witherbloom_balancer_grants_affinity_to_is_spells() {
    // With Balancer + 1 bear (2 creatures you control), the caster's
    // Mind Rot ({2}{B}) gets {2} less = costs {B}.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::witherbloom_the_balancer());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Stock opp hand to discard.
    g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::lightning_bolt());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::mind_rot());
    g.players[0].mana_pool.add(Color::Black, 1);
    // {B} only — Mind Rot is normally {2}{B} but with 2 creatures you
    // control the generic side is consumed.
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mind Rot castable at {B} via Balancer's grant");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "Opp discarded both bolts");
}

#[test]
fn witherbloom_balancer_static_does_not_affect_opp_spells() {
    // Opp's IS spell should NOT get any Affinity discount from our Balancer.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::witherbloom_the_balancer());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::lightning_bolt());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(1, catalog::mind_rot());
    g.players[1].mana_pool.add(Color::Black, 1);
    // Opp has only {B} — Mind Rot costs {2}{B}. With no Affinity grant
    // for opp, the cast should fail (no generic mana available).
    g.priority.player_with_priority = 1;
    let result = g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::types::Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(result.is_err(), "Opp's Mind Rot not discounted by our Balancer");
}

// ── Cast-from-exile / may-play permission (Practiced Scrollsmith etc.) ──────

#[test]
fn practiced_scrollsmith_grants_may_play_on_exiled_card() {
    // The ETB trigger should both exile a noncreature/nonland card from
    // the controller's graveyard *and* stamp it with a
    // `may_play_until = EndOfControllersNextTurn` permission.
    let mut g = two_player_game();
    let pox_id = g.next_id();
    let mut pox = crabomination::card::CardInstance::new(pox_id, catalog::pox_plague(), 0);
    pox.controller = 0;
    g.players[0].graveyard.push(pox);

    let id = g.add_card_to_hand(0, catalog::practiced_scrollsmith());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Practiced Scrollsmith castable");
    drain_stack(&mut g);

    let exiled = g.exile.iter().find(|c| c.id == pox_id).expect("Pox in exile");
    let perm = exiled.may_play_until.expect("may_play permission stamped");
    assert_eq!(perm.player, 0, "permission goes to the ETB controller");
    assert!(matches!(perm.duration,
        crabomination::card::MayPlayDuration::EndOfControllersNextTurn));
}

#[test]
fn practiced_scrollsmith_may_play_expires_after_controllers_next_turn() {
    // EndOfControllersNextTurn semantics in a 2-player game: the
    // permission survives the granting turn's cleanup and the opp's
    // turn's cleanup, then clears on the controller's next cleanup.
    // We approximate this by checking the permission persists across
    // one cleanup but clears once `turn_number - granted_turn >=
    // player_count`. In a 2p game that's 2 turns later — the
    // controller's next cleanup.
    let mut g = two_player_game();
    let pox_id = g.next_id();
    let mut pox = crabomination::card::CardInstance::new(pox_id, catalog::pox_plague(), 0);
    pox.controller = 0;
    pox.may_play_until = Some(crabomination::card::MayPlayPermission {
        player: 0,
        granted_turn: g.turn_number,
        duration: crabomination::card::MayPlayDuration::EndOfControllersNextTurn,
        exile_after: false,
        miracle: false,
    });
    g.exile.push(pox);

    // Burn through 2 cleanup steps (each `do_cleanup` advances the
    // turn). After cleanup #1 the permission persists; after #2 it
    // clears.
    g.do_cleanup(&mut Vec::new());
    assert!(g.exile.iter().find(|c| c.id == pox_id).unwrap()
        .may_play_until.is_some(), "permission survives first cleanup");
    g.do_cleanup(&mut Vec::new());
    assert!(g.exile.iter().find(|c| c.id == pox_id).unwrap()
        .may_play_until.is_none(), "permission expires after controllers next turn cleanup");
}

#[test]
fn cast_from_zone_without_paying_recurs_practiced_scrollsmiths_exiled_card() {
    // End-to-end: ETB exiles Pox Plague + stamps may_play; the printed
    // "you may cast that card" is a normal cast (`pay_own_cost: true`),
    // so recasting it from exile charges Pox's real {B}{B}{B}{B}{B}.
    let mut g = two_player_game();
    let pox_id = g.next_id();
    let mut pox = crabomination::card::CardInstance::new(pox_id, catalog::pox_plague(), 0);
    pox.controller = 0;
    g.players[0].graveyard.push(pox);

    let id = g.add_card_to_hand(0, catalog::practiced_scrollsmith());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Practiced Scrollsmith castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == pox_id), "Pox exiled after ETB");

    // Recast Pox Plague from exile, paying its own cost.
    g.players[0].mana_pool.add(Color::Black, 5);
    let p0_mana_before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: pox_id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Pox recastable via may_play permission, paying its cost");
    drain_stack(&mut g);
    // The {B}{B}{B}{B}{B} cost was deducted.
    assert_eq!(g.players[0].mana_pool.total(), p0_mana_before - 5,
        "pay-own-cost recast deducts Pox's mana cost");
    // Pox resolved → it landed in graveyard (it's a sorcery, exile_after
    // = false in the permission).
    assert!(g.players[0].graveyard.iter().any(|c| c.id == pox_id),
        "Pox went to graveyard after recast resolve");
    // No more permission outstanding (consumed by cast).
    assert!(g.players[0].graveyard.iter().find(|c| c.id == pox_id)
        .unwrap().may_play_until.is_none(),
        "permission cleared on cast");
}

#[test]
fn cast_from_zone_without_paying_rejected_without_permission() {
    // A card with no may_play permission can't be cast for free.
    let mut g = two_player_game();
    let pox_id = g.next_id();
    let mut pox = crabomination::card::CardInstance::new(pox_id, catalog::pox_plague(), 0);
    pox.controller = 0;
    g.exile.push(pox);

    let result = g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: pox_id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(result.is_err(), "no permission → cast rejected");
}

#[test]
fn suspend_aggression_grants_may_play_to_each_exiled_card() {
    // Suspend Aggression exiles a target nonland permanent + top of
    // your library; each exiled card gets `may_play_until` stamped
    // with `to_owner: true` so the card's owner can cast it later.
    let mut g = two_player_game();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    // P1 controls a creature we'll target.
    let opp_creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // P0's top-of-library has a known card so we can identify it.
    let _top = g.add_card_to_library(0, catalog::lightning_bolt());
    let suspend_id = g.add_card_to_hand(0, catalog::suspend_aggression());

    g.perform_action(GameAction::CastSpell {
        card_id: suspend_id,
        target: Some(crabomination::game::types::Target::Permanent(opp_creature)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Suspend Aggression castable");
    drain_stack(&mut g);

    // Both exiled cards should have permissions.
    let exiled: Vec<_> = g.exile.iter().filter(|c|
        c.may_play_until.is_some()
    ).collect();
    assert_eq!(exiled.len(), 2, "both exiled cards get may_play");
    // Each permission routes to that card's owner.
    for c in &exiled {
        let perm = c.may_play_until.unwrap();
        assert_eq!(perm.player, c.owner,
            "permission goes to card's owner (to_owner = true)");
    }
}

#[test]
fn tablet_of_discovery_etb_mills_and_grants_may_play() {
    let mut g = two_player_game();
    let top_id = g.add_card_to_library(0, catalog::lightning_bolt());
    let tablet_id = g.add_card_to_hand(0, catalog::tablet_of_discovery());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: tablet_id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Tablet castable");
    drain_stack(&mut g);

    // The top library card was milled to graveyard.
    let milled = g.players[0].graveyard.iter().find(|c| c.id == top_id)
        .expect("top card milled to gy");
    let perm = milled.may_play_until.expect("milled card has may_play");
    assert!(matches!(perm.duration,
        crabomination::card::MayPlayDuration::EndOfThisTurn));
}

#[test]
fn ark_of_hunger_mill_activation_grants_may_play() {
    let mut g = two_player_game();
    let top_id = g.add_card_to_library(0, catalog::lightning_bolt());
    let ark_id = g.add_card_to_battlefield(0, catalog::ark_of_hunger());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == ark_id) {
        c.summoning_sick = false;
    }

    g.perform_action(GameAction::ActivateAbility {
        card_id: ark_id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("Ark mill activation");
    drain_stack(&mut g);

    let milled = g.players[0].graveyard.iter().find(|c| c.id == top_id)
        .expect("top milled");
    assert!(milled.may_play_until.is_some(), "milled card may-play granted");
}

#[test]
fn improvisation_capstone_exiles_four_cards_and_registers_paradigm() {
    let mut g = two_player_game();
    // Stack the top of P0's library with 4 known cards.
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    let id = g.add_card_to_hand(0, catalog::improvisation_capstone());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(5);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Improvisation Capstone castable");
    drain_stack(&mut g);

    // 4 cards from library should be exiled (Improvisation Capstone
    // itself also lands in exile via exile_on_resolve).
    let exiled_bolts = g.exile.iter().filter(|c|
        c.definition.name == "Lightning Bolt"
    ).count();
    assert_eq!(exiled_bolts, 4, "exiled top 4 library cards");
    let capstone_in_exile = g.exile.iter().any(|c|
        c.definition.name == "Improvisation Capstone"
    );
    assert!(capstone_in_exile, "Capstone exiled (exile_on_resolve)");
    // Paradigm registered: there's a YourNextMainPhase delayed trigger
    // whose source is the Capstone.
    let cap_id = g.exile.iter().find(|c|
        c.definition.name == "Improvisation Capstone"
    ).map(|c| c.id).unwrap();
    let registered = g.delayed_triggers.iter().any(|dt|
        dt.source == cap_id
        && matches!(dt.kind, crabomination::game::types::DelayedKind::YourNextMainPhase)
        && !dt.fires_once
    );
    assert!(registered, "Paradigm trigger registered (recurring)");
}

#[test]
fn improvisation_capstone_digs_past_lands_until_mv_threshold_hit() {
    // Top of library: 3 Forests (MV 0) + 1 Lightning Bolt (MV 1) +
    // 1 Cancel (MV 3). Running MV sum walks 0, 0, 0, 1, 4 — gate hit
    // after Cancel. Five cards exiled (was four under the prior
    // hard-coded Const(4)). Validates the new
    // `Selector::TopOfLibraryUntilMvAtLeast` primitive.
    let mut g = two_player_game();
    // add_card_to_library pushes onto the END (= bottom). Insert in
    // reverse so the top-of-library order is Forest, Forest, Forest,
    // Bolt, Cancel.
    use crabomination::card::CardInstance;
    let mut top_to_bottom: Vec<CardInstance> = vec![
        CardInstance::new(g.next_id(), catalog::forest(), 0),
        CardInstance::new(g.next_id(), catalog::forest(), 0),
        CardInstance::new(g.next_id(), catalog::forest(), 0),
        CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0),
        CardInstance::new(g.next_id(), catalog::cancel(), 0),
    ];
    for c in top_to_bottom.iter_mut() { c.controller = 0; }
    // Splice these in at the top of P0's library.
    for c in top_to_bottom.into_iter().rev() {
        g.players[0].library.insert(0, c);
    }
    let id = g.add_card_to_hand(0, catalog::improvisation_capstone());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(5);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Improvisation Capstone castable");
    drain_stack(&mut g);

    // Three Forests + one Bolt + one Cancel = 5 cards in exile.
    // (Lightning Bolt's free cast resolves immediately and may leave
    //  the IS card on the stack / battlefield mid-stack drain; what
    //  we're checking is that all five made it OUT of the library.)
    let lib_remaining = g.players[0].library.len();
    assert_eq!(
        lib_remaining, 0,
        "All five seeded cards walked out of the library (sum 0+0+0+1+3 ≥ 4)",
    );
}

#[test]
fn the_dawning_archaic_attack_trigger_uses_immediate_free_cast() {
    // The attack trigger is `CastWithoutPayingImmediate { source: Graveyard }`
    // — by default AutoDecider declines (Bool(false)), so nothing
    // happens. The Archaic ETBs/attacks as usual.
    let mut g = two_player_game();
    let _bolt = g.next_id();
    // Seed an IS card in P0's graveyard for the trigger to find.
    let mut bolt = crabomination::card::CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    g.players[0].graveyard.push(bolt);
    let arc = g.add_card_to_battlefield(0, catalog::the_dawning_archaic());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == arc) {
        c.summoning_sick = false;
        c.tapped = false;
    }
    // Sanity: card has Attacks trigger with CastWithoutPayingImmediate.
    let def = catalog::the_dawning_archaic();
    let has_attack_free_cast = def.triggered_abilities.iter().any(|ta| {
        matches!(ta.effect, crabomination::effect::Effect::CastWithoutPayingImmediate { .. })
    });
    assert!(has_attack_free_cast,
        "Dawning Archaic has an attack-triggered free-cast effect");
}

#[test]
fn the_dawning_archaic_cost_reduces_per_is_in_graveyard() {
    // Push (modern_decks, batch 78): Dawning Archaic's "This spell
    // costs {1} less to cast for each instant and sorcery card in
    // your graveyard" rider is now wired via the per-card
    // graveyard-IS counter in `cost_reduction_for_spell`. With 3 IS
    // cards in P0's gy, the printed {10} cost reduces to {7}.
    let mut g = two_player_game();
    let archaic_id = g.add_card_to_hand(0, catalog::the_dawning_archaic());

    // Seed 3 IS cards in P0's graveyard.
    for _ in 0..3 {
        let mut bolt = crabomination::card::CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
        bolt.controller = 0;
        g.players[0].graveyard.push(bolt);
    }

    // Pay only 7 generic mana — should succeed thanks to the gy discount.
    g.players[0].mana_pool.add_colorless(7);
    g.perform_action(GameAction::CastSpell {
        card_id: archaic_id, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Dawning Archaic castable at {7} with 3 IS cards in gy");
    drain_stack(&mut g);

    assert!(
        g.battlefield.iter().any(|c| c.id == archaic_id),
        "Dawning Archaic resolved onto the battlefield",
    );
}

#[test]
fn the_dawning_archaic_cost_does_not_reduce_with_empty_graveyard() {
    // With an empty IS-in-gy count, full {10} is required.
    let mut g = two_player_game();
    let archaic_id = g.add_card_to_hand(0, catalog::the_dawning_archaic());
    g.players[0].mana_pool.add_colorless(7);
    let result = g.perform_action(GameAction::CastSpell {
        card_id: archaic_id, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(
        result.is_err(),
        "Dawning Archaic at {{7}} with empty gy should be rejected (full cost {{10}})",
    );
}

#[test]
fn rabid_attack_grants_die_draws_card_trigger() {
    // Push (modern_decks, batch 85): Rabid Attack grants each pumped
    // target a CreatureDied/SelfSource trigger ("draw a card on die")
    // until end of turn. Kill the bear after the grant lands — the
    // granted trigger fires from the SBA dies handler (now consulting
    // `granted_triggers_eot` alongside printed Dies triggers).
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let ra = g.add_card_to_hand(0, catalog::rabid_attack());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.add_card_to_library(0, catalog::lightning_bolt()); // for the draw

    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: ra,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Rabid Attack castable");
    drain_stack(&mut g);

    // Hand size now: -1 (Rabid Attack left hand) → hand_before - 1.
    let hand_after_cast = g.players[0].hand.len();
    // Kill the bear via lethal damage.
    g.battlefield_find_mut(bear).unwrap().damage = 2;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    // We need to also dispatch the CreatureDied event so the trigger
    // we registered via granted_triggers_eot actually fires. The
    // SBA-dies-handler already pushes printed Dies triggers + granted
    // ones to the stack inside check_state_based_actions, so drain
    // again to resolve.
    drain_stack(&mut g);

    // Player should have drawn 1 card from the granted die-trigger.
    let hand_after_die = g.players[0].hand.len();
    assert_eq!(
        hand_after_die,
        hand_after_cast + 1,
        "Rabid Attack's granted die-trigger fired → +1 card",
    );
    let _ = hand_before;
}

#[test]
fn root_manipulation_grants_attack_lifegain_trigger() {
    // Push (modern_decks, batch 84): Root Manipulation grants each
    // friendly creature an Attacks/SelfSource trigger ("gain 1 life
    // on attack") via the new `Effect::GrantTriggeredAbility`. When
    // the bear attacks after Root Manipulation resolved, the trigger
    // fires and P0 gains 1 life.
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let rm = g.add_card_to_hand(0, catalog::root_manipulation());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: rm, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Root Manipulation castable");
    drain_stack(&mut g);

    let life_before = g.players[0].life;
    g.step = crabomination::game::TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }])).expect("bear can attack with menace");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 1,
        "Root Manipulation's granted attack-trigger fires → gain 1 life");
}

#[test]
fn group_project_flashback_taps_three_creatures_and_mints_spirit() {
    // Push (modern_decks, batch 83): Group Project's "Flashback—Tap
    // three untapped creatures you control" wired via the new
    // `Keyword::FlashbackTap(3)` + `GameAction::CastFlashbackTap`. Seed
    // Group Project in P0's graveyard, three untapped bears on the bf,
    // and invoke the flashback action — the three bears tap, Group
    // Project moves to exile, a 2/2 R/W Spirit token enters.
    let mut g = two_player_game();
    let mut gp = crabomination::card::CardInstance::new(g.next_id(), catalog::group_project(), 0);
    gp.controller = 0;
    let gp_id = gp.id;
    g.players[0].graveyard.push(gp);
    let bear_a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear_b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear_c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for cid in [bear_a, bear_b, bear_c] {
        g.clear_sickness(cid);
    }

    g.perform_action(GameAction::CastFlashbackTap {
        card_id: gp_id,
        tap_creatures: vec![bear_a, bear_b, bear_c],
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Group Project flashback castable with 3 bears");
    drain_stack(&mut g);

    // Three bears all tapped.
    for cid in [bear_a, bear_b, bear_c] {
        assert!(
            g.battlefield_find(cid).unwrap().tapped,
            "bear {} was tapped as flashback cost", cid.0,
        );
    }
    // Group Project landed in exile (cast_via_flashback routing).
    assert!(
        g.exile.iter().any(|c| c.id == gp_id),
        "Group Project exiled after flashback resolution",
    );
    // A Spirit token entered.
    let spirit_present = g.battlefield.iter()
        .any(|c| c.controller == 0 && c.is_token && c.definition.name == "Spirit");
    assert!(spirit_present, "2/2 R/W Spirit token entered");
}

#[test]
fn group_project_flashback_rejects_wrong_tap_count() {
    // Only 2 creatures listed → flashback rejected.
    let mut g = two_player_game();
    let mut gp = crabomination::card::CardInstance::new(g.next_id(), catalog::group_project(), 0);
    gp.controller = 0;
    let gp_id = gp.id;
    g.players[0].graveyard.push(gp);
    let bear_a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear_b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear_a);
    g.clear_sickness(bear_b);

    let result = g.perform_action(GameAction::CastFlashbackTap {
        card_id: gp_id,
        tap_creatures: vec![bear_a, bear_b],
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(result.is_err(),
        "Group Project flashback requires exactly 3 creatures");
}

#[test]
fn fractal_tender_end_step_mints_fractal_when_gained_counter() {
    // Push (modern_decks, batch 82): Fractal Tender's end-step "if you
    // put a counter on this creature this turn, mint a Fractal with 3
    // +1/+1 counters" rider. Add a counter, then advance to end step
    // — the trigger fires and a Fractal token enters with 3 +1/+1
    // counters (= a 3/3 Fractal).
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let tender = g.add_card_to_battlefield(0, catalog::fractal_tender());
    // Manually add a +1/+1 counter to Tender (simulating the Increment
    // trigger that fires on big-spell casts).
    g.battlefield_find_mut(tender).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.permanents_gained_counter_this_turn.insert(tender);
    // Advance to End step.
    let bf_before = g.battlefield.len();
    g.step = crabomination::game::TurnStep::End;
    g.fire_step_triggers(crabomination::game::TurnStep::End);
    drain_stack(&mut g);

    let fractal_present = g.battlefield.iter()
        .any(|c| c.is_token && c.definition.subtypes.creature_types
            .contains(&crabomination::card::CreatureType::Fractal));
    assert!(
        fractal_present,
        "Fractal Tender minted a Fractal at end step (gained counter this turn)",
    );
    let _ = bf_before;
}

#[test]
fn fractal_tender_end_step_skips_when_no_counter_gained() {
    // No counters added → trigger should not fire.
    let mut g = two_player_game();
    let _tender = g.add_card_to_battlefield(0, catalog::fractal_tender());
    let bf_before = g.battlefield.len();
    g.step = crabomination::game::TurnStep::End;
    g.fire_step_triggers(crabomination::game::TurnStep::End);
    drain_stack(&mut g);

    let fractal_present = g.battlefield.iter()
        .any(|c| c.is_token && c.definition.subtypes.creature_types
            .contains(&crabomination::card::CreatureType::Fractal));
    assert!(
        !fractal_present,
        "Fractal Tender does NOT mint a Fractal when no counter was added this turn",
    );
    assert_eq!(g.battlefield.len(), bf_before);
}

#[test]
fn quandrix_the_proof_cascade_exiles_nonland_with_lower_mv() {
    // Real cascade (CR 702.85): when Quandrix is cast, exile from the top
    // until a nonland with MV < 6 — 2 Forests (lands) then a Lightning
    // Bolt (MV 1). The free-cast offer is accepted (AutoDecider), so the
    // Bolt is cast for free and ends in the graveyard; the exiled Forests
    // are bottomed.
    let mut g = two_player_game();
    use crabomination::card::CardInstance;
    let mut bolt = CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    let bolt_id = bolt.id;
    let mut top: Vec<CardInstance> = vec![
        CardInstance::new(g.next_id(), catalog::forest(), 0),
        CardInstance::new(g.next_id(), catalog::forest(), 0),
        bolt,
    ];
    for c in top.iter_mut() { c.controller = 0; }
    for c in top.into_iter().rev() {
        g.players[0].library.insert(0, c);
    }
    let qp_id = g.add_card_to_hand(0, catalog::quandrix_the_proof());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    // Accept the cascade free-cast offer.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    g.perform_action(GameAction::CastSpell {
        card_id: qp_id, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quandrix castable at {4}{G}{U}");
    drain_stack(&mut g);

    // Bolt was cast for free during trigger resolution → graveyard.
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == bolt_id),
        "Bolt free-cast by cascade and resolved to the graveyard"
    );
    assert!(
        !g.exile.iter().any(|c| c.id == bolt_id),
        "no card stranded in exile after cascade"
    );
    // The exiled Forests were bottomed, not left in exile.
    assert!(
        !g.exile.iter().any(|c| c.definition.name == "Forest"),
        "cascade misses go to the bottom of the library"
    );
    assert_eq!(
        g.players[0]
            .library
            .iter()
            .filter(|c| c.definition.name == "Forest")
            .count(),
        2,
        "both Forests back in the library"
    );
}

#[test]
fn quandrix_the_proof_grants_cascade_to_your_is_spells_from_hand() {
    // "Instant and sorcery spells you cast from your hand have cascade."
    // With Quandrix on the battlefield, casting Divination (MV 3) cascades
    // at its own mana value, exiling the top nonland with MV < 3 — a
    // Lightning Bolt (MV 1) — with a free-cast permission.
    use crabomination::card::CardInstance;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::quandrix_the_proof());
    drain_stack(&mut g);

    // Library top → bottom: Bolt (the cascade hit), then two Islands for
    // Divination to draw after the cascade resolves.
    let mut bolt = CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    let bolt_id = bolt.id;
    let mut top: Vec<CardInstance> = vec![
        bolt,
        CardInstance::new(g.next_id(), catalog::island(), 0),
        CardInstance::new(g.next_id(), catalog::island(), 0),
    ];
    for c in top.iter_mut() { c.controller = 0; }
    for c in top.into_iter().rev() { g.players[0].library.insert(0, c); }

    let div = g.add_card_to_hand(0, catalog::divination());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    // Accept the cascade's free cast so the Bolt leaves the library
    // (declining would merely bottom it, indistinguishable from a no-op).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastSpell {
        card_id: div, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Divination castable for {2}{U}");
    drain_stack(&mut g);

    // The granted cascade (cap = Divination's MV 3) hit the Bolt (MV 1)
    // and cast it from exile — so it's no longer in the library or hand.
    assert!(
        !g.players[0].library.iter().any(|c| c.id == bolt_id)
            && !g.players[0].hand.iter().any(|c| c.id == bolt_id),
        "granted cascade found and cast the cheaper nonland",
    );
}

#[test]
fn quandrix_the_proof_does_not_cascade_your_creature_spells() {
    // The grant is restricted to instants and sorceries — a creature spell
    // you cast does not cascade.
    use crabomination::card::CardInstance;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::quandrix_the_proof());
    drain_stack(&mut g);

    let mut bolt = CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    let bolt_id = bolt.id;
    g.players[0].library.insert(0, bolt);

    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Grizzly Bears castable for {1}{G}");
    drain_stack(&mut g);

    assert_eq!(
        g.players[0].library.first().map(|c| c.id),
        Some(bolt_id),
        "no cascade off a creature spell — the Bolt stays on top of the library",
    );
}

#[test]
fn nita_forum_conciliator_activation_exiles_and_grants_may_play() {
    let mut g = two_player_game();
    // Seed an IS card in P1's graveyard.
    let mut bolt = crabomination::card::CardInstance::new(g.next_id(), catalog::lightning_bolt(), 1);
    bolt.controller = 1;
    let bolt_id = bolt.id;
    g.players[1].graveyard.push(bolt);
    // Nita + a sacrificial creature on P0's bf.
    let nita = g.add_card_to_battlefield(0, catalog::nita_forum_conciliator());
    let sac = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == nita) {
        c.summoning_sick = false; c.tapped = false;
    }
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == sac) {
        c.summoning_sick = false;
    }
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::ActivateAbility {
        card_id: nita, ability_index: 0,
        // The printed "exile TARGET instant or sorcery card" — the ability
        // targets the graveyard card; the sacrifice is a cost picked by
        // the engine's sac_cost machinery.
        target: Some(crabomination::game::types::Target::Permanent(bolt_id)),
        additional_targets: Vec::new(), x_value: None }).expect("Nita activation");
    drain_stack(&mut g);

    // Bolt should now be in exile with may_play + pay-own-cost-any-color
    // stamped (printed: "mana of any type can be spent to cast that
    // spell" — NOT a free cast), and no exile-after rider.
    let exiled = g.exile.iter().find(|c| c.id == bolt_id)
        .expect("bolt moved to exile by Nita");
    let perm = exiled.may_play_until.expect("may_play stamped");
    assert!(!perm.exile_after, "printed text has no exile-on-resolution rider");
    assert_eq!(perm.player, 0, "permission goes to Nita's controller");
    let alt = exiled.granted_alt_cast_cost_eot.clone().expect("pays its own cost");
    assert_eq!(alt.cmc(), 1, "Bolt's mana value, payable with any type of mana");
}

#[test]
fn nita_trigger_fans_counters_when_casting_unowned_spell() {
    // Set up Nita + a friendly bear on P0's battlefield. Manually
    // place a P1-owned Lightning Bolt in exile with may_play_until
    // permission granted to P0 (bypassing Nita's own activation,
    // which would sac her and remove her trigger). When P0 then casts
    // the Bolt, Nita's trigger fires because the spell's owner (P1)
    // ≠ Nita's controller (P0).
    use crabomination::card::{MayPlayDuration, MayPlayPermission};
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let mut bolt = crabomination::card::CardInstance::new(g.next_id(), catalog::lightning_bolt(), 1);
    bolt.controller = 0; // P0 is the caster
    bolt.may_play_until = Some(MayPlayPermission {
        player: 0,
        granted_turn: g.turn_number,
        duration: MayPlayDuration::EndOfThisTurn,
        exile_after: false,
        miracle: false,
    });
    let bolt_id = bolt.id;
    g.exile.push(bolt);

    let nita = g.add_card_to_battlefield(0, catalog::nita_forum_conciliator());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for cid in [nita, bear] {
        if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == cid) {
            c.summoning_sick = false; c.tapped = false;
        }
    }
    g.players[0].mana_pool.add(Color::Red, 1); // for the Bolt cast

    let counters_before_nita = g.battlefield.iter()
        .find(|c| c.id == nita)
        .map(|c| c.counter_count(crabomination::card::CounterType::PlusOnePlusOne))
        .unwrap_or(0);
    let counters_before_bear = g.battlefield.iter()
        .find(|c| c.id == bear)
        .map(|c| c.counter_count(crabomination::card::CounterType::PlusOnePlusOne))
        .unwrap_or(0);

    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bolt_id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bolt castable by P0 via may_play permission");
    drain_stack(&mut g);

    let nita_after = g.battlefield.iter()
        .find(|c| c.id == nita)
        .map(|c| c.counter_count(crabomination::card::CounterType::PlusOnePlusOne))
        .unwrap_or(0);
    let bear_after = g.battlefield.iter()
        .find(|c| c.id == bear)
        .map(|c| c.counter_count(crabomination::card::CounterType::PlusOnePlusOne))
        .unwrap_or(0);
    assert_eq!(nita_after - counters_before_nita, 1,
        "Nita gets a +1/+1 counter from her own trigger");
    assert_eq!(bear_after - counters_before_bear, 1,
        "the friendly bear also gets a counter");
}

#[test]
fn nita_trigger_does_not_fire_on_own_spells() {
    // Casting one of your OWN spells (owner = controller = you) does
    // NOT fire Nita's "spell you don't own" trigger.
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let nita = g.add_card_to_battlefield(0, catalog::nita_forum_conciliator());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for cid in [nita, bear] {
        if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == cid) {
            c.summoning_sick = false; c.tapped = false;
        }
    }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    let counters_before: i32 = g.battlefield.iter()
        .find(|c| c.id == nita).map(|c| c.counter_count(crabomination::card::CounterType::PlusOnePlusOne) as i32).unwrap_or(0);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let counters_after: i32 = g.battlefield.iter()
        .find(|c| c.id == nita).map(|c| c.counter_count(crabomination::card::CounterType::PlusOnePlusOne) as i32).unwrap_or(0);
    assert_eq!(counters_after, counters_before,
        "Nita's trigger does NOT fire on own-spell casts");
}

#[test]
fn paradigm_card_registers_recurring_yournextmainphase_trigger() {
    // Restoration Seminar resolves → lands in exile + registers a
    // recurring YourNextMainPhase delayed trigger.
    let mut g = two_player_game();
    // Seed a creature in P0's graveyard for the body to target.
    let bears = crabomination::card::CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0);
    let bears_id = bears.id;
    g.players[0].graveyard.push(bears);

    let id = g.add_card_to_hand(0, catalog::restoration_seminar());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(5);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crabomination::game::types::Target::Permanent(bears_id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Restoration Seminar castable");
    drain_stack(&mut g);

    // Body fired: bears moved from gy to battlefield.
    assert!(g.battlefield.iter().any(|c| c.id == bears_id),
        "bears reanimated");
    // Seminar landed in exile (exile_on_resolve = true).
    let seminar_in_exile = g.exile.iter().find(|c|
        c.definition.name == "Restoration Seminar"
    );
    assert!(seminar_in_exile.is_some(), "Seminar exiled");
    let seminar_id = seminar_in_exile.unwrap().id;
    // Recurring trigger registered.
    let registered = g.delayed_triggers.iter().any(|dt|
        dt.source == seminar_id
        && matches!(dt.kind, crabomination::game::types::DelayedKind::YourNextMainPhase)
        && !dt.fires_once
    );
    assert!(registered, "Paradigm trigger registered");
}

#[test]
fn paradigm_free_copy_resolves_with_scripted_yes() {
    // Direct unit test of `Effect::CastFreeParadigmCopy`: park a
    // Paradigm card in exile, then resolve the effect via a trigger.
    // With a scripted yes, a tokenized copy is minted and free-cast.
    let mut g = two_player_game();
    // Seed a creature in gy for the copy's body to target.
    let bears = crabomination::card::CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0);
    let bears_id = bears.id;
    g.players[0].graveyard.push(bears);

    // Park Restoration Seminar in exile.
    let seminar = crabomination::card::CardInstance::new(
        g.next_id(), catalog::restoration_seminar(), 0
    );
    let seminar_id = seminar.id;
    g.exile.push(seminar);

    // Script "yes" for the paradigm offer.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    // Resolve `CastFreeParadigmCopy` directly as a trigger from the
    // Paradigm-exiled card. Threads `source = seminar_id` so the
    // effect locates the original in exile.
    g.continue_trigger_resolution_with_source(
        seminar_id,
        0,
        crabomination::effect::Effect::CastFreeParadigmCopy,
        None,
        0, 0, 0, 0, None, 0,
        Vec::new(),
    ).expect("paradigm copy effect resolves");
    drain_stack(&mut g);

    // The tokenized copy resolved → its body reanimated the bears.
    assert!(g.battlefield.iter().any(|c| c.id == bears_id),
        "paradigm-copied seminar reanimated the bears");
    // Original seminar still in exile (paradigm copies are tokenized
    // and removed by SBA from non-battlefield zones).
    assert!(g.exile.iter().any(|c| c.id == seminar_id),
        "original Seminar stays in exile");
}

