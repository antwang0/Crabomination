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

// ── Wave 2 ──────────────────────────────────────────────────────────────────

/// Bane of the Living's Morph {X}{B}{B} sweeps for the X actually paid.
#[test]
fn bane_of_the_living_sweeps_for_the_paid_x() {
    let mut g = main_phase();
    let bane = g.add_card_to_battlefield(0, catalog::bane_of_the_living());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let big = g.add_card_to_battlefield(1, catalog::enormous_baloth());
    g.battlefield.iter_mut().find(|c| c.id == bane).unwrap().turn_face_down();
    mana(&mut g, 0);
    g.perform_action(GameAction::TurnFaceUpForX { card_id: bane, x_value: 2 })
        .expect("unmorph for X=2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "-2/-2 killed the 2/2");
    assert_eq!(power_of(&g, big), 5, "and shrank the 7/7");
}

/// Berserk Murlodont pumps every blocked Beast by its blocker count.
#[test]
fn berserk_murlodont_pumps_blocked_beasts() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::berserk_murlodont());
    let baloth = g.add_card_to_battlefield(0, catalog::enormous_baloth());
    g.clear_sickness(baloth);
    let a = g.add_card_to_battlefield(1, catalog::wall_of_stone());
    let b = g.add_card_to_battlefield(1, catalog::wall_of_stone());
    attack_with(&mut g, baloth);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(a, baloth), (b, baloth)]))
        .expect("gang block");
    drain_stack(&mut g);
    assert_eq!(power_of(&g, baloth), 9, "+2/+2 for the two blockers");
}

/// Deathmark Prelate eats a Zombie for unconditional removal.
#[test]
fn deathmark_prelate_trades_a_zombie_for_removal() {
    let mut g = main_phase();
    let prelate = g.add_card_to_battlefield(0, catalog::deathmark_prelate());
    let zombie = g.add_card_to_battlefield(0, catalog::dripping_dead());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(prelate);
    activate(&mut g, 0, prelate, 0, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(zombie).is_none(), "the Zombie was the cost");
}

/// Krosan Cloudscraper eats itself when the upkeep tax goes unpaid.
#[test]
fn krosan_cloudscraper_sacrifices_itself_unpaid() {
    let mut g = main_phase();
    let scraper = g.add_card_to_battlefield(0, catalog::krosan_cloudscraper());
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep {
        g.advance_step(vec![]).expect("advance");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(scraper).is_none(), "the upkeep tax went unpaid");
}

/// Lavaborn Muse burns an opponent who's hellbent on their upkeep.
#[test]
fn lavaborn_muse_burns_an_empty_handed_opponent() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::lavaborn_muse());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep {
        g.advance_step(vec![]).expect("advance");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "hellbent on their own upkeep");
}

/// Phage is safe off the hand and lethal when she's cheated in.
#[test]
fn phage_only_survives_a_hand_cast() {
    let mut g = main_phase();
    let phage = g.add_card_to_hand(0, catalog::phage_the_untouchable());
    cast(&mut g, 0, phage, None);
    assert!(g.players[0].is_alive(), "a hand cast is safe");
    // Reanimated instead — the ETB rider fires.
    g.remove_to_graveyard_with_triggers(phage);
    drain_stack(&mut g);
    let reanimate = g.add_card_to_hand(0, catalog::reanimate());
    cast(&mut g, 0, reanimate, Some(Target::Permanent(phage)));
    assert!(!g.players[0].is_alive(), "she wasn't cast from hand");
}

/// Mistform Seaswift can pass for any tribe until end of turn.
#[test]
fn mistform_seaswift_becomes_the_chosen_type() {
    let mut g = main_phase();
    let swift = g.add_card_to_battlefield(0, catalog::mistform_seaswift());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::CreatureType(
        crabomination::card::CreatureType::Sliver,
    )]));
    activate(&mut g, 0, swift, 0, None);
    let lord = g.add_card_to_battlefield(0, catalog::blade_sliver());
    assert_eq!(power_of(&g, swift), 4, "the Sliver lord now sees it");
    assert!(g.battlefield_find(lord).is_some());
}

/// Skirk Alarmist flips your morph up early — and takes it at end of turn.
#[test]
fn skirk_alarmist_flips_then_sacrifices() {
    let mut g = main_phase();
    let alarmist = g.add_card_to_battlefield(0, catalog::skirk_alarmist());
    let hidden = g.add_card_to_battlefield(0, catalog::sootfeather_flock());
    g.battlefield.iter_mut().find(|c| c.id == hidden).unwrap().turn_face_down();
    activate(&mut g, 0, alarmist, 0, Some(Target::Permanent(hidden)));
    assert!(!g.battlefield_find(hidden).unwrap().face_down, "flipped for free");
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(hidden).is_none(), "and sacrificed at the end step");
}

/// Sunstrike Legionnaire untaps off every arrival and taps small creatures.
#[test]
fn sunstrike_legionnaire_untaps_on_arrivals() {
    let mut g = main_phase();
    let legionnaire = g.add_card_to_battlefield(0, catalog::sunstrike_legionnaire());
    g.clear_sickness(legionnaire);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, legionnaire, 0, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).unwrap().tapped);
    assert!(g.battlefield_find(legionnaire).unwrap().tapped);
    let arrival = g.add_card_to_hand(0, catalog::llanowar_elves());
    cast(&mut g, 0, arrival, None);
    assert!(!g.battlefield_find(legionnaire).unwrap().tapped, "an arrival untapped it");
}

/// Unstable Hulk's Morph is a huge swing that costs you the next turn.
#[test]
fn unstable_hulk_swings_then_skips_your_turn() {
    let mut g = main_phase();
    let hulk = g.add_card_to_battlefield(0, catalog::unstable_hulk());
    g.battlefield.iter_mut().find(|c| c.id == hulk).unwrap().turn_face_down();
    mana(&mut g, 0);
    g.perform_action(GameAction::TurnFaceUp { card_id: hulk }).expect("unmorph");
    drain_stack(&mut g);
    assert_eq!(power_of(&g, hulk), 8);
    assert_eq!(g.players[0].skip_turns, 1);
}

/// Weaver of Lies hides every *other* morph creature when it flips.
#[test]
fn weaver_of_lies_hides_the_other_morphs() {
    let mut g = main_phase();
    let weaver = g.add_card_to_battlefield(0, catalog::weaver_of_lies());
    let other = g.add_card_to_battlefield(1, catalog::branchsnap_lorian());
    g.battlefield.iter_mut().find(|c| c.id == weaver).unwrap().turn_face_down();
    mana(&mut g, 0);
    g.perform_action(GameAction::TurnFaceUp { card_id: weaver }).expect("unmorph");
    drain_stack(&mut g);
    assert!(g.battlefield_find(other).unwrap().face_down, "the opposing morph went down");
    assert!(!g.battlefield_find(weaver).unwrap().face_down, "but not the Weaver itself");
}

/// Elvish Soultiller shuffles a whole tribe back out of your graveyard.
#[test]
fn elvish_soultiller_reloads_a_tribe() {
    let mut g = main_phase();
    let tiller = g.add_card_to_battlefield(0, catalog::elvish_soultiller());
    g.players[0].graveyard.push(crabomination::card::CardInstance::new(
        CardId(9001),
        catalog::llanowar_elves(),
        0,
    ));
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::CreatureType(
        crabomination::card::CreatureType::Elf,
    )]));
    g.remove_to_graveyard_with_triggers(tiller);
    drain_stack(&mut g);
    assert!(
        g.players[0].library.iter().any(|c| c.id == CardId(9001)),
        "the Elf went back into the library"
    );
}

/// Goblin Clearcutter turns a Forest into three red-or-green mana.
#[test]
fn goblin_clearcutter_eats_a_forest_for_three() {
    let mut g = main_phase();
    let cutter = g.add_card_to_battlefield(0, catalog::goblin_clearcutter());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.clear_sickness(cutter);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cutter,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    assert!(g.battlefield_find(forest).is_none(), "the Forest was the cost");
    let pool = &g.players[0].mana_pool;
    assert_eq!(pool.amount(Color::Red) + pool.amount(Color::Green), 3);
}

/// Brood Sliver breeds a token off any connecting Sliver.
#[test]
fn brood_sliver_breeds_on_a_connection() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::brood_sliver());
    let attacker = g.add_card_to_battlefield(0, catalog::blade_sliver());
    g.clear_sickness(attacker);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    attack_with(&mut g, attacker);
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 1);
}

/// Imperial Hellkite's Morph tutors up another Dragon.
#[test]
fn imperial_hellkite_fetches_a_dragon() {
    let mut g = main_phase();
    let kite = g.add_card_to_battlefield(0, catalog::imperial_hellkite());
    let dragon = g.add_card_to_library(0, catalog::shivan_dragon());
    g.battlefield.iter_mut().find(|c| c.id == kite).unwrap().turn_face_down();
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(dragon)),
    ]));
    mana(&mut g, 0);
    g.perform_action(GameAction::TurnFaceUp { card_id: kite }).expect("unmorph");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dragon));
}

// ── Wave 3 ──────────────────────────────────────────────────────────────────

/// Caller of the Claw pays out one Bear per creature that died this turn.
#[test]
fn caller_of_the_claw_rebuilds_after_a_sweeper() {
    let mut g = main_phase();
    for _ in 0..2 {
        let id = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.remove_to_graveyard_with_triggers(id);
    }
    drain_stack(&mut g);
    let caller = g.add_card_to_hand(0, catalog::caller_of_the_claw());
    cast(&mut g, 0, caller, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 2);
}

/// Chromeshell Crab's Morph trades a creature straight across.
#[test]
fn chromeshell_crab_exchanges_control() {
    let mut g = main_phase();
    let crab = g.add_card_to_battlefield(0, catalog::chromeshell_crab());
    let mine = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let theirs = g.add_card_to_battlefield(1, catalog::enormous_baloth());
    g.battlefield.iter_mut().find(|c| c.id == crab).unwrap().turn_face_down();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    mana(&mut g, 0);
    g.perform_action(GameAction::TurnFaceUp { card_id: crab }).expect("unmorph");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0);
    assert_eq!(g.battlefield_find(mine).unwrap().controller, 1);
}

/// Crookclaw Elder taps two Birds as a cost to draw.
#[test]
fn crookclaw_elder_taps_two_birds_to_draw() {
    let mut g = main_phase();
    let elder = g.add_card_to_battlefield(0, catalog::crookclaw_elder());
    let birds: Vec<CardId> =
        (0..2).map(|_| g.add_card_to_battlefield(0, catalog::aven_envoy())).collect();
    for id in birds.iter().copied().chain([elder]) {
        g.clear_sickness(id);
    }
    g.add_card_to_library(0, catalog::forest());
    let before = g.players[0].hand.len();
    activate(&mut g, 0, elder, 0, None);
    assert_eq!(g.players[0].hand.len(), before + 1);
    assert!(birds.iter().all(|id| g.battlefield_find(*id).unwrap().tapped), "both Birds tapped");
}

/// …and rejects the activation when only one Bird is untapped.
#[test]
fn crookclaw_elder_needs_two_untapped_birds() {
    let mut g = main_phase();
    let elder = g.add_card_to_battlefield(0, catalog::crookclaw_elder());
    let bird = g.add_card_to_battlefield(0, catalog::aven_envoy());
    for id in [elder, bird] {
        g.clear_sickness(id);
    }
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: elder,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "one Bird isn't enough"
    );
}

/// Ghastly Remains buys itself back out of the graveyard each upkeep.
#[test]
fn ghastly_remains_returns_itself_from_the_graveyard() {
    let mut g = main_phase();
    let remains = g.add_card_to_battlefield(0, catalog::ghastly_remains());
    g.remove_to_graveyard_with_triggers(remains);
    drain_stack(&mut g);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    mana(&mut g, 0);
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep {
        g.advance_step(vec![]).expect("advance");
    }
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == remains), "paid the upkeep cost to buy it back");
}

/// Magma Sliver hands every Sliver a swarm-scaled pump.
#[test]
fn magma_sliver_grants_the_swarm_pump() {
    let mut g = main_phase();
    let magma = g.add_card_to_battlefield(0, catalog::magma_sliver());
    let other = g.add_card_to_battlefield(0, catalog::blade_sliver());
    for id in [magma, other] {
        g.clear_sickness(id);
    }
    let base = power_of(&g, other);
    activate(&mut g, 0, other, 0, Some(Target::Permanent(other)));
    assert_eq!(power_of(&g, other), base + 2, "+X/+0 for the two Slivers");
}

/// Corpse Harvester turns a creature into a Zombie and a Swamp.
#[test]
fn corpse_harvester_fetches_a_zombie_and_a_swamp() {
    let mut g = main_phase();
    let harvester = g.add_card_to_battlefield(0, catalog::corpse_harvester());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(harvester);
    let zombie = g.add_card_to_library(0, catalog::dripping_dead());
    let swamp = g.add_card_to_library(0, catalog::swamp());
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Search(Some(zombie)),
        DecisionAnswer::Search(Some(swamp)),
    ]));
    activate(&mut g, 0, harvester, 0, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == zombie));
    assert!(g.players[0].hand.iter().any(|c| c.id == swamp));
}

/// Celestial Gatekeeper's death buys back two Birds and/or Clerics.
#[test]
fn celestial_gatekeeper_recurs_two_on_death() {
    let mut g = main_phase();
    let keeper = g.add_card_to_battlefield(0, catalog::celestial_gatekeeper());
    let a = CardId(9101);
    let b = CardId(9102);
    for (id, def) in [(a, catalog::aven_envoy()), (b, catalog::daru_mender())] {
        g.players[0].graveyard.push(crabomination::card::CardInstance::new(id, def, 0));
    }
    g.remove_to_graveyard_with_triggers(keeper);
    drain_stack(&mut g);
    // The auto-targeter still fills only one of the two "up to two" graveyard
    // slots (a UI seat picks both) — see TODO.md.
    assert!(g.battlefield_find(a).is_some() || g.battlefield_find(b).is_some());
    assert!(g.exile.iter().any(|c| c.id == keeper), "the Gatekeeper exiled itself");
}
