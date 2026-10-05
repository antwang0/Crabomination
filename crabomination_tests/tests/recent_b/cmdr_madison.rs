//! Commander: the Science! precon (PIP, Dr. Madison Li, `decks::cmdr_madison`),
//! and its primitives: a graveyard-cast grant capped by the source's counters,
//! abilities of exiled cards with a counter whoever owns them, the permanent
//! exiled for a cost as a copy source (CR 707.2), a secret-number match, a
//! per-artifact attack tax, and "return some, bottom the rest".

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::effect::{Effect, PlayerRef, Selector};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.turn_number = 3;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..12 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast_x(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    cast_x(g, seat, id, target, None)
}

fn activate_x(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: x,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    activate_x(g, seat, id, index, target, None)
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

/// Arcade Gannon — its graveyard cast is capped by its quest counters.
#[test]
fn arcade_gannon_casts_up_to_its_quest_counters() {
    let mut g = pod(2);
    let ag = g.add_card_to_battlefield(0, catalog::arcade_gannon());
    g.battlefield_find_mut(ag).unwrap().add_counters(CounterType::Quest, 1);
    let arena = g.add_card_to_graveyard(0, catalog::phyrexian_arena());
    let thopter = g.add_card_to_graveyard(0, catalog::ornithopter());
    flood(&mut g, 0);
    assert!(cast(&mut g, 0, arena, None).is_err(), "not an artifact or Human");
    cast(&mut g, 0, thopter, None).expect("Ornithopter (0) under one quest counter");
    assert_eq!(named(&g, 0, "Ornithopter").len(), 1);
}

/// Arcade Gannon — mana value above the quest counters is refused.
#[test]
fn arcade_gannon_cap_refuses_pricier_spells() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::arcade_gannon());
    let sol = g.add_card_to_graveyard(0, catalog::sol_ring());
    flood(&mut g, 0);
    assert!(cast(&mut g, 0, sol, None).is_err(), "Sol Ring (1) over zero quest counters");
}

/// Rex — pay {E}{E}: exile a creature card from any graveyard with a brain
/// counter; Rex then has its activated abilities.
#[test]
fn rex_borrows_brain_countered_abilities() {
    let mut g = pod(2);
    let rex = g.add_card_to_battlefield(0, catalog::rex_cyber_hound());
    g.clear_sickness(rex);
    let elves = g.add_card_to_graveyard(1, catalog::llanowar_elves());
    g.players[0].energy = 2;
    activate(&mut g, 0, rex, 0, Some(Target::Permanent(elves))).expect("brain it");
    let ex = g.exile.iter().find(|c| c.id == elves).expect("exiled");
    assert_eq!(ex.counter_count(CounterType::Brain), 1);
    activate(&mut g, 0, rex, 1, None).expect("Llanowar Elves' ability, on Rex");
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

/// CR 707.2 — Curie becomes a copy of the artifact creature it exiled, and
/// keeps its combat-damage draw.
#[test]
fn cr_707_2_curie_copies_the_exiled_creature() {
    let mut g = pod(2);
    let curie = g.add_card_to_battlefield(0, catalog::curie_emergent_intelligence());
    let sol = g.add_card_to_battlefield(0, catalog::solemn_simulacrum());
    flood(&mut g, 0);
    activate(&mut g, 0, curie, 0, None).expect("exile Solemn");
    assert!(g.exile.iter().any(|c| c.id == sol));
    let c = g.battlefield_find(curie).unwrap();
    assert_eq!(c.definition.name, "Solemn Simulacrum");
    assert!(
        c.definition.triggered_abilities.iter().any(|t| t.event.kind == crabomination::card::EventKind::DealsCombatDamageToPlayer),
        "keeps its draw trigger"
    );
}

/// Expert-Level Safe — a match cracks it: the stash goes to hand.
#[test]
fn expert_level_safe_eventually_cracks() {
    let mut g = pod(2);
    flood(&mut g, 0);
    let safe = g.add_card_to_hand(0, catalog::expert_level_safe());
    cast(&mut g, 0, safe, None).expect("Safe");
    let stash = g.exile.iter().filter(|c| c.exiled_with == Some(safe)).count();
    assert_eq!(stash, 2);
    let hand = g.players[0].hand.len();
    for _ in 0..200 {
        if g.battlefield_find(safe).is_none() {
            break;
        }
        g.battlefield_find_mut(safe).unwrap().tapped = false;
        g.players[0].mana_pool.add_colorless(1);
        activate(&mut g, 0, safe, 0, Some(Target::Player(1))).expect("crack attempt");
    }
    assert!(g.battlefield_find(safe).is_none(), "a match sacrificed it");
    assert!(g.players[0].hand.len() >= hand + 2, "every stashed card came to hand");
}

/// Expert-Level Safe — a prompting seat names its secret number (each
/// seat's own ask): matching picks crack it, differing ones stash a card.
#[test]
fn expert_level_safe_prompting_seats_name_their_numbers() {
    use crabomination::decision::{Decision, DecisionAnswer};
    for (picks, cracks) in [([1u32, 1], true), ([0, 2], false)] {
        let mut g = pod(2);
        flood(&mut g, 0);
        let safe = g.add_card_to_hand(0, catalog::expert_level_safe());
        cast(&mut g, 0, safe, None).expect("Safe");
        g.players[0].wants_ui = true;
        g.players[1].wants_ui = true;
        g.players[0].mana_pool.add_colorless(1);
        let _ = activate(&mut g, 0, safe, 0, Some(Target::Player(1)));
        let mut asked = Vec::new();
        for &i in &picks {
            let Some(p) = g.pending_decision.as_ref() else { break };
            assert!(matches!(p.decision, Decision::ChooseOption { .. }), "{:?}", p.decision);
            asked.push(p.acting_player());
            g.submit_decision(DecisionAnswer::Amount(i)).expect("answer");
            drain_stack(&mut g);
        }
        assert_eq!(asked, vec![0, 1], "you, then the target opponent");
        assert!(g.pending_decision.is_none());
        let stash = g.exile.iter().filter(|c| c.exiled_with == Some(safe)).count();
        assert_eq!(g.battlefield_find(safe).is_none(), cracks);
        assert_eq!(stash, if cracks { 0 } else { 3 });
    }
}

/// Overencumbered — the enchanted opponent gets three tokens, and must pay
/// {1} per artifact at combat or no creature attacks.
#[test]
fn overencumbered_taxes_the_attack() {
    let mut g = pod(2);
    flood(&mut g, 0);
    let oe = g.add_card_to_hand(0, catalog::overencumbered());
    cast(&mut g, 0, oe, Some(Target::Player(1))).expect("Overencumbered");
    for name in ["Clue", "Food", "Junk"] {
        assert_eq!(named(&g, 1, name).len(), 1, "{name}");
    }
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 1;
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    let r = g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }]));
    assert!(r.is_err(), "no mana for {{3}}: nobody attacks");
}

/// Vault 13 III — two exiled cards return, the rest go to the bottom.
#[test]
fn vault_13_returns_two_bottoms_the_rest() {
    let mut g = pod(2);
    let saga = g.add_card_to_battlefield(0, catalog::vault_13_dwellers_journey());
    let ids: Vec<CardId> = (0..3).map(|_| g.add_card_to_battlefield(1, catalog::grizzly_bears())).collect();
    let ctx = EffectContext::for_ability(saga, 0, None);
    for &id in &ids {
        g.resolve_effect(
            &Effect::ExileUntilSourceLeaves {
                what: Selector::ExactObjects(vec![id]),
                return_to: crabomination::card::ExileReturnZone::Battlefield,
            },
            &ctx,
        )
        .expect("exile");
    }
    g.resolve_effect(&Effect::ReturnSomeExiledWithSourceRestToBottom { count: 2 }, &ctx).expect("III");
    let back = ids.iter().filter(|&&id| g.battlefield_find(id).is_some()).count();
    assert_eq!(back, 2);
    let bottom = g.players[1].library.last().map(|c| c.id);
    assert!(ids.contains(&bottom.unwrap()), "the third is on the bottom");
}

/// HELIOS One — pay X energy: destroy a nonland permanent with mana value X.
#[test]
fn helios_one_spends_x_energy() {
    let mut g = pod(2);
    let helios = g.add_card_to_battlefield(0, catalog::helios_one());
    let arena = g.add_card_to_battlefield(1, catalog::phyrexian_arena());
    g.players[0].energy = 3;
    g.players[0].mana_pool.add_colorless(3);
    activate_x(&mut g, 0, helios, 2, Some(Target::Permanent(arena)), Some(3)).expect("X = 3");
    assert!(g.battlefield_find(arena).is_none());
    assert_eq!(g.players[0].energy, 0);
}

/// Electrosiphon — counter a spell, energy equal to its mana value.
#[test]
fn electrosiphon_banks_the_mana_value() {
    let mut g = pod(2);
    let arena = g.add_card_to_hand(1, catalog::phyrexian_arena());
    flood(&mut g, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: arena, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("Arena on the stack");
    flood(&mut g, 0);
    let es = g.add_card_to_hand(0, catalog::electrosiphon());
    cast(&mut g, 0, es, Some(Target::Permanent(arena))).expect("Electrosiphon");
    assert_eq!(g.players[0].energy, 3);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == arena));
}

/// Bottle-Cap Blast — excess damage to a creature becomes tapped Treasures.
#[test]
fn bottle_cap_blast_excess_treasures() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    flood(&mut g, 0);
    let bcb = g.add_card_to_hand(0, catalog::bottle_cap_blast());
    cast(&mut g, 0, bcb, Some(Target::Permanent(bear))).expect("Blast");
    let t = named(&g, 0, "Treasure");
    assert_eq!(t.len(), 3, "5 damage to a 2-toughness Bear");
    assert!(t.iter().all(|&id| g.battlefield_find(id).unwrap().tapped));
}

/// T-45 Power Armor — the wearer doesn't untap; paying {E} at upkeep untaps
/// it with a keyword counter.
#[test]
fn t45_power_armor_untaps_for_energy() {
    let mut g = pod(2);
    flood(&mut g, 0);
    let armor = g.add_card_to_hand(0, catalog::t_45_power_armor());
    cast(&mut g, 0, armor, None).expect("armor");
    assert_eq!(g.players[0].energy, 2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: armor, target: bear }).expect("equip");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (5, 5));
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
        crabomination::decision::DecisionAnswer::Mode(1),
    ]));
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped for {{E}}");
    assert_eq!(g.players[0].energy, 1);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Trample));
}

/// Dr. Madison Li — an artifact spell nets {E}; three pay for a card.
#[test]
fn madison_li_energy_engine() {
    let mut g = pod(2);
    let li = g.add_card_to_battlefield(0, catalog::dr_madison_li());
    g.clear_sickness(li);
    flood(&mut g, 0);
    let sol = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, 0, sol, None).expect("Sol Ring");
    assert_eq!(g.players[0].energy, 1);
    g.players[0].energy = 3;
    let hand = g.players[0].hand.len();
    activate(&mut g, 0, li, 1, None).expect("draw");
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.players[0].energy, 0);
}

/// Brotherhood Scribe — energy on your turn pumps your team.
#[test]
fn brotherhood_scribe_pumps_on_energy() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::brotherhood_scribe());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(&Effect::AddEnergy(crabomination::card::Value::Const(2)), &ctx).expect("energy");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3));
    let _ = PlayerRef::You;
}

/// Plasma Caster — "target creature that's blocking EQUIPPED creature": a
/// blocker on another attacker isn't a legal target (it read any blocker).
#[test]
fn plasma_caster_targets_only_the_equipped_creatures_blocker() {
    use crabomination::card::SelectionRequirement as R;
    let mut g = two_player_game();
    let (a, b) = (g.add_card_to_battlefield(0, catalog::grizzly_bears()), g.add_card_to_battlefield(0, catalog::grizzly_bears()));
    let caster = g.add_card_to_battlefield(0, catalog::plasma_caster());
    g.battlefield_find_mut(caster).unwrap().attached_to = Some(a);
    let (x, y) = (g.add_card_to_battlefield(1, catalog::grizzly_bears()), g.add_card_to_battlefield(1, catalog::grizzly_bears()));
    for id in [a, b] {
        g.clear_sickness(id);
    }
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ])
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(x, a), (y, b)])).expect("block");
    let legal = |t| g.evaluate_requirement_static(&R::BlockingHostOfSource, &Target::Permanent(t), 0, Some(caster));
    assert!(legal(x), "blocking the equipped creature");
    assert!(!legal(y), "blocking another attacker");
}

/// Endurance Bobblehead — up to X target creatures you control (X = one
/// Bobblehead here) get +1/+0 and indestructible: the chosen one, not the
/// biggest (CR 601.2c).
#[test]
fn endurance_bobblehead_targets_up_to_x() {
    let mut g = pod(2);
    let eb = g.add_card_to_battlefield(0, catalog::endurance_bobblehead());
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: eb,
        ability_index: 1,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Permanent(wurm)],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let cp = |id| g.computed_permanent(id).unwrap();
    assert_eq!(cp(bear).power, 3);
    assert!(cp(bear).keywords().contains(&Keyword::Indestructible));
    assert_eq!(cp(wurm).power, 6, "X = 1: the second target is dropped");
}

/// Vault 112, chapter III: the energy paid exiles that many cards from the
/// top (not a reveal: the rest stay in exile, CR 406.3), and one nonland card
/// among them may be cast free as the chapter resolves (CR 608.2g).
#[test]
fn vault_112_chapter_three_exiles_and_casts_one() {
    use crabomination::game::effects::EffectContext;
    let mut g = pod(2);
    let saga = g.add_card_to_battlefield(0, catalog::vault_112_sadistic_simulation());
    g.players[0].library.clear(); // the chapter shuffles: just these two
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let island = g.add_card_to_library(0, catalog::island());
    g.players[0].energy = 2;
    g.players[0].lands_played_this_turn = 1; // no land play: the land isn't offered
    let chapter = g.battlefield_find(saga).unwrap().definition.saga_chapters[2].1.clone();
    let events = g.resolve_effect(&chapter, &EffectContext::for_trigger(saga, 0, None, 0)).expect("chapter III");
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 0, "both paid");
    assert!(g.battlefield_find(bear).is_some(), "the creature card cast free");
    assert!(g.exile.iter().any(|c| c.id == island), "the land stays exiled");
}

/// Vault 112, chapter III: "play one of those cards" includes a land — on
/// your turn with a land play left (CR 305.2 / 305.3), and it uses that play.
#[test]
fn vault_112_chapter_three_plays_a_land() {
    use crabomination::game::effects::EffectContext;
    let run = |played: u32| {
        let mut g = pod(2);
        g.active_player_idx = 0;
        let saga = g.add_card_to_battlefield(0, catalog::vault_112_sadistic_simulation());
        g.players[0].library.clear();
        let island = g.add_card_to_library(0, catalog::island());
        g.players[0].energy = 1;
        g.players[0].lands_played_this_turn = played;
        let chapter = g.battlefield_find(saga).unwrap().definition.saga_chapters[2].1.clone();
        let events = g.resolve_effect(&chapter, &EffectContext::for_trigger(saga, 0, None, 0)).expect("chapter III");
        g.dispatch_triggers_for_events(&events);
        drain_stack(&mut g);
        (g.battlefield_find(island).is_some(), g.players[0].lands_played_this_turn)
    };
    assert_eq!(run(0), (true, 1), "played as the land drop");
    assert_eq!(run(1), (false, 1), "no land play left: it stays exiled");
}
