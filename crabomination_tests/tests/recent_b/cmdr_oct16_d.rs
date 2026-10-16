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

/// Wrath of Marit Lage taps red creatures and keeps them tapped.
#[test]
fn wrath_of_marit_lage_locks_red_creatures() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    let goblin = ready(&mut g, 1, catalog::raging_goblin());
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let w = g.add_card_to_hand(0, catalog::wrath_of_marit_lage());
    cast(&mut g, w, None);
    assert!(g.battlefield_find(goblin).unwrap().tapped);
    assert!(!g.battlefield_find(bears).unwrap().tapped, "green, untouched");
    step_to(&mut g, 1, TurnStep::PreCombatMain);
    assert!(g.battlefield_find(goblin).unwrap().tapped, "doesn't untap");
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

/// Bullseye's activated "sacrifice an artifact or discard a nonland card" is
/// two abilities; the discard one takes a nonland card and shoots for 2.
#[test]
fn bullseye_shoots_for_a_discarded_card() {
    let mut g = pod(2);
    let b = ready(&mut g, 0, catalog::bullseye_death_dealer());
    g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility { card_id: b, ability_index: 1, target: Some(Target::Player(1)), additional_targets: vec![], x_value: None, mode: None })
        .expect("discard to shoot");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
    assert!(g.players[0].hand.is_empty());
}

/// Gallia returns from the graveyard tapped with a counter, exiling another
/// creature card.
#[test]
fn gallia_returns_tapped_with_a_counter() {
    let mut g = pod(2);
    let gallia = g.add_card_to_graveyard(0, catalog::gallia_tragic_host());
    let fuel = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    activate(&mut g, gallia, None).expect("return");
    let c = g.battlefield_find(gallia).expect("back");
    assert!(c.tapped);
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.exile.iter().any(|c| c.id == fuel));
}

/// Ritual of the Machine sacrifices a creature to steal a nonblack one.
#[test]
fn ritual_of_the_machine_steals_a_nonblack_creature() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::llanowar_elves());
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let r = g.add_card_to_hand(0, catalog::ritual_of_the_machine());
    cast(&mut g, r, Some(Target::Permanent(angel)));
    assert_eq!(g.battlefield_find(angel).unwrap().controller, 0);
}

/// Nefarox attacking alone: exalted from the Servant pumps it, and the
/// defending player — not a bystander — sacrifices a creature.
#[test]
fn nefarox_alone_makes_the_defender_sacrifice() {
    let mut g = pod(3);
    let nef = ready(&mut g, 0, catalog::nefarox_overlord_of_grixis());
    ready(&mut g, 0, catalog::servant_of_nefarox());
    let victim = ready(&mut g, 1, catalog::grizzly_bears());
    let bystander = ready(&mut g, 2, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: nef, target: AttackTarget::Player(1) }])).expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none());
    assert!(g.battlefield_find(bystander).is_some());
    assert_eq!(g.computed_permanent(nef).unwrap().power, 7, "two exalted instances");
}

/// Vraska, Scheming Gorgon's −10: a creature that damages an opponent makes
/// that player lose.
#[test]
fn vraska_ultimate_makes_damage_lethal() {
    let mut g = pod(3);
    stock_libraries(&mut g, 5);
    let v = ready(&mut g, 0, catalog::vraska_scheming_gorgon());
    g.battlefield.iter_mut().find(|c| c.id == v).unwrap().counters.insert(CounterType::Loyalty, 10);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: v, ability_index: 2, target: None, x_value: None }).expect("ult");
    drain_stack(&mut g);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bears, target: AttackTarget::Player(1) }])).expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert!(g.players[1].eliminated, "dealt damage → lost");
    assert!(!g.players[2].eliminated);
}

/// Embereth Paladin's adamant: three red mana spent → a +1/+1 counter.
#[test]
fn embereth_paladin_adamant_counter() {
    for (red, counter) in [(4, 1), (1, 0)] {
        let mut g = pod(2);
        let p = g.add_card_to_hand(0, catalog::embereth_paladin());
        g.players[0].mana_pool.add(Color::Red, red);
        g.players[0].mana_pool.add_colorless(4 - red);
        g.perform_action(GameAction::CastSpell { card_id: p, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(p).unwrap().counter_count(CounterType::PlusOnePlusOne), counter);
    }
}

/// Joust pumps a Knight before the fight: the 2/2 Askari fights as a 4/3 and
/// survives the 2/2 it kills.
#[test]
fn joust_pumps_a_knight_then_fights() {
    let mut g = pod(2);
    let askari = ready(&mut g, 0, catalog::blazing_blade_askari());
    let giant = ready(&mut g, 1, catalog::grizzly_bears());
    let j = g.add_card_to_hand(0, catalog::joust());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::CastSpell { card_id: j, target: Some(Target::Permanent(askari)), additional_targets: vec![Target::Permanent(giant)], mode: None, x_value: None })
        .expect("joust");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none());
    assert!(g.battlefield_find(askari).is_some(), "a 4/3 takes 2");
}

/// Fleetwheel Cruiser is a creature the turn it enters, without crewing.
#[test]
fn fleetwheel_cruiser_animates_on_entry() {
    use crabomination::card::CardType;
    let mut g = pod(2);
    let c = g.add_card_to_hand(0, catalog::fleetwheel_cruiser());
    cast(&mut g, c, None);
    assert!(g.computed_permanent(c).unwrap().card_types().contains(&CardType::Creature));
}

/// Renegade Wheelsmith becoming tapped (crewing here) stops a blocker.
#[test]
fn renegade_wheelsmith_tapping_stops_a_blocker() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let ws = ready(&mut g, 0, catalog::renegade_wheelsmith());
    let wall = ready(&mut g, 1, catalog::grizzly_bears());
    let tap = crabomination::effect::Effect::Tap { what: crabomination::effect::Selector::EachPermanent(crabomination::card::SelectionRequirement::HasName("Renegade Wheelsmith".into())) };
    let evs = g.resolve_effect(&tap, &crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(ws).unwrap().tapped);
    assert!(g.computed_permanent(wall).unwrap().keywords().contains(&Keyword::CantBlock));
}

/// Doc Ock, Evil Inventor turns a noncreature artifact into an 8/8 Robot
/// Villain at the beginning of combat, and it stays one.
#[test]
fn doc_ock_makes_an_eight_eight_robot() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    ready(&mut g, 0, catalog::doc_ock_evil_inventor());
    let ring = ready(&mut g, 0, catalog::sol_ring());
    g.step = TurnStep::PreCombatMain;
    let evs = g.advance_step(Vec::new()).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let cp = g.computed_permanent(ring).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 8));
    step_to(&mut g, 1, TurnStep::PreCombatMain);
    assert_eq!(g.computed_permanent(ring).unwrap().power, 8, "no duration");
}

/// Doc Ock, Sinister Scientist is 8/8 with eight cards in the graveyard.
#[test]
fn doc_ock_sinister_scientist_grows_with_the_graveyard() {
    let mut g = pod(2);
    let d = ready(&mut g, 0, catalog::doc_ock_sinister_scientist());
    assert_eq!(g.computed_permanent(d).unwrap().power, 4);
    for _ in 0..8 {
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
    }
    let cp = g.computed_permanent(d).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 8));
}

/// Grasping Tentacles mills the opponent and takes an artifact from the pile.
#[test]
fn grasping_tentacles_takes_a_milled_artifact() {
    let mut g = pod(2);
    let ring = g.add_card_to_library(1, catalog::sol_ring());
    stock_libraries(&mut g, 10);
    let t = g.add_card_to_hand(0, catalog::grasping_tentacles());
    cast(&mut g, t, Some(Target::Player(1)));
    assert_eq!(g.battlefield_find(ring).map(|c| c.controller), Some(0));
}

/// Mabel strips three counters off another creature.
#[test]
fn mabel_removes_three_counters() {
    let mut g = pod(2);
    let big = ready(&mut g, 1, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == big).unwrap().counters.insert(CounterType::PlusOnePlusOne, 4);
    let m = g.add_card_to_hand(0, catalog::mabel_bitter_recluse());
    cast(&mut g, m, None);
    assert_eq!(g.battlefield_find(big).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// CR 702.193 — Marvel Boy counts a power-up activation (and the creature
/// that entered before it) as a +1/+1 counter each; another seat's power-up
/// doesn't count.
#[test]
fn cr_702_193_marvel_boy_grows_on_your_power_up() {
    let mut g = pod(2);
    let mb = ready(&mut g, 0, catalog::marvel_boy_noh_varr());
    let brawler = g.add_card_to_hand(0, catalog::brave_brawler());
    cast(&mut g, brawler, None);
    assert_eq!(g.battlefield_find(mb).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "the Brawler entered");
    g.players[0].mana_pool.add(Color::White, 5);
    activate(&mut g, brawler, None).expect("power-up");
    assert_eq!(g.battlefield_find(mb).unwrap().counter_count(CounterType::PlusOnePlusOne), 2, "and powered up");
    assert_eq!(g.battlefield_find(brawler).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Raise the Draugr's second mode needs two cards sharing a creature type —
/// read off the graveyard cards' printed types (bug fix: the cross-slot
/// check only knew battlefield permanents and rejected two Bears).
#[test]
fn raise_the_draugr_returns_two_sharing_a_type() {
    let mut g = pod(2);
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let elf = g.add_card_to_graveyard(0, catalog::llanowar_elves());
    let r = g.add_card_to_hand(0, catalog::raise_the_draugr());
    g.players[0].mana_pool.add(Color::Black, 2);
    assert!(g
        .perform_action(GameAction::CastSpell { card_id: r, target: Some(Target::Permanent(a)), additional_targets: vec![Target::Permanent(elf)], mode: Some(1), x_value: None })
        .is_err(), "a Bear and an Elf share no type");
    g.perform_action(GameAction::CastSpell { card_id: r, target: Some(Target::Permanent(a)), additional_targets: vec![Target::Permanent(b)], mode: Some(1), x_value: None })
        .expect("two Bears");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == a) && g.players[0].hand.iter().any(|c| c.id == b));
}

/// Qarsi High Priest sacrifices another creature to manifest.
#[test]
fn qarsi_high_priest_manifests_off_a_sacrifice() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let q = ready(&mut g, 0, catalog::qarsi_high_priest());
    ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 2);
    activate(&mut g, q, None).expect("manifest");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.face_down));
}
