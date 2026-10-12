//! Commander: the two-card EDHREC gaps of commanders ranked 1001-1500
//! (`decks::cmdr_edhrec15`).

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

