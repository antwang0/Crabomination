//! Most-built commanders, second batch (`decks::cmdr_legends2`,
//! COMMANDER_BACKLOG §1 regenerated 2026-09-27).

use crabomination::card::{CardId, Keyword, KeywordSlice};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::grizzly_bears());
        }
    }
    g
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

fn attack(g: &mut GameState, attackers: &[CardId]) {
    for &a in attackers {
        g.clear_sickness(a);
    }
    advance_to(g, TurnStep::DeclareAttackers);
    let attacks = attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(1) }).collect();
    g.perform_action(GameAction::DeclareAttackers(attacks)).expect("attack");
    drain_stack(g);
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

/// CR 202.3f — Reaper King's {2/W}…{2/G} is mana value 10; another Scarecrow
/// entering under its controller destroys target permanent, and gets +1/+1.
#[test]
fn reaper_king_destroys_on_another_scarecrow() {
    let mut g = pod(3);
    assert_eq!(catalog::reaper_king().cost.cmc(), 10);
    g.add_card_to_battlefield(0, catalog::reaper_king());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let guide = g.add_card_to_hand(0, catalog::scarecrow_guide());
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, 0, guide, None).expect("cast the Scarecrow");
    assert!(g.battlefield_find(giant).is_none(), "the Giant was destroyed");
    let base = catalog::scarecrow_guide();
    assert_eq!(pt(&g, guide), (base.power + 1, base.toughness + 1));
}

/// CR 510.2 — Shroofus (a Saproling) connecting for 1 makes one Saproling.
#[test]
fn shroofus_saprolings_make_saprolings_on_combat_damage() {
    let mut g = pod(3);
    let shroofus = g.add_card_to_battlefield(0, catalog::shroofus_sproutsire());
    let life = g.players[1].life;
    attack(&mut g, &[shroofus]);
    advance_to(&mut g, TurnStep::PostCombatMain);
    let saprolings = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Saproling").count();
    assert_eq!(saprolings, 1);
    assert_eq!(g.players[1].life, life - 1);
}

/// Kratos gives every creature haste (CR 702.10b), and at a player's end step
/// deals that player damage per creature of theirs that didn't attack.
#[test]
fn kratos_punishes_creatures_that_stayed_home() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::kratos_god_of_war());
    let theirs = g.add_card_to_battlefield(1, catalog::hill_giant());
    assert!(g.computed_permanent(theirs).unwrap().keywords().has_kw(&Keyword::Haste), "anyone's creature");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    attack(&mut g, &[bear]);
    let life = g.players[0].life;
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2, "Kratos and the second Bear stayed home");
}

/// Myrel: attacking makes a Soldier per Soldier you control (itself
/// included), and during your turn an opponent can't cast a spell.
#[test]
fn myrel_makes_soldiers_and_silences_opponents() {
    let mut g = pod(3);
    let myrel = g.add_card_to_battlefield(0, catalog::myrel_shield_of_argive());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    assert!(cast(&mut g, 1, bolt, Some(Target::Player(0))).is_err(), "not during Myrel's controller's turn");
    attack(&mut g, &[myrel]);
    let soldiers = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Soldier").count();
    assert_eq!(soldiers, 1);
}

/// Doran: a creature spell with toughness above power costs {1} less, and an
/// attacker gets +X/+X for the gap between its power and toughness.
#[test]
fn doran_discounts_walls_and_pumps_by_the_gap() {
    let mut g = pod(3);
    let doran = g.add_card_to_battlefield(0, catalog::doran_besieged_by_time());
    let turtle = g.add_card_to_hand(0, catalog::horned_turtle());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, 0, turtle, None).expect("{2}{U} for {1}{U}");
    attack(&mut g, &[doran]);
    assert_eq!(pt(&g, doran), (5, 10), "0/5 attacking: +5/+5");
}

/// Gargos: a creature of yours becoming the target of a spell makes Gargos
/// fight a creature you don't control.
#[test]
fn gargos_fights_when_your_creature_is_targeted_by_a_spell() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::gargos_vicious_watcher());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let growth = g.add_card_to_hand(0, catalog::giant_growth());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, 0, growth, Some(Target::Permanent(bear))).expect("Giant Growth");
    assert!(g.battlefield_find(giant).is_none(), "Gargos (8 power) fought the Giant");
}

/// CR 702.6c — Syr Gwyn's equip Knight {0}: a Knight is equipped for free, a
/// Bear still pays; an equipped attacker draws a card for 1 life.
#[test]
fn syr_gwyn_equips_knights_for_free_and_draws_on_equipped_attacks() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::syr_gwyn_hero_of_ashvale());
    let knight = g.add_card_to_battlefield(0, catalog::white_knight());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blade = g.add_card_to_battlefield(0, catalog::bonesplitter());
    assert!(g.perform_action(GameAction::Equip { equipment: blade, target: bear }).is_err(), "equip {{1}} unpaid");
    g.perform_action(GameAction::Equip { equipment: blade, target: knight }).expect("equip Knight {0}");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(blade).unwrap().attached_to, Some(knight));
    let (hand, life) = (g.players[0].hand.len(), g.players[0].life);
    attack(&mut g, &[knight]);
    assert_eq!((g.players[0].hand.len(), g.players[0].life), (hand + 1, life - 1));
}

/// CR 118.9 — Jodah, Archmage Eternal lets a spell be cast for WUBRG.
#[test]
fn jodah_archmage_casts_anything_for_wubrg() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::jodah_archmage_eternal());
    let wurm = g.add_card_to_hand(0, catalog::craw_wurm());
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 1);
    }
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: wurm, pitch_card: None, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast for WUBRG");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wurm).is_some());
}

/// Raggadragga: a mana creature gets +2/+2 and untaps when it attacks; a
/// spell cast with seven or more mana gives a creature +7/+7 and trample.
#[test]
fn raggadragga_pumps_mana_creatures_and_rewards_big_spells() {
    let mut g = pod(3);
    let boss = g.add_card_to_battlefield(0, catalog::raggadragga_goreguts_boss());
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    assert_eq!(pt(&g, elf), (3, 3));
    attack(&mut g, &[elf]);
    assert!(!g.battlefield_find(elf).unwrap().tapped, "untapped as it attacked");
    advance_to(&mut g, TurnStep::PostCombatMain);
    let crusher = g.add_card_to_hand(0, catalog::ulamogs_crusher());
    g.players[0].mana_pool.add_colorless(8);
    cast(&mut g, 0, crusher, None).expect("eight mana into the Crusher");
    let pumped = [boss, elf, crusher].iter().any(|&c| {
        let cp = g.computed_permanent(c).unwrap();
        cp.keywords().has_kw(&Keyword::Trample) && cp.power >= 10
    });
    assert!(pumped, "+7/+7 and trample on a creature");
}

/// Alexios moves to each player at their upkeep, untapped, with a +1/+1
/// counter and haste.
#[test]
fn alexios_changes_hands_every_upkeep() {
    let mut g = pod(3);
    let alexios = g.add_card_to_battlefield(0, catalog::alexios_deimos_of_kosmos());
    advance_to(&mut g, TurnStep::End);
    advance_to(&mut g, TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.active_player_idx, 1);
    let a = g.battlefield_find(alexios).unwrap();
    assert_eq!(a.controller, 1);
    assert_eq!(a.counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1);
    assert!(g.computed_permanent(alexios).unwrap().keywords().has_kw(&Keyword::Haste));
}

/// Narset gives your creatures prowess; attacking exiles a cheap noncreature
/// card from a graveyard to cast a copy.
#[test]
fn narset_recasts_a_graveyard_spell_on_attack() {
    let mut g = pod(3);
    let narset = g.add_card_to_battlefield(0, catalog::narset_enlightened_exile());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().keywords().has_kw(&Keyword::Prowess));
    let bolt = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    attack(&mut g, &[narset]);
    assert!(g.exile.iter().any(|c| c.id == bolt), "the Bolt left the graveyard for exile");
}

/// CR 605.1a — a permanent's printed mana ability counts on the battlefield
/// (Raggadragga's pump, Midnight Arsonist's "without mana abilities").
#[test]
fn cr_605_1a_battlefield_permanents_read_their_mana_abilities() {
    use crabomination::card::SelectionRequirement as R;
    let mut g = pod(3);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let thopter = g.add_card_to_battlefield(1, catalog::ornithopter());
    let without = R::Artifact.and(R::HasManaAbility.negate());
    assert!(!g.evaluate_requirement_static(&without, &Target::Permanent(ring), 0, None), "Sol Ring has one");
    assert!(g.evaluate_requirement_static(&without, &Target::Permanent(thopter), 0, None));
}

/// Thalia and The Gitrog Monster: an opponent's creature enters tapped
/// (CR 614.12), and attacking sacrifices a creature or land, then draws.
#[test]
fn thalia_and_gitrog_taps_their_entrants_and_trades_a_land_for_a_card() {
    let mut g = pod(3);
    let tg = g.add_card_to_battlefield(0, catalog::thalia_and_the_gitrog_monster());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 2);
    g.active_player_idx = 1;
    cast(&mut g, 1, bear, None).expect("their Bear");
    assert!(g.battlefield_find(bear).unwrap().tapped, "entered tapped");
    g.active_player_idx = 0;
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let hand = g.players[0].hand.len();
    attack(&mut g, &[tg]);
    assert!(g.battlefield_find(forest).is_none(), "the Forest was sacrificed");
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Rocco, Cabaretti Caterer cast for X = 2 puts a creature with mana value
/// 2 or less onto the battlefield from the library.
#[test]
fn rocco_fetches_a_creature_up_to_x() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(3);
    let rocco = g.add_card_to_hand(0, catalog::rocco_cabaretti_caterer());
    let angel = g.add_card_to_library(0, catalog::serra_angel());
    let elf = g.add_card_to_library(0, catalog::llanowar_elves());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Search(Some(elf))]));
    for c in [Color::Red, Color::Green, Color::White] {
        g.players[0].mana_pool.add(c, 1);
    }
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell { card_id: rocco, target: None, additional_targets: vec![], mode: None, x_value: Some(2) })
        .expect("X = 2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(elf).is_some(), "the Elves came in");
    assert!(g.battlefield_find(angel).is_none(), "the Angel is too big");
}

/// Anti-Venom: cast, he returns a creature card; damage to him becomes
/// +1/+1 counters.
#[test]
fn anti_venom_reanimates_and_grows_from_damage() {
    let mut g = pod(3);
    let dead = g.add_card_to_graveyard(0, catalog::serra_angel());
    let av = g.add_card_to_hand(0, catalog::anti_venom_horrifying_healer());
    g.players[0].mana_pool.add(Color::White, 5);
    cast(&mut g, 0, av, None).expect("cast Anti-Venom");
    assert!(g.battlefield_find(dead).is_some(), "the Angel returned");
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    cast(&mut g, 1, bolt, Some(Target::Permanent(av))).expect("Bolt him");
    assert_eq!(pt(&g, av), (8, 8));
}

/// Sonic: attacking puts a counter on each creature of yours with haste
/// (Sonic among them).
#[test]
fn sonic_counters_hasty_creatures_on_attack() {
    let mut g = pod(3);
    let sonic = g.add_card_to_battlefield(0, catalog::sonic_the_hedgehog());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    attack(&mut g, &[sonic]);
    assert_eq!(pt(&g, sonic), (3, 5));
    assert_eq!(pt(&g, bear), (2, 2), "no flash or haste");
}

/// Iron Man: attacking makes a Treasure; sacrificing a noncreature artifact
/// (that Treasure, mana value 0) fetches a mana value 1 artifact, tapped.
#[test]
fn iron_man_climbs_the_artifact_ladder() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(3);
    let im = g.add_card_to_battlefield(0, catalog::iron_man_titan_of_innovation());
    let sol = g.add_card_to_library(0, catalog::sol_ring());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Search(Some(sol))]));
    attack(&mut g, &[im]);
    let ring = g.battlefield_find(sol).expect("Sol Ring (mana value 1) came in");
    assert!(ring.tapped);
}

/// CR 702.124i — Kratos and Atreus share "Partner—Father & son". Attacking
/// with a God gets Kratos's controller an experience counter; the end step
/// spends it as +1/+1 counters.
#[test]
fn kratos_stoic_father_banks_experience_and_atreus_cashes_it() {
    assert!(crabomination::format::commanders_may_pair(&catalog::kratos_stoic_father(), &catalog::atreus_impulsive_son()));
    let mut g = pod(3);
    let kratos = g.add_card_to_battlefield(0, catalog::kratos_stoic_father());
    attack(&mut g, &[kratos]);
    assert_eq!(g.players[0].experience, 1);
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(kratos).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1);

    let mut g = pod(3);
    let atreus = g.add_card_to_battlefield(0, catalog::atreus_impulsive_son());
    g.clear_sickness(atreus);
    g.players[0].experience = 2;
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(3);
    let (hand, life) = (g.players[0].hand.len(), g.players[1].life);
    g.perform_action(GameAction::ActivateAbility {
        card_id: atreus, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("activate Atreus");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew two, discarded one");
    assert_eq!(g.players[1].life, life - 2);
}

/// CR 506.2 — Jin Sakai counts attackers per player: two creatures attacking
/// two different opponents each get the choice (the headless pick is double
/// strike); two attacking the same opponent get nothing.
#[test]
fn jin_sakai_rewards_a_creature_alone_at_its_player() {
    let setup = |targets: [usize; 2]| {
        let mut g = pod(4);
        g.add_card_to_battlefield(0, catalog::jin_sakai_ghost_of_tsushima());
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        for id in [a, b] {
            g.clear_sickness(id);
        }
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![
            Attack { attacker: a, target: AttackTarget::Player(targets[0]) },
            Attack { attacker: b, target: AttackTarget::Player(targets[1]) },
        ]))
        .expect("attack");
        drain_stack(&mut g);
        let ds = |id| g.computed_permanent(id).unwrap().keywords().has_kw(&Keyword::DoubleStrike);
        (ds(a), ds(b))
    };
    assert_eq!(setup([1, 2]), (true, true), "each alone at its player");
    assert_eq!(setup([1, 1]), (false, false), "sharing a player");
}

fn activate_loyalty(g: &mut GameState, id: CardId, index: usize, target: Option<Target>, x: Option<u32>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: id, ability_index: index, target, x_value: x,
    })
    .expect("loyalty ability");
    drain_stack(g);
}

/// CR 903.8 — Jeska enters with a loyalty counter per command-zone cast of
/// her controller's commanders; her 0 triples a creature's combat damage to
/// an opponent (CR 614.1a) until her controller's next turn.
#[test]
fn jeska_loyalty_counts_commander_casts_and_triples_combat_damage() {
    let mut g = pod(3);
    let cmd = g.seat_commanders(0, vec![catalog::grizzly_bears()])[0];
    g.commander_cast_count.insert(cmd, 2);
    let jeska = g.add_card_to_hand(0, catalog::jeska_thrice_reborn());
    g.players[0].mana_pool.add(Color::Red, 3);
    cast(&mut g, 0, jeska, None).expect("cast Jeska");
    assert_eq!(g.battlefield_find(jeska).unwrap().counter_count(crabomination::card::CounterType::Loyalty), 2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate_loyalty(&mut g, jeska, 0, Some(Target::Permanent(bear)), None);
    let life = g.players[1].life;
    attack(&mut g, &[bear]);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, life - 6, "2 power, tripled");
}

/// Jeska's −X: X damage to the target, X loyalty paid. (Slots 2 and 3 are
/// the engine's auto-fill — INCOMPLETE_CARDS.)
#[test]
fn jeska_minus_x_pays_x_and_deals_x() {
    let mut g = pod(3);
    let jeska = g.add_card_to_battlefield(0, catalog::jeska_thrice_reborn());
    g.battlefield_find_mut(jeska).unwrap().add_counters(crabomination::card::CounterType::Loyalty, 3);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate_loyalty(&mut g, jeska, 1, Some(Target::Permanent(a)), Some(2));
    assert!(g.battlefield_find(a).is_none(), "2 damage kills the Bears");
    assert!(g.battlefield_find(mine).is_some_and(|c| c.damage == 0), "never her controller's");
    assert_eq!(g.battlefield_find(jeska).unwrap().counter_count(crabomination::card::CounterType::Loyalty), 1);
}

/// Tevesh Szat's +1 draws a third card when the sacrificed permanent was a
/// commander (any player's, CR 903.3); the −10 takes every commander, on the
/// battlefield and in each command zone.
#[test]
fn tevesh_szat_rewards_a_commander_sacrifice_and_steals_every_commander() {
    let mut g = pod(3);
    let mine = g.seat_commanders(0, vec![catalog::grizzly_bears()])[0];
    let pos = g.players[0].command.iter().position(|c| c.id == mine).unwrap();
    let card = g.players[0].command.remove(pos);
    g.battlefield.push(card);
    let szat = g.add_card_to_battlefield(0, catalog::tevesh_szat_doom_of_fools());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    activate_loyalty(&mut g, szat, 1, None, None);
    assert_eq!(g.players[0].hand.len(), hand + 3, "two, and one for the commander");

    let mut g = pod(3);
    let theirs = g.seat_commanders(1, vec![catalog::serra_angel()])[0];
    let other = g.seat_commanders(2, vec![catalog::craw_wurm()])[0];
    let pos = g.players[2].command.iter().position(|c| c.id == other).unwrap();
    let card = g.players[2].command.remove(pos);
    g.battlefield.push(card);
    g.battlefield_find_mut(other).unwrap().controller = 2;
    let szat = g.add_card_to_battlefield(0, catalog::tevesh_szat_doom_of_fools());
    g.battlefield_find_mut(szat).unwrap().add_counters(crabomination::card::CounterType::Loyalty, 6);
    activate_loyalty(&mut g, szat, 2, None, None);
    for id in [theirs, other] {
        assert_eq!(g.battlefield_find(id).map(|c| c.controller), Some(0), "{id:?} is ours");
    }
}
