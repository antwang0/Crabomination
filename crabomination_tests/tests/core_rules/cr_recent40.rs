//! CR conformance for the modern_decks text-change / zone / face-down pass:
//! - CR 612.2/612.3 — a text change rewrites printed color words and basic
//!   land types only; granted keywords aren't text.
//! - CR 400.3/400.4a/400.7 — owner's-zone routing, nonpermanents can't enter
//!   the battlefield, and a card that changes zones is a new object.
//! - CR 708.2b/708.8 — a face-down permanent can't be turned face down again,
//!   and turning face up keeps effects already applied to it.

use crabomination::card::{CardInstance, CounterType, Keyword, LandType};
use crabomination::catalog;
use crabomination::game::layers::{
    AffectedPermanents, ContinuousEffect, EffectDuration, Layer, Modification,
};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn install(g: &mut GameState, on: crabomination::card::CardId, m: Modification, layer: Layer) {
    let ts = g.next_timestamp();
    g.add_continuous_effect(ContinuousEffect {
        timestamp: ts,
        source: on,
        affected: AffectedPermanents::Specific(vec![on]),
        layer,
        sublayer: None,
        duration: EffectDuration::UntilEndOfTurn,
        modification: m,
    });
}

/// CR 612.2 — rewriting a basic land type changes the type line (and the
/// intrinsic mana ability that follows it) without touching the card's name.
#[test]
fn cr_612_2_land_type_rewrite_moves_the_mana_ability_not_the_name() {
    let mut g = main_phase();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    install(
        &mut g,
        forest,
        Modification::ReplaceBasicLandType(LandType::Forest, LandType::Island),
        Layer::L3Text,
    );
    let cp = g.computed_permanent(forest).expect("computed");
    assert_eq!(cp.subtypes.land_types, vec![LandType::Island]);
    assert_eq!(
        g.battlefield_find(forest).unwrap().definition.name,
        "Forest",
        "CR 612.2 — a subtype rewrite can't change a card name"
    );
    // CR 305.6 — the intrinsic mana ability follows the computed type line:
    // the printed Forest tap is gone and an Island tap appears in its place.
    let mana: Vec<_> = g.effective_mana_abilities(forest);
    assert_eq!(mana.len(), 1, "exactly one mana ability");
    g.perform_action(GameAction::ActivateAbility {
        card_id: forest,
        ability_index: mana[0].0,
        target: None,
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 0);
}

/// CR 612.3 — granted abilities aren't part of an object's text, so a
/// color-word rewrite reaches the printed protection but not a granted one.
#[test]
fn cr_612_3_color_word_rewrite_skips_a_granted_protection() {
    let mut g = main_phase();
    let printed = g.add_card_to_battlefield(0, catalog::white_knight());
    let granted = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    install(
        &mut g,
        granted,
        Modification::AddKeyword(Keyword::Protection(Color::Black)),
        Layer::L6Ability,
    );
    for id in [printed, granted] {
        install(
            &mut g,
            id,
            Modification::ReplaceColorWord(Color::Black, Color::Red),
            Layer::L3Text,
        );
    }
    let pk = g.computed_permanent(printed).expect("computed").keywords;
    assert!(pk.contains(&Keyword::Protection(Color::Red)), "printed word rewritten");
    assert!(!pk.contains(&Keyword::Protection(Color::Black)));
    let gk = g.computed_permanent(granted).expect("computed").keywords;
    assert!(
        gk.contains(&Keyword::Protection(Color::Black)),
        "CR 612.3 — a granted keyword isn't text and survives the rewrite"
    );
}

/// CR 400.3 — a stolen creature that dies goes to its *owner's* graveyard,
/// not the thief's.
#[test]
fn cr_400_3_stolen_creature_dies_to_its_owners_graveyard() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().controller = 0;
    let mut events = Vec::new();
    g.destroy_permanent(bear, false, &mut events);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "owner's graveyard");
    assert!(g.players[0].graveyard.is_empty(), "not the thief's");
}

/// CR 400.4a — an instant card that would be put onto the battlefield stays
/// in its previous zone.
#[test]
fn cr_400_4a_instant_cant_enter_the_battlefield() {
    let mut g = main_phase();
    let bolt = g.next_id();
    g.players[0]
        .graveyard
        .push(CardInstance::new(bolt, catalog::lightning_bolt(), 0));
    let mut events = Vec::new();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.move_card_to(
        bolt,
        &crabomination::effect::ZoneDest::Battlefield {
            controller: crabomination::effect::PlayerRef::Seat(0),
            tapped: false,
        },
        &ctx,
        &mut events,
    );
    assert!(g.battlefield_find(bolt).is_none(), "instant never enters");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt), "stays in the graveyard");
}

/// CR 400.7 — a permanent that leaves and returns is a new object: its
/// counters and any continuous effect keyed to it are gone.
#[test]
fn cr_400_7_returning_permanent_is_a_new_object() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear)
        .unwrap()
        .counters
        .insert(CounterType::PlusOnePlusOne, 2);
    install(&mut g, bear, Modification::ModifyPowerToughness(3, 3), Layer::L7PowerTough);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 7);
    let mut events = Vec::new();
    g.destroy_permanent(bear, false, &mut events);
    let pos = g.players[0].graveyard.iter().position(|c| c.id == bear).expect("in graveyard");
    let card = g.players[0].graveyard.remove(pos);
    g.battlefield.push(card);
    let cp = g.computed_permanent(bear).expect("computed");
    assert_eq!(cp.power, 2, "the pump doesn't follow the new object");
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        0,
        "counters don't follow either"
    );
}

/// CR 708.2b — turning an already-face-down permanent face down does nothing;
/// the real card stashed behind the 2/2 survives.
#[test]
fn cr_708_2b_face_down_permanent_cant_be_turned_face_down_again() {
    let mut g = main_phase();
    let ogre = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let card = g.battlefield_find_mut(ogre).unwrap();
    card.turn_face_down();
    assert_eq!(card.definition.power, 2);
    card.turn_face_down();
    assert_eq!(
        card.face_up_def.as_ref().map(|d| d.name),
        Some("Grizzly Bears"),
        "the second flip must not overwrite the stash with the 2/2 body"
    );
}

/// CR 708.8 — turning a permanent face up reverts its copiable values but
/// keeps effects already applied to the face-down permanent.
#[test]
fn cr_708_8_turning_face_up_keeps_effects_applied_while_face_down() {
    let mut g = main_phase();
    let gargaroth = g.add_card_to_battlefield(0, catalog::elder_gargaroth());
    let card = g.battlefield_find_mut(gargaroth).unwrap();
    card.turn_face_down();
    card.counters.insert(CounterType::PlusOnePlusOne, 1);
    assert_eq!(g.computed_permanent(gargaroth).unwrap().power, 3, "2/2 + counter");
    g.battlefield_find_mut(gargaroth).unwrap().turn_face_up();
    let cp = g.computed_permanent(gargaroth).expect("computed");
    assert_eq!(cp.power, 7, "6/6 base plus the counter it kept");
    assert!(cp.keywords.contains(&Keyword::Vigilance), "printed abilities are back");
}

/// CR 708.2a / 604.3 — Ixidron turns the other nontoken creatures into
/// vanilla 2/2s and sizes itself off the resulting face-down count.
#[test]
fn cr_708_2a_turned_face_down_creatures_become_vanilla_two_twos() {
    let mut g = main_phase();
    let gargaroth = g.add_card_to_battlefield(0, catalog::elder_gargaroth());
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel());
    let ixidron = g.add_card_to_hand(0, catalog::ixidron());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: ixidron,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    for id in [gargaroth, foe] {
        let cp = g.computed_permanent(id).expect("computed");
        assert_eq!((cp.power, cp.toughness), (2, 2), "vanilla 2/2 body");
        assert!(cp.keywords.is_empty(), "no abilities while face down");
    }
    // Ixidron itself isn't face down, so its CDA reads the two it flipped.
    let cp = g.computed_permanent(ixidron).expect("computed");
    assert_eq!((cp.power, cp.toughness), (2, 2), "two face-down creatures");
}
