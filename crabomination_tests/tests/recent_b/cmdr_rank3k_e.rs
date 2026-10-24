//! Commander: two-card gaps of EDHREC ranks 2501-3500 (`decks::cmdr_rank3k_e`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn act(g: &mut GameState, action: GameAction) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(action).expect("action");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None });
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    act(g, GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    });
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne)
}

fn fire(g: &mut GameState, step: TurnStep) {
    g.step = step;
    g.fire_step_triggers(step);
    drain_stack(g);
}

fn declare(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.into_iter().map(|(attacker, p)| Attack { attacker, target: AttackTarget::Player(p) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn finish_combat(g: &mut GameState) {
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
}

/// Hope Tender untaps a land; its exert half untaps two and skips its own
/// next untap.
#[test]
fn hope_tender_untaps_lands() {
    let mut g = pod(2);
    let druid = ready(&mut g, 0, catalog::hope_tender());
    let a = ready(&mut g, 0, catalog::forest());
    let b = ready(&mut g, 0, catalog::forest());
    for id in [a, b] {
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    act(&mut g, GameAction::ActivateAbility {
        card_id: druid,
        ability_index: 1,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        x_value: None,
        mode: None,
    });
    assert!(!g.battlefield_find(a).unwrap().tapped && !g.battlefield_find(b).unwrap().tapped);
}

/// Blightcaster shrinks a creature off an enchantment spell; Sinister
/// Concoction kills for its pile of costs.
#[test]
fn aphemia_pair() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::blightcaster());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ench = g.add_card_to_hand(0, catalog::sinister_concoction());
    cast(&mut g, ench, None);
    assert!(g.battlefield_find(bear).is_none());
    let angel = ready(&mut g, 1, catalog::serra_angel());
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    activate(&mut g, ench, 0, Some(Target::Permanent(angel)));
    assert!(g.battlefield_find(angel).is_none());
    assert_eq!(g.players[0].life, 19);
}

/// Casey Jones digs an artifact; Curse of Stalked Prey grows the hitter.
#[test]
fn casey_jones_pair() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let ring = g.add_card_to_library(0, catalog::sol_ring());
    let casey = g.add_card_to_hand(0, catalog::casey_jones_jury_rig_justiciar());
    cast(&mut g, casey, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == ring));
    let curse = g.add_card_to_hand(0, catalog::curse_of_stalked_prey());
    cast(&mut g, curse, Some(Target::Player(1)));
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    declare(&mut g, vec![(bear, 1)]);
    finish_combat(&mut g);
    assert_eq!(plus(&g, bear), 1);
}

/// Savanti Romero grows and draws its counters; Madame Null sells counters
/// for life; Origin of Black Widow's chapters land.
#[test]
fn savanti_and_black_widow_cards() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let savanti = ready(&mut g, 0, catalog::savanti_romero_times_exile());
    fire(&mut g, TurnStep::BeginCombat);
    assert_eq!(plus(&g, savanti), 1);
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[0].life, 19);
    g.step = TurnStep::PreCombatMain;
    ready(&mut g, 0, catalog::madame_null_power_broker());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let angel = g.add_card_to_hand(0, catalog::serra_angel());
    cast(&mut g, angel, None);
    assert_eq!(plus(&g, angel), 4);
    assert_eq!(g.players[0].life, 15);
    let victim = ready(&mut g, 1, catalog::grizzly_bears());
    let origin = g.add_card_to_hand(0, catalog::origin_of_black_widow());
    cast(&mut g, origin, None);
    assert!(g.battlefield_find(victim).is_none(), "chapter I");
}

/// Wand of Vertebrae mills; Airlift Chaplain takes a cheap creature or grows.
#[test]
fn cyan_pair() {
    let mut g = pod(2);
    let wand = ready(&mut g, 0, catalog::wand_of_vertebrae());
    g.add_card_to_library(0, catalog::island());
    activate(&mut g, wand, 0, None);
    assert_eq!(g.players[0].graveyard.len(), 1);
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let chaplain = g.add_card_to_hand(0, catalog::airlift_chaplain());
    cast(&mut g, chaplain, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
    assert_eq!(plus(&g, chaplain), 0);
}

/// Immortus taps for {U} per card drawn; Refocus untaps and draws.
#[test]
fn immortus_pair() {
    let mut g = pod(2);
    let immortus = ready(&mut g, 0, catalog::immortus_master_of_eternity());
    g.players[0].cards_drawn_this_turn = 2;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: immortus,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("mana");
    assert_eq!(g.players[0].mana_pool.restricted_total(), 2, "noncreature-only blue mana");
    g.add_card_to_library(0, catalog::island());
    let refocus = g.add_card_to_hand(0, catalog::refocus());
    cast(&mut g, refocus, Some(Target::Permanent(immortus)));
    assert!(!g.battlefield_find(immortus).unwrap().tapped);
}

/// Kabira Vindicator's anthem steps up at levels 2 and 5; In the Trenches is
/// a flat anthem.
#[test]
fn kongming_pair() {
    let mut g = pod(2);
    let kabira = ready(&mut g, 0, catalog::kabira_vindicator());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(kabira).unwrap().add_counters(CounterType::Level, 2);
    assert_eq!(pt(&g, bear), (3, 3));
    assert_eq!(pt(&g, kabira), (3, 6));
    g.battlefield_find_mut(kabira).unwrap().add_counters(CounterType::Level, 3);
    assert_eq!(pt(&g, bear), (4, 4));
    assert_eq!(pt(&g, kabira), (4, 8));
    ready(&mut g, 0, catalog::in_the_trenches());
    assert_eq!(pt(&g, bear), (5, 5));
}

/// Benalish Commander counts Soldiers; Captain America shields Soldiers and
/// Heroes.
#[test]
fn heidegger_pair() {
    let mut g = pod(2);
    let cmdr = ready(&mut g, 0, catalog::benalish_commander());
    ready(&mut g, 0, catalog::general_thunderbolt_ross());
    assert_eq!(pt(&g, cmdr), (2, 2));
    let cap = g.add_card_to_hand(0, catalog::captain_america_unbowed());
    cast(&mut g, cap, None);
    assert!(g.computed_permanent(cmdr).unwrap().keywords().contains(&Keyword::Indestructible));
}

/// Doc Samson adds a counter of each kind; Restorative Technique fetches a
/// land and grows a creature.
#[test]
fn doc_samson_pair() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::doc_samson_super_psychiatrist());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let tech = g.add_card_to_hand(0, catalog::restorative_technique());
    act(&mut g, GameAction::CastSpell {
        card_id: tech,
        target: Some(Target::Player(0)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: None,
    });
    assert_eq!(plus(&g, bear), 2, "one plus Doc Samson's one");
    assert_eq!(g.players[0].life, 22);
    assert_eq!(named(&g, 0, "Forest"), 1);
    // CR 614.16 — "a permanent you control" reaches a land, but not the
    // counters a player gets ({E}).
    use crabomination::effect::{Effect, Selector};
    let land = ready(&mut g, 0, catalog::forest());
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let add = Effect::AddCounter {
        what: Selector::ExactObjects(vec![land]),
        kind: CounterType::Charge,
        amount: crabomination::card::Value::ONE,
    };
    g.resolve_effect(&add, &ctx).expect("charge");
    assert_eq!(g.battlefield_find(land).unwrap().counter_count(CounterType::Charge), 2);
    g.resolve_effect(&Effect::AddEnergy(crabomination::card::Value::ONE), &ctx).expect("energy");
    assert_eq!(g.players[0].energy, 1);
    // CR 614.16 — "if YOU would put": an opponent's counter isn't scaled.
    let theirs = EffectContext::for_spell(1, None, 0, 0);
    g.resolve_effect(&add, &theirs).expect("their charge");
    assert_eq!(g.battlefield_find(land).unwrap().counter_count(CounterType::Charge), 3);
}

/// Dyfed makes two tapped Powerstones; Geology Enthusiast adds one each end
/// step.
#[test]
fn dyfed_pair() {
    let mut g = pod(2);
    let dyfed = ready(&mut g, 0, catalog::dyfed_the_guiding_hand());
    assert!(catalog::dyfed_the_guiding_hand().can_be_commander);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: dyfed, ability_index: 0, target: None, x_value: None })
        .expect("+1");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Powerstone"), 2);
    ready(&mut g, 0, catalog::geology_enthusiast());
    fire(&mut g, TurnStep::End);
    assert_eq!(named(&g, 0, "Powerstone"), 3);
}

/// Unwelcome Sprite surveils off an opponent's-turn spell; Gustha draws three
/// and stacks two back.
#[test]
fn gustha_pair() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::unwelcome_sprite());
    ready(&mut g, 0, catalog::gustha_ebbasdotter());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
    }
    g.active_player_idx = 1;
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast(&mut g, opt, None);
    assert_eq!(g.players[0].hand.len(), 2, "Opt's draw plus three, two put back");
}

/// Elektra kills a small creature on entry; Boss Uramon makes Rats off Rogues.
#[test]
fn boss_uramon_pair() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let elektra = g.add_card_to_hand(0, catalog::elektra_daughter_of_the_hand());
    cast(&mut g, elektra, None);
    assert!(g.battlefield_find(bear).is_none());
    ready(&mut g, 0, catalog::boss_uramon_shadows_reach());
    g.clear_sickness(elektra);
    declare(&mut g, vec![(elektra, 1)]);
    finish_combat(&mut g);
    assert_eq!(named(&g, 0, "Rat Rogue"), 1);
}

/// Syr Faren lends its power to another attacker.
#[test]
fn syr_faren_pumps_another_attacker() {
    let mut g = pod(2);
    let faren = ready(&mut g, 0, catalog::syr_faren_the_hengehammer());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    declare(&mut g, vec![(faren, 1), (bear, 1)]);
    assert_eq!(pt(&g, bear), (4, 4));
}

/// Dragonlord Kolaghan gives haste and punishes a recast name.
#[test]
fn dragonlord_kolaghan_punishes_repeats() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::dragonlord_kolaghan());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Haste));
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let theirs = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.players[1].mana_pool.add(Color::Green, 2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: theirs, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 10);
}

/// Runechanter's Pike counts instants and sorceries in your graveyard.
#[test]
fn runechanters_pike_counts_spells() {
    let mut g = pod(2);
    let pike = ready(&mut g, 0, catalog::runechanters_pike());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::opt());
    act(&mut g, GameAction::Equip { equipment: pike, target: bear });
    assert_eq!(pt(&g, bear), (4, 2));
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::FirstStrike));
}

/// Edgar grows on a spell-less turn; Spirit of the Hunt shields Wolves.
#[test]
fn edgar_pair() {
    let mut g = pod(2);
    let edgar = ready(&mut g, 0, catalog::edgar_moonlit_sovereign());
    fire(&mut g, TurnStep::End);
    assert_eq!(plus(&g, edgar), 2);
    let spirit = g.add_card_to_hand(0, catalog::spirit_of_the_hunt());
    cast(&mut g, spirit, None);
    assert_eq!(pt(&g, edgar), (6, 9));
}

/// The Vision and Scarlet Witch adds {R} and grows on each spell; Viv Vision
/// draws when she attacks at 4 power.
#[test]
fn vision_and_viv() {
    let mut g = pod(2);
    let vsw = ready(&mut g, 0, catalog::the_vision_and_scarlet_witch());
    let opt = g.add_card_to_hand(0, catalog::opt());
    g.add_card_to_library(0, catalog::island());
    cast(&mut g, opt, None);
    assert_eq!(plus(&g, vsw), 1);
    let viv = ready(&mut g, 0, catalog::viv_vision_teen_synthezoid());
    activate(&mut g, viv, 0, None);
    g.add_card_to_library(0, catalog::island());
    let hand = g.players[0].hand.len();
    declare(&mut g, vec![(viv, 1)]);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// April O'Neil mints Mutagen on every artifact, instant or sorcery; Daily
/// Bugle loots for a Treasure.
#[test]
fn april_and_the_bugle() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::april_oneil_human_element());
    let bugle = g.add_card_to_hand(0, catalog::daily_bugle_newspaper());
    cast(&mut g, bugle, None);
    assert_eq!(named(&g, 0, "Mutagen"), 1);
    g.add_card_to_library(0, catalog::island());
    activate(&mut g, bugle, 0, None);
    assert_eq!(named(&g, 0, "Treasure"), 1);
}

/// Hardy Outlander's granted trigger pumps another creature off a commander
/// attacking the life leader — among opponents: your own higher total doesn't
/// switch it off.
#[test]
fn hardy_outlander_grants_the_commander_a_pump() {
    let mut g = pod(2);
    g.players[0].life = 60;
    ready(&mut g, 0, catalog::hardy_outlander());
    let cmdr = ready(&mut g, 0, catalog::serra_angel());
    g.players[0].commanders.push(cmdr);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    declare(&mut g, vec![(cmdr, 1)]);
    assert_eq!(pt(&g, bear), (6, 6));
}

/// Glen Elendra Pranksters bounces your creature off an opponent's-turn spell.
#[test]
fn glen_elendra_pranksters_saves_a_creature() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::glen_elendra_pranksters());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.add_card_to_library(0, catalog::island());
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast(&mut g, opt, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
}
