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

/// CR 115.1 / 603.7 — a delayed trigger's captured object is not a target: the
/// body's slot 0 names what the capture recorded. Nahiri, the Harbinger's −8
/// ("search … it gains haste; return it to your hand at the beginning of the
/// next end step") declares no target, so it activates with none.
#[test]
fn cr_115_1_a_captured_delayed_body_declares_no_target() {
    use crabomination::card::CounterType;
    let mut g = main_phase();
    let nahiri = g.add_card_to_battlefield(0, catalog::nahiri_the_harbinger());
    g.battlefield_find_mut(nahiri).unwrap().counters.insert(CounterType::Loyalty, 8);
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: nahiri,
        ability_index: 2,
        target: None,
        x_value: None,
    })
    .expect("the −8 has no target");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_some(), "fetched onto the battlefield");
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(in_hand(&g, 0, bears), "returned to hand at the next end step");
}

/// CR 202.3e — on the stack a spell's mana value counts the X it was cast
/// for: Blaze for 5 is a 6 to "target spell with mana value N" filters, in
/// both requirement walkers. In the hand it is a 1.
#[test]
fn cr_202_3e_a_stack_x_spell_counts_its_x() {
    use crabomination::card::SelectionRequirement as R;
    use crabomination::game::types::Target;
    let mut g = main_phase();
    let blaze = g.add_card_to_hand(0, catalog::blaze());
    let in_hand = g.players[0].hand.iter().find(|c| c.id == blaze).unwrap().clone();
    assert!(g.evaluate_requirement_on_card(&R::ManaValueAtMost(1), &in_hand, 0), "MV 1 in hand");
    g.players[0].mana_pool.add(Color::Red, 6);
    g.perform_action(GameAction::CastSpell {
        card_id: blaze,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(5),
    })
    .expect("Blaze for 5");
    let spell = stack_card(&g, blaze);
    let at_most_5 = R::ManaValueAtMost(5);
    assert!(!g.evaluate_requirement_on_card(&at_most_5, &spell, 1), "card walker: MV 6");
    assert!(
        !g.evaluate_requirement_static(&at_most_5, &Target::Permanent(blaze), 1, None),
        "target walker: MV 6"
    );
    assert!(g.evaluate_requirement_static(&R::ManaValueAtMost(6), &Target::Permanent(blaze), 1, None));
}

/// CR 202.3e — "that spell's mana value" read after it is countered counts
/// its X: Mana Drain on Blaze for 5 banks six.
#[test]
fn cr_202_3e_a_countered_x_spells_mana_value_counts_its_x() {
    use crabomination::game::types::Target;
    let mut g = main_phase();
    let blaze = g.add_card_to_hand(0, catalog::blaze());
    g.players[0].mana_pool.add(Color::Red, 6);
    g.perform_action(GameAction::CastSpell {
        card_id: blaze,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(5),
    })
    .expect("Blaze for 5");
    let drain = g.add_card_to_hand(1, catalog::mana_drain());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: drain,
        target: Some(Target::Permanent(blaze)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Mana Drain");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == blaze), "countered");
    assert_eq!(g.countered_spell_mana_value, 6, "X counts");
}

/// CR 707.2 / 614.1c — Vesuva "enters tapped as a copy of any land": an
/// as-enters choice (no target, no trigger to respond to), made as it enters.
#[test]
fn cr_707_2_vesuva_enters_as_a_copy() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::breeding_pool());
    let vesuva = g.add_card_to_hand(0, catalog::vesuva());
    g.perform_action(GameAction::PlayLand(vesuva)).expect("land drop");
    assert!(g.stack.is_empty(), "no trigger");
    let v = g.battlefield_find(vesuva).expect("on the battlefield");
    assert_eq!(v.definition.name, "Breeding Pool");
    assert!(v.tapped);
}

/// CR 605.3b / 603.3 — a mana ability's cost is still an event: sacrificing
/// a Treasure for mana triggers Disciple of the Vault, and the trigger goes
/// on the stack the next time a player would receive priority.
#[test]
fn cr_605_3b_a_mana_abilitys_sacrifice_triggers() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::disciple_of_the_vault());
    let tok = crabomination::game::effects::EffectContext::for_ability(CardId(0), 0, None);
    g.resolve_effect(
        &crabomination::effect::Effect::CreateToken {
            who: crabomination::effect::PlayerRef::You,
            count: crabomination::card::Value::ONE,
            definition: std::sync::Arc::new(crabomination::game::effects::treasure_token()),
        },
        &tok,
    )
    .expect("a Treasure");
    let t = g.battlefield.iter().find(|c| c.definition.name == "Treasure").unwrap().id;
    g.perform_action(GameAction::ActivateAbility {
        card_id: t,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("Treasure for mana");
    assert_eq!(g.players[0].mana_pool.total(), 1);
    assert_eq!(g.stack.len(), 1, "Disciple's trigger waits on the stack");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "Disciple drained off the mana ability's sacrifice");
}

/// CR 603.10 — a "leaves the battlefield" trigger looks back: "whenever an
/// artifact an opponent controls dies" reads the dead artifact's last-known
/// controller. `ControlledByOpponent` (and `…Seat` / `…ActivePlayer`) had no
/// look-back and answered `false` for every permanent already gone.
#[test]
fn cr_603_10_an_opponents_dead_artifact_is_read_by_its_last_controller() {
    use crabomination::card::{
        CardDefinition, CardType, EventKind, EventScope, EventSpec, SelectionRequirement as R, TriggeredAbility,
        Value,
    };
    use crabomination::effect::{Effect, Predicate, Selector};
    let mut g = main_phase();
    g.add_card_to_battlefield(
        0,
        CardDefinition {
            name: "Watcher",
            card_types: vec![CardType::Enchantment],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentDied, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Artifact.and(R::ControlledByOpponent),
                    },
                ),
                effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
            }],
            ..Default::default()
        },
    );
    g.add_card_to_battlefield(1, catalog::worn_powerstone());
    let ctx = crabomination::game::effects::EffectContext::for_ability(CardId(0), 1, None);
    let evs = g
        .resolve_effect(
            &Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Artifact },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 21, "the opponent's artifact died");
}

/// CR 400.7 — a flickered permanent is a new object: effects that applied to
/// the old one don't follow it. Giant Growth's +3/+3 is gone, and a creature
/// stolen with Act of Treason and Ephemerated comes back to its owner and
/// stays (Cloudshift would keep it: "under your control").
#[test]
fn cr_400_7_a_flickered_creature_sheds_pumps_and_control() {
    use crabomination::game::types::Target;
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let growth = g.add_card_to_hand(0, catalog::giant_growth());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: growth, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Giant Growth");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 5);
    let shift = g.add_card_to_hand(0, catalog::cloudshift());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: shift, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cloudshift");
    drain_stack(&mut g);
    let back = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").map(|c| c.id).unwrap();
    assert_eq!(g.computed_permanent(back).unwrap().power, 2, "the pump didn't follow it");

    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let treason = g.add_card_to_hand(0, catalog::act_of_treason());
    g.players[0].mana_pool.add(Color::Red, 3);
    g.perform_action(GameAction::CastSpell {
        card_id: treason, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Act of Treason");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0);
    let eph = g.add_card_to_hand(0, catalog::ephemerate());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: eph, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ephemerate the stolen Bears");
    drain_stack(&mut g);
    let back = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").map(|c| c.id).unwrap();
    let cp = g.computed_permanent(back).unwrap();
    assert_eq!(cp.controller, 1, "it returns under its owner's control and stays there");
}

/// CR 709.5 — a Room permanent has only its unlocked doors' mana cost: none
/// unlocked is mana value 0, the right door alone is that door's.
#[test]
fn cr_709_5_a_rooms_mana_value_is_its_unlocked_doors() {
    use crabomination::card::SelectionRequirement as R;
    use crabomination::game::types::Target;
    let mut g = main_phase();
    let room = g.add_card_to_battlefield(0, catalog::bottomless_pool_locker_room());
    g.battlefield_find_mut(room).unwrap().unlocked_doors = 0;
    let mv0 = R::ManaValueAtMost(0);
    assert!(g.evaluate_requirement_static(&mv0, &Target::Permanent(room), 0, None), "no door: MV 0");
    g.battlefield_find_mut(room).unwrap().unlocked_doors = 0b10;
    let right = catalog::bottomless_pool_locker_room().room.unwrap().right.cost.cmc();
    let exact = R::ManaValueAtMost(right).and(R::ManaValueAtMost(right.saturating_sub(1)).negate());
    assert!(g.evaluate_requirement_static(&exact, &Target::Permanent(room), 0, None), "the right door's");
}

/// CR 601.2 / 601.3 — "You may cast this card from exile"
/// (`Keyword::ExileCast`): the owner casts it from face-up exile for its own
/// cost. It's a cast (the spell count moves, it uses the stack), not a cast
/// from hand; another player can't cast it, and a face-down exiled card has
/// no abilities, so it can't be cast this way.
#[test]
fn cr_601_exile_cast_keyword_casts_the_card_from_exile() {
    use crabomination::card::Keyword;
    let exile_bear = || {
        let mut d = catalog::grizzly_bears();
        d.keywords.push(Keyword::ExileCast);
        d
    };
    let mut g = main_phase();
    let bear = g.add_card_to_exile(0, exile_bear());
    let cast_it = |g: &mut GameState, id| {
        g.perform_action(GameAction::CastAdventureCreature {
            card_id: id,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    let (spells, from_hand) = (g.spells_cast_this_turn, g.players[0].spells_cast_from_hand_this_turn);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(g.compute_hand_affordances(0).adventure_exile.contains(&bear), "offered to its owner");
    cast_it(&mut g, bear).expect("cast from exile");
    assert!(
        g.stack.iter().any(|si| matches!(si, StackItem::Spell { card, .. } if card.id == bear)),
        "on the stack"
    );
    assert_eq!(g.spells_cast_this_turn, spells + 1, "a cast");
    assert_eq!(g.players[0].spells_cast_from_hand_this_turn, from_hand, "not from hand");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());

    // Another player can't cast it.
    let theirs = g.add_card_to_exile(1, exile_bear());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(cast_it(&mut g, theirs).is_err(), "only its owner");
    // Face down, it has no abilities.
    let hidden = g.add_card_to_exile(0, exile_bear());
    g.exile.iter_mut().find(|c| c.id == hidden).unwrap().face_down = true;
    assert!(cast_it(&mut g, hidden).is_err(), "face-down exile");
}

/// CR 105.3 / 613.1e — "becomes a 2/2 red and green … creature" sets the
/// animated permanent's colors (layer 5) for the animation's duration
/// (`shortcut::colored_animation`); a colorless land is colorless again once
/// the animation ends.
#[test]
fn cr_105_3_colored_animation_sets_colors_for_its_duration() {
    use crabomination::effect::shortcut::colored_animation;
    use crabomination::effect::Effect;
    let mut def = catalog::mutavault();
    let idx = def
        .activated_abilities
        .iter()
        .position(|a| matches!(a.effect, Effect::BecomeCreature { .. }))
        .expect("Mutavault animates");
    let animate = def.activated_abilities[idx].effect.clone();
    def.activated_abilities[idx].effect = colored_animation(animate, &[Color::Red, Color::Green]);
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, def);
    assert!(g.computed_permanent(land).unwrap().colors.is_empty(), "colorless land");
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: idx,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("animate");
    drain_stack(&mut g);
    let post = g.computed_permanent(land).unwrap();
    assert!(post.card_types().contains(&crabomination::card::CardType::Creature));
    let mut colors = post.colors.to_vec();
    colors.sort_by_key(|c| *c as u8);
    let mut want = vec![Color::Red, Color::Green];
    want.sort_by_key(|c| *c as u8);
    assert_eq!(colors, want, "red and green while animated");
    g.expire_end_of_turn_effects();
    assert!(g.computed_permanent(land).unwrap().colors.is_empty(), "colorless again");
}
