//! Commander: Omarthis's, Setzer's, Atemsis's, Serah Farron's, Lazav's and
//! Kaza's last gaps (`decks::cmdr_edhrec33`).

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

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn act(g: &mut GameState, action: GameAction) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
}

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) -> Result<(), String> {
    act(
        g,
        GameAction::ActivateAbility {
            card_id: id,
            ability_index: idx,
            target,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        },
    )
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn attack_and_hit(g: &mut GameState, attackers: &[CardId], defender: usize) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let attacks = attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(defender) });
    g.perform_action(GameAction::DeclareAttackers(attacks.collect())).expect("attack");
    drain_stack(g);
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

fn has_kw(g: &GameState, id: CardId, kw: Keyword) -> bool {
    g.computed_permanent(id).is_some_and(|cp| cp.keywords().contains(&kw))
}

/// CR 104.3a — Atemsis: damage to an opponent while your hand shows six
/// different mana values makes that player lose the game; five don't.
#[test]
fn cr_104_3a_atemsis_wins_on_six_mana_values() {
    let mut g = pod(3);
    let atemsis = ready(&mut g, 0, catalog::atemsis_all_seeing());
    for def in [catalog::lightning_bolt(), catalog::grizzly_bears(), catalog::hill_giant(), catalog::craw_wurm(), catalog::shivan_dragon()] {
        g.add_card_to_hand(0, def);
    }
    attack_and_hit(&mut g, &[atemsis], 1);
    assert!(g.players[1].is_alive(), "mana values 1, 2, 4, 6, 6 — four distinct");
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::colossal_dreadmaw());
    g.add_card_to_hand(0, catalog::lava_axe());
    g.battlefield_find_mut(atemsis).unwrap().tapped = false;
    attack_and_hit(&mut g, &[atemsis], 2);
    assert!(!g.players[2].is_alive(), "0, 1, 2, 4, 5, 6");
    assert!(g.players[1].is_alive());
}

/// Volshe Tideturner's {U} pays for a kicked spell but not an unkicked
/// creature spell.
#[test]
fn volshe_tideturner_mana_pays_kicked_spells() {
    let mut g = pod(2);
    let volshe = ready(&mut g, 0, catalog::volshe_tideturner());
    activate(&mut g, volshe, 0, None).expect("tap for mana");
    g.players[0].mana_pool.add_colorless(1);
    let hermit = g.add_card_to_hand(0, catalog::malevolent_hermit());
    assert!(cast(&mut g, hermit, None).is_err(), "a creature spell can't use it");
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Blue, 2);
    let spout = g.add_card_to_hand(0, catalog::waterspout_elemental());
    act(
        &mut g,
        GameAction::CastSpellKicked { card_id: spout, target: None, additional_targets: vec![], mode: None, x_value: None },
    )
    .expect("{3}{U}{U} + kicker {U}, the kicker paid with Volshe's {U}");
}

/// Casal pays {1}{G} on attack to become the Owlbear, whose transform pumps
/// the other legends; at your upkeep she transforms back.
#[test]
fn casal_transforms_on_attack_and_back_at_upkeep() {
    let mut g = pod(2);
    let casal = ready(&mut g, 0, catalog::casal_lurkwood_pathfinder());
    let other = ready(&mut g, 0, catalog::aragorn_hornburg_hero());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    flood(&mut g, 0);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: casal, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let cp = g.computed_permanent(casal).expect("casal");
    assert_eq!((cp.def.name, cp.power), ("Casal, Pathbreaker Owlbear", 6));
    assert_eq!(g.computed_permanent(other).unwrap().power, 6, "Aragorn +2/+2");
    assert!(has_kw(&g, other, Keyword::Trample));
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(casal).unwrap().def.name, "Casal, Lurkwood Pathfinder");
}

/// Senator Peacock gives every artifact the Clue's draw ability, and
/// sacrificing one makes a creature unblockable; Fleeting Memories mills
/// when a real Clue is sacrificed.
#[test]
fn senator_peacock_and_fleeting_memories_reward_clue_sacrifices() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    let peacock = ready(&mut g, 0, catalog::senator_peacock());
    let ring = ready(&mut g, 0, catalog::sol_ring());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    activate(&mut g, ring, 1, Some(Target::Permanent(peacock))).expect("Sol Ring's granted Clue ability");
    assert!(g.battlefield_find(ring).is_none());
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert!(has_kw(&g, peacock, Keyword::Unblockable));
    let memories = g.add_card_to_hand(0, catalog::fleeting_memories());
    cast(&mut g, memories, None).expect("memories");
    let clue = g.battlefield.iter().find(|c| c.definition.name == "Clue").map(|c| c.id).expect("clue");
    let lib1 = g.players[1].library.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    activate(&mut g, clue, 0, None).expect("sacrifice the Clue");
    assert_eq!(g.players[1].library.len() + 3, lib1, "Fleeting Memories mills the target player");
}

/// Start Your Engines animates Vehicles and gives the team +2/+0; Speedway
/// Fanatic's crew gives the Vehicle haste.
#[test]
fn start_your_engines_and_speedway_fanatic() {
    let mut g = pod(2);
    let truck = g.add_card_to_battlefield(0, catalog::renegade_freighter());
    let fanatic = ready(&mut g, 0, catalog::speedway_fanatic());
    let engines = g.add_card_to_hand(0, catalog::start_your_engines());
    flood(&mut g, 0);
    cast(&mut g, engines, None).expect("engines");
    let v = g.computed_permanent(truck).expect("truck");
    assert!(v.card_types().contains(&crabomination::card::CardType::Creature));
    assert_eq!(v.power, 4 + 2);
    act(&mut g, GameAction::Crew { vehicle: truck, crew_creatures: vec![fanatic] }).expect("crew");
    assert!(has_kw(&g, truck, Keyword::Haste));
}

/// Master's Guidance: two attacking legends earn a counter each (Aragorn's
/// first-strike renown waits for a first-strike step this helper skips); a 4-power
/// creature draws at your end step.
#[test]
fn masters_guidance_rewards_legends() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    ready(&mut g, 0, catalog::masters_guidance());
    let a = ready(&mut g, 0, catalog::aragorn_hornburg_hero());
    let b = ready(&mut g, 0, catalog::senator_peacock());
    attack_and_hit(&mut g, &[a, b], 1);
    let counters = |id| g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne));
    // CR 115.1c — the "whenever you attack" path fills both slots.
    assert_eq!((counters(a), counters(b)), (1, 1));
    let hand = g.players[0].hand.len();
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Sorcerer's Wand pings for 2 on a Wizard, 1 otherwise.
#[test]
fn sorcerers_wand_hits_harder_on_a_wizard() {
    let mut g = pod(2);
    let wand = ready(&mut g, 0, catalog::sorcerers_wand());
    let wizard = ready(&mut g, 0, catalog::volshe_tideturner());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(wand).unwrap().attached_to = Some(wizard);
    let life = g.players[1].life;
    activate(&mut g, wizard, 1, Some(Target::Player(1))).expect("wizard ping");
    assert_eq!(g.players[1].life, life - 2);
    g.battlefield_find_mut(wand).unwrap().attached_to = Some(bear);
    activate(&mut g, bear, 0, Some(Target::Player(1))).expect("bear ping");
    assert_eq!(g.players[1].life, life - 3);
}

