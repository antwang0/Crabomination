//! Commander: two-card gaps of EDHREC ranks 2501-3500 (`decks::cmdr_rank3k_d`).

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

fn cast2(g: &mut GameState, id: CardId, target: Target, second: Target) {
    act(g, GameAction::CastSpell {
        card_id: id,
        target: Some(target),
        additional_targets: vec![second],
        mode: None,
        x_value: None,
    });
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

fn has(g: &GameState, id: CardId, kw: Keyword) -> bool {
    g.computed_permanent(id).unwrap().keywords().contains(&kw)
}

fn kill(g: &mut GameState, id: CardId) {
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(g, murder, Some(Target::Permanent(id)));
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
        // `perform_action` re-dispatches what `advance_step` returns.
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
}

fn fire(g: &mut GameState, step: TurnStep) {
    g.step = step;
    g.fire_step_triggers(step);
    drain_stack(g);
}

/// Conclave Cavalier leaves two Elf Knights; the Guildmage makes one more.
#[test]
fn tolsimir_pair_makes_elf_knights() {
    let mut g = pod(2);
    let cav = ready(&mut g, 0, catalog::conclave_cavalier());
    kill(&mut g, cav);
    assert_eq!(named(&g, 0, "Elf Knight"), 2);
    let mage = ready(&mut g, 0, catalog::conclave_guildmage());
    activate(&mut g, mage, 1, None);
    assert_eq!(named(&g, 0, "Elf Knight"), 3);
}

/// Suki pumps other Warriors and gives attackers double strike.
#[test]
fn suki_lifts_warriors() {
    let mut g = pod(2);
    let suki = ready(&mut g, 0, catalog::suki_kyoshi_captain());
    let warrior = ready(&mut g, 0, catalog::elvish_warrior());
    assert_eq!(pt(&g, warrior), (3, 4));
    assert_eq!(pt(&g, suki), (3, 3), "only other Warriors");
    declare(&mut g, vec![(warrior, 1)]);
    activate(&mut g, suki, 0, None);
    assert!(has(&g, warrior, Keyword::DoubleStrike));
}

/// Inspired Insurgent destroys an artifact.
#[test]
fn inspired_insurgent_breaks_artifacts() {
    let mut g = pod(2);
    let rebel = ready(&mut g, 0, catalog::inspired_insurgent());
    let ring = ready(&mut g, 1, catalog::sol_ring());
    activate(&mut g, rebel, 0, Some(Target::Permanent(ring)));
    assert!(g.battlefield_find(ring).is_none());
}

/// Elven Bow can come with its own Elf; King Harald's Revenge scales with
/// your creatures and must be blocked.
#[test]
fn harald_pair() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bow = g.add_card_to_hand(0, catalog::elven_bow());
    cast(&mut g, bow, None);
    let elf = g.battlefield.iter().find(|c| c.definition.name == "Elf Warrior").expect("Elf").id;
    assert_eq!(pt(&g, elf), (2, 3));
    assert!(has(&g, elf, Keyword::Reach));
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let revenge = g.add_card_to_hand(0, catalog::king_haralds_revenge());
    cast(&mut g, revenge, Some(Target::Permanent(bear)));
    assert_eq!(pt(&g, bear), (4, 4), "+1/+1 per creature: Elf and Bears");
    assert!(has(&g, bear, Keyword::Trample) && has(&g, bear, Keyword::MustBeBlocked));
}

/// Kolaghan Monument becomes a flying Dragon; Boltwing Marauder pumps on an
/// arrival.
#[test]
fn kolaghan_pair() {
    let mut g = pod(2);
    let monument = ready(&mut g, 0, catalog::kolaghan_monument());
    activate(&mut g, monument, 1, None);
    assert_eq!(pt(&g, monument), (4, 4));
    assert!(has(&g, monument, Keyword::Flying));
    let marauder = ready(&mut g, 0, catalog::boltwing_marauder());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None);
    let total: i32 = [marauder, monument, bear].iter().map(|&id| pt(&g, id).0).sum();
    assert_eq!(total, 5 + 4 + 2 + 2, "+2/+0 on one of them");
}

/// Relic Robber hands the damaged player a Goblin Construct; Sardian Avenger
/// pings when an opponent's artifact dies.
#[test]
fn avalanche_pair() {
    let mut g = pod(3);
    let robber = ready(&mut g, 0, catalog::relic_robber());
    declare(&mut g, vec![(robber, 2)]);
    finish_combat(&mut g);
    assert_eq!(named(&g, 2, "Goblin Construct"), 1);
    ready(&mut g, 0, catalog::sardian_avenger());
    let ring = ready(&mut g, 1, catalog::sol_ring());
    let shatter = g.add_card_to_hand(0, catalog::shatter());
    cast(&mut g, shatter, Some(Target::Permanent(ring)));
    assert_eq!(g.players[1].life, 19);
}

/// Cunning Advisor works before attackers only; Wei Night Raiders strips a
/// card on damage.
#[test]
fn cao_cao_pair() {
    let mut g = pod(2);
    let advisor = ready(&mut g, 0, catalog::cunning_advisor());
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_hand(1, catalog::island());
    activate(&mut g, advisor, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].hand.len(), 1);
    let raiders = ready(&mut g, 0, catalog::wei_night_raiders());
    declare(&mut g, vec![(raiders, 1)]);
    finish_combat(&mut g);
    assert_eq!(g.players[1].hand.len(), 0);
}

/// Verdant Rebirth's granted trigger returns the creature; Viridescent Wisps
/// recolors and draws.
#[test]
fn anthousa_pair() {
    let mut g = pod(2);
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let rebirth = g.add_card_to_hand(0, catalog::verdant_rebirth());
    cast(&mut g, rebirth, Some(Target::Permanent(bear)));
    kill(&mut g, bear);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
    let angel = ready(&mut g, 0, catalog::serra_angel());
    let wisps = g.add_card_to_hand(0, catalog::viridescent_wisps());
    cast(&mut g, wisps, Some(Target::Permanent(angel)));
    assert_eq!(pt(&g, angel).0, 5);
    assert!(g.computed_permanent(angel).unwrap().colors.contains(Color::Green));
}

/// Mindshrieker grows by the milled card's mana value; Shriekgeist mills two.
#[test]
fn cloudhoof_pair() {
    let mut g = pod(2);
    let bird = ready(&mut g, 0, catalog::mindshrieker());
    g.add_card_to_library(1, catalog::serra_angel());
    activate(&mut g, bird, 0, Some(Target::Player(1)));
    assert_eq!(pt(&g, bird), (6, 6));
    let geist = ready(&mut g, 0, catalog::shriekgeist());
    for _ in 0..2 {
        g.add_card_to_library(1, catalog::island());
    }
    declare(&mut g, vec![(geist, 1)]);
    finish_combat(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 3);
}

/// Rob the Hoard returns a legendary card; White Dragon freezes a creature.
#[test]
fn icingdeath_pair() {
    let mut g = pod(2);
    let legend = g.add_card_to_graveyard(0, catalog::suki_kyoshi_captain());
    let dragon = g.add_card_to_hand(0, catalog::crystal_dragon());
    act(&mut g, GameAction::CastAdventure {
        card_id: dragon,
        target: Some(Target::Permanent(legend)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(g.players[0].hand.iter().any(|c| c.id == legend));
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let white = g.add_card_to_hand(0, catalog::white_dragon());
    cast(&mut g, white, None);
    assert!(g.battlefield_find(bear).unwrap().tapped);
}

/// Foul Renewal shrinks by the returned card's toughness; kicked Blood
/// Beckoning returns two.
#[test]
fn tetzimoc_pair() {
    let mut g = pod(2);
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    let victim = ready(&mut g, 1, catalog::hill_giant());
    let renewal = g.add_card_to_hand(0, catalog::foul_renewal());
    cast2(&mut g, renewal, Target::Permanent(angel), Target::Permanent(victim));
    assert!(g.players[0].hand.iter().any(|c| c.id == angel));
    assert!(g.battlefield_find(victim).is_none(), "-4/-4 kills a 3/3");
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::hill_giant());
    let beckon = g.add_card_to_hand(0, catalog::blood_beckoning());
    act(&mut g, GameAction::CastSpellKicked {
        card_id: beckon,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    });
    assert!(g.players[0].hand.iter().any(|c| c.id == a) && g.players[0].hand.iter().any(|c| c.id == b));
}

/// Match the Odds counts the opponents' creatures; The Art of Tea makes Food;
/// Kyoshi Warrior Exemplars needs eight lands.
#[test]
fn lion_turtle_and_haru_cards() {
    let mut g = pod(3);
    ready(&mut g, 1, catalog::grizzly_bears());
    ready(&mut g, 2, catalog::grizzly_bears());
    let odds = g.add_card_to_hand(0, catalog::match_the_odds());
    cast(&mut g, odds, None);
    let ally = g.battlefield.iter().find(|c| c.definition.name == "Ally").expect("Ally").id;
    assert_eq!(pt(&g, ally), (3, 3));
    let tea = g.add_card_to_hand(0, catalog::the_art_of_tea());
    cast(&mut g, tea, Some(Target::Permanent(ally)));
    assert_eq!(pt(&g, ally), (4, 4));
    assert_eq!(named(&g, 0, "Food"), 1);
    let kyoshi = ready(&mut g, 0, catalog::kyoshi_warrior_exemplars());
    for _ in 0..8 {
        ready(&mut g, 0, catalog::forest());
    }
    declare(&mut g, vec![(kyoshi, 1)]);
    assert_eq!(pt(&g, kyoshi), (6, 5));
}

/// Strider's landfall gives first strike at 4 power; Gimli pings on an
/// opposing death.
#[test]
fn strider_pair() {
    let mut g = pod(2);
    let strider = ready(&mut g, 0, catalog::strider_ranger_of_the_north());
    let forest = g.add_card_to_hand(0, catalog::forest());
    act(&mut g, GameAction::PlayLand(forest));
    assert!(has(&g, strider, Keyword::FirstStrike));
    ready(&mut g, 0, catalog::gimli_counter_of_kills());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    kill(&mut g, bear);
    assert_eq!(g.players[1].life, 19);
}

/// Basal Thrull sacrifices for {B}{B}; Viper pumps a lone attacker.
#[test]
fn purraj_pair() {
    let mut g = pod(2);
    let thrull = ready(&mut g, 0, catalog::basal_thrull());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: thrull,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("mana");
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 2);
    let viper = ready(&mut g, 0, catalog::viper_cruel_conspirator());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    declare(&mut g, vec![(bear, 1)]);
    activate(&mut g, viper, 0, Some(Target::Permanent(bear)));
    assert_eq!(pt(&g, bear), (3, 3));
}

/// Nobilis of War pumps attackers; Rise of the Hobgoblins makes X Goblins.
#[test]
fn agrus_kos_pair() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::nobilis_of_war());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    declare(&mut g, vec![(bear, 1)]);
    assert_eq!(pt(&g, bear), (4, 2));
    let mut g = pod(2);
    let rise = g.add_card_to_hand(0, catalog::rise_of_the_hobgoblins());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(3)]));
    cast(&mut g, rise, None);
    assert_eq!(named(&g, 0, "Goblin Soldier"), 3);
}

/// Blood Mist grants double strike at combat; Rush of Adrenaline pumps.
#[test]
fn ty_lee_pair() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::blood_mist());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    fire(&mut g, TurnStep::BeginCombat);
    assert!(has(&g, bear, Keyword::DoubleStrike));
    let rush = g.add_card_to_hand(0, catalog::rush_of_adrenaline());
    cast(&mut g, rush, Some(Target::Permanent(bear)));
    assert_eq!(pt(&g, bear), (4, 3));
}

/// Gray Harbor Merfolk grows with a commander out; Streambed Aquitects turns a
/// land into an Island.
#[test]
fn attuma_pair() {
    let mut g = pod(2);
    let fish = ready(&mut g, 0, catalog::gray_harbor_merfolk());
    assert_eq!(pt(&g, fish), (0, 3));
    let cmdr = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].commanders.push(cmdr);
    assert_eq!(pt(&g, fish), (2, 3));
    let scout = ready(&mut g, 0, catalog::streambed_aquitects());
    activate(&mut g, scout, 0, Some(Target::Permanent(fish)));
    assert!(has(&g, fish, Keyword::Landwalk(crabomination::card::LandType::Island)));
}

/// Bushmaster lends deathtouch to countered creatures; Tenderize bites.
#[test]
fn leatherhead_deck_pair() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::bushmaster_coiled_henchman());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    assert!(!has(&g, bear, Keyword::Deathtouch));
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert!(has(&g, bear, Keyword::Deathtouch));
    let giant = ready(&mut g, 1, catalog::colossal_dreadmaw());
    let bite = g.add_card_to_hand(0, catalog::tenderize());
    cast2(&mut g, bite, Target::Permanent(bear), Target::Permanent(giant));
    assert!(g.battlefield_find(giant).is_none(), "deathtouch bite");
}

/// Tempting Witch's Food drains 3; Giant's Skewer makes Food from creature
/// combat damage.
#[test]
fn old_flitterfang_pair() {
    let mut g = pod(2);
    let witch = g.add_card_to_hand(0, catalog::tempting_witch());
    cast(&mut g, witch, None);
    g.clear_sickness(witch);
    activate(&mut g, witch, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 17);
    let skewer = ready(&mut g, 0, catalog::giants_skewer());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    act(&mut g, GameAction::Equip { equipment: skewer, target: bear });
    assert_eq!(pt(&g, bear), (4, 3));
}

/// Fire Nation Archers hit each opponent; Loyal Fire Sage makes an Ally.
#[test]
fn zuko_pair() {
    let mut g = pod(3);
    let archers = ready(&mut g, 0, catalog::fire_nation_archers());
    activate(&mut g, archers, 0, None);
    assert_eq!((g.players[1].life, g.players[2].life), (18, 18));
    assert_eq!(named(&g, 0, "Soldier"), 1);
    let sage = ready(&mut g, 0, catalog::loyal_fire_sage());
    activate(&mut g, sage, 0, None);
    assert_eq!(named(&g, 0, "Ally"), 1);
}

/// General Ross's battalion and Horn of the Mark's two-attacker dig.
#[test]
fn paladin_pair() {
    let mut g = pod(2);
    let ross = ready(&mut g, 0, catalog::general_thunderbolt_ross());
    ready(&mut g, 0, catalog::horn_of_the_mark());
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let angel = g.add_card_to_library(0, catalog::serra_angel());
    declare(&mut g, vec![(ross, 1), (a, 1), (b, 1)]);
    assert_eq!(pt(&g, a).0, 3, "battalion");
    assert!(g.players[0].hand.iter().any(|c| c.id == angel), "Horn dug the Angel");
}

