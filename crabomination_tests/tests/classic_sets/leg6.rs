//! Legends (LEG) wave 7 — the set's last creatures, artifacts, Auras and
//! spells (`catalog::sets::leg6`).

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

/// Sacrifice a battlefield permanent, firing its dies trigger (CR 701.16).
fn kill(g: &mut GameState, id: CardId) {
    let ctl = g.battlefield_find(id).unwrap().controller;
    let ctx = crabomination::game::effects::EffectContext::for_ability(
        id,
        ctl,
        Some(Target::Permanent(id)),
    );
    g.resolve_effect(
        &crabomination::effect::Effect::SacrificePermanent {
            what: crabomination::effect::Selector::Target(0),
        },
        &ctx,
    )
    .expect("sacrifice");
    drain_stack(g);
}

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

fn activate(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Seat `atk_seat`'s `attacker` swings and `blocker` blocks.
fn block(g: &mut GameState, atk_seat: usize, attacker: CardId, blocker: CardId) {
    g.clear_sickness(attacker);
    g.active_player_idx = atk_seat;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = atk_seat;
    g.declare_attackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1 - atk_seat),
    }])
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).expect("block");
    drain_stack(g);
}

fn to_end_of_combat(g: &mut GameState) {
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(g);
}

/// Hand the turn to `seat`, running the untap step's roll-overs.
fn start_turn_of(g: &mut GameState, seat: usize) {
    g.active_player_idx = seat;
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(g);
}

// ── Creatures ──────────────────────────────────────────────────────────────

/// Blazing Effigy's death bolt counts the burn an earlier copy put into it.
#[test]
fn blazing_effigy_adds_damage_from_its_namesakes() {
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = main_phase();
    let effigy = g.add_card_to_battlefield(0, catalog::blazing_effigy());
    // A second Effigy, parked in a graveyard, is only here to be the named
    // damage source.
    let namesake = g.add_card_to_graveyard(1, catalog::blazing_effigy());
    let turtle = g.add_card_to_battlefield(1, catalog::giant_turtle());
    let ctx = crabomination::game::effects::EffectContext::for_ability(
        namesake,
        1,
        Some(Target::Permanent(effigy)),
    );
    g.resolve_effect(
        &Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(2) },
        &ctx,
    )
    .expect("ping");
    kill(&mut g, effigy);
    assert!(g.battlefield_find(turtle).is_none(), "3 + 2 kills a 2/4");
}

/// Brine Hag permanently shrinks whatever killed it. The first-striking
/// Knight takes nothing back, so it survives to be measured at 0/2.
#[test]
fn brine_hag_shrinks_its_killers_to_0_2() {
    let mut g = main_phase();
    let hag = g.add_card_to_battlefield(0, catalog::brine_hag());
    let knight = g.add_card_to_battlefield(1, catalog::white_knight());
    block(&mut g, 1, knight, hag);
    to_end_of_combat(&mut g);
    let pt = g.computed_permanent(knight).expect("alive");
    assert_eq!((pt.power, pt.toughness), (0, 2));
}

/// Giant Turtle sits out the turn after it attacks.
#[test]
fn giant_turtle_cant_attack_two_turns_running() {
    let mut g = main_phase();
    let turtle = g.add_card_to_battlefield(0, catalog::giant_turtle());
    g.clear_sickness(turtle);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: turtle, target: AttackTarget::Player(1) }])
        .expect("first swing");
    start_turn_of(&mut g, 1);
    start_turn_of(&mut g, 0);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert!(
        g.declare_attackers(vec![Attack { attacker: turtle, target: AttackTarget::Player(1) }])
            .is_err(),
        "attacked last turn"
    );
}

/// Wall of Dust benches its blocker for that player's next turn.
#[test]
fn wall_of_dust_benches_what_it_blocks() {
    let mut g = main_phase();
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_dust());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    block(&mut g, 1, giant, wall);
    start_turn_of(&mut g, 0);
    start_turn_of(&mut g, 1);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    assert!(
        g.declare_attackers(vec![Attack { attacker: giant, target: AttackTarget::Player(0) }])
            .is_err(),
        "banned this turn"
    );
}

/// Halfdane copies the target's base P/T on your upkeep.
#[test]
fn halfdane_takes_a_rivals_body() {
    let mut g = main_phase();
    let dane = g.add_card_to_battlefield(0, catalog::halfdane());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    let _ = giant;
    let pt = g.computed_permanent(dane).expect("alive");
    assert_eq!((pt.power, pt.toughness), (3, 3));
}

/// Sentinel's toughness is set to one more than what it's blocking.
#[test]
fn sentinel_sets_toughness_off_its_blocker() {
    let mut g = main_phase();
    let sentinel = g.add_card_to_battlefield(0, catalog::sentinel());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    block(&mut g, 0, sentinel, giant);
    activate(&mut g, 0, sentinel, Some(Target::Permanent(giant)));
    let pt = g.computed_permanent(sentinel).expect("alive");
    assert_eq!(pt.toughness, 4, "1 + the Giant's 3");
}

// ── Artifacts ──────────────────────────────────────────────────────────────

/// Sword of the Ages bills the whole sacrifice batch, then exiles it.
#[test]
fn sword_of_the_ages_throws_the_boards_total_power() {
    let mut g = main_phase();
    let sword = g.add_card_to_battlefield(0, catalog::sword_of_the_ages());
    let a = g.add_card_to_battlefield(0, catalog::hill_giant());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(sword).unwrap().tapped = false;
    activate(&mut g, 0, sword, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 20 - 5, "3 + 2 total power");
    assert!(g.exile.iter().any(|c| c.id == a), "the sacrificed bodies are exiled");
    assert!(g.exile.iter().any(|c| c.id == b));
    assert!(g.exile.iter().any(|c| c.id == sword));
}

// ── Enchantments and Auras ─────────────────────────────────────────────────

/// Backfire reflects the enchanted creature's hit onto its controller.
#[test]
fn backfire_reflects_onto_the_creatures_controller() {
    let mut g = main_phase();
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let aura = g.add_card_to_hand(0, catalog::backfire());
    cast(&mut g, 0, aura, Some(Target::Permanent(giant)));
    g.clear_sickness(giant);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker: giant, target: AttackTarget::Player(0) }])
        .expect("attack");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 17, "took the hit");
    assert_eq!(g.players[1].life, 17, "and gave it back");
}

/// Greater Realm of Preservation stops a red source's damage.
#[test]
fn greater_realm_of_preservation_shields_from_red() {
    let mut g = main_phase();
    let realm = g.add_card_to_battlefield(0, catalog::greater_realm_of_preservation());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    // The shield names a source, so the Bolt has to be on the stack first.
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    activate(&mut g, 0, realm, None);
    assert_eq!(g.players[0].life, 20, "prevented");
}

// ── Spells ─────────────────────────────────────────────────────────────────

/// Enchantment Alteration walks an Aura onto a new host.
#[test]
fn enchantment_alteration_moves_the_aura() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::hill_giant());
    let aura = g.add_card_to_hand(0, catalog::spirit_link());
    cast(&mut g, 0, aura, Some(Target::Permanent(a)));
    let spell = g.add_card_to_hand(0, catalog::enchantment_alteration());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(aura)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(aura).unwrap().attached_to, Some(b));
}

/// Hellfire bills its caster three plus its own body count.
#[test]
fn hellfire_bills_its_caster_for_its_own_kills() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::hellfire());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.players[0].life, 20 - 5, "3 + two dead nonblack creatures");
}

/// Eureka keeps going around the table while anyone can still deploy.
#[test]
fn eureka_empties_permanents_from_every_hand() {
    let mut g = main_phase();
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::hill_giant());
    g.add_card_to_hand(1, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::eureka());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_creature()).count(), 3);
}

/// Petra Sphinx bins the top card when the guess misses.
#[test]
fn petra_sphinx_bins_a_missed_guess() {
    let mut g = main_phase();
    let sphinx = g.add_card_to_battlefield(0, catalog::petra_sphinx());
    g.clear_sickness(sphinx);
    g.players[1].library.clear();
    g.add_card_to_library(1, catalog::hill_giant());
    g.add_card_to_library(1, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::grizzly_bears());
    let before = g.players[1].graveyard.len();
    activate(&mut g, 0, sphinx, Some(Target::Player(1)));
    assert_eq!(g.players[1].graveyard.len(), before + 1, "named the denser card, missed");
}

/// Disharmony pulls an attacker out of combat and under your control.
#[test]
fn disharmony_steals_an_attacker() {
    let mut g = main_phase();
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::disharmony());
    g.clear_sickness(giant);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker: giant, target: AttackTarget::Player(0) }])
        .expect("attack");
    cast(&mut g, 0, spell, Some(Target::Permanent(giant)));
    assert_eq!(g.battlefield_find(giant).unwrap().controller, 0);
    assert!(!g.attacking.iter().any(|a| a.attacker == giant), "out of combat");
}

/// Giant Slug's landwalk arrives on the upkeep after you pay.
#[test]
fn giant_slug_gains_landwalk_next_upkeep() {
    let mut g = main_phase();
    let slug = g.add_card_to_battlefield(0, catalog::giant_slug());
    activate(&mut g, 0, slug, None);
    assert!(
        !g.computed_permanent(slug)
            .unwrap()
            .keywords
            .iter()
            .any(|k| matches!(k, Keyword::Landwalk(_))),
        "not yet"
    );
    start_turn_of(&mut g, 1);
    start_turn_of(&mut g, 0);
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(slug)
            .unwrap()
            .keywords
            .iter()
            .any(|k| matches!(k, Keyword::Landwalk(_))),
        "chosen on the upkeep"
    );
}


