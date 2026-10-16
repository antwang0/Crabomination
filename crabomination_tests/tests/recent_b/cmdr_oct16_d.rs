//! Commander: the 2026-10-16 rescan's short-text gaps (`decks::cmdr_oct16_d`).

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

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn activate(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: 0, target, additional_targets: vec![], x_value: None, mode: None })?;
    drain_stack(g);
    Ok(())
}

/// CR 603.8 — Skeleton Ship's "when you control no Islands" is a state
/// trigger: it goes the moment the last Island leaves, not at upkeep.
#[test]
fn skeleton_ship_dies_with_its_last_island() {
    let mut g = pod(2);
    let island = g.add_card_to_battlefield(0, catalog::island());
    let ship = ready(&mut g, 0, catalog::skeleton_ship());
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    activate(&mut g, ship, Some(Target::Permanent(bears))).expect("ping");
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::MinusOneMinusOne), 1);
    let destroy = crabomination::effect::Effect::Destroy {
        what: crabomination::effect::Selector::EachPermanent(crabomination::card::SelectionRequirement::Land),
    };
    let ctx = crabomination::game::effects::EffectContext::for_spell(1, None, 0, 0);
    let evs = g.resolve_effect(&destroy, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(island).is_none());
    assert!(g.battlefield_find(ship).is_none());
}

/// Apprentice Sorcerer pings only on its controller's turn before attackers;
/// Zuran Spellcaster any time.
#[test]
fn apprentice_sorcerer_pings_only_before_your_attack() {
    let mut g = pod(2);
    let app = ready(&mut g, 0, catalog::apprentice_sorcerer());
    let zur = ready(&mut g, 0, catalog::zuran_spellcaster());
    g.step = TurnStep::DeclareBlockers;
    assert!(activate(&mut g, app, Some(Target::Player(1))).is_err(), "after attackers");
    activate(&mut g, zur, Some(Target::Player(1))).expect("Zuran any time");
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, app, Some(Target::Player(1))).expect("main phase, your turn");
    assert_eq!(g.players[1].life, 18);
}

/// Soul Barrier: an opponent who won't (or can't) pay {2} takes 2.
#[test]
fn soul_barrier_taxes_opposing_creature_spells() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::soul_barrier());
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell { card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
}

/// Tranquilize taps and stuns for three untaps.
#[test]
fn tranquilize_taps_and_stuns() {
    let mut g = pod(2);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let t = g.add_card_to_hand(0, catalog::tranquilize());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell { card_id: t, target: Some(Target::Permanent(angel)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    let a = g.battlefield_find(angel).unwrap();
    assert!(a.tapped);
    assert_eq!(a.counter_count(CounterType::Stun), 3);
}

/// Wargling's ferocious attack gives the team trample only beside a 4-power
/// creature.
#[test]
fn wargling_tramples_the_team_when_ferocious() {
    for (big, trample) in [(true, true), (false, false)] {
        let mut g = pod(2);
        let w = ready(&mut g, 0, catalog::wargling());
        let other = ready(&mut g, 0, if big { catalog::craw_wurm() } else { catalog::grizzly_bears() });
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: w, target: AttackTarget::Player(1) }])).expect("attack");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(other).unwrap().keywords().contains(&Keyword::Trample), trample);
    }
}

/// Phantasmagorian: an opponent may discard three to counter it; from the
/// graveyard, discarding three returns it to hand.
#[test]
fn phantasmagorian_can_be_countered_by_a_discard_and_recurs() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::lightning_bolt());
    }
    let p = g.add_card_to_hand(0, catalog::phantasmagorian());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false), DecisionAnswer::Bool(true)]));
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell { card_id: p, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == p), "countered");
    assert!(g.players[1].hand.is_empty(), "the opponent paid three cards");
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::lightning_bolt());
    }
    activate(&mut g, p, None).expect("discard three");
    assert!(g.players[0].hand.iter().any(|c| c.id == p));
}

/// Soul Spike's alternative cost exiles two black cards from hand.
#[test]
fn soul_spike_pitches_two_black_cards() {
    let mut g = pod(2);
    let spike = g.add_card_to_hand(0, catalog::soul_spike());
    let a = g.add_card_to_hand(0, catalog::dark_ritual());
    let b = g.add_card_to_hand(0, catalog::dark_ritual());
    g.perform_action(GameAction::CastSpellAlternative { card_id: spike, pitch_card: Some(a), target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None })
        .expect("pitch cast");
    drain_stack(&mut g);
    assert_eq!((g.players[1].life, g.players[0].life), (16, 24));
    assert!(g.exile.iter().any(|c| c.id == a) && g.exile.iter().any(|c| c.id == b));
}

/// Ebon Stronghold sacrifices for {B}{B}.
#[test]
fn ebon_stronghold_sacrifices_for_two_black() {
    let mut g = pod(2);
    let land = ready(&mut g, 0, catalog::ebon_stronghold());
    g.perform_action(GameAction::ActivateAbility { card_id: land, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("sac");
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 2);
    assert!(g.battlefield_find(land).is_none());
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 10);
    }
    g.players[0].mana_pool.add_colorless(10);
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(g);
}

/// Valor of the Worthy leaves a Spirit behind when the enchanted creature
/// leaves; Squire's Devotion's bonus is on the host, its Vampire on the board.
#[test]
fn valor_of_the_worthy_leaves_a_spirit() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let valor = g.add_card_to_hand(0, catalog::valor_of_the_worthy());
    cast(&mut g, valor, Some(Target::Permanent(bears)));
    assert_eq!(g.computed_permanent(bears).unwrap().power, 3);
    let kill = crabomination::effect::Effect::Destroy {
        what: crabomination::effect::Selector::EachPermanent(crabomination::card::SelectionRequirement::Creature),
    };
    let evs = g.resolve_effect(&kill, &crabomination::game::effects::EffectContext::for_spell(1, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Spirit"));
}

/// Nurturing Presence's granted trigger pumps the host when another creature
/// of yours enters (its own Spirit token included).
#[test]
fn nurturing_presence_pumps_on_each_entry() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let np = g.add_card_to_hand(0, catalog::nurturing_presence());
    cast(&mut g, np, Some(Target::Permanent(bears)));
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    cast(&mut g, elf, None);
    assert!(g.computed_permanent(bears).unwrap().power >= 3, "the elf's entry pumped it");
}

/// Palazzo Archers shoot a flier attacking their controller.
#[test]
fn palazzo_archers_shoot_an_attacking_flier() {
    let mut g = pod(2);
    let archers = ready(&mut g, 1, catalog::palazzo_archers());
    let bird = ready(&mut g, 0, catalog::birds_of_paradise());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bird, target: AttackTarget::Player(1) }])).expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bird).is_none());
    assert!(g.battlefield_find(archers).is_some());
}

/// Arbalest Elite shoots a blocker for 3 and then skips its next untap.
#[test]
fn arbalest_elite_shoots_and_stays_tapped() {
    let mut g = pod(2);
    let elite = ready(&mut g, 1, catalog::arbalest_elite());
    let wurm = ready(&mut g, 0, catalog::hill_giant());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: wurm, target: AttackTarget::Player(1) }])).expect("attack");
    drain_stack(&mut g);
    g.players[1].mana_pool.add(Color::White, 3);
    g.priority.player_with_priority = 1;
    activate(&mut g, elite, Some(Target::Permanent(wurm))).expect("shoot");
    assert!(g.battlefield_find(wurm).is_none(), "3 to the 3/3");
    assert!(g.battlefield_find(elite).unwrap().skip_next_untap);
}

/// Arni gets +X/+0 from an entering creature's power.
#[test]
fn arni_grows_with_each_entry() {
    let mut g = pod(2);
    let arni = ready(&mut g, 0, catalog::arni_renowned_champion());
    let wurm = g.add_card_to_hand(0, catalog::craw_wurm());
    cast(&mut g, wurm, None);
    assert_eq!(g.computed_permanent(arni).unwrap().power, 7);
}

/// Sima Yi's power counts Swamps; its toughness stays 4.
#[test]
fn sima_yi_counts_swamps_for_power_only() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::swamp());
    }
    let s = ready(&mut g, 0, catalog::sima_yi_wei_field_marshal());
    let cp = g.computed_permanent(s).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4));
}

/// Intimidator Initiate: a red spell lets you pay {1} so a creature can't block.
#[test]
fn intimidator_initiate_stops_a_blocker_on_a_red_spell() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::intimidator_initiate());
    let wall = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert!(g.computed_permanent(wall).unwrap().keywords().contains(&Keyword::CantBlock));
}
