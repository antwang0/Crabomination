//! Onslaught (ONS) wave 7 — the Chain cycle, the "choose a creature type"
//! rares and the remaining utility shell (`catalog::sets::ons3`).

use crabomination::card::{CardId, CounterType, CreatureType, Keyword};
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

fn bear() -> crabomination::card::CardDefinition {
    catalog::grizzly_bears()
}

fn attack_with(g: &mut GameState, attacker: CardId) {
    use crabomination::game::types::{Attack, AttackTarget};
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
}

// ── "Choose a creature type" ────────────────────────────────────────────────

/// Harsh Mercy spares one type per player and sweeps the rest (CR 701.7 —
/// destroyed without regeneration).
#[test]
fn harsh_mercy_spares_each_players_named_type() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::CreatureType(CreatureType::Bear),
        DecisionAnswer::CreatureType(CreatureType::Goblin),
    ]));
    let mine = g.add_card_to_battlefield(0, bear());
    let theirs = g.add_card_to_battlefield(1, catalog::goblin_sledder());
    let doomed = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    let mercy = g.add_card_to_hand(0, catalog::harsh_mercy());
    cast(&mut g, 0, mercy, None);
    assert!(g.battlefield_find(mine).is_some(), "Bears were named");
    assert!(g.battlefield_find(theirs).is_some(), "Goblins were named");
    assert!(g.battlefield_find(doomed).is_none(), "the Elf wasn't");
}

/// Patriarch's Bidding reanimates one named type out of every graveyard.
#[test]
fn patriarchs_bidding_reanimates_each_named_type() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::CreatureType(CreatureType::Bear),
        DecisionAnswer::CreatureType(CreatureType::Goblin),
    ]));
    g.add_card_to_graveyard(0, bear());
    g.add_card_to_graveyard(0, catalog::llanowar_elves());
    g.add_card_to_graveyard(1, catalog::goblin_sledder());
    let bidding = g.add_card_to_hand(0, catalog::patriarchs_bidding());
    cast(&mut g, 0, bidding, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), 1, "the Bear");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1).count(), 1, "the Goblin");
    assert_eq!(g.players[0].graveyard.len(), 2, "the Elf and the spell stay down");
}

/// Peer Pressure only fires when you control the most of the named type.
#[test]
fn peer_pressure_needs_the_biggest_tribe() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::CreatureType(
        CreatureType::Bear,
    )]));
    g.add_card_to_battlefield(0, bear());
    g.add_card_to_battlefield(0, bear());
    let stolen = g.add_card_to_battlefield(1, bear());
    let pressure = g.add_card_to_hand(0, catalog::peer_pressure());
    cast(&mut g, 0, pressure, None);
    assert_eq!(g.battlefield_find(stolen).unwrap().controller, 0);

    // Tied on Bears — nothing moves.
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::CreatureType(
        CreatureType::Bear,
    )]));
    g.add_card_to_battlefield(0, bear());
    let theirs = g.add_card_to_battlefield(1, bear());
    let pressure = g.add_card_to_hand(0, catalog::peer_pressure());
    cast(&mut g, 0, pressure, None);
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 1);
}

/// Riptide Chronologist untaps only the named tribe.
#[test]
fn riptide_chronologist_untaps_the_named_type() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::CreatureType(
        CreatureType::Bear,
    )]));
    let chrono = g.add_card_to_battlefield(0, catalog::riptide_chronologist());
    let b = g.add_card_to_battlefield(0, bear());
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    for id in [b, elf] {
        g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = true;
    }
    activate(&mut g, 0, chrono, 0, None);
    assert!(!g.battlefield_find(b).unwrap().tapped, "Bears untapped");
    assert!(g.battlefield_find(elf).unwrap().tapped, "the Elf stayed tapped");
}

/// Walking Desecration forces the named tribe to attack.
#[test]
fn walking_desecration_forces_the_named_tribe() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::CreatureType(
        CreatureType::Bear,
    )]));
    let desecration = g.add_card_to_battlefield(0, catalog::walking_desecration());
    g.clear_sickness(desecration);
    let b = g.add_card_to_battlefield(1, bear());
    let elf = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    activate(&mut g, 0, desecration, 0, None);
    assert!(g.computed_permanent(b).unwrap().keywords.contains(&Keyword::MustAttack));
    assert!(!g.computed_permanent(elf).unwrap().keywords.contains(&Keyword::MustAttack));
}

/// Endemic Plague sweeps everything sharing a type with the sacrificed creature.
#[test]
fn endemic_plague_sweeps_the_sacrificed_creatures_tribe() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, bear());
    let other_bear = g.add_card_to_battlefield(1, bear());
    let elf = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    let plague = g.add_card_to_hand(0, catalog::endemic_plague());
    cast(&mut g, 0, plague, None);
    assert!(g.battlefield_find(other_bear).is_none(), "shares Bear with the sacrifice");
    assert!(g.battlefield_find(elf).is_some(), "the Elf survives");
}

/// Callous Oppressor lets an opponent name the tribe it can't steal.
#[test]
fn callous_oppressor_cannot_steal_the_named_type() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::CreatureType(
        CreatureType::Bear,
    )]));
    let opp = g.add_card_to_hand(0, catalog::callous_oppressor());
    cast(&mut g, 0, opp, None);
    g.clear_sickness(opp);
    let named = g.add_card_to_battlefield(1, bear());
    let fair_game = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    assert_eq!(
        g.battlefield_find(opp).unwrap().chosen_creature_type,
        Some(CreatureType::Bear)
    );
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: opp,
            ability_index: 0,
            target: Some(Target::Permanent(named)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the named type is off limits"
    );
    activate(&mut g, 0, opp, 0, Some(Target::Permanent(fair_game)));
    assert_eq!(g.battlefield_find(fair_game).unwrap().controller, 0);
}

// ── Utility shell ───────────────────────────────────────────────────────────

/// Future Sight reveals the top card and lets you cast off it (CR 401.5).
#[test]
fn future_sight_casts_off_the_library_top() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::future_sight());
    let top = g.add_card_to_library(0, bear());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: top,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from the top of the library");
    drain_stack(&mut g);
    assert!(g.battlefield_find(top).is_some());
}

/// Goblin Machinist reads the first nonland off the top for its pump.
#[test]
fn goblin_machinist_pumps_by_the_revealed_mana_value() {
    let mut g = main_phase();
    let machinist = g.add_card_to_battlefield(0, catalog::goblin_machinist());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::krosan_colossus()); // {6}{G}{G}
    activate(&mut g, 0, machinist, 0, None);
    assert_eq!(g.computed_permanent(machinist).unwrap().power, 9);
}

/// Kaboom! flips the top of your deck once per chosen target.
#[test]
fn kaboom_damages_each_chosen_player() {
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::krosan_colossus());
    let boom = g.add_card_to_hand(0, catalog::kaboom());
    let life = g.players[1].life;
    cast(&mut g, 0, boom, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 9);
}

/// Astral Slide blinks a creature off any player's cycling trigger.
#[test]
fn astral_slide_blinks_on_a_cycle() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::astral_slide());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let victim = g.add_card_to_battlefield(1, bear());
    let cycler = g.add_card_to_hand(1, catalog::backslide());
    g.add_card_to_library(1, catalog::forest());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::Cycle { card_id: cycler, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "exiled until the end step");
}

/// Fleeting Aven bounces itself whenever anyone cycles.
#[test]
fn fleeting_aven_returns_on_a_cycle() {
    let mut g = main_phase();
    let aven = g.add_card_to_battlefield(0, catalog::fleeting_aven());
    let cycler = g.add_card_to_hand(1, catalog::backslide());
    g.add_card_to_library(1, catalog::forest());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::Cycle { card_id: cycler, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert!(g.battlefield_find(aven).is_none());
    assert!(g.players[0].hand.iter().any(|c| c.id == aven));
}

/// Cryptic Gateway only cheats in a creature sharing a type with BOTH taps.
#[test]
fn cryptic_gateway_needs_a_shared_type_with_each_tap() {
    let mut g = main_phase();
    let gateway = g.add_card_to_battlefield(0, catalog::cryptic_gateway());
    for _ in 0..2 {
        let b = g.add_card_to_battlefield(0, bear());
        g.clear_sickness(b);
    }
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    activate(&mut g, 0, gateway, 0, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == elf), "no shared type — stays in hand");

    let mut g = main_phase();
    let gateway = g.add_card_to_battlefield(0, catalog::cryptic_gateway());
    for _ in 0..2 {
        let b = g.add_card_to_battlefield(0, bear());
        g.clear_sickness(b);
    }
    let cheated = g.add_card_to_hand(0, bear());
    activate(&mut g, 0, gateway, 0, None);
    assert!(g.battlefield_find(cheated).is_some(), "Bear shares with both taps");
}

/// CR 702.6e — Lavamancer's Skill grants a stronger ping to a Wizard host.
#[test]
fn lavamancers_skill_scales_on_a_wizard() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.clear_sickness(host);
    let skill = g.add_card_to_hand(0, catalog::lavamancers_skill());
    cast(&mut g, 0, skill, Some(Target::Permanent(host)));
    let victim = g.add_card_to_battlefield(1, catalog::krosan_colossus());
    activate(&mut g, 0, host, 1, Some(Target::Permanent(victim)));
    assert_eq!(g.battlefield_find(victim).unwrap().damage, 1, "a Druid pings for 1");

    let mut g = main_phase();
    let wizard = g.add_card_to_battlefield(0, catalog::riptide_chronologist());
    g.clear_sickness(wizard);
    let skill = g.add_card_to_hand(0, catalog::lavamancers_skill());
    cast(&mut g, 0, skill, Some(Target::Permanent(wizard)));
    let victim = g.add_card_to_battlefield(1, catalog::krosan_colossus());
    activate(&mut g, 0, wizard, 2, Some(Target::Permanent(victim)));
    assert_eq!(g.battlefield_find(victim).unwrap().damage, 2, "a Wizard pings for 2");
}

/// Aurification turns anything that hits you into a gold-plated Wall, and
/// releases them when it leaves.
#[test]
fn aurification_walls_off_its_attackers() {
    let mut g = main_phase();
    let gold = g.add_card_to_battlefield(0, catalog::aurification());
    let attacker = g.add_card_to_battlefield(1, bear());
    let mut ev = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(0),
        2,
        Some(attacker),
        &mut ev,
    );
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(attacker).unwrap().counter_count(CounterType::Gold), 1);
    let computed = g.computed_permanent(attacker).unwrap();
    assert!(computed.keywords.contains(&Keyword::Defender));
    assert!(computed.subtypes.creature_types.contains(&CreatureType::Wall));

    let mut ev2 = vec![];
    g.destroy_permanent(gold, false, &mut ev2);
    g.dispatch_triggers_for_events(&ev2);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(attacker).unwrap().counter_count(CounterType::Gold), 0);
}

/// Tephraderm hands back every point of damage it takes.
#[test]
fn tephraderm_reflects_creature_damage() {
    let mut g = main_phase();
    let derm = g.add_card_to_battlefield(0, catalog::tephraderm());
    let attacker = g.add_card_to_battlefield(1, catalog::krosan_colossus());
    let mut ev = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(derm),
        3,
        Some(attacker),
        &mut ev,
    );
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(attacker).unwrap().damage, 3);
}

/// Read the Runes charges a permanent or a discard per card drawn.
#[test]
fn read_the_runes_charges_per_card_drawn() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_library(0, bear());
    }
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::forest());
    let runes = g.add_card_to_hand(0, catalog::read_the_runes());
    cast_x(&mut g, 0, runes, None, Some(2));
    assert_eq!(g.players[0].hand.len(), 0, "two drawn, both discarded to pay");
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0).count(),
        2,
        "the default keeps the Forests"
    );
}

/// Shieldmage Elder taps two Clerics to blank a creature's damage.
#[test]
fn shieldmage_elder_taps_two_clerics() {
    let mut g = main_phase();
    let elder = g.add_card_to_battlefield(0, catalog::shieldmage_elder());
    let cleric = g.add_card_to_battlefield(0, catalog::nova_cleric());
    g.clear_sickness(elder);
    g.clear_sickness(cleric);
    let attacker = g.add_card_to_battlefield(1, catalog::krosan_colossus());
    activate(&mut g, 0, elder, 0, Some(Target::Permanent(attacker)));
    let mut ev = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(0),
        9,
        Some(attacker),
        &mut ev,
    );
    assert_eq!(g.players[0].life, 20, "all of its damage is prevented");
}

/// Backslide only offers creatures with a morph ability as targets.
#[test]
fn backslide_needs_a_morph_ability() {
    let mut g = main_phase();
    let plain = g.add_card_to_battlefield(1, bear());
    let morpher = g.add_card_to_battlefield(1, catalog::skittish_valesk());
    let slide = g.add_card_to_hand(0, catalog::backslide());
    let effect = g.find_card_anywhere(slide).unwrap().definition.effect.clone();
    let legal = g.enumerate_legal_targets(&effect, 0);
    assert!(!legal.contains(&Target::Permanent(plain)));
    assert!(legal.contains(&Target::Permanent(morpher)));
    cast(&mut g, 0, slide, Some(Target::Permanent(morpher)));
    assert!(g.battlefield_find(morpher).unwrap().face_down);
}



/// Elvish Guidance adds a {G} per Elf when the enchanted land taps.
#[test]
fn elvish_guidance_scales_with_elves() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let guidance = g.add_card_to_hand(0, catalog::elvish_guidance());
    cast(&mut g, 0, guidance, Some(Target::Permanent(land)));
    g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.add_card_to_battlefield(1, catalog::llanowar_elves());
    g.players[0].mana_pool = Default::default();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 3, "one green plus one per Elf");
}

/// CR 509.1b — Graxiplon needs three defenders sharing a creature type.
#[test]
fn graxiplon_needs_a_defending_tribe() {
    let mut g = main_phase();
    let grax = g.add_card_to_battlefield(0, catalog::graxiplon());
    g.clear_sickness(grax);
    let blockers: Vec<CardId> = (0..2).map(|_| g.add_card_to_battlefield(1, bear())).collect();
    attack_with(&mut g, grax);
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(blockers[0], grax)])).is_err(),
        "only two Bears"
    );
    let third = g.add_card_to_battlefield(1, bear());
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(third, grax)])).is_ok());
}

/// Mana Echoes pays out per tribemate of the entering creature.
#[test]
fn mana_echoes_counts_the_shared_tribe() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.add_card_to_battlefield(0, catalog::mana_echoes());
    g.add_card_to_battlefield(0, bear());
    g.players[0].mana_pool = Default::default();
    let newcomer = g.add_card_to_hand(0, bear());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: newcomer,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 2, "itself plus the other Bear");
}

/// Thoughtbound Primoc defects to the player with the most Wizards.
#[test]
fn thoughtbound_primoc_follows_the_wizards() {
    let mut g = main_phase();
    let primoc = g.add_card_to_battlefield(0, catalog::thoughtbound_primoc());
    g.add_card_to_battlefield(1, catalog::riptide_chronologist());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(primoc).unwrap().controller, 1);

    // Tied on Wizards (zero each) — nobody takes it.
    let mut g = main_phase();
    let primoc = g.add_card_to_battlefield(0, catalog::thoughtbound_primoc());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(primoc).unwrap().controller, 0);
}

/// False Cure turns every player's life gain into a bigger loss.
#[test]
fn false_cure_punishes_any_life_gain() {
    let mut g = main_phase();
    let cure = g.add_card_to_hand(0, catalog::false_cure());
    cast(&mut g, 0, cure, None);
    let before = g.players[1].life;
    g.adjust_life(1, 3);
    assert_eq!(g.players[1].life, before + 3 - 6, "gained 3, lost 6");
}

/// Head Games swaps a hand for a hand-sized tutor out of the same library.
#[test]
fn head_games_rebuilds_the_hand_from_the_library() {
    let mut g = main_phase();
    for _ in 0..2 {
        g.add_card_to_hand(1, bear());
    }
    let wanted = g.add_card_to_library(1, catalog::forest());
    g.add_card_to_library(1, catalog::forest());
    let games = g.add_card_to_hand(0, catalog::head_games());
    let other = g.players[1].library.last().unwrap().id;
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Cards(vec![wanted, other])]));
    cast(&mut g, 0, games, Some(Target::Player(1)));
    assert_eq!(g.players[1].hand.len(), 2, "same size");
    assert!(
        g.players[1].hand.iter().all(|c| c.id == wanted || c.id == other),
        "the caster picked the replacements"
    );
}

/// Strongarm Tactics only spares the players who pitched a creature.
#[test]
fn strongarm_tactics_punishes_a_noncreature_discard() {
    let mut g = main_phase();
    g.add_card_to_hand(0, bear());
    g.add_card_to_hand(1, catalog::forest());
    let tactics = g.add_card_to_hand(0, catalog::strongarm_tactics());
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    cast(&mut g, 0, tactics, None);
    assert_eq!(g.players[0].life, l0, "pitched a creature");
    assert_eq!(g.players[1].life, l1 - 4, "pitched a land");
}

/// Kamahl's Summons mints a Bear per creature card in each hand.
#[test]
fn kamahls_summons_mints_a_bear_per_creature() {
    let mut g = main_phase();
    for _ in 0..2 {
        g.add_card_to_hand(0, bear());
    }
    g.add_card_to_hand(1, bear());
    g.add_card_to_hand(1, catalog::forest());
    let summons = g.add_card_to_hand(0, catalog::kamahls_summons());
    cast(&mut g, 0, summons, None);
    let tokens = |g: &GameState, seat: usize| {
        g.battlefield.iter().filter(|c| c.controller == seat && c.is_token).count()
    };
    assert_eq!(tokens(&g, 0), 2);
    assert_eq!(tokens(&g, 1), 1);
}

/// Trade Secrets runs one round when the opponent declines to repeat.
#[test]
fn trade_secrets_runs_once_when_declined() {
    let mut g = main_phase();
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(1, catalog::forest());
    }
    let secrets = g.add_card_to_hand(0, catalog::trade_secrets());
    cast(&mut g, 0, secrets, Some(Target::Player(1)));
    assert_eq!(g.players[1].hand.len(), 2);
    assert_eq!(g.players[0].hand.len(), 4);
}

/// Animal Magnetism deploys the creature and bins the rest.
#[test]
fn animal_magnetism_deploys_a_revealed_creature() {
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::krosan_colossus());
    let cheap = g.add_card_to_library(0, bear());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let magnetism = g.add_card_to_hand(0, catalog::animal_magnetism());
    cast(&mut g, 0, magnetism, None);
    assert!(g.battlefield_find(cheap).is_some(), "the opponent hands over the cheap one");
    assert_eq!(g.players[0].graveyard.len(), 5, "the other four plus the spell");
}

/// Circle of Solace shields you from one hit by the named tribe only.
#[test]
fn circle_of_solace_shields_the_named_tribe() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::CreatureType(
        CreatureType::Bear,
    )]));
    let circle = g.add_card_to_hand(0, catalog::circle_of_solace());
    cast(&mut g, 0, circle, None);
    let permanent =
        g.battlefield.iter().find(|c| c.definition.name == "Circle of Solace").unwrap().id;
    let b = g.add_card_to_battlefield(1, bear());
    let elf = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    activate(&mut g, 0, permanent, 0, None);
    let mut ev = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(0),
        2,
        Some(b),
        &mut ev,
    );
    assert_eq!(g.players[0].life, 20, "the Bear's hit is prevented");
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(0),
        1,
        Some(elf),
        &mut ev,
    );
    assert_eq!(g.players[0].life, 19, "the Elf isn't covered");
}

/// Riptide Replicator mints X/X tokens of the chosen colour and tribe.
#[test]
fn riptide_replicator_mints_the_chosen_tribe() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Color(Color::Blue),
        DecisionAnswer::CreatureType(CreatureType::Wizard),
    ]));
    let rep = g.add_card_to_hand(0, catalog::riptide_replicator());
    cast_x(&mut g, 0, rep, None, Some(3));
    let permanent =
        g.battlefield.iter().find(|c| c.definition.name == "Riptide Replicator").unwrap().id;
    assert_eq!(g.battlefield_find(permanent).unwrap().counter_count(CounterType::Charge), 3);
    g.clear_sickness(permanent);
    activate(&mut g, 0, permanent, 0, None);
    let token = g.battlefield.iter().find(|c| c.is_token).expect("token");
    assert_eq!((token.definition.power, token.definition.toughness), (3, 3));
    assert!(token.definition.subtypes.creature_types.contains(&CreatureType::Wizard));
    assert!(token.definition.printed_colors().contains(&Color::Blue));
}
