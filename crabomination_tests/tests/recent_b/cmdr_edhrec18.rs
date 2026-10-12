//! Commander: the top 600's last claimed gaps (`decks::cmdr_edhrec18`).

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

fn cast_full(
    g: &mut GameState,
    seat: usize,
    id: CardId,
    targets: &[Target],
    mode: Option<usize>,
    x: Option<u32>,
) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode,
        x_value: x,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_as(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_full(g, seat, id, targets, None, None)
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_as(g, 0, id, targets)
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn perm(id: CardId) -> Target {
    Target::Permanent(id)
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let cp = g.computed_permanent(id).expect("on battlefield");
    (cp.power, cp.toughness)
}

fn declare(g: &mut GameState, seat: usize, attacks: &[(CardId, usize)]) -> Result<(), String> {
    g.active_player_idx = seat;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.iter().map(|&(a, d)| Attack { attacker: a, target: AttackTarget::Player(d) }).collect(),
    ))
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn block(g: &mut GameState, seat: usize, blocks: Vec<(CardId, CardId)>) -> Result<(), String> {
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::DeclareBlockers(blocks)).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn combat_damage(g: &mut GameState) {
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

/// General Marhault: +3/+3 per blocker, counted as the trigger resolves.
#[test]
fn general_marhault_pumps_per_blocker() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::general_marhault_elsdragon());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let (b1, b2) = (ready(&mut g, 1, catalog::grizzly_bears()), ready(&mut g, 1, catalog::grizzly_bears()));
    declare(&mut g, 0, &[(bear, 1)]).expect("attack");
    block(&mut g, 1, vec![(b1, bear), (b2, bear)]).expect("block");
    assert_eq!(pt(&g, bear), (8, 8), "two blockers: +6/+6");
}

/// You Look Upon the Tarrasque's second mode: every creature able to block the
/// target must (CR 509.1c), so an empty block declaration is illegal.
#[test]
fn tarrasque_mode_two_forces_blocks() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let wall = ready(&mut g, 1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::you_look_upon_the_tarrasque());
    cast_full(&mut g, 0, s, &[perm(bear)], Some(1), None).expect("cast");
    assert_eq!(pt(&g, bear), (7, 7));
    declare(&mut g, 0, &[(bear, 1)]).expect("attack");
    assert!(block(&mut g, 1, vec![]).is_err(), "the bear must block");
    block(&mut g, 1, vec![(wall, bear)]).expect("block");
}

/// Spellwild Ouphe discounts ANY player's spell that targets it (2007-05-01).
#[test]
fn spellwild_ouphe_discounts_opponents_too() {
    let mut g = pod(2);
    let ouphe = ready(&mut g, 0, catalog::spellwild_ouphe());
    let bolt = g.add_card_to_hand(1, catalog::lightning_strike());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(perm(ouphe)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{1}{R} less {2} is {R}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ouphe).is_none());
}

/// Auramancer's Guise counts every Aura on the host, itself included.
#[test]
fn auramancers_guise_counts_auras() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let guise = g.add_card_to_hand(0, catalog::auramancers_guise());
    cast(&mut g, guise, &[perm(bear)]).expect("guise");
    assert_eq!(pt(&g, bear), (4, 4));
    let rune = g.add_card_to_hand(0, catalog::rune_of_might());
    cast(&mut g, rune, &[perm(bear)]).expect("rune");
    assert_eq!(pt(&g, bear), (7, 7), "+4/+4 for two Auras, +1/+1 from the Rune");
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Vigilance));
}

/// Imodane echoes a single-target spell's damage to each opponent; a spell
/// with two targets doesn't count.
#[test]
fn imodane_echoes_single_target_burn() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::imodane_the_pyrohammer());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let life = g.players[1].life;
    let s = g.add_card_to_hand(0, catalog::lightning_strike());
    cast(&mut g, s, &[perm(bear)]).expect("strike");
    assert_eq!((g.players[1].life, g.players[2].life), (life - 3, life - 3));
    let (b1, b2) = (ready(&mut g, 1, catalog::grizzly_bears()), ready(&mut g, 2, catalog::grizzly_bears()));
    let swarm = g.add_card_to_hand(0, catalog::meteor_swarm());
    cast_full(&mut g, 0, swarm, &[perm(b1), perm(b2)], None, Some(2)).expect("swarm");
    assert!(g.battlefield_find(b1).is_none() && g.battlefield_find(b2).is_none());
    assert_eq!((g.players[1].life, g.players[2].life), (life - 3, life - 3), "two targets: no echo");
}

/// Meteor Swarm takes exactly X targets (CR 601.2c).
#[test]
fn meteor_swarm_needs_exactly_x_targets() {
    let mut g = pod(2);
    let b1 = ready(&mut g, 1, catalog::grizzly_bears());
    let swarm = g.add_card_to_hand(0, catalog::meteor_swarm());
    assert!(cast_full(&mut g, 0, swarm, &[perm(b1)], None, Some(2)).is_err());
}

/// Chandra copies the first instant/sorcery each turn only.
#[test]
fn chandra_hopes_beacon_copies_once_a_turn() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::chandra_hopes_beacon());
    let life = g.players[1].life;
    for _ in 0..2 {
        let s = g.add_card_to_hand(0, catalog::lightning_strike());
        cast(&mut g, s, &[Target::Player(1)]).expect("strike");
    }
    assert_eq!(g.players[1].life, life - 9, "3 + 3 copied + 3 uncopied");
}

/// Reality Spasm's untap mode takes exactly X targets.
#[test]
fn reality_spasm_untaps_x() {
    let mut g = pod(2);
    let (a, b) = (ready(&mut g, 0, catalog::grizzly_bears()), ready(&mut g, 0, catalog::grizzly_bears()));
    for id in [a, b] {
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    let s = g.add_card_to_hand(0, catalog::reality_spasm());
    assert!(cast_full(&mut g, 0, s, &[perm(a)], Some(1), Some(2)).is_err(), "X=2 needs two targets");
    cast_full(&mut g, 0, s, &[perm(a), perm(b)], Some(1), Some(2)).expect("cast");
    assert!(!g.battlefield_find(a).unwrap().tapped && !g.battlefield_find(b).unwrap().tapped);
}

/// Boxing Ring: an entering creature fights one with the same mana value;
/// then the Ring makes a Treasure (only once someone fought).
#[test]
fn boxing_ring_fights_same_mana_value() {
    let mut g = pod(2);
    let ring = ready(&mut g, 0, catalog::boxing_ring());
    let tap = |g: &mut GameState| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: ring,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(tap(&mut g).is_err(), "nobody fought yet");
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let three = ready(&mut g, 1, catalog::hill_giant());
    let tusker = g.add_card_to_hand(0, catalog::kalonian_tusker());
    cast(&mut g, tusker, &[]).expect("tusker");
    assert!(g.battlefield_find(three).is_some_and(|c| c.damage == 0), "a 3-drop is not a legal foe");
    assert!(g.battlefield_find(theirs).is_none(), "the 3/3 Tusker killed the bear");
    tap(&mut g).expect("treasure");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Treasure"));
}

/// Maarika: damage to a creature dealt excess damage makes its controller
/// sacrifice a noncreature, nonland permanent.
#[test]
fn maarika_punishes_excess_damage() {
    let mut g = pod(2);
    let m = ready(&mut g, 0, catalog::maarika_brutal_gladiator());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let rock = ready(&mut g, 1, catalog::mind_stone());
    declare(&mut g, 0, &[(m, 1)]).expect("attack");
    block(&mut g, 1, vec![(bear, m)]).expect("block");
    combat_damage(&mut g);
    assert!(g.battlefield_find(rock).is_none(), "the Mind Stone was sacrificed");
    assert!(g.battlefield_find(m).is_some(), "indestructible on its own turn");
}

/// Hunter's Bow attaches and bites.
#[test]
fn hunters_bow_bites_on_entry() {
    let mut g = pod(2);
    let giant = ready(&mut g, 0, catalog::hill_giant());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let bow = g.add_card_to_hand(0, catalog::hunters_bow());
    cast(&mut g, bow, &[]).expect("bow");
    let bow_id = g.battlefield.iter().find(|c| c.definition.name == "Hunter's Bow").unwrap().id;
    assert_eq!(g.battlefield_find(bow_id).unwrap().attached_to, Some(giant));
    assert!(g.battlefield_find(bear).is_none(), "3 damage killed the bear");
    assert!(g.computed_permanent(giant).unwrap().keywords().contains(&Keyword::Reach));
}

/// Abigale strips the target's abilities, then its counters grant three.
#[test]
fn abigale_rewrites_a_creature() {
    let mut g = pod(2);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let abi = g.add_card_to_hand(0, catalog::abigale_eloquent_first_year());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: abi,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("abigale");
    drain_stack(&mut g);
    let kw = g.computed_permanent(angel).unwrap().keywords().to_vec();
    assert!(!kw.contains(&Keyword::Vigilance), "lost vigilance");
    for k in [Keyword::Flying, Keyword::FirstStrike, Keyword::Lifelink] {
        assert!(kw.contains(&k), "{k:?} from its counter");
    }
}

/// Brigone grows on heroic and cashes the counter for a card.
#[test]
fn brigone_heroic_draws() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    let b = ready(&mut g, 0, catalog::brigone_soldier_of_meletis());
    let s = g.add_card_to_hand(0, catalog::giant_growth());
    cast(&mut g, s, &[perm(b)]).expect("growth");
    assert_eq!(g.battlefield_find(b).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let hand = g.players[0].hand.len();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: b,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Noggle Ransacker: each player draws two, then discards one at random.
#[test]
fn noggle_ransacker_wheels_everyone() {
    let mut g = pod(3);
    for seat in 0..3 {
        for _ in 0..2 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    let hands: Vec<usize> = (0..3).map(|s| g.players[s].hand.len()).collect();
    let n = g.add_card_to_hand(0, catalog::noggle_ransacker());
    cast(&mut g, n, &[]).expect("cast");
    for (s, h) in hands.iter().enumerate() {
        assert_eq!(g.players[s].hand.len(), h + 1, "seat {s}");
    }
}

fn run(g: &mut GameState, effect: &crabomination::effect::Effect) {
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn loyalty(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: id, ability_index: index, target, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn upkeep(g: &mut GameState, seat: usize) {
    g.active_player_idx = seat;
    g.turn_number += 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(g);
}

fn named(g: &GameState, id: CardId) -> &'static str {
    g.battlefield_find(id).map_or("", |c| c.definition.name)
}

/// The Kenriths' Royal Funeral: exiles two legends (draw/lose the bigger mana
/// value), then legendary spells cost {1} less per exiled card.
#[test]
fn kenriths_funeral_exiles_legends_and_discounts() {
    let mut g = pod(2);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_graveyard(0, catalog::imodane_the_pyrohammer());
    g.add_card_to_graveyard(0, catalog::brigone_soldier_of_meletis());
    let (hand, life) = (g.players[0].hand.len(), g.players[0].life);
    let f = g.add_card_to_hand(0, catalog::the_kenriths_royal_funeral());
    cast(&mut g, f, &[]).expect("funeral");
    assert!(g.players[0].graveyard.is_empty(), "both legends exiled");
    assert_eq!(g.players[0].hand.len(), hand + 4, "X = Imodane's 4");
    assert_eq!(g.players[0].life, life - 4);
    let h = g.add_card_to_hand(0, catalog::hinata_dawn_crowned());
    g.players[0].mana_pool.empty();
    for c in [Color::Blue, Color::Red, Color::White] {
        g.players[0].mana_pool.add(c, 1);
    }
    g.perform_action(GameAction::CastSpell { card_id: h, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{1}{U}{R}{W} less {2}");
}

/// Kaya's +1: the creature comes back to hand with a Spirit when it dies.
#[test]
fn kaya_ghostform_returns_the_creature() {
    let mut g = pod(2);
    let kaya = ready(&mut g, 0, catalog::kaya_the_inexorable());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    loyalty(&mut g, kaya, 0, Some(perm(bear))).expect("+1");
    let s = g.add_card_to_hand(1, catalog::lightning_strike());
    cast_as(&mut g, 1, s, &[perm(bear)]).expect("strike");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"), "back to hand");
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Spirit"));
}

/// Kaya's emblem casts a legendary card from exile for free.
#[test]
fn kaya_emblem_casts_a_legend_from_exile() {
    let mut g = pod(2);
    let kaya = ready(&mut g, 0, catalog::kaya_the_inexorable());
    g.battlefield_find_mut(kaya).unwrap().add_counters(CounterType::Loyalty, 5);
    loyalty(&mut g, kaya, 2, None).expect("-7");
    let legend = g.add_card_to_exile(0, catalog::brigone_soldier_of_meletis());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![legend]), DecisionAnswer::Bool(true)]));
    upkeep(&mut g, 0);
    assert_eq!(named(&g, legend), "Brigone, Soldier of Meletis");
}

/// Megatron converts at the postcombat main phase for {C} per life lost.
#[test]
fn megatron_converts_for_colorless() {
    let mut g = pod(2);
    let m = ready(&mut g, 0, catalog::megatron_tyrant());
    let s = g.add_card_to_hand(0, catalog::lightning_strike());
    cast(&mut g, s, &[Target::Player(1)]).expect("strike");
    g.players[0].mana_pool.empty();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::PostCombatMain;
    g.fire_step_triggers(TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert_eq!(named(&g, m), "Megatron, Destructive Force");
    assert_eq!(g.players[0].mana_pool.total(), 3);
}

/// Megatron, Destructive Force: the sacrificed artifact's mana value hits a
/// creature, the excess goes to its controller, and the excess converts it.
#[test]
fn megatron_destructive_force_redirects_excess() {
    let mut g = pod(2);
    let m = ready(&mut g, 0, catalog::megatron_tyrant());
    ready(&mut g, 0, catalog::su_chi_cave_guard());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let ctx = EffectContext { source: Some(m), ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&crabomination::effect::Effect::Transform { what: crabomination::effect::Selector::This }, &ctx)
        .unwrap();
    assert_eq!(named(&g, m), "Megatron, Destructive Force");
    let life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    declare(&mut g, 0, &[(m, 1)]).expect("attack");
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[1].life, life - 6, "8 less the bear's lethal 2");
    assert_eq!(named(&g, m), "Megatron, Tyrant", "the excess converted it back");
}

/// Starscream drains while you're the monarch; its back face crowns the
/// player it hits when nobody is.
#[test]
fn starscream_and_the_monarchy() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    ready(&mut g, 0, catalog::starscream_power_hungry());
    g.monarch = Some(0);
    let life = g.players[1].life;
    run(&mut g, &crabomination::effect::Effect::Draw {
        who: crabomination::effect::Selector::You,
        amount: crabomination::effect::Value::ONE,
    });
    assert_eq!(g.players[1].life, life - 2);
}

/// Mishra: ward—sacrifice a permanent; an opponent with nothing to sacrifice
/// has its spell countered.
#[test]
fn mishra_grants_sacrifice_ward() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::mishra_tamer_of_mak_fawa());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(1, catalog::lightning_strike());
    cast_as(&mut g, 1, s, &[perm(bear)]).expect("strike");
    assert!(g.battlefield_find(bear).is_some(), "countered by ward");
}

/// Su-Chi Cave Guard's eight {C} survive the step change.
#[test]
fn su_chi_mana_lasts_the_turn() {
    let mut g = pod(2);
    let su = ready(&mut g, 0, catalog::su_chi_cave_guard());
    run(&mut g, &crabomination::effect::Effect::Destroy {
        what: crabomination::effect::Selector::EachPermanent(crabomination::card::SelectionRequirement::Creature),
    });
    assert!(g.battlefield_find(su).is_none());
    g.step = TurnStep::PostCombatMain;
    g.empty_mana_pools();
    assert_eq!(g.players[0].mana_pool.total(), 8);
}

/// Avatar Roku: an attack adds six {R} that last through combat's steps.
#[test]
fn avatar_roku_mana_lasts_through_combat() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::avatar_roku_firebender());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.players[0].mana_pool.empty();
    declare(&mut g, 1, &[(bear, 0)]).expect("their attack");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 6);
    g.step = TurnStep::DeclareBlockers;
    g.empty_mana_pools();
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 6, "kept until end of combat");
    g.step = TurnStep::EndCombat;
    g.empty_mana_pools();
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 0);
}

/// CR 702.24a — Braid of Fire's cumulative upkeep adds {R} per age counter.
#[test]
fn cr_702_24a_braid_of_fire_adds_mana() {
    let mut g = pod(2);
    let braid = ready(&mut g, 0, catalog::braid_of_fire());
    for n in 1..=2 {
        g.players[0].mana_pool.empty();
        g.step = TurnStep::Upkeep;
        let ev = g.process_cumulative_upkeep();
        g.dispatch_triggers_for_events(&ev);
        drain_stack(&mut g);
        assert_eq!(g.players[0].mana_pool.amount(Color::Red), n);
        assert_eq!(g.battlefield_find(braid).unwrap().counter_count(CounterType::Age), n);
    }
}

/// The Rise of Sozin: I wipes the board; III returns it as Fire Lord Sozin.
#[test]
fn rise_of_sozin_chapters() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let def = catalog::the_rise_of_sozin();
    run(&mut g, &def.saga_chapters[0].1);
    assert!(g.battlefield_find(bear).is_none());
    let saga = ready(&mut g, 0, catalog::the_rise_of_sozin());
    let ctx = EffectContext { source: Some(saga), ..EffectContext::for_spell(0, None, 0, 0) };
    let evs = g.resolve_effect(&def.saga_chapters[2].1, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Fire Lord Sozin"));
}

/// Hinata: your spells cost {1} less per target, an opponent's {1} more.
#[test]
fn hinata_shifts_costs_per_target() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::hinata_dawn_crowned());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let mine = g.add_card_to_hand(0, catalog::lightning_strike());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: mine, target: Some(perm(bear)), additional_targets: vec![], mode: None, x_value: None })
        .expect("{1}{R} less {1}");
    drain_stack(&mut g);
    let theirs = g.add_card_to_hand(1, catalog::lightning_strike());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    g.players[1].mana_pool.add_colorless(1);
    let cast = GameAction::CastSpell { card_id: theirs, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None };
    assert!(g.perform_action(cast.clone()).is_err(), "{{1}}{{R}} plus {{1}}");
    g.players[1].mana_pool.add_colorless(1);
    g.perform_action(cast).expect("{2}{R}");
}

/// Soulfire Grand Master: lifelink spells; the activation buys the next one back.
#[test]
fn soulfire_grand_master_buys_back() {
    let mut g = pod(2);
    let sgm = ready(&mut g, 0, catalog::soulfire_grand_master());
    let life = g.players[0].life;
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sgm,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let s = g.add_card_to_hand(0, catalog::lightning_strike());
    cast(&mut g, s, &[Target::Player(1)]).expect("strike");
    assert_eq!(g.players[0].life, life + 3, "the spell has lifelink");
    assert!(g.players[0].hand.iter().any(|c| c.id == s), "returned to hand");
}

/// March of Swirling Mist phases out up to X creatures.
#[test]
fn march_of_swirling_mist_phases_out() {
    let mut g = pod(2);
    let (a, b) = (ready(&mut g, 0, catalog::grizzly_bears()), ready(&mut g, 0, catalog::grizzly_bears()));
    let m = g.add_card_to_hand(0, catalog::march_of_swirling_mist());
    cast_full(&mut g, 0, m, &[perm(a), perm(b)], None, Some(2)).expect("march");
    assert!([a, b].iter().all(|id| g.phased_out.iter().any(|c| c.id == *id)));
}

/// Ian Malcolm: a second draw exiles the drawer's top card; another player
/// may cast it on their own turn.
#[test]
fn ian_malcolm_shares_the_exiles() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::ian_malcolm_chaotician());
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::lightning_strike());
    }
    let draw2 = crabomination::effect::Effect::Draw {
        who: crabomination::effect::Selector::Player(crabomination::effect::PlayerRef::Seat(1)),
        amount: crabomination::effect::Value::Const(2),
    };
    run(&mut g, &draw2);
    let exiled = g.exile.iter().find(|c| c.owner == 1).map(|c| c.id).expect("their top card exiled");
    upkeep(&mut g, 0);
    g.step = TurnStep::PreCombatMain;
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: exiled,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("seat 0 casts seat 1's exiled card");
}

/// Hive Mind: each OTHER player copies the spell (three seats, two copies).
#[test]
fn hive_mind_copies_for_each_other_player() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::hive_mind());
    let life = g.players[0].life;
    let s = g.add_card_to_hand(0, catalog::lightning_strike());
    cast(&mut g, s, &[Target::Player(0)]).expect("strike myself");
    assert_eq!(g.players[0].life, life - 9, "the original and two copies keep their target");
}

/// Scrambleverse redistributes every nonland permanent and untaps them.
#[test]
fn scrambleverse_keeps_lands_and_untaps() {
    let mut g = pod(3);
    let land = ready(&mut g, 0, catalog::island());
    let bears: Vec<CardId> = (0..6).map(|_| ready(&mut g, 1, catalog::grizzly_bears())).collect();
    for id in &bears {
        g.battlefield_find_mut(*id).unwrap().tapped = true;
    }
    let s = g.add_card_to_hand(0, catalog::scrambleverse());
    cast(&mut g, s, &[]).expect("scramble");
    assert_eq!(g.battlefield_find(land).unwrap().controller, 0, "lands stay");
    assert!(bears.iter().all(|id| !g.battlefield_find(*id).unwrap().tapped));
}

/// Saruman the White amasses Orcs 2 on your second spell only.
#[test]
fn saruman_the_white_amasses_on_the_second_spell() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::saruman_the_white());
    let army = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name.contains("Army")).count();
    for n in [0, 1, 1] {
        let s = g.add_card_to_hand(0, catalog::lightning_strike());
        cast(&mut g, s, &[Target::Player(1)]).expect("strike");
        assert_eq!(army(&g), n);
    }
}

/// Saruman of Many Colors: the second spell mills each opponent two, then
/// copies an instant from their graveyard with no greater mana value.
#[test]
fn saruman_of_many_colors_steals_a_spell() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::saruman_of_many_colors());
    g.add_card_to_library(1, catalog::island());
    g.add_card_to_library(1, catalog::lightning_strike());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true); 4]));
    let life = g.players[1].life;
    for _ in 0..2 {
        let s = g.add_card_to_hand(0, catalog::lightning_strike());
        cast(&mut g, s, &[Target::Player(1)]).expect("strike");
    }
    assert!(g.players[1].life <= life - 9, "two strikes and the stolen copy");
}

/// Press the Enemy: bounce an opposing permanent, then cast a cheaper
/// instant from hand free.
#[test]
fn press_the_enemy_bounces_and_casts() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let s = g.add_card_to_hand(0, catalog::lightning_strike());
    let p = g.add_card_to_hand(0, catalog::press_the_enemy());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Cards(vec![s]),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
    ]));
    let life = g.players[1].life;
    cast_full(&mut g, 0, p, &[perm(giant)], Some(0), None).expect("press");
    assert!(g.battlefield_find(giant).is_none());
    assert!(g.players[0].hand.iter().all(|c| c.id != s), "the strike was cast");
    assert!(g.players[1].life < life);
}

/// Fire Lord Sozin steals only from the graveyard of the player it hit.
#[test]
fn fire_lord_sozin_reanimates_from_the_damaged_player() {
    let mut g = pod(3);
    let other = g.add_card_to_graveyard(2, catalog::hill_giant());
    let theirs = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let sozin = ready(&mut g, 0, catalog::fire_lord_sozin());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(5)]));
    flood(&mut g, 0);
    declare(&mut g, 0, &[(sozin, 1)]).expect("attack");
    combat_damage(&mut g);
    assert_eq!(g.battlefield_find(theirs).map(|c| c.controller), Some(0), "seat 1's bear is ours");
    assert!(g.battlefield_find(other).is_none(), "seat 2's graveyard is off limits");
}
