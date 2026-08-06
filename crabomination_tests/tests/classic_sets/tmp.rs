//! Tempest (TMP) — `catalog::sets::tmp`.

use crabomination::card::{CardDefinition, CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EntityRef;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

fn activate(
    g: &mut GameState,
    id: CardId,
    index: usize,
    target: Option<Target>,
) -> Result<(), crabomination::game::GameError> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map(|_| ())
}

fn cast(
    g: &mut GameState,
    id: CardId,
    target: Option<Target>,
) -> Result<(), crabomination::game::GameError> {
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map(|_| ())
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Put `def` on the battlefield ready to attack.
fn ready(g: &mut GameState, seat: usize, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

// ── Shadow ──────────────────────────────────────────────────────────────────

/// Shadow gates blocking both ways: a Dauthi can only be blocked by shadow,
/// and it can only block shadow.
#[test]
fn shadow_only_trades_with_shadow() {
    let mut g = two_player_game();
    let dauthi = ready(&mut g, 0, catalog::dauthi_marauder());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let soltari = ready(&mut g, 1, catalog::soltari_foot_soldier());
    assert!(!g.blocker_can_block_attacker(bear, dauthi), "a nonshadow blocker can't stop shadow");
    assert!(g.blocker_can_block_attacker(soltari, dauthi), "shadow blocks shadow");
}

/// Dauthi Ghoul grows on every shadow creature that dies, not on the rest.
#[test]
fn dauthi_ghoul_feeds_on_shadow_deaths() {
    let mut g = two_player_game();
    let ghoul = g.add_card_to_battlefield(0, catalog::dauthi_ghoul());
    let soltari = g.add_card_to_battlefield(1, catalog::soltari_foot_soldier());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let mut events = vec![];
    g.destroy_permanent(bear, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ghoul).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);

    let mut events = vec![];
    g.destroy_permanent(soltari, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ghoul).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Shadowstorm only burns the shadow half of the board.
#[test]
fn shadowstorm_spares_nonshadow_creatures() {
    let mut g = two_player_game();
    let soltari = g.add_card_to_battlefield(1, catalog::soltari_foot_soldier());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let storm = g.add_card_to_hand(0, catalog::shadowstorm());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, storm, None).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(soltari).is_none(), "1/1 shadow dies");
    assert!(g.battlefield_find(bear).is_some(), "the bear is untouched");
}

// ── Slivers ─────────────────────────────────────────────────────────────────

/// Armor Sliver hands its `{2}` pump to every Sliver, itself included.
#[test]
fn armor_sliver_grants_its_pump_to_every_sliver() {
    let mut g = two_player_game();
    let lord = g.add_card_to_battlefield(0, catalog::armor_sliver());
    let metallic = g.add_card_to_battlefield(0, catalog::metallic_sliver());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    let granted = |g: &GameState, id| g.granted_abilities_for(id).len();
    assert_eq!(granted(&g, metallic), 1, "the Sliver picks the ability up");
    assert_eq!(granted(&g, lord), 1, "so does the lord");
    assert_eq!(granted(&g, bear), 0, "the bear doesn't");

    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, metallic, 0, None).expect("granted pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(metallic).unwrap().toughness, 2);
}

// ── Engine primitives added for this set ────────────────────────────────────

/// `EventSpec::causer_filter` — Fugitive Druid cantrips off an Aura spell but
/// not off an ordinary targeted spell.
#[test]
fn fugitive_druid_only_draws_off_aura_spells() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let druid = g.add_card_to_battlefield(0, catalog::fugitive_druid());
    let growth = g.add_card_to_hand(0, catalog::giant_growth());
    g.players[0].mana_pool.add(Color::Green, 1);
    let before = g.players[0].hand.len();
    cast(&mut g, growth, Some(Target::Permanent(druid))).expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before - 1, "an ordinary spell doesn't cantrip");

    let aura = g.add_card_to_hand(0, catalog::heros_resolve());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let before = g.players[0].hand.len();
    cast(&mut g, aura, Some(Target::Permanent(druid))).expect("aura");
    drain_stack(&mut g);
    // -1 for the Aura leaving hand, +1 for the trigger's draw.
    assert_eq!(g.players[0].hand.len(), before, "the Aura draws a card");
}

/// `remove_all_counters_cost` — Essence Bottle banks elixir counters and pays
/// 2 life for each one it strips.
#[test]
fn essence_bottle_cashes_in_every_elixir_counter() {
    let mut g = two_player_game();
    let bottle = ready(&mut g, 0, catalog::essence_bottle());
    g.step = TurnStep::PreCombatMain;

    // Nothing banked — the second ability can't be activated.
    assert!(activate(&mut g, bottle, 1, None).is_err(), "no counters, no activation");

    for _ in 0..3 {
        g.battlefield_find_mut(bottle).unwrap().tapped = false;
        g.players[0].mana_pool.add_colorless(3);
        activate(&mut g, bottle, 0, None).expect("bank");
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(bottle).unwrap().counter_count(CounterType::Elixir), 3);

    g.battlefield_find_mut(bottle).unwrap().tapped = false;
    let life = g.players[0].life;
    activate(&mut g, bottle, 1, None).expect("cash in");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 6);
    assert_eq!(g.battlefield_find(bottle).unwrap().counter_count(CounterType::Elixir), 0);
}

/// Torture Chamber shares the same cost: the ping scales with the counters the
/// activation removed.
#[test]
fn torture_chamber_spends_its_whole_pain_pile() {
    let mut g = two_player_game();
    let chamber = ready(&mut g, 0, catalog::torture_chamber());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(chamber).unwrap().add_counters(CounterType::Pain, 3);
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, chamber, 0, Some(Target::Permanent(bear))).expect("fire");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "3 damage kills a 2/2");
    assert_eq!(g.battlefield_find(chamber).unwrap().counter_count(CounterType::Pain), 0);
}

// ── Buyback ─────────────────────────────────────────────────────────────────

/// Whispers of the Muse bought back draws and returns to hand.
#[test]
fn whispers_of_the_muse_returns_on_buyback() {
    let mut g = two_player_game();
    let whispers = g.add_card_to_hand(0, catalog::whispers_of_the_muse());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellBuyback {
        card_id: whispers,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("buyback cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == whispers), "bought back");
}

/// Worthy Cause's additional cost feeds it the sacrificed creature's toughness.
#[test]
fn worthy_cause_gains_the_sacrificed_toughness() {
    let mut g = two_player_game();
    let cause = g.add_card_to_hand(0, catalog::worthy_cause());
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_wood()); // 0/3
    g.players[0].mana_pool.add(Color::White, 1);
    let life = g.players[0].life;
    cast(&mut g, cause, None).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wall).is_none(), "the wall paid the cost");
    assert_eq!(g.players[0].life, life + 3);
}

// ── Statics ─────────────────────────────────────────────────────────────────

/// Chill taxes every red spell, not just an opponent's.
#[test]
fn chill_taxes_red_spells_on_both_sides() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::chill());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    assert!(cast(&mut g, bolt, Some(Target::Player(1))).is_err(), "{{R}} alone isn't enough");
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, bolt, Some(Target::Player(1))).expect("cast with the tax paid");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
}

/// Humility blanks abilities and flattens power and toughness.
#[test]
fn humility_makes_everything_a_vanilla_one_one() {
    let mut g = two_player_game();
    let flier = g.add_card_to_battlefield(0, catalog::serra_angel());
    g.add_card_to_battlefield(1, catalog::humility());
    let cp = g.computed_permanent(flier).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    assert!(!cp.keywords.contains(&Keyword::Flying));
}

/// Marble Titan locks down the big creatures at untap.
#[test]
fn marble_titan_keeps_power_three_creatures_tapped() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::marble_titan());
    let big = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let small = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    for id in [big, small] {
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    g.active_player_idx = 0;
    g.do_untap();
    assert!(g.battlefield_find(big).unwrap().tapped, "the 4/4 stays down");
    assert!(!g.battlefield_find(small).unwrap().tapped, "the 2/2 untaps");
}

/// Nature's Revolt animates every land into a 2/2 that is still a land.
#[test]
fn natures_revolt_animates_all_lands() {
    let mut g = two_player_game();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(1, catalog::natures_revolt());
    let cp = g.computed_permanent(forest).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
    assert!(cp.card_types.contains(&crabomination::card::CardType::Creature));
    assert!(cp.card_types.contains(&crabomination::card::CardType::Land));
}

// ── Triggers ────────────────────────────────────────────────────────────────

/// Death Pits of Rath turns any damage into a kill.
#[test]
fn death_pits_of_rath_kills_anything_it_scratches() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::death_pits_of_rath());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let mut events = vec![];
    g.deal_damage_to_from(EntityRef::Permanent(angel), 1, None, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(angel).is_none(), "one point is lethal here");
}

/// Field of Souls leaves a flying Spirit behind for a nontoken death only.
#[test]
fn field_of_souls_replaces_nontoken_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::field_of_souls());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut events = vec![];
    g.destroy_permanent(bear, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let spirits =
        g.battlefield.iter().filter(|c| c.definition.name == "Spirit" && c.is_token).count();
    assert_eq!(spirits, 1);
}

/// Apes of Rath pays for its 5 power by staying tapped.
#[test]
fn apes_of_rath_skips_its_next_untap() {
    let mut g = two_player_game();
    let apes = ready(&mut g, 0, catalog::apes_of_rath());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: apes, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    g.do_untap();
    assert!(g.battlefield_find(apes).unwrap().tapped, "it doesn't untap");
}

/// Havoc drains an opponent who casts white; Warmth pays its controller for the
/// same opponent's red spells.
#[test]
fn the_color_watchers_fire_off_opposing_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::havoc());
    g.add_card_to_battlefield(0, catalog::warmth());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    let (life0, life1) = (g.players[0].life, g.players[1].life);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 2 - 3, "Warmth pays 2, the Bolt takes 3");
    assert_eq!(g.players[1].life, life1, "Havoc doesn't fire on a red spell");
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Extinction names a type and sweeps only that type.
#[test]
fn extinction_wipes_the_named_creature_type() {
    let mut g = two_player_game();
    let elf = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ext = g.add_card_to_hand(0, catalog::extinction());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::CreatureType(
        crabomination::card::CreatureType::Elf,
    )]));
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, ext, None).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(elf).is_none(), "the named type is swept");
    assert!(g.battlefield_find(bear).is_some(), "everything else lives");
}

/// Kindle scales with the Kindles already in graveyards.
#[test]
fn kindle_scales_with_its_own_graveyard_copies() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(1, catalog::kindle());
    g.add_card_to_graveyard(0, catalog::kindle());
    let kindle = g.add_card_to_hand(0, catalog::kindle());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, kindle, Some(Target::Player(1))).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 16, "2 base + 2 in graveyards");
}

/// Repentance turns a creature's own power against it.
#[test]
fn repentance_makes_a_creature_hit_itself() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let rep = g.add_card_to_hand(0, catalog::repentance());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, rep, Some(Target::Permanent(angel))).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(angel).is_none(), "4 damage kills a 4/4");
}

/// Meditate buys four cards for the next turn.
#[test]
fn meditate_skips_your_next_turn() {
    let mut g = two_player_game();
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::forest());
    }
    let med = g.add_card_to_hand(0, catalog::meditate());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let before = g.players[0].hand.len();
    cast(&mut g, med, None).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before - 1 + 4);
    assert!(g.players[0].skip_turns > 0, "the next turn is skipped");
}

// ── Lands ───────────────────────────────────────────────────────────────────

/// The slow-dual cycle: the colorless tap is free, the colored one costs the
/// land its next untap.
#[test]
fn slow_duals_stay_tapped_after_a_colored_tap() {
    let mut g = two_player_game();
    let marsh = ready(&mut g, 0, catalog::cinder_marsh());
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, marsh, 1, None).expect("tap for {B}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1);
    g.active_player_idx = 0;
    g.do_untap();
    assert!(g.battlefield_find(marsh).unwrap().tapped, "it misses its untap");
}

/// The damage-tapland cycle enters tapped and pings for its colored mana.
#[test]
fn damage_taplands_enter_tapped_and_bite() {
    let mut g = two_player_game();
    let lake = g.move_card_to_battlefield_for_test(0, catalog::caldera_lake());
    drain_stack(&mut g);
    assert!(g.battlefield_find(lake).unwrap().tapped, "enters tapped");
    g.battlefield_find_mut(lake).unwrap().tapped = false;
    g.step = TurnStep::PreCombatMain;
    let life = g.players[0].life;
    activate(&mut g, lake, 1, None).expect("tap for {U}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1);
    assert_eq!(g.players[0].life, life - 1);
}

/// Ghost Town only bounces itself on someone else's turn.
#[test]
fn ghost_town_bounces_only_off_turn() {
    let mut g = two_player_game();
    let town = ready(&mut g, 0, catalog::ghost_town());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    assert!(activate(&mut g, town, 1, None).is_err(), "not on your own turn");
    g.active_player_idx = 1;
    activate(&mut g, town, 1, None).expect("bounce");
    drain_stack(&mut g);
    assert!(g.battlefield_find(town).is_none(), "back to hand");
}

// ── Second batch ────────────────────────────────────────────────────────────

/// `Keyword::CanBlockShadow` lifts only the attacker-side half of CR 702.28:
/// the Dryad catches a Soltari and still blocks ordinary creatures.
#[test]
fn can_block_shadow_catches_shadow_without_becoming_shadow() {
    let mut g = two_player_game();
    let dryad = ready(&mut g, 0, catalog::heartwood_dryad());
    let soltari = ready(&mut g, 1, catalog::soltari_foot_soldier());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    assert!(g.blocker_can_block_attacker(dryad, soltari), "it can catch shadow");
    assert!(g.blocker_can_block_attacker(dryad, bear), "and still blocks normally");
}

/// `PowerAtMostSourceCounters` — Legacy's Allure only reaches creatures its own
/// treasure pile can pay for.
#[test]
fn legacys_allure_steals_only_within_its_counters() {
    let mut g = two_player_game();
    let allure = ready(&mut g, 0, catalog::legacys_allure());
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.battlefield_find_mut(allure).unwrap().add_counters(CounterType::Currency, 2);
    g.step = TurnStep::PreCombatMain;

    assert!(activate(&mut g, allure, 0, Some(Target::Permanent(big))).is_err(), "4 power is out of reach");
    activate(&mut g, allure, 0, Some(Target::Permanent(small))).expect("steal");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(small).unwrap().controller, 0);
}

/// `RevealMissDest::Exile` — Sacred Guide keeps the white card and exiles the
/// cards it dug past.
#[test]
fn sacred_guide_exiles_everything_it_digs_past() {
    let mut g = two_player_game();
    // Library top-down: two nonwhite, then a white card.
    let plains = catalog::serra_angel();
    for def in [catalog::grizzly_bears(), catalog::llanowar_elves()] {
        g.add_card_to_library(0, def);
    }
    g.add_card_to_library(0, plains);
    let guide = ready(&mut g, 0, catalog::sacred_guide());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, guide, 0, None).expect("dig");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Serra Angel"), "the white card");
    assert_eq!(g.exile.iter().filter(|c| c.owner == 0).count(), 2, "the misses are exiled");
}

/// Kezzerdrix only bites when the other side of the board is empty.
#[test]
fn kezzerdrix_bites_an_empty_board() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kezzerdrix());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::Untap;
    advance_to(&mut g, TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "an opposing creature keeps it calm");

    let mut events = vec![];
    g.destroy_permanent(bear, false, &mut events);
    drain_stack(&mut g);
    g.step = TurnStep::Untap;
    advance_to(&mut g, TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 16);
}

/// Mnemonic Sliver's sac-for-a-card reaches every Sliver, not just itself.
#[test]
fn mnemonic_sliver_arms_the_whole_hive() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.add_card_to_battlefield(0, catalog::mnemonic_sliver());
    let metallic = ready(&mut g, 0, catalog::metallic_sliver());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(2);
    let before = g.players[0].hand.len();
    // Index 0 is the granted ability (Metallic Sliver prints none).
    activate(&mut g, metallic, 0, None).expect("sac for a card");
    drain_stack(&mut g);
    assert!(g.battlefield_find(metallic).is_none(), "it ate itself");
    assert_eq!(g.players[0].hand.len(), before + 1);
}

/// Watchdog dulls the attack only while it is untapped.
#[test]
fn watchdog_softens_attackers_while_untapped() {
    let mut g = two_player_game();
    let dog = g.add_card_to_battlefield(0, catalog::watchdog());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }])
        .expect("attack");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 1, "-1/-0 while the dog is up");
    g.battlefield_find_mut(dog).unwrap().tapped = true;
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "tapped, so no longer");
}

// ── Third batch ─────────────────────────────────────────────────────────────

/// Mogg Squad shrinks for every other creature, itself excluded.
#[test]
fn mogg_squad_shrinks_with_the_board() {
    let mut g = two_player_game();
    let squad = g.add_card_to_battlefield(0, catalog::mogg_squad());
    assert_eq!(g.computed_permanent(squad).unwrap().power, 3, "alone it's a 3/3");
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let cp = g.computed_permanent(squad).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "both sides count");
}

/// Bounty Hunter has to mark a creature before it can kill one, and the mark
/// only lands on nonblack creatures.
#[test]
fn bounty_hunter_marks_then_collects() {
    let mut g = two_player_game();
    let hunter = ready(&mut g, 0, catalog::bounty_hunter());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;

    assert!(
        activate(&mut g, hunter, 1, Some(Target::Permanent(bear))).is_err(),
        "no bounty counter yet"
    );
    activate(&mut g, hunter, 0, Some(Target::Permanent(bear))).expect("mark");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::Bounty), 1);

    g.battlefield_find_mut(hunter).unwrap().tapped = false;
    activate(&mut g, hunter, 1, Some(Target::Permanent(bear))).expect("collect");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
}

/// Deadshot taps one creature and fires its power at another.
#[test]
fn deadshot_turns_a_creature_into_a_gun() {
    let mut g = two_player_game();
    let gun = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let shot = g.add_card_to_hand(0, catalog::deadshot());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: shot,
        target: Some(Target::Permanent(gun)),
        additional_targets: vec![Target::Permanent(victim)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(gun).unwrap().tapped, "the gun is tapped");
    assert!(g.battlefield_find(victim).is_none(), "4 damage kills the 2/2");
}
