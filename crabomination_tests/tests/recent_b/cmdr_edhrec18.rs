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

/// "Able to block that creature this turn do so" binds a creature that
/// enters after the spell resolves too (CR 509.1c, the requirement is on
/// the attacker).
#[test]
fn tarrasque_mode_two_binds_a_later_blocker() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::you_look_upon_the_tarrasque());
    cast_full(&mut g, 0, s, &[perm(bear)], Some(1), None).expect("cast");
    let late = ready(&mut g, 1, catalog::grizzly_bears());
    declare(&mut g, 0, &[(bear, 1)]).expect("attack");
    assert!(block(&mut g, 1, vec![]).is_err(), "the late bear must block too");
    block(&mut g, 1, vec![(late, bear)]).expect("block");
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

/// The Rise of Sozin II targets one opponent and exiles up to four cards
/// with the chosen name from that player's zones only.
#[test]
fn rise_of_sozin_chapter_two_strips_the_target_opponent() {
    let mut g = pod(3);
    let def = catalog::the_rise_of_sozin();
    assert!(def.saga_chapters[1].1.requires_target(), "target opponent");
    for _ in 0..3 {
        g.add_card_to_graveyard(2, catalog::grizzly_bears());
        g.add_card_to_hand(2, catalog::grizzly_bears());
    }
    let other = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::NamedCard("Grizzly Bears".to_string())]));
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Player(2)];
    g.resolve_effect(&def.saga_chapters[1].1, &ctx).unwrap();
    assert_eq!(g.exile.iter().filter(|c| c.owner == 2).count(), 4, "up to four");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == other), "another opponent's are untouched");
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

/// Seated decks' residuals closed: Steel Seraph's keyword is a mode; Lightning
/// Axe pays {5} with no card to discard; Frantic Firebolt counts Adventures.
#[test]
fn seat_residuals_steel_seraph_axe_firebolt() {
    let mut g = pod(2);
    let seraph = ready(&mut g, 0, catalog::steel_seraph());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let _ = seraph;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(2)]));
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let kw = |g: &GameState, id| g.computed_permanent(id).unwrap().keywords().to_vec();
    assert!(kw(&g, bear).contains(&Keyword::Lifelink) || kw(&g, seraph).contains(&Keyword::Lifelink));

    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let axe = g.add_card_to_hand(0, catalog::lightning_axe());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    let axe_cast = GameAction::CastSpell { card_id: axe, target: Some(perm(giant)), additional_targets: vec![], mode: None, x_value: None };
    assert!(g.perform_action(axe_cast.clone()).is_err(), "no card to discard: {{5}} more");
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(axe_cast).expect("{R} + {5}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none());

    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    g.add_card_to_graveyard(0, catalog::bonecrusher_giant());
    let ff = g.add_card_to_hand(0, catalog::frantic_firebolt());
    cast(&mut g, ff, &[perm(giant)]).expect("firebolt");
    assert!(g.battlefield_find(giant).is_none(), "2 + the Adventure card = 3");
}

/// Runadi's haste grant reads a live counter count: a creature with three
/// +1/+1 counters has haste, one with two doesn't.
#[test]
fn runadi_grants_haste_at_three_counters() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::runadi_behemoth_caller());
    let (a, b) = (ready(&mut g, 0, catalog::grizzly_bears()), ready(&mut g, 0, catalog::grizzly_bears()));
    g.battlefield_find_mut(a).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    g.battlefield_find_mut(b).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let haste = |g: &GameState, id| g.computed_permanent(id).unwrap().keywords().contains(&Keyword::Haste);
    assert!(haste(&g, a) && !haste(&g, b));
}

/// Shay Cormac strips every protection and ward instance, not just hexproof.
#[test]
fn shay_cormac_strips_protection_and_ward() {
    let mut g = pod(2);
    let shay = ready(&mut g, 0, catalog::shay_cormac());
    let su = ready(&mut g, 1, catalog::su_chi_cave_guard());
    let knight = ready(&mut g, 1, catalog::white_knight());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: shay,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let kw = |g: &GameState, id| g.computed_permanent(id).unwrap().keywords().to_vec();
    assert!(!kw(&g, su).iter().any(|k| matches!(k, Keyword::Ward(_))), "ward gone");
    assert!(!kw(&g, knight).iter().any(|k| k.is_protection()), "protection gone");
}

/// The Rune cycle on an Equipment: the equipped creature gets the bonus.
#[test]
fn rune_of_might_on_equipment_pumps_the_wielder() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let flail = ready(&mut g, 0, catalog::gorgon_flail());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: flail, target: bear }).expect("equip");
    drain_stack(&mut g);
    let rune = g.add_card_to_hand(0, catalog::rune_of_might());
    cast(&mut g, rune, &[perm(flail)]).expect("rune on the flail");
    assert_eq!(pt(&g, bear), (4, 4), "+1/+1 Flail, +1/+1 relayed Rune");
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Trample));
}

/// Thirst for Discovery — "Draw three cards. Then discard two cards
/// unless you discard a basic land card." (CR 701.9a discard; the
/// "unless" exemption is the DiscardUnlessKind shape). With only basic
/// Islands drawn, a single Island is pitched and the hand keeps two.
#[test]
fn thirst_for_discovery_pitches_one_basic_land() {
    let mut g = pod(2);
    g.players[0].hand.clear();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::thirst_for_discovery());
    cast(&mut g, id, &[]).expect("castable");
    assert_eq!(g.players[0].hand.len(), 2, "drew 3, pitched one basic land");
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Island").count(), 1);
}

/// Without a basic land among the cards, Thirst for Discovery discards two.
#[test]
fn thirst_for_discovery_discards_two_without_a_basic() {
    let mut g = pod(2);
    g.players[0].hand.clear();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    let id = g.add_card_to_hand(0, catalog::thirst_for_discovery());
    cast(&mut g, id, &[]).expect("castable");
    assert_eq!(g.players[0].hand.len(), 1, "drew 3, discarded 2");
}

/// The Endstone — "your life total becomes half your starting life total,
/// rounded up" (CR 119.5 set life total; CR 903.7 Commander starts at 40):
/// a 40-life seat ends its turn at 20, not the 2-player 10.
#[test]
fn the_endstone_halves_the_commander_starting_life() {
    let mut g = pod(2);
    g.players[0].starting_life = 40;
    g.players[0].life = 37;
    g.add_card_to_battlefield(0, catalog::the_endstone());
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20);
}

/// Nexus of Fate — "If Nexus of Fate would be put into a graveyard from
/// anywhere, reveal it and shuffle it into its owner's library instead."
/// (CR 614.6.) It resolves and lands in the library, not the graveyard.
#[test]
fn nexus_of_fate_shuffles_back_after_resolving() {
    let mut g = pod(2);
    let lib = g.players[0].library.len();
    let id = g.add_card_to_hand(0, catalog::nexus_of_fate());
    cast(&mut g, id, &[]).expect("castable");
    assert!(g.players[0].graveyard.iter().all(|c| c.id != id));
    assert_eq!(g.players[0].library.len(), lib + 1);
}

/// Part the Waterveil — "Take an extra turn after this one. Exile Part the
/// Waterveil." (CR 608.2n: the exile replaces the graveyard trip.)
#[test]
fn part_the_waterveil_exiles_itself() {
    let mut g = pod(2);
    let id = g.add_card_to_hand(0, catalog::part_the_waterveil());
    cast(&mut g, id, &[]).expect("castable");
    assert!(g.players[0].graveyard.iter().all(|c| c.id != id));
    assert!(g.exile.iter().any(|c| c.id == id));
}

/// Llanowar Greenwidow — "Domain — {7}{G}: Return this card from your
/// graveyard to the battlefield tapped. This ability costs {1} less to
/// activate for each basic land type among lands you control." (CR 702.
/// domain ability word; CR 601.2f cost reduction.) Three basic types make it
/// {4}{G}, so five mana is enough.
#[test]
fn llanowar_greenwidow_domain_discount() {
    let mut g = pod(2);
    for land in [catalog::forest(), catalog::island(), catalog::swamp()] {
        let id = g.add_card_to_battlefield(0, land);
        // Tapped, so only the floating five can pay.
        g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = true;
    }
    let spider = g.add_card_to_graveyard(0, catalog::llanowar_greenwidow());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: spider,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("{4}{G} after domain 3");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == spider && c.tapped));
}

/// Elvish Refueler — "During your turn, as long as you haven't activated an
/// exhaust ability this turn, you may activate exhaust abilities as though
/// they haven't been activated." (CR 702.177a exhaust.) A spent exhaust
/// ability reactivates once per turn of yours, and never on an opponent's.
#[test]
fn elvish_refueler_reopens_one_exhaust_per_turn() {
    let mut g = pod(2);
    let elf = ready(&mut g, 0, catalog::elvish_refueler());
    let pump = |g: &mut GameState| {
        flood(g, 0);
        g.priority.player_with_priority = 0;
        let r = g.perform_action(GameAction::ActivateAbility {
            card_id: elf,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        });
        drain_stack(g);
        r
    };
    let counters = |g: &GameState| {
        g.battlefield.iter().find(|c| c.id == elf).unwrap().counter_count(CounterType::PlusOnePlusOne)
    };
    pump(&mut g).expect("first exhaust");
    assert!(pump(&mut g).is_err(), "already activated an exhaust ability this turn");
    assert_eq!(counters(&g), 1);
    // Next turn of yours: reopens once.
    g.do_cleanup(&mut vec![]);
    g.active_player_idx = 0;
    pump(&mut g).expect("refueled");
    assert_eq!(counters(&g), 2);
    assert!(pump(&mut g).is_err());
    // On an opponent's turn the spent mark holds.
    g.do_cleanup(&mut vec![]);
    g.active_player_idx = 1;
    assert!(pump(&mut g).is_err(), "only during your turn");
}

/// Bonecache Overseer — "Activate only if three or more cards left your
/// graveyard this turn or if you've sacrificed a Food this turn." (CR
/// 602.5b activation restriction.) Sacrificing a Food opens the draw.
#[test]
fn bonecache_overseer_opens_after_a_food_sacrifice() {
    let mut g = pod(2);
    let squirrel = ready(&mut g, 0, catalog::bonecache_overseer());
    g.add_card_to_library(0, catalog::island());
    let food = g.add_token_to_battlefield(0, &crabomination::game::effects::food_token());
    let activate = |g: &mut GameState, id: CardId| {
        flood(g, 0);
        g.priority.player_with_priority = 0;
        let r = g.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        });
        drain_stack(g);
        r
    };
    assert!(activate(&mut g, squirrel).is_err(), "no Food sacrificed yet");
    activate(&mut g, food).expect("eat the Food");
    let hand = g.players[0].hand.len();
    activate(&mut g, squirrel).expect("a Food was sacrificed");
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Werewolf Pack Leader — "{3}{G}: Until end of turn, this creature has base
/// power and toughness 5/3, gains trample…" (CR 613.4b base P/T in layer
/// 7b; counters in 7c still apply on top.)
#[test]
fn werewolf_pack_leader_sets_base_five_three() {
    let mut g = pod(2);
    let wolf = ready(&mut g, 0, catalog::werewolf_pack_leader());
    g.battlefield.iter_mut().find(|c| c.id == wolf).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wolf,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activates");
    drain_stack(&mut g);
    assert_eq!(pt(&g, wolf), (6, 4), "base 5/3 plus the counter");
    assert!(g.computed_permanent(wolf).unwrap().keywords().contains(&Keyword::Trample));
}

/// Brazen Collector — "Whenever this creature attacks, add {R}. Until end of
/// turn, you don't lose this mana as steps and phases end." (CR 500.4 pools
/// empty between steps; the rider exempts this {R}.)
#[test]
fn brazen_collector_mana_survives_step_ends() {
    let mut g = pod(2);
    let bc = ready(&mut g, 0, catalog::brazen_collector());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: bc, target: AttackTarget::Player(1) }])
        .expect("attacks");
    drain_stack(&mut g);
    g.empty_mana_pools();
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "kept through the step end");
}

/// Pride of the Clouds — "gets +1/+1 for each other creature with flying on
/// the battlefield" (CR 613.4c): an opponent's flyer counts too.
#[test]
fn pride_of_the_clouds_counts_every_flyer() {
    let mut g = pod(3);
    let pride = g.add_card_to_battlefield(0, catalog::pride_of_the_clouds());
    g.add_card_to_battlefield(1, catalog::serra_angel());
    g.add_card_to_battlefield(2, catalog::serra_angel());
    assert_eq!(pt(&g, pride), (3, 3));
}

/// Susurian Voidborn — "Whenever this creature or another creature or
/// artifact you control dies, target opponent loses 1 life and you gain 1
/// life." (CR 700.4: an artifact put into a graveyard from the battlefield
/// dies.)
#[test]
fn susurian_voidborn_drains_on_an_artifact_death() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::susurian_voidborn());
    let rock = g.add_card_to_battlefield(0, catalog::mind_stone());
    let mut events = vec![];
    g.sacrifice_one(rock, 0, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19);
    assert_eq!(g.players[0].life, 21);
}

/// Perigee Beckoner — "another target creature you control gets +2/+0 and
/// gains 'When this creature dies, return it to the battlefield tapped under
/// its owner's control.'" (CR 603.6c leaves-the-battlefield trigger.)
#[test]
fn perigee_beckoner_returns_the_target_tapped() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let horror = g.add_card_to_hand(0, catalog::perigee_beckoner());
    cast(&mut g, horror, &[perm(bear)]).expect("castable");
    assert_eq!(pt(&g, bear), (4, 2));
    let mut events = vec![];
    g.sacrifice_one(bear, 0, &mut events);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear && c.tapped), "back, tapped");
}

/// Dropkick Bomber — "{R}: Until end of turn, another target Goblin you
/// control gains flying and 'When this creature deals combat damage,
/// sacrifice it.'" (CR 510.3a: the trigger fires off the combat damage.)
#[test]
fn dropkick_bomber_goblin_flies_and_is_sacrificed() {
    let mut g = pod(2);
    let bomber = ready(&mut g, 0, catalog::dropkick_bomber());
    let gob = ready(&mut g, 0, catalog::raging_goblin());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bomber,
        ability_index: 0,
        target: Some(perm(gob)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activates");
    drain_stack(&mut g);
    assert!(g.computed_permanent(gob).unwrap().keywords().contains(&Keyword::Flying));
    declare(&mut g, 0, &[(gob, 1)]).unwrap();
    block(&mut g, 1, vec![]).unwrap();
    combat_damage(&mut g);
    assert_eq!(g.players[1].life, 18, "2/2 with the Bomber's lord bonus");
    assert!(g.battlefield.iter().all(|c| c.id != gob), "sacrificed after dealing damage");
}

/// Salvation Swan — "exile up to one target creature you control without
/// flying. Return it to the battlefield under its owner's control with a
/// flying counter on it at the beginning of the next end step." (CR 122.1b
/// keyword counters; no +1/+1 counter is printed.)
#[test]
fn salvation_swan_returns_with_a_flying_counter() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let swan = g.add_card_to_hand(0, catalog::salvation_swan());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: swan,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "blinked");
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let back = g.computed_permanent(bear).expect("returned");
    assert!(back.keywords().contains(&Keyword::Flying));
    assert_eq!(pt(&g, bear), (2, 2), "no +1/+1 counter");
}

/// Jace Reawakened — "You can't cast Jace Reawakened during your first,
/// second, or third turns of the game." (CR 601.2 cast restriction; extra
/// turns count as turns begun.)
#[test]
fn jace_reawakened_waits_for_your_fourth_turn() {
    let mut g = pod(2);
    g.starting_player = 1;
    let jace = g.add_card_to_hand(0, catalog::jace_reawakened());
    g.players[0].turns_begun = 3;
    assert!(cast(&mut g, jace, &[]).is_err(), "third turn");
    g.players[0].turns_begun = 4;
    cast(&mut g, jace, &[]).expect("fourth turn");
    assert!(g.battlefield.iter().any(|c| c.id == jace));
}

/// Hazoret, Godseeker — "Hazoret can't attack or block unless you have max
/// speed." (CR 702.179 speed; max speed is 4.)
#[test]
fn hazoret_godseeker_attacks_only_at_max_speed() {
    let mut g = pod(2);
    let god = ready(&mut g, 0, catalog::hazoret_godseeker());
    assert!(declare(&mut g, 0, &[(god, 1)]).is_err(), "speed below 4");
    g.players[0].speed = 4;
    declare(&mut g, 0, &[(god, 1)]).expect("max speed");
}

/// Far Fortune, End Boss — "Max speed — If a source you control would deal
/// damage to an opponent …, it deals that much damage plus 1 instead." (CR
/// 614.1a.)
#[test]
fn far_fortune_adds_one_at_max_speed() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::far_fortune_end_boss());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].speed = 4;
    cast(&mut g, bolt, &[Target::Player(1)]).unwrap();
    assert_eq!(g.players[1].life, 16);
}

/// Cynical Loner — "This creature can't be blocked by Glimmers." (CR 509.1b.)
#[test]
fn cynical_loner_slips_past_glimmers() {
    let mut g = pod(2);
    let loner = ready(&mut g, 0, catalog::cynical_loner());
    let mut glimmer = catalog::grizzly_bears();
    glimmer.subtypes.creature_types = vec![crabomination::card::CreatureType::Glimmer];
    let wall = ready(&mut g, 1, glimmer);
    declare(&mut g, 0, &[(loner, 1)]).unwrap();
    assert!(block(&mut g, 1, vec![(wall, loner)]).is_err());
}

/// Asinine Antics — "You may cast this spell as though it had flash if you pay
/// {2} more to cast it." (CR 601.2f.) Six mana casts it on an opponent's turn.
#[test]
fn asinine_antics_flashes_for_two_more() {
    let mut g = pod(2);
    g.active_player_idx = 1;
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let antics = g.add_card_to_hand(0, catalog::asinine_antics());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: antics, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .is_err(),
        "five mana isn't enough at instant speed"
    );
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: antics, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("six mana flashes it in");
}

/// CR 104.4b / CR 614.6 — "reveal until a land" over a landless library
/// whose cards shuffle back instead of reaching the graveyard (Nexus of
/// Fate) must end. Consuming Aberration's cast trigger used to loop until a
/// bot look-ahead was OOM-killed (6-seat pod, seed 990884); the reveal is
/// capped at the library's starting size.
#[test]
fn mill_until_land_ends_on_a_shuffle_back_library() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::consuming_aberration());
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::nexus_of_fate());
    }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Player(1)]).unwrap();
    assert_eq!(g.players[1].library.len(), 3, "each Nexus shuffled back; the reveal stopped");
}

/// Bloodsoaked Insight — "This spell costs {1} less to cast for each 1 life
/// your opponents have lost this turn" (CR 601.2f; summed over every
/// opponent in a pod), and "target opponent" can't be you.
#[test]
fn bloodsoaked_insight_discounts_by_total_opponent_life_loss() {
    let mut g = pod(3);
    g.players[1].life_lost_this_turn = 2;
    g.players[2].life_lost_this_turn = 3;
    for p in [1, 2] {
        for _ in 0..3 {
            g.add_card_to_library(p, catalog::island());
        }
    }
    let insight = g.add_card_to_hand(0, catalog::bloodsoaked_insight());
    g.players[0].mana_pool.add(Color::Black, 2);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: insight, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
        })
        .is_err(),
        "target opponent"
    );
    g.perform_action(GameAction::CastSpell {
        card_id: insight, target: Some(Target::Player(2)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("five life lost pays the generic five");
    drain_stack(&mut g);
    assert_eq!(g.players[2].library.len(), 0, "the target's top three are exiled");
}

/// Blitzball — "Activate only if an opponent was dealt combat damage by a
/// legendary creature this turn." (CR 602.5b.) A nonlegendary hit doesn't
/// open it; a legendary one does.
#[test]
fn blitzball_needs_a_legendary_hit() {
    let mut g = pod(2);
    let ball = ready(&mut g, 0, catalog::blitzball());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    declare(&mut g, 0, &[(bear, 1)]).unwrap();
    block(&mut g, 1, vec![]).unwrap();
    combat_damage(&mut g);
    let sac = |g: &mut GameState| {
        g.step = TurnStep::PostCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: ball, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None,
        })
    };
    assert!(sac(&mut g).is_err(), "the Bear isn't legendary");
    let legend = ready(&mut g, 0, catalog::isamaru_hound_of_konda());
    g.players[1].creatures_that_combat_damaged_me_this_turn.push(legend);
    sac(&mut g).expect("a legendary creature connected");
}

/// Country Roads — "This land enters tapped unless you control a Mount or a
/// Vehicle." (CR 614.1c.)
#[test]
fn country_roads_enters_tapped_without_a_vehicle() {
    let mut g = pod(2);
    let land = g.add_card_to_hand(0, catalog::country_roads());
    g.perform_action(GameAction::PlayLand(land)).unwrap();
    assert!(g.battlefield.iter().any(|c| c.id == land && c.tapped));
}

/// Careful Consideration — "If you cast this spell during your main phase,
/// instead … discards two cards." (CR 505.1: an opponent's main phase isn't
/// yours.) Cast in an opponent's main phase it discards three.
#[test]
fn careful_consideration_discount_needs_your_main_phase() {
    let mut g = pod(2);
    g.active_player_idx = 1;
    g.players[0].hand.clear();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let cc = g.add_card_to_hand(0, catalog::careful_consideration());
    cast(&mut g, cc, &[Target::Player(0)]).expect("instant");
    assert_eq!(g.players[0].hand.len(), 1, "drew four, discarded three");
}

/// CR 115.1 — Careful Consideration's "target player" can be an opponent: they
/// draw four and discard (two — it's the caster's main phase that counts).
#[test]
fn careful_consideration_targets_a_player() {
    let mut g = pod(2);
    g.players[1].hand.clear();
    for _ in 0..4 {
        g.add_card_to_library(1, catalog::island());
    }
    let mine = g.players[0].hand.len();
    let cc = g.add_card_to_hand(0, catalog::careful_consideration());
    cast(&mut g, cc, &[Target::Player(1)]).expect("cast at seat 1");
    assert_eq!(g.players[1].hand.len(), 2, "they drew four and discarded two");
    assert_eq!(g.players[0].hand.len(), mine, "the caster drew nothing");
}

/// Nightveil Specter — "You may play lands and cast spells from among cards
/// exiled with this creature." (CR 607.2a linked abilities: the permission
/// ends when the Specter leaves.)
#[test]
fn nightveil_specter_permission_ends_when_it_leaves() {
    let mut g = pod(2);
    let specter = ready(&mut g, 0, catalog::nightveil_specter());
    let top = g.add_card_to_library(1, catalog::grizzly_bears());
    declare(&mut g, 0, &[(specter, 1)]).unwrap();
    block(&mut g, 1, vec![]).unwrap();
    combat_damage(&mut g);
    let perm = g.exile.iter().find(|c| c.id == top).and_then(|c| c.may_play_until).expect("exiled, playable");
    assert_eq!(perm.player, 0);
    let mut events = vec![];
    g.sacrifice_one(specter, 0, &mut events);
    g.dispatch_triggers_for_events(&events);
    assert!(
        g.exile.iter().find(|c| c.id == top).unwrap().may_play_until.is_none(),
        "the link ends with the Specter"
    );
}

/// Momentum Breaker — "each opponent sacrifices a creature or Vehicle of
/// their choice. Each opponent who can't discards a card." (CR 101.4 APNAP.)
#[test]
fn momentum_breaker_discards_from_an_empty_board() {
    let mut g = pod(3);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[2].hand.clear();
    g.add_card_to_hand(2, catalog::island());
    let mb = g.add_card_to_hand(0, catalog::momentum_breaker());
    cast(&mut g, mb, &[]).unwrap();
    assert!(g.battlefield.iter().all(|c| c.id != bear), "seat 1 sacrificed its Bear");
    assert!(g.players[2].hand.is_empty(), "seat 2 had nothing to sacrifice and discarded");
}

/// Vampire Socialite — "As long as an opponent has lost life this turn, each
/// other Vampire you control enters with an additional +1/+1 counter on it."
/// (CR 614.1c.)
#[test]
fn vampire_socialite_vampires_enter_bigger_after_life_loss() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::vampire_socialite());
    g.players[1].lost_life_this_turn = true;
    let mut vamp = catalog::grizzly_bears();
    vamp.subtypes.creature_types = vec![crabomination::card::CreatureType::Vampire];
    let v = g.add_card_to_hand(0, vamp);
    cast(&mut g, v, &[]).unwrap();
    assert_eq!(pt(&g, v), (3, 3));
}

/// Caradora, Heart of Alacria — "If one or more +1/+1 counters would be put
/// on a creature or Vehicle you control, that many plus one … instead."
/// (CR 614.1a.) An uncrewed Vehicle gets the extra counter too.
#[test]
fn caradora_adds_a_counter_to_a_vehicle() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::caradora_heart_of_alacria());
    let ship = g.add_card_to_battlefield(0, catalog::subterranean_schooner());
    let ctx = crabomination::game::effects::EffectContext::for_ability(ship, 0, None);
    g.resolve_effect(
        &crabomination::effect::Effect::AddCounter {
            what: crabomination::effect::Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: crabomination::effect::Value::ONE,
        },
        &ctx,
    )
    .unwrap();
    assert_eq!(
        g.battlefield.iter().find(|c| c.id == ship).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2
    );
}

/// Slicer, Hired Muscle — "If you do, untap Slicer, goad it, and it can't be
/// sacrificed this turn." (CR 701.21 sacrifice.)
#[test]
fn slicer_rented_out_cant_be_sacrificed() {
    let mut g = pod(2);
    let slicer = ready(&mut g, 0, catalog::slicer_hired_muscle());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let s = g.computed_permanent(slicer).unwrap();
    assert_eq!(s.controller, 1, "rented out");
    assert!(s.keywords().contains(&Keyword::CantBeSacrificed));
}

/// Zurgo, Thunder's Decree — "During your end step, Warrior tokens you control
/// have 'This token can't be sacrificed.'" (CR 701.21a.) Mobilize's end-step
/// sacrifice leaves the tokens.
#[test]
fn zurgo_keeps_its_mobilized_warriors() {
    let mut g = pod(2);
    let zurgo = ready(&mut g, 0, catalog::zurgo_thunders_decree());
    declare(&mut g, 0, &[(zurgo, 1)]).unwrap();
    let warriors = || {
        |g: &GameState| g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Warrior").count()
    };
    assert_eq!(warriors()(&g), 2, "mobilize 2");
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(warriors()(&g), 2, "can't be sacrificed during your end step");
    // Control: without Zurgo, Mobilize's token goes at the end step.
    let mut g = pod(2);
    let vanguard = ready(&mut g, 0, catalog::zurgos_vanguard());
    declare(&mut g, 0, &[(vanguard, 1)]).unwrap();
    assert_eq!(warriors()(&g), 1);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(warriors()(&g), 0, "sacrificed");
}

/// Magmatic Galleon — "Whenever one or more creatures your opponents control
/// are dealt excess noncombat damage, create a Treasure token." (CR 120.4a
/// excess damage.)
#[test]
fn magmatic_galleon_makes_treasure_on_excess_damage() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::magmatic_galleon());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[perm(bear)]).unwrap();
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count(), 1);
}

/// Quilled Greatwurm — "Whenever a creature you control deals combat damage
/// during your turn, put that many +1/+1 counters on it." (CR 510.2: damage to
/// a blocker counts too.)
#[test]
fn quilled_greatwurm_counts_damage_to_blockers() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::quilled_greatwurm());
    let tusker = ready(&mut g, 0, catalog::kalonian_tusker());
    let chump = ready(&mut g, 1, catalog::llanowar_elves());
    declare(&mut g, 0, &[(tusker, 1)]).unwrap();
    block(&mut g, 1, vec![(chump, tusker)]).unwrap();
    combat_damage(&mut g);
    assert_eq!(pt(&g, tusker), (6, 6), "three counters from the blocked hit");
}

/// Quilled Greatwurm — "You may cast this card from your graveyard by
/// removing six counters from among creatures you control" (CR 601.2b, an
/// additional cost): five counters can't pay it, six can and are spent.
#[test]
fn quilled_greatwurm_casts_from_the_graveyard_for_six_counters() {
    let mut g = pod(2);
    let wurm = g.add_card_to_graveyard(0, catalog::quilled_greatwurm());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 5);
    assert!(cast(&mut g, wurm, &[]).is_err(), "five counters short");
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    cast(&mut g, wurm, &[]).expect("cast from the graveyard");
    assert!(g.battlefield_find(wurm).is_some());
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Long River Lurker — "Whenever that creature deals combat damage this turn,
/// you may exile it. If you do, return it to the battlefield under its owner's
/// control." (CR 603.6a; the returned creature is untapped.)
#[test]
fn long_river_lurker_blinks_the_unblockable_creature() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let lurker = g.add_card_to_hand(0, catalog::long_river_lurker());
    cast(&mut g, lurker, &[perm(bear)]).unwrap();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    declare(&mut g, 0, &[(bear, 1)]).unwrap();
    block(&mut g, 1, vec![]).unwrap();
    combat_damage(&mut g);
    assert_eq!(g.players[1].life, 18);
    let b = g.battlefield.iter().find(|c| c.id == bear).expect("back on the battlefield");
    assert!(!b.tapped, "returned untapped");
}

/// Zero Point Ballad — "If X is 6 or more, return a creature card put into a
/// graveyard this way to the battlefield under your control." (CR 608.2d.)
#[test]
fn zero_point_ballad_reanimates_at_x_six() {
    let mut g = pod(2);
    let theirs = g.add_card_to_battlefield(1, catalog::hill_giant());
    let zpb = g.add_card_to_hand(0, catalog::zero_point_ballad());
    cast_full(&mut g, 0, zpb, &[], None, Some(6)).unwrap();
    let back = g.battlefield.iter().find(|c| c.id == theirs).expect("returned");
    assert_eq!(back.controller, 0, "under your control");
    // X below six returns nothing.
    let mut g = pod(2);
    let theirs = g.add_card_to_battlefield(1, catalog::hill_giant());
    let zpb = g.add_card_to_hand(0, catalog::zero_point_ballad());
    cast_full(&mut g, 0, zpb, &[], None, Some(5)).unwrap();
    assert!(g.battlefield.iter().all(|c| c.id != theirs));
}

/// Clement, the Worrywort — "Frogs you control have '{T}: Add {G} or {U}.
/// Spend this mana only to cast a creature spell.'" (CR 106.6 mana spending
/// restriction.) Clement is a Frog: it taps itself for creature-only mana.
#[test]
fn clement_frogs_tap_for_creature_mana() {
    let mut g = pod(2);
    let clement = ready(&mut g, 0, catalog::clement_the_worrywort());
    assert_eq!(g.granted_abilities_for(clement).len(), 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: clement,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("granted mana ability");
    assert_eq!(g.players[0].mana_pool.restricted_total(), 1, "one creature-only mana");
    let bolt = g.add_card_to_hand(0, catalog::giant_growth());
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(perm(clement)), additional_targets: vec![], mode: None, x_value: None,
        })
        .is_err(),
        "creature-spell-only mana"
    );
}

/// Invasion of Zendikar // Awakened Skyclave — "As long as this creature is
/// on the battlefield, it's a land in addition to its other types." (CR
/// 205.1b.)
#[test]
fn awakened_skyclave_is_a_land_creature() {
    let mut g = pod(2);
    let def = catalog::invasion_of_zendikar();
    let back = (*def.back_face.clone().unwrap()).clone();
    let sky = g.add_card_to_battlefield(0, back);
    let cp = g.computed_permanent(sky).unwrap();
    assert!(cp.card_types().contains(&crabomination::card::CardType::Land));
    assert!(cp.card_types().contains(&crabomination::card::CardType::Creature));
}

/// Case of the Uneaten Feast — "Solved — Sacrifice this Case: Creature cards
/// in your graveyard gain 'You may cast this card from your graveyard' until
/// end of turn." (CR 601.2; no exile rider, unlike flashback.)
#[test]
fn case_of_the_uneaten_feast_lets_graveyard_creatures_be_cast() {
    let mut g = pod(2);
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_uneaten_feast());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let def = catalog::case_of_the_uneaten_feast();
    let solved = &def.case.as_ref().unwrap().solved_activated[0];
    g.resolve_effect(&solved.effect, &crabomination::game::effects::EffectContext::for_ability(case, 0, None))
        .unwrap();
    let perm = g.players[0].graveyard.iter().find(|c| c.id == bear).unwrap().may_play_until;
    assert!(perm.is_some_and(|p| p.player == 0), "castable from the graveyard this turn");
}

/// Obosh, the Preypiercer — "If a source you control with an odd mana value
/// would deal damage to a permanent or player, it deals double that damage
/// instead." (CR 614.1a.) An odd-mana-value attacker deals double; an even
/// one doesn't.
#[test]
fn obosh_doubles_odd_mana_value_sources() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::obosh_the_preypiercer());
    let elf = ready(&mut g, 0, catalog::llanowar_elves());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    declare(&mut g, 0, &[(elf, 1), (bear, 1)]).unwrap();
    block(&mut g, 1, vec![]).unwrap();
    combat_damage(&mut g);
    assert_eq!(g.players[1].life, 20 - 2 - 2, "the Elf (MV 1) doubles to 2, the Bear (MV 2) stays 2");
}

/// Genku, Future Shaper — "create a token … that hasn't been chosen this
/// turn" (CR 700.2 modal choice with a per-turn memory).
#[test]
fn genku_rotates_its_token_choices() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::genku_future_shaper());
    for _ in 0..3 {
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().damage = 99;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    let names: std::collections::BTreeSet<_> = g
        .battlefield
        .iter()
        .filter(|c| c.is_token)
        .map(|c| c.definition.name.to_string())
        .collect();
    assert_eq!(names.len(), 3, "Fox, Moonfolk and Rat once each: {names:?}");
}

/// Death's Oasis — "return a creature card with lesser mana value than the
/// creature that died from your graveyard to your hand." The card is the
/// controller's choice (CR 608.2d): a scripted pick takes the cheaper one
/// over the engine's priciest default.
#[test]
fn deaths_oasis_return_is_the_controllers_pick() {
    let mut g = pod(2);
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let elf = g.add_card_to_graveyard(0, catalog::llanowar_elves());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::deaths_oasis());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![elf])]));
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, &[perm(giant)]).expect("murder");
    assert!(g.players[0].hand.iter().any(|c| c.id == elf), "the chosen Elf");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"));
}

/// Nahiri, Forged in Fury — "You may play that card this turn. You may cast
/// Equipment spells this way without paying their mana costs." (CR 118.9.)
/// An exiled Equipment casts with no mana; anything else still pays.
#[test]
fn nahiri_casts_exiled_equipment_free() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::nahiri_forged_in_fury());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let sword = g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(bear);
    let top = g.add_card_to_library(0, catalog::bonesplitter());
    declare(&mut g, 0, &[(bear, 1)]).unwrap();
    let ex = g.exile.iter().find(|c| c.id == top).expect("exiled");
    let perm = ex.may_play_until.expect("granted");
    assert!(perm.player == 0 && !perm.cast_only, "{perm:?}");
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool = Default::default();
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: top, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("free Equipment");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == top));
}

/// CR 115.1 — Press the Enemy has ONE target, "spell or nonland permanent an
/// opponent controls" (it was two modes, one per kind): the same cast takes an
/// opponent's spell off the stack or a permanent off the battlefield.
#[test]
fn press_the_enemy_returns_a_spell_or_a_permanent_in_one_slot() {
    let mut g = pod(3);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    flood(&mut g, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast the bear");
    let press = g.add_card_to_hand(0, catalog::press_the_enemy());
    cast(&mut g, press, &[perm(bear)]).expect("Press on the spell");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "the spell went back to hand");
    assert!(g.battlefield_find(bear).is_none(), "and never resolved");

    let angel = ready(&mut g, 2, catalog::serra_angel());
    let press = g.add_card_to_hand(0, catalog::press_the_enemy());
    cast(&mut g, press, &[perm(angel)]).expect("Press on the permanent");
    assert!(g.players[2].hand.iter().any(|c| c.id == angel), "the permanent went back to hand");
}

/// Sorcerous Squall: "Target opponent mills nine cards, then you may cast an
/// instant or sorcery spell from that player's graveyard without paying its
/// mana cost. If that spell would be put into a graveyard, exile it instead."
/// The card itself is cast (not a copy), from the milled player's graveyard
/// only — a third seat's instant is not offered (it had been, at N > 2).
#[test]
fn sorcerous_squall_casts_from_the_milled_players_graveyard() {
    let mut g = pod(3);
    let theirs = g.add_card_to_graveyard(2, catalog::divination());
    g.players[1].library.clear();
    let milled = g.add_card_to_library(1, catalog::divination());
    for _ in 0..8 {
        g.add_card_to_library(1, catalog::forest());
    }
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand = g.players[0].hand.len();
    let squall = g.add_card_to_hand(0, catalog::sorcerous_squall());
    cast(&mut g, squall, &[Target::Player(1)]).expect("cast Sorcerous Squall");
    assert_eq!(g.players[0].hand.len(), hand + 2, "the milled Divination drew two for its caster");
    assert!(g.exile.iter().any(|c| c.id == milled), "cast from seat 1's graveyard, then exiled");
    assert!(g.players[2].graveyard.iter().any(|c| c.id == theirs), "seat 2's instant untouched");
}
