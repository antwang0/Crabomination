//! Commander: the 2026-10-21 (`01Jywzky`) pair-deck closers
//! (`decks::cmdr_01jy_a`) and the primitives they need.

use crabomination::card::{CardId, Keyword};
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
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn cast_as(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), GameError> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn attack_with(g: &mut GameState, attackers: &[CardId]) {
    for &a in attackers {
        g.clear_sickness(a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

/// Through the next turn's upkeep, resolving whatever triggers there.
fn to_next_upkeep(g: &mut GameState) {
    let turn = g.turn_number;
    while !(g.step == TurnStep::Upkeep && g.turn_number > turn) {
        let _ = g.advance_step(Vec::new());
    }
    drain_stack(g);
}

/// Burnout may target any instant spell and counters it only if it's blue;
/// the delayed draw waits for the next turn's upkeep either way (CR 603.7).
#[test]
fn burnout_counters_only_a_blue_instant_and_draws_next_upkeep() {
    for (blue, spell) in [(true, catalog::opt as fn() -> _), (false, catalog::lightning_bolt)] {
        let mut g = pod(2);
        let instant = g.add_card_to_hand(1, spell());
        cast_as(&mut g, 1, instant, (!blue).then_some(Target::Player(0)));
        let burnout = g.add_card_to_hand(0, catalog::burnout());
        cast_as(&mut g, 0, burnout, Some(Target::Permanent(instant)));
        let life = g.players[0].life;
        drain_stack(&mut g);
        assert!(g.players[1].graveyard.iter().any(|c| c.id == instant));
        assert_eq!(g.players[0].life == life, blue, "only the blue instant is countered");
        let hand = g.players[0].hand.len();
        to_next_upkeep(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "the draw lands at the next turn's upkeep");
    }
}

/// Mana Short taps every land the target player controls and empties their
/// pool — even mana Upwelling would keep past a step's end (CR 500.4 governs
/// only the step-end empty, not an effect).
#[test]
fn mana_short_taps_lands_and_empties_the_pool() {
    let mut g = pod(3);
    let land = g.add_card_to_battlefield(1, catalog::island());
    let other = g.add_card_to_battlefield(2, catalog::island());
    g.add_card_to_battlefield(1, catalog::upwelling());
    g.players[1].mana_pool.add(Color::Green, 4);
    let short = g.add_card_to_hand(0, catalog::mana_short());
    cast_as(&mut g, 0, short, Some(Target::Player(1)));
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).unwrap().tapped);
    assert!(!g.battlefield_find(other).unwrap().tapped, "only the target player's lands");
    assert_eq!(g.players[1].mana_pool.total(), 0);
}

/// Energy Storm prevents a spell's damage to every recipient (CR 615) but not
/// a permanent's, and a flier stays tapped through its untap step (CR 502.3).
#[test]
fn energy_storm_stops_spell_damage_and_untapping_fliers() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::energy_storm());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let life = g.players[1].life;
    cast_as(&mut g, 0, bolt, Some(Target::Player(1)));
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life, "the instant's damage is prevented");
    let flier = g.add_card_to_battlefield(0, catalog::serra_angel());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [flier, bears] {
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    g.active_player_idx = 1;
    to_next_upkeep(&mut g);
    assert_eq!(g.active_player_idx, 0);
    assert!(g.battlefield_find(flier).unwrap().tapped, "the flier doesn't untap");
    assert!(!g.battlefield_find(bears).unwrap().tapped);
}

/// Ambitious Farmhand transforms only with coven — three creatures of
/// different powers — and its back is a 3/3 lifelinker.
#[test]
fn ambitious_farmhand_transforms_only_with_coven() {
    let mut g = pod(2);
    let farmhand = g.add_card_to_battlefield(0, catalog::ambitious_farmhand());
    assert!(activate(&mut g, farmhand, 0, None).is_err(), "no coven yet");
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::hill_giant());
    activate(&mut g, farmhand, 0, None).expect("coven: powers 1, 2, 3");
    assert_eq!(pt(&g, farmhand), (3, 3));
    assert!(g.computed_permanent(farmhand).unwrap().keywords().contains(&Keyword::Lifelink));
}

/// Pit Trap destroys an attacker without flying, never a flier or a creature
/// that isn't attacking.
#[test]
fn pit_trap_takes_only_a_grounded_attacker() {
    let mut g = pod(2);
    let trap = g.add_card_to_battlefield(1, catalog::pit_trap());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    attack_with(&mut g, &[bears, angel]);
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    let aim = |g: &mut GameState, t: CardId| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: trap,
            ability_index: 0,
            target: Some(Target::Permanent(t)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(aim(&mut g, angel).is_err(), "a flier is out of reach");
    aim(&mut g, bears).expect("a grounded attacker");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none());
    assert!(g.battlefield_find(angel).is_some());
}

/// Covenant of Minds: the target opponent picks — the three revealed cards to
/// your hand, or into your graveyard while you draw five.
#[test]
fn covenant_of_minds_gives_three_or_five() {
    let mut g = pod(2);
    let covenant = g.add_card_to_hand(0, catalog::covenant_of_minds());
    let hand = g.players[0].hand.len();
    let library = g.players[0].library.len();
    cast_as(&mut g, 0, covenant, Some(Target::Player(1)));
    drain_stack(&mut g);
    let drew = g.players[0].hand.len() + 1 - hand;
    let binned = g.players[0].graveyard.len() - 1;
    assert!((drew, binned) == (3, 0) || (drew, binned) == (5, 3), "drew {drew}, binned {binned}");
    assert_eq!(library - g.players[0].library.len(), drew + binned);
}

/// Impetuous Protege attacks for the greatest power among its opponents'
/// TAPPED creatures; an untapped bigger one doesn't count.
#[test]
fn impetuous_protege_reads_tapped_opposing_power() {
    let mut g = pod(2);
    let protege = g.add_card_to_battlefield(0, catalog::impetuous_protege());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.add_card_to_battlefield(1, catalog::serra_angel());
    g.battlefield_find_mut(giant).unwrap().tapped = true;
    attack_with(&mut g, &[protege]);
    assert_eq!(pt(&g, protege), (3, 4));
}

/// Inscribed Tablet takes a land from the top five; with none there it draws.
#[test]
fn inscribed_tablet_finds_a_land_or_draws() {
    for land in [true, false] {
        let mut g = multi_player_game(2);
        if land {
            g.add_card_to_library(0, catalog::forest());
        }
        for _ in 0..8 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let tablet = g.add_card_to_battlefield(0, catalog::inscribed_tablet());
        let hand = g.players[0].hand.len();
        activate(&mut g, tablet, 0, None).expect("activate");
        assert_eq!(g.players[0].hand.len(), hand + 1);
        let got_forest = g.players[0].hand.iter().any(|c| c.definition.name == "Forest");
        assert_eq!(got_forest, land);
    }
}

/// Bloodhaze Wolverine's trigger is the second draw of the turn, once.
#[test]
fn bloodhaze_wolverine_wakes_on_the_second_draw() {
    let mut g = pod(2);
    let wolverine = g.add_card_to_battlefield(0, catalog::bloodhaze_wolverine());
    let one = g.add_card_to_hand(0, catalog::opt());
    cast_as(&mut g, 0, one, None);
    drain_stack(&mut g);
    assert_eq!(pt(&g, wolverine), (2, 1), "one card drawn");
    let two = g.add_card_to_hand(0, catalog::divination());
    cast_as(&mut g, 0, two, None);
    drain_stack(&mut g);
    assert_eq!(pt(&g, wolverine), (3, 2), "the second and third draws trigger once");
    assert!(g.computed_permanent(wolverine).unwrap().keywords().contains(&Keyword::FirstStrike));
}

/// Weathered Runestone: a nonland permanent card can't be reanimated and
/// nothing is cast from a graveyard; a land card still enters from one.
#[test]
fn weathered_runestone_locks_graveyards_for_nonland_permanents() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::weathered_runestone());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let land = g.add_card_to_graveyard(0, catalog::forest());
    let back = crabomination::effect::Effect::Move {
        what: crabomination::effect::Selector::Target(0),
        to: crabomination::effect::ZoneDest::Battlefield {
            controller: crabomination::effect::PlayerRef::You,
            tapped: false,
        },
    };
    for what in [bears, land] {
        let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(what)), 0, 0);
        g.resolve_effect(&back, &ctx).unwrap();
    }
    assert!(g.battlefield_find(bears).is_none(), "a creature card stays in the graveyard");
    assert!(g.battlefield_find(land).is_some(), "a land card isn't covered");
    let current = g.add_card_to_graveyard(0, catalog::direct_current());
    flood(&mut g, 0);
    g.add_card_to_hand(0, catalog::island());
    let jump = g.perform_action(GameAction::CastSpell {
        card_id: current,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(jump.is_err(), "no jump-start from the graveyard");
}

/// Knight of Cliffhaven levels: 2/2, then 2/3 flier at 1, then 4/4 flying
/// vigilance at 4 (CR 702.87 / 711).
#[test]
fn knight_of_cliffhaven_levels_up() {
    let mut g = pod(2);
    let knight = g.add_card_to_battlefield(0, catalog::knight_of_cliffhaven());
    assert_eq!(pt(&g, knight), (2, 2));
    activate(&mut g, knight, 0, None).unwrap();
    assert_eq!(pt(&g, knight), (2, 3));
    for _ in 0..3 {
        activate(&mut g, knight, 0, None).unwrap();
    }
    assert_eq!(pt(&g, knight), (4, 4));
    let knight = g.computed_permanent(knight).unwrap();
    let kws = knight.keywords();
    assert!(kws.contains(&Keyword::Flying) && kws.contains(&Keyword::Vigilance));
}

/// Flower // Flourish: the left half fetches a basic Forest or Plains, the
/// right half pumps the team (CR 709.3).
#[test]
fn flower_flourish_halves() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::plains());
    let flower = g.add_card_to_hand(0, catalog::flower_flourish());
    cast_as(&mut g, 0, flower, None);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Plains"));
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let flourish = g.add_card_to_hand(0, catalog::flower_flourish());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSplitRight {
        card_id: flourish,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Flourish");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bears), (4, 4));
}

/// Tenacity untaps the creatures it pumps.
#[test]
fn tenacity_pumps_and_untaps() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().tapped = true;
    let tenacity = g.add_card_to_hand(0, catalog::tenacity());
    cast_as(&mut g, 0, tenacity, None);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bears).unwrap().tapped);
    assert_eq!(pt(&g, bears), (3, 3));
    assert!(g.computed_permanent(bears).unwrap().keywords().contains(&Keyword::Lifelink));
}

/// Dancing Sword: once its creature dies it may stand up as a 2/1 flying
/// Construct, and it is no longer an Equipment (CR 301.5).
#[test]
fn dancing_sword_dances_when_its_creature_dies() {
    let mut g = pod(2);
    let sword = g.add_card_to_battlefield(0, catalog::dancing_sword());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: sword, target: bears }).expect("equip");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bears), (4, 3));
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.remove_to_graveyard_with_triggers(bears);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none());
    let sword = g.computed_permanent(sword).expect("the sword stays");
    assert_eq!((sword.power, sword.toughness), (2, 1));
    assert!(sword.keywords().contains(&Keyword::Flying));
    assert!(sword.subtypes().artifact_subtypes.is_empty(), "it isn't an Equipment");
}

/// Tromp the Domains pumps by the basic land types among your lands.
#[test]
fn tromp_the_domains_counts_basic_types() {
    let mut g = pod(2);
    for land in [catalog::forest(), catalog::island(), catalog::island()] {
        g.add_card_to_battlefield(0, land);
    }
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let tromp = g.add_card_to_hand(0, catalog::tromp_the_domains());
    cast_as(&mut g, 0, tromp, None);
    drain_stack(&mut g);
    assert_eq!(pt(&g, bears), (4, 4));
    assert!(g.computed_permanent(bears).unwrap().keywords().contains(&Keyword::Trample));
}
