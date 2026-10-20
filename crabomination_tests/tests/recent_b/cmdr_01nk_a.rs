//! Commander: the 2026-10-20 (`01NkAE5B`) one-gap pair closers
//! (`decks::cmdr_01nk_a`) and the per-object locks they need.

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector, Value};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn cast_with(g: &mut GameState, id: CardId, target: Option<Target>, more: Vec<Target>, x: Option<u32>, mode: Option<usize>) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: more, mode, x_value: x })
        .expect("cast");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    cast_with(g, id, target, vec![], None, None);
}

fn counter_on_each_creature(g: &mut GameState) {
    let each = Effect::AddCounter {
        what: Selector::EachPermanent(crabomination::card::SelectionRequirement::Creature),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::ONE,
    };
    g.resolve_effect(&each, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
}

/// CR 122.1 / 614.17 — Melira's Keepers' "this creature can't have counters
/// put on it" locks only the Keepers (it was encoded as Solemnity's global
/// lock, which stopped counters on every permanent), and proliferate skips it.
#[test]
fn cr_614_17_cant_have_counters_is_per_object() {
    let mut g = pod(2);
    let keepers = g.add_card_to_battlefield(0, catalog::meliras_keepers());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    counter_on_each_creature(&mut g);
    assert_eq!(g.battlefield_find(keepers).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    g.resolve_effect(&Effect::Proliferate, &EffectContext::for_spell(1, None, 1, 0)).unwrap();
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(g.battlefield_find(keepers).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// CR 701.26b / 614.17 — Blossombind taps its creature, which then neither
/// untaps in its controller's untap step nor by an effect, and takes no
/// counters; another creature still does.
#[test]
fn cr_701_26b_blossombind_locks_untap_and_counters() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::blossombind());
    cast(&mut g, aura, Some(Target::Permanent(bears)));
    assert!(g.battlefield_find(bears).unwrap().tapped, "entering, it taps the creature");
    counter_on_each_creature(&mut g);
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!(g.battlefield_find(other).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let untap = Effect::Untap { what: Selector::Target(0), up_to: None };
    g.resolve_effect(&untap, &EffectContext::for_spell(1, Some(Target::Permanent(bears)), 1, 0)).unwrap();
    assert!(g.battlefield_find(bears).unwrap().tapped, "an effect can't untap it");
    // Seat 1's untap step.
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(bears).unwrap().tapped, "nor can its untap step");
}

/// CR 615 — Purity turns noncombat damage to its controller into life; combat
/// damage and damage to other players are untouched.
#[test]
fn cr_615_purity_turns_noncombat_damage_into_life() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::purity());
    let life = g.players[0].life;
    let bolt = Effect::DealDamage { to: Selector::Player(crabomination::effect::PlayerRef::EachPlayer), amount: Value::Const(3) };
    g.resolve_effect(&bolt, &EffectContext::for_spell(1, None, 1, 0)).unwrap();
    assert_eq!(g.players[0].life, life + 3, "prevented, and gained");
    assert_eq!(g.players[2].life, life - 3);
}

/// Creeping Trailblazer pumps other Elementals' power and itself per Elemental.
#[test]
fn creeping_trailblazer_lords_and_pumps_per_elemental() {
    let mut g = pod(2);
    let tb = g.add_card_to_battlefield(0, catalog::creeping_trailblazer());
    let purity = g.add_card_to_battlefield(0, catalog::purity());
    assert_eq!(g.computed_permanent(purity).unwrap().power, 7);
    assert_eq!(g.computed_permanent(tb).unwrap().power, 2, "\"other\" Elementals");
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: tb,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .unwrap();
    drain_stack(&mut g);
    let cp = g.computed_permanent(tb).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "two Elementals");
}

/// Mantle of Leadership's creature gets +2/+2 whenever any creature enters.
#[test]
fn mantle_of_leadership_pumps_on_any_creature_entering() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mantle = g.add_card_to_hand(0, catalog::mantle_of_leadership());
    cast(&mut g, mantle, Some(Target::Permanent(bears)));
    let theirs = g.add_card_to_hand(1, catalog::grizzly_bears());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.active_player_idx = 1;
    g.perform_action(GameAction::CastSpell { card_id: theirs, target: None, additional_targets: vec![], mode: None, x_value: None })
        .unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bears).unwrap().power, 4, "an opponent's creature counts");
}

/// Timeline Inquiry discards one unless teamwork was paid.
#[test]
fn timeline_inquiry_discards_unless_teamwork() {
    let mut g = pod(2);
    let ti = g.add_card_to_hand(0, catalog::timeline_inquiry());
    let hand = g.players[0].hand.len();
    cast(&mut g, ti, None);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3 - 1);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().summoning_sick = false;
    let ti = g.add_card_to_hand(0, catalog::timeline_inquiry());
    let hand = g.players[0].hand.len();
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked { card_id: ti, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("teamwork cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3, "teamwork skips the discard");
    assert!(g.battlefield_find(bears).unwrap().tapped);
}

/// CR 603.2 — Stormfront Riders bounces two of your creatures and makes a
/// Soldier for EACH creature returned to your hand (the kind fired once per
/// batch before it fanned out).
#[test]
fn stormfront_riders_bounces_two_and_makes_soldiers() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let riders = g.add_card_to_hand(0, catalog::stormfront_riders());
    cast(&mut g, riders, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == a));
    assert!(g.players[0].hand.iter().any(|c| c.id == b));
    let soldiers = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Soldier").count();
    assert_eq!(soldiers, 2);
}

/// Impelled Giant taps another red creature for +X/+0, X its power.
#[test]
fn impelled_giant_borrows_a_red_creatures_power() {
    let mut g = pod(2);
    let giant = g.add_card_to_battlefield(0, catalog::impelled_giant());
    let other = g.add_card_to_battlefield(0, catalog::impelled_giant());
    g.battlefield_find_mut(other).unwrap().summoning_sick = false;
    g.perform_action(GameAction::ActivateAbility { card_id: giant, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("tap the other giant");
    drain_stack(&mut g);
    assert!(g.battlefield_find(other).unwrap().tapped);
    assert_eq!(g.computed_permanent(giant).unwrap().power, 6);
}

/// Doubling Chant fetches a same-name copy for each creature you control.
#[test]
fn doubling_chant_fetches_a_copy_per_creature() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let chant = g.add_card_to_hand(0, catalog::doubling_chant());
    cast(&mut g, chant, None);
    let bears = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Grizzly Bears").count();
    assert_eq!(bears, 2);
}

/// Dichotomancy steals a same-name copy for each tapped nonland permanent.
#[test]
fn dichotomancy_steals_copies_of_tapped_permanents() {
    let mut g = pod(3);
    let t = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(t).unwrap().tapped = true;
    g.add_card_to_battlefield(1, catalog::hill_giant());
    g.add_card_to_library(1, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::hill_giant());
    let d = g.add_card_to_hand(0, catalog::dichotomancy());
    cast(&mut g, d, Some(Target::Player(1)));
    let mine: Vec<_> = g.battlefield.iter().filter(|c| c.controller == 0).map(|c| c.definition.name).collect();
    assert_eq!(mine, vec!["Grizzly Bears"], "only the tapped one's name");
}

/// Sphere of Annihilation exiles itself, small creatures and graveyard
/// creature cards at your upkeep; bigger ones stay.
#[test]
fn sphere_of_annihilation_exiles_by_void_counters() {
    let mut g = pod(2);
    let sphere = g.add_card_to_hand(0, catalog::sphere_of_annihilation());
    cast_with(&mut g, sphere, None, vec![], Some(2), None);
    assert_eq!(g.battlefield_find(sphere).unwrap().counter_count(CounterType::Void), 2);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep {
        let _ = g.advance_step(Vec::new());
    }
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bears));
    assert!(g.exile.iter().any(|c| c.id == dead));
    assert!(g.exile.iter().any(|c| c.id == sphere));
    assert!(g.battlefield_find(giant).is_some(), "mana value 4 survives");
}

/// Tarnation Vista's second ability adds one mana per colour among
/// monocolored permanents only.
#[test]
fn tarnation_vista_counts_monocolored_permanents() {
    let mut g = pod(2);
    let vista = g.add_card_to_battlefield(0, catalog::tarnation_vista());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::creeping_trailblazer());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility { card_id: vista, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None })
        .unwrap();
    drain_stack(&mut g);
    let pool = &g.players[0].mana_pool;
    assert_eq!(pool.amount(Color::Green), 1, "the green Bears");
    assert_eq!(pool.amount(Color::Red), 0, "the R/G Trailblazer is multicolored");
}

/// Acquisitions Expert reveals as many cards as your party is large and
/// discards the one you choose.
#[test]
fn acquisitions_expert_reveals_party_size() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    let ae = g.add_card_to_hand(0, catalog::acquisitions_expert());
    cast(&mut g, ae, None);
    if !g.stack.is_empty() {
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].hand.len(), 2, "a party of one (the Rogue) reveals one; it's discarded");
}

/// Distended Mindbender's cast trigger takes a small nonland card and a big one.
#[test]
fn distended_mindbender_strips_a_small_and_a_big_card() {
    let mut g = pod(2);
    let small = g.add_card_to_hand(1, catalog::grizzly_bears());
    let big = g.add_card_to_hand(1, catalog::hill_giant());
    let land = g.add_card_to_hand(1, catalog::plains());
    let mb = g.add_card_to_hand(0, catalog::distended_mindbender());
    cast(&mut g, mb, Some(Target::Player(1)));
    let hand: Vec<_> = g.players[1].hand.iter().map(|c| c.id).collect();
    assert_eq!(hand, vec![land], "{small:?} and {big:?} discarded");
}

/// Soul Separator makes a 1/1 flying Spirit copy and a Zombie with the card's
/// power and toughness.
#[test]
fn soul_separator_splits_a_dead_creature() {
    let mut g = pod(2);
    let sep = g.add_card_to_battlefield(0, catalog::soul_separator());
    let dead = g.add_card_to_graveyard(0, catalog::hill_giant());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility { card_id: sep, ability_index: 0, target: Some(Target::Permanent(dead)), additional_targets: vec![], x_value: None, mode: None })
        .unwrap();
    drain_stack(&mut g);
    let copy = g.battlefield.iter().find(|c| c.definition.name == "Hill Giant").expect("the copy").id;
    let cp = g.computed_permanent(copy).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    assert!(cp.keywords().contains(&crabomination::card::Keyword::Flying));
    let zombie = g.battlefield.iter().find(|c| c.definition.name == "Zombie").expect("the zombie").id;
    let zp = g.computed_permanent(zombie).unwrap();
    assert_eq!((zp.power, zp.toughness), (3, 3));
    assert!(g.exile.iter().any(|c| c.id == dead));
}

/// Ajani's Last Stand trades itself for a 4/4 flying Avatar when a creature
/// you control dies.
#[test]
fn ajanis_last_stand_answers_a_death() {
    let mut g = pod(2);
    let stand = g.add_card_to_battlefield(0, catalog::ajanis_last_stand());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    let evs = g
        .resolve_effect(&Effect::Destroy { what: Selector::Target(0) }, &EffectContext::for_spell(1, Some(Target::Permanent(bears)), 1, 0))
        .unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(stand).is_none(), "sacrificed");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Avatar"));
}

/// Veteran's Armaments may jump onto an entering Soldier.
#[test]
fn veterans_armaments_attaches_to_entering_soldiers() {
    let mut g = pod(2);
    let arm = g.add_card_to_battlefield(0, catalog::veterans_armaments());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    let vanguard = g.add_card_to_hand(0, catalog::elite_vanguard());
    cast(&mut g, vanguard, None);
    assert_eq!(g.battlefield_find(arm).unwrap().attached_to, Some(vanguard), "attached to the entering Soldier");
}

/// Jace, Cunning Castaway's −5 makes two non-legendary copies of itself.
#[test]
fn jace_cunning_castaway_copies_itself() {
    let mut g = pod(2);
    let jace = g.add_card_to_battlefield(0, catalog::jace_cunning_castaway());
    g.battlefield_find_mut(jace).unwrap().add_counters(CounterType::Loyalty, 3);
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: jace, ability_index: 2, target: None, x_value: None })
        .unwrap();
    drain_stack(&mut g);
    let jaces = g.battlefield.iter().filter(|c| c.definition.name == "Jace, Cunning Castaway").count();
    assert_eq!(jaces, 3, "the original survives the legend rule beside two non-legendary copies");
}

/// CR 101.2 — under Experimental Frenzy no land or spell comes from the hand,
/// while the top of the library plays.
#[test]
fn cr_101_2_experimental_frenzy_locks_the_hand_and_opens_the_top() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::experimental_frenzy());
    let in_hand = g.add_card_to_hand(0, catalog::grizzly_bears());
    let land = g.add_card_to_hand(0, catalog::plains());
    flood(&mut g, 0);
    let cast_hand = g.perform_action(GameAction::CastSpell { card_id: in_hand, target: None, additional_targets: vec![], mode: None, x_value: None });
    assert!(cast_hand.is_err(), "no spell from the hand");
    assert!(g.perform_action(GameAction::PlayLand(land)).is_err(), "no land from the hand");
    let top = g.players[0].library[0].id;
    g.perform_action(GameAction::PlayLand(top)).expect("the top card plays");
    assert!(g.battlefield_find(top).is_some());
}

/// CR 614 — Nephalia Academy puts a card an opponent's spell makes you
/// discard on top of your library; your own discards still go to the
/// graveyard.
#[test]
fn nephalia_academy_saves_an_opponents_discard() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::nephalia_academy());
    let card = g.add_card_to_hand(1, catalog::grizzly_bears());
    let duress = g.add_card_to_hand(0, catalog::mind_rot());
    cast(&mut g, duress, Some(Target::Player(1)));
    assert_eq!(g.players[1].library[0].id, card, "back on top");
    assert!(g.players[1].graveyard.iter().all(|c| c.id != card));
}

/// Smoldering Egg banks the mana spent on instants and sorceries as ember
/// counters and hatches at seven.
#[test]
fn smoldering_egg_hatches_at_seven_embers() {
    let mut g = pod(2);
    let egg = g.add_card_to_battlefield(0, catalog::smoldering_egg());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.battlefield_find(egg).unwrap().counter_count(CounterType::Ember), 1);
    g.battlefield_find_mut(egg).unwrap().add_counters(CounterType::Ember, 5);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.battlefield_find(egg).unwrap().definition.name, "Ashmouth Dragon");
    assert_eq!(g.battlefield_find(egg).unwrap().counter_count(CounterType::Ember), 0);
}

/// Visions of Dread: the target opponent picks a creature card from their own
/// graveyard and you get it.
#[test]
fn visions_of_dread_takes_an_opponents_creature_card() {
    let mut g = pod(3);
    let dead = g.add_card_to_graveyard(2, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::hill_giant());
    let v = g.add_card_to_hand(0, catalog::visions_of_dread());
    cast(&mut g, v, Some(Target::Player(2)));
    assert_eq!(g.battlefield_find(dead).map(|c| c.controller), Some(0));
    assert_eq!(g.players[1].graveyard.len(), 1, "the untargeted opponent keeps theirs");
}

/// Uncage the Menagerie tutors X creature cards of mana value X, all with
/// different names.
#[test]
fn uncage_the_menagerie_fetches_distinct_names_at_x() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::impelled_giant());
    g.add_card_to_library(0, catalog::creeping_trailblazer());
    let u = g.add_card_to_hand(0, catalog::uncage_the_menagerie());
    let before = g.players[0].hand.len();
    cast_with(&mut g, u, None, vec![], Some(2), None);
    let got: Vec<_> = g.players[0].hand.iter().map(|c| c.definition.name).collect();
    assert_eq!(got.len(), before - 1 + 2, "{got:?}");
    assert!(got.contains(&"Grizzly Bears") && got.contains(&"Creeping Trailblazer"), "one Bears, not two: {got:?}");
}

/// Storm of Memories casts a random cheap instant from the graveyard for free,
/// then exiles it.
#[test]
fn storm_of_memories_recasts_from_the_graveyard() {
    let mut g = pod(2);
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let som = g.add_card_to_hand(0, catalog::storm_of_memories());
    let life = g.players[1].life;
    cast(&mut g, som, None);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt), "exiled after the free cast");
    assert_eq!((g.players[0].life, g.players[1].life), (life, life - 3), "the Bolt hit an opponent");
}

/// CR 702.88 — under Cast Through Time an instant cast from hand is exiled as
/// it resolves (rebound) instead of going to the graveyard.
#[test]
fn cr_702_88_cast_through_time_grants_rebound() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::cast_through_time());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert!(g.exile.iter().any(|c| c.id == bolt), "rebounded into exile");
    assert!(g.players[0].graveyard.iter().all(|c| c.id != bolt));
}

/// Helm of Obedience mills until a creature card and steals it, sacrificing
/// itself; with none in reach it mills X and stays.
#[test]
fn helm_of_obedience_mills_to_a_creature_and_takes_it() {
    let mut g = pod(2);
    let helm = g.add_card_to_battlefield(0, catalog::helm_of_obedience());
    g.players[1].library.clear();
    g.add_card_to_library(1, catalog::island());
    g.add_card_to_library(1, catalog::hill_giant());
    g.add_card_to_library(1, catalog::island());
    // library order: index 0 is the top.
    let top: Vec<_> = g.players[1].library.iter().map(|c| c.definition.name).collect();
    let giant = g.players[1].library.iter().find(|c| c.definition.name == "Hill Giant").unwrap().id;
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility { card_id: helm, ability_index: 0, target: Some(Target::Player(1)), additional_targets: vec![], x_value: Some(5), mode: None })
        .unwrap();
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(giant).map(|c| c.controller), Some(0), "{top:?}");
    assert!(g.battlefield_find(helm).is_none(), "sacrificed");
    assert_eq!(g.players[1].library.len(), 3 - 1 - top.iter().position(|n| *n == "Hill Giant").unwrap());
}

/// Territory Forge exiles a land on entering and taps for its mana.
#[test]
fn territory_forge_borrows_an_exiled_lands_ability() {
    let mut g = pod(2);
    let land = g.add_card_to_battlefield(1, catalog::nephalia_academy());
    let forge = g.add_card_to_hand(0, catalog::territory_forge());
    cast(&mut g, forge, Some(Target::Permanent(land)));
    assert!(g.exile.iter().any(|c| c.id == land && c.exiled_with == Some(forge)));
    let before = g.players[0].mana_pool.colorless_amount();
    g.perform_action(GameAction::ActivateAbility { card_id: forge, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("the Academy's {T}: add {C}");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), before + 1);
}

/// Animate Artifact makes a noncreature artifact an MV/MV creature, keeping
/// it an artifact creature.
#[test]
fn animate_artifact_animates_a_noncreature_artifact() {
    let mut g = pod(2);
    let helm = g.add_card_to_battlefield(0, catalog::helm_of_obedience());
    let aa = g.add_card_to_hand(0, catalog::animate_artifact());
    cast(&mut g, aa, Some(Target::Permanent(helm)));
    let cp = g.computed_permanent(helm).unwrap();
    assert!(cp.card_types().contains(&crabomination::card::CardType::Creature));
    assert_eq!((cp.power, cp.toughness), (4, 4));
}
