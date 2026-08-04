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
    // Enough library for the tests that run several turns out.
    for seat in 0..2 {
        for _ in 0..20 {
            g.add_card_to_library(seat, catalog::mountain());
        }
    }
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

/// Run to the start of the next turn, so untap-step roll-overs really fire.
fn end_turn(g: &mut GameState) {
    let started = g.turn_number;
    while g.turn_number == started {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

/// Hand the turn to `seat` (one full turn cycle per call is two `end_turn`s).
fn start_turn_of(g: &mut GameState, seat: usize) {
    for _ in 0..4 {
        if g.active_player_idx == seat && g.step == TurnStep::Upkeep {
            return;
        }
        end_turn(g);
    }
    panic!("never reached seat {seat}");
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



// ── Wave 7b ────────────────────────────────────────────────────────────────

/// Ayesha Tanaka counters an artifact's ability when its controller can't pay.
#[test]
fn ayesha_tanaka_counters_an_unpaid_artifact_ability() {
    let mut g = main_phase();
    let ayesha = g.add_card_to_battlefield(0, catalog::ayesha_tanaka());
    g.clear_sickness(ayesha);
    let doll = g.add_card_to_battlefield(1, catalog::voodoo_doll());
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: doll,
        ability_index: 0,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        x_value: Some(0),
        mode: None,
    })
    .expect("activate");
    let before = g.stack.len();
    activate(&mut g, 0, ayesha, Some(Target::Permanent(doll)));
    assert!(g.stack.len() < before, "the ability is gone");
    assert_eq!(g.players[0].life, 20);
}

/// Cocoon locks its host down for three upkeeps, then upgrades it.
#[test]
fn cocoon_locks_then_upgrades_its_host() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::cocoon());
    cast(&mut g, 0, aura, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).unwrap().tapped, "tapped on entry");
    // Three upkeeps shed the counters; the fourth untap frees it and the
    // fourth upkeep cashes the Cocoon in.
    for _ in 0..4 {
        start_turn_of(&mut g, 1);
        start_turn_of(&mut g, 0);
        drain_stack(&mut g);
    }
    assert!(!g.battlefield_find(bear).unwrap().tapped, "the last counter came off");
    let pt = g.computed_permanent(bear).unwrap();
    assert!(pt.keywords.contains(&Keyword::Flying));
    assert!(g.battlefield_find(aura).is_none(), "the Cocoon is spent");
}

/// Rasputin arrives with seven dreams and spends one for mana.
#[test]
fn rasputin_spends_a_dream_for_mana() {
    use crabomination::card::CounterType;
    let mut g = main_phase();
    let card = g.add_card_to_hand(0, catalog::rasputin_dreamweaver());
    cast(&mut g, 0, card, None);
    let rasputin = card;
    assert_eq!(g.battlefield_find(rasputin).unwrap().counter_count(CounterType::Dream), 7);
    let before = g.players[0].mana_pool.total();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: rasputin,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    assert_eq!(g.battlefield_find(rasputin).unwrap().counter_count(CounterType::Dream), 6);
    assert_eq!(g.players[0].mana_pool.total(), before + 1);
}

/// Voodoo Doll gains a pin each upkeep and blows up on an untapped end step.
#[test]
fn voodoo_doll_punishes_you_for_leaving_it_untapped() {
    use crabomination::card::CounterType;
    let mut g = main_phase();
    let doll = g.add_card_to_battlefield(0, catalog::voodoo_doll());
    // Tapped, it survives its own end step and just banks pins.
    g.battlefield_find_mut(doll).unwrap().tapped = true;
    end_turn(&mut g);
    start_turn_of(&mut g, 0);
    assert_eq!(g.battlefield_find(doll).unwrap().counter_count(CounterType::Pin), 1);
    g.battlefield_find_mut(doll).unwrap().tapped = false;
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(doll).is_none(), "destroyed");
    assert_eq!(g.players[0].life, 19);
}

/// Johan trades his own attack for a board-wide vigilance.
#[test]
fn johan_keeps_the_team_untapped() {
    let mut g = main_phase();
    let johan = g.add_card_to_battlefield(0, catalog::johan());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.clear_sickness(johan);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
        .expect("attack");
    assert!(!g.battlefield_find(bear).unwrap().tapped, "vigilance");
    assert!(
        g.declare_attackers(vec![Attack { attacker: johan, target: AttackTarget::Player(1) }])
            .is_err(),
        "Johan sat this one out"
    );
}

/// Gabriel Angelfire picks up one of its four abilities each upkeep.
#[test]
fn gabriel_angelfire_gains_a_chosen_ability() {
    let mut g = main_phase();
    let gabriel = g.add_card_to_battlefield(0, catalog::gabriel_angelfire());
    start_turn_of(&mut g, 0);
    drain_stack(&mut g);
    assert!(g.computed_permanent(gabriel).unwrap().keywords.contains(&Keyword::Flying));
}

/// Nova Pentacle hands the next hit to a creature the opponent controls.
#[test]
fn nova_pentacle_redirects_onto_their_creature() {
    let mut g = main_phase();
    let pentacle = g.add_card_to_battlefield(0, catalog::nova_pentacle());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
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
    activate(&mut g, 0, pentacle, Some(Target::Permanent(bear)));
    assert_eq!(g.players[0].life, 20, "not me");
    assert!(g.battlefield_find(bear).is_none(), "the Bear ate it");
}

/// Puppet Master buys the dead creature back to its owner's hand.
#[test]
fn puppet_master_returns_the_dead_creature() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::puppet_master());
    cast(&mut g, 0, aura, Some(Target::Permanent(bear)));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Permanent(bear)));
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "back to its owner");
}

/// Relic Bind pings when the enchanted artifact taps.
#[test]
fn relic_bind_pings_on_a_tap() {
    let mut g = main_phase();
    let doll = g.add_card_to_battlefield(1, catalog::voodoo_doll());
    let aura = g.add_card_to_hand(0, catalog::relic_bind());
    cast(&mut g, 0, aura, Some(Target::Permanent(doll)));
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: doll,
        ability_index: 0,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        x_value: Some(0),
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "the tap cost them a point");
}

/// Floral Spuzzem trades its damage for an artifact.
#[test]
fn floral_spuzzem_smashes_an_artifact_instead() {
    let mut g = main_phase();
    let spuzzem = g.add_card_to_battlefield(0, catalog::floral_spuzzem());
    let doll = g.add_card_to_battlefield(1, catalog::voodoo_doll());
    g.clear_sickness(spuzzem);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: spuzzem, target: AttackTarget::Player(1) }])
        .expect("attack");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(doll).is_none(), "the Doll is gone");
    assert_eq!(g.players[1].life, 20, "and no damage got through");
}

/// Remove Enchantments takes yours back and sweeps the rest.
#[test]
fn remove_enchantments_returns_yours_and_destroys_the_rest() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mine = g.add_card_to_hand(0, catalog::spirit_link());
    cast(&mut g, 0, mine, Some(Target::Permanent(bear)));
    let spell = g.add_card_to_hand(0, catalog::remove_enchantments());
    cast(&mut g, 0, spell, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == mine), "yours comes home");
}



// ── Wave 7c ────────────────────────────────────────────────────────────────

/// Nebuchadnezzar strips the copies of the name he calls.
#[test]
fn nebuchadnezzar_discards_the_named_copies() {
    let mut g = main_phase();
    let neb = g.add_card_to_battlefield(0, catalog::nebuchadnezzar());
    g.clear_sickness(neb);
    g.players[1].hand.clear();
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: neb,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: Some(3),
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.players[1].hand.is_empty(), "all three shared the named card");
}

/// Quarum Trench Gnomes make a Plains produce colorless, indefinitely.
#[test]
fn quarum_trench_gnomes_turn_a_plains_colorless() {
    let mut g = main_phase();
    let gnomes = g.add_card_to_battlefield(0, catalog::quarum_trench_gnomes());
    g.clear_sickness(gnomes);
    let plains = g.add_card_to_battlefield(1, catalog::plains());
    activate(&mut g, 0, gnomes, Some(Target::Permanent(plains)));
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: plains,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap for mana");
    assert_eq!(g.players[1].mana_pool.amount(Color::White), 0, "no white");
    assert_eq!(g.players[1].mana_pool.total(), 1, "one colorless instead");
}

/// Juxtapose trades the biggest creature on each side.
#[test]
fn juxtapose_swaps_the_biggest_creatures() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::juxtapose());
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0);
    assert_eq!(g.battlefield_find(mine).unwrap().controller, 1);
}

/// Bronze Horse shrugs off a targeted burn spell while it has company.
#[test]
fn bronze_horse_ignores_targeted_burn_with_company() {
    let mut g = main_phase();
    let horse = g.add_card_to_battlefield(0, catalog::bronze_horse());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    // Alone, the Bolt lands.
    cast(&mut g, 1, bolt, Some(Target::Permanent(horse)));
    assert_eq!(g.battlefield_find(horse).unwrap().damage, 3);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt2 = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt2, Some(Target::Permanent(horse)));
    assert_eq!(g.battlefield_find(horse).unwrap().damage, 3, "the second is prevented");
}

/// Silhouette blanks targeted damage on a creature for the turn.
#[test]
fn silhouette_prevents_targeted_damage() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cover = g.add_card_to_hand(0, catalog::silhouette());
    cast(&mut g, 0, cover, Some(Target::Permanent(bear)));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_some(), "still alive");
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0);
}
