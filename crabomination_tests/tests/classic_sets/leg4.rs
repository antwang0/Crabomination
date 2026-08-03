//! Legends (LEG) wave 5 — the Elder Dragon cycle, the Glyph cycle and the
//! remaining legends, artifacts and spells (`catalog::sets::leg4`).

use crabomination::card::{CardDefinition, CardId, CounterType, Keyword};
use crabomination::catalog;
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

/// Seat 0 swings, seat 1 blocks, and combat runs to the end-of-combat step.
fn combat(g: &mut GameState, attacker: CardId, blocker: Option<CardId>) {
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]).expect("attack");
    if let Some(b) = blocker {
        while g.step != TurnStep::DeclareBlockers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareBlockers(vec![(b, attacker)])).expect("block");
        drain_stack(g);
    }
}

fn to_end_of_combat(g: &mut GameState) {
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(g);
}

// ── The Elder Dragon cycle ─────────────────────────────────────────────────

/// All five Elder Dragons are 7/7 fliers with an upkeep tithe.
#[test]
fn the_elder_dragon_cycle_shares_its_body() {
    let cases: &[fn() -> CardDefinition] = &[
        catalog::arcades_sabboth,
        catalog::chromium,
        catalog::nicol_bolas,
        catalog::palladia_mors,
        catalog::vaevictis_asmadi,
    ];
    for factory in cases {
        let def = factory();
        assert_eq!((def.power, def.toughness), (7, 7), "{}", def.name);
        assert!(def.keywords.contains(&Keyword::Flying), "{}", def.name);
        assert_eq!(def.cost.cmc(), 8, "{}", def.name);
        assert!(
            def.triggered_abilities.iter().any(|t| matches!(
                t.effect,
                crabomination::effect::Effect::SacrificeSourceUnlessPay { .. }
            )),
            "{} pays its upkeep",
            def.name
        );
    }
}

/// The unpaid upkeep sacrifices the dragon.
#[test]
fn palladia_mors_is_sacrificed_when_the_upkeep_goes_unpaid() {
    let mut g = main_phase();
    let dragon = g.add_card_to_battlefield(0, catalog::palladia_mors());
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(dragon).is_none(), "no {{R}}{{G}}{{W}} in the pool");
}

/// Arcades Sabboth props up your untapped, non-attacking creatures.
#[test]
fn arcades_sabboth_shields_untapped_defenders() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::arcades_sabboth());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bear).unwrap().toughness, 4);
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    assert_eq!(g.computed_permanent(bear).unwrap().toughness, 2, "tapped gets nothing");
}

/// Nicol Bolas strips the hand when it connects.
#[test]
fn nicol_bolas_empties_the_hand_on_damage() {
    let mut g = main_phase();
    let bolas = g.add_card_to_battlefield(0, catalog::nicol_bolas());
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    combat(&mut g, bolas, None);
    to_end_of_combat(&mut g);
    assert!(g.players[1].hand.is_empty());
}

// ── The Glyph cycle ────────────────────────────────────────────────────────

/// Glyph of Doom kills whatever the Wall stopped, at end of combat.
#[test]
fn glyph_of_doom_kills_what_the_wall_blocked() {
    let mut g = main_phase();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_heat());
    combat(&mut g, attacker, Some(wall));
    let glyph = g.add_card_to_hand(1, catalog::glyph_of_doom());
    cast(&mut g, 1, glyph, Some(Target::Permanent(wall)));
    assert!(g.battlefield_find(attacker).is_some(), "not yet — end of combat");
    to_end_of_combat(&mut g);
    assert!(g.battlefield_find(attacker).is_none());
}

/// Glyph of Delusion locks the blocked creature down under glyph counters.
#[test]
fn glyph_of_delusion_locks_the_blocked_creature() {
    let mut g = main_phase();
    let attacker = g.add_card_to_battlefield(0, catalog::hill_giant());
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_heat());
    combat(&mut g, attacker, Some(wall));
    let glyph = g.add_card_to_hand(1, catalog::glyph_of_delusion());
    cast(&mut g, 1, glyph, Some(Target::Permanent(attacker)));
    assert_eq!(g.battlefield_find(attacker).unwrap().counter_count(CounterType::Glyph), 3);
    assert!(
        g.computed_permanent(attacker)
            .unwrap()
            .keywords
            .contains(&Keyword::DoesntUntapWhileCounter(CounterType::Glyph))
    );
}

// ── Legends ────────────────────────────────────────────────────────────────

/// Bartel Runeaxe shrugs off Aura spells but not other removal.
#[test]
fn bartel_runeaxe_cant_be_targeted_by_auras() {
    let mut g = main_phase();
    let bartel = g.add_card_to_battlefield(0, catalog::bartel_runeaxe());
    let aura = g.add_card_to_hand(1, catalog::pacifism());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: aura,
            target: Some(Target::Permanent(bartel)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "an Aura can't target him"
    );
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(bartel)));
    assert_eq!(g.battlefield_find(bartel).unwrap().damage, 3, "Bolt still lands");
}

/// Livonya Silone walks past a defender with a legendary land.
#[test]
fn livonya_silone_has_legendary_landwalk() {
    let mut g = main_phase();
    let livonya = g.add_card_to_battlefield(0, catalog::livonya_silone());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::urborg_tomb_of_yawgmoth());
    combat(&mut g, livonya, None);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(blocker, livonya)])).is_err());
}

/// Rubinia keeps the creature only while she stays tapped.
#[test]
fn rubinia_soulsinger_holds_the_creature_while_tapped() {
    let mut g = main_phase();
    let rubinia = g.add_card_to_battlefield(0, catalog::rubinia_soulsinger());
    g.clear_sickness(rubinia);
    let prize = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, rubinia, 0, Some(Target::Permanent(prize)));
    assert_eq!(g.battlefield_find(prize).unwrap().controller, 0);
    g.battlefield_find_mut(rubinia).unwrap().tapped = false;
    g.check_state_based_actions();
    assert_eq!(g.battlefield_find(prize).unwrap().controller, 1, "untapping gives it back");
}

/// Tetsuo only kills tapped or blocking creatures.
#[test]
fn tetsuo_umezawa_kills_only_tapped_or_blocking() {
    let mut g = main_phase();
    let tetsuo = g.add_card_to_battlefield(0, catalog::tetsuo_umezawa());
    g.clear_sickness(tetsuo);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: tetsuo,
            ability_index: 0,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "untapped and out of combat"
    );
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    activate(&mut g, 0, tetsuo, 0, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none());
}

/// Evil Eye benches your non-Eyes and can only be chumped by Walls.
#[test]
fn evil_eye_benches_your_other_creatures() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::evil_eye_of_orms_by_gore());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    assert!(
        g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
            .is_err()
    );
}

/// Rabid Wombat scales with every Aura on it.
#[test]
fn rabid_wombat_grows_per_aura() {
    let mut g = main_phase();
    let wombat = g.add_card_to_battlefield(0, catalog::rabid_wombat());
    assert_eq!(g.computed_permanent(wombat).unwrap().power, 0);
    let aura = g.add_card_to_hand(0, catalog::spirit_link());
    cast(&mut g, 0, aura, Some(Target::Permanent(wombat)));
    assert_eq!(g.computed_permanent(wombat).unwrap().power, 2);
}

// ── Enchantments ───────────────────────────────────────────────────────────

/// The Abyss eats a nonartifact creature at each upkeep.
#[test]
fn the_abyss_eats_a_nonartifact_creature() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::the_abyss());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bot = g.add_card_to_battlefield(0, catalog::ornithopter());
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(bot).is_some(), "artifacts are safe");
}

/// Storm World burns a small hand.
#[test]
fn storm_world_burns_the_empty_handed() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::storm_world());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20 - 3, "4 minus one card in hand");
}

/// Angelic Voices only pumps while your board stays white (or artifact).
#[test]
fn angelic_voices_needs_a_white_board() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::angelic_voices());
    let lions = g.add_card_to_battlefield(0, catalog::savannah_lions());
    assert_eq!(g.computed_permanent(lions).unwrap().power, 3);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(lions).unwrap().power, 2, "a green creature turns it off");
}

/// Land's Edge lets either player pitch a land for two damage.
#[test]
fn lands_edge_is_open_to_both_players() {
    let mut g = main_phase();
    let edge = g.add_card_to_battlefield(0, catalog::lands_edge());
    g.add_card_to_hand(1, catalog::forest());
    activate(&mut g, 1, edge, 0, Some(Target::Player(0)));
    assert_eq!(g.players[0].life, 18);
}

// ── Artifacts and spells ───────────────────────────────────────────────────

/// Triassic Egg only cracks once it has two hatchling counters.
#[test]
fn triassic_egg_needs_two_counters() {
    let mut g = main_phase();
    let egg = g.add_card_to_battlefield(0, catalog::triassic_egg());
    g.clear_sickness(egg);
    let corpse = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: egg,
            ability_index: 1,
            target: Some(Target::Permanent(corpse)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "no counters yet"
    );
    for _ in 0..2 {
        activate(&mut g, 0, egg, 0, None);
        g.battlefield_find_mut(egg).unwrap().tapped = false;
    }
    activate(&mut g, 0, egg, 1, Some(Target::Permanent(corpse)));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"));
}

/// North Star relaxes the colours of your spells for the turn.
#[test]
fn north_star_lets_you_spend_any_colour() {
    let mut g = main_phase();
    let star = g.add_card_to_battlefield(0, catalog::north_star());
    g.clear_sickness(star);
    activate(&mut g, 0, star, 0, None);
    assert!(g.players[0].may_spend_any_color_this_turn);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.empty();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("green pays for a red spell");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
}

/// Blood Lust leaves the creature at exactly 1 toughness when it's small.
#[test]
fn blood_lust_leaves_one_toughness() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::blood_lust());
    cast(&mut g, 0, spell, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 1));
}

/// Typhoon reads the opponent's Islands.
#[test]
fn typhoon_counts_their_islands() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::island());
    }
    let spell = g.add_card_to_hand(0, catalog::typhoon());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.players[1].life, 17);
}

/// Energy Tap converts a creature into its mana value in {C}.
#[test]
fn energy_tap_pays_the_creature_mana_value() {
    let mut g = main_phase();
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::energy_tap());
    cast(&mut g, 0, spell, Some(Target::Permanent(giant)));
    assert!(g.battlefield_find(giant).unwrap().tapped);
    assert!(g.players[0].mana_pool.total() >= 4);
}

/// Hell's Caretaker trades a body for a better one, at upkeep only.
#[test]
fn hells_caretaker_reanimates_at_upkeep() {
    let mut g = main_phase();
    let caretaker = g.add_card_to_battlefield(0, catalog::hells_caretaker());
    g.clear_sickness(caretaker);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let corpse = g.add_card_to_graveyard(0, catalog::hill_giant());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: caretaker,
            ability_index: 0,
            target: Some(Target::Permanent(corpse)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "main phase is too late"
    );
    g.step = TurnStep::Upkeep;
    activate(&mut g, 0, caretaker, 0, Some(Target::Permanent(corpse)));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Hill Giant"));
}
