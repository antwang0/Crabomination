//! CR 205.3 / 702.73a — a permanent's creature types are its current ones: a
//! changeling is every creature type, and an animated land is a creature of
//! the types its animation gives. Battlefield walks that read the printed
//! type line got both wrong (`GameState::permanent_has_creature_type`).

use crabomination::card::{CreatureType, CardDefinition};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;
use crabomination::TurnStep;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g
}

fn activate(g: &mut GameState, id: crabomination::card::CardId, index: usize, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: index, target, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Crippling Fear naming Elf: a changeling keeps its size, an animated Treetop
/// Village (a creature that isn't an Elf) shrinks — the printed read had both
/// backwards (the Village isn't a printed creature, the Outcast isn't a
/// printed Elf).
#[test]
fn cr_702_73a_crippling_fear_reads_current_types() {
    let mut g = main_phase();
    let outcast = g.add_card_to_battlefield(1, catalog::changeling_outcast());
    let village = g.add_card_to_battlefield(1, catalog::treetop_village());
    g.clear_sickness(village);
    g.battlefield_find_mut(village).unwrap().controller = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, village, 1, None);
    g.battlefield_find_mut(village).unwrap().controller = 1;
    let fear = g.add_card_to_hand(0, catalog::crippling_fear());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Elf)]));
    g.perform_action(GameAction::CastSpell { card_id: fear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(outcast).is_some(), "a changeling is an Elf");
    assert!(g.battlefield_find(village).is_none(), "the 3/3 Ape took -3/-3");
}

/// Elvish Guidance ("an additional {G} for each Elf on the battlefield") and
/// Sliver Legion (+1/+1 per other Sliver) count a changeling.
#[test]
fn cr_702_73a_type_counts_include_changelings() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let guidance = g.add_card_to_battlefield(0, catalog::elvish_guidance());
    g.battlefield_find_mut(guidance).unwrap().attached_to = Some(land);
    g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.add_card_to_battlefield(1, catalog::changeling_outcast());
    activate(&mut g, land, 0, None);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 3, "{{G}} + Elves + the changeling");

    let mut g = main_phase();
    let legion: CardDefinition = catalog::sliver_legion();
    let legion = g.add_card_to_battlefield(0, legion);
    g.add_card_to_battlefield(1, catalog::changeling_outcast());
    assert_eq!(g.computed_permanent(legion).unwrap().power, 8, "7/7 + one other Sliver");
}

/// CR 613.1d — "is a creature" on the battlefield is the layer view: Living
/// Death's "sacrifices all creatures they control" takes an animated Treetop
/// Village (the walk read the printed type line and left it).
#[test]
fn cr_613_1d_living_death_sacrifices_an_animated_land() {
    let mut g = main_phase();
    let village = g.add_card_to_battlefield(0, catalog::treetop_village());
    g.clear_sickness(village);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, village, 1, None);
    let ld = g.add_card_to_hand(0, catalog::living_death());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell { card_id: ld, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(village).is_none(), "the 3/3 Ape was a creature");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == village));
}

/// CR 702.14c / 305.7 — landwalk reads the defender's current land types:
/// Urborg makes their Forest a Swamp, so Bog Wraith's swampwalk makes it
/// unblockable (the check read the printed type line).
#[test]
fn cr_702_14c_swampwalk_sees_urborg() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::urborg_tomb_of_yawgmoth());
    let wraith = g.add_card_to_battlefield(0, catalog::bog_wraith());
    g.clear_sickness(wraith);
    g.add_card_to_battlefield(1, catalog::forest());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: wraith, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(bear, wraith)])).is_err(), "the Forest is a Swamp");
}

/// CR 105.2 — "colors among permanents you control" are current colours: an
/// animated Treetop Village is a green Ape, so Shimmercreep (black) drains 2,
/// not 1 (the count read the Village's printed, colourless line).
#[test]
fn cr_105_2_vivid_counts_an_animated_lands_colour() {
    let mut g = main_phase();
    let village = g.add_card_to_battlefield(0, catalog::treetop_village());
    g.clear_sickness(village);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, village, 1, None);
    let opp = g.players[1].life;
    g.move_card_to_battlefield_for_test(0, catalog::shimmercreep());
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 2, "black + green");
}

/// CR 613.1d / 702.73a — Shared Animosity counts an animated Mutavault (a
/// creature with every creature type while animated) as an attacker sharing
/// the Elf's type: the walk read the printed, noncreature line.
#[test]
fn cr_613_1d_shared_animosity_counts_an_animated_mutavault() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::shared_animosity());
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let vault = g.add_card_to_battlefield(0, catalog::mutavault());
    g.clear_sickness(elf);
    g.clear_sickness(vault);
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, vault, 1, None);
    g.step = TurnStep::DeclareAttackers;
    let at = |attacker| Attack { attacker, target: AttackTarget::Player(1) };
    g.perform_action(GameAction::DeclareAttackers(vec![at(elf), at(vault)])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(elf).unwrap().power, 2, "1 + the other attacking Elf-typed creature");
}

/// CR 613.1d — "an artifact" on the battlefield is the layer view: a Mutavault
/// made an artifact by Liquimetal Coating is one to Tezzeret, Betrayer of
/// Flesh, so its {1} animation costs nothing (the discount read the printed,
/// non-artifact line).
#[test]
fn cr_613_1d_liquimetal_coated_land_is_an_artifact_to_tezzeret() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::tezzeret_betrayer_of_flesh());
    let coating = g.add_card_to_battlefield(0, catalog::liquimetal_coating());
    let vault = g.add_card_to_battlefield(0, catalog::mutavault());
    // The Coating's effect, not its activation: activating the Coating (an
    // artifact) would itself spend Tezzeret's once-a-turn discount.
    let coat = catalog::liquimetal_coating().activated_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(coating, 0, Some(Target::Permanent(vault)), 0);
    g.resolve_effect(&coat, &ctx).unwrap();
    // Tapped, so it can't pay its own {1}: only the discount covers it.
    g.battlefield_find_mut(vault).unwrap().tapped = true;
    activate(&mut g, vault, 1, None);
    assert!(g.computed_permanent(vault).unwrap().card_types().contains(&crabomination::card::CardType::Creature));
}

/// Two abilities of one permanent pushed by hand, a 1-life and a 5-life gain;
/// returns (source, lower ability id, upper ability id).
fn two_abilities_of_one_source(g: &mut GameState) -> (crabomination::card::CardId, crabomination::card::CardId, crabomination::card::CardId) {
    use crabomination::effect::{Effect, Selector, Value};
    use crabomination::game::types::TriggerPush;
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let gain = |n| Effect::GainLife { who: Selector::You, amount: Value::Const(n) };
    g.push_stack(TriggerPush::new(src, 0, gain(1)).build());
    g.push_stack(TriggerPush::new(src, 0, gain(5)).build());
    let ids: Vec<_> = g.stack.iter().filter_map(|si| match si {
        StackItem::Trigger { ability_id, .. } => Some(crabomination::card::CardId(*ability_id)),
        _ => None,
    }).collect();
    assert_ne!(ids[0], ids[1], "each ability is its own object");
    (src, ids[0], ids[1])
}

use crabomination::game::types::StackItem;

/// CR 115.1 / 113.7 — "copy target triggered ability" names one ability, not
/// its source: of two abilities of one permanent on the stack, Strionic
/// Resonator copies the LOWER one when that is the one targeted (the old
/// source-addressed target always took the topmost). The source permanent
/// itself is no longer a legal target.
#[test]
fn cr_115_1_strionic_resonator_copies_the_targeted_one_of_two_abilities() {
    let mut g = main_phase();
    let resonator = g.add_card_to_battlefield(0, catalog::strionic_resonator());
    let (src, lower, upper) = two_abilities_of_one_source(&mut g);
    g.players[0].mana_pool.add_colorless(2);
    let legal = g.legal_targets_for_filter(
        &crabomination::card::SelectionRequirement::HasTriggeredAbilityOnStack
            .and(crabomination::card::SelectionRequirement::ControlledByYou),
        false,
        0,
        Some(resonator),
    );
    assert_eq!(legal, vec![Target::Permanent(upper), Target::Permanent(lower)], "both abilities, topmost first");
    let by_source = GameAction::ActivateAbility {
        card_id: resonator, ability_index: 0, target: Some(Target::Permanent(src)),
        additional_targets: vec![], x_value: None, mode: None,
    };
    assert!(g.perform_action(by_source).is_err(), "the source is not an ability");
    let life = g.players[0].life;
    activate(&mut g, resonator, 0, Some(Target::Permanent(lower)));
    assert_eq!(g.players[0].life, life + 1 + 5 + 1, "the 1-life ability was copied");
}

/// CR 115.1 — Stifle counters the targeted ability; the other ability of the
/// same source still resolves.
#[test]
fn cr_115_1_stifle_counters_the_targeted_one_of_two_abilities() {
    let mut g = main_phase();
    let (_, lower, _) = two_abilities_of_one_source(&mut g);
    let stifle = g.add_card_to_hand(0, catalog::stifle());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: stifle, target: Some(Target::Permanent(lower)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Stifle the lower ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 5, "only the 5-life ability resolved");
}

/// CR 707.10 — a copy of an ability is a new object with its own id, so a
/// second copier can't confuse it with the original.
#[test]
fn cr_707_10_a_copied_ability_gets_its_own_id() {
    let mut g = main_phase();
    let resonator = g.add_card_to_battlefield(0, catalog::strionic_resonator());
    let (_, _, upper) = two_abilities_of_one_source(&mut g);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: resonator, ability_index: 0, target: Some(Target::Permanent(upper)),
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("activate");
    // Resolve only the Resonator's ability: the copy lands on top.
    g.perform_action(GameAction::PassPriority).unwrap();
    g.perform_action(GameAction::PassPriority).unwrap();
    let ids: Vec<u32> = g.stack.iter().filter_map(|si| match si {
        StackItem::Trigger { ability_id, .. } => Some(*ability_id),
        _ => None,
    }).collect();
    assert_eq!(ids.len(), 3, "two originals and the copy: {ids:?}");
    let mut uniq = ids.clone();
    uniq.sort_unstable();
    uniq.dedup();
    assert_eq!(uniq.len(), 3, "every ability on the stack has its own id");
}

/// CR 303.4f — an Aura card a plain "return it to the battlefield" puts there
/// enters attached to something it can enchant (its controller's choice), not
/// unattached for the state-based check to bin (CR 704.5m). Rise to Glory's
/// Aura mode returns Wild Growth onto a land; with nothing to enchant it
/// stays in the graveyard (CR 303.4i).
#[test]
fn cr_303_4f_an_aura_returned_to_the_battlefield_enters_attached() {
    let mut g = main_phase();
    let growth = g.add_card_to_graveyard(0, catalog::wild_growth());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(growth)), 0, 0);
    let mode = match &catalog::rise_to_glory().effect {
        crabomination::effect::Effect::ChooseModesCast { modes, .. } => modes[1].clone(),
        other => panic!("{other:?}"),
    };
    g.resolve_effect(&mode, &ctx).expect("no land: stays");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == growth), "nothing to enchant: it stays put");
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.resolve_effect(&mode, &ctx).expect("return");
    g.check_state_based_actions();
    assert_eq!(g.battlefield_find(growth).and_then(|c| c.attached_to), Some(forest));
}

/// CR 110.2 — a permanent enters under the control the effect names: a
/// `ZoneDest` naming a TARGETED player puts it under that player even though
/// the placement runs in a context without the caster's targets. It used to
/// pass `PlayerRef::Target(0)` through unresolved, so the card landed under
/// whoever the bare context read as slot 0 (The Beamtown Bullies at three
/// seats gave the creature to its own controller).
#[test]
fn cr_110_2_a_zone_dest_naming_a_target_player_resolves_it() {
    use crabomination::effect::{Effect, PlayerRef, Selector, ZoneDest};
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    let wurm = g.add_card_to_graveyard(0, catalog::craw_wurm());
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ctx = crabomination::game::effects::EffectContext::for_ability(src, 0, None);
    ctx.targets = vec![Target::Player(2), Target::Permanent(wurm)];
    let effect = Effect::Move {
        what: Selector::Target(1),
        to: ZoneDest::Battlefield { controller: PlayerRef::Target(0), tapped: false },
    };
    g.resolve_effect(&effect, &ctx).expect("move");
    assert_eq!(g.battlefield_find(wurm).map(|c| c.controller), Some(2));
}

/// CR 611.2a — a continuous effect from a resolved ability lasts for its
/// stated duration even if its source leaves: a creature's "until end of
/// turn" base-P/T change and an Island granted "for as long as it has a
/// flood counter" both outlive the creature.
#[test]
fn cr_611_2a_resolved_effects_outlive_their_source() {
    use crabomination::card::{CounterType, LandType};
    use crabomination::effect::{Duration, Effect, Selector, Value};
    let mut g = crabomination::game::two_player_game();
    let src = g.add_card_to_battlefield(0, crabomination::catalog::grizzly_bears());
    let wurm = g.add_card_to_battlefield(1, crabomination::catalog::craw_wurm());
    let land = g.add_card_to_battlefield(0, crabomination::catalog::forest());
    g.battlefield_find_mut(land).unwrap().add_counters(CounterType::Flood, 1);
    let run = |g: &mut crabomination::game::GameState, effect: Effect, t: crabomination::card::CardId| {
        let target = crabomination::game::types::Target::Permanent(t);
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(src, 0, Some(target.clone()));
        ctx.targets = vec![target];
        g.resolve_effect(&effect, &ctx).expect("resolves");
    };
    run(&mut g, Effect::SetBasePT { what: Selector::Target(0), power: Value::Const(0), toughness: Value::Const(1), duration: Duration::EndOfTurn }, wurm);
    run(
        &mut g,
        Effect::GainLandType { what: Selector::Target(0), land_type: LandType::Island, duration: Duration::WhileHasCounter(CounterType::Flood) },
        land,
    );
    g.destroy_permanent(src, false, &mut Vec::new());
    g.check_state_based_actions();
    assert_eq!(g.computed_permanent(wurm).unwrap().power, 0, "the base 0/1 lasts until end of turn");
    assert!(g.computed_permanent(land).unwrap().subtypes().land_types.contains(&LandType::Island));
}

/// CR 611.2c — a resolved "creatures you control get +3/+3" locks its
/// affected set as it resolves: a creature entering later isn't pumped.
#[test]
fn cr_611_2c_a_resolved_pump_skips_later_creatures() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ov = g.add_card_to_hand(0, catalog::overrun());
    g.players[0].mana_pool.add(Color::Green, 3);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell { card_id: ov, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    let late = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bear).unwrap().power, 5);
    assert_eq!(g.computed_permanent(late).unwrap().power, 2, "entered after Overrun resolved");
    assert!(!g.computed_permanent(late).unwrap().keywords().contains(&crabomination::card::Keyword::Trample));
}

/// CR 119.9 / 120.3f — two lifelinkers' combat damage is dealt at once, but
/// each source causes its own life gain: Ajani's Pridemate triggers twice.
#[test]
fn cr_119_9_each_lifelink_source_is_its_own_life_gain() {
    use crabomination::card::CounterType;
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = main_phase();
    let cat = g.add_card_to_battlefield(0, catalog::ajanis_pridemate());
    let a = g.add_card_to_battlefield(0, catalog::vampire_nighthawk());
    let b = g.add_card_to_battlefield(0, catalog::vampire_nighthawk());
    g.clear_sickness(a);
    g.clear_sickness(b);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ])
    .expect("attack");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 24);
    assert_eq!(g.battlefield_find(cat).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// CR 702.73a / 613.8 — a creature GIVEN changeling by a static (Maskwood
/// Nexus) is every creature type inside the layer pass too: a Goblin lord's
/// type-filtered anthem reaches it.
#[test]
fn cr_702_73a_granted_changeling_is_every_creature_type() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::maskwood_nexus());
    g.add_card_to_battlefield(0, catalog::goblin_king());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "Goblin King pumps the Nexus-made Goblin");
}

/// CR 613.8 — a color-filtered anthem reads colors after layer 5: Honor of the
/// Pure pumps a creature made white and stops pumping a white one made red.
#[test]
fn cr_613_8_a_color_anthem_reads_layer_5_colors() {
    use crabomination::effect::{Duration, Effect, Selector};
    use crabomination::mana::Color;
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::honor_of_the_pure());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let knight = g.add_card_to_battlefield(0, catalog::white_knight());
    let paint = |g: &mut GameState, id, color| {
        let t = Target::Permanent(id);
        let mut ctx = crabomination::game::effects::EffectContext::for_ability(id, 0, Some(t.clone()));
        ctx.targets = vec![t];
        let e = Effect::BecomeColor { what: Selector::Target(0), colors: vec![color], duration: Duration::EndOfTurn, additive: false };
        g.resolve_effect(&e, &ctx).expect("resolves");
    };
    assert_eq!(g.computed_permanent(knight).unwrap().power, 3);
    paint(&mut g, bear, Color::White);
    paint(&mut g, knight, Color::Red);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "made white");
    assert_eq!(g.computed_permanent(knight).unwrap().power, 2, "made red");
}

/// CR 613.8 — a keyword-filtered anthem reads layer-6 abilities: Favorable
/// Winds pumps a creature Levitation gives flying.
#[test]
fn cr_613_8_a_keyword_anthem_reads_granted_keywords() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::favorable_winds());
    g.add_card_to_battlefield(0, catalog::levitation());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&crabomination::card::Keyword::Flying));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "Levitation's flying meets Favorable Winds");
}

/// CR 603.2c / 120.3 — one damage-dealing sentence is one damage event per
/// source: Pestilence Demon's "1 damage to each creature and each player"
/// fires Spirit Link's recipient-agnostic "whenever enchanted creature deals
/// damage" ONCE, for the total, not once per recipient.
#[test]
fn cr_603_2c_a_noncombat_sweep_fires_deals_damage_once() {
    let mut g = main_phase();
    let demon = g.add_card_to_battlefield(0, catalog::pestilence_demon());
    let link = g.add_card_to_battlefield(0, catalog::spirit_link());
    g.battlefield_find_mut(link).unwrap().attached_to = Some(demon);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let sweep = g.battlefield_find(demon).unwrap().definition.activated_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_ability(demon, 0, None);
    g.resolve_effect(&sweep, &ctx).expect("resolves");
    let fires: Vec<u32> = g
        .stack
        .iter()
        .filter_map(|si| match si {
            crabomination::game::types::StackItem::Trigger { source, event_amount, .. } if *source == link => {
                Some(*event_amount)
            }
            _ => None,
        })
        .collect();
    assert_eq!(fires, vec![5], "three creatures and two players: one fire for 5");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 24, "19 after the ping, +5");
}

/// CR 603.2 — an ability triggers when its event happens, not when the
/// resolution ends: Martial Coup (X = 5) makes five Soldiers and THEN
/// destroys every other creature, so Soul Warden, on the battlefield as each
/// Soldier entered, gains 1 life five times though it is dead by dispatch.
#[test]
fn cr_603_2_a_listener_the_resolution_removes_saw_the_entries_before() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::soul_warden());
    let coup = g.add_card_to_hand(0, catalog::martial_coup());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: coup,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(5),
    })
    .expect("cast Martial Coup for X=5");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Soul Warden"), "the sweep took the Warden");
    assert_eq!(g.players[0].life, 25, "one life per Soldier that entered before the sweep");
}

/// CR 704.3 — state-based actions aren't checked part-way through a
/// resolution: an Aura whose host a spell destroys is still on the
/// battlefield until the spell is done, then goes in the sweep, and its own
/// "put into a graveyard" trigger still fires (Reach for the Sky draws).
#[test]
fn cr_704_3_an_orphaned_aura_goes_in_the_sweep_after_the_resolution() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::reach_for_the_sky());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Reach for the Sky");
    drain_stack(&mut g);
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
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
    .expect("cast Doom Blade");
    drain_stack(&mut g);
    assert!(g.battlefield_find(aura).is_none(), "swept once the spell resolved");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == aura));
    assert_eq!(g.players[0].hand.len(), hand + 1, "the Aura's trigger drew one");
}

/// CR 603.2 — the same for a "whenever you gain life" listener: a resolution
/// that gains life and then destroys every creature still triggers Marauding
/// Blight-Priest, which was on the battlefield when the life came in.
#[test]
fn cr_603_2_a_departed_life_gain_listener_saw_the_gain() {
    use crabomination::card::SelectionRequirement;
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = main_phase();
    let priest = g.add_card_to_battlefield(0, catalog::marauding_blight_priest());
    let seq = Effect::Seq(vec![
        Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
        Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
        },
    ]);
    let ctx = crabomination::game::effects::EffectContext::for_ability(priest, 0, None);
    let events = g.resolve_effect(&seq, &ctx).expect("resolves");
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(priest).is_none(), "the sweep took the priest");
    assert_eq!(g.players[0].life, 23);
    assert_eq!(g.players[1].life, 19, "the priest saw the gain before it died");
}

/// CR 111.8 / 700.11 — a token that dies is no card: it doesn't count as a
/// card put into your graveyard this turn and doesn't descend. A creature
/// card dying does both.
#[test]
fn cr_111_8_a_dying_token_is_no_card_put_into_a_graveyard() {
    use crabomination::effect::{Effect, Selector};
    let mut g = main_phase();
    let tok = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(tok).unwrap().is_token = true;
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&Effect::Destroy { what: Selector::ExactObjects(vec![tok]) }, &ctx).unwrap();
    assert!(!g.players[0].descended_this_turn, "a token doesn't descend");
    assert_eq!(g.players[0].cards_to_graveyard_this_turn, 0);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.resolve_effect(&Effect::Destroy { what: Selector::ExactObjects(vec![bear]) }, &ctx).unwrap();
    assert!(g.players[0].descended_this_turn, "a permanent card does");
    assert_eq!(g.players[0].cards_to_graveyard_this_turn, 1);
}
