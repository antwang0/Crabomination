//! Commander: the 2026-10-17 (`3chree`) third closer batch
//! (`decks::cmdr_oct17_s`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].hostile_player_targets = true;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(6);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn step_to(g: &mut GameState, seat: usize, step: TurnStep) {
    for _ in 0..200 {
        if g.active_player_idx == seat && g.step == step {
            return;
        }
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
    panic!("never reached seat {seat} {step:?}");
}






/// Mirror-Sigil Sergeant copies itself at upkeep only while you control a blue
/// permanent.
#[test]
fn mirror_sigil_sergeant_copies_with_blue() {
    let count = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Mirror-Sigil Sergeant").count();
    let mut g = pod(2);
    ready(&mut g, 0, catalog::mirror_sigil_sergeant());
    ready(&mut g, 0, catalog::storm_crow());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    step_to(&mut g, 0, TurnStep::Upkeep);
    assert_eq!(count(&g), 2, "a blue permanent: one token copy");
    let mut g = pod(2);
    ready(&mut g, 0, catalog::mirror_sigil_sergeant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    step_to(&mut g, 0, TurnStep::Upkeep);
    assert_eq!(count(&g), 1, "no blue permanent, no copy");
}

/// Secrets of the Golden City draws two, or three once ten permanents give
/// the city's blessing (CR 702.131a — ascend on a spell).
#[test]
fn secrets_of_the_golden_city_draws_three_with_blessing() {
    let mut g = pod(2);
    let s = g.add_card_to_hand(0, catalog::secrets_of_the_golden_city());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    cast(&mut g, s, None).expect("secrets");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2);
    assert!(!g.players[0].city_blessing);
    let mut g = pod(2);
    for _ in 0..10 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let s = g.add_card_to_hand(0, catalog::secrets_of_the_golden_city());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    cast(&mut g, s, None).expect("secrets");
    assert!(g.players[0].city_blessing);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3);
}

/// Magmatic Channeler is 4/4 with four instants in the graveyard; its
/// activation discards, exiles two and grants play on exactly one.
#[test]
fn magmatic_channeler_grows_and_digs_two() {
    let mut g = pod(2);
    let ch = ready(&mut g, 0, catalog::magmatic_channeler());
    assert_eq!(g.computed_permanent(ch).unwrap().power, 1);
    for _ in 0..4 {
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
    }
    let cp = g.computed_permanent(ch).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    g.add_card_to_hand(0, catalog::forest());
    let (hand, exile) = (g.players[0].hand.len(), g.exile.len());
    g.perform_action(GameAction::ActivateAbility { card_id: ch, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("tap, discard");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1, "discarded a card");
    assert_eq!(g.exile.len(), exile + 2);
    assert_eq!(g.exile.iter().filter(|c| c.may_play_until.is_some()).count(), 1, "only the chosen one is playable");
}

/// Twinned Vision draws one from hand, two when flashed back (discarding a
/// card), then is exiled.
#[test]
fn twinned_vision_draws_two_on_flashback() {
    let mut g = pod(2);
    let tv = g.add_card_to_hand(0, catalog::twinned_vision());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    cast(&mut g, tv, None).expect("from hand");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 1);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == tv));
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastFlashback { card_id: tv, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("flashback, discarding the drawn card");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "discard one, draw two");
    assert!(g.exile.iter().any(|c| c.id == tv));
}

/// Choose Your Weapon: mode 0 doubles a 2/2 to 4/4; mode 1 can only target a
/// flier, and kills it.
#[test]
fn choose_your_weapon_doubles_or_shoots_a_flier() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let crow = ready(&mut g, 1, catalog::storm_crow());
    let go = |g: &mut GameState, t: CardId, mode: usize| -> Result<(), GameError> {
        let s = g.add_card_to_hand(0, catalog::choose_your_weapon());
        flood(g, 0);
        g.perform_action(GameAction::CastSpell { card_id: s, target: Some(Target::Permanent(t)), additional_targets: vec![], mode: Some(mode), x_value: None })?;
        drain_stack(g);
        Ok(())
    };
    go(&mut g, bear, 0).expect("double");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(go(&mut g, bear, 1).is_err(), "the bear has no flying");
    go(&mut g, crow, 1).expect("shoot the crow");
    assert!(g.battlefield_find(crow).is_none());
}

/// Arcum's Astrolabe needs snow mana for its {S} and draws on entry.
#[test]
fn arcums_astrolabe_costs_snow_and_cantrips() {
    let mut g = pod(2);
    let a = g.add_card_to_hand(0, catalog::arcums_astrolabe());
    g.players[0].mana_pool.add(Color::Blue, 2);
    assert!(cast(&mut g, a, None).is_err(), "non-snow mana can't pay {{S}}");
    g.players[0].mana_pool.add_snow(Color::Blue, 1);
    let hand = g.players[0].hand.len();
    cast(&mut g, a, None).expect("snow pays {S}");
    assert!(g.battlefield_find(a).is_some());
    assert_eq!(g.players[0].hand.len(), hand - 1 + 1, "ETB draw");
}


fn activate(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: 0, target, additional_targets: vec![], x_value: None, mode: None })?;
    drain_stack(g);
    Ok(())
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

/// Favor of Jukai: attached, +3/+3 and reach; channelled from hand, the same
/// until end of turn and the card goes to the graveyard.
#[test]
fn favor_of_jukai_aura_and_channel() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::favor_of_jukai());
    flood(&mut g, 0);
    cast(&mut g, aura, Some(Target::Permanent(bear))).expect("aura");
    assert_eq!(pt(&g, bear), (5, 5));
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Reach));
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let card = g.add_card_to_hand(0, catalog::favor_of_jukai());
    flood(&mut g, 0);
    activate(&mut g, card, Some(Target::Permanent(bear))).expect("channel");
    assert_eq!(pt(&g, bear), (5, 5));
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Reach));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == card), "discarded as the cost");
}

/// Twisted Fates: the permanent dies, and only the chosen player's creatures
/// get a counter.
#[test]
fn twisted_fates_destroys_and_counters_chosen_player() {
    let mut g = pod(3);
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let other = ready(&mut g, 2, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::twisted_fates());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: s,
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![Target::Player(0)],
        mode: None,
        x_value: None,
    })
    .expect("fates");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none());
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(other).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}


/// Fateshaper Aspirant's first mode returns the legendary card only.
#[test]
fn fateshaper_aspirant_returns_a_legend() {
    let mut g = pod(2);
    let legend = g.add_card_to_graveyard(0, catalog::isamaru_hound_of_konda());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let a = g.add_card_to_hand(0, catalog::fateshaper_aspirant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    flood(&mut g, 0);
    cast(&mut g, a, None).expect("aspirant");
    assert!(g.players[0].hand.iter().any(|c| c.id == legend));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear), "non-legendary stays");
}

/// Snowslope Hunter: sacrifice an artifact to impulse the top card; a second
/// activation the same turn is refused.
#[test]
fn snowslope_hunter_impulses_once_per_turn() {
    let mut g = pod(2);
    let hunter = ready(&mut g, 0, catalog::snowslope_hunter());
    let thopter = ready(&mut g, 0, catalog::ornithopter());
    ready(&mut g, 0, catalog::memnite());
    let id = g.next_id();
    g.players[0].add_to_library_top(id, catalog::lightning_bolt());
    activate(&mut g, hunter, None).expect("sac the thopter");
    assert!(g.battlefield_find(thopter).is_none());
    assert!(g.exile.iter().any(|c| c.id == id && c.may_play_until.is_some()));
    assert!(activate(&mut g, hunter, None).is_err(), "only once each turn");
}

/// Singing Bell Strike taps and locks the creature; its granted {6} untaps it.
#[test]
fn singing_bell_strike_locks_until_six_paid() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::singing_bell_strike());
    flood(&mut g, 0);
    cast(&mut g, aura, Some(Target::Permanent(bear))).expect("aura");
    assert!(g.battlefield_find(bear).unwrap().tapped);
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(bear).unwrap().tapped, "doesn't untap");
    g.players[1].mana_pool.add_colorless(6);
    g.priority.player_with_priority = 1;
    activate(&mut g, bear, None).expect("{6}: untap");
    assert!(!g.battlefield_find(bear).unwrap().tapped);
}

/// Underhanded Designs: paying {1} on an artifact entering drains; the
/// sacrifice kill needs two artifacts.
#[test]
fn underhanded_designs_drains_and_kills_with_two_artifacts() {
    let mut g = pod(2);
    let ud = ready(&mut g, 0, catalog::underhanded_designs());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let thopter = g.add_card_to_hand(0, catalog::ornithopter());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    flood(&mut g, 0);
    let (me, foe) = (g.players[0].life, g.players[1].life);
    cast(&mut g, thopter, None).expect("thopter");
    assert_eq!((g.players[0].life, g.players[1].life), (me + 1, foe - 1));
    assert!(activate(&mut g, ud, Some(Target::Permanent(bear))).is_err(), "one artifact");
    assert!(g.battlefield_find(ud).is_some(), "gate checked before the sacrifice");
    ready(&mut g, 0, catalog::memnite());
    activate(&mut g, ud, Some(Target::Permanent(bear))).expect("two artifacts");
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(ud).is_none());
}

/// Last One Standing: exactly one creature survives, across both sides.
#[test]
fn last_one_standing_leaves_one_creature() {
    let mut g = pod(2);
    for seat in [0, 1, 1] {
        ready(&mut g, seat, catalog::grizzly_bears());
    }
    ready(&mut g, 0, catalog::serra_angel());
    let s = g.add_card_to_hand(0, catalog::last_one_standing());
    flood(&mut g, 0);
    cast(&mut g, s, None).expect("cast");
    let alive = g.battlefield.iter().filter(|c| c.definition.card_types.contains(&crabomination::card::CardType::Creature)).count();
    assert_eq!(alive, 1);
}



/// Rewrite Regrets: MV 6 comes back with a 2-loyalty Jace; MV 8 is no target.
#[test]
fn rewrite_regrets_reanimates_and_empowers_jace() {
    let mut g = pod(2);
    let big = g.add_card_to_graveyard(0, catalog::ancient_brontodon());
    let s = g.add_card_to_hand(0, catalog::rewrite_regrets());
    flood(&mut g, 0);
    assert!(cast(&mut g, s, Some(Target::Permanent(big))).is_err(), "MV 8 too big");
    let dragon = g.add_card_to_graveyard(0, catalog::shivan_dragon());
    cast(&mut g, s, Some(Target::Permanent(dragon))).expect("cast");
    assert!(g.battlefield_find(dragon).is_some());
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.counter_count(CounterType::Loyalty) == 2));
}

/// Armguard Familiar reconfigured: host gets +2/+1 and ward.
#[test]
fn armguard_familiar_reconfigures_for_ward() {
    let mut g = pod(2);
    let fam = ready(&mut g, 0, catalog::armguard_familiar());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::Reconfigure { equipment: fam, target: Some(bear) }).expect("reconfigure");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (4, 3));
    assert!(c.keywords().iter().any(|k| matches!(k, Keyword::Ward(_))));
}

/// Warrior's Blades: ETB drains 3; equip costs {1} onto a creature with two
/// +1/+1 counters, and the bonus is +2/+1.
#[test]
fn warriors_blades_burns_and_equips_cheaply() {
    let mut g = pod(2);
    let bl = g.add_card_to_hand(0, catalog::warriors_blades());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let (me, opp) = (g.players[0].life, g.players[1].life);
    cast(&mut g, bl, None).expect("cast");
    assert_eq!((g.players[0].life, g.players[1].life), (me + 3, opp - 3));
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: bl, target: bear }).expect("equip for {1}");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (6, 5));
}

/// Veilstone Amulet: after a spell, an opponent's Bolt can't target your
/// creature — including one that entered afterwards (a rules change, not a
/// granted keyword); your own spells still can.
#[test]
fn veilstone_amulet_shields_from_opponents_only() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::veilstone_amulet());
    let opt = g.add_card_to_hand(0, catalog::opt());
    flood(&mut g, 0);
    cast(&mut g, opt, None).expect("opt");
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    let r = g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None });
    assert!(r.is_err(), "an opponent can't target it");
    g.priority.player_with_priority = 0;
    let mine = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, mine, Some(Target::Permanent(bear))).expect("its controller can");
    assert!(g.battlefield_find(bear).is_none());
}

/// Light from Within (chroma): +1/+1 per {W} in each creature's own cost.
#[test]
fn light_from_within_counts_white_pips() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::light_from_within());
    let knight = ready(&mut g, 0, catalog::white_knight());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let pt = |g: &GameState, id| {
        let c = g.computed_permanent(id).unwrap();
        (c.power, c.toughness)
    };
    assert_eq!(pt(&g, knight), (4, 4), "{{W}}{{W}}: +2/+2");
    assert_eq!(pt(&g, bear), (2, 2), "no white pips");
}

/// Corpse Explosion exiles the biggest creature card as its cost and deals its
/// power to every creature; it can't be cast with none.
#[test]
fn corpse_explosion_reads_the_exiled_power() {
    let mut g = pod(2);
    let ce = g.add_card_to_hand(0, catalog::corpse_explosion());
    flood(&mut g, 0);
    assert!(cast(&mut g, ce, None).is_err(), "no creature card to exile");
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    let theirs = ready(&mut g, 1, catalog::hill_giant());
    cast(&mut g, ce, None).expect("corpse explosion");
    assert!(g.exile.iter().any(|c| c.id == giant), "the 3-power card paid");
    assert!(g.battlefield_find(theirs).is_none(), "3 damage kills a 3/3");
}

/// Command the Stage: the second Cadet grows the first, and the card comes
/// back at upkeep after an opponent took noncombat damage the turn before.
#[test]
fn command_the_stage_returns_after_noncombat_damage() {
    let mut g = pod(2);
    let cs = g.add_card_to_hand(0, catalog::command_the_stage());
    flood(&mut g, 0);
    cast(&mut g, cs, None).expect("first");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == cs));
    step_to(&mut g, 1, TurnStep::Upkeep);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == cs), "no damage last turn: stays");
    step_to(&mut g, 0, TurnStep::PreCombatMain);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
    step_to(&mut g, 1, TurnStep::Draw);
    assert!(g.players[0].hand.iter().any(|c| c.id == cs), "returned at the next upkeep");
    step_to(&mut g, 0, TurnStep::PreCombatMain);
    flood(&mut g, 0);
    cast(&mut g, cs, None).expect("second");
    let mut cadets: Vec<i32> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Cadet")
        .map(|c| g.computed_permanent(c.id).unwrap().power)
        .collect();
    cadets.sort();
    assert_eq!(cadets, vec![2, 3]);
}
