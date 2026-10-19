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
