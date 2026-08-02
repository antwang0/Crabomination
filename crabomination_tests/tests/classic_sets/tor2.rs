//! Torment (TOR) — the closing wave: the Dreams cycle, the Possessed cycle
//! and the Nightmare Horrors.

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target};
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

/// Seven cards in the graveyard turns Threshold on.
fn fill_graveyard(g: &mut GameState, seat: usize) {
    for _ in 0..7 {
        g.add_card_to_graveyard(seat, catalog::forest());
    }
}

// ── The Dreams cycle ────────────────────────────────────────────────────────

/// Restless Dreams discards X and buys back that many creature cards.
#[test]
fn restless_dreams_trades_discards_for_creatures() {
    let mut g = main_phase();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let other = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::restless_dreams());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear, other])]));
    cast_x(&mut g, 0, spell, None, Some(2));
    assert_eq!(g.players[0].hand.iter().filter(|c| c.id == bear || c.id == other).count(), 2);
    assert!(g.players[0].hand.iter().all(|c| !c.definition.is_land()), "both Forests discarded");
}

/// Turbulent Dreams bounces one nonland permanent per card discarded.
#[test]
fn turbulent_dreams_bounces_x_permanents() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::turbulent_dreams());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![victim])]));
    cast_x(&mut g, 0, spell, None, Some(1));
    assert!(g.battlefield_find(victim).is_none(), "bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == victim));
}

/// Devastating Dreams eats X lands from each player and sweeps for X.
#[test]
fn devastating_dreams_wipes_lands_and_creatures() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::forest());
    let theirs = g.add_card_to_battlefield(1, catalog::forest());
    let victim = g.add_card_to_battlefield(1, catalog::mystic_familiar());
    g.add_card_to_hand(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::devastating_dreams());
    cast_x(&mut g, 0, spell, None, Some(1));
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none());
    assert_eq!(g.battlefield_find(victim).map(|c| c.damage), Some(1), "the 1/2 took 1");
}

// ── The Possessed cycle ─────────────────────────────────────────────────────

/// Possessed Centaur is a plain 3/3 until Threshold, then a black 4/4 with a
/// green-creature kill switch.
#[test]
fn possessed_centaur_turns_on_at_threshold() {
    let mut g = main_phase();
    let centaur = g.add_card_to_battlefield(0, catalog::possessed_centaur());
    g.battlefield_find_mut(centaur).unwrap().summoning_sick = false;
    let cp = g.computed_permanent(centaur).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(!cp.colors.contains(&Color::Black));
    fill_graveyard(&mut g, 0);
    let cp = g.computed_permanent(centaur).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.colors.contains(&Color::Black));
    let prey = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, centaur, 0, Some(Target::Permanent(prey)));
    assert!(g.battlefield_find(prey).is_none(), "the green creature died");
}

// ── Nightmare Horrors ───────────────────────────────────────────────────────

/// Soul Scourge drains three, and refunds it when it leaves.
#[test]
fn soul_scourge_drains_then_refunds() {
    let mut g = main_phase();
    let scourge = g.add_card_to_hand(0, catalog::soul_scourge());
    cast(&mut g, 0, scourge, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 17);
    let id = g.battlefield.iter().find(|c| c.definition.name == "Soul Scourge").unwrap().id;
    let _ = g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20, "the remembered player got it back");
}

/// Petradon jails two lands until it leaves.
#[test]
fn petradon_holds_two_lands() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::forest());
    let b = g.add_card_to_battlefield(1, catalog::forest());
    let don = g.add_card_to_hand(0, catalog::petradon());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: don,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    let id = g.battlefield.iter().find(|c| c.definition.name == "Petradon").unwrap().id;
    let _ = g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_some() && g.battlefield_find(b).is_some(), "both lands back");
}

// ── Threshold creatures ─────────────────────────────────────────────────────

/// Seton's Scout is a 2/1 that grows past Threshold.
#[test]
fn setons_scout_grows_at_threshold() {
    let mut g = main_phase();
    let scout = g.add_card_to_battlefield(0, catalog::setons_scout());
    assert_eq!(g.computed_permanent(scout).unwrap().power, 2);
    fill_graveyard(&mut g, 0);
    let cp = g.computed_permanent(scout).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3));
}

/// Nantuko Blightcutter counts the opponents' black permanents past Threshold.
#[test]
fn nantuko_blightcutter_scales_off_black() {
    let mut g = main_phase();
    let cutter = g.add_card_to_battlefield(0, catalog::nantuko_blightcutter());
    g.add_card_to_battlefield(1, catalog::nantuko_shade());
    assert_eq!(g.computed_permanent(cutter).unwrap().power, 2, "off before Threshold");
    fill_graveyard(&mut g, 0);
    assert_eq!(g.computed_permanent(cutter).unwrap().power, 3, "+1 for the Shade");
}

/// Pardic Arsonist only brings its Lava Spike past Threshold.
#[test]
fn pardic_arsonist_pings_at_threshold() {
    let mut g = main_phase();
    fill_graveyard(&mut g, 0);
    let spell = g.add_card_to_hand(0, catalog::pardic_arsonist());
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 17);
}

/// Gloomdrifter's arrival shrinks the nonblack board past Threshold.
#[test]
fn gloomdrifter_shrinks_nonblack_creatures() {
    let mut g = main_phase();
    fill_graveyard(&mut g, 0);
    let white = g.add_card_to_battlefield(1, catalog::mystic_familiar());
    let black = g.add_card_to_battlefield(1, catalog::nantuko_shade());
    let drifter = g.add_card_to_hand(0, catalog::gloomdrifter());
    cast(&mut g, 0, drifter, None);
    assert!(g.battlefield_find(white).is_none(), "the 1/2 died to -2/-2");
    assert!(g.battlefield_find(black).is_some(), "black creatures are spared");
}

/// Reborn Hero buys itself back past Threshold.
#[test]
fn reborn_hero_returns_at_threshold() {
    let mut g = main_phase();
    fill_graveyard(&mut g, 0);
    let hero = g.add_card_to_battlefield(0, catalog::reborn_hero());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, 1, bolt, Some(Target::Permanent(hero)));
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Reborn Hero"),
        "paid {{W}}{{W}} to come back"
    );
}

// ── Other creatures ─────────────────────────────────────────────────────────

/// Barbarian Outcast needs a Swamp on the board.
#[test]
fn barbarian_outcast_needs_a_swamp() {
    let mut g = main_phase();
    let swamp = g.add_card_to_battlefield(0, catalog::swamp());
    let outcast = g.add_card_to_battlefield(0, catalog::barbarian_outcast());
    g.check_state_based_actions();
    assert!(g.battlefield_find(outcast).is_some(), "the Swamp keeps it alive");
    let _ = g.remove_to_graveyard_with_triggers(swamp);
    g.check_state_based_actions();
    assert!(g.battlefield_find(outcast).is_none(), "sacrificed with no Swamp");
}

/// Carrion Rats can be blanked by any player for a graveyard card.
#[test]
fn carrion_rats_blanked_by_a_graveyard_exile() {
    let mut g = main_phase();
    g.add_card_to_graveyard(1, catalog::forest());
    let rats = g.add_card_to_battlefield(0, catalog::carrion_rats());
    g.battlefield_find_mut(rats).unwrap().summoning_sick = false;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: rats,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.combat_damage_prevented_from(rats), "the Rats assign no combat damage");
}

/// Longhorn Firebeast dies when an opponent takes the 5.
#[test]
fn longhorn_firebeast_can_be_bought_off() {
    let mut g = main_phase();
    let spell = g.add_card_to_hand(0, catalog::longhorn_firebeast());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, 0, spell, None);
    assert_eq!(g.players[1].life, 15);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Longhorn Firebeast"));
}

/// Gurzigost eats two graveyard cards each upkeep or dies.
#[test]
fn gurzigost_pays_its_upkeep_from_the_graveyard() {
    let mut g = main_phase();
    fill_graveyard(&mut g, 0);
    let beast = g.add_card_to_battlefield(0, catalog::gurzigost());
    let before = g.players[0].graveyard.len();
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(beast).is_some(), "the graveyard paid");
    assert_eq!(g.players[0].graveyard.len(), before - 2);
}

/// Nantuko Cultivator turns spare lands into counters and cards.
#[test]
fn nantuko_cultivator_cashes_in_lands() {
    let mut g = main_phase();
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::forest());
        g.add_card_to_library(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::nantuko_cultivator());
    cast(&mut g, 0, spell, None);
    let id = g.battlefield.iter().find(|c| c.definition.name == "Nantuko Cultivator").unwrap().id;
    assert_eq!(
        g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "one counter per land discarded"
    );
    assert_eq!(g.players[0].hand.len(), 2, "and two cards back");
}

/// Stern Judge taxes every player per Swamp they control.
#[test]
fn stern_judge_taxes_per_swamp() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::swamp());
    g.add_card_to_battlefield(1, catalog::swamp());
    let judge = g.add_card_to_battlefield(0, catalog::stern_judge());
    g.battlefield_find_mut(judge).unwrap().summoning_sick = false;
    activate(&mut g, 0, judge, 0, None);
    assert_eq!(g.players[1].life, 18);
    assert_eq!(g.players[0].life, 20, "no Swamps, no loss");
}

/// Zombie Trailblazer taps a friend to turn a land into a Swamp.
#[test]
fn zombie_trailblazer_makes_swamps() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let blazer = g.add_card_to_battlefield(0, catalog::zombie_trailblazer());
    let helper = g.add_card_to_battlefield(0, catalog::zombie_trailblazer());
    g.battlefield_find_mut(helper).unwrap().summoning_sick = false;
    activate(&mut g, 0, blazer, 0, Some(Target::Permanent(land)));
    assert!(
        g.computed_permanent(land)
            .unwrap()
            .subtypes
            .land_types
            .contains(&crabomination::card::LandType::Swamp)
    );
}

/// Shambling Swarm spreads three -1/-1 counters as it dies.
#[test]
fn shambling_swarm_spreads_minus_counters() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let swarm = g.add_card_to_battlefield(0, catalog::shambling_swarm());
    let _ = g.remove_to_graveyard_with_triggers(swarm);
    drain_stack(&mut g);
    assert!(
        g.battlefield_find(victim).is_none_or(|c| c.counter_count(CounterType::MinusOneMinusOne) > 0),
        "the 2/2 took counters"
    );
}

// ── Enchantments and Auras ──────────────────────────────────────────────────

/// Last Laugh pings the table on every death, then sacrifices itself.
#[test]
fn last_laugh_pings_on_every_death() {
    let mut g = main_phase();
    let laugh = g.add_card_to_battlefield(0, catalog::last_laugh());
    let survivor = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Permanent(victim)));
    assert_eq!(g.players[0].life, 19);
    assert_eq!(g.players[1].life, 19);
    assert_eq!(g.battlefield_find(survivor).map(|c| c.damage), Some(1));
    let _ = g.remove_to_graveyard_with_triggers(survivor);
    g.check_state_based_actions();
    assert!(g.battlefield_find(laugh).is_none(), "no creatures left, so it goes");
}

/// Transcendence turns life loss into life gain, and 20 into a loss.
#[test]
fn transcendence_inverts_life() {
    let mut g = main_phase();
    g.players[0].life = 10;
    g.add_card_to_battlefield(0, catalog::transcendence());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Player(0)));
    assert_eq!(g.players[0].life, 13, "lost 3, gained 6");
}

/// Stupefying Touch draws a card and switches the host's abilities off.
#[test]
fn stupefying_touch_locks_activated_abilities() {
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::forest());
    let shade = g.add_card_to_battlefield(1, catalog::nantuko_shade());
    let aura = g.add_card_to_hand(0, catalog::stupefying_touch());
    cast(&mut g, 0, aura, Some(Target::Permanent(shade)));
    assert_eq!(g.players[0].hand.len(), 1, "drew a card");
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: shade,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the Shade is switched off"
    );
}

/// Shade's Form steals the creature it kills.
#[test]
fn shades_form_steals_the_host_on_death() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::shades_form());
    cast(&mut g, 0, aura, Some(Target::Permanent(victim)));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Permanent(victim)));
    let back = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears");
    assert_eq!(back.map(|c| c.controller), Some(0), "it came back under your control");
}

/// Floating Shield hands out protection from the colour it named.
#[test]
fn floating_shield_grants_chosen_protection() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::floating_shield());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    cast(&mut g, 0, aura, Some(Target::Permanent(host)));
    assert!(
        g.computed_permanent(host).unwrap().keywords.contains(&Keyword::Protection(Color::Red))
    );
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Kamahl's Sledge only hits the controller past Threshold.
#[test]
fn kamahls_sledge_burns_the_controller_at_threshold() {
    let mut g = main_phase();
    fill_graveyard(&mut g, 0);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::kamahls_sledge());
    cast(&mut g, 0, spell, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(victim).is_none());
    assert_eq!(g.players[1].life, 16);
}

/// Temporary Insanity only steals creatures smaller than your graveyard.
#[test]
fn temporary_insanity_steals_a_small_creature() {
    let mut g = main_phase();
    fill_graveyard(&mut g, 0);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::temporary_insanity());
    cast(&mut g, 0, spell, Some(Target::Permanent(victim)));
    let cp = g.computed_permanent(victim).unwrap();
    assert_eq!(cp.controller, 0);
    assert!(cp.keywords.contains(&Keyword::Haste));
}

/// Spirit Flare taps one of yours to shoot an attacker.
#[test]
fn spirit_flare_shoots_an_attacker() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
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
    let spell = g.add_card_to_hand(0, catalog::spirit_flare());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(attacker)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).unwrap().tapped);
    assert!(g.battlefield_find(attacker).is_none(), "took 2 from a 2/2");
}

/// Plagiarize takes over the target player's draws for the turn.
#[test]
fn plagiarize_steals_the_draw() {
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(1, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::plagiarize());
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    let (mine, theirs) = (g.players[0].hand.len(), g.players[1].hand.len());
    let mut events = Vec::new();
    g.draw_one(1, &mut events);
    assert_eq!(g.players[1].hand.len(), theirs, "they skipped the draw");
    assert_eq!(g.players[0].hand.len(), mine + 1, "you drew instead");
}

/// Equal Treatment rewrites every damage event this turn to exactly 2.
#[test]
fn equal_treatment_flattens_damage() {
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::equal_treatment());
    cast(&mut g, 0, spell, None);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 18, "the 3 became a 2");
}

/// Flaming Gambit lets the victim take it on a creature instead.
#[test]
fn flaming_gambit_can_be_redirected() {
    let mut g = main_phase();
    let body = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::flaming_gambit());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast_x(&mut g, 0, spell, Some(Target::Player(1)), Some(2));
    assert_eq!(g.players[1].life, 20, "the player ducked it");
    assert!(g.battlefield_find(body).is_none(), "the 2/2 ate the 2 instead");
}

/// Retraced Image replays a card you already have a copy of.
#[test]
fn retraced_image_replays_a_matching_name() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let twin = g.add_card_to_hand(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::retraced_image());
    cast(&mut g, 0, spell, None);
    assert!(g.battlefield_find(twin).is_some(), "the twin hit the battlefield");
}

/// Parallel Evolution doubles every creature token.
#[test]
fn parallel_evolution_doubles_tokens() {
    let mut g = main_phase();
    let maker = g.add_card_to_hand(0, catalog::acorn_harvest());
    cast(&mut g, 0, maker, None);
    let before = g.battlefield.iter().filter(|c| c.is_token).count();
    assert!(before > 0, "Squirrels made");
    let spell = g.add_card_to_hand(0, catalog::parallel_evolution());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), before * 2);
}

/// Radiate forks a single-target spell onto every other legal target.
#[test]
fn radiate_forks_onto_every_other_target() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::mystic_familiar());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast bolt");
    let spell = g.add_card_to_hand(0, catalog::radiate());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast radiate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "both burned");
}
