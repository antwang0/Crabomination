//! CR 608.2d — "return a [permanent] you control to its owner's hand" is a
//! choice made as the effect resolves, not a target
//! (`Effect::ReturnOneYouControl`). Whitemane Lion, Stonecloaker, Kor
//! Skyfisher, Cavern Harpy, Guildless Commons, Species Gorger, Zell Dincht
//! and Time Wipe declared a target, so the ability could fizzle and the bot
//! named the card when the trigger went on the stack.

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::game::types::{GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g
}

fn cast(g: &mut GameState, id: CardId) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn in_hand(g: &GameState, seat: usize, id: CardId) -> bool {
    g.players[seat].hand.iter().any(|c| c.id == id)
}

#[test]
fn cr_608_2d_return_a_creature_you_control_is_not_a_target() {
    for def in [
        catalog::whitemane_lion(),
        catalog::stonecloaker(),
        catalog::kor_skyfisher(),
        catalog::cavern_harpy(),
        catalog::guildless_commons(),
        catalog::species_gorger(),
        catalog::zell_dincht(),
    ] {
        let body = def.triggered_abilities.iter().find(|t| {
            matches!(t.effect, crabomination::effect::Effect::ReturnOneYouControl { .. })
        });
        assert!(body.is_some_and(|t| !t.effect.requires_target()), "{}", def.name);
    }
    assert!(!catalog::time_wipe().effect.requires_target());
}

/// Whitemane Lion returns another creature when there is one (the cheapest),
/// and itself when it is alone.
#[test]
fn cr_608_2d_whitemane_lion_picks_on_resolution() {
    let mut g = main_phase();
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let lion = g.add_card_to_hand(0, catalog::whitemane_lion());
    g.players[0].mana_pool.add(Color::White, 2);
    cast(&mut g, lion);
    assert!(in_hand(&g, 0, bears), "the cheapest other creature");
    assert!(g.battlefield_find(angel).is_some() && g.battlefield_find(lion).is_some());

    let mut g = main_phase();
    let lion = g.add_card_to_hand(0, catalog::whitemane_lion());
    g.players[0].mana_pool.add(Color::White, 2);
    cast(&mut g, lion);
    assert!(in_hand(&g, 0, lion), "alone, it returns itself");
}

/// Guildless Commons returns a tapped land, never itself while another land
/// is there to bounce.
#[test]
fn cr_608_2d_guildless_commons_keeps_itself() {
    let mut g = main_phase();
    let island = g.add_card_to_battlefield(0, catalog::island());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(forest).unwrap().tapped = true;
    let commons = g.add_card_to_hand(0, catalog::guildless_commons());
    g.perform_action(GameAction::PlayLand(commons)).expect("land drop");
    drain_stack(&mut g);
    assert!(in_hand(&g, 0, forest), "the tapped land comes back");
    assert!(g.battlefield_find(commons).is_some() && g.battlefield_find(island).is_some());
}

/// Time Wipe saves the best creature you control, then destroys the rest.
#[test]
fn cr_608_2d_time_wipe_saves_the_best_creature() {
    let mut g = main_phase();
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wipe = g.add_card_to_hand(0, catalog::time_wipe());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, wipe);
    assert!(in_hand(&g, 0, angel));
    assert!(g.battlefield_find(bears).is_none() && g.battlefield_find(theirs).is_none());
}

/// CR 605.1a (2026-09-25 text) — an ability whose cost or effect moves a card
/// to or from a library is not a mana ability: Chromatic Sphere's "add one
/// mana of any color, draw a card" goes on the stack (its 2026 ruling), while
/// Mind Stone's plain {C} resolves at once.
#[test]
fn cr_605_1a_a_library_move_is_not_a_mana_ability() {
    let mut g = main_phase();
    let sphere = g.add_card_to_battlefield(0, catalog::chromatic_sphere());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sphere,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("Chromatic Sphere");
    assert_eq!(g.stack.len(), 1, "the ability is on the stack");
    let hand = g.players[0].hand.len();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "it drew on resolution");

    let mut g = main_phase();
    let stone = g.add_card_to_battlefield(0, catalog::mind_stone());
    g.perform_action(GameAction::ActivateAbility {
        card_id: stone,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("Mind Stone");
    assert!(g.stack.is_empty(), "a plain mana ability doesn't use the stack");
}

/// CR 602.2b / 601.2c-h — an ability's targets are chosen before its costs
/// are paid, so a "sacrifice this" cost can't make the source its own
/// graveyard target (Priest of Fell Rites' ruling).
#[test]
fn cr_602_2b_a_sacrificed_source_is_not_its_own_graveyard_target() {
    let mut g = main_phase();
    let priest = g.add_card_to_battlefield(0, catalog::priest_of_fell_rites());
    g.clear_sickness(priest);
    let attempt = g.perform_action(GameAction::ActivateAbility {
        card_id: priest,
        ability_index: 0,
        target: Some(crabomination::game::types::Target::Permanent(priest)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    });
    assert!(attempt.is_err(), "the Priest is on the battlefield as its target is chosen");
    assert!(g.battlefield_find(priest).is_some(), "and no cost was paid");
}

/// Mistbreath Elder: "return another creature you control … If you do, put a
/// +1/+1 counter on this creature" — the other creature is chosen on
/// resolution, and the counter follows only a return.
#[test]
fn cr_608_2d_mistbreath_elder_returns_another_then_grows() {
    let mut g = main_phase();
    let elder = g.add_card_to_battlefield(0, catalog::mistbreath_elder());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(in_hand(&g, 0, bears));
    let elder = g.battlefield_find(elder).unwrap();
    assert_eq!(elder.counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1);
}

// ── CR 709.3b / 709.4b / 715.3b — a half's characteristics on the stack ─────

fn stack_card(g: &GameState, id: CardId) -> crabomination::card::CardInstance {
    g.stack
        .iter()
        .find_map(|si| match si {
            crabomination::game::types::StackItem::Spell { card, .. } if card.id == id => {
                Some((**card).clone())
            }
            _ => None,
        })
        .expect("on the stack")
}

/// CR 715.3b — Stomp on the stack is an instant with only Stomp's
/// characteristics: prowess triggers and "counter target creature spell"
/// can't target it.
#[test]
fn cr_715_3b_an_adventure_spell_is_not_a_creature_spell() {
    use crabomination::card::SelectionRequirement as R;
    let mut g = main_phase();
    let monk = g.add_card_to_battlefield(0, catalog::monastery_swiftspear());
    let giant = g.add_card_to_hand(0, catalog::bonecrusher_giant());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::CastAdventure {
        card_id: giant,
        target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Stomp");
    let spell = stack_card(&g, giant);
    assert!(!g.evaluate_requirement_on_card(&R::Creature, &spell, 1), "not a creature spell");
    assert!(g.evaluate_requirement_on_card(&R::ManaValueAtMost(2), &spell, 1), "MV 2, Stomp's");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(monk).unwrap().power(), 2, "prowess saw a noncreature spell");
}

/// CR 709.3b / 709.4b — Ice on the stack is a blue MV-2 spell; Fire // Ice in
/// any other zone has the combined cost {1}{R}{1}{U}: red and blue, MV 4.
#[test]
fn cr_709_split_card_characteristics_by_zone() {
    use crabomination::card::SelectionRequirement as R;
    let mut g = main_phase();
    let fi = g.add_card_to_hand(0, catalog::fire_ice());
    let in_hand = g.players[0].hand.iter().find(|c| c.id == fi).unwrap().clone();
    assert!(!g.evaluate_requirement_on_card(&R::ManaValueAtMost(3), &in_hand, 0), "MV 4 in hand");
    assert!(g.evaluate_requirement_on_card(&R::HasColor(Color::Blue), &in_hand, 0), "blue in hand");
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSplitRight {
        card_id: fi,
        target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Ice");
    let spell = stack_card(&g, fi);
    assert!(g.evaluate_requirement_on_card(&R::HasColor(Color::Blue), &spell, 1), "Ice is blue");
    assert!(!g.evaluate_requirement_on_card(&R::HasColor(Color::Red), &spell, 1), "not red");
    assert!(g.evaluate_requirement_on_card(&R::ManaValueAtMost(2), &spell, 1), "MV 2 on the stack");
}

/// CR 709.4b / 702.85a — cascade from a four-drop skips Fire // Ice: in the
/// library its mana value is 4, both halves', not Fire's 2.
#[test]
fn cr_709_4b_cascade_reads_a_split_cards_combined_mana_value() {
    let mut g = main_phase();
    let bears = g.next_id();
    g.players[0].add_to_library_top(bears, catalog::grizzly_bears());
    let fi = g.next_id();
    g.players[0].add_to_library_top(fi, catalog::fire_ice());
    let elf = g.add_card_to_hand(0, catalog::bloodbraid_elf());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, elf);
    assert!(g.players[0].library.iter().any(|c| c.id == fi), "Fire // Ice went to the bottom");
    assert!(g.players[0].library.iter().all(|c| c.id != bears), "cascade hit the Bears");
}

/// CR 702.103b — a bestowed spell is an Aura enchantment spell, not a creature
/// spell: prowess sees a noncreature spell and "creature spell" filters miss.
#[test]
fn cr_702_103b_a_bestowed_spell_is_not_a_creature_spell() {
    use crabomination::card::SelectionRequirement as R;
    let mut g = main_phase();
    let monk = g.add_card_to_battlefield(0, catalog::monastery_swiftspear());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let satyr = g.add_card_to_hand(0, catalog::boon_satyr());
    g.players[0].mana_pool.add(Color::Green, 5);
    g.perform_action(GameAction::CastBestow {
        card_id: satyr,
        target: Some(crabomination::game::types::Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bestow");
    let spell = stack_card(&g, satyr);
    assert!(!g.evaluate_requirement_on_card(&R::Creature, &spell, 1), "not a creature spell");
    assert!(g.evaluate_requirement_on_card(&R::Enchantment, &spell, 1), "an Aura spell");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(monk).unwrap().power(), 2, "prowess saw a noncreature spell");
}

