//! Commander: the two-card EDHREC gaps of commanders ranked 1001-1500
//! (`decks::cmdr_edhrec15`).

use crabomination::card::{CardId, CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, StackItem, Target, TurnStep};
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

/// The Marvel modal DFCs whose front also transforms (CR 712.11b): each back
/// face is cast straight from hand for its printed cost.
#[test]
fn modal_transform_backs_are_cast_from_hand() {
    let cards: [(crabomination::cube::CardFactory, &str); 11] = [
        (catalog::king_tchalla, "Black Panther, Hope Enduring"),
        (catalog::tony_stark, "The Invincible Iron Man"),
        (catalog::monica_rambeau, "Photon, Living Light"),
        (catalog::norman_osborn, "Green Goblin"),
        (catalog::jennifer_walters, "The Sensational She-Hulk"),
        (catalog::eddie_brock, "Venom, Lethal Protector"),
        (catalog::magik_illyana_rasputin, "Darkchild, Demon Uncaged"),
        (catalog::gwen_stacy, "Ghost-Spider"),
        (catalog::bruce_banner, "The Incredible Hulk"),
        (catalog::peter_parker, "Amazing Spider-Man"),
        (catalog::miles_morales, "Ultimate Spider-Man"),
    ];
    for (f, back) in cards {
        let mut g = pod(2);
        let id = g.add_card_to_hand(0, f());
        flood(&mut g, 0);
        g.perform_action(GameAction::CastSpellBack {
            card_id: id,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .unwrap_or_else(|e| panic!("{back}: {e:?}"));
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(id).map(|c| c.def.name), Some(back));
    }
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

/// Says yes to Doors of Durin's optional reveal; everything else is the auto
/// decider's.
struct YesToReveal;
impl crabomination::decision::Decider for YesToReveal {
    fn decide(&mut self, d: &crabomination::decision::Decision) -> crabomination::decision::DecisionAnswer {
        match d {
            crabomination::decision::Decision::OptionalTrigger { .. } => {
                crabomination::decision::DecisionAnswer::Bool(true)
            }
            _ => crabomination::decision::AutoDecider.decide(d),
        }
    }
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
    // "You may reveal" — take it (the auto decider declines optional asks).
    g.decider = Box::new(YesToReveal);
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
    // Haste from the counters, on a creature that has none of its own.
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let haste = |g: &GameState| g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Haste);
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    assert!(!haste(&g), "two counters");
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert!(haste(&g), "three counters");
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

/// Wheel of Potential: seven {E} paid wheels every hand for seven, and you
/// may play your own exiled cards until the end of your next turn.
#[test]
fn wheel_of_potential_wheels_for_the_energy_paid() {
    let mut g = pod(2);
    library(&mut g, 0, 10);
    library(&mut g, 1, 10);
    g.players[0].energy = 4;
    let keep = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::island());
    let wheel = g.add_card_to_hand(0, catalog::wheel_of_potential());
    cast(&mut g, wheel, &[]).expect("wheel");
    assert_eq!(g.players[0].energy, 0, "the bot pays all seven");
    assert_eq!(g.players[0].hand.len(), 7);
    assert_eq!(g.players[1].hand.len(), 7, "a one-card hand takes seven");
    let bolt = g.exile.iter().find(|c| c.id == keep).expect("exiled");
    assert!(bolt.may_play_until.is_some(), "playable from exile");
}

/// Gauntlet of Power: the chosen color's creatures get +1/+1 and its basics
/// make one more of that color.
#[test]
fn gauntlet_of_power_doubles_the_chosen_basics() {
    let mut g = pod(2);
    let mtn = ready(&mut g, 0, catalog::mountain());
    let isl = ready(&mut g, 0, catalog::island());
    let gp = ready(&mut g, 0, catalog::gauntlet_of_power());
    g.battlefield_find_mut(gp).unwrap().chosen_color = Some(Color::Red);
    let gob = ready(&mut g, 1, catalog::goblin_guide());
    assert_eq!(pt(&g, gob), (3, 3), "every player's red creature");
    let tap = |g: &mut GameState, id: CardId| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("tap");
    };
    tap(&mut g, mtn);
    tap(&mut g, isl);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 2);
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1);
}

/// Storm Cauldron (CR 305.2): every player gets an extra land drop; a land
/// tapped for mana returns to its owner's hand.
#[test]
fn storm_cauldron_bounces_tapped_lands() {
    let mut g = pod(2);
    ready(&mut g, 1, catalog::storm_cauldron());
    let mtn = ready(&mut g, 0, catalog::mountain());
    assert!(g.can_player_play_land(0) && g.extra_land_plays_per_turn(0) == 1, "the other player's Cauldron counts");
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: mtn,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == mtn));
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "the mana stays");
}

/// Sheoldred (MOM): ETB edicts each opponent; it transforms only once an
/// opponent's graveyard holds eight cards; The True Scriptures III returns
/// every creature card in every graveyard to you.
#[test]
fn sheoldred_edicts_then_reads_the_scriptures() {
    let mut g = pod(2);
    let ogre = ready(&mut g, 1, catalog::gray_ogre());
    let sh = g.add_card_to_hand(0, catalog::sheoldred());
    cast(&mut g, sh, &[]).expect("sheoldred");
    assert!(g.battlefield_find(ogre).is_none(), "edicted");
    assert!(activate(&mut g, sh, 0, None).is_err(), "seven cards isn't enough");
    for _ in 0..7 {
        g.add_card_to_graveyard(1, catalog::grizzly_bears());
    }
    activate(&mut g, sh, 0, None).expect("eight cards");
    let saga = g.battlefield.iter().find(|c| c.definition.name == "The True Scriptures").map(|c| c.id).expect("flipped");
    g.saga_advance(saga);
    drain_stack(&mut g);
    g.saga_advance(saga);
    drain_stack(&mut g);
    let mine = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Grizzly Bears").count();
    assert_eq!(mine, 7, "every Bears came back under your control");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Sheoldred"), "front face up again");
}

/// Unseal the Necropolis: each player mills three; you take back two creature
/// cards.
#[test]
fn unseal_the_necropolis_returns_two() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(1, catalog::island());
    }
    let us = g.add_card_to_hand(0, catalog::unseal_the_necropolis());
    cast(&mut g, us, &[]).expect("unseal");
    assert_eq!(g.players[0].hand.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 2);
    assert_eq!(g.players[1].graveyard.len(), 3);
}

/// Boseiju Reaches Skyward: I fetches two basic Forests; III flips it into a
/// Plant that counts your lands.
#[test]
fn boseiju_reaches_skyward_grows_a_branch() {
    let mut g = pod(2);
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::forest());
    }
    ready(&mut g, 0, catalog::forest());
    let bs = g.add_card_to_hand(0, catalog::boseiju_reaches_skyward());
    cast(&mut g, bs, &[]).expect("saga");
    assert_eq!(g.players[0].hand.iter().filter(|c| c.definition.name == "Forest").count(), 2);
    g.saga_advance(bs);
    drain_stack(&mut g);
    g.saga_advance(bs);
    drain_stack(&mut g);
    let branch = g.battlefield.iter().find(|c| c.definition.name == "Branch of Boseiju").map(|c| c.id).expect("flipped");
    assert_eq!(pt(&g, branch), (1, 1), "one land");
}

/// Waking the Trolls III: a Troll per land you have over the target opponent.
#[test]
fn waking_the_trolls_makes_the_difference() {
    let mut g = pod(2);
    for _ in 0..4 {
        ready(&mut g, 0, catalog::forest());
    }
    ready(&mut g, 1, catalog::forest());
    let wt = g.add_card_to_battlefield(0, catalog::waking_the_trolls());
    g.battlefield.iter_mut().filter(|c| c.id == wt).for_each(|c| c.add_counters(CounterType::Lore, 2));
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Target(Target::Player(1)),
    ]));
    g.saga_advance(wt);
    drain_stack(&mut g);
    assert_eq!(tokens_named(&g, "Troll Warrior"), 3);
}

/// Crovax, Ascendant Hero: other white creatures +1/+1, nonwhite -1/-1.
#[test]
fn crovax_splits_the_board_by_white() {
    let mut g = pod(2);
    let cx = ready(&mut g, 0, catalog::crovax_ascendant_hero());
    let lions = ready(&mut g, 1, catalog::savannah_lions());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    assert_eq!(pt(&g, cx), (4, 4));
    assert_eq!(pt(&g, lions), (3, 2));
    assert_eq!(pt(&g, bear), (1, 1));
}

/// Grafted Butcher: returns from the graveyard for {3}{B} and a sacrifice.
#[test]
fn grafted_butcher_returns_for_a_sacrifice() {
    let mut g = pod(2);
    let fodder = ready(&mut g, 0, catalog::grizzly_bears());
    let gb = g.add_card_to_graveyard(0, catalog::grafted_butcher());
    activate(&mut g, gb, 0, None).expect("return");
    assert!(g.battlefield_find(gb).is_some());
    assert!(g.battlefield_find(fodder).is_none());
}

/// Nevinyrral, Urborg Tyrant (CR 702.11d): an opponent's creature ability
/// can't target it, an opponent's instant can. ETB makes a tapped Zombie per
/// creature that died this turn.
#[test]
fn nevinyrral_is_hexproof_from_permanents_and_counts_the_dead() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[perm(bear)]).expect("one died");
    let nev = g.add_card_to_hand(0, catalog::nevinyrral_urborg_tyrant());
    cast(&mut g, nev, &[]).expect("nevinyrral");
    let zombies: Vec<_> = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Zombie").collect();
    assert_eq!(zombies.len(), 1);
    assert!(zombies[0].tapped);
    let pinger = ready(&mut g, 1, catalog::prodigal_pyromancer());
    g.priority.player_with_priority = 1;
    let ping = |t: Target| GameAction::ActivateAbility {
        card_id: pinger,
        ability_index: 0,
        target: Some(t),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    };
    assert!(g.clone().perform_action(ping(perm(nev))).is_err(), "a creature's ability can't target it");
    let shock = g.add_card_to_hand(1, catalog::shock());
    cast_as(&mut g, 1, shock, &[perm(nev)]).expect("an instant can");
}

/// Rise of the Dread Marn: a Zombie Berserker per nontoken creature that died
/// this turn.
#[test]
fn rise_of_the_dread_marn_counts_nontoken_deaths() {
    let mut g = pod(2);
    for _ in 0..2 {
        let b = ready(&mut g, 1, catalog::grizzly_bears());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast(&mut g, bolt, &[perm(b)]).expect("bolt");
    }
    let r = g.add_card_to_hand(0, catalog::rise_of_the_dread_marn());
    cast(&mut g, r, &[]).expect("rise");
    assert_eq!(tokens_named(&g, "Zombie Berserker"), 2);
}

/// Bane, Lord of Darkness: indestructible at half life; a nontoken creature
/// of yours dying draws.
#[test]
fn bane_turns_indestructible_at_half_life() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    let bane = ready(&mut g, 0, catalog::bane_lord_of_darkness());
    let ind = |g: &GameState| g.computed_permanent(bane).unwrap().keywords().contains(&Keyword::Indestructible);
    assert!(!ind(&g));
    g.players[0].life = 10;
    assert!(ind(&g), "10 of 20");
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, bolt, &[perm(bear)]).expect("bolt");
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Body Launderer: dying returns a non-Rogue creature card with lesser or
/// equal power.
#[test]
fn body_launderer_returns_a_smaller_creature() {
    let mut g = pod(2);
    let bl = ready(&mut g, 0, catalog::body_launderer());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, bolt, &[perm(bl)]).expect("bolt the 3/3");
    assert!(g.battlefield_find(bear).is_some());
}

/// Adept Watershaper: your other tapped creatures are indestructible, the
/// untapped ones aren't.
#[test]
fn adept_watershaper_shields_tapped_creatures() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::adept_watershaper());
    let tapped = ready(&mut g, 0, catalog::grizzly_bears());
    let untapped = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(tapped).unwrap().tapped = true;
    let ind = |g: &GameState, id| g.computed_permanent(id).unwrap().keywords().contains(&Keyword::Indestructible);
    assert!(ind(&g, tapped));
    assert!(!ind(&g, untapped));
}

/// Titania, Voice of Gaea and Argoth (CR 701.42 meld, CR 603.4 intervening
/// "if"): Argoth's bear-and-mill gains 2 life once for the batch of lands;
/// with four land cards in the graveyard the upkeep melds them into Titania,
/// Gaea Incarnate, which returns those lands and is sized by your lands.
#[test]
fn titania_and_argoth_meld_at_four_graveyard_lands() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let titania = ready(&mut g, 0, catalog::titania_voice_of_gaea());
    let argoth = ready(&mut g, 0, catalog::argoth_sanctum_of_nature());
    // A land besides Argoth, or the 0/0 melded Titania dies before her
    // enter trigger returns any (CR 704.5f).
    ready(&mut g, 0, catalog::plains());
    let life = g.players[0].life;
    activate(&mut g, argoth, 1, None).expect("bear and mill");
    assert_eq!(tokens_named(&g, "Bear"), 1);
    assert_eq!(g.players[0].life, life + 2, "one gain for the three-land batch");
    // Three lands in the graveyard: no meld yet.
    g.battlefield_find_mut(argoth).unwrap().tapped = false;
    let to_upkeep = |g: &mut GameState| {
        g.step = TurnStep::Untap;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::Upkeep {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        drain_stack(g);
    };
    to_upkeep(&mut g);
    assert!(g.battlefield_find(titania).is_some(), "three lands is not enough");
    g.add_card_to_graveyard(0, catalog::island());
    to_upkeep(&mut g);
    let gaea = g.battlefield.iter().find(|c| c.definition.name == "Titania, Gaea Incarnate").map(|c| c.id);
    let gaea = gaea.expect("melded");
    assert!(g.players[0].graveyard.iter().all(|c| !c.definition.is_land()), "the lands came back");
    let lands = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count() as i32;
    assert_eq!(lands, 5, "the Plains and four returned lands");
    assert_eq!(pt(&g, gaea), (5, 5));
}

/// Titania, Gaea Incarnate: {3}{G} animates a land of yours as a 4/4
/// Elemental with haste.
#[test]
fn titania_gaea_incarnate_awakens_a_land() {
    let mut g = pod(2);
    let gaea = ready(&mut g, 0, catalog::titania_gaea_incarnate());
    let forest = ready(&mut g, 0, catalog::forest());
    activate(&mut g, gaea, 0, Some(perm(forest))).expect("awaken");
    let c = g.computed_permanent(forest).unwrap();
    let types = c.card_types();
    assert!(types.contains(&CardType::Creature) && types.contains(&CardType::Land), "a creature land");
    assert_eq!((c.power, c.toughness), (4, 4));
    assert_eq!(pt(&g, gaea), (1, 1), "one land");
}

fn activate_with_pool(g: &mut GameState, id: CardId, index: usize, colorless: u32) -> Result<(), String> {
    g.players[0].mana_pool.add_colorless(colorless);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// Mishra, Excavation Prodigy (CR 603.2c): discarding an artifact card adds
/// {R}{R}, once each turn.
#[test]
fn mishra_excavation_prodigy_adds_red_once_a_turn() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    let mishra = ready(&mut g, 0, catalog::mishra_excavation_prodigy());
    g.add_card_to_hand(0, catalog::ornithopter());
    activate_with_pool(&mut g, mishra, 0, 1).expect("rummage");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 2);
    g.battlefield_find_mut(mishra).unwrap().tapped = false;
    g.players[0].hand.clear();
    g.add_card_to_hand(0, catalog::ornithopter());
    activate_with_pool(&mut g, mishra, 0, 1).expect("rummage again");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 2, "once each turn");
}

/// Urza, Powerstone Prodigy: looting away an artifact card makes a tapped
/// Powerstone; a nonartifact discard doesn't.
#[test]
fn urza_powerstone_prodigy_makes_a_tapped_powerstone() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::ornithopter());
    let urza = ready(&mut g, 0, catalog::urza_powerstone_prodigy());
    g.add_card_to_hand(0, catalog::ornithopter());
    activate_with_pool(&mut g, urza, 0, 1).expect("loot");
    let stones: Vec<_> = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Powerstone").collect();
    assert_eq!(stones.len(), 1);
    assert!(stones[0].tapped);
}

/// Queen Kayla bin-Kroog: discard the hand, draw that many, and one
/// artifact or creature card each of mana value 1, 2 and 3 returns; a
/// five-drop stays in the graveyard.
#[test]
fn queen_kayla_returns_one_of_each_low_mana_value() {
    let mut g = pod(2);
    library(&mut g, 0, 5);
    let kayla = ready(&mut g, 0, catalog::queen_kayla_bin_kroog());
    let picks = [catalog::savannah_lions(), catalog::grizzly_bears(), catalog::gray_ogre(), catalog::serra_angel()];
    let ids: Vec<_> = picks.into_iter().map(|d| g.add_card_to_hand(0, d)).collect();
    activate(&mut g, kayla, 0, None).expect("wheel");
    assert_eq!(g.players[0].hand.len(), 4, "drew four");
    for id in &ids[..3] {
        assert!(g.battlefield_find(*id).is_some(), "{id:?} returned");
    }
    assert!(g.players[0].graveyard.iter().any(|c| c.id == ids[3]), "the angel stays");
}

/// Nexus of Becoming: at your beginning of combat, draw, then exile an
/// artifact card from hand for a 3/3 Golem artifact-creature copy of it.
#[test]
fn nexus_of_becoming_makes_a_golem_copy() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::forest());
    ready(&mut g, 0, catalog::nexus_of_becoming());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    while g.step != TurnStep::BeginCombat {
        g.priority.player_with_priority = g.active_player_idx;
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == ring), "the Sol Ring was exiled");
    let golem = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Sol Ring").map(|c| c.id).expect("copy");
    let c = g.computed_permanent(golem).unwrap();
    assert!(c.def.is_creature() && c.def.is_artifact());
    assert_eq!((c.power, c.toughness), (3, 3));
}

/// Drillworks Mole grows itself; Farmstead Gleaner's {Q} ability needs it
/// tapped and untaps it; Marvin borrows the Mole's ability.
#[test]
fn drillworks_mole_and_farmstead_gleaner_grow() {
    let mut g = pod(2);
    let mole = ready(&mut g, 0, catalog::drillworks_mole());
    activate(&mut g, mole, 0, None).expect("mole");
    assert_eq!(pt(&g, mole), (2, 2));
    let gleaner = ready(&mut g, 0, catalog::farmstead_gleaner());
    assert!(activate(&mut g, gleaner, 0, None).is_err(), "{{Q}} needs it tapped");
    g.battlefield_find_mut(gleaner).unwrap().tapped = true;
    activate(&mut g, gleaner, 0, None).expect("untap for a counter");
    assert!(!g.battlefield_find(gleaner).unwrap().tapped);
    assert_eq!(pt(&g, gleaner), (3, 3));
    let marvin = ready(&mut g, 0, catalog::marvin_murderous_mimic());
    activate(&mut g, marvin, 0, None).expect("Marvin uses the Mole's ability");
    assert_eq!(pt(&g, marvin), (3, 3));
}

/// Jackal, Genius Geneticist (CR 707.10, 707.9b): a creature spell with
/// mana value equal to its power is copied and Jackal grows; one that no
/// longer matches isn't.
#[test]
fn jackal_copies_a_creature_spell_at_its_power() {
    let mut g = pod(2);
    let jackal = ready(&mut g, 0, catalog::jackal_genius_geneticist());
    let lions = g.add_card_to_hand(0, catalog::savannah_lions());
    cast(&mut g, lions, &[]).expect("lions");
    let named = |g: &GameState, n: &str| g.battlefield.iter().filter(|c| c.definition.name == n).count();
    assert_eq!(named(&g, "Savannah Lions"), 2, "the lions and a token copy");
    assert_eq!(pt(&g, jackal), (2, 2));
    let more = g.add_card_to_hand(0, catalog::savannah_lions());
    cast(&mut g, more, &[]).expect("lions again");
    assert_eq!(named(&g, "Savannah Lions"), 3, "mana value 1 against power 2: no copy");
}

/// Errant, Street Artist (CR 707.10): a cast spell is no target; a copy of
/// one ("a spell that wasn't cast") is, and Errant copies it again.
#[test]
fn errant_copies_only_a_spell_that_wasnt_cast() {
    let mut g = pod(2);
    let errant = ready(&mut g, 0, catalog::errant_street_artist());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    let ability = |g: &mut GameState, t: CardId| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: errant,
            ability_index: 0,
            target: Some(perm(t)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(ability(&mut g, bolt).is_err(), "the Bolt was cast");
    let rev = g.add_card_to_hand(0, catalog::reverberate());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: rev,
        target: Some(perm(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("reverberate");
    g.resolve_top_of_stack().expect("copy the bolt");
    let copy = g.stack.iter().rev().find_map(|s| match s {
        StackItem::Spell { card, .. } if card.is_token => Some(card.id),
        _ => None,
    });
    let copy = copy.expect("a copy on the stack");
    ability(&mut g, copy).expect("the copy wasn't cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20 - 9, "three Bolts");
}

/// Lluwen, Imperfect Naturalist: mills four and may keep a creature on top;
/// its Worm ability counts the land cards in the graveyard.
#[test]
fn lluwen_mills_and_makes_worms() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.add_card_to_library(0, catalog::grizzly_bears());
    let lluwen = g.add_card_to_hand(0, catalog::lluwen_imperfect_naturalist());
    cast(&mut g, lluwen, &[]).expect("lluwen");
    let top = g.players[0].library.first().map(|c| c.definition.name);
    assert!(matches!(top, Some("Grizzly Bears" | "Forest")), "a milled creature or land went back on top: {top:?}");
    let lands_in_gy = |g: &GameState| g.players[0].graveyard.iter().filter(|c| c.definition.is_land()).count();
    let before = lands_in_gy(&g);
    g.clear_sickness(lluwen);
    g.add_card_to_hand(0, catalog::island());
    activate(&mut g, lluwen, 0, None).expect("worms");
    assert_eq!(tokens_named(&g, "Worm"), before + 1, "the discarded Island counts");
}

/// Tectonic Split: casting it sacrifices half your lands, rounded up; the
/// rest tap for three mana of one color.
#[test]
fn tectonic_split_halves_the_lands_and_triples_the_rest() {
    let mut g = pod(2);
    let lands: Vec<_> = (0..5).map(|_| ready(&mut g, 0, catalog::forest())).collect();
    let split = g.add_card_to_hand(0, catalog::tectonic_split());
    cast(&mut g, split, &[]).expect("split");
    let left: Vec<_> = lands.iter().copied().filter(|&l| g.battlefield_find(l).is_some()).collect();
    assert_eq!(left.len(), 2, "three of five sacrificed");
    g.players[0].mana_pool = Default::default();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: left[0],
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("three mana");
    drain_stack(&mut g);
    let total: u32 = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green]
        .into_iter()
        .map(|c| g.players[0].mana_pool.amount(c))
        .sum();
    assert_eq!(total, 3);
}

/// Inniaz, the Gale Force (CR 101.4): three fliers attacking swap a
/// nonland permanent each way with the player to the right.
#[test]
fn inniaz_rotates_permanents_rightward() {
    let mut g = pod(2);
    let inniaz = ready(&mut g, 0, catalog::inniaz_the_gale_force());
    let birds: Vec<_> = (0..2).map(|_| ready(&mut g, 0, catalog::ornithopter())).collect();
    let angel = ready(&mut g, 1, catalog::serra_angel());
    declare(&mut g, &[(inniaz, 1), (birds[0], 1), (birds[1], 1)]).expect("attack");
    assert_eq!(g.battlefield_find(angel).unwrap().controller, 0, "Inniaz's side takes the Angel");
    let given = g.battlefield.iter().filter(|c| c.controller == 1).count();
    assert_eq!(given, 1, "and hands one permanent across");
}

/// Faerie Slumber Party: every creature goes home; two Faeries per opponent
/// who had one.
#[test]
fn faerie_slumber_party_counts_opponents_with_creatures() {
    let mut g = pod(3);
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let p = g.add_card_to_hand(0, catalog::faerie_slumber_party());
    cast(&mut g, p, &[]).expect("party");
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none());
    assert!(g.players[1].hand.iter().any(|c| c.id == theirs));
    assert_eq!(tokens_named(&g, "Faerie"), 2, "only one opponent had a creature");
    // Two opponents with creatures: four Faeries from one bounce.
    let mut g = pod(3);
    ready(&mut g, 1, catalog::grizzly_bears());
    ready(&mut g, 2, catalog::grizzly_bears());
    let p = g.add_card_to_hand(0, catalog::faerie_slumber_party());
    cast(&mut g, p, &[]).expect("party");
    assert_eq!(tokens_named(&g, "Faerie"), 4);
}

/// Tezzeret, Artifice Master: +1 makes a flying Thopter; 0 draws two with
/// three artifacts out; the −9 emblem fetches a permanent at your end step.
#[test]
fn tezzeret_artifice_master_thopters_and_draws() {
    let mut g = pod(2);
    library(&mut g, 0, 4);
    let tez = ready(&mut g, 0, catalog::tezzeret_artifice_master());
    let loyal = |g: &mut GameState, i: usize| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: tez, ability_index: i, target: None, x_value: None })
            .expect("loyalty");
        drain_stack(g);
    };
    loyal(&mut g, 0);
    assert_eq!(tokens_named(&g, "Thopter"), 1);
    g.battlefield_find_mut(tez).unwrap().loyalty_uses_this_turn = 0;
    ready(&mut g, 0, catalog::ornithopter());
    ready(&mut g, 0, catalog::ornithopter());
    let hand = g.players[0].hand.len();
    loyal(&mut g, 1);
    assert_eq!(g.players[0].hand.len(), hand + 2, "three artifacts: two cards");
}

/// Karfell Harbinger (CR 106.6): its {U} casts an instant but not a
/// creature spell.
#[test]
fn karfell_harbinger_mana_is_for_instants_and_sorceries() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    ready(&mut g, 0, catalog::karfell_harbinger());
    let sailor = g.add_card_to_hand(0, catalog::spectral_sailor());
    let try_cast = |g: &mut GameState, id: CardId| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
    };
    assert!(try_cast(&mut g, sailor).is_err(), "not a creature spell");
    let opt = g.add_card_to_hand(0, catalog::opt());
    try_cast(&mut g, opt).expect("an instant");
}

/// Poppet Stitcher // Poppet Factory: instants and sorceries make decayed
/// Zombies; at three tokens the upkeep flips it, and the Factory makes the
/// tokens vanilla 3/3s.
#[test]
fn poppet_stitcher_makes_zombies_then_a_factory() {
    let mut g = pod(2);
    library(&mut g, 0, 6);
    let stitcher = ready(&mut g, 0, catalog::poppet_stitcher());
    for _ in 0..3 {
        let opt = g.add_card_to_hand(0, catalog::opt());
        cast(&mut g, opt, &[]).expect("opt");
    }
    let zombies: Vec<CardId> = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Zombie").map(|c| c.id).collect();
    assert_eq!(zombies.len(), 3);
    assert!(g.computed_permanent(zombies[0]).unwrap().keywords().contains(&Keyword::Decayed));
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    g.step = TurnStep::Untap;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::Upkeep {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(stitcher).unwrap().transformed, "Poppet Factory");
    let z = g.computed_permanent(zombies[0]).unwrap();
    assert_eq!((z.power, z.toughness), (3, 3));
    assert!(!z.keywords().contains(&Keyword::Decayed), "lost all abilities");
}

/// The Sentry hands an opponent The Void; Lucy MacLean's Golden Rule copies
/// that token for a player other than its controller (drawing if that's an
/// opponent), once each turn.
#[test]
fn the_sentry_gives_the_void_and_lucy_copies_it() {
    let mut g = pod(3);
    library(&mut g, 0, 3);
    ready(&mut g, 0, catalog::lucy_maclean_positively_armed());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    let sentry = g.add_card_to_hand(0, catalog::the_sentry_golden_guardian());
    cast(&mut g, sentry, &[Target::Player(1)]).expect("sentry");
    let voids: Vec<_> = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "The Void").map(|c| c.controller).collect();
    assert_eq!(voids.len(), 2, "The Void and Lucy's copy: {voids:?}");
    assert_eq!(voids[0], 1, "the opponent got The Void");
    assert_ne!(voids[1], 1, "the copy went to a player other than its controller");
    let drew = if voids[1] == 0 { 0 } else { 1 };
    assert_eq!(g.players[0].hand.len(), hand + drew, "draw only when an opponent made the copy");
}

/// Superior Spider-Man (CR 707.9b, 603.12): enters as a 4/4 copy of a
/// graveyard creature card, keeping its name and adding Spider Human Hero,
/// then exiles the copied card.
#[test]
fn superior_spider_man_copies_a_graveyard_creature() {
    let mut g = pod(2);
    let angel = g.add_card_to_graveyard(1, catalog::serra_angel());
    let spidey = g.add_card_to_hand(0, catalog::superior_spider_man());
    cast(&mut g, spidey, &[]).expect("spider-man");
    let c = g.computed_permanent(spidey).expect("on the battlefield");
    assert_eq!(c.def.name, "Superior Spider-Man");
    assert!(c.keywords().contains(&Keyword::Flying), "Serra's flying");
    assert_eq!((c.power, c.toughness), (4, 4));
    assert!(g.exile.iter().any(|e| e.id == angel), "the Angel card was exiled");
}

/// Taskmaster, Mercenary Mimic: at your first main phase he becomes a copy
/// of a creature, keeping his name and his legendary Human Mercenary
/// Villain type line.
#[test]
fn taskmaster_copies_at_the_first_main_phase() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    let tm = ready(&mut g, 0, catalog::taskmaster_mercenary_mimic());
    ready(&mut g, 1, catalog::serra_angel());
    g.step = TurnStep::Draw;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::PreCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    let c = g.computed_permanent(tm).unwrap();
    assert_eq!(c.def.name, "Taskmaster, Mercenary Mimic");
    assert!(c.keywords().contains(&Keyword::Flying) && c.keywords().contains(&Keyword::Vigilance));
    assert_eq!((c.power, c.toughness), (4, 4));
    assert!(c.def.subtypes.creature_types.contains(&crabomination::card::CreatureType::Mercenary));
    assert!(!c.def.subtypes.creature_types.contains(&crabomination::card::CreatureType::Angel));
}

/// Huatli, Poet of Unity // Roar of the Fifth People: fetches a basic, then
/// flips into the Saga — Dinosaurs, a creature mana grant from chapter II,
/// a Dinosaur tutor, then double strike and trample.
#[test]
fn huatli_flips_into_roar_of_the_fifth_people() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let huatli = g.add_card_to_hand(0, catalog::huatli_poet_of_unity());
    cast(&mut g, huatli, &[]).expect("huatli");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"), "a basic to hand");
    g.clear_sickness(huatli);
    activate(&mut g, huatli, 0, None).expect("flip");
    let saga = g.battlefield.iter().find(|c| c.definition.name == "Roar of the Fifth People").map(|c| c.id).expect("saga");
    assert_eq!(tokens_named(&g, "Dinosaur"), 2);
    let dino = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Dinosaur").map(|c| c.id).unwrap();
    g.clear_sickness(dino);
    assert!(activate(&mut g, dino, 0, None).is_err(), "no mana grant before chapter II");
    g.saga_advance(saga);
    drain_stack(&mut g);
    g.players[0].mana_pool = Default::default();
    activate(&mut g, dino, 0, None).expect("chapter II: {T}: R, G or W");
    g.saga_advance(saga);
    drain_stack(&mut g);
    g.saga_advance(saga);
    drain_stack(&mut g);
    let c = g.computed_permanent(dino).unwrap();
    assert!(c.keywords().contains(&Keyword::DoubleStrike) && c.keywords().contains(&Keyword::Trample));
}

/// Intrepid Paleontologist: exiles a graveyard card; a Dinosaur creature
/// card you own exiled this way can be cast and enters with a finality
/// counter.
#[test]
fn intrepid_paleontologist_casts_exiled_dinosaurs() {
    let mut g = pod(2);
    let pal = ready(&mut g, 0, catalog::intrepid_paleontologist());
    let dino = g.add_card_to_graveyard(0, catalog::ghalta_the_unstoppable());
    activate(&mut g, pal, 1, Some(Target::Permanent(dino))).expect("exile");
    assert!(g.exile.iter().any(|c| c.id == dino));
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: dino,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from exile");
    drain_stack(&mut g);
    let c = g.battlefield_find(dino).expect("Ghalta entered");
    assert_eq!(c.counter_count(CounterType::Finality), 1);
}

/// Nashi, Moon's Legacy (CR 707.12): attacking exiles a legendary card from
/// your graveyard and casts a copy of it, paying its cost — the copy of a
/// permanent spell becomes a token.
#[test]
fn nashi_casts_a_copy_of_an_exiled_legend() {
    let mut g = pod(2);
    let nashi = ready(&mut g, 0, catalog::nashi_moons_legacy());
    let bane = g.add_card_to_graveyard(0, catalog::bane_lord_of_darkness());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    flood(&mut g, 0);
    declare(&mut g, &[(nashi, 1)]).expect("attack");
    assert!(g.exile.iter().any(|c| c.id == bane), "the card was exiled");
    assert_eq!(tokens_named(&g, "Bane, Lord of Darkness"), 1, "its copy resolved as a token");
}

/// Elesh Norn (MOM): damage from an opponent's source to you or to a
/// permanent you control costs that source's controller 2 life unless they
/// pay {1}.
#[test]
fn elesh_norn_taxes_damage_to_you_and_yours() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::elesh_norn());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let bolt_at = |g: &mut GameState, t: Target| {
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(t), additional_targets: vec![], mode: None, x_value: None })
            .expect("bolt");
        drain_stack(g);
    };
    bolt_at(&mut g, Target::Player(0));
    assert_eq!(g.players[0].life, 17);
    assert_eq!(g.players[1].life, 18, "no {{1}} to pay: two life");
    bolt_at(&mut g, perm(bear));
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[1].life, 16, "damage to a permanent of yours counts too");
}

/// Sivriss, Nightmare Speaker: one mill per opponent, and each milled card
/// comes back unless that opponent pays 3 life.
#[test]
fn sivriss_mills_one_per_opponent_back_unless_they_pay() {
    let mut g = pod(3);
    library(&mut g, 0, 3);
    let sivriss = ready(&mut g, 0, catalog::sivriss_nightmare_speaker());
    ready(&mut g, 0, catalog::ornithopter());
    let (hand, lives) = (g.players[0].hand.len(), [g.players[1].life, g.players[2].life]);
    activate(&mut g, sivriss, 0, None).expect("sivriss");
    let returned = g.players[0].hand.len() - hand;
    let paid = [1, 2].iter().filter(|&&p| g.players[p].life == lives[p - 1] - 3).count();
    assert_eq!(g.players[0].library.len(), 1, "two opponents, two mills");
    assert_eq!(returned + paid, 2, "each card returned or paid for");
}

/// Tameshi, Reality Architect: returning a land as the cost draws a card
/// (a noncreature permanent went to hand, once a turn), and an artifact
/// with mana value X or less comes back from the graveyard.
#[test]
fn tameshi_bounces_a_land_to_reanimate_an_artifact() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    let tameshi = ready(&mut g, 0, catalog::tameshi_reality_architect());
    let island = ready(&mut g, 0, catalog::island());
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    let hand = g.players[0].hand.len();
    activate_x(&mut g, tameshi, 0, Some(perm(ring)), Some(1)).expect("tameshi");
    assert!(g.battlefield_find(ring).is_some(), "Sol Ring returned");
    assert!(g.players[0].hand.iter().any(|c| c.id == island), "the land went home");
    assert_eq!(g.players[0].hand.len(), hand + 2, "the Island and a drawn card");
}

/// Starving Revenant: both surveiled cards kept on top are drawn at 3 life
/// each; with descend 8 every draw drains the opponent for 1.
#[test]
fn starving_revenant_draws_what_it_keeps_and_descends() {
    let mut g = pod(2);
    library(&mut g, 0, 4);
    for _ in 0..8 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let hand = g.players[0].hand.len();
    let rev = g.add_card_to_hand(0, catalog::starving_revenant());
    cast(&mut g, rev, &[]).expect("revenant");
    assert_eq!(g.players[0].hand.len(), hand + 2, "two kept, two drawn");
    assert_eq!(g.players[0].life, 20 - 6 + 2);
    assert_eq!(g.players[1].life, 18, "descend 8 on each draw");
}

/// Sokrates, Athenian Teacher (CR 615): the targeted attacker's combat
/// damage to a player is prevented, and both players draw half of it.
#[test]
fn sokrates_turns_combat_damage_into_draws() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    library(&mut g, 1, 3);
    let sok = ready(&mut g, 0, catalog::sokrates_athenian_teacher());
    let angel = ready(&mut g, 0, catalog::serra_angel());
    activate(&mut g, sok, 0, Some(perm(angel))).expect("dialogue");
    let hands = (g.players[0].hand.len(), g.players[1].hand.len());
    declare(&mut g, &[(angel, 1)]).expect("attack");
    combat_damage(&mut g);
    assert_eq!(g.players[1].life, 20, "the 4 damage was prevented");
    assert_eq!((g.players[0].hand.len(), g.players[1].hand.len()), (hands.0 + 2, hands.1 + 2));
}

/// Clone Shell: imprints one of the top four face down; when it dies the
/// imprinted creature card enters.
#[test]
fn clone_shell_releases_its_imprinted_creature() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let angel = g.add_card_to_library(0, catalog::serra_angel());
    let shell = g.add_card_to_hand(0, catalog::clone_shell());
    cast(&mut g, shell, &[]).expect("shell");
    assert!(g.exile.iter().any(|c| c.id == angel && c.face_down), "the Angel hides face down");
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, bolt, &[perm(shell)]).expect("bolt");
    assert_eq!(g.battlefield_find(angel).map(|c| c.controller), Some(0));
}

/// Rowan's Talent (CR 707.10): a loyalty ability of the enchanted
/// planeswalker is copied; the Aura's own +1 shows up past the printed ones.
#[test]
fn rowans_talent_copies_loyalty_abilities() {
    let mut g = pod(2);
    let tez = ready(&mut g, 0, catalog::tezzeret_artifice_master());
    let talent = g.add_card_to_hand(0, catalog::rowans_talent());
    cast(&mut g, talent, &[perm(tez)]).expect("talent");
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: tez, ability_index: 0, target: None, x_value: None })
        .expect("+1 thopter");
    drain_stack(&mut g);
    assert_eq!(tokens_named(&g, "Thopter"), 2, "the +1 and its copy");
    g.battlefield_find_mut(tez).unwrap().loyalty_uses_this_turn = 0;
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: tez, ability_index: 3, target: Some(perm(bear)), x_value: None })
        .expect("the Talent's +1");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (6, 2), "+2/+0 twice");
}

/// Vorinclex // The Grand Evolution: two Forests to hand, then the Saga —
/// two milled creatures enter, seven counters spread, a fight grant, and
/// Vorinclex returns front face up.
#[test]
fn vorinclex_runs_the_grand_evolution() {
    let mut g = pod(2);
    for _ in 0..8 {
        g.add_card_to_library(0, catalog::island());
    }
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::forest());
    }
    let v = g.add_card_to_hand(0, catalog::vorinclex());
    cast(&mut g, v, &[]).expect("vorinclex");
    assert_eq!(g.players[0].hand.iter().filter(|c| c.definition.name == "Forest").count(), 2);
    g.clear_sickness(v);
    activate(&mut g, v, 0, None).expect("transform");
    let saga = g.battlefield.iter().find(|c| c.definition.name == "The Grand Evolution").map(|c| c.id).expect("saga");
    let bears = || g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    assert_eq!(bears(), 2, "both milled creatures entered");
    g.saga_advance(saga);
    drain_stack(&mut g);
    let counters: u32 = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .sum();
    assert_eq!(counters, 7);
    g.saga_advance(saga);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Vorinclex"), "front face up again");
}

/// Jacob Hauken, Inspector // Hauken's Insight: the {T} hides a hand card
/// face down and may pay to transform; the Insight hides the top card each
/// upkeep and casts one of its cards free, once during each of your turns.
#[test]
fn jacob_hauken_hides_cards_and_casts_one_free() {
    let mut g = pod(2);
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::lightning_bolt());
    library(&mut g, 0, 3);
    let jacob = ready(&mut g, 0, catalog::jacob_hauken_inspector());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Discard(vec![bolt]),
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    activate(&mut g, jacob, 0, None).expect("inspect");
    assert!(g.exile.iter().any(|c| c.id == bolt && c.face_down), "the drawn Bolt hides face down");
    assert!(g.battlefield_find(jacob).unwrap().transformed, "paid {{4}}{{U}}{{U}}");
    g.step = TurnStep::Untap;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::Upkeep {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    let hidden: Vec<CardId> = g.exile.iter().filter(|c| c.exiled_with == Some(jacob)).map(|c| c.id).collect();
    assert_eq!(hidden.len(), 2, "the upkeep hid the top card too");
    g.step = TurnStep::PreCombatMain;
    let mut unspent = g.clone();
    let free = |g: &mut GameState, id: CardId| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastFromZoneWithoutPaying {
            card_id: id,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    free(&mut g, hidden[0]).expect("one free cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
    assert!(free(&mut g, hidden[1]).is_err(), "once each turn");
    // "Once during each of your turns": a use left unspent on your turn is not
    // there on an opponent's.
    let mut events = Vec::new();
    unspent.do_cleanup(&mut events);
    unspent.active_player_idx = 1;
    assert!(free(&mut unspent, hidden[0]).is_err(), "an opponent's turn");
}

/// Havengul Laboratory // Havengul Mystery: three Clues sacrificed in a
/// turn flip it at the end step, returning a creature card; when that
/// creature leaves, it flips back.
#[test]
fn havengul_laboratory_flips_on_three_clues() {
    let mut g = pod(2);
    library(&mut g, 0, 4);
    let lab = ready(&mut g, 0, catalog::havengul_laboratory());
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    for _ in 0..3 {
        g.battlefield_find_mut(lab).unwrap().tapped = false;
        activate(&mut g, lab, 1, None).expect("investigate");
    }
    let clues: Vec<CardId> = g.battlefield.iter().filter(|c| c.definition.name == "Clue").map(|c| c.id).collect();
    assert_eq!(clues.len(), 3);
    for c in clues {
        activate(&mut g, c, 0, None).expect("crack a Clue");
    }
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(lab).unwrap().transformed, "Havengul Mystery");
    assert!(g.battlefield_find(angel).is_some(), "the Angel came back");
    let blade = g.add_card_to_hand(1, catalog::doom_blade());
    cast_as(&mut g, 1, blade, &[perm(angel)]).expect("doom blade");
    assert!(!g.battlefield_find(lab).unwrap().transformed, "back to the Laboratory");
}

fn loyalty_at(g: &mut GameState, pw: CardId, i: usize, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: pw, ability_index: i, target, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// Garruk, Unleashed: +1 is +3/+3 and trample; −2 makes a Beast and adds a
/// loyalty when an opponent has more creatures. Garruk, Caller of Beasts:
/// +1 takes every creature among the top five.
#[test]
fn the_garruks_pump_beasts_and_dig() {
    let mut g = pod(2);
    let gu = ready(&mut g, 0, catalog::garruk_unleashed());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    loyalty_at(&mut g, gu, 0, Some(perm(bear))).expect("+1");
    assert_eq!(pt(&g, bear), (5, 5));
    for _ in 0..3 {
        ready(&mut g, 1, catalog::grizzly_bears());
    }
    g.battlefield_find_mut(gu).unwrap().loyalty_uses_this_turn = 0;
    loyalty_at(&mut g, gu, 1, None).expect("-2");
    assert_eq!(tokens_named(&g, "Beast"), 1);
    assert_eq!(g.battlefield_find(gu).unwrap().counter_count(CounterType::Loyalty), 5 - 2 + 1, "three Bears beat two");
    let gc = ready(&mut g, 0, catalog::garruk_caller_of_beasts());
    library(&mut g, 0, 2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand = g.players[0].hand.len();
    loyalty_at(&mut g, gc, 0, None).expect("+1");
    assert_eq!(g.players[0].hand.len(), hand + 2, "the two Bears of the top five");
}

/// Copy Artifact enters as a copy of an artifact and stays an enchantment.
#[test]
fn copy_artifact_copies_an_artifact() {
    let mut g = pod(2);
    ready(&mut g, 1, catalog::sol_ring());
    let ca = g.add_card_to_hand(0, catalog::copy_artifact());
    cast(&mut g, ca, &[]).expect("copy artifact");
    let c = g.computed_permanent(ca).unwrap();
    assert_eq!(c.def.name, "Sol Ring");
    assert!(c.card_types().contains(&CardType::Enchantment) && c.card_types().contains(&CardType::Artifact));
}

/// Lim-Dûl's Hex: each player who can't pay {B} or {3} takes 1 at your
/// upkeep.
#[test]
fn lim_duls_hex_pings_each_player_who_cant_pay() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::lim_duls_hex());
    g.step = TurnStep::Untap;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::Upkeep {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!([g.players[0].life, g.players[1].life, g.players[2].life], [19, 19, 19]);
}

/// Wail of the Forgotten: one mode without descend — the bounce.
#[test]
fn wail_of_the_forgotten_bounces() {
    let mut g = pod(2);
    let ring = ready(&mut g, 1, catalog::sol_ring());
    let wail = g.add_card_to_hand(0, catalog::wail_of_the_forgotten());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: wail,
        target: Some(perm(ring)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("wail");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == ring));
}

/// Leonardo da Vinci: looting away an artifact card makes a 0/2 flying
/// Thopter copy of it.
#[test]
fn leonardo_turns_a_discarded_artifact_into_a_thopter() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::sol_ring());
    let leo = ready(&mut g, 0, catalog::leonardo_da_vinci());
    g.players[0].hand.clear();
    activate(&mut g, leo, 1, None).expect("loot");
    let t = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Sol Ring").map(|c| c.id).expect("a copy");
    let c = g.computed_permanent(t).unwrap();
    assert_eq!((c.power, c.toughness), (0, 2));
    assert!(c.keywords().contains(&Keyword::Flying));
    assert!(c.def.subtypes.creature_types.contains(&crabomination::card::CreatureType::Thopter));
}

/// Tarrian's Journal // The Tomb of Aclazotz: discard your hand to flip;
/// the Tomb's creature cast from the graveyard enters with a finality
/// counter as a Vampire.
#[test]
fn tarrians_journal_flips_and_raises_a_vampire() {
    let mut g = pod(2);
    let journal = ready(&mut g, 0, catalog::tarrians_journal());
    g.add_card_to_hand(0, catalog::island());
    activate(&mut g, journal, 1, None).expect("flip");
    assert!(g.players[0].hand.is_empty(), "discarded the hand");
    assert!(g.battlefield_find(journal).unwrap().transformed);
    g.battlefield_find_mut(journal).unwrap().tapped = false;
    activate(&mut g, journal, 1, None).expect("tomb");
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast from the graveyard");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("entered");
    assert_eq!(c.counter_count(CounterType::Finality), 1);
    assert!(c.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Vampire));
}

/// Mirrormind Crown (CR 614.1a): the first token batch each turn is copies
/// of the equipped creature; the second is as printed.
#[test]
fn mirrormind_crown_copies_the_first_tokens_each_turn() {
    let mut g = pod(2);
    let angel = ready(&mut g, 0, catalog::serra_angel());
    let crown = ready(&mut g, 0, catalog::mirrormind_crown());
    g.battlefield_find_mut(crown).unwrap().attached_to = Some(angel);
    let alarm = g.add_card_to_hand(0, catalog::raise_the_alarm());
    cast(&mut g, alarm, &[]).expect("alarm");
    assert_eq!(tokens_named(&g, "Serra Angel"), 2, "two Angels instead of Soldiers");
    let again = g.add_card_to_hand(0, catalog::raise_the_alarm());
    cast(&mut g, again, &[]).expect("alarm again");
    assert_eq!(tokens_named(&g, "Soldier"), 2, "only the first batch each turn");
}

/// Defiler of Dreams: a blue permanent spell draws a card, and its {U} can
/// be 2 life.
#[test]
fn defiler_of_dreams_draws_on_blue_permanents() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    ready(&mut g, 0, catalog::defiler_of_dreams());
    let sailor = g.add_card_to_hand(0, catalog::spectral_sailor());
    let hand = g.players[0].hand.len();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: sailor, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("no mana: {U} paid with 2 life");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 18);
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
}

/// Devour (CR 702.82): Thromok's Devour X eats three tokens for nine
/// counters; Dragon Broodmother's upkeep Dragon devours 2 per creature.
#[test]
fn devour_thromok_and_broodmother() {
    let mut g = pod(2);
    let alarm = g.add_card_to_hand(0, catalog::raise_the_alarm());
    cast(&mut g, alarm, &[]).expect("two Soldiers");
    let one = g.add_card_to_hand(0, catalog::raise_the_alarm());
    cast(&mut g, one, &[]).expect("two more");
    let soldiers: Vec<CardId> = g.battlefield.iter().filter(|c| c.definition.name == "Soldier").map(|c| c.id).collect();
    g.battlefield_find_mut(soldiers[3]).unwrap().controller = 1;
    let thromok = g.add_card_to_hand(0, catalog::thromok_the_insatiable());
    cast(&mut g, thromok, &[]).expect("thromok");
    assert_eq!(pt(&g, thromok), (9, 9), "three devoured, three squared");
    assert_eq!(tokens_named(&g, "Soldier"), 1, "the opponent's stays");
    ready(&mut g, 0, catalog::dragon_broodmother());
    let more = g.add_card_to_hand(0, catalog::raise_the_alarm());
    cast(&mut g, more, &[]).expect("two Soldiers");
    g.step = TurnStep::Untap;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::Upkeep {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    let dragon = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Dragon").map(|c| c.id).expect("a Dragon");
    assert_eq!(pt(&g, dragon), (5, 5), "two Soldiers at two counters each");
}

/// Ellie and Alan, Paleontologists (CR 701.57): exiling a five-drop from
/// the graveyard discovers 5.
#[test]
fn ellie_and_alan_discover_by_the_exiled_creature() {
    let mut g = pod(2);
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    library(&mut g, 0, 2);
    let ea = ready(&mut g, 0, catalog::ellie_and_alan_paleontologists());
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    activate(&mut g, ea, 0, None).expect("discover");
    assert!(g.exile.iter().any(|c| c.id == angel), "the Angel paid the cost");
    let got = g.battlefield_find(bears).is_some() || g.players[0].hand.iter().any(|c| c.id == bears);
    assert!(got, "the Bears were discovered");
}

/// Baru, Wurmspeaker: Wurms are +2/+2 tramplers, and a 6/6 Wurm makes the
/// next Wurm cost {1}{G}.
#[test]
fn baru_discounts_wurms_by_their_power() {
    let mut g = pod(2);
    let baru = ready(&mut g, 0, catalog::baru_wurmspeaker());
    activate(&mut g, baru, 0, None).expect("first Wurm");
    let wurm = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Wurm").map(|c| c.id).unwrap();
    assert_eq!(pt(&g, wurm), (6, 6));
    assert!(g.computed_permanent(wurm).unwrap().keywords().contains(&Keyword::Trample));
    g.battlefield_find_mut(baru).unwrap().tapped = false;
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: baru,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("{7}{G} less 6");
    drain_stack(&mut g);
    assert_eq!(tokens_named(&g, "Wurm"), 2);
}

/// Mossbridge Troll (CR 701.15): destruction regenerates it; tapping 10
/// power of other creatures makes it +20/+20.
#[test]
fn mossbridge_troll_regenerates_and_grows() {
    let mut g = pod(2);
    let troll = ready(&mut g, 0, catalog::mossbridge_troll());
    let angels: Vec<CardId> = (0..3).map(|_| ready(&mut g, 0, catalog::serra_angel())).collect();
    activate(&mut g, troll, 0, None).expect("tap twelve power");
    assert_eq!(pt(&g, troll), (25, 25));
    assert!(angels.iter().all(|a| g.battlefield_find(*a).unwrap().tapped));
    let blade = g.add_card_to_hand(1, catalog::doom_blade());
    cast_as(&mut g, 1, blade, &[perm(troll)]).expect("doom blade");
    assert!(g.battlefield_find(troll).is_some_and(|c| c.tapped), "regenerated, tapped");
}

/// Havi, the All-Father: four historic cards make it indestructible; a
/// legendary creature of yours dying returns a cheaper legend, tapped.
#[test]
fn havi_returns_a_lesser_legend() {
    let mut g = pod(2);
    let havi = ready(&mut g, 0, catalog::havi_the_all_father());
    let ind = |g: &GameState| g.computed_permanent(havi).unwrap().keywords().contains(&Keyword::Indestructible);
    assert!(!ind(&g));
    for _ in 0..4 {
        g.add_card_to_graveyard(0, catalog::sol_ring());
    }
    assert!(ind(&g), "four artifacts in the graveyard");
    let bane = ready(&mut g, 0, catalog::bane_lord_of_darkness());
    let jackal = g.add_card_to_graveyard(0, catalog::jackal_genius_geneticist());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, bolt, &[perm(bane)]).expect("bolt Bane");
    assert!(g.battlefield_find(jackal).is_some_and(|c| c.tapped), "Jackal (2) under Bane (4), tapped");
}

/// Bard Class: legendary creatures enter with a counter; level 2 takes
/// {R}{G} off a legendary spell's colored pips only; level 3 exiles two to
/// play on a legendary cast.
#[test]
fn bard_class_levels_up() {
    let mut g = pod(2);
    library(&mut g, 0, 4);
    let bard = g.add_card_to_hand(0, catalog::bard_class());
    cast(&mut g, bard, &[]).expect("bard class");
    let jackal = g.add_card_to_hand(0, catalog::jackal_genius_geneticist());
    cast(&mut g, jackal, &[]).expect("jackal");
    assert_eq!(g.battlefield_find(jackal).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    activate(&mut g, bard, 0, None).expect("level 2");
    let kayla = g.add_card_to_hand(0, catalog::queen_kayla_bin_kroog());
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: kayla, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{1}{R}{W} less {R}");
    drain_stack(&mut g);
    activate(&mut g, bard, 1, None).expect("level 3");
    let exiled = g.exile.len();
    let bane = g.add_card_to_hand(0, catalog::bane_lord_of_darkness());
    cast(&mut g, bane, &[]).expect("bane");
    assert_eq!(g.exile.len(), exiled + 2, "two exiled to play");
}

/// Korvold, Gleeful Glutton: sacrificing an artifact creature takes {2} off
/// (two card types); its combat damage then counts the permanent types in
/// your graveyard (artifact, creature) for counters and draws.
#[test]
fn korvold_gleeful_glutton_feeds_on_sacrifice_types() {
    let mut g = pod(2);
    library(&mut g, 0, 4);
    let altar = ready(&mut g, 0, catalog::ashnods_altar());
    ready(&mut g, 0, catalog::ornithopter());
    activate(&mut g, altar, 0, None).expect("sacrifice the Ornithopter");
    let korvold = g.add_card_to_hand(0, catalog::korvold_gleeful_glutton());
    for c in [Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 1);
    }
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: korvold, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{5}{B}{R}{G} less {2}, with the altar's {C}{C}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(korvold).is_some());
    let hand = g.players[0].hand.len();
    declare(&mut g, &[(korvold, 1)]).expect("haste");
    combat_damage(&mut g);
    assert_eq!(pt(&g, korvold), (6, 6), "two +1/+1 counters");
    assert_eq!(g.players[0].hand.len(), hand + 2, "two cards");
}

/// Invasion of Ergamon: a Treasure and a rummage on entry; defeated
/// (CR 310.11), Truga Cliffcharger's discard fetches a land.
#[test]
fn invasion_of_ergamon_rummages_then_charges() {
    use crabomination::decision::{DecisionAnswer as A, ScriptedDecider};
    let mut g = pod(2);
    library(&mut g, 0, 3);
    let forest = g.add_card_to_library(0, catalog::forest());
    let first = g.add_card_to_hand(0, catalog::grizzly_bears());
    let inv = g.add_card_to_hand(0, catalog::invasion_of_ergamon());
    g.decider = Box::new(ScriptedDecider::new([A::Bool(true), A::Discard(vec![first])]));
    cast(&mut g, inv, &[]).expect("invasion");
    assert_eq!(tokens_named(&g, "Treasure"), 1);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == first), "discarded");
    assert_eq!(g.players[0].hand.len(), 1, "drew one");
    let second = g.players[0].hand[0].id;
    g.decider = Box::new(ScriptedDecider::new([A::Bool(true), A::Discard(vec![second]), A::Search(Some(forest))]));
    g.battlefield_find_mut(inv).unwrap().remove_counters(CounterType::Defense, 5);
    g.check_state_based_actions();
    drain_stack(&mut g);
    let charger = g.battlefield.iter().find(|c| c.definition.name == "Truga Cliffcharger").map(|c| c.id).expect("back face");
    assert_eq!(pt(&g, charger), (3, 4));
    assert!(g.players[0].hand.iter().any(|c| c.id == forest), "fetched the Forest");
}

/// The Tarrasque cast from the command zone "was cast" (CR 601.2): it has
/// haste; one put onto the battlefield by an effect doesn't.
#[test]
fn the_tarrasque_was_cast_from_the_command_zone() {
    let mut g = pod(2);
    let cmd = g.seat_commanders(0, vec![catalog::the_tarrasque()])[0];
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("commander");
    drain_stack(&mut g);
    assert!(g.computed_permanent(cmd).unwrap().keywords().contains(&Keyword::Haste));
    let put = g.add_card_to_battlefield(1, catalog::the_tarrasque());
    assert!(!g.computed_permanent(put).unwrap().keywords().contains(&Keyword::Haste));
}

/// Rulik Mons: a land on top "may" go onto the battlefield tapped; declining
/// it makes the Goblin instead.
#[test]
fn rulik_mons_may_put_the_land() {
    for take in [true, false] {
        let mut g = pod(2);
        let rulik = ready(&mut g, 0, catalog::rulik_mons_warren_chief());
        let forest = g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Bool(take),
        ]));
        declare(&mut g, &[(rulik, 1)]).expect("attack");
        assert_eq!(g.battlefield_find(forest).is_some(), take, "land: {take}");
        assert_eq!(tokens_named(&g, "Goblin"), usize::from(!take), "goblin: {take}");
    }
}

/// Invoke Justice's counters go among the creatures of the *target player*
/// (CR 115.1, its second target); the auto-targeter names you for it.
#[test]
fn invoke_justice_distributes_among_the_target_players_creatures() {
    let mut g = pod(3);
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 2, catalog::grizzly_bears());
    let ij = g.add_card_to_hand(0, catalog::invoke_justice());
    let (first, rest) = g.auto_targets_for_effect_all_slots(&catalog::invoke_justice().effect, 0, None);
    assert_eq!((first, rest), (Some(perm(dead)), vec![Target::Player(0)]), "the friendly pick");
    cast(&mut g, ij, &[perm(dead), Target::Player(2)]).expect("invoke justice");
    let n = |g: &GameState, id| g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!((n(&g, theirs), n(&g, mine), n(&g, dead)), (4, 0, 0));
}

// ── CR 603.12 "When you do" payoffs (reflexive triggers) ───────────────────

/// Doombot Harbinger: "you may exile this card. When you do, return target
/// creature card" — the return targets after the exile, so the Harbinger can
/// never fetch itself; declining the exile returns nothing.
#[test]
fn doombot_harbinger_exiles_itself_then_returns_a_creature_card() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    for exile in [true, false] {
        let mut g = pod(2);
        let bot = ready(&mut g, 0, catalog::doombot_harbinger());
        let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(exile)]));
        let murder = g.add_card_to_hand(1, catalog::murder());
        cast_as(&mut g, 1, murder, &[Target::Permanent(bot)]).expect("murder");
        assert_eq!(g.exile.iter().any(|c| c.id == bot), exile);
        assert_eq!(g.players[0].hand.iter().any(|c| c.id == bears), exile);
    }
}

/// Ancient Bronze Dragon: the d20 is rolled first, then the reflexive trigger
/// puts that many +1/+1 counters on up to two target creatures.
#[test]
fn ancient_bronze_dragon_puts_the_roll_on_up_to_two_creatures() {
    let mut g = pod(2);
    let dragon = ready(&mut g, 0, catalog::ancient_bronze_dragon());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    declare(&mut g, &[(dragon, 1)]).expect("attack");
    combat_damage(&mut g);
    assert_eq!(g.players[1].life, 13);
    let n = g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert!((1..=20).contains(&n), "the roll: {n}");
    assert_eq!(g.battlefield_find(dragon).unwrap().counter_count(CounterType::PlusOnePlusOne), n);
}
