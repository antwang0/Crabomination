//! Commander: the two-card EDHREC gaps of commanders ranked 1001-1500
//! (`decks::cmdr_edhrec15`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
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

fn cast_x(g: &mut GameState, seat: usize, id: CardId, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_as(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_x(g, seat, id, targets, None)
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_as(g, 0, id, targets)
}

fn activate_x(g: &mut GameState, id: CardId, index: usize, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: x,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    activate_x(g, id, index, target, None)
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::grizzly_bears());
    }
}

fn perm(id: CardId) -> Target {
    Target::Permanent(id)
}

fn tokens_named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.is_token && c.definition.name == name).count()
}

fn declare(g: &mut GameState, attacks: &[(CardId, usize)]) -> Result<(), String> {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.iter().map(|&(a, d)| Attack { attacker: a, target: AttackTarget::Player(d) }).collect(),
    ))
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn combat_damage(g: &mut GameState) {
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

/// Battletide Alchemist (CR 615): damage to its controller is shaved by the
/// number of Clerics they control; damage to an opponent is not.
#[test]
fn battletide_alchemist_prevents_one_per_cleric_on_your_side() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::battletide_alchemist());
    ready(&mut g, 0, catalog::adept_watershaper());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, bolt, &[Target::Player(0)]).expect("bolt you");
    assert_eq!(g.players[0].life, 19, "two Clerics prevent 2 of 3");
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, bolt, &[Target::Player(2)]).expect("bolt the other opponent");
    assert_eq!(g.players[2].life, 17, "an opponent's damage isn't prevented");
}

/// King T'Challa (CR 121.2): an opponent's second draw of the turn draws you a
/// card; the {4}{W}{U} activation transforms it into Black Panther, whose
/// damage is prevented.
#[test]
fn king_tchalla_draws_on_a_second_draw_and_transforms() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    library(&mut g, 1, 3);
    let king = ready(&mut g, 0, catalog::king_tchalla());
    let div = g.add_card_to_hand(1, catalog::quick_study());
    let hand = g.players[0].hand.len();
    cast_as(&mut g, 1, div, &[]).expect("quick study");
    assert_eq!(g.players[0].hand.len(), hand + 1, "their second card drew one of ours");
    activate(&mut g, king, 0, None).expect("transform");
    let c = g.computed_permanent(king).unwrap();
    assert_eq!(c.def.name, "Black Panther, Hope Enduring");
    assert!(c.keywords().contains(&Keyword::DoubleStrike));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, bolt, &[perm(king)]).expect("bolt");
    assert!(g.battlefield_find(king).is_some(), "damage to it is prevented");
}

/// Black Cat, Cunning Thief: two of the opponent's top nine go face down into
/// exile, castable by Black Cat's controller with mana of any type; the other
/// seven go to the bottom.
#[test]
fn black_cat_exiles_two_of_nine_to_play() {
    let mut g = pod(2);
    g.add_card_to_library(1, catalog::grizzly_bears());
    for _ in 0..12 {
        g.add_card_to_library(1, catalog::island());
    }
    let cat = g.add_card_to_hand(0, catalog::black_cat_cunning_thief());
    cast(&mut g, cat, &[]).expect("black cat");
    let taken: Vec<_> = g.exile.iter().filter(|c| c.exiled_with == Some(cat)).collect();
    assert_eq!(taken.len(), 2, "two exiled");
    assert!(taken.iter().all(|c| c.face_down));
    assert!(taken.iter().any(|c| c.definition.name == "Grizzly Bears"), "the nonland is the bot's pick");
    assert_eq!(g.players[1].library.len(), 11);
}

/// Mirror-Mad Phantasm: the owner shuffles it in, reveals to it, puts it back
/// onto the battlefield and every card revealed before it into the graveyard.
#[test]
fn mirror_mad_phantasm_mills_until_it_comes_back() {
    let mut g = pod(2);
    library(&mut g, 0, 5);
    let ph = ready(&mut g, 0, catalog::mirror_mad_phantasm());
    activate(&mut g, ph, 0, None).expect("shuffle in");
    assert!(g.battlefield_find(ph).is_some(), "it comes back");
    let gy = g.players[0].graveyard.len();
    assert_eq!(gy + g.players[0].library.len(), 5, "the misses are binned, the rest stay");
}

/// Iymrith, Desert Doom: ward {4} only while untapped; a hit draws one, then
/// fills the hand up to three.
#[test]
fn iymrith_draws_up_to_three_and_loses_ward_when_tapped() {
    let mut g = pod(2);
    library(&mut g, 0, 6);
    let iy = ready(&mut g, 0, catalog::iymrith_desert_doom());
    let ward = |g: &GameState| g.computed_permanent(iy).unwrap().keywords().iter().any(|k| matches!(k, Keyword::Ward(_)));
    assert!(ward(&g));
    g.players[0].hand.clear();
    declare(&mut g, &[(iy, 1)]).expect("attack");
    assert!(!ward(&g), "tapped by attacking");
    combat_damage(&mut g);
    assert_eq!(g.players[0].hand.len(), 3, "one, then two more to reach three");
}

/// Barrin, Tolarian Archmage: its ETB bounces an opponent's creature (their
/// hand, so no draw); a permanent put into *your* hand from the battlefield
/// this turn draws a card at your end step.
#[test]
fn barrin_draws_after_a_bounce_into_your_hand() {
    let end_step = |g: &mut GameState| {
        g.step = TurnStep::End;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(g);
    };
    let mut g = pod(2);
    library(&mut g, 0, 3);
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let barrin = g.add_card_to_hand(0, catalog::barrin_tolarian_archmage());
    cast(&mut g, barrin, &[]).expect("barrin");
    assert!(g.players[1].hand.iter().any(|c| c.id == theirs), "bounced to its owner");
    let hand = g.players[0].hand.len();
    end_step(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "nothing came to your hand");
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    let unsummon = g.add_card_to_hand(0, catalog::unsummon());
    cast(&mut g, unsummon, &[perm(mine)]).expect("unsummon");
    let hand = g.players[0].hand.len();
    end_step(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Phial of Galadriel (CR 614): life gained at 5 or less is doubled, and a
/// draw with an empty hand draws two.
#[test]
fn phial_doubles_low_life_gain_and_empty_hand_draws() {
    let mut g = pod(2);
    library(&mut g, 0, 6);
    ready(&mut g, 0, catalog::phial_of_galadriel());
    g.players[0].life = 5;
    g.players[0].hand.clear();
    let rev = g.add_card_to_hand(0, catalog::revitalize());
    cast(&mut g, rev, &[]).expect("revitalize");
    assert_eq!(g.players[0].life, 11, "3 doubled at 5 life");
    assert_eq!(g.players[0].hand.len(), 2, "the draw with an empty hand drew two");
    let rev = g.add_card_to_hand(0, catalog::revitalize());
    cast(&mut g, rev, &[]).expect("revitalize again");
    assert_eq!(g.players[0].life, 14, "above 5, not doubled");
}

/// Draconic Visitor (CR 614.1a): an artifact token becomes a 5/5 Dragon.
#[test]
fn draconic_visitor_turns_treasure_into_dragons() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::draconic_visitor());
    let rich = g.add_card_to_hand(0, catalog::strike_it_rich());
    cast(&mut g, rich, &[]).expect("strike it rich");
    assert_eq!(tokens_named(&g, "Treasure"), 0);
    assert_eq!(tokens_named(&g, "Dragon"), 1);
}

/// Champion of the Flame and Goblin Gaveleer (CR 613.4c): per-attachment
/// growth — the Champion counts Auras and Equipment at +2/+2, the Gaveleer
/// Equipment only at +2/+0.
#[test]
fn per_attachment_growth() {
    for (def, rancor_counts, base, per) in [
        (catalog::champion_of_the_flame() as crabomination::card::CardDefinition, true, (1, 1), (2, 2)),
        (catalog::goblin_gaveleer(), false, (1, 1), (2, 0)),
    ] {
        let mut g = pod(2);
        let id = ready(&mut g, 0, def);
        let sword = ready(&mut g, 0, catalog::bonesplitter());
        flood(&mut g, 0);
        g.perform_action(GameAction::Equip { equipment: sword, target: id }).expect("equip");
        let rancor = g.add_card_to_hand(0, catalog::rancor());
        cast(&mut g, rancor, &[perm(id)]).expect("rancor");
        let n = 1 + i32::from(rancor_counts);
        // Bonesplitter +2/+0 and Rancor +2/+0 ride on top.
        assert_eq!(pt(&g, id), (base.0 + n * per.0 + 4, base.1 + n * per.1));
    }
}

/// Retribution of the Ancients: removing X +1/+1 counters from among your
/// creatures gives -X/-X.
#[test]
fn retribution_of_the_ancients_spends_counters_for_minus_x() {
    let mut g = pod(2);
    let ret = ready(&mut g, 0, catalog::retribution_of_the_ancients());
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(a).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    g.battlefield_find_mut(b).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let ogre = ready(&mut g, 1, catalog::gray_ogre());
    activate_x(&mut g, ret, 0, Some(perm(ogre)), Some(3)).expect("x = 3");
    assert!(g.battlefield_find(ogre).is_none(), "2/2 at -3/-3 dies");
    let left = |id| g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne));
    assert_eq!(left(a) + left(b), 0);
}

/// Doctor Doom, Unrivaled: the tap draw that empties the library wins.
#[test]
fn doctor_doom_wins_off_an_empty_library() {
    let mut g = pod(2);
    library(&mut g, 0, 1);
    let doom = ready(&mut g, 0, catalog::doctor_doom_unrivaled());
    activate(&mut g, doom, 0, None).expect("tap");
    assert_eq!(g.players[0].life, 19);
    assert!(g.players[1].eliminated || g.is_game_over(), "the last card won");
}

/// Koll, the Forgemaster (CR 603.10a): an equipped creature that dies returns
/// to hand; an unequipped one stays in the graveyard.
#[test]
fn koll_returns_a_creature_that_died_equipped() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::koll_the_forgemaster());
    let geared = ready(&mut g, 0, catalog::grizzly_bears());
    let bare = ready(&mut g, 0, catalog::grizzly_bears());
    let sword = ready(&mut g, 0, catalog::bonesplitter());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: sword, target: geared }).expect("equip");
    for id in [geared, bare] {
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        cast_as(&mut g, 1, bolt, &[perm(id)]).expect("bolt");
    }
    assert!(g.players[0].hand.iter().any(|c| c.id == geared));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bare));
}

/// Henrika Domnathi: each beginning of combat picks a mode not chosen before —
/// the third transforms her.
#[test]
fn henrika_rotates_through_her_modes() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    let h = ready(&mut g, 0, catalog::henrika_domnathi());
    ready(&mut g, 0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.step = TurnStep::BeginCombat;
        g.fire_step_triggers(TurnStep::BeginCombat);
        drain_stack(&mut g);
        if g.battlefield_find(h).is_none() {
            break;
        }
    }
    let c = g.computed_permanent(h).expect("the edict takes the Bears");
    assert_eq!(c.def.name, "Henrika, Infernal Seer");
    assert!(c.keywords().contains(&Keyword::Lifelink));
}

/// Rolling Earthquake: X to each creature without horsemanship and each player.
#[test]
fn rolling_earthquake_hits_players_and_creatures() {
    let mut g = pod(3);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let quake = g.add_card_to_hand(0, catalog::rolling_earthquake());
    cast_x(&mut g, 0, quake, &[], Some(2)).expect("x = 2");
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.players.iter().all(|p| p.life == 18));
}

/// Vessel of Volatility: {1}{R}, sacrifice: {R}{R}{R}{R}.
#[test]
fn vessel_of_volatility_makes_four_red() {
    let mut g = pod(2);
    let v = ready(&mut g, 0, catalog::vessel_of_volatility());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: v,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("crack it");
    drain_stack(&mut g);
    assert!(g.battlefield_find(v).is_none());
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 4);
}


/// Locket of Yesterdays (CR 601.2f): a spell costs {1} less per same-named
/// card in your graveyard — a second Divination costs {1}{U}.
#[test]
fn locket_discounts_a_spell_by_its_namesakes() {
    let mut g = pod(2);
    library(&mut g, 0, 4);
    ready(&mut g, 0, catalog::locket_of_yesterdays());
    g.add_card_to_graveyard(0, catalog::divination());
    let div = g.add_card_to_hand(0, catalog::divination());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: div, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{1}{U} pays for it");
}

/// Hawkeye, Young Avenger (CR 614.1a): your noncombat damage to an opponent
/// gets +X, X = Hawkeye's power; your own face takes no bonus.
#[test]
fn hawkeye_adds_its_power_to_noncombat_damage() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::hawkeye_young_avenger());
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast(&mut g, shock, &[Target::Player(1)]).expect("shock them");
    assert_eq!(g.players[1].life, 16, "2 + Hawkeye's 2");
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast(&mut g, shock, &[Target::Player(0)]).expect("shock yourself");
    assert_eq!(g.players[0].life, 18);
}

/// Gelatinous Cube: Engulf exiles an opponent's non-Ooze creature; Dissolve
/// for X = its mana value sends it to its owner's graveyard.
#[test]
fn gelatinous_cube_engulfs_then_dissolves() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let cube = g.add_card_to_hand(0, catalog::gelatinous_cube());
    cast(&mut g, cube, &[]).expect("cube");
    assert!(g.exile.iter().any(|c| c.id == bear));
    activate_x(&mut g, cube, 0, Some(perm(bear)), Some(2)).expect("dissolve x = 2");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
}

/// Harbin, Vanguard Aviator: five attacking Soldiers pump the team and give
/// it flying; four don't.
#[test]
fn harbin_needs_five_soldiers() {
    for (soldiers, pumped) in [(3, false), (4, true)] {
        let mut g = pod(2);
        let harbin = ready(&mut g, 0, catalog::harbin_vanguard_aviator());
        let mut attackers = vec![(harbin, 1)];
        for _ in 0..soldiers {
            attackers.push((ready(&mut g, 0, catalog::elite_vanguard()), 1));
        }
        let home = ready(&mut g, 0, catalog::grizzly_bears());
        declare(&mut g, &attackers).expect("attack");
        let (p, t) = pt(&g, home);
        assert_eq!((p, t) == (3, 3), pumped, "{} Soldiers attacking", soldiers + 1);
        let flies = g.computed_permanent(home).unwrap().keywords().contains(&Keyword::Flying);
        assert_eq!(flies, pumped);
    }
}

/// Lo and Li, Royal Advisors: an opponent milling grows every Advisor you
/// control.
#[test]
fn lo_and_li_grow_advisors_when_an_opponent_mills() {
    let mut g = pod(2);
    library(&mut g, 1, 8);
    let lo = ready(&mut g, 0, catalog::lo_and_li_royal_advisors());
    let mc = ready(&mut g, 0, catalog::masters_councillors());
    activate(&mut g, lo, 0, Some(Target::Player(1))).expect("mill four");
    assert_eq!(g.players[1].graveyard.len(), 4);
    for id in [lo, mc] {
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }
}

/// Master's Councillors: +2/+0 for each graveyard with seven or more cards.
#[test]
fn masters_councillors_counts_full_graveyards() {
    let mut g = pod(3);
    let mc = ready(&mut g, 0, catalog::masters_councillors());
    for seat in [1, 2] {
        for _ in 0..7 {
            g.add_card_to_graveyard(seat, catalog::island());
        }
    }
    assert_eq!(pt(&g, mc), (5, 3));
}

/// Krav, the Unredeemed: sacrifice X creatures — the target draws X and gains
/// X, and Krav takes X counters.
#[test]
fn krav_sacrifices_x_for_cards_life_and_counters() {
    let mut g = pod(2);
    library(&mut g, 0, 4);
    let krav = ready(&mut g, 0, catalog::krav_the_unredeemed());
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    activate_x(&mut g, krav, 0, Some(Target::Player(0)), Some(2)).expect("x = 2");
    assert_eq!(g.players[0].hand.len(), hand + 2);
    assert_eq!(g.players[0].life, 22);
    assert_eq!(g.battlefield_find(krav).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Protect the Negotiators: the tax is {1} per creature you control — two
/// creatures, and an opponent with one spare mana is countered.
#[test]
fn protect_the_negotiators_taxes_per_creature() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    let ptn = g.add_card_to_hand(0, catalog::protect_the_negotiators());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: ptn,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("counter");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "one spare mana can't pay {{2}}");
}

/// Last Light of Durin's Day: the sixth Mountain sacrifices it and puts a
/// Dragon from hand onto the battlefield.
#[test]
fn last_light_fetches_a_dragon_at_six_quest_counters() {
    let mut g = pod(2);
    let ll = ready(&mut g, 0, catalog::last_light_of_durins_day());
    let shivan = g.add_card_to_hand(0, catalog::shivan_dragon());
    for _ in 0..6 {
        let m = g.add_card_to_hand(0, catalog::mountain());
        g.players[0].lands_played_this_turn = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::PlayLand(m)).expect("land drop");
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(ll).is_none(), "sacrificed");
    assert!(g.battlefield_find(shivan).is_some(), "the Dragon came from hand");
}

/// Themberchaud: ETB deals X — Mountains you control — to each other
/// non-flier and each player.
#[test]
fn themberchaud_quakes_by_mountains() {
    let mut g = pod(2);
    for _ in 0..3 {
        ready(&mut g, 0, catalog::mountain());
    }
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let serra = ready(&mut g, 1, catalog::serra_angel());
    let th = g.add_card_to_hand(0, catalog::themberchaud());
    cast(&mut g, th, &[]).expect("themberchaud");
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(serra).is_some(), "fliers are spared");
    assert!(g.battlefield_find(th).is_some(), "not itself");
    assert_eq!((g.players[0].life, g.players[1].life), (17, 17));
}

/// Rigo, Streetwise Mentor: enters with a shield counter; attacking with a
/// power-1 creature draws a card.
#[test]
fn rigo_draws_when_a_small_creature_attacks() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    let rigo = g.add_card_to_hand(0, catalog::rigo_streetwise_mentor());
    cast(&mut g, rigo, &[]).expect("rigo");
    assert_eq!(g.battlefield_find(rigo).unwrap().counter_count(CounterType::Shield), 1);
    let lions = ready(&mut g, 0, catalog::savannah_lions());
    let hand = g.players[0].hand.len();
    declare(&mut g, &[(lions, 1)]).expect("attack with a 2/1");
    assert_eq!(g.players[0].hand.len(), hand, "power 2 doesn't count");
    let mut g = pod(2);
    library(&mut g, 0, 2);
    ready(&mut g, 0, catalog::rigo_streetwise_mentor());
    let mite = ready(&mut g, 0, catalog::memnite());
    let hand = g.players[0].hand.len();
    declare(&mut g, &[(mite, 1)]).expect("attack with a 1/1");
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Lapis Orb of Dragonkind: its {U} spent on a Dragon creature spell puts a
/// scry 2 trigger above that spell.
#[test]
fn lapis_orb_scries_for_a_dragon() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    let orb = ready(&mut g, 0, catalog::lapis_orb_of_dragonkind());
    let iy = g.add_card_to_hand(0, catalog::iymrith_desert_doom());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: orb,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap for U");
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell { card_id: iy, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast the Dragon");
    assert_eq!(g.stack.len(), 2, "the scry trigger rides above the Dragon");
}

/// Isengard Unleashed (CR 614.1a): your sources deal triple damage to
/// opponents this turn, not to you.
#[test]
fn isengard_unleashed_triples_damage_to_opponents() {
    let mut g = pod(2);
    let ise = g.add_card_to_hand(0, catalog::isengard_unleashed());
    cast(&mut g, ise, &[]).expect("isengard");
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast(&mut g, shock, &[Target::Player(1)]).expect("shock them");
    assert_eq!(g.players[1].life, 14, "2 tripled");
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast(&mut g, shock, &[Target::Player(0)]).expect("shock yourself");
    assert_eq!(g.players[0].life, 18);
}

/// Chromium, the Mutable: discarding a card makes it a 1/1 Human with no
/// abilities but hexproof, unblockable this turn.
#[test]
fn chromium_hides_as_a_hexproof_human() {
    let mut g = pod(2);
    let ch = ready(&mut g, 0, catalog::chromium_the_mutable());
    g.add_card_to_hand(0, catalog::island());
    activate(&mut g, ch, 0, None).expect("discard");
    let c = g.computed_permanent(ch).unwrap();
    assert_eq!((c.power, c.toughness), (1, 1));
    assert!(c.keywords().contains(&Keyword::Hexproof));
    assert!(!c.keywords().contains(&Keyword::Flying), "lost its abilities");
}

/// Owlbear Cub (CR 508.4): attacking a player with eight lands puts a creature
/// from the top eight onto the battlefield tapped and attacking — no
/// indestructible, no bounce.
#[test]
fn owlbear_cub_calls_mama_against_eight_lands() {
    let mut g = pod(2);
    for _ in 0..8 {
        ready(&mut g, 1, catalog::forest());
    }
    g.add_card_to_library(0, catalog::island());
    let mama = g.add_card_to_library(0, catalog::serra_angel());
    let cub = ready(&mut g, 0, catalog::owlbear_cub());
    declare(&mut g, &[(cub, 1)]).expect("attack");
    assert!(g.attacking.iter().any(|a| a.attacker == mama), "joined the attack");
    assert!(!g.computed_permanent(mama).unwrap().keywords().contains(&Keyword::Indestructible));
}

/// Ulvenwald Oddity transforms into the Behemoth, which pumps the team.
#[test]
fn ulvenwald_behemoth_pumps_the_team() {
    let mut g = pod(2);
    let odd = ready(&mut g, 0, catalog::ulvenwald_oddity());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, odd, 0, None).expect("transform");
    assert_eq!(pt(&g, odd), (8, 8));
    assert_eq!(pt(&g, bear), (3, 3));
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Haste));
}

/// Shanna, Sisay's Legacy: +1/+1 per creature you control, herself included.
#[test]
fn shanna_counts_your_creatures() {
    let mut g = pod(2);
    let sh = ready(&mut g, 0, catalog::shanna_sisays_legacy());
    ready(&mut g, 0, catalog::grizzly_bears());
    assert_eq!(pt(&g, sh), (2, 2));
}

/// Brigid: entering makes a Kithkin; paying {G} at the first main phase
/// transforms her, and the back taps for X mana, X = your other creatures.
#[test]
fn brigid_flips_for_mana() {
    let mut g = pod(2);
    let br = g.add_card_to_hand(0, catalog::brigid_clachans_heart());
    cast(&mut g, br, &[]).expect("brigid");
    assert_eq!(tokens_named(&g, "Kithkin"), 1);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    flood(&mut g, 0);
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(br).unwrap().def.name, "Brigid, Doun's Mind");
    assert_eq!(tokens_named(&g, "Kithkin"), 1, "flipping away makes no Kithkin");
    g.clear_sickness(br);
    g.players[0].mana_pool.empty();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: br,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap for G");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1, "one other creature: the Kithkin");
}

/// Rona, Sheoldred's Faithful: each instant or sorcery drains every opponent.
#[test]
fn rona_drains_on_spells() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::rona_sheoldreds_faithful());
    let opt = g.add_card_to_hand(0, catalog::opt());
    library(&mut g, 0, 2);
    cast(&mut g, opt, &[]).expect("opt");
    assert_eq!((g.players[1].life, g.players[2].life), (19, 19));
}

/// The Grey Havens: {T} adds a color of a legendary creature card in your
/// graveyard — none, no mana.
#[test]
fn the_grey_havens_reads_legends_in_the_graveyard() {
    let mut g = pod(2);
    let gh = ready(&mut g, 0, catalog::the_grey_havens());
    let tap = |g: &mut GameState| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: gh,
            ability_index: 1,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    let _ = tap(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 0, "no legend in the graveyard");
    g.battlefield_find_mut(gh).unwrap().tapped = false;
    g.add_card_to_graveyard(0, catalog::isamaru_hound_of_konda());
    tap(&mut g).expect("tap for W");
    assert_eq!(g.players[0].mana_pool.amount(Color::White), 1);
}

/// Stigma Lasher (CR 119.7): a player it damages can't gain life for the rest
/// of the game.
#[test]
fn stigma_lasher_locks_life_gain() {
    let mut g = pod(2);
    let sl = ready(&mut g, 0, catalog::stigma_lasher());
    declare(&mut g, &[(sl, 1)]).expect("attack");
    combat_damage(&mut g);
    assert_eq!(g.players[1].life, 18);
    let rev = g.add_card_to_hand(1, catalog::revitalize());
    library(&mut g, 1, 1);
    cast_as(&mut g, 1, rev, &[]).expect("revitalize");
    assert_eq!(g.players[1].life, 18, "no life gained");
}

/// Phyrexian Purge: each target creature dies and costs its caster 3 life.
#[test]
fn phyrexian_purge_charges_three_life_a_target() {
    let mut g = pod(2);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::gray_ogre());
    let purge = g.add_card_to_hand(0, catalog::phyrexian_purge());
    cast(&mut g, purge, &[perm(a), perm(b)]).expect("purge two");
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    assert_eq!(g.players[0].life, 14);
}

/// Doors of Durin (CR 508.4): a creature on top enters tapped and attacking,
/// with trample while you control a Dwarf.
#[test]
fn doors_of_durin_sends_the_top_creature_in() {
    let mut g = pod(2);
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::serra_angel());
    ready(&mut g, 0, catalog::doors_of_durin());
    ready(&mut g, 0, catalog::dwarven_trader());
    let att = ready(&mut g, 0, catalog::gray_ogre());
    declare(&mut g, &[(att, 1)]).expect("attack");
    let joined: Vec<_> = g.attacking.iter().map(|a| a.attacker).filter(|&id| id != att).collect();
    assert_eq!(joined.len(), 1, "one creature joined");
    let c = g.computed_permanent(joined[0]).unwrap();
    assert!(c.keywords().contains(&Keyword::Trample), "a Dwarf grants trample");
    let _ = top;
}

/// Mighty Servant of Leuk-o: crewed by exactly two creatures, it gains "combat
/// damage to a player draws two" this turn.
#[test]
fn mighty_servant_draws_two_when_two_crew_it() {
    let mut g = pod(2);
    library(&mut g, 0, 4);
    let ms = ready(&mut g, 0, catalog::mighty_servant_of_leuk_o());
    let a = ready(&mut g, 0, catalog::gray_ogre());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Crew { vehicle: ms, crew_creatures: vec![a, b] }).expect("crew with two");
    drain_stack(&mut g);
    let hand = g.players[0].hand.len();
    declare(&mut g, &[(ms, 1)]).expect("attack");
    combat_damage(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

/// Demonlord Belzenlok: digs to a nonland card and takes it, going again while
/// the card costs 4 or more, 1 damage per card.
#[test]
fn belzenlok_digs_until_a_cheap_card() {
    let mut g = pod(2);
    // Top to bottom: Forest, Craterhoof (8), Island, Bears (2), Serra Angel.
    for def in [
        catalog::forest(),
        catalog::craterhoof_behemoth(),
        catalog::island(),
        catalog::grizzly_bears(),
        catalog::serra_angel(),
    ] {
        g.add_card_to_library(0, def);
    }
    let bz = g.add_card_to_hand(0, catalog::demonlord_belzenlok());
    let hand = g.players[0].hand.len();
    cast(&mut g, bz, &[]).expect("belzenlok");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "Craterhoof, then the Bears");
    assert_eq!(g.players[0].life, 18);
    assert_eq!(g.players[0].library.len(), 1, "Serra Angel is left");
}

/// The Immortal Sun (CR 606.3): no loyalty abilities; spells cost {1} less;
/// creatures you control get +1/+1.
#[test]
fn the_immortal_sun_locks_loyalty_and_discounts() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::the_immortal_sun());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    assert_eq!(pt(&g, bear), (3, 3));
    let lili = ready(&mut g, 0, catalog::liliana_of_the_veil());
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: lili, ability_index: 0, target: None, x_value: None })
            .is_err(),
        "loyalty is locked"
    );
    let div = g.add_card_to_hand(0, catalog::divination());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: div, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("Divination for {1}{U}");
}

/// Runadi, Behemoth Caller: an 8-drop enters with four more counters and,
/// with three or more, haste.
#[test]
fn runadi_grows_big_creature_spells() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    ready(&mut g, 0, catalog::runadi_behemoth_caller());
    let hoof = g.add_card_to_hand(0, catalog::craterhoof_behemoth());
    cast(&mut g, hoof, &[]).expect("craterhoof");
    let c = g.battlefield_find(hoof).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 4);
    assert!(g.computed_permanent(hoof).unwrap().keywords().contains(&Keyword::Haste));
}

/// Vraska, the Cutting Glare: with six lands, ETB destroys an opponent's
/// permanent and hands them a Treasure; with five, nothing.
#[test]
fn vraska_needs_six_lands() {
    for (lands, kills) in [(5, false), (6, true)] {
        let mut g = pod(2);
        for _ in 0..lands {
            ready(&mut g, 0, catalog::forest());
        }
        let ogre = ready(&mut g, 1, catalog::gray_ogre());
        let v = g.add_card_to_hand(0, catalog::vraska_the_cutting_glare());
        cast(&mut g, v, &[]).expect("vraska");
        assert_eq!(g.battlefield_find(ogre).is_none(), kills, "{lands} lands");
        let theirs = g.battlefield.iter().filter(|c| c.is_token && c.controller == 1 && c.definition.name == "Treasure").count();
        assert_eq!(theirs, usize::from(kills));
    }
}

/// Long Road Home: the creature comes back at the next end step with a
/// +1/+1 counter.
#[test]
fn long_road_home_returns_with_a_counter() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let lrh = g.add_card_to_hand(0, catalog::long_road_home());
    cast(&mut g, lrh, &[perm(bear)]).expect("long road home");
    assert!(g.battlefield_find(bear).is_none());
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let back = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").expect("back");
    assert_eq!(back.counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Brigid, Hero of Kinsbaile: 2 damage to each attacking or blocking creature
/// the target player controls — only those in combat.
#[test]
fn brigid_hero_shoots_the_attackers() {
    let mut g = pod(2);
    let br = ready(&mut g, 1, catalog::brigid_hero_of_kinsbaile());
    let att = ready(&mut g, 0, catalog::grizzly_bears());
    let home = ready(&mut g, 0, catalog::grizzly_bears());
    declare(&mut g, &[(att, 1)]).expect("attack");
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: br,
        ability_index: 0,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap brigid");
    drain_stack(&mut g);
    assert!(g.battlefield_find(att).is_none());
    assert!(g.battlefield_find(home).is_some());
}

/// Fabrication Module: getting energy puts a +1/+1 counter on a creature you
/// control; {4}, {T} is the energy.
#[test]
fn fabrication_module_counts_energy() {
    let mut g = pod(2);
    let fm = ready(&mut g, 0, catalog::fabrication_module());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, fm, 0, None).expect("energy");
    assert_eq!(g.players[0].energy, 1);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Primal Prayers (CR 118.9): {E} casts a creature spell of mana value 3 or
/// less in place of its mana cost.
#[test]
fn primal_prayers_casts_small_creatures_for_energy() {
    let mut g = pod(2);
    let pp = g.add_card_to_hand(0, catalog::primal_prayers());
    cast(&mut g, pp, &[]).expect("prayers");
    assert_eq!(g.players[0].energy, 2);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.empty();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: bear,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("pay {E}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
    assert_eq!(g.players[0].energy, 1);
}

/// Sedris, the Traitor King: creature cards in your graveyard have unearth
/// {2}{B}.
#[test]
fn sedris_grants_unearth() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::sedris_the_traitor_king());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    activate(&mut g, bear, 0, None).expect("unearth");
    assert!(g.battlefield_find(bear).is_some());
}

/// Obeka, Brute Chronologist (CR 724): its tap lets the active player end the
/// turn — the stack is exiled and the turn skips to cleanup.
#[test]
fn obeka_ends_the_turn() {
    let mut g = pod(2);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    let ob = ready(&mut g, 0, catalog::obeka_brute_chronologist());
    activate(&mut g, ob, 0, None).expect("tap");
    assert!(matches!(g.step, TurnStep::Cleanup | TurnStep::Untap | TurnStep::Upkeep) || g.active_player_idx != 0, "turn ended at {:?}", g.step);
}

/// Mazzy, Truesword Paladin: an Aura of yours dying is exiled and castable;
/// an enchanted attacker gets +2/+0.
#[test]
fn mazzy_recasts_dead_auras() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::mazzy_truesword_paladin());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let str_ = g.add_card_to_hand(0, catalog::holy_strength());
    cast(&mut g, str_, &[perm(bear)]).expect("holy strength");
    declare(&mut g, &[(bear, 1)]).expect("attack");
    assert_eq!(pt(&g, bear), (5, 4), "+1/+2 aura and +2/+0");
    let dis = g.add_card_to_hand(1, catalog::disenchant());
    cast_as(&mut g, 1, dis, &[perm(str_)]).expect("disenchant the Aura");
    let aura = g.exile.iter().find(|c| c.id == str_).expect("the Aura went to exile");
    assert!(aura.may_play_until.is_some(), "castable from exile");
}

/// Unbridled Growth: the enchanted land taps for any color.
#[test]
fn unbridled_growth_fixes_a_land() {
    let mut g = pod(2);
    let land = ready(&mut g, 0, catalog::mountain());
    let ug = g.add_card_to_hand(0, catalog::unbridled_growth());
    cast(&mut g, ug, &[perm(land)]).expect("aura");
    g.players[0].mana_pool.empty();
    let idx = catalog::mountain().activated_abilities.len();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: idx,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("granted any-color tap");
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

/// Kellan, Planar Trailblazer: Scout → Detective (impulse on hit) → 3/2
/// double-striking Rogue; the second upgrade needs the first.
#[test]
fn kellan_levels_up_in_order() {
    let mut g = pod(2);
    let k = ready(&mut g, 0, catalog::kellan_planar_trailblazer());
    activate(&mut g, k, 1, None).expect("rogue step does nothing yet");
    assert_eq!(pt(&g, k), (2, 1));
    activate(&mut g, k, 0, None).expect("detective");
    activate(&mut g, k, 1, None).expect("rogue");
    assert_eq!(pt(&g, k), (3, 2));
    assert!(g.computed_permanent(k).unwrap().keywords().contains(&Keyword::DoubleStrike));
}

/// View from Above: with a white permanent it returns to your hand.
#[test]
fn view_from_above_comes_back_with_white() {
    for (white, back) in [(false, false), (true, true)] {
        let mut g = pod(2);
        let bear = ready(&mut g, 0, catalog::grizzly_bears());
        if white {
            ready(&mut g, 0, catalog::savannah_lions());
        }
        let v = g.add_card_to_hand(0, catalog::view_from_above());
        cast(&mut g, v, &[perm(bear)]).expect("view");
        assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Flying));
        assert_eq!(g.players[0].hand.iter().any(|c| c.id == v), back);
    }
}

/// Consuming Tide: each player keeps one nonland permanent; the rest bounce.
#[test]
fn consuming_tide_keeps_one_each() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::serra_angel());
    ready(&mut g, 1, catalog::gray_ogre());
    ready(&mut g, 1, catalog::grizzly_bears());
    ready(&mut g, 1, catalog::forest());
    library(&mut g, 0, 2);
    let ct = g.add_card_to_hand(0, catalog::consuming_tide());
    cast(&mut g, ct, &[]).expect("tide");
    let nonland = |s: usize| g.battlefield.iter().filter(|c| c.controller == s && !c.definition.is_land()).count();
    assert_eq!((nonland(0), nonland(1)), (1, 1));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Forest"), "lands stay");
}
