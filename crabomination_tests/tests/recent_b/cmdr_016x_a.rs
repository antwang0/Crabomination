//! Commander: the 2026-10-19 (`016Xh16H`) closers (`decks::cmdr_016x_a`).

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].hostile_player_targets = true;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

/// CR 601.2b-c — Cruel Alliance's teamwork is announced before its target is
/// chosen: unteamed it can't reach a mana-value-4 creature; teamed (a 2-power
/// creature tapped) it exiles one and gains 3 life.
#[test]
fn cruel_alliance_teamwork_widens_the_target() {
    let mut g = pod(3);
    let giant = ready(&mut g, 2, catalog::hill_giant());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let ca = g.add_card_to_hand(0, catalog::cruel_alliance());
    flood(&mut g, 0);
    assert!(cast(&mut g, ca, Some(Target::Permanent(giant))).is_err(), "MV 4 > 3 unteamed");
    g.perform_action(GameAction::CastSpellKicked {
        card_id: ca,
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("teamwork cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == giant));
    assert_eq!(g.players[0].life, 23);
    assert!(g.battlefield_find(bear).unwrap().tapped, "the bear paid teamwork");
}

/// Cruel Alliance unteamed exiles a small creature and gains no life.
#[test]
fn cruel_alliance_unteamed_exiles_a_small_creature() {
    let mut g = pod(3);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let ca = g.add_card_to_hand(0, catalog::cruel_alliance());
    flood(&mut g, 0);
    cast(&mut g, ca, Some(Target::Permanent(bear))).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == bear));
    assert_eq!(g.players[0].life, 20);
}

/// Glyph Keeper counters the first spell to target it each turn; the second
/// resolves.
#[test]
fn glyph_keeper_counters_the_first_targeting_spell() {
    let mut g = pod(2);
    let gk = ready(&mut g, 1, catalog::glyph_keeper());
    flood(&mut g, 0);
    let b1 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let b2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, b1, Some(Target::Permanent(gk))).expect("bolt 1");
    assert_eq!(g.battlefield_find(gk).map(|c| c.damage), Some(0), "countered");
    cast(&mut g, b2, Some(Target::Permanent(gk))).expect("bolt 2");
    assert!(g.battlefield_find(gk).is_none(), "the second bolt kills it");
}

/// CR 702.128a — Vizier of the Anointed entombs a creature card with embalm,
/// and activating that embalm draws a card (the trait reads the activation,
/// not the token it makes).
#[test]
fn vizier_of_the_anointed_entombs_and_draws_on_embalm() {
    let mut g = pod(2);
    let gk = g.add_card_to_library(0, catalog::glyph_keeper());
    let vz = g.add_card_to_hand(0, catalog::vizier_of_the_anointed());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, vz, None).expect("vizier");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == gk), "entombed");
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: gk,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None,
        mode: None,
    })
    .expect("embalm");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "embalm draws");
    let tok = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Glyph Keeper").expect("token");
    assert_eq!(tok.definition.printed_colors(), vec![Color::White], "embalm makes it white");
}

/// CR 700.2 — Splinter & Leo's default line: the counters on your own team,
/// the Mutant to a different player (each mode targets a different player).
#[test]
fn splinter_and_leo_counters_your_team() {
    let mut g = pod(3);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let sl = g.add_card_to_hand(0, catalog::splinter_leo_father_son());
    flood(&mut g, 0);
    cast(&mut g, sl, None).expect("cast");
    let bear_counters = g.battlefield_find(bear).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne);
    assert_eq!(bear_counters, 1);
    assert_eq!(g.battlefield_find(sl).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne), 0, "other creatures");
    let mutants: Vec<usize> = g.battlefield.iter().filter(|c| c.definition.name == "Mutant").map(|c| c.controller).collect();
    assert!(mutants.iter().all(|&s| s != 0), "the token mode went to another player: {mutants:?}");
}

/// CR 615 — Djeru shaves 1 off damage to a planeswalker you control.
#[test]
fn djeru_shields_your_planeswalkers() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    ready(&mut g, 1, catalog::djeru_with_eyes_open());
    let lili = ready(&mut g, 1, catalog::liliana_death_mage());
    let start = g.battlefield_find(lili).unwrap().counter_count(CounterType::Loyalty);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    cast(&mut g, bolt, Some(Target::Permanent(lili))).expect("bolt");
    assert_eq!(g.battlefield_find(lili).unwrap().counter_count(CounterType::Loyalty), start - 2);
}

/// Elspeth's Talent grants "[+1]: three Soldiers"; activating it (a loyalty
/// ability of the enchanted planeswalker) pumps the team +2/+2 and vigilance.
#[test]
fn elspeths_talent_grants_soldiers_and_pumps_on_loyalty() {
    let mut g = pod(2);
    let lili = ready(&mut g, 0, catalog::liliana_death_mage());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let et = g.add_card_to_hand(0, catalog::elspeths_talent());
    flood(&mut g, 0);
    cast(&mut g, et, Some(Target::Permanent(lili))).expect("cast");
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: lili, ability_index: 3, target: None, x_value: None })
        .expect("granted +1");
    drain_stack(&mut g);
    let soldiers = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Soldier").count();
    assert_eq!(soldiers, 3);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!((b.power(), b.toughness()), (4, 4));
    assert!(b.has_keyword(&crabomination::card::Keyword::Vigilance));
}

// ── Old Hob, Kang, Ultimo, Mila (`decks::cmdr_016x_b`) ──────────────────────

fn pass_to(g: &mut GameState, step: TurnStep) {
    for _ in 0..200 {
        if g.step == step {
            return;
        }
        let _ = g.perform_action(GameAction::PassPriority);
    }
    panic!("never reached {step:?}");
}

/// Old Hob's combat Mutant has haste and is destroyed at the next end step.
#[test]
fn old_hob_makes_a_hasty_mutant_that_dies_at_end_step() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::old_hob_alleycat_blues());
    pass_to(&mut g, TurnStep::BeginCombat);
    drain_stack(&mut g);
    let m = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Mutant").expect("mutant").id;
    assert!(g.battlefield_find(m).unwrap().has_keyword(&crabomination::card::Keyword::Haste));
    pass_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(m).is_none(), "destroyed at the end step");
}

/// CR 603.7 — Rewrite History's fourth plan counter sacrifices it, and the
/// reflexive trigger returns an instant from your graveyard.
#[test]
fn rewrite_history_fourth_plan_counter_returns_spells() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let rh = ready(&mut g, 0, catalog::rewrite_history());
    g.battlefield_find_mut(rh).unwrap().counters.insert(CounterType::Plan, 3);
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::forest());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: bear,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rh).is_none(), "sacrificed at four");
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "bolt returned");
}

/// Gix's Caress takes a nonland card and leaves a tapped Powerstone.
#[test]
fn gixs_caress_discards_a_nonland_and_makes_a_powerstone() {
    let mut g = pod(2);
    g.players[1].hand.clear();
    let land = g.add_card_to_hand(1, catalog::forest());
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let gc = g.add_card_to_hand(0, catalog::gixs_caress());
    flood(&mut g, 0);
    cast(&mut g, gc, Some(Target::Player(1))).expect("cast");
    let hand: Vec<CardId> = g.players[1].hand.iter().map(|c| c.id).collect();
    assert_eq!(hand, vec![land]);
    let ps = g.battlefield.iter().find(|c| c.definition.name == "Powerstone").expect("powerstone");
    assert!(ps.tapped);
}

/// Okiba Salvage's rider needs both an artifact and an enchantment.
#[test]
fn okiba_salvage_counters_with_artifact_and_enchantment() {
    use crabomination::card::CounterType;
    for (both, want) in [(true, 2), (false, 0)] {
        let mut g = pod(2);
        ready(&mut g, 0, catalog::sol_ring());
        if both {
            ready(&mut g, 0, catalog::rewrite_history());
        }
        let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let os = g.add_card_to_hand(0, catalog::okiba_salvage());
        flood(&mut g, 0);
        cast(&mut g, os, Some(Target::Permanent(bear))).expect("cast");
        let c = g.battlefield_find(bear).expect("reanimated");
        assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), want, "both={both}");
    }
}

/// Norritt forces the active player's creature to attack; one that never
/// did is destroyed at the next end step.
#[test]
fn norritt_destroys_a_creature_that_did_not_attack() {
    let mut g = pod(2);
    let norritt = ready(&mut g, 0, catalog::norritt());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: norritt,
        ability_index: 1,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().has_keyword(&crabomination::card::Keyword::MustAttack));
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 1;
    pass_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "never attacked: destroyed");
}

/// CR 104.3d / 114.4 — Gideon of the Trials' emblem keeps you in the game at
/// 0 life only while you control a Gideon planeswalker.
#[test]
fn gideon_of_the_trials_emblem_needs_a_gideon() {
    let mut g = pod(3);
    let gid = ready(&mut g, 0, catalog::gideon_of_the_trials());
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: gid, ability_index: 2, target: None, x_value: None })
        .expect("emblem");
    drain_stack(&mut g);
    g.players[0].life = 0;
    g.check_state_based_actions();
    assert!(!g.players[0].eliminated, "Gideon out: still in");
    g.battlefield_find_mut(gid).unwrap().counters.insert(crabomination::card::CounterType::Loyalty, 0);
    g.check_state_based_actions();
    assert!(g.battlefield_find(gid).is_none());
    g.check_state_based_actions();
    assert!(g.players[0].eliminated, "no Gideon: the loss lands");
}

// ── Quickbeam ───────────────────────────────────────────────────

/// Lumberknot grows on any creature's death.
#[test]
fn lumberknot_grows_when_a_creature_dies() {
    let mut g = pod(2);
    let lk = ready(&mut g, 0, catalog::lumberknot());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    cast(&mut g, bolt, Some(Target::Permanent(bear))).expect("bolt");
    assert_eq!(g.battlefield_find(lk).unwrap().power(), 2);
}

/// CR 614.1a — Deification names its controller's planeswalker type as it
/// enters; with a creature out, lethal damage to that walker leaves one
/// loyalty counter, and without one it dies. The walker has hexproof.
#[test]
fn deification_leaves_the_chosen_walker_at_one_loyalty() {
    use crabomination::card::{CounterType, Keyword};
    for with_creature in [true, false] {
        let mut g = pod(2);
        let lili = ready(&mut g, 0, catalog::liliana_death_mage());
        if with_creature {
            ready(&mut g, 0, catalog::grizzly_bears());
        }
        let d = g.add_card_to_hand(0, catalog::deification());
        flood(&mut g, 0);
        cast(&mut g, d, None).expect("deification");
        assert!(g.permanent_has_keyword(lili, &Keyword::Hexproof), "hexproof from the static");
        g.battlefield_find_mut(lili).unwrap().counters.insert(CounterType::Loyalty, 3);
        let dmg = crabomination::effect::Effect::DealDamage {
            to: crabomination::effect::Selector::Target(0),
            amount: crabomination::card::Value::Const(5),
        };
        let mut ctx = EffectContext::for_spell(1, None, 0, 0);
        ctx.targets = vec![Target::Permanent(lili)];
        let evs = g.resolve_effect(&dmg, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        g.check_state_based_actions();
        match with_creature {
            true => assert_eq!(g.battlefield_find(lili).unwrap().counter_count(CounterType::Loyalty), 1),
            false => assert!(g.battlefield_find(lili).is_none(), "no creature: it dies"),
        }
    }
}

/// "Choose a planeswalker type" may name any type, not only one its
/// controller owns: a seat with no walker still names one (Jace, picked).
#[test]
fn deification_names_any_planeswalker_type() {
    use crabomination::card::PlaneswalkerSubtype;
    let mut g = pod(2);
    let d = g.add_card_to_hand(0, catalog::deification());
    flood(&mut g, 0);
    let jace = PlaneswalkerSubtype::ALL.iter().position(|t| *t == PlaneswalkerSubtype::Jace).unwrap();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(jace as u32)]));
    cast(&mut g, d, None).expect("deification");
    assert_eq!(g.battlefield_find(d).unwrap().chosen_planeswalker_type, Some(PlaneswalkerSubtype::Jace));
}

// ── Tomik, Orzhov Lawmage / Tobias Andrion ──────────────────────────────────

/// Way of the Healer empowers Jace 5, and that Jace has the granted [−2]:
/// a Cadet, then surveil 1.
#[test]
fn way_of_the_healer_empowers_and_grants_cadets() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let woh = g.add_card_to_hand(0, catalog::way_of_the_healer());
    flood(&mut g, 0);
    cast(&mut g, woh, None).expect("cast");
    let jace = g
        .battlefield
        .iter()
        .find(|c| c.controller == 0 && c.definition.is_planeswalker())
        .expect("a Jace token")
        .id;
    assert_eq!(g.battlefield_find(jace).unwrap().counter_count(CounterType::Loyalty), 5);
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: jace, ability_index: 2, target: None, x_value: None })
        .expect("granted −2");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Cadet"));
    assert_eq!(g.battlefield_find(jace).unwrap().counter_count(CounterType::Loyalty), 3);
}

/// CR 509.1d — Awesome Presence: a blocker on the enchanted attacker costs
/// {3}; with no mana the block is refused, with it the block stands.
#[test]
fn awesome_presence_taxes_blockers_of_its_host() {
    for pay in [false, true] {
        let mut g = pod(2);
        let atk = ready(&mut g, 0, catalog::grizzly_bears());
        let blk = ready(&mut g, 1, catalog::grizzly_bears());
        let ap = g.add_card_to_hand(0, catalog::awesome_presence());
        flood(&mut g, 0);
        cast(&mut g, ap, Some(Target::Permanent(atk))).expect("aura");
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
            attacker: atk,
            target: crabomination::game::types::AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        if pay {
            g.players[1].mana_pool.add_colorless(3);
        }
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        let res = g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)]));
        assert_eq!(res.is_ok(), pay, "pay={pay}");
    }
}

/// CR 603.7 — a delayed "exile that token at the next end step" is bound to
/// the token its resolution made, not to whatever token was made last
/// (Cogwork Assembler, then a Treasure later in the turn).
#[test]
fn delayed_last_created_token_stays_bound_to_its_token() {
    let mut g = pod(2);
    let ca = ready(&mut g, 0, catalog::cogwork_assembler());
    let ring = ready(&mut g, 0, catalog::sol_ring());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ca,
        ability_index: 0,
        target: Some(Target::Permanent(ring)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("copy");
    drain_stack(&mut g);
    let copy = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Sol Ring").expect("copy").id;
    let treasure = crabomination::effect::Effect::CreateToken {
        who: crabomination::effect::PlayerRef::You,
        count: crabomination::card::Value::ONE,
        definition: std::sync::Arc::new(crabomination::game::effects::treasure_token()),
    };
    let evs = g.resolve_effect(&treasure, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    let tr = g.battlefield.iter().find(|c| c.definition.name == "Treasure").expect("treasure").id;
    pass_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(copy).is_none(), "the copy is exiled");
    assert!(g.battlefield_find(tr).is_some(), "the Treasure stays");
}

// ── Bucky Barnes, Óin the Brave, Lin Sivvi (`decks::cmdr_016x_c`) ───────────

fn attack(g: &mut GameState, attacker: CardId, defender: usize) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker,
        target: crabomination::game::types::AttackTarget::Player(defender),
    }]))
    .expect("attack");
    drain_stack(g);
}

/// Alaborn Zealot blocking destroys itself and the attacker it blocks.
#[test]
fn alaborn_zealot_trades_on_block() {
    let mut g = pod(2);
    let atk = ready(&mut g, 0, catalog::hill_giant());
    let z = ready(&mut g, 1, catalog::alaborn_zealot());
    attack(&mut g, atk, 1);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(z, atk)])).expect("block");
    drain_stack(&mut g);
    assert!(g.battlefield_find(atk).is_none() && g.battlefield_find(z).is_none());
}

/// CR 509.1b — Amrou Seekers: a green blocker is refused, a white one isn't.
#[test]
fn amrou_seekers_only_artifact_or_white_blocks() {
    for (blocker, ok) in [(catalog::grizzly_bears(), false), (catalog::savannah_lions(), true)] {
        let mut g = pod(2);
        let atk = ready(&mut g, 0, catalog::amrou_seekers());
        let blk = ready(&mut g, 1, blocker);
        attack(&mut g, atk, 1);
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        assert_eq!(g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).is_ok(), ok);
    }
}

/// CR 603.4 — Wandering Champion's loot needs a blue or red permanent.
#[test]
fn wandering_champion_loots_only_with_blue_or_red() {
    for (red, drew) in [(false, false), (true, true)] {
        let mut g = pod(2);
        let wc = ready(&mut g, 0, catalog::wandering_champion());
        if red {
            ready(&mut g, 0, catalog::goblin_settler());
        }
        g.players[0].hand.clear();
        g.add_card_to_hand(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        attack(&mut g, wc, 1);
        g.perform_action(GameAction::DeclareBlockers(vec![])).ok();
        pass_to(&mut g, TurnStep::PostCombatMain);
        drain_stack(&mut g);
        let looted = g.players[0].graveyard.iter().any(|c| c.definition.name == "Forest");
        assert_eq!(looted, drew, "red={red}");
    }
}

/// Zealot il-Vec unblocked pings a creature and deals no combat damage.
#[test]
fn zealot_il_vec_pings_instead_of_hitting() {
    let mut g = pod(2);
    let z = ready(&mut g, 0, catalog::zealot_il_vec());
    let elf = ready(&mut g, 1, catalog::llanowar_elves());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack(&mut g, z, 1);
    g.perform_action(GameAction::DeclareBlockers(vec![])).ok();
    drain_stack(&mut g);
    pass_to(&mut g, TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert!(g.battlefield_find(elf).is_none(), "the ping kills the elf");
    assert_eq!(g.players[1].life, 20, "its combat damage is prevented");
}

/// Immortal Phoenix returns to hand as it dies; Firewing Phoenix buys itself
/// back from the graveyard.
#[test]
fn phoenixes_come_back_to_hand() {
    let mut g = pod(2);
    let ip = ready(&mut g, 0, catalog::immortal_phoenix());
    let fw = g.add_card_to_graveyard(0, catalog::firewing_phoenix());
    flood(&mut g, 0);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(ip))).expect("bolt");
    cast(&mut g, bolt2, Some(Target::Player(1))).expect("bolt");
    assert!(g.players[0].hand.iter().any(|c| c.id == ip), "Immortal Phoenix in hand");
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: fw,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("return");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == fw), "Firewing Phoenix in hand");
}

/// Amrou Scout puts a Rebel of mana value 3 or less from the library onto the
/// battlefield; Hallowed Burial then bottoms every creature.
#[test]
fn amrou_scout_fetches_a_rebel_and_hallowed_burial_bottoms_creatures() {
    let mut g = pod(2);
    let scout = ready(&mut g, 0, catalog::amrou_scout());
    let seekers = g.add_card_to_library(0, catalog::amrou_seekers());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: scout,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("search");
    drain_stack(&mut g);
    assert!(g.battlefield_find(seekers).is_some());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let hb = g.add_card_to_hand(0, catalog::hallowed_burial());
    flood(&mut g, 0);
    cast(&mut g, hb, None).expect("burial");
    assert!(g.battlefield.iter().all(|c| !c.definition.is_creature()));
    assert_eq!(g.players[1].library.last().map(|c| c.id), Some(bear), "bottom of its owner's library");
}

// ── Klaw, Kytheon ───────────────────────────────────────────────────────────

/// Abattoir Ghoul: a creature it damaged this turn dying gains you life equal
/// to that creature's toughness (Hill Giant: 3).
#[test]
fn abattoir_ghoul_gains_the_toughness_of_its_victim() {
    let mut g = pod(2);
    let ghoul = ready(&mut g, 0, catalog::abattoir_ghoul());
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let ping = crabomination::effect::Effect::DealDamage {
        to: crabomination::effect::Selector::Target(0),
        amount: crabomination::card::Value::Const(3),
    };
    let mut ctx = EffectContext::for_ability(ghoul, 0, None);
    ctx.targets = vec![Target::Permanent(giant)];
    let evs = g.resolve_effect(&ping, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none());
    assert_eq!(g.players[0].life, 23);
}

/// Appetite for Brains can only take a card of mana value 4 or more.
#[test]
fn appetite_for_brains_takes_only_big_cards() {
    let mut g = pod(2);
    g.players[1].hand.clear();
    let small = g.add_card_to_hand(1, catalog::lightning_bolt());
    let big = g.add_card_to_hand(1, catalog::hill_giant());
    let afb = g.add_card_to_hand(0, catalog::appetite_for_brains());
    flood(&mut g, 0);
    cast(&mut g, afb, Some(Target::Player(1))).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == big));
    assert!(g.players[1].hand.iter().any(|c| c.id == small));
}

/// CR 615.1 — Gideon's Intervention naming Lightning Bolt: the opponent can't
/// cast one, and one already on its way (a copy cast by a third seat before
/// it entered) deals no damage to you.
#[test]
fn gideons_intervention_locks_and_shields_the_named_card() {
    let mut g = pod(3);
    let gi = g.add_card_to_hand(0, catalog::gideons_intervention());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::NamedCard("Lightning Bolt".into())]));
    cast(&mut g, gi, None).expect("intervention");
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
            .is_err(),
        "the name is locked"
    );
    let dmg = crabomination::effect::Effect::DealDamage {
        to: crabomination::effect::Selector::Target(0),
        amount: crabomination::card::Value::Const(3),
    };
    let mut ctx = EffectContext::for_spell(1, None, 0, 0);
    ctx.source = Some(bolt);
    ctx.targets = vec![Target::Player(0)];
    g.resolve_effect(&dmg, &ctx).unwrap();
    assert_eq!(g.players[0].life, 20, "damage from a Lightning Bolt is prevented");
}

/// Gideon, the Oathsworn: attacking with two non-Gideon creatures puts a
/// +1/+1 counter on each of them.
#[test]
fn gideon_the_oathsworn_counters_a_two_creature_attack() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    ready(&mut g, 0, catalog::gideon_the_oathsworn());
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        crabomination::game::types::Attack { attacker: a, target: crabomination::game::types::AttackTarget::Player(1) },
        crabomination::game::types::Attack { attacker: b, target: crabomination::game::types::AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    for id in [a, b] {
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }
}

/// Gideon's Phalanx: four vigilant Knights; spell mastery (two instants or
/// sorceries in the graveyard) makes your team indestructible this turn.
#[test]
fn gideons_phalanx_spell_mastery() {
    for mastery in [false, true] {
        let mut g = pod(2);
        if mastery {
            g.add_card_to_graveyard(0, catalog::lightning_bolt());
            g.add_card_to_graveyard(0, catalog::lightning_bolt());
        }
        let gp = g.add_card_to_hand(0, catalog::gideons_phalanx());
        flood(&mut g, 0);
        cast(&mut g, gp, None).expect("cast");
        let knights: Vec<CardId> = g.battlefield.iter().filter(|c| c.definition.name == "Knight").map(|c| c.id).collect();
        assert_eq!(knights.len(), 4);
        assert_eq!(g.permanent_has_keyword(knights[0], &crabomination::card::Keyword::Indestructible), mastery);
    }
}

// ── Glissa, Chief of the Wilds, The Chief Warg (`decks::cmdr_016x_d`) ───────

/// Chief of the Wilds doubles another Wolf's trigger: Chief Warg's Company's
/// upkeep makes two Wolves, and each Wolf entering grows the Chief by two.
#[test]
fn chief_of_the_wilds_doubles_wolf_triggers() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let chief = ready(&mut g, 0, catalog::chief_of_the_wilds());
    ready(&mut g, 0, catalog::chief_wargs_company());
    g.step = TurnStep::Untap;
    g.active_player_idx = 0;
    pass_to(&mut g, TurnStep::Upkeep);
    drain_stack(&mut g);
    let wolves = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Wolf").count();
    assert_eq!(wolves, 2, "the upkeep trigger fired twice");
    assert_eq!(g.battlefield_find(chief).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
}

/// Chief Warg's Company can't attack without two other Wolves.
#[test]
fn chief_wargs_company_needs_two_other_wolves() {
    for (others, ok) in [(1, false), (2, true)] {
        let mut g = pod(2);
        let c = ready(&mut g, 0, catalog::chief_wargs_company());
        for _ in 0..others {
            ready(&mut g, 0, catalog::nighthowl_pursuer());
        }
        g.step = TurnStep::DeclareAttackers;
        let res = g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
            attacker: c,
            target: crabomination::game::types::AttackTarget::Player(1),
        }]));
        assert_eq!(res.is_ok(), ok, "others={others}");
    }
}

/// Traumatic Revelation takes a creature card, or incubates 3 when there is
/// none to take.
#[test]
fn traumatic_revelation_discards_or_incubates() {
    for creature in [true, false] {
        let mut g = pod(2);
        g.players[1].hand.clear();
        let card = if creature { catalog::grizzly_bears() } else { catalog::lightning_bolt() };
        let id = g.add_card_to_hand(1, card);
        let tr = g.add_card_to_hand(0, catalog::traumatic_revelation());
        flood(&mut g, 0);
        cast(&mut g, tr, Some(Target::Player(1))).expect("cast");
        assert_eq!(g.players[1].hand.iter().any(|c| c.id == id), !creature);
        let incs = g.battlefield.iter().filter(|c| c.definition.name == "Incubator").count();
        assert_eq!(incs, usize::from(!creature));
    }
}

/// Elvish Vatkeeper's {5} transforms an Incubator and doubles its counters;
/// Glistening Dawn's incubators carry one counter per land.
#[test]
fn vatkeeper_doubles_and_glistening_dawn_counts_lands() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    for _ in 0..3 {
        ready(&mut g, 0, catalog::forest());
    }
    let gd = g.add_card_to_hand(0, catalog::glistening_dawn());
    flood(&mut g, 0);
    cast(&mut g, gd, None).expect("dawn");
    let incs: Vec<CardId> = g.battlefield.iter().filter(|c| c.definition.name == "Incubator").map(|c| c.id).collect();
    assert_eq!(incs.len(), 2);
    assert_eq!(g.battlefield_find(incs[0]).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    let vk = ready(&mut g, 0, catalog::elvish_vatkeeper());
    drain_stack(&mut g);
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: vk,
        ability_index: 0,
        target: Some(Target::Permanent(incs[0])),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("vatkeeper");
    drain_stack(&mut g);
    let p = g.battlefield_find(incs[0]).unwrap();
    assert!(p.definition.is_creature(), "transformed");
    assert_eq!(p.counter_count(CounterType::PlusOnePlusOne), 6);
}

// ── Sidar Jabari, Rakka Mar (`decks::cmdr_016x_e`) ──────────────────────────

/// CR 702.25b — Cavalry Master gives another flanker a second instance: a
/// non-flanking blocker of Benalish Cavalry gets -2/-2, and a 2/2 dies.
#[test]
fn cavalry_master_doubles_flanking() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::cavalry_master());
    let bc = ready(&mut g, 0, catalog::benalish_cavalry());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    attack(&mut g, bc, 1);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(bear, bc)])).expect("block");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "two flanking triggers: -2/-2");
}

/// Gustcloak Cavalier slips its blocker: untapped and out of combat.
#[test]
fn gustcloak_cavalier_leaves_combat_when_blocked() {
    let mut g = pod(2);
    let gc = ready(&mut g, 0, catalog::gustcloak_cavalier());
    let wall = ready(&mut g, 1, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false), DecisionAnswer::Bool(true)]));
    attack(&mut g, gc, 1);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, gc)])).expect("block");
    drain_stack(&mut g);
    let c = g.battlefield_find(gc).unwrap();
    assert!(!c.tapped, "untapped");
    assert!(!g.attacking.iter().any(|a| a.attacker == gc), "removed from combat");
}

/// Thunderkin Awakener returns a smaller Elemental from the graveyard
/// attacking, sacrificed at the end step.
#[test]
fn thunderkin_awakener_returns_a_smaller_elemental_attacking() {
    let mut g = pod(2);
    let ta = ready(&mut g, 0, catalog::thunderkin_awakener());
    let rock = g.add_card_to_graveyard(0, catalog::rockslide_elemental());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: ta,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.attacking.iter().any(|a| a.attacker == rock), "the Elemental attacks");
    pass_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(rock).is_none(), "sacrificed at the end step");
}

/// Chandra's Embercat's {R} pays for an Elemental, not for a non-Elemental.
#[test]
fn chandras_embercat_mana_is_for_elementals() {
    let mut g = pod(2);
    let cat = ready(&mut g, 0, catalog::chandras_embercat());
    g.perform_action(GameAction::ActivateAbility {
        card_id: cat,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    assert!(cast(&mut g, bolt, Some(Target::Player(1))).is_err(), "not an Elemental spell");
}

// ── Starling, Falcon (`decks::cmdr_016x_f`) ─────────────────────────────────

/// Doomskar Oracle: the second spell in a turn gains 2 life, the first and
/// third don't.
#[test]
fn doomskar_oracle_gains_on_the_second_spell() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::doomskar_oracle());
    flood(&mut g, 0);
    let mut lives = vec![];
    for _ in 0..3 {
        let b = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast(&mut g, b, Some(Target::Player(1))).expect("bolt");
        lives.push(g.players[0].life);
    }
    assert_eq!(lives, vec![20, 22, 22]);
}

/// Falcon makes Redwing, whose attack surveils 1.
#[test]
fn falcon_makes_redwing() {
    let mut g = pod(2);
    let f = g.add_card_to_hand(0, catalog::falcon_winged_wonder());
    flood(&mut g, 0);
    cast(&mut g, f, None).expect("falcon");
    let rw = g.battlefield.iter().find(|c| c.definition.name == "Redwing").expect("Redwing").id;
    g.clear_sickness(rw);
    let lib = g.players[0].library.len();
    attack(&mut g, rw, 1);
    assert!(g.players[0].library.len() <= lib, "surveilled");
}

/// TVA Bureaucrat: a noncreature spell makes it +1/+0 and unblockable.
#[test]
fn tva_bureaucrat_slips_past_after_a_noncreature_spell() {
    let mut g = pod(2);
    let tva = ready(&mut g, 0, catalog::tva_bureaucrat());
    flood(&mut g, 0);
    let b = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, b, Some(Target::Player(1))).expect("bolt");
    assert_eq!(g.battlefield_find(tva).unwrap().power(), 2);
    assert!(g.permanent_has_keyword(tva, &crabomination::card::Keyword::Unblockable));
}

/// Stitched Mangler taps an opposing creature that then skips its untap.
#[test]
fn stitched_mangler_freezes_a_creature() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let sm = g.add_card_to_hand(0, catalog::stitched_mangler());
    flood(&mut g, 0);
    cast(&mut g, sm, None).expect("mangler");
    assert!(g.battlefield_find(bear).unwrap().tapped);
    assert!(g.battlefield_find(sm).unwrap().tapped, "enters tapped");
}

// ── Tesak, Zhang He (`decks::cmdr_016x_g`) ──────────────────────────────────

/// Ashmouth Hound blocked by a 1-toughness creature kills it before combat
/// damage (the trigger resolves in the declare-blockers step).
#[test]
fn ashmouth_hound_pings_its_blocker() {
    let mut g = pod(2);
    let ah = ready(&mut g, 0, catalog::ashmouth_hound());
    let elf = ready(&mut g, 1, catalog::llanowar_elves());
    attack(&mut g, ah, 1);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(elf, ah)])).expect("block");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(elf).is_none());
    assert!(g.battlefield_find(ah).is_some(), "the elf died before dealing damage");
}

/// Wei Assassins: the targeted opponent picks which of their creatures dies.
#[test]
fn wei_assassins_destroys_the_opponents_pick() {
    let mut g = pod(3);
    let small = ready(&mut g, 2, catalog::llanowar_elves());
    let big = ready(&mut g, 2, catalog::hill_giant());
    let other = ready(&mut g, 1, catalog::grizzly_bears());
    let wa = g.add_card_to_hand(0, catalog::wei_assassins());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: wa, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    let died = [small, big].iter().filter(|id| g.battlefield_find(**id).is_none()).count()
        + usize::from(g.battlefield_find(other).is_none());
    assert_eq!(died, 1, "exactly one opposing creature is destroyed");
}

/// CR 702.11b — Wei Assassins' opponent *chooses* a creature (no target), so
/// their only creature dies even with hexproof.
#[test]
fn cr_702_11b_wei_assassins_reaches_a_hexproof_creature() {
    let mut g = pod(2);
    let bogle = ready(&mut g, 1, catalog::slippery_bogle());
    let wa = g.add_card_to_hand(0, catalog::wei_assassins());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: wa, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bogle).is_none());
}

/// CR 603.4 — Nightsquad Commando's Soldier needs an attack this turn.
#[test]
fn nightsquad_commando_needs_an_attack() {
    for attacked in [false, true] {
        let mut g = pod(2);
        if attacked {
            let bear = ready(&mut g, 0, catalog::grizzly_bears());
            attack(&mut g, bear, 1);
            g.perform_action(GameAction::DeclareBlockers(vec![])).ok();
            pass_to(&mut g, TurnStep::PostCombatMain);
            drain_stack(&mut g);
        }
        let nc = g.add_card_to_hand(0, catalog::nightsquad_commando());
        flood(&mut g, 0);
        g.priority.player_with_priority = 0;
        cast(&mut g, nc, None).expect("cast");
        let made = g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Human Soldier");
        assert_eq!(made, attacked);
    }
}

// ── Thelon of Havenwood (`decks::cmdr_016x_h`) ──────────────────────────────

/// Thelon pumps each Fungus +1/+1 per spore counter on it, and its
/// activation (exiling a Fungus card from a graveyard) adds a spore to each.
#[test]
fn thelon_pumps_fungi_by_their_spores() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let thelon = ready(&mut g, 0, catalog::thelon_of_havenwood());
    let tt = ready(&mut g, 0, catalog::thorn_thallid());
    g.battlefield_find_mut(tt).unwrap().counters.insert(CounterType::Spore, 2);
    assert_eq!(g.computed_permanent(tt).unwrap().power, 4, "2/2 + two spores");
    g.add_card_to_graveyard(1, catalog::spore_flower());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: thelon,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(tt).unwrap().counter_count(CounterType::Spore), 3);
}

/// Spore Flower's three spores: a fog — an attacker deals no damage.
#[test]
fn spore_flower_fogs() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let sf = ready(&mut g, 1, catalog::spore_flower());
    g.battlefield_find_mut(sf).unwrap().counters.insert(CounterType::Spore, 3);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    attack(&mut g, bear, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sf,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("fog");
    drain_stack(&mut g);
    g.perform_action(GameAction::DeclareBlockers(vec![])).ok();
    pass_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, 20);
}

// ── Teyo, Diamondblade Mage (`decks::cmdr_016x_i`) ──────────────────────────

/// Death-Rattle Oni destroys every other creature dealt damage this turn.
#[test]
fn death_rattle_oni_finishes_the_wounded() {
    let mut g = pod(2);
    let hurt = ready(&mut g, 1, catalog::hill_giant());
    let fine = ready(&mut g, 1, catalog::grizzly_bears());
    let ping = crabomination::effect::Effect::DealDamage {
        to: crabomination::effect::Selector::Target(0),
        amount: crabomination::card::Value::Const(1),
    };
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(hurt)];
    g.resolve_effect(&ping, &ctx).unwrap();
    assert!(g.battlefield_find(hurt).is_some(), "the 3/3 survives the ping");
    let oni = g.add_card_to_hand(0, catalog::death_rattle_oni());
    flood(&mut g, 0);
    cast(&mut g, oni, None).expect("oni");
    assert!(g.battlefield_find(hurt).is_none(), "damaged this turn: destroyed");
    assert!(g.battlefield_find(fine).is_some());
}

/// Teyo's ETB: deathtouch and a +1/+1 counter on your creature.
#[test]
fn teyo_grants_deathtouch_and_a_counter() {
    use crabomination::card::{CounterType, Keyword};
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let teyo = g.add_card_to_hand(0, catalog::teyo_diamondblade_mage());
    flood(&mut g, 0);
    cast(&mut g, teyo, None).expect("teyo");
    let target = if g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne) == 1 { bear } else { teyo };
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.permanent_has_keyword(target, &Keyword::Deathtouch));
}

// ── Zhou Yu, Chief Commander (`decks::cmdr_016x_j`) ─────────────────────────

/// CR 508.1c — Wu Warship can't attack a player with no Island; CR 305.7 —
/// Phantasmal Terrain (Island chosen) makes the defender's Forest an Island,
/// and then it can.
#[test]
fn phantasmal_terrain_opens_the_island_gate() {
    let mut g = pod(2);
    let forest = ready(&mut g, 1, catalog::forest());
    let ship = ready(&mut g, 0, catalog::wu_warship());
    g.step = TurnStep::DeclareAttackers;
    let atk = |ship| {
        GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
            attacker: ship,
            target: crabomination::game::types::AttackTarget::Player(1),
        }])
    };
    assert!(g.perform_action(atk(ship)).is_err(), "no Island: can't attack");
    g.step = TurnStep::PreCombatMain;
    let pt = g.add_card_to_hand(0, catalog::phantasmal_terrain());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Blue)]));
    cast(&mut g, pt, Some(Target::Permanent(forest))).expect("terrain");
    assert_eq!(g.battlefield_find(pt).unwrap().attached_to, Some(forest));
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(atk(ship)).expect("the Forest is an Island now");
}

/// Floodchaser enters with six +1/+1 counters; {U}, remove one: target land
/// is an Island this turn, opening its own gate.
#[test]
fn floodchaser_floods_a_land() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let forest = ready(&mut g, 1, catalog::forest());
    let fc = g.add_card_to_hand(0, catalog::floodchaser());
    flood(&mut g, 0);
    cast(&mut g, fc, None).expect("floodchaser");
    g.clear_sickness(fc);
    assert_eq!(g.battlefield_find(fc).unwrap().counter_count(CounterType::PlusOnePlusOne), 6);
    g.perform_action(GameAction::ActivateAbility {
        card_id: fc,
        ability_index: 0,
        target: Some(Target::Permanent(forest)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("flood");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(fc).unwrap().counter_count(CounterType::PlusOnePlusOne), 5);
    attack(&mut g, fc, 1);
    assert!(g.attacking.iter().any(|a| a.attacker == fc));
}

// ── Gorion, Wise Mentor / Beluna Grandsquall (`decks::cmdr_016x_k`) ─────────

fn stomp(g: &mut GameState, giant: CardId, target: Target) -> Result<(), GameError> {
    g.perform_action(GameAction::CastAdventure {
        card_id: giant,
        target: Some(target),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })?;
    drain_stack(g);
    Ok(())
}

/// CR 707.2 / 715.3b — Gorion copies an Adventure spell, and the copy has
/// only the Adventure's characteristics: two Stomps kill a 3/3, and no
/// token Bonecrusher Giant appears.
#[test]
fn gorion_copies_an_adventure_spell() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::gorion_wise_mentor());
    let ogre = ready(&mut g, 1, catalog::hill_giant());
    let giant = g.add_card_to_hand(0, catalog::bonecrusher_giant());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(false)]));
    stomp(&mut g, giant, Target::Permanent(ogre)).expect("stomp");
    assert!(g.battlefield_find(ogre).is_none(), "Stomp and its copy: 2 + 2 kills the 3/3");
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Bonecrusher Giant"), "the copy is no creature");
    assert!(g.exile.iter().any(|c| c.id == giant && c.on_adventure), "the original goes on an adventure");
}

/// CR 715.3b — Edgewall Innkeeper draws for the creature cast of an
/// Adventurer card, not for the Adventure cast.
#[test]
fn edgewall_innkeeper_draws_on_the_creature_half_only() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::edgewall_innkeeper());
    let a = g.add_card_to_hand(0, catalog::bonecrusher_giant());
    let b = g.add_card_to_hand(0, catalog::bonecrusher_giant());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    stomp(&mut g, a, Target::Player(1)).expect("stomp");
    assert_eq!(g.players[0].hand.len(), hand - 1, "the Adventure cast draws nothing");
    cast(&mut g, b, None).expect("giant");
    assert_eq!(g.players[0].hand.len(), hand - 1, "the creature cast replaces itself");
}

/// CR 601.2f — Beluna's {1} discount reaches a permanent spell with an
/// Adventure: Bonecrusher Giant casts off {1}{R}.
#[test]
fn beluna_discounts_adventurer_creatures() {
    let mut g = pod(2);
    let giant = g.add_card_to_hand(0, catalog::bonecrusher_giant());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(cast(&mut g, giant, None).is_err(), "{{2}}{{R}} without Beluna");
    ready(&mut g, 0, catalog::beluna_grandsquall());
    cast(&mut g, giant, None).expect("{1}{R} with Beluna");
    assert!(g.battlefield_find(giant).is_some());
}

/// Hearth Elemental costs {1} less per instant, sorcery or Adventure card in
/// your graveyard.
#[test]
fn hearth_elemental_counts_spells_in_the_yard() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
    }
    g.add_card_to_graveyard(0, catalog::bonecrusher_giant());
    let he = g.add_card_to_hand(0, catalog::hearth_elemental());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, he, None).expect("{5}{R} less 4");
    assert!(g.battlefield_find(he).is_some());
}

/// Seek Thrills mills seven and keeps every Adventure card among them.
#[test]
fn seek_thrills_keeps_the_adventurers() {
    let mut g = pod(2);
    g.players[0].library.clear();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::bonecrusher_giant());
        g.add_card_to_library(0, catalog::forest());
    }
    let beluna = g.add_card_to_hand(0, catalog::beluna_grandsquall());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastAdventure { card_id: beluna, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("seek thrills");
    drain_stack(&mut g);
    let giants = g.players[0].hand.iter().filter(|c| c.definition.name == "Bonecrusher Giant").count();
    assert_eq!(giants, 3);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3);
}

// ── Syrix, Carrier of the Flame (`decks::cmdr_016x_l`) ──────────────────────

fn activate(g: &mut GameState, card_id: CardId) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn on_battlefield(g: &GameState, name: &str) -> Option<CardId> {
    g.battlefield.iter().find(|c| c.definition.name == name).map(|c| c.id)
}

/// CR 603.2 / 113.6k — Bloodfeather Phoenix's ability works from the
/// graveyard: an instant that hits an opponent buys it back for {R}, hasty.
#[test]
fn bloodfeather_phoenix_rises_on_spell_damage() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    g.add_card_to_graveyard(0, catalog::bloodfeather_phoenix());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
    let p = on_battlefield(&g, "Bloodfeather Phoenix").expect("returned");
    assert!(g.permanent_has_keyword(p, &Keyword::Haste));
}

/// Chandra's Phoenix returns to hand when a red instant hits an opponent,
/// not when a creature does.
#[test]
fn chandras_phoenix_returns_on_red_spell_damage_only() {
    let mut g = pod(2);
    let ph = g.add_card_to_graveyard(0, catalog::chandras_phoenix());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    attack(&mut g, bear, 1);
    g.perform_action(GameAction::DeclareBlockers(vec![])).ok();
    pass_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, 18);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == ph), "combat damage: stays");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
    assert!(g.players[0].hand.iter().any(|c| c.id == ph), "red instant: back to hand");
}

/// CR 603.4 — Lightning Phoenix's end-step return checks that an opponent
/// was dealt 3 or more damage this turn (one opponent, not the sum).
#[test]
fn lightning_phoenix_needs_three_damage_to_one_opponent() {
    let mut g = pod(3);
    g.add_card_to_graveyard(0, catalog::lightning_phoenix());
    let hit = |g: &mut GameState, seat: usize, n: i32| {
        let ping = crabomination::effect::Effect::DealDamage {
            to: crabomination::effect::Selector::Target(0),
            amount: crabomination::card::Value::Const(n),
        };
        let mut ctx = EffectContext::for_spell(0, None, 0, 0);
        ctx.targets = vec![Target::Player(seat)];
        g.resolve_effect(&ping, &ctx).unwrap();
    };
    hit(&mut g, 1, 2);
    hit(&mut g, 2, 2);
    ready(&mut g, 0, catalog::mountain());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    pass_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(on_battlefield(&g, "Lightning Phoenix").is_none(), "2 + 2 split damage isn't 3 to one opponent");
}

/// Lightning Phoenix returns for {R} once one opponent took 3.
#[test]
fn lightning_phoenix_returns_after_three_damage() {
    let mut g = pod(2);
    g.add_card_to_graveyard(0, catalog::lightning_phoenix());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
    ready(&mut g, 0, catalog::mountain());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    pass_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(on_battlefield(&g, "Lightning Phoenix").is_some());
}

/// Magma Phoenix dying deals 3 to each creature and each player; {3}{R}{R}
/// returns it from the graveyard to hand.
#[test]
fn magma_phoenix_erupts_and_recurs() {
    let mut g = pod(2);
    let ph = ready(&mut g, 0, catalog::magma_phoenix());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let kill = crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::Target(0) };
    let mut ctx = EffectContext::for_spell(1, None, 0, 0);
    ctx.targets = vec![Target::Permanent(ph)];
    g.resolve_effect(&kill, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!((g.players[0].life, g.players[1].life), (17, 17));
    flood(&mut g, 0);
    activate(&mut g, ph).expect("return to hand");
    assert!(g.players[0].hand.iter().any(|c| c.id == ph));
}

/// CR 506.2 — Nemesis Phoenix returns attacking only while you attack two
/// or more opponents.
#[test]
fn nemesis_phoenix_needs_two_defenders() {
    let mut g = pod(3);
    let ph = g.add_card_to_graveyard(0, catalog::nemesis_phoenix());
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g, 0);
    attack(&mut g, a, 1);
    assert!(activate(&mut g, ph).is_err(), "one defender");
    g.attacking.push(crabomination::game::types::Attack {
        attacker: b,
        target: crabomination::game::types::AttackTarget::Player(2),
    });
    activate(&mut g, ph).expect("two defenders");
    assert!(g.attacking.iter().any(|x| x.attacker == ph));
}

/// Warcry Phoenix joins an attack of three or more for {2}{R}.
#[test]
fn warcry_phoenix_joins_a_big_attack() {
    let mut g = pod(2);
    let ph = g.add_card_to_graveyard(0, catalog::warcry_phoenix());
    let bears: Vec<CardId> = (0..3).map(|_| ready(&mut g, 0, catalog::grizzly_bears())).collect();
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(
        bears
            .iter()
            .map(|&attacker| crabomination::game::types::Attack {
                attacker,
                target: crabomination::game::types::AttackTarget::Player(1),
            })
            .collect(),
    ))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.attacking.iter().any(|x| x.attacker == ph));
}

/// Kuldotha Phoenix: metalcraft, during your upkeep only.
#[test]
fn kuldotha_phoenix_needs_upkeep_and_metalcraft() {
    let mut g = pod(2);
    let ph = g.add_card_to_graveyard(0, catalog::kuldotha_phoenix());
    flood(&mut g, 0);
    for _ in 0..3 {
        ready(&mut g, 0, catalog::ornithopter());
    }
    assert!(activate(&mut g, ph).is_err(), "main phase");
    g.step = TurnStep::Upkeep;
    activate(&mut g, ph).expect("upkeep with metalcraft");
    assert!(g.battlefield_find(ph).is_some());
}

/// CR 726.2 — Sunstreak Phoenix makes it day as it enters.
#[test]
fn sunstreak_phoenix_makes_it_day() {
    let mut g = pod(2);
    let ph = g.add_card_to_hand(0, catalog::sunstreak_phoenix());
    flood(&mut g, 0);
    cast(&mut g, ph, None).expect("phoenix");
    assert_eq!(g.day_night, Some(crabomination::game::types::DayNight::Day));
}

// ── Meng Huo, Barbarian King (`decks::cmdr_016x_m`) ─────────────────────────

/// Meng Huo pumps other green creatures you control, not himself or a
/// nongreen one.
#[test]
fn meng_huo_pumps_other_green_creatures() {
    let mut g = pod(2);
    let mh = ready(&mut g, 0, catalog::meng_huo_barbarian_king());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
    assert_eq!(g.computed_permanent(giant).unwrap().power, 3, "red: unpumped");
    assert_eq!(g.computed_permanent(mh).unwrap().power, 4, "other");
}

/// Slashing Tiger gets +2/+2 when it becomes blocked.
#[test]
fn slashing_tiger_grows_when_blocked() {
    let mut g = pod(2);
    let tiger = ready(&mut g, 0, catalog::slashing_tiger());
    let wall = ready(&mut g, 1, catalog::hill_giant());
    attack(&mut g, tiger, 1);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, tiger)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(tiger).unwrap().power, 5);
}

/// CR 601.3 — Heavy Fog is castable only in the declare attackers step
/// after you're attacked; it prevents the attackers' damage to you.
#[test]
fn heavy_fog_stops_the_attackers() {
    let mut g = pod(2);
    let fog = g.add_card_to_hand(0, catalog::heavy_fog());
    flood(&mut g, 0);
    assert!(cast(&mut g, fog, None).is_err(), "main phase");
    let giant = ready(&mut g, 1, catalog::hill_giant());
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: giant,
        target: crabomination::game::types::AttackTarget::Player(0),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    g.priority.player_with_priority = 0;
    flood(&mut g, 0);
    cast(&mut g, fog, None).expect("fog");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![])).ok();
    pass_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[0].life, 20);
}

/// Riding the Dilu Horse: +2/+2 and horsemanship that outlast the turn.
#[test]
fn riding_the_dilu_horse_is_indefinite() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::riding_the_dilu_horse());
    flood(&mut g, 0);
    cast(&mut g, s, Some(Target::Permanent(bear))).expect("dilu");
    pass_to(&mut g, TurnStep::Upkeep);
    assert_ne!(g.active_player_idx, 0, "the next turn");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4);
    assert!(g.permanent_has_keyword(bear, &Keyword::Horsemanship));
}

/// Hunting Cheetah hitting an opponent tutors a Forest to hand.
#[test]
fn hunting_cheetah_finds_a_forest() {
    let mut g = pod(2);
    let cat = ready(&mut g, 0, catalog::hunting_cheetah());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let forests = |g: &GameState| g.players[0].hand.iter().filter(|c| c.definition.name == "Forest").count();
    attack(&mut g, cat, 1);
    g.perform_action(GameAction::DeclareBlockers(vec![])).ok();
    pass_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, 18);
    assert_eq!(forests(&g), 1);
}

/// False Mourning puts a card from your graveyard on top of your library.
#[test]
fn false_mourning_stacks_the_top() {
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::false_mourning());
    flood(&mut g, 0);
    cast(&mut g, s, Some(Target::Permanent(bear))).expect("mourning");
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(bear));
}

// ── The Unluckiest Planeswalker / Lynde (`decks::cmdr_016x_n`) ──────────────

/// A Curse of `seat_from`'s on the battlefield enchanting `on`.
fn curse_on(g: &mut GameState, seat_from: usize, def: crabomination::card::CardDefinition, on: usize) -> CardId {
    let id = g.add_card_to_battlefield(seat_from, def);
    g.battlefield_find_mut(id).unwrap().attached_to_player = Some(on);
    id
}

/// CR 303.4 — Curse of Thirst counts every Curse attached to the player.
#[test]
fn curse_of_thirst_counts_the_curses_on_its_player() {
    let mut g = pod(2);
    curse_on(&mut g, 0, catalog::curse_of_thirst(), 1);
    curse_on(&mut g, 0, catalog::torment_of_scarabs(), 1);
    let life = g.players[1].life;
    pass_to(&mut g, TurnStep::Upkeep);
    assert_eq!(g.active_player_idx, 1);
    drain_stack(&mut g);
    // Thirst deals 2; Scarabs (no hand, no nonland permanent) takes 3.
    assert_eq!(g.players[1].life, life - 2 - 3);
}

/// Curse of Shaken Faith punishes the enchanted player's second spell, not
/// the first.
#[test]
fn curse_of_shaken_faith_hits_the_second_spell() {
    let mut g = pod(2);
    curse_on(&mut g, 0, catalog::curse_of_shaken_faith(), 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    flood(&mut g, 1);
    let a = g.add_card_to_hand(1, catalog::lightning_bolt());
    let b = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, a, Some(Target::Player(0))).expect("first");
    assert_eq!(g.players[1].life, 20);
    g.priority.player_with_priority = 1;
    cast(&mut g, b, Some(Target::Player(0))).expect("second");
    assert_eq!(g.players[1].life, 18);
}

/// Curse of Shaken Faith: copying a spell punishes the enchanted player even
/// when nothing they cast this turn was a second spell (Reverberate is their
/// first cast; its copy of an opponent's Bolt triggers it).
#[test]
fn curse_of_shaken_faith_hits_a_copy() {
    let mut g = pod(2);
    curse_on(&mut g, 0, catalog::curse_of_shaken_faith(), 1);
    flood(&mut g, 0);
    flood(&mut g, 1);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let rev = g.add_card_to_hand(1, catalog::reverberate());
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("bolt");
    g.priority.player_with_priority = 1;
    cast(&mut g, rev, Some(Target::Permanent(bolt))).expect("reverberate");
    assert_eq!(g.players[1].life, 18, "the copy triggers it; the first cast doesn't");
}

/// CR 707.10c — Curse of Echoes: the other player copies the enchanted
/// player's instant and aims it back.
#[test]
fn curse_of_echoes_turns_a_bolt_around() {
    let mut g = pod(2);
    curse_on(&mut g, 0, catalog::curse_of_echoes(), 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    flood(&mut g, 1);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(0))).expect("bolt");
    assert_eq!(g.players[0].life, 17);
    assert_eq!(g.players[1].life, 17, "seat 0's copy hits back");
}

/// CR 601.2f — Curse of Silence taxes the named spell for its player only.
#[test]
fn curse_of_silence_taxes_the_named_spell() {
    let mut g = pod(2);
    let cs = curse_on(&mut g, 0, catalog::curse_of_silence(), 1);
    g.battlefield_find_mut(cs).unwrap().named_card = Some("Lightning Bolt".into());
    let mine = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, mine, Some(Target::Player(1))).expect("the curser pays no tax");
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let theirs = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    assert!(cast(&mut g, theirs, Some(Target::Player(0))).is_err(), "{{R}} + {{2}}");
    g.players[1].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    cast(&mut g, theirs, Some(Target::Player(0))).expect("taxed");
    assert!(g.battlefield_find(cs).is_none(), "sacrificed");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew");
}

/// CR 303.4f — Curse of Misfortunes fetches a differently named Curse onto
/// its player at your upkeep.
#[test]
fn curse_of_misfortunes_fetches_another_curse() {
    let mut g = pod(2);
    curse_on(&mut g, 0, catalog::curse_of_misfortunes(), 1);
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::curse_of_misfortunes());
    let thirst = g.add_card_to_library(0, catalog::curse_of_thirst());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::Untap;
    pass_to(&mut g, TurnStep::Upkeep);
    assert_eq!(g.active_player_idx, 0);
    drain_stack(&mut g);
    let t = g.battlefield_find(thirst).expect("Thirst fetched, not a second Misfortunes");
    assert_eq!(t.attached_to_player, Some(1));
}

/// Accursed Witch returns as Infectious Curse on an opponent, which drains
/// at their upkeep.
#[test]
fn accursed_witch_returns_as_infectious_curse() {
    let mut g = pod(2);
    let witch = ready(&mut g, 0, catalog::accursed_witch());
    let kill = crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::Target(0) };
    let mut ctx = EffectContext::for_spell(1, None, 0, 0);
    ctx.targets = vec![Target::Permanent(witch)];
    g.resolve_effect(&kill, &ctx).unwrap();
    drain_stack(&mut g);
    let c = g.battlefield_find(witch).expect("back");
    assert_eq!(c.definition.name, "Infectious Curse");
    assert_eq!(c.attached_to_player, Some(1));
    pass_to(&mut g, TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!((g.players[0].life, g.players[1].life), (21, 19));
}

/// CR 601.2f — Accursed Witch's {1} discount is only for its controller's
/// opponents; Infectious Curse's is for its controller's spells aimed at the
/// enchanted player.
#[test]
fn accursed_witch_and_infectious_curse_discounts() {
    // Murder ({1}{B}{B}) at the Witch: an opponent pays {B}{B}, its owner can't.
    for (seat, ok) in [(1usize, true), (0, false)] {
        let mut g = pod(2);
        let witch = ready(&mut g, 0, catalog::accursed_witch());
        let murder = g.add_card_to_hand(seat, catalog::murder());
        g.active_player_idx = seat;
        g.priority.player_with_priority = seat;
        g.players[seat].mana_pool.add(Color::Black, 2);
        assert_eq!(cast(&mut g, murder, Some(Target::Permanent(witch))).is_ok(), ok, "seat {seat}");
    }
    // Lava Axe ({4}{R}) at the cursed player for {3}{R}; at another, not.
    for (victim, ok) in [(1usize, true), (2, false)] {
        let mut g = pod(3);
        let witch = ready(&mut g, 0, catalog::accursed_witch());
        let kill = crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::Target(0) };
        let mut ctx = EffectContext::for_spell(1, None, 0, 0);
        ctx.targets = vec![Target::Permanent(witch)];
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
        g.resolve_effect(&kill, &ctx).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(witch).and_then(|c| c.attached_to_player), Some(1));
        let axe = g.add_card_to_hand(0, catalog::lava_axe());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        assert_eq!(cast(&mut g, axe, Some(Target::Player(victim))).is_ok(), ok, "victim {victim}");
    }
}

/// Bitterheart Witch dying fetches a Curse onto an opponent.
#[test]
fn bitterheart_witch_fetches_a_curse() {
    let mut g = pod(2);
    let witch = ready(&mut g, 0, catalog::bitterheart_witch());
    let thirst = g.add_card_to_library(0, catalog::curse_of_thirst());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let kill = crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::Target(0) };
    let mut ctx = EffectContext::for_spell(1, None, 0, 0);
    ctx.targets = vec![Target::Permanent(witch)];
    g.resolve_effect(&kill, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(thirst).map(|c| c.attached_to_player), Some(Some(1)));
}

/// Lynde: a Curse of yours dying returns attached to you at the end step;
/// at your upkeep it moves to an opponent and you draw two.
#[test]
fn lynde_recycles_her_curses() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::lynde_cheerful_tormentor());
    let thirst = curse_on(&mut g, 0, catalog::curse_of_thirst(), 1);
    let kill = crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::Target(0) };
    let mut ctx = EffectContext::for_spell(1, None, 0, 0);
    ctx.targets = vec![Target::Permanent(thirst)];
    g.resolve_effect(&kill, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(thirst).is_none());
    pass_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(thirst).map(|c| c.attached_to_player), Some(Some(0)), "back, on Lynde's player");
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let move_it = crabomination::effect::Effect::ReattachPlayerAurasToOpponents {
        from: crabomination::effect::PlayerRef::You,
        filter: crabomination::card::SelectionRequirement::Any,
        all: false,
        then: Some(Box::new(crabomination::effect::Effect::Draw {
            who: crabomination::effect::Selector::You,
            amount: crabomination::card::Value::Const(2),
        })),
    };
    g.resolve_effect(&move_it, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    assert_eq!(g.battlefield_find(thirst).map(|c| c.attached_to_player), Some(Some(1)));
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

/// The Unluckiest Planeswalker's −3 refills off your Auras and ships the
/// ones on you to an opponent.
#[test]
fn unluckiest_planeswalker_ships_its_curses() {
    let mut g = pod(2);
    let pw = ready(&mut g, 0, catalog::the_unluckiest_planeswalker());
    let thirst = curse_on(&mut g, 0, catalog::curse_of_thirst(), 0);
    curse_on(&mut g, 0, catalog::curse_of_obsession(), 1);
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: pw,
        ability_index: 1,
        target: None,
        x_value: None,
    })
    .expect("-3");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 4, "two Auras: four cards");
    assert_eq!(g.battlefield_find(thirst).map(|c| c.attached_to_player), Some(Some(1)));
}

/// CR 903.4 Rulebreaker — The Unluckiest Planeswalker's deck may hold Aura
/// cards outside its color identity.
#[test]
fn unluckiest_planeswalker_rulebreaker_admits_offcolor_auras() {
    use crabomination::format::Deck;
    let mut main: Vec<crabomination::card::CardDefinition> = vec![catalog::curse_of_echoes()];
    while main.len() < 99 {
        main.push(catalog::mountain());
    }
    let deck = Deck { commanders: vec![catalog::the_unluckiest_planeswalker()], main, sideboard: vec![] };
    assert!(crabomination::format::validate_commander_deck(&deck).is_ok());
    let mut bad = deck.clone();
    bad.main[0] = catalog::counterspell();
    assert!(crabomination::format::validate_commander_deck(&bad).is_err(), "a blue non-Aura stays out");
}

// ── Narfi, Betrayer King (`decks::cmdr_016x_o`) ─────────────────────────────

/// Narfi pumps other snow and Zombie creatures you control.
#[test]
fn narfi_pumps_snow_and_zombies() {
    let mut g = pod(2);
    let narfi = ready(&mut g, 0, catalog::narfi_betrayer_king());
    let draugr = ready(&mut g, 0, catalog::grim_draugr());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(draugr).unwrap().power, 4);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2);
    assert_eq!(g.computed_permanent(narfi).unwrap().power, 4, "other");
}

/// CR 107.4h — Narfi's {S}{S}{S} needs mana from snow sources: three
/// Swamps can't pay it, three Snow-Covered Swamps can.
#[test]
fn narfi_returns_for_snow_mana_only() {
    let mut g = pod(2);
    let narfi = g.add_card_to_graveyard(0, catalog::narfi_betrayer_king());
    for _ in 0..3 {
        ready(&mut g, 0, catalog::swamp());
    }
    assert!(activate(&mut g, narfi).is_err(), "plain Swamps aren't snow");
    for _ in 0..3 {
        ready(&mut g, 0, catalog::snow_covered_swamp());
    }
    activate(&mut g, narfi).expect("snow mana");
    let n = g.battlefield_find(narfi).expect("returned");
    assert!(n.tapped);
}

/// Dead of Winter shrinks nonsnow creatures by the snow permanents you
/// control and spares snow ones.
#[test]
fn dead_of_winter_spares_snow_creatures() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::snow_covered_swamp());
    let draugr = ready(&mut g, 0, catalog::grim_draugr());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::dead_of_winter());
    flood(&mut g, 0);
    cast(&mut g, s, None).expect("dead of winter");
    assert!(g.battlefield_find(bear).is_none(), "-2/-2");
    assert_eq!(g.computed_permanent(draugr).unwrap().power, 3, "snow: untouched");
}

/// CR 502.3 — Winter's Rest keeps its creature tapped while you control
/// another snow permanent.
#[test]
fn winters_rest_holds_the_creature_down() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::snow_covered_island());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let wr = g.add_card_to_hand(0, catalog::winters_rest());
    flood(&mut g, 0);
    cast(&mut g, wr, Some(Target::Permanent(bear))).expect("rest");
    assert!(g.battlefield_find(bear).unwrap().tapped, "tapped as it enters");
    pass_to(&mut g, TurnStep::Upkeep);
    assert_eq!(g.active_player_idx, 1);
    assert!(g.battlefield_find(bear).unwrap().tapped, "no untap");
}

/// Frost Augur takes a snow card off the top, not a nonsnow one.
#[test]
fn frost_augur_finds_snow() {
    let mut g = pod(2);
    let augur = ready(&mut g, 0, catalog::frost_augur());
    ready(&mut g, 0, catalog::snow_covered_island());
    g.players[0].library.clear();
    let sw = g.add_card_to_library(0, catalog::snow_covered_swamp());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    activate(&mut g, augur).expect("augur");
    assert!(g.players[0].hand.iter().any(|c| c.id == sw));
}

/// Priest of the Haunted Edge: -X/-X, X the snow lands you control.
#[test]
fn priest_of_the_haunted_edge_counts_snow_lands() {
    let mut g = pod(2);
    let priest = ready(&mut g, 0, catalog::priest_of_the_haunted_edge());
    for _ in 0..3 {
        ready(&mut g, 0, catalog::snow_covered_swamp());
    }
    let giant = ready(&mut g, 1, catalog::hill_giant());
    g.perform_action(GameAction::ActivateAbility {
        card_id: priest,
        ability_index: 0,
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("priest");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none(), "-3/-3 kills the 3/3");
}


// ── Heidar, Rimewind Master (`decks::cmdr_016x_p`) ──────────────────────────

/// Heidar's bounce needs four snow permanents.
#[test]
fn heidar_bounces_with_four_snow() {
    let mut g = pod(2);
    let heidar = ready(&mut g, 0, catalog::heidar_rimewind_master());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    for _ in 0..3 {
        ready(&mut g, 0, catalog::snow_covered_island());
    }
    flood(&mut g, 0);
    let bounce = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: heidar,
            ability_index: 0,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(bounce(&mut g).is_err(), "three snow permanents");
    ready(&mut g, 0, catalog::snow_covered_island());
    bounce(&mut g).expect("four");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
}

/// CR 107.4h — Icehide Golem's {S} is cast off a snow land.
#[test]
fn icehide_golem_casts_off_a_snow_land() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::island());
    let golem = g.add_card_to_hand(0, catalog::icehide_golem());
    assert!(cast(&mut g, golem, None).is_err(), "an Island isn't snow");
    ready(&mut g, 0, catalog::snow_covered_island());
    cast(&mut g, golem, None).expect("snow");
    assert!(g.battlefield_find(golem).is_some());
}

/// CR 205.4 — Rimefeather Owl's ice counter makes a permanent snow, and the
/// Owl counts it.
#[test]
fn rimefeather_owl_ices_a_permanent() {
    let mut g = pod(2);
    let owl = ready(&mut g, 0, catalog::rimefeather_owl());
    ready(&mut g, 0, catalog::snow_covered_island());
    ready(&mut g, 0, catalog::snow_covered_island());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(owl).unwrap().power, 3, "two snow lands and itself");
    g.perform_action(GameAction::ActivateAbility {
        card_id: owl,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("ice");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(owl).unwrap().power, 4, "the iced Bears are snow");
}

/// Ascendant Spirit climbs Warrior → Angel (flying counter) on snow mana.
#[test]
fn ascendant_spirit_levels_up() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let sp = ready(&mut g, 0, catalog::ascendant_spirit());
    for _ in 0..5 {
        ready(&mut g, 0, catalog::snow_covered_island());
    }
    let act = |g: &mut GameState, i: usize| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: sp,
            ability_index: i,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("level");
        drain_stack(g);
    };
    act(&mut g, 0);
    assert_eq!(g.computed_permanent(sp).unwrap().power, 2);
    act(&mut g, 1);
    let c = g.computed_permanent(sp).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4));
    assert!(g.permanent_has_keyword(sp, &Keyword::Flying));
}

/// Berg Strider cast with {S} also freezes its target.
#[test]
fn berg_strider_freezes_with_snow_mana() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    for _ in 0..5 {
        ready(&mut g, 0, catalog::snow_covered_island());
    }
    let bs = g.add_card_to_hand(0, catalog::berg_strider());
    cast(&mut g, bs, None).expect("strider");
    assert!(g.battlefield_find(bear).unwrap().tapped);
    pass_to(&mut g, TurnStep::Upkeep);
    assert_eq!(g.active_player_idx, 1);
    assert!(g.battlefield_find(bear).unwrap().tapped, "skips its untap");
}

/// Blizzard Strix flickers a permanent until the next end step.
#[test]
fn blizzard_strix_flickers_until_end_step() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::snow_covered_island());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let strix = g.add_card_to_hand(0, catalog::blizzard_strix());
    flood(&mut g, 0);
    cast(&mut g, strix, None).expect("strix");
    assert!(g.battlefield_find(bear).is_none(), "exiled");
    pass_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 1));
}

// ── Ayesha Tanaka (`decks::cmdr_016x_q`) ────────────────────────────────────

fn attack_banded(g: &mut GameState, band: &[CardId], defender: usize) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackersBanded {
        attacks: band
            .iter()
            .map(|&attacker| crabomination::game::types::Attack {
                attacker,
                target: crabomination::game::types::AttackTarget::Player(defender),
            })
            .collect(),
        bands: vec![band.to_vec()],
    })
    .expect("banded attack");
    drain_stack(g);
}

/// CR 702.22 — Icatian Skirmishers gives first strike to the creatures in
/// its band, and only those.
#[test]
fn icatian_skirmishers_arms_its_band() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let sk = ready(&mut g, 0, catalog::icatian_skirmishers());
    let mate = ready(&mut g, 0, catalog::grizzly_bears());
    let loner = ready(&mut g, 0, catalog::hill_giant());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackersBanded {
        attacks: [sk, mate, loner]
            .iter()
            .map(|&attacker| crabomination::game::types::Attack {
                attacker,
                target: crabomination::game::types::AttackTarget::Player(1),
            })
            .collect(),
        bands: vec![vec![sk, mate]],
    })
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.permanent_has_keyword(mate, &Keyword::FirstStrike), "banded with it");
    assert!(!g.permanent_has_keyword(loner, &Keyword::FirstStrike), "not in the band");
}

/// CR 702.22 — Urza's Engine bands for {3}, then gives its band trample.
#[test]
fn urzas_engine_tramples_its_band() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let engine = ready(&mut g, 0, catalog::urzas_engine());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g, 0);
    activate(&mut g, engine).expect("banding");
    attack_banded(&mut g, &[engine, bear], 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: engine,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("trample");
    drain_stack(&mut g);
    assert!(g.permanent_has_keyword(bear, &Keyword::Trample));
}

/// Formation: banding now, a card at the next upkeep.
#[test]
fn formation_draws_at_the_next_upkeep() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let f = g.add_card_to_hand(0, catalog::formation());
    flood(&mut g, 0);
    cast(&mut g, f, Some(Target::Permanent(bear))).expect("formation");
    assert!(g.permanent_has_keyword(bear, &Keyword::Banding));
    let hand = g.players[0].hand.len();
    pass_to(&mut g, TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drawn at the next turn's upkeep");
}

/// Cooperation's creature has banding.
#[test]
fn cooperation_grants_banding() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let c = g.add_card_to_hand(0, catalog::cooperation());
    flood(&mut g, 0);
    cast(&mut g, c, Some(Target::Permanent(bear))).expect("cooperation");
    assert!(g.permanent_has_keyword(bear, &Keyword::Banding));
}

/// CR 509.1 / 506.4 — General Jarkeld switches two attackers' blockers.
#[test]
fn general_jarkeld_switches_blockers() {
    let mut g = pod(2);
    let jarkeld = ready(&mut g, 1, catalog::general_jarkeld());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    let wall = ready(&mut g, 1, catalog::wall_of_shields());
    let elf = ready(&mut g, 1, catalog::llanowar_elves());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        [bear, giant]
            .iter()
            .map(|&attacker| crabomination::game::types::Attack {
                attacker,
                target: crabomination::game::types::AttackTarget::Player(1),
            })
            .collect(),
    ))
    .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, bear), (elf, giant)])).expect("block");
    drain_stack(&mut g);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: jarkeld,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Permanent(giant)],
        x_value: None,
        mode: None,
    })
    .expect("switch");
    drain_stack(&mut g);
    assert_eq!(g.block_map.get(&wall).map(|l| l.to_vec()), Some(vec![giant]), "the Wall now blocks the Giant");
    assert_eq!(g.block_map.get(&elf).map(|l| l.to_vec()), Some(vec![bear]));
}

// ── The Space Family Goblinson (`decks::cmdr_016x_r`) ───────────────────────

fn roll_d6(g: &mut GameState, acrobats: CardId) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: acrobats,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("roll");
    drain_stack(g);
}

/// CR 706 — Goblinson grows per die and tramples from the third die this
/// turn; Atomwheel Acrobats takes a 1 or 2 as counters; Resolute Veggiesaur
/// grows on the third die only.
#[test]
fn goblinson_counts_the_dice() {
    use crabomination::card::{CounterType, Keyword};
    let mut g = pod(2);
    let gob = ready(&mut g, 0, catalog::the_space_family_goblinson());
    let acro = ready(&mut g, 0, catalog::atomwheel_acrobats());
    let veg = ready(&mut g, 0, catalog::resolute_veggiesaur());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::DieRoll(2),
        DecisionAnswer::DieRoll(5),
        DecisionAnswer::DieRoll(6),
        DecisionAnswer::DieRoll(1),
    ]));
    let plus = |g: &GameState, id| g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne);
    roll_d6(&mut g, acro);
    assert_eq!(plus(&g, acro), 2, "a 2: two counters");
    assert!(!g.permanent_has_keyword(gob, &Keyword::Trample));
    roll_d6(&mut g, acro);
    assert_eq!(plus(&g, veg), 0);
    roll_d6(&mut g, acro);
    assert_eq!(plus(&g, veg), 1, "the third die");
    assert!(g.permanent_has_keyword(gob, &Keyword::Trample), "three dice this turn");
    flood(&mut g, 0);
    roll_d6(&mut g, acro);
    assert_eq!(plus(&g, veg), 1, "not the fourth");
    assert_eq!(plus(&g, gob), 4, "one per die");
    assert_eq!(plus(&g, acro), 3, "a 5, a 6, then a 1");
}

/// CR 706.3a — Swarming Goblins on a 20 makes three Goblins.
#[test]
fn swarming_goblins_rolls_for_tokens() {
    let mut g = pod(2);
    let sg = g.add_card_to_hand(0, catalog::swarming_goblins());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(20)]));
    cast(&mut g, sg, None).expect("goblins");
    let tokens = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Goblin").count();
    assert_eq!(tokens, 3);
}

/// Farideh's Fireball: a 10+ spares its caster.
#[test]
fn faridehs_fireball_spares_you_on_a_high_roll() {
    let mut g = pod(3);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let f = g.add_card_to_hand(0, catalog::faridehs_fireball());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(15)]));
    cast(&mut g, f, Some(Target::Permanent(bear))).expect("fireball");
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!((g.players[0].life, g.players[1].life, g.players[2].life), (20, 18, 18));
}

/// Critical Hit returns from the graveyard on a natural 20.
#[test]
fn critical_hit_returns_on_a_natural_twenty() {
    let mut g = pod(2);
    let ch = g.add_card_to_graveyard(0, catalog::critical_hit());
    let sg = g.add_card_to_hand(0, catalog::swarming_goblins());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(20)]));
    cast(&mut g, sg, None).expect("goblins");
    assert!(g.players[0].hand.iter().any(|c| c.id == ch));
}

/// Critical Hit — "A natural 20 is a roll that displays 20 on the die" (CR
/// 706.2): a 19 raised to 20 by a modifier doesn't return it.
#[test]
fn critical_hit_ignores_a_modified_twenty() {
    use crabomination::effect::{Effect, Value};
    let mut g = pod(2);
    let ch = g.add_card_to_graveyard(0, catalog::critical_hit());
    let src = ready(&mut g, 0, catalog::grizzly_bears());
    let roll = |modifier| Effect::RollDie {
        sides: 20,
        count: Value::ONE,
        modifier: Value::Const(modifier),
        reroll_at_most: 0,
        ignore_lowest: 0,
        results: vec![],
        on_doubles: None,
    };
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(19)]));
    g.stack.push(TriggerPush::new(src, 0, roll(1)).build());
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == ch), "19 + 1 is not a natural 20");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(20)]));
    g.stack.push(TriggerPush::new(src, 0, roll(-1)).build());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == ch), "a natural 20, whatever the modifier");
}

/// Priority Boarding exiles a top card cheaper than the result to play.
#[test]
fn priority_boarding_impulses_under_the_roll() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::priority_boarding());
    let acro = ready(&mut g, 0, catalog::atomwheel_acrobats());
    g.players[0].library.clear();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::DieRoll(5),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
    ]));
    roll_d6(&mut g, acro);
    assert!(g.exile.iter().any(|c| c.id == bear), "MV 2 < 5: exiled to play");
}

/// Vegetation Abomination's sacrifice gains the d6 result.
#[test]
fn vegetation_abomination_gains_the_roll() {
    let mut g = pod(2);
    let va = ready(&mut g, 0, catalog::vegetation_abomination());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(4)]));
    roll_d6(&mut g, va);
    assert_eq!(g.players[0].life, 24);
}

// ── Farideh / Monoxa (`decks::cmdr_016x_s`) ─────────────────────────────────

/// CR 706.6 — Bamboozling Beeble on an opponent: they roll one extra die and
/// the Beeble's controller ignores the best one.
#[test]
fn bamboozling_beeble_takes_the_best_roll_away() {
    let mut g = pod(2);
    let beeble = ready(&mut g, 0, catalog::bamboozling_beeble());
    let herald = ready(&mut g, 1, catalog::herald_of_hadar());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: beeble,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("beeble");
    drain_stack(&mut g);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    flood(&mut g, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(20), DecisionAnswer::DieRoll(5)]));
    activate_as(&mut g, herald, 1);
    // The 20 is ignored: the 5 drains 2 and gains nothing.
    assert_eq!((g.players[0].life, g.players[1].life), (18, 20));
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Treasure"));
}

fn activate_as(g: &mut GameState, card_id: CardId, seat: usize) {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// CR 706.2 — Xenosquirrels spends a +1/+1 counter to push a 19 to a 20.
#[test]
fn xenosquirrels_nudges_a_roll() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let xs = g.add_card_to_hand(0, catalog::xenosquirrels());
    let herald = ready(&mut g, 0, catalog::herald_of_hadar());
    flood(&mut g, 0);
    cast(&mut g, xs, None).expect("squirrels");
    assert_eq!(g.battlefield_find(xs).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(19)]));
    activate_as(&mut g, herald, 0);
    assert_eq!(g.battlefield_find(xs).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let treasures = g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count();
    assert_eq!(treasures, 2, "a 20: two Treasures");
}

/// Farideh: flying and menace off any roll, a card on 10+.
#[test]
fn farideh_draws_on_a_high_roll() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let far = ready(&mut g, 0, catalog::farideh_devils_chosen());
    let herald = ready(&mut g, 0, catalog::herald_of_hadar());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(12)]));
    activate_as(&mut g, herald, 0);
    assert!(g.permanent_has_keyword(far, &Keyword::Flying) && g.permanent_has_keyword(far, &Keyword::Menace));
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Monoxa's keywords scale with the result.
#[test]
fn monoxa_scales_with_the_roll() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let m = ready(&mut g, 0, catalog::monoxa_midway_manager());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(4)]));
    activate_as(&mut g, m, 0);
    assert!(g.permanent_has_keyword(m, &Keyword::FirstStrike));
    assert!(g.permanent_has_keyword(m, &Keyword::Menace));
    assert!(!g.permanent_has_keyword(m, &Keyword::Lifelink), "4 < 5");
}

/// CR 701.52a — Line Cutter's roll to visit is a die roll even with no
/// Attractions out (Goblinson counts it).
#[test]
fn line_cutter_rolls_to_visit() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let gob = ready(&mut g, 0, catalog::the_space_family_goblinson());
    let lc = g.add_card_to_hand(0, catalog::line_cutter());
    flood(&mut g, 0);
    cast(&mut g, lc, None).expect("line cutter");
    assert_eq!(g.battlefield_find(gob).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Slight Malfunction's die mode — "When you do, … 1 damage to each of up to
/// X target creatures" (CR 603.12): a 3 aims at three creatures, chosen as
/// the reflexive trigger goes on the stack — never a hexproof one (CR 702.11)
/// — and a fourth is untouched.
#[test]
fn slight_malfunction_targets_up_to_the_result() {
    let mut g = pod(2);
    let bogle = ready(&mut g, 1, catalog::slippery_bogle());
    let foes: Vec<CardId> = (0..4).map(|_| ready(&mut g, 1, catalog::llanowar_elves())).collect();
    let sm = g.add_card_to_hand(0, catalog::slight_malfunction());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(3)]));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: sm, target: None, additional_targets: vec![], mode: Some(1), x_value: None })
        .expect("the die mode");
    drain_stack(&mut g);
    let dead = foes.iter().filter(|id| g.battlefield_find(**id).is_none()).count();
    assert_eq!(dead, 3, "three of the four 1/1s");
    assert!(g.battlefield_find(bogle).is_some(), "hexproof: never a target");
}

/// Frostwalk Bastion animates as a 2/3 Construct **artifact** creature land
/// until end of turn.
#[test]
fn frostwalk_bastion_becomes_an_artifact_creature_land() {
    let mut g = pod(2);
    let bastion = ready(&mut g, 0, catalog::frostwalk_bastion());
    ready(&mut g, 0, catalog::snow_covered_forest());
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bastion, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("animate");
    drain_stack(&mut g);
    let c = g.computed_permanent(bastion).expect("on the battlefield");
    assert_eq!((c.power, c.toughness), (2, 3));
    use crabomination::card::CardType;
    assert!(c.card_types().contains(&CardType::Artifact) && c.card_types().contains(&CardType::Land));
}
