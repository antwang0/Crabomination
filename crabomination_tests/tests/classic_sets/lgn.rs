//! Legions (LGN) — Amplify, Morph turn-up triggers, Provoke, the cycling
//! creatures and the Sliver lords (`catalog::sets::lgn`).

use crabomination::card::{CardDefinition, CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
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

fn cycle(g: &mut GameState, seat: usize, id: CardId) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None }).expect("cycle");
    drain_stack(g);
}

fn attack_with(g: &mut GameState, attacker: CardId) {
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(g);
}

fn power_of(g: &GameState, id: CardId) -> i32 {
    g.computed_permanent(id).expect("computed").power
}

// ── Vanilla / keyword bodies ────────────────────────────────────────────────

/// The plain bodies: printed stats and keywords, one table over the catalog.
#[test]
fn lgn_vanilla_bodies_have_their_printed_stats() {
    let rows: &[(fn() -> CardDefinition, i32, i32, &[Keyword])] = &[
        (
            catalog::akroma_angel_of_wrath,
            6,
            6,
            &[
                Keyword::Flying,
                Keyword::FirstStrike,
                Keyword::Vigilance,
                Keyword::Trample,
                Keyword::Haste,
                Keyword::Protection(Color::Black),
                Keyword::Protection(Color::Red),
            ],
        ),
        (catalog::aven_envoy, 0, 2, &[Keyword::Flying]),
        (catalog::covert_operative, 3, 2, &[Keyword::Unblockable]),
        (catalog::defiant_elf, 1, 1, &[Keyword::Trample]),
        (catalog::enormous_baloth, 7, 7, &[]),
        (catalog::needleshot_gourna, 3, 6, &[Keyword::Reach]),
        (catalog::mistform_ultimus, 3, 3, &[Keyword::Changeling]),
        (catalog::rockshard_elemental, 4, 3, &[Keyword::DoubleStrike]),
        (catalog::sootfeather_flock, 3, 2, &[Keyword::Flying]),
        (catalog::branchsnap_lorian, 4, 1, &[Keyword::Trample]),
        (catalog::krosan_vorine, 3, 2, &[Keyword::CantBeBlockedByMoreThanOne]),
        (catalog::dripping_dead, 4, 1, &[Keyword::CantBlock]),
    ];
    for (make, p, t, kws) in rows {
        let def = make();
        assert_eq!((def.power, def.toughness), (*p, *t), "{}", def.name);
        for kw in *kws {
            assert!(def.keywords.contains(kw), "{} missing {kw:?}", def.name);
        }
    }
}

// ── Amplify (CR 702.38) ─────────────────────────────────────────────────────

/// Amplify 1 counts every matching card in hand as it enters; Aven Warhawk
/// reads two tribes at once.
#[test]
fn amplify_counts_the_revealed_hand() {
    let mut g = main_phase();
    g.add_card_to_hand(0, catalog::aven_envoy()); // Bird Soldier
    g.add_card_to_hand(0, catalog::zombie_brute()); // Zombie — not counted
    let hawk = g.add_card_to_hand(0, catalog::aven_warhawk());
    cast(&mut g, 0, hawk, None);
    assert_eq!(g.battlefield_find(hawk).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(power_of(&g, hawk), 3);
}

/// Zombie Brute's Amplify 1 stacks over several Zombies in hand.
#[test]
fn amplify_scales_with_the_matching_hand() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::gempalm_polluter());
    }
    let brute = g.add_card_to_hand(0, catalog::zombie_brute());
    cast(&mut g, 0, brute, None);
    assert_eq!(g.battlefield_find(brute).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}

// ── Morph turn-up triggers (CR 708.8) ───────────────────────────────────────

/// Aphetto Exterminator's flip is a -3/-3.
#[test]
fn aphetto_exterminator_shrinks_on_the_flip() {
    let mut g = main_phase();
    let exterminator = g.add_card_to_battlefield(0, catalog::aphetto_exterminator());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == exterminator).unwrap().turn_face_down();
    mana(&mut g, 0);
    g.perform_action(GameAction::TurnFaceUp { card_id: exterminator }).expect("unmorph");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "the 2/2 died to -3/-3");
}

/// Liege of the Axe untaps itself when it flips.
#[test]
fn liege_of_the_axe_untaps_on_the_flip() {
    let mut g = main_phase();
    let liege = g.add_card_to_battlefield(0, catalog::liege_of_the_axe());
    {
        let c = g.battlefield.iter_mut().find(|c| c.id == liege).unwrap();
        c.turn_face_down();
        c.tapped = true;
    }
    mana(&mut g, 0);
    g.perform_action(GameAction::TurnFaceUp { card_id: liege }).expect("unmorph");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(liege).unwrap().tapped);
}

/// Warbreak Trumpeter's Morph {X}{X}{R} mints X Goblins.
#[test]
fn warbreak_trumpeter_mints_x_goblins() {
    let mut g = main_phase();
    let trumpeter = g.add_card_to_battlefield(0, catalog::warbreak_trumpeter());
    g.battlefield.iter_mut().find(|c| c.id == trumpeter).unwrap().turn_face_down();
    mana(&mut g, 0);
    g.perform_action(GameAction::TurnFaceUpForX { card_id: trumpeter, x_value: 3 })
        .expect("unmorph for X=3");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 3);
}

// ── Provoke (CR 702.39) ─────────────────────────────────────────────────────

/// Provoke untaps a defender and forces it to block.
#[test]
fn provoke_untaps_and_forces_the_block() {
    let mut g = main_phase();
    let grappler = g.add_card_to_battlefield(0, catalog::goblin_grappler());
    g.clear_sickness(grappler);
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_stone());
    g.battlefield.iter_mut().find(|c| c.id == wall).unwrap().tapped = true;
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(wall)),
    ]));
    attack_with(&mut g, grappler);
    let blocker = g.battlefield_find(wall).unwrap();
    assert!(!blocker.tapped, "provoke untapped it");
    assert_eq!(blocker.must_block, Some(grappler), "and it must block the provoker");
}

// ── Cycling riders ──────────────────────────────────────────────────────────

/// Gempalm Strider's cycle pumps every Elf.
#[test]
fn gempalm_strider_cycle_pumps_the_elves() {
    let mut g = main_phase();
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let strider = g.add_card_to_hand(0, catalog::gempalm_strider());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    cycle(&mut g, 0, strider);
    assert_eq!(power_of(&g, elf), 3, "1/1 Elf became a 3/3");
}

/// Gempalm Polluter's cycle drains for the Zombie count.
#[test]
fn gempalm_polluter_cycle_drains_for_the_zombies() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::zombie_brute());
    g.add_card_to_battlefield(1, catalog::dripping_dead());
    let polluter = g.add_card_to_hand(0, catalog::gempalm_polluter());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    cycle(&mut g, 0, polluter);
    assert_eq!(g.players[1].life, 18, "two Zombies on the battlefield");
}

/// Stoic Champion grows off anyone's cycle.
#[test]
fn stoic_champion_grows_on_every_cycle() {
    let mut g = main_phase();
    let champ = g.add_card_to_battlefield(0, catalog::stoic_champion());
    let hundroog = g.add_card_to_hand(1, catalog::hundroog());
    cycle(&mut g, 1, hundroog);
    assert_eq!(power_of(&g, champ), 4, "the opponent's cycle still pumps it");
}

// ── Conditional statics ─────────────────────────────────────────────────────

/// Cloudreach Cavalry is a 3/3 flier only while you control a Bird.
#[test]
fn cloudreach_cavalry_needs_a_bird() {
    let mut g = main_phase();
    let cavalry = g.add_card_to_battlefield(0, catalog::cloudreach_cavalry());
    assert_eq!(power_of(&g, cavalry), 1);
    let bird = g.add_card_to_battlefield(0, catalog::aven_envoy());
    assert_eq!(power_of(&g, cavalry), 3);
    assert!(g.computed_permanent(cavalry).unwrap().keywords.contains(&Keyword::Flying));
    g.remove_from_battlefield_to_graveyard_raw(bird);
    assert_eq!(power_of(&g, cavalry), 1, "the bonus is conditional");
}

/// Vexing Beetle is uncounterable and swells while the other side is empty.
#[test]
fn vexing_beetle_is_uncounterable_and_scales() {
    let mut g = main_phase();
    let beetle = g.add_card_to_hand(0, catalog::vexing_beetle());
    let counter = g.add_card_to_hand(1, catalog::counterspell());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: beetle,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: counter,
        target: Some(Target::Permanent(beetle)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("the counter may be cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(beetle).is_some(), "but it can't be countered");
    assert_eq!(power_of(&g, beetle), 6, "no opposing creature");
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert_eq!(power_of(&g, beetle), 3);
}

/// Primal Whisperer scales with every face-down creature in play.
#[test]
fn primal_whisperer_scales_with_face_down_creatures() {
    let mut g = main_phase();
    let whisperer = g.add_card_to_battlefield(0, catalog::primal_whisperer());
    assert_eq!(power_of(&g, whisperer), 2);
    let hidden = g.add_card_to_battlefield(1, catalog::sootfeather_flock());
    g.battlefield.iter_mut().find(|c| c.id == hidden).unwrap().turn_face_down();
    assert_eq!(power_of(&g, whisperer), 4, "+2/+2 per face-down creature");
}

// ── Sliver lords ────────────────────────────────────────────────────────────

/// Blade Sliver pumps every Sliver, including the opponent's.
#[test]
fn blade_sliver_pumps_all_slivers() {
    let mut g = main_phase();
    let blade = g.add_card_to_battlefield(0, catalog::blade_sliver());
    let theirs = g.add_card_to_battlefield(1, catalog::root_sliver());
    assert_eq!(power_of(&g, blade), 3);
    assert_eq!(power_of(&g, theirs), 3, "all Slivers, not just yours");
}

/// Root Sliver protects every player's Sliver spells.
#[test]
fn root_sliver_makes_sliver_spells_uncounterable() {
    let mut g = main_phase();
    // The protection is symmetric: an opponent's Root Sliver still covers it.
    g.add_card_to_battlefield(1, catalog::root_sliver());
    let sliver = g.add_card_to_hand(0, catalog::blade_sliver());
    let counter = g.add_card_to_hand(1, catalog::counterspell());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: sliver,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: counter,
        target: Some(Target::Permanent(sliver)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("the counter may be cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(sliver).is_some(), "the Sliver spell resolved anyway");
}

/// Spectral Sliver grants every Sliver a firebreathing pump.
#[test]
fn spectral_sliver_grants_the_pump_to_every_sliver() {
    let mut g = main_phase();
    let spectral = g.add_card_to_battlefield(0, catalog::spectral_sliver());
    let other = g.add_card_to_battlefield(0, catalog::blade_sliver());
    let base = power_of(&g, other);
    activate(&mut g, 0, other, 0, None);
    assert_eq!(power_of(&g, other), base + 1);
    assert!(g.battlefield_find(spectral).is_some());
}

/// Toxin Sliver kills whatever any Sliver connects with in combat.
#[test]
fn toxin_sliver_kills_what_slivers_hit() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::toxin_sliver());
    let attacker = g.add_card_to_battlefield(0, catalog::blade_sliver());
    g.clear_sickness(attacker);
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_stone());
    attack_with(&mut g, attacker);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, attacker)])).expect("block");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.battlefield_find(wall).is_none(), "the 0/5 Wall died to the toxin");
}

// ── Utility bodies ──────────────────────────────────────────────────────────

/// Wirewood Channeler taps for one color, Elf-many.
#[test]
fn wirewood_channeler_taps_for_the_elf_count() {
    let mut g = main_phase();
    let channeler = g.add_card_to_battlefield(0, catalog::wirewood_channeler());
    g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.clear_sickness(channeler);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Color(Color::Green)]));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: channeler,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "Channeler + Llanowar Elves");
}

/// Riptide Director draws one per Wizard you control.
#[test]
fn riptide_director_draws_per_wizard() {
    let mut g = main_phase();
    let director = g.add_card_to_battlefield(0, catalog::riptide_director());
    g.add_card_to_battlefield(0, catalog::merchant_of_secrets());
    g.clear_sickness(director);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let before = g.players[0].hand.len();
    activate(&mut g, 0, director, 0, None);
    assert_eq!(g.players[0].hand.len(), before + 2, "two Wizards");
}

/// Noxious Ghoul shrinks everything that isn't a Zombie when one arrives.
#[test]
fn noxious_ghoul_sweeps_the_non_zombies() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::noxious_ghoul());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let zombie = g.add_card_to_hand(0, catalog::dripping_dead());
    cast(&mut g, 0, zombie, None);
    assert_eq!(power_of(&g, bear), 1, "-1/-1 for the arriving Zombie");
}

/// Graveborn Muse draws — and drains — for your Zombie count each upkeep.
#[test]
fn graveborn_muse_draws_and_drains() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::graveborn_muse());
    g.add_card_to_battlefield(0, catalog::dripping_dead());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let (hand, life) = (g.players[0].hand.len(), g.players[0].life);
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep {
        g.advance_step(vec![]).expect("advance");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2, "the Muse is a Zombie too");
    assert_eq!(g.players[0].life, life - 2);
}

/// Havoc Demon's death is a -5/-5 sweeper.
#[test]
fn havoc_demon_sweeps_when_it_dies() {
    let mut g = main_phase();
    let demon = g.add_card_to_battlefield(0, catalog::havoc_demon());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.remove_to_graveyard_with_triggers(demon);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "-5/-5 killed the 2/2");
}

/// Clickslither eats a Goblin for +2/+2 and trample.
#[test]
fn clickslither_eats_a_goblin() {
    let mut g = main_phase();
    let click = g.add_card_to_battlefield(0, catalog::clickslither());
    let goblin = g.add_card_to_battlefield(0, catalog::goblin_grappler());
    activate(&mut g, 0, click, 0, None);
    assert!(g.battlefield_find(goblin).is_none(), "the Goblin was the cost");
    assert_eq!(power_of(&g, click), 5);
    assert!(g.computed_permanent(click).unwrap().keywords.contains(&Keyword::Trample));
}

/// Goblin Firebug takes a land with it when it leaves.
#[test]
fn goblin_firebug_costs_a_land_on_the_way_out() {
    let mut g = main_phase();
    let bug = g.add_card_to_battlefield(0, catalog::goblin_firebug());
    g.add_card_to_battlefield(0, catalog::forest());
    g.remove_to_graveyard_with_triggers(bug);
    drain_stack(&mut g);
    assert!(
        !g.battlefield.iter().any(|c| c.definition.is_land() && c.controller == 0),
        "the land was sacrificed"
    );
}

/// Wall of Hope turns every point it takes into life.
#[test]
fn wall_of_hope_converts_damage_to_life() {
    let mut g = main_phase();
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_hope());
    let mut events = vec![];
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(wall), 2, None, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 22);
}

/// Mistform Ultimus is every creature type at once.
#[test]
fn mistform_ultimus_is_every_type() {
    let mut g = main_phase();
    let ultimus = g.add_card_to_battlefield(0, catalog::mistform_ultimus());
    let lord = g.add_card_to_battlefield(0, catalog::blade_sliver());
    assert_eq!(power_of(&g, ultimus), 4, "the Sliver lord sees it as a Sliver");
    assert!(g.battlefield_find(lord).is_some());
}

/// Totem Speaker pays out on every Beast that shows up.
#[test]
fn totem_speaker_gains_on_a_beast() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::totem_speaker());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let baloth = g.add_card_to_hand(0, catalog::enormous_baloth());
    cast(&mut g, 0, baloth, None);
    assert_eq!(g.players[0].life, 23);
}

/// Dreamborn Muse mills each player their own hand size on their upkeep.
#[test]
fn dreamborn_muse_mills_the_upkeep_players_hand_size() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::dreamborn_muse());
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::forest());
        g.add_card_to_library(1, catalog::forest());
    }
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep {
        g.advance_step(vec![]).expect("advance");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 3, "milled their hand size");
}

/// The {7} invokers: a late-game mana sink apiece.
#[test]
fn invokers_pay_off_at_seven() {
    let mut g = main_phase();
    let stonewood = g.add_card_to_battlefield(0, catalog::stonewood_invoker());
    g.clear_sickness(stonewood);
    activate(&mut g, 0, stonewood, 0, None);
    assert_eq!(power_of(&g, stonewood), 7);
    let starlight = g.add_card_to_battlefield(0, catalog::starlight_invoker());
    g.clear_sickness(starlight);
    activate(&mut g, 0, starlight, 0, None);
    assert_eq!(g.players[0].life, 25);
}
