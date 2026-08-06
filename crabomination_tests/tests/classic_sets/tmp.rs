//! Tempest (TMP) — `catalog::sets::tmp`.

use crabomination::card::{CardDefinition, CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::{EffectContext, EntityRef};
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

// ── Set-closing wave ────────────────────────────────────────────────────────

/// Grindstone keeps milling while the pair shares a colour and stops on the
/// first mismatched pair.
#[test]
fn grindstone_repeats_while_colors_match() {
    let mut g = two_player_game();
    // Top-down: a blue pair repeats into a red/green pair, which stops it.
    for def in [
        catalog::counterspell(),   // blue
        catalog::counterspell(),   // blue
        catalog::lightning_bolt(), // red
        catalog::grizzly_bears(),  // green
    ] {
        g.add_card_to_library(1, def);
    }
    let stone = ready(&mut g, 0, catalog::grindstone());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(3);
    activate(&mut g, stone, 0, Some(Target::Player(1))).expect("grind");
    drain_stack(&mut g);
    // Blue+blue repeats; red+green stops it. All four are gone.
    assert_eq!(g.players[1].graveyard.len(), 4);
    assert!(g.players[1].library.is_empty());
}

/// Cursed Scroll only fires when the random reveal matches the named card.
#[test]
fn cursed_scroll_needs_the_named_card() {
    let mut g = two_player_game();
    let scroll = ready(&mut g, 0, catalog::cursed_scroll());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;

    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::NamedCard(
        "Lightning Bolt".into(),
    )]));
    g.players[0].mana_pool.add_colorless(3);
    activate(&mut g, scroll, 0, Some(Target::Player(1))).expect("miss");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20, "a miss deals nothing");

    g.battlefield_find_mut(scroll).unwrap().tapped = false;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::NamedCard(
        "Grizzly Bears".into(),
    )]));
    g.players[0].mana_pool.add_colorless(3);
    activate(&mut g, scroll, 0, Some(Target::Player(1))).expect("hit");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "a match deals 2");
}

/// Cold Storage banks a creature and gives it back when it's sacrificed.
#[test]
fn cold_storage_returns_its_whole_stash() {
    let mut g = two_player_game();
    let storage = g.add_card_to_battlefield(0, catalog::cold_storage());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(3);
    activate(&mut g, storage, 0, Some(Target::Permanent(bear))).expect("bank");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "the bear is exiled");

    activate(&mut g, storage, 1, None).expect("cash out");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 0),
        "the bear comes back under your control"
    );
}

/// Helm of Possession holds a stolen creature only while it stays tapped.
#[test]
fn helm_of_possession_steal_unwinds_on_untap() {
    let mut g = two_player_game();
    let helm = ready(&mut g, 0, catalog::helm_of_possession());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // the sacrifice
    let prize = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, helm, 0, Some(Target::Permanent(prize))).expect("possess");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(prize).unwrap().controller, 0);

    g.battlefield_find_mut(helm).unwrap().tapped = false;
    g.check_state_based_actions();
    assert_eq!(g.battlefield_find(prize).unwrap().controller, 1, "untapping hands it back");
}

/// Minion of the Wastes is as big as the life paid for it.
#[test]
fn minion_of_the_wastes_sizes_to_the_life_paid() {
    let mut g = two_player_game();
    let minion = g.add_card_to_hand(0, catalog::minion_of_the_wastes());
    g.players[0].mana_pool.add(Color::Black, 3);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(7)]));
    cast(&mut g, minion, None).expect("cast");
    drain_stack(&mut g);
    let live = g.computed_permanent(minion).expect("on the battlefield");
    assert_eq!((live.power, live.toughness), (7, 7));
    assert_eq!(g.players[0].life, 13);
}

/// Unstable Shapeshifter copies each creature that enters and keeps copying.
#[test]
fn unstable_shapeshifter_keeps_its_own_trigger() {
    let mut g = two_player_game();
    let shifter = g.add_card_to_battlefield(0, catalog::unstable_shapeshifter());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: bear }]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(shifter).unwrap().definition.name, "Grizzly Bears");

    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: angel }]);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(shifter).unwrap().definition.name,
        "Serra Angel",
        "the kept trigger fires again"
    );
}

/// Carrionette's graveyard ability exiles both bodies when the tax goes
/// unpaid, and neither when it's paid.
#[test]
fn carrionette_exiles_unless_the_tax_is_paid() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
    let carrionette = g.add_card_to_graveyard(0, catalog::carrionette());
    g.step = TurnStep::PreCombatMain;

    // Paid: nothing happens.
    g.players[1].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    activate(&mut g, carrionette, 0, Some(Target::Permanent(victim))).expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_some(), "the tax saved it");

    // Declined: both go.
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    activate(&mut g, carrionette, 0, Some(Target::Permanent(victim))).expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "the creature is exiled");
    assert!(
        !g.players[0].graveyard.iter().any(|c| c.id == carrionette),
        "Carrionette exiles itself too"
    );
}

/// Starke of Rath hands himself to whoever owned what he killed.
#[test]
fn starke_of_rath_changes_hands_after_killing() {
    let mut g = two_player_game();
    let starke = ready(&mut g, 0, catalog::starke_of_rath());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, starke, 0, Some(Target::Permanent(victim))).expect("kill");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none());
    assert_eq!(g.battlefield_find(starke).unwrap().controller, 1, "Starke defects");
}

/// Magmasaur eats a counter each upkeep, or blows up for the rest.
#[test]
fn magmasaur_detonates_when_you_stop_feeding_it() {
    let mut g = two_player_game();
    let saur = g.add_card_to_hand(0, catalog::magmasaur());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, saur, None).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(saur).unwrap().counter_count(CounterType::PlusOnePlusOne), 5);
    let def = catalog::magmasaur();
    let ctx = EffectContext::for_ability(saur, 0, None);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.resolve_effect(&def.triggered_abilities[0].effect, &ctx).expect("fed");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(saur).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);

    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    g.resolve_effect(&def.triggered_abilities[0].effect, &ctx).expect("starved");
    drain_stack(&mut g);
    assert!(g.battlefield_find(saur).is_none(), "it sacrifices itself");
    assert!(g.battlefield_find(bear).is_none(), "4 damage sweeps the ground");
    assert_eq!(g.players[1].life, 16);
}

/// Wood Sage takes every copy of the named creature and bins the rest.
#[test]
fn wood_sage_takes_the_named_creatures() {
    let mut g = two_player_game();
    for def in [
        catalog::grizzly_bears(),
        catalog::forest(),
        catalog::grizzly_bears(),
        catalog::lightning_bolt(),
    ] {
        g.add_card_to_library(0, def);
    }
    let sage = ready(&mut g, 0, catalog::wood_sage());
    g.step = TurnStep::PreCombatMain;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::NamedCard("Grizzly Bears".into())]));
    activate(&mut g, sage, 0, None).expect("sage");
    drain_stack(&mut g);
    assert_eq!(
        g.players[0].hand.iter().filter(|c| c.definition.name == "Grizzly Bears").count(),
        2
    );
    assert_eq!(g.players[0].graveyard.len(), 2, "the rest is binned");
}

/// Abandon Hope discards X as a cost, then strips X cards out of the
/// opponent's hand.
#[test]
fn abandon_hope_trades_x_for_x() {
    let mut g = two_player_game();
    let hope = g.add_card_to_hand(0, catalog::abandon_hope());
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::forest());
    }
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: hope,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.is_empty(), "both lands were discarded as the cost");
    assert_eq!(g.players[1].hand.len(), 1, "two of the three are gone");
}

/// Interdict counters an activated ability and locks the source for the turn.
#[test]
fn interdict_locks_the_permanent_it_answers() {
    let mut g = two_player_game();
    let scroll = ready(&mut g, 1, catalog::cursed_scroll());
    let interdict = g.add_card_to_hand(0, catalog::interdict());
    g.add_card_to_library(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.players[1].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: scroll,
        ability_index: 0,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");

    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: interdict,
        target: Some(Target::Permanent(scroll)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "the ability was countered");

    g.battlefield_find_mut(scroll).unwrap().tapped = false;
    g.players[1].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: scroll,
            ability_index: 0,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the lock holds for the rest of the turn"
    );
}

/// Phyrexian Grimoire lets the opponent pick which of your top two graveyard
/// cards is exiled; the other comes back.
#[test]
fn phyrexian_grimoire_splits_the_top_two() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    let grimoire = ready(&mut g, 0, catalog::phyrexian_grimoire());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![angel])]));
    activate(&mut g, grimoire, 0, Some(Target::Player(1))).expect("split");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "the unpicked card goes to hand");
    assert!(
        g.exile.iter().any(|c| c.id == angel),
        "the opponent's pick is exiled"
    );
}

/// Maze of Shadows unties an attacking shadow creature and fogs it both ways.
#[test]
fn maze_of_shadows_only_answers_shadow() {
    let mut g = two_player_game();
    let maze = ready(&mut g, 1, catalog::maze_of_shadows());
    let shadow = ready(&mut g, 0, catalog::dauthi_marauder());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: shadow, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    assert!(
        activate(&mut g, maze, 1, Some(Target::Permanent(bear))).is_err(),
        "the nonshadow attacker isn't a legal target"
    );
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: maze,
        ability_index: 1,
        target: Some(Target::Permanent(shadow)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("maze");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(shadow).unwrap().tapped, "it untapped");
    advance_to(&mut g, TurnStep::End);
    assert_eq!(g.players[1].life, 18, "only the bear connected");
}

/// Reap returns as many graveyard cards as the target opponent has black
/// permanents.
#[test]
fn reap_scales_off_the_opponents_black_board() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let spare = g.add_card_to_graveyard(0, catalog::counterspell());
    g.add_card_to_battlefield(1, catalog::dauthi_marauder()); // black
    g.add_card_to_battlefield(1, catalog::dauthi_slayer()); // black
    g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green — doesn't count
    let reap = g.add_card_to_hand(0, catalog::reap());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: reap,
        target: Some(Target::Player(1)),
        additional_targets: vec![
            Target::Permanent(bear),
            Target::Permanent(bolt),
            Target::Permanent(spare),
        ],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 2, "two black permanents, two cards back");
}

/// Static Orb caps every player's untap step at two permanents.
#[test]
fn static_orb_caps_untaps_at_two() {
    let mut g = two_player_game();
    let orb = g.add_card_to_battlefield(0, catalog::static_orb());
    let lands: Vec<CardId> =
        (0..4).map(|_| g.add_card_to_battlefield(0, catalog::forest())).collect();
    for id in &lands {
        g.battlefield_find_mut(*id).unwrap().tapped = true;
    }
    g.do_untap();
    assert_eq!(lands.iter().filter(|id| !g.battlefield_find(**id).unwrap().tapped).count(), 2);

    // Tapping the Orb lifts the cap.
    for id in &lands {
        g.battlefield_find_mut(*id).unwrap().tapped = true;
    }
    g.battlefield_find_mut(orb).unwrap().tapped = true;
    g.do_untap();
    assert_eq!(lands.iter().filter(|id| !g.battlefield_find(**id).unwrap().tapped).count(), 4);
}

/// Hand to Hand shuts instants and non-mana abilities off during combat only.
#[test]
fn hand_to_hand_locks_combat_tricks() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hand_to_hand());
    let scroll = ready(&mut g, 0, catalog::cursed_scroll());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());

    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "no instants during combat"
    );
    assert!(activate(&mut g, scroll, 0, Some(Target::Player(1))).is_err(), "no abilities either");

    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    activate(&mut g, scroll, 0, Some(Target::Player(1))).expect("fine outside combat");
}

/// Pallimud's power tracks the chosen opponent's tapped lands.
#[test]
fn pallimud_counts_tapped_lands() {
    let mut g = two_player_game();
    let lands: Vec<CardId> =
        (0..3).map(|_| g.add_card_to_battlefield(1, catalog::forest())).collect();
    let mud = g.add_card_to_hand(0, catalog::pallimud());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, mud, None).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(mud).unwrap().power, 0);
    for id in &lands {
        g.battlefield_find_mut(*id).unwrap().tapped = true;
    }
    let live = g.computed_permanent(mud).unwrap();
    assert_eq!((live.power, live.toughness), (3, 3));
}

/// Dracoplasm enters as the sum of what it ate.
#[test]
fn dracoplasm_sums_its_sacrifices() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let plasm = g.add_card_to_hand(0, catalog::dracoplasm());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear, angel])]));
    cast(&mut g, plasm, None).expect("cast");
    drain_stack(&mut g);
    let live = g.computed_permanent(plasm).unwrap();
    assert_eq!((live.power, live.toughness), (6, 6));
}

/// Escaped Shapeshifter picks up keywords from across the table.
#[test]
fn escaped_shapeshifter_mirrors_opposing_keywords() {
    let mut g = two_player_game();
    let shifter = g.add_card_to_battlefield(0, catalog::escaped_shapeshifter());
    let has = |g: &GameState, kw| g.computed_permanent(shifter).unwrap().keywords.contains(&kw);
    assert!(!has(&g, Keyword::Flying));
    g.add_card_to_battlefield(0, catalog::serra_angel()); // yours doesn't count
    assert!(!has(&g, Keyword::Flying));
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    assert!(has(&g, Keyword::Flying), "an opponent's flier lends its wings");
    g.destroy_permanent(angel, false, &mut vec![]);
    assert!(!has(&g, Keyword::Flying));
}

/// Flowstone Sculpture's mode 2 buys first strike permanently.
#[test]
fn flowstone_sculpture_buys_keywords() {
    let mut g = two_player_game();
    let sculpt = g.add_card_to_battlefield(0, catalog::flowstone_sculpture());
    g.add_card_to_hand(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sculpt,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: Some(2),
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(sculpt).unwrap().keywords.contains(&Keyword::FirstStrike),
        "the grant sticks"
    );
    assert!(g.players[0].hand.is_empty(), "the discard was paid");
}

/// Excavator hands over the sacrificed land's landwalk.
#[test]
fn excavator_grants_the_sacrificed_lands_walk() {
    let mut g = two_player_game();
    let digger = ready(&mut g, 0, catalog::excavator());
    g.add_card_to_battlefield(0, catalog::forest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, digger, 0, Some(Target::Permanent(bear))).expect("dig");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(bear)
            .unwrap()
            .keywords
            .contains(&Keyword::Landwalk(crabomination::card::LandType::Forest))
    );
}

/// Soltari Guerrillas points its unblocked damage at a creature instead.
#[test]
fn soltari_guerrillas_redirects_to_a_creature() {
    let mut g = two_player_game();
    let guerrillas = ready(&mut g, 0, catalog::soltari_guerrillas());
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, guerrillas, 0, Some(Target::Permanent(victim))).expect("arm");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: guerrillas,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    advance_to(&mut g, TurnStep::End);
    assert_eq!(g.players[1].life, 20, "the player took nothing");
    assert_eq!(g.battlefield_find(victim).unwrap().damage, 3);
}

/// No Quarter kills whichever half of a block is outclassed.
#[test]
fn no_quarter_kills_the_weaker_half() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::no_quarter());
    let big = ready(&mut g, 0, catalog::hulking_cyclops()); // 5/5, no evasion
    let chump = ready(&mut g, 1, catalog::grizzly_bears()); // 2/2
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: big,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(chump, big)])).expect("block");
    drain_stack(&mut g);
    assert!(g.battlefield_find(chump).is_none(), "the weaker blocker is destroyed");
    assert!(g.battlefield_find(big).is_some(), "the attacker survives");
}

/// Nurturing Licid keeps its regenerate ability once it becomes an Aura.
#[test]
fn nurturing_licid_regenerates_as_an_aura() {
    let mut g = two_player_game();
    let licid = ready(&mut g, 0, catalog::nurturing_licid());
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Green, 1);
    activate(&mut g, licid, 0, Some(Target::Permanent(host))).expect("attach");
    drain_stack(&mut g);
    let live = g.battlefield_find(licid).unwrap();
    assert_eq!(live.attached_to, Some(host));
    assert_eq!(live.definition.activated_abilities.len(), 2, "detach + regenerate");

    g.players[0].mana_pool.add(Color::Green, 1);
    activate(&mut g, licid, 1, None).expect("regenerate");
    drain_stack(&mut g);
    g.destroy_permanent(host, false, &mut vec![]);
    assert!(g.battlefield_find(host).is_some(), "the shield soaked the kill");
}

/// Leeching Licid drains the host's controller each of their upkeeps.
#[test]
fn leeching_licid_pings_the_hosts_controller() {
    let mut g = two_player_game();
    let licid = ready(&mut g, 0, catalog::leeching_licid());
    let host = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Black, 1);
    activate(&mut g, licid, 0, Some(Target::Permanent(host))).expect("attach");
    drain_stack(&mut g);

    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20, "not on your upkeep");
    g.active_player_idx = 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19);
}

/// Stinging Licid shocks the host's controller when the host taps.
#[test]
fn stinging_licid_punishes_tapping() {
    let mut g = two_player_game();
    let licid = ready(&mut g, 0, catalog::stinging_licid());
    let host = ready(&mut g, 1, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, licid, 0, Some(Target::Permanent(host))).expect("attach");
    drain_stack(&mut g);

    g.battlefield_find_mut(host).unwrap().tapped = true;
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped {
        card_id: host,
        actor: None,
        as_attacker: false,
    }]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
}

/// Thalakos Dreamsower pins a creature for as long as it stays tapped.
#[test]
fn thalakos_dreamsower_pins_a_creature() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let sower = ready(&mut g, 0, catalog::thalakos_dreamsower());
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sower,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "shadow got through");
    assert!(g.battlefield_find(victim).unwrap().tapped, "the trigger tapped it");

    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(victim).unwrap().tapped, "and holds it down");
}

/// Volrath's Curse shuts a creature off until its controller pays a permanent.
#[test]
fn volraths_curse_can_be_shrugged_off() {
    let mut g = two_player_game();
    let victim = ready(&mut g, 1, catalog::grizzly_bears());
    let curse = g.add_card_to_hand(0, catalog::volraths_curse());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, curse, Some(Target::Permanent(victim))).expect("enchant");
    drain_stack(&mut g);
    assert!(g.computed_permanent(victim).unwrap().keywords.contains(&Keyword::CantAttack));

    g.add_card_to_battlefield(1, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: curse,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("shrug it off");
    drain_stack(&mut g);
    assert!(
        !g.computed_permanent(victim).unwrap().keywords.contains(&Keyword::CantAttack),
        "the pass holds for the turn"
    );
}

/// Coffin Queen's loan is called in the moment she untaps.
#[test]
fn coffin_queen_exiles_on_untap() {
    let mut g = two_player_game();
    let queen = ready(&mut g, 0, catalog::coffin_queen());
    let corpse = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, queen, 0, Some(Target::Permanent(corpse))).expect("reanimate");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(corpse).unwrap().controller, 0);

    g.do_untap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(corpse).is_none(), "untapping exiles the loan");
}

/// Maddening Imp forces the active player's board into combat and kills the
/// holdouts.
#[test]
fn maddening_imp_drags_everyone_into_combat() {
    let mut g = two_player_game();
    let imp = ready(&mut g, 1, catalog::maddening_imp());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: imp,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("madden");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::MustAttack));
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![])).expect_err("must attack");
}

/// Phyrexian Splicer moves an evasion keyword from one creature to another.
#[test]
fn phyrexian_splicer_moves_a_keyword() {
    let mut g = two_player_game();
    let splicer = ready(&mut g, 0, catalog::phyrexian_splicer());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // flying
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: splicer,
        ability_index: 0,
        target: Some(Target::Permanent(angel)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: None,
    })
    .expect("splice");
    drain_stack(&mut g);
    assert!(!g.computed_permanent(angel).unwrap().keywords.contains(&Keyword::Flying));
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying));
}

/// Scroll Rack swaps hand cards for the top of the library and stacks the
/// exiled ones back on top.
#[test]
fn scroll_rack_swaps_hand_for_top() {
    let mut g = two_player_game();
    let rack = ready(&mut g, 0, catalog::scroll_rack());
    let junk = g.add_card_to_hand(0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![junk])]));
    activate(&mut g, rack, 0, None).expect("rack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[0].hand[0].definition.name, "Forest", "drew off the top");
    assert_eq!(g.players[0].library[0].id, junk, "the exiled card is back on top");
}

/// Echo Chamber rents an opponent's creature for the turn.
#[test]
fn echo_chamber_rents_a_creature() {
    let mut g = two_player_game();
    let chamber = ready(&mut g, 0, catalog::echo_chamber());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![angel])]));
    activate(&mut g, chamber, 0, None).expect("echo");
    drain_stack(&mut g);
    let token = g
        .battlefield
        .iter()
        .find(|c| c.is_token && c.controller == 0)
        .expect("a token copy");
    assert_eq!(token.definition.name, "Serra Angel");
    assert!(g.computed_permanent(token.id).unwrap().keywords.contains(&Keyword::Haste));
}
