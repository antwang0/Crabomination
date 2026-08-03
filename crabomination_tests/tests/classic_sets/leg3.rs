//! Legends (LEG) wave 4 — Walls, mana batteries, the plain legends and the
//! set's one-line spells (`catalog::sets::leg3`).

use crabomination::card::{CardDefinition, CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

/// Tap `id` for real, so its `Tapped` triggers fire.
fn tap(g: &mut GameState, id: CardId) {
    g.battlefield_find_mut(id).unwrap().tapped = true;
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped {
        card_id: id,
        actor: None,
        as_attacker: false,
    }]);
    drain_stack(g);
}

/// Seat 0's `attacker` swings at seat 1, optionally blocked, through to damage.
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
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
}

// ── Walls ──────────────────────────────────────────────────────────────────

/// CR 615 — Wall of Vapor soaks its attacker's whole swing.
#[test]
fn wall_of_vapor_prevents_damage_from_what_it_blocks() {
    let mut g = main_phase();
    let attacker = g.add_card_to_battlefield(0, catalog::hill_giant());
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_vapor());
    combat(&mut g, attacker, Some(wall));
    assert_eq!(g.battlefield_find(wall).unwrap().damage, 0);
    assert!(g.battlefield_find(wall).is_some(), "the 0/1 survives a 3/3");
}

/// The prevention is scoped: an unrelated source still gets through.
#[test]
fn wall_of_vapor_only_stops_the_creature_it_blocks() {
    let mut g = main_phase();
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_vapor());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Permanent(wall)));
    assert!(g.battlefield_find(wall).is_none(), "Bolt is not prevented");
}

/// Wall of Tombstones' upkeep trigger sets base toughness from the graveyard.
#[test]
fn wall_of_tombstones_grows_with_the_graveyard() {
    let mut g = main_phase();
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_tombstones());
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(wall).unwrap().toughness, 4, "1 + 3 creature cards");
}

// ── The mana batteries ─────────────────────────────────────────────────────

/// Charge up, then cash in: one mana plus one per counter removed.
#[test]
fn black_mana_battery_pays_out_per_charge_counter() {
    let mut g = main_phase();
    let battery = g.add_card_to_battlefield(0, catalog::black_mana_battery());
    g.clear_sickness(battery);
    for _ in 0..2 {
        activate(&mut g, 0, battery, 0, None);
        g.battlefield_find_mut(battery).unwrap().tapped = false;
    }
    assert_eq!(g.battlefield_find(battery).unwrap().counter_count(CounterType::Charge), 2);

    g.players[0].mana_pool.empty();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: battery,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 3, "one {{B}} plus one per counter");
    assert_eq!(g.battlefield_find(battery).unwrap().counter_count(CounterType::Charge), 0);
}

/// The cycle covers all five colours.
#[test]
fn the_mana_battery_cycle_covers_every_colour() {
    let cases: &[(fn() -> CardDefinition, &str)] = &[
        (catalog::white_mana_battery, "White Mana Battery"),
        (catalog::blue_mana_battery, "Blue Mana Battery"),
        (catalog::black_mana_battery, "Black Mana Battery"),
        (catalog::red_mana_battery, "Red Mana Battery"),
        (catalog::green_mana_battery, "Green Mana Battery"),
    ];
    for (factory, name) in cases {
        let def = factory();
        assert_eq!(def.name, *name);
        assert_eq!(def.activated_abilities.len(), 2, "{name}");
        assert_eq!(def.cost.cmc(), 4, "{name}");
    }
}

// ── Creatures ──────────────────────────────────────────────────────────────

/// Elder Land Wurm sheds defender the moment it blocks.
#[test]
fn elder_land_wurm_loses_defender_when_it_blocks() {
    let mut g = main_phase();
    let wurm = g.add_card_to_battlefield(1, catalog::elder_land_wurm());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(wurm).unwrap().keywords.contains(&Keyword::Defender));
    combat(&mut g, attacker, Some(wurm));
    assert!(!g.computed_permanent(wurm).unwrap().keywords.contains(&Keyword::Defender));
}

/// Ivory Guardians' anthem only switches on against a red opponent.
#[test]
fn ivory_guardians_grow_against_a_red_board() {
    let mut g = main_phase();
    let guard = g.add_card_to_battlefield(0, catalog::ivory_guardians());
    assert_eq!(g.computed_permanent(guard).unwrap().power, 3);
    g.add_card_to_battlefield(1, catalog::mons_goblin_raiders());
    assert_eq!(g.computed_permanent(guard).unwrap().power, 4);
}

/// Beasts of Bogardan reads the opposing board the same way.
#[test]
fn beasts_of_bogardan_grow_against_a_white_board() {
    let mut g = main_phase();
    let beast = g.add_card_to_battlefield(0, catalog::beasts_of_bogardan());
    assert_eq!(g.computed_permanent(beast).unwrap().toughness, 3);
    g.add_card_to_battlefield(1, catalog::savannah_lions());
    assert_eq!(g.computed_permanent(beast).unwrap().toughness, 4);
}

/// Tor Wauki only shoots creatures already in combat.
#[test]
fn tor_wauki_shoots_an_attacker() {
    let mut g = main_phase();
    let wauki = g.add_card_to_battlefield(0, catalog::tor_wauki());
    g.clear_sickness(wauki);
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(
        activate_result(&mut g, wauki, Target::Permanent(attacker)).is_err(),
        "not in combat yet"
    );

    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.clear_sickness(attacker);
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(0) }]).expect("attack");
    activate(&mut g, 0, wauki, 0, Some(Target::Permanent(attacker)));
    assert!(g.battlefield_find(attacker).is_none(), "2 damage kills the 2/2");
}

fn activate_result(
    g: &mut GameState,
    card_id: CardId,
    target: Target,
) -> Result<(), crabomination::game::types::GameError> {
    mana(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: 0,
        target: Some(target),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map(|_| ())
}

/// The plain legends carry their printed stats and the Legendary supertype.
#[test]
fn the_plain_legends_carry_their_printed_bodies() {
    use crabomination::card::Supertype;
    let cases: &[(fn() -> CardDefinition, i32, i32)] = &[
        (catalog::sir_shandlar_of_eberyn, 4, 7),
        (catalog::sivitri_scarzam, 6, 4),
        (catalog::the_lady_of_the_mountain, 5, 5),
        (catalog::tobias_andrion, 4, 4),
        (catalog::torsten_von_ursus, 5, 5),
    ];
    for (factory, p, t) in cases {
        let def = factory();
        assert_eq!((def.power, def.toughness), (*p, *t), "{}", def.name);
        assert!(def.supertypes.contains(&Supertype::Legendary), "{}", def.name);
    }
}

/// Sol'kanar drinks off every black spell, including an opponent's.
#[test]
fn solkanar_gains_life_on_any_black_spell() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::solkanar_the_swamp_king());
    let before = g.players[0].life;
    let spell = g.add_card_to_hand(1, catalog::dark_ritual());
    cast(&mut g, 1, spell, None);
    assert_eq!(g.players[0].life, before + 1);
}

// ── Spells ─────────────────────────────────────────────────────────────────

/// Syphon Soul drains for the damage it actually dealt.
#[test]
fn syphon_soul_drains_two() {
    let mut g = main_phase();
    let life = g.players[0].life;
    let spell = g.add_card_to_hand(0, catalog::syphon_soul());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.players[1].life, 20 - 2);
    assert_eq!(g.players[0].life, life + 2);
}

/// Storm Seeker scales off the victim's hand.
#[test]
fn storm_seeker_reads_the_target_hand() {
    let mut g = main_phase();
    for _ in 0..4 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::storm_seeker());
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 20 - 4);
}

/// Untamed Wilds puts the basic straight onto the battlefield.
#[test]
fn untamed_wilds_fetches_a_basic_untapped() {
    let mut g = main_phase();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let spell = g.add_card_to_hand(0, catalog::untamed_wilds());
    cast(&mut g, 0, spell, None);
    let land = g.battlefield.iter().find(|c| c.definition.name == "Forest").expect("fetched");
    assert!(!land.tapped);
}

/// Reset untaps only your own lands.
#[test]
fn reset_untaps_your_lands() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::island());
    let theirs = g.add_card_to_battlefield(1, catalog::island());
    g.battlefield_find_mut(mine).unwrap().tapped = true;
    g.battlefield_find_mut(theirs).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::reset());
    cast(&mut g, 0, spell, None);
    assert!(!g.battlefield_find(mine).unwrap().tapped);
    assert!(g.battlefield_find(theirs).unwrap().tapped);
}

// ── Enchantments ───────────────────────────────────────────────────────────

/// Presence of the Master counters every enchantment, including its owner's.
#[test]
fn presence_of_the_master_counters_enchantments() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::presence_of_the_master());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::pacifism());
    cast(&mut g, 0, aura, Some(Target::Permanent(bear)));
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Pacifism"));
}

/// In the Eye of Chaos taxes an instant by its own mana value.
#[test]
fn in_the_eye_of_chaos_taxes_instants_by_mana_value() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::in_the_eye_of_chaos());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "Bolt is countered — the {{1}} tax went unpaid");
}

/// Caverns of Despair caps the swing at two.
#[test]
fn caverns_of_despair_caps_attackers_at_two() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::caverns_of_despair());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [a, b, c] {
        g.clear_sickness(id);
    }
    g.step = TurnStep::DeclareAttackers;
    let swing: Vec<Attack> =
        [a, b, c].iter().map(|&x| Attack { attacker: x, target: AttackTarget::Player(1) }).collect();
    assert!(g.declare_attackers(swing).is_err(), "three is one too many");
    assert!(
        g.declare_attackers(vec![
            Attack { attacker: a, target: AttackTarget::Player(1) },
            Attack { attacker: b, target: AttackTarget::Player(1) },
        ])
        .is_ok()
    );
}

/// Living Plane makes every land a 1/1 that's still a land.
#[test]
fn living_plane_animates_all_lands() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(1, catalog::island());
    g.add_card_to_battlefield(0, catalog::living_plane());
    let cp = g.computed_permanent(land).expect("animated");
    assert!(cp.card_types.contains(&crabomination::card::CardType::Creature));
    assert!(cp.card_types.contains(&crabomination::card::CardType::Land));
    assert_eq!((cp.power, cp.toughness), (1, 1));
}

// ── Auras ──────────────────────────────────────────────────────────────────

/// Spirit Link pays its *own* controller, even on a stolen host.
#[test]
fn spirit_link_pays_the_aura_controller() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let link = g.add_card_to_hand(0, catalog::spirit_link());
    cast(&mut g, 0, link, Some(Target::Permanent(bear)));
    let before = g.players[0].life;
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.clear_sickness(bear);
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }])
        .expect("attack");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[0].life, before - 2 + 2, "took 2, gained 2");
}

/// Spirit Shackle stacks a -0/-2 counter every time the host taps.
#[test]
fn spirit_shackle_shrinks_the_host_on_tap() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::hill_giant());
    let aura = g.add_card_to_hand(0, catalog::spirit_shackle());
    cast(&mut g, 0, aura, Some(Target::Permanent(bear)));
    tap(&mut g, bear);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::MinusZeroMinusTwo), 1);
    assert_eq!(g.computed_permanent(bear).unwrap().toughness, 1, "3 - 2");
}

/// Blight kills the land the turn it's tapped for mana.
#[test]
fn blight_destroys_the_land_when_it_taps() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let aura = g.add_card_to_hand(0, catalog::blight());
    cast(&mut g, 0, aura, Some(Target::Permanent(land)));
    tap(&mut g, land);
    assert!(g.battlefield_find(land).is_none());
}

/// Spectral Cloak's shroud is untapped-only.
#[test]
fn spectral_cloak_shroud_switches_off_when_tapped() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::spectral_cloak());
    cast(&mut g, 0, aura, Some(Target::Permanent(bear)));
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Shroud));
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Shroud));
}

/// Demonic Torment benches the host in both directions.
#[test]
fn demonic_torment_benches_the_host() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::demonic_torment());
    cast(&mut g, 0, aura, Some(Target::Permanent(bear)));
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    assert!(
        g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
            .is_err()
    );
}

/// Seeker's host walks past anything that isn't an artifact or white.
#[test]
fn seeker_restricts_who_can_block() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::seeker());
    cast(&mut g, 0, aura, Some(Target::Permanent(bear)));
    let green = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(green, bear)])).is_err());
}

// ── Artifacts ──────────────────────────────────────────────────────────────

/// Serpent Generator's Snakes poison on contact.
#[test]
fn serpent_generator_snakes_poison() {
    let mut g = main_phase();
    let forge = g.add_card_to_battlefield(0, catalog::serpent_generator());
    g.clear_sickness(forge);
    activate(&mut g, 0, forge, 0, None);
    let snake = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Snake")
        .map(|c| c.id)
        .expect("token");
    combat(&mut g, snake, None);
    assert_eq!(g.players[1].poison_counters, 1);
}

/// Life Chisel only works at upkeep, and pays the sacrificed toughness.
#[test]
fn life_chisel_is_upkeep_only() {
    let mut g = main_phase();
    let chisel = g.add_card_to_battlefield(0, catalog::life_chisel());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: chisel,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "main phase is too late"
    );

    let life = g.players[0].life;
    g.step = TurnStep::Upkeep;
    activate(&mut g, 0, chisel, 0, None);
    assert_eq!(g.players[0].life, life + 2, "the Bears' toughness");
}

/// Horn of Deafening blanks one attacker's damage.
#[test]
fn horn_of_deafening_silences_an_attacker() {
    let mut g = main_phase();
    let horn = g.add_card_to_battlefield(1, catalog::horn_of_deafening());
    g.clear_sickness(horn);
    let attacker = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]).expect("attack");
    activate(&mut g, 1, horn, 0, Some(Target::Permanent(attacker)));
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[1].life, 20);
}

/// Avoid Fate protects your own permanent from a targeted instant.
#[test]
fn avoid_fate_counters_a_spell_aimed_at_your_board() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    mana(&mut g, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    let fate = g.add_card_to_hand(0, catalog::avoid_fate());
    cast(&mut g, 0, fate, Some(Target::Permanent(bolt)));
    assert!(g.battlefield_find(bear).is_some(), "Bolt was countered");
}
