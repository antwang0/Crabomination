//! Judgment (JUD) — the block-closing set: Threshold payoffs, the Dwarf
//! tribe and the white Nomad/Cleric shell.

use crabomination::card::{CardId, Keyword, LandType, Zone};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    cast_x(g, seat, id, target, None);
}

fn cast_x(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>, x: Option<u32>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: x,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, seat: usize, card_id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn fill_graveyard(g: &mut GameState, seat: usize) {
    for _ in 0..7 {
        g.add_card_to_graveyard(seat, catalog::forest());
    }
}

/// Send `attacker` in unblocked and run the combat-damage step.
fn combat_damage_to_player(g: &mut GameState, attacker: CardId, defender: usize) {
    g.clear_sickness(attacker);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(defender),
    }]))
    .expect("attack");
    drain_stack(g);
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    drain_stack(g);
}

/// Ancestor's Chosen banks a life per card in your graveyard.
#[test]
fn ancestors_chosen_gains_life_for_the_dead() {
    let mut g = main_phase();
    fill_graveyard(&mut g, 0);
    let spell = g.add_card_to_hand(0, catalog::ancestors_chosen());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.players[0].life, 27);
}

/// Battlewise Aven only gets its bonus past Threshold.
#[test]
fn battlewise_aven_hardens_at_threshold() {
    let mut g = main_phase();
    let aven = g.add_card_to_battlefield(0, catalog::battlewise_aven());
    assert_eq!(g.computed_permanent(aven).unwrap().power, 2);
    fill_graveyard(&mut g, 0);
    let cp = g.computed_permanent(aven).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
}

/// Benevolent Bodyguard trades itself for a colour of protection.
#[test]
fn benevolent_bodyguard_buys_protection() {
    let mut g = main_phase();
    let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let guard = g.add_card_to_battlefield(0, catalog::benevolent_bodyguard());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    activate(&mut g, 0, guard, 0, Some(Target::Permanent(friend)));
    assert!(
        g.computed_permanent(friend).unwrap().keywords.contains(&Keyword::Protection(Color::Red))
    );
}

/// Chastise kills an attacker and banks its power as life.
#[test]
fn chastise_kills_an_attacker_for_life() {
    let mut g = main_phase();
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(attacker).unwrap().summoning_sick = false;
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    let spell = g.add_card_to_hand(0, catalog::chastise());
    cast(&mut g, 0, spell, Some(Target::Permanent(attacker)));
    assert!(g.battlefield_find(attacker).is_none());
    assert_eq!(g.players[0].life, 22);
}

/// Aven Fogbringer bounces a land on the way in.
#[test]
fn aven_fogbringer_bounces_a_land() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let bird = g.add_card_to_hand(0, catalog::aven_fogbringer());
    cast(&mut g, 0, bird, Some(Target::Permanent(land)));
    assert!(g.battlefield_find(land).is_none());
    assert!(g.players[1].hand.iter().any(|c| c.id == land));
}

/// Envelop counters a sorcery and nothing else.
#[test]
fn envelop_only_counters_sorceries() {
    let mut g = main_phase();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Player(0)));
    let sorcery = g.add_card_to_hand(1, catalog::crush_of_wurms());
    mana(&mut g, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: sorcery,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast wurms");
    let envelop = g.add_card_to_hand(0, catalog::envelop());
    cast(&mut g, 0, envelop, Some(Target::Permanent(sorcery)));
    assert!(!g.battlefield.iter().any(|c| c.is_token), "the Wurms never landed");
}

/// Keep Watch draws one per attacker.
#[test]
fn keep_watch_draws_per_attacker() {
    let mut g = main_phase();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for id in [a, b] {
        g.battlefield_find_mut(id).unwrap().summoning_sick = false;
    }
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(0) },
        Attack { attacker: b, target: AttackTarget::Player(0) },
    ]))
    .expect("attack");
    let spell = g.add_card_to_hand(0, catalog::keep_watch());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.players[0].hand.len(), 2);
}

/// Guiltfeeder drains for the defender's graveyard when it gets through.
#[test]
fn guiltfeeder_drains_for_the_graveyard() {
    let mut g = main_phase();
    fill_graveyard(&mut g, 1);
    let feeder = g.add_card_to_battlefield(0, catalog::guiltfeeder());
    g.battlefield_find_mut(feeder).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: feeder,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 13);
}

/// Earsplitting Rats strips a card from each hand.
#[test]
fn earsplitting_rats_strips_each_hand() {
    let mut g = main_phase();
    g.add_card_to_hand(0, catalog::forest());
    g.add_card_to_hand(1, catalog::forest());
    let rats = g.add_card_to_hand(0, catalog::earsplitting_rats());
    cast(&mut g, 0, rats, None);
    assert_eq!(g.players[0].hand.len(), 0);
    assert_eq!(g.players[1].hand.len(), 0);
}

/// Arcane Teachings pumps its host and hands it a tap-to-ping.
#[test]
fn arcane_teachings_pumps_and_pings() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(host).unwrap().summoning_sick = false;
    let victim = g.add_card_to_battlefield(1, catalog::mystic_familiar());
    let aura = g.add_card_to_hand(0, catalog::arcane_teachings());
    cast(&mut g, 0, aura, Some(Target::Permanent(host)));
    assert_eq!(g.computed_permanent(host).unwrap().power, 4);
    activate(&mut g, 0, host, 0, Some(Target::Permanent(victim)));
    assert_eq!(g.battlefield_find(victim).unwrap().damage, 1);
}

/// Firecat Blitz mints X hasty Cats and takes them back at end of turn.
#[test]
fn firecat_blitz_mints_transient_cats() {
    let mut g = main_phase();
    let spell = g.add_card_to_hand(0, catalog::firecat_blitz());
    cast_x(&mut g, 0, spell, None, Some(3));
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 3);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 0, "exiled at end of turn");
}

/// Lightning Surge scales past Threshold.
#[test]
fn lightning_surge_scales_at_threshold() {
    let mut g = main_phase();
    fill_graveyard(&mut g, 0);
    let spell = g.add_card_to_hand(0, catalog::lightning_surge());
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 14);
}

/// Grizzly Fate doubles its Bears past Threshold.
#[test]
fn grizzly_fate_doubles_at_threshold() {
    let mut g = main_phase();
    fill_graveyard(&mut g, 0);
    let spell = g.add_card_to_hand(0, catalog::grizzly_fate());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 4);
}

/// Exoskeletal Armor is sized by every graveyard's dead.
#[test]
fn exoskeletal_armor_counts_all_graveyards() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::exoskeletal_armor());
    cast(&mut g, 0, aura, Some(Target::Permanent(host)));
    assert_eq!(g.computed_permanent(host).unwrap().power, 4, "2/2 plus two dead creatures");
}

/// Krosan Wayfarer trades itself for a land drop.
#[test]
fn krosan_wayfarer_deploys_a_land() {
    let mut g = main_phase();
    let land = g.add_card_to_hand(0, catalog::forest());
    let wayfarer = g.add_card_to_battlefield(0, catalog::krosan_wayfarer());
    activate(&mut g, 0, wayfarer, 0, None);
    assert!(g.battlefield_find(land).is_some());
}

/// Krosan Verge fetches a Forest and a Plains, both tapped.
#[test]
fn krosan_verge_fetches_a_pair() {
    let mut g = main_phase();
    let forest = g.add_card_to_library(0, catalog::forest());
    let plains = g.add_card_to_library(0, catalog::plains());
    let verge = g.add_card_to_battlefield(0, catalog::krosan_verge());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest)),
        DecisionAnswer::Search(Some(plains)),
    ]));
    activate(&mut g, 0, verge, 1, None);
    let fetched: Vec<LandType> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.is_land() && c.tapped)
        .flat_map(|c| c.definition.subtypes.land_types.clone())
        .collect();
    assert!(fetched.contains(&LandType::Forest) && fetched.contains(&LandType::Plains));
}

/// Anurid Brushhopper blinks itself out and comes back at end of turn.
#[test]
fn anurid_brushhopper_blinks_out_of_removal() {
    let mut g = main_phase();
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::forest());
    }
    let frog = g.add_card_to_battlefield(0, catalog::anurid_brushhopper());
    activate(&mut g, 0, frog, 0, None);
    assert!(g.battlefield_find(frog).is_none(), "exiled");
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Anurid Brushhopper"),
        "back at end of turn"
    );
}

/// Balthor the Defiled lords over Minions and mass-reanimates on the way out.
#[test]
fn balthor_lords_and_reanimates() {
    let mut g = main_phase();
    let balthor = g.add_card_to_battlefield(0, catalog::balthor_the_defiled());
    let minion = g.add_card_to_battlefield(0, catalog::cabal_trainee());
    assert_eq!(g.computed_permanent(minion).unwrap().power, 2, "Minion lord");
    let dead = g.add_card_to_graveyard(0, catalog::nantuko_shade());
    activate(&mut g, 0, balthor, 0, None);
    assert!(g.battlefield_find(dead).is_some(), "the black creature came back");
}

/// Epic Struggle wins the game once the board is wide enough.
#[test]
fn epic_struggle_wins_at_twenty_creatures() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::epic_struggle());
    for _ in 0..20 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.is_game_over() && g.players[0].is_alive(), "you win at twenty creatures");
}

/// A Phantom trades exactly one counter per damage event, however big.
#[test]
fn phantom_centaur_sheds_one_counter_per_hit() {
    let mut g = main_phase();
    let spell = g.add_card_to_hand(0, catalog::phantom_centaur());
    cast(&mut g, 0, spell, None);
    let centaur = g.battlefield.iter().find(|c| c.definition.name == "Phantom Centaur").unwrap().id;
    let cp = g.computed_permanent(centaur).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 3), "2/0 plus three counters");
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(centaur)));
    let cp = g.computed_permanent(centaur).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 2), "three damage cost exactly one counter");
    assert_eq!(g.battlefield_find(centaur).unwrap().damage, 0, "the damage was prevented");
}

/// Phantom Nantuko can rebuild its own counters.
#[test]
fn phantom_nantuko_regrows_counters() {
    let mut g = main_phase();
    let nantuko = g.add_card_to_battlefield(0, catalog::phantom_nantuko());
    g.battlefield_find_mut(nantuko).unwrap().summoning_sick = false;
    let before = g.computed_permanent(nantuko).unwrap().power;
    activate(&mut g, 0, nantuko, 0, None);
    assert_eq!(g.computed_permanent(nantuko).unwrap().power, before + 1);
}

/// Forcemage Advocate pays an opponent a card for a +1/+1 counter.
#[test]
fn forcemage_advocate_trades_a_card_for_a_counter() {
    let mut g = main_phase();
    let theirs = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mage = g.add_card_to_battlefield(0, catalog::forcemage_advocate());
    g.battlefield_find_mut(mage).unwrap().summoning_sick = false;
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mage,
        ability_index: 0,
        target: Some(Target::Permanent(theirs)),
        additional_targets: vec![Target::Permanent(mine)],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == theirs), "they got the card back");
    assert_eq!(g.computed_permanent(mine).unwrap().power, 3, "and you got the counter");
}

/// Mirari's Wake anthems the team and doubles a land tap.
#[test]
fn miraris_wake_anthems_and_doubles_mana() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::miraris_wake());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("tap the Forest");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "the Wake doubled it");
}

/// Mirror Wall buys its way out of Defender for a turn.
#[test]
fn mirror_wall_can_be_let_off_the_leash() {
    let mut g = main_phase();
    let wall = g.add_card_to_battlefield(0, catalog::mirror_wall());
    assert!(g.computed_permanent(wall).unwrap().keywords.contains(&Keyword::Defender));
    activate(&mut g, 0, wall, 0, None);
    assert!(!g.computed_permanent(wall).unwrap().keywords.contains(&Keyword::Defender));
}

/// Nantuko Monastery only animates past Threshold.
#[test]
fn nantuko_monastery_animates_at_threshold() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::nantuko_monastery());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: land,
            ability_index: 1,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "closed before Threshold"
    );
    fill_graveyard(&mut g, 0);
    activate(&mut g, 0, land, 1, None);
    let cp = g.computed_permanent(land).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.card_types.contains(&crabomination::card::CardType::Land), "it's still a land");
}

/// Quiet Speculation stocks a graveyard with flashback cards.
#[test]
fn quiet_speculation_stocks_flashback() {
    let mut g = main_phase();
    let a = g.add_card_to_library(0, catalog::ray_of_revelation());
    let b = g.add_card_to_library(0, catalog::canopy_claws());
    g.add_card_to_library(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::quiet_speculation());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(a)),
        DecisionAnswer::Search(Some(b)),
        DecisionAnswer::Search(None),
    ]));
    cast(&mut g, 0, spell, Some(Target::Player(0)));
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.id == a || c.id == b).count(), 2);
}

/// Rats' Feast eats X cards out of one graveyard.
#[test]
fn rats_feast_eats_x_cards() {
    let mut g = main_phase();
    for _ in 0..4 {
        g.add_card_to_graveyard(1, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::rats_feast());
    cast_x(&mut g, 0, spell, Some(Target::Player(1)), Some(3));
    assert_eq!(g.players[1].graveyard.len(), 1);
}

/// Masked Gorgon gives green and white creatures protection from Gorgons.
#[test]
fn masked_gorgon_walls_off_green_and_white() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::masked_gorgon());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(
        g.computed_permanent(bear)
            .unwrap()
            .keywords
            .iter()
            .any(|k| matches!(k, Keyword::ProtectionFromMatching(_)))
    );
}

/// Nomad Mythmaker replays an Aura out of a graveyard.
#[test]
fn nomad_mythmaker_replays_an_aura() {
    let mut g = main_phase();
    let aura = g.add_card_to_graveyard(1, catalog::cagemail());
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mythmaker = g.add_card_to_battlefield(0, catalog::nomad_mythmaker());
    g.battlefield_find_mut(mythmaker).unwrap().summoning_sick = false;
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mythmaker,
        ability_index: 0,
        target: Some(Target::Permanent(aura)),
        additional_targets: vec![Target::Permanent(host)],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(aura).map(|c| c.attached_to), Some(Some(host)));
}

/// Book Burning mills when nobody takes the six.
#[test]
fn book_burning_mills_when_nobody_bites() {
    let mut g = main_phase();
    for _ in 0..8 {
        g.add_card_to_library(1, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::book_burning());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(false),
        DecisionAnswer::Bool(false),
    ]));
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].graveyard.len(), 6);
    assert_eq!(g.players[1].life, 20);
}

/// …and deals the six to the first taker instead.
#[test]
fn book_burning_burns_the_volunteer() {
    let mut g = main_phase();
    for _ in 0..8 {
        g.add_card_to_library(1, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::book_burning());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert_eq!(g.players[0].life, 14, "the caster is asked first");
    assert_eq!(g.players[1].graveyard.len(), 0, "no mill");
}

/// Breaking Point wraths when nobody takes the six.
#[test]
fn breaking_point_wraths_when_declined() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::breaking_point());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(false),
        DecisionAnswer::Bool(false),
    ]));
    cast(&mut g, 0, spell, None);
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none());
}

/// Dwarven Driller kills the land unless its controller eats the two.
#[test]
fn dwarven_driller_punishes_the_land() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let driller = g.add_card_to_battlefield(0, catalog::dwarven_driller());
    g.battlefield_find_mut(driller).unwrap().summoning_sick = false;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    activate(&mut g, 0, driller, 0, Some(Target::Permanent(land)));
    assert!(g.battlefield_find(land).is_none(), "they declined, so the land died");
}

/// Toxic Stench upgrades to a kill past Threshold.
#[test]
fn toxic_stench_kills_at_threshold() {
    let mut g = main_phase();
    fill_graveyard(&mut g, 0);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::toxic_stench());
    cast(&mut g, 0, spell, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(victim).is_none());
}

/// Stitch Together reanimates straight to play past Threshold.
#[test]
fn stitch_together_reanimates_at_threshold() {
    let mut g = main_phase();
    fill_graveyard(&mut g, 0);
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::stitch_together());
    cast(&mut g, 0, spell, Some(Target::Permanent(dead)));
    assert!(g.battlefield_find(dead).is_some());
}

/// Silver Seraph anthems the rest of the team, not itself.
#[test]
fn silver_seraph_anthems_the_others() {
    let mut g = main_phase();
    fill_graveyard(&mut g, 0);
    let seraph = g.add_card_to_battlefield(0, catalog::silver_seraph());
    let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(friend).unwrap().power, 4);
    assert_eq!(g.computed_permanent(seraph).unwrap().power, 6, "not itself");
}

/// Treacherous Werewolf costs you 4 when it dies past Threshold.
#[test]
fn treacherous_werewolf_bites_back_at_threshold() {
    let mut g = main_phase();
    fill_graveyard(&mut g, 0);
    let wolf = g.add_card_to_battlefield(0, catalog::treacherous_werewolf());
    assert_eq!(g.computed_permanent(wolf).unwrap().power, 4);
    for _ in 0..2 {
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        cast(&mut g, 1, bolt, Some(Target::Permanent(wolf)));
    }
    assert!(g.battlefield_find(wolf).is_none(), "six damage kills the 4/4");
    assert_eq!(g.players[0].life, 16);
}

/// Wormfang Newt holds a land hostage until it leaves.
#[test]
fn wormfang_newt_holds_a_land_hostage() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let newt = g.add_card_to_hand(0, catalog::wormfang_newt());
    cast(&mut g, 0, newt, None);
    assert!(g.battlefield_find(land).is_none(), "the land is jailed");
    let id = g.battlefield.iter().find(|c| c.definition.name == "Wormfang Newt").unwrap().id;
    let _ = g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_some(), "and back when the Newt goes");
}

/// Worldgorger Dragon eats your board while it's out.
#[test]
fn worldgorger_dragon_eats_the_board() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let dragon = g.add_card_to_hand(0, catalog::worldgorger_dragon());
    cast(&mut g, 0, dragon, None);
    assert!(g.battlefield_find(land).is_none() && g.battlefield_find(bear).is_none());
    let id = g.battlefield.iter().find(|c| c.definition.name == "Worldgorger Dragon").unwrap().id;
    let _ = g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_some() && g.battlefield_find(bear).is_some());
}

/// Soulcatchers' Aerie grows the flock for each dead Bird.
#[test]
fn soulcatchers_aerie_counts_dead_birds() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::soulcatchers_aerie());
    let flock = g.add_card_to_battlefield(0, catalog::battlewise_aven());
    let dead_bird = g.add_card_to_battlefield(0, catalog::phantom_flock());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(dead_bird)));
    let _ = g.remove_to_graveyard_with_triggers(dead_bird);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(flock).unwrap().power, 3, "2/2 plus one feather");
}

/// Swelter splits two damage onto two creatures.
#[test]
fn swelter_hits_two_creatures() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::swelter());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}

/// Spirit Cairn turns any discard into a Spirit for {W}.
#[test]
fn spirit_cairn_mints_on_a_discard() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::spirit_cairn());
    let wurm = g.add_card_to_battlefield(0, catalog::tunneler_wurm());
    g.add_card_to_hand(0, catalog::forest());
    mana(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    activate(&mut g, 0, wurm, 0, None);
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Spirit"));
}

/// Barbarian Bully's pump only lands if nobody eats the four.
#[test]
fn barbarian_bully_pumps_when_declined() {
    let mut g = main_phase();
    g.add_card_to_hand(0, catalog::forest());
    let bully = g.add_card_to_battlefield(0, catalog::barbarian_bully());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(false),
        DecisionAnswer::Bool(false),
    ]));
    activate(&mut g, 0, bully, 0, None);
    assert_eq!(g.computed_permanent(bully).unwrap().power, 4);
}

/// Infectious Rage hops to another creature when its host dies.
#[test]
fn infectious_rage_hops_on_death() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let next = g.add_card_to_battlefield(1, catalog::giant_warthog());
    let aura = g.add_card_to_hand(0, catalog::infectious_rage());
    cast(&mut g, 0, aura, Some(Target::Permanent(host)));
    assert_eq!(g.computed_permanent(host).unwrap().power, 4, "+2/-1");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Permanent(host)));
    assert_eq!(g.battlefield_find(aura).map(|c| c.attached_to), Some(Some(next)));
}

/// Lost in Thought switches the host off.
#[test]
fn lost_in_thought_locks_the_host_down() {
    let mut g = main_phase();
    let shade = g.add_card_to_battlefield(1, catalog::nantuko_shade());
    let aura = g.add_card_to_hand(0, catalog::lost_in_thought());
    cast(&mut g, 0, aura, Some(Target::Permanent(shade)));
    let kws = g.computed_permanent(shade).unwrap().keywords;
    assert!(kws.contains(&Keyword::CantAttack) && kws.contains(&Keyword::CantBlock));
    assert!(kws.contains(&Keyword::CantActivateAbilities));
}

/// Morality Shift turns your graveyard into your library.
#[test]
fn morality_shift_swaps_the_zones() {
    let mut g = main_phase();
    for _ in 0..5 {
        g.add_card_to_graveyard(0, catalog::forest());
    }
    g.add_card_to_library(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::morality_shift());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.players[0].library.len(), 5);
    assert_eq!(g.players[0].graveyard.len(), 2, "the old library plus the spell");
}

/// Seedtime only takes the extra turn after a blue spell.
#[test]
fn seedtime_needs_a_blue_spell() {
    let mut g = main_phase();
    let spell = g.add_card_to_hand(0, catalog::seedtime());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.players[0].extra_turns, 0, "no blue, no turn");
    let counter = g.add_card_to_hand(1, catalog::envelop());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    let _ = g.perform_action(GameAction::CastSpell {
        card_id: counter,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    let again = g.add_card_to_hand(0, catalog::seedtime());
    cast(&mut g, 0, again, None);
    assert_eq!(g.players[0].extra_turns, 1);
}

/// Wormfang Manta mortgages your next turn and pays it back when it leaves.
#[test]
fn wormfang_manta_trades_turns() {
    let mut g = main_phase();
    let manta = g.add_card_to_hand(0, catalog::wormfang_manta());
    cast(&mut g, 0, manta, None);
    assert_eq!(g.players[0].skip_turns, 1);
    let id = g.battlefield.iter().find(|c| c.definition.name == "Wormfang Manta").unwrap().id;
    let _ = g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    assert_eq!(g.players[0].extra_turns, 1);
}

// ── Closing wave (jud2) ─────────────────────────────────────────────────────

/// Burning Wish pulls a sorcery out of the sideboard and exiles itself.
#[test]
fn burning_wish_fetches_a_sorcery_and_exiles() {
    let mut g = main_phase();
    g.add_card_to_sideboard(0, catalog::grizzly_bears());
    let wanted = g.add_card_to_sideboard(0, catalog::morality_shift());
    let spell = g.add_card_to_hand(0, catalog::burning_wish());
    cast(&mut g, 0, spell, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == wanted), "the sorcery, not the bear");
    assert!(g.exile.iter().any(|c| c.id == spell), "Burning Wish exiles itself");
}

/// Death Wish takes any card and half your life, rounded up.
#[test]
fn death_wish_costs_half_your_life() {
    let mut g = main_phase();
    g.players[0].life = 21;
    g.add_card_to_sideboard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::death_wish());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[0].life, 10, "21 - 11");
}

/// Grave Consequences drains for every card left in a graveyard.
#[test]
fn grave_consequences_drains_per_graveyard_card() {
    let mut g = main_phase();
    fill_graveyard(&mut g, 1);
    let spell = g.add_card_to_hand(0, catalog::grave_consequences());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.players[1].life, 13, "seven cards, seven life");
    assert_eq!(g.players[0].life, 20, "an empty graveyard costs nothing");
}

/// Scalpelexis exiles four, then four more on a duplicate name.
#[test]
fn scalpelexis_repeats_on_duplicate_names() {
    let mut g = main_phase();
    let fish = g.add_card_to_battlefield(0, catalog::scalpelexis());
    // First four share a name (repeat), the next four don't (stop).
    for _ in 0..4 {
        g.add_card_to_library(1, catalog::forest());
    }
    for f in [catalog::grizzly_bears, catalog::mountain, catalog::island, catalog::plains] {
        g.add_card_to_library(1, f());
    }
    combat_damage_to_player(&mut g, fish, 1);
    assert_eq!(g.exile.len(), 8, "one repeat, then a clean batch");
}

/// Soulgorger Orgg takes you to 1 and pays it all back when it leaves.
#[test]
fn soulgorger_orgg_repays_the_life_it_took() {
    let mut g = main_phase();
    let orgg = g.add_card_to_hand(0, catalog::soulgorger_orgg());
    cast(&mut g, 0, orgg, None);
    assert_eq!(g.players[0].life, 1);
    let id = g.battlefield.iter().find(|c| c.definition.name == "Soulgorger Orgg").unwrap().id;
    let _ = g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20);
}

/// Sutured Ghoul's body is the pile it exiled on the way in.
#[test]
fn sutured_ghoul_is_its_exiled_pile() {
    let mut g = main_phase();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let ghoul = g.add_card_to_hand(0, catalog::sutured_ghoul());
    cast(&mut g, 0, ghoul, None);
    let id = g.battlefield.iter().find(|c| c.definition.name == "Sutured Ghoul").unwrap().id;
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "two 2/2s stitched together");
}

/// Spelljack counters a spell and hands it to you for free.
#[test]
fn spelljack_steals_the_countered_spell() {
    let mut g = main_phase();
    let victim = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: victim,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    let jack = g.add_card_to_hand(0, catalog::spelljack());
    cast(&mut g, 0, jack, Some(Target::Permanent(victim)));
    let exiled = g.exile.iter().find(|c| c.id == victim).expect("countered into exile");
    assert_eq!(exiled.may_play_until.map(|m| m.player), Some(0));
}

/// Mist of Stagnation freezes the untap step and untaps one per graveyard card.
#[test]
fn mist_of_stagnation_freezes_and_ransoms_untaps() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::mist_of_stagnation());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.do_untap();
    assert!(g.battlefield_find(bear).unwrap().tapped, "nothing untaps");
    g.add_card_to_graveyard(0, catalog::forest());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bear).unwrap().tapped, "one card, one untap");
}

/// Web of Inertia shuts down attacks unless the attacker pays a graveyard card.
#[test]
fn web_of_inertia_taxes_attacks_with_the_graveyard() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::web_of_inertia());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().summoning_sick = false;
    g.active_player_idx = 1;
    g.step = TurnStep::BeginCombat;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![])]));
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bear,
            target: AttackTarget::Player(0),
        }]))
        .is_err(),
        "declining the exile locks out the attack"
    );
}

/// Riftstone Portal turns your lands into Selesnya duals from the graveyard.
#[test]
fn riftstone_portal_fixes_from_the_graveyard() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::mountain());
    assert_eq!(g.granted_abilities_for(land).len(), 0);
    g.add_card_to_graveyard(0, catalog::riftstone_portal());
    assert_eq!(g.granted_abilities_for(land).len(), 1, "lands you control gain the G/W tap");
}

/// Prismatic Strands' flashback only accepts a white creature's tap.
#[test]
fn prismatic_strands_flashback_needs_a_white_creature() {
    let mut g = main_phase();
    let strands = g.add_card_to_graveyard(0, catalog::prismatic_strands());
    let green = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastFlashbackTap {
            card_id: strands,
            tap_creatures: vec![green],
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "a green bear can't pay a white tap"
    );
}

/// Shaman's Trance opens every graveyard to you and closes them to everyone else.
#[test]
fn shamans_trance_pools_the_graveyards() {
    let mut g = main_phase();
    let spell = g.add_card_to_hand(0, catalog::shamans_trance());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.graveyard_play_pooled_for, Some(0));
    let bear = catalog::grizzly_bears();
    assert!(
        g.cast_from_zone_blocked(1, &bear, Zone::Graveyard),
        "opponents lose their own graveyards"
    );
    assert!(!g.cast_from_zone_blocked(0, &bear, Zone::Graveyard));
}

/// Cephalid Constable bounces one permanent per point of combat damage.
#[test]
fn cephalid_constable_bounces_per_damage() {
    let mut g = main_phase();
    let cop = g.add_card_to_battlefield(0, catalog::cephalid_constable());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::forest());
    combat_damage_to_player(&mut g, cop, 1);
    assert_eq!(g.players[1].hand.len(), 1, "1 power, one permanent home");
}

/// Planar Chaos counters a spell whose caster loses the flip.
#[test]
fn planar_chaos_counters_on_a_lost_flip() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::planar_chaos());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    cast(&mut g, 0, bear, None);
    assert!(g.battlefield_find(bear).is_none(), "tails counters it");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear));
}

/// Telekinetic Bonds buys a tap off any discard for {1}{U}.
#[test]
fn telekinetic_bonds_taps_on_a_discard() {
    let mut g = main_phase();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::telekinetic_bonds());
    let junk = g.add_card_to_hand(1, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let mut events = Vec::new();
    g.discard_card(1, junk, &mut events);
    g.dispatch_triggers_for_events(&events);
    mana(&mut g, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).unwrap().tapped);
}

/// Living Wish only reaches creature and land cards.
#[test]
fn living_wish_ignores_a_sideboard_instant() {
    let mut g = main_phase();
    g.add_card_to_sideboard(0, catalog::envelop());
    let wanted = g.add_card_to_sideboard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::living_wish());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.players[0].hand.iter().map(|c| c.id).collect::<Vec<_>>(), vec![wanted]);
}

/// Prismatic Strands fogs every source of the chosen colour.
#[test]
fn prismatic_strands_fogs_a_colour() {
    let mut g = main_phase();
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::prismatic_strands());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Green)]));
    cast(&mut g, 0, spell, None);
    let before = g.players[0].life;
    let mut events = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(0),
        2,
        Some(attacker),
        &mut events,
    );
    assert_eq!(g.players[0].life, before, "green sources deal nothing");
}
