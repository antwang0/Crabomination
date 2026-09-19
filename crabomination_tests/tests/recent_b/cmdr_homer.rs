//! Commander batch — Homer, the Hermit and friends (`decks::cmdr_homer`).

use crabomination::card::{CardType, CounterType, CreatureType, Keyword, Supertype};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(players: usize) -> GameState {
    let mut g = multi_player_game(players);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    stock_libraries(&mut g, 30);
    g
}

/// Enter `step` from the step before it (firing its "at the beginning of"
/// triggers) and resolve them.
fn enter_step(g: &mut GameState, from: TurnStep) {
    g.step = from;
    let _ = g.advance_step(Vec::new());
    drain_stack(g);
}

fn play_land(g: &mut GameState, p: usize, def: CardDefinition) -> CardId {
    let id = g.add_card_to_hand(p, def);
    g.perform_action(GameAction::PlayLand(id)).expect("play land");
    drain_stack(g);
    id
}

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn cast_x(g: &mut GameState, id: CardId, x: u32) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(x),
    })
    .expect("cast with X");
    drain_stack(g);
}

fn on_bf(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == name).count()
}

/// Destroy `id` as if by an opponent's removal spell.
fn destroy(g: &mut GameState, id: CardId) {
    let ctx = EffectContext::for_ability(id, 1, Some(Target::Permanent(id)));
    let events = g
        .resolve_effect(&Effect::Destroy { what: crabomination::effect::Selector::Target(0) }, &ctx)
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(g);
}

/// Mill `n` off `seat`'s library, dispatching the mill's triggers.
fn mill_seat(g: &mut GameState, seat: usize, n: i32) {
    use crabomination::effect::{PlayerRef, Selector, Value};
    let ctx = EffectContext::for_ability(CardId(0), 0, None);
    let events = g
        .resolve_effect(
            &Effect::Mill { who: Selector::Player(PlayerRef::Seat(seat)), amount: Value::Const(n) },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(g);
}

fn gy(g: &GameState, p: usize) -> usize {
    g.players[p].graveyard.len()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let cp = g.computed_permanent(id).expect("on battlefield");
    (cp.power, cp.toughness)
}

// ── Table-driven shapes ──────────────────────────────────────────────────────

/// Printed body and a signature creature type for every creature in the batch.
#[test]
fn cmdr_homer_creatures_have_their_printed_bodies() {
    let table: Vec<(CardDefinition, i32, i32, CreatureType)> = vec![
        (catalog::homer_the_hermit(), 0, 9, CreatureType::Crab),
        (catalog::rikala_homarid_king(), 0, 4, CreatureType::Lobster),
        (catalog::gandalf_shadows_foe(), 3, 4, CreatureType::Avatar),
        (catalog::mirelurk_queen(), 4, 4, CreatureType::Mutant),
        (catalog::charix_the_raging_isle(), 0, 17, CreatureType::Leviathan),
        (catalog::purple_pentapus(), 1, 1, CreatureType::Starfish),
        (catalog::aesi_tyrant_of_gyre_strait(), 5, 5, CreatureType::Serpent),
        (catalog::chomping_changeling(), 1, 2, CreatureType::Shapeshifter),
        (catalog::purple_crystal_crab(), 1, 1, CreatureType::Crab),
        (catalog::shorecomber_crab(), 0, 4, CreatureType::Crab),
        (catalog::spiny_starfish(), 0, 1, CreatureType::Starfish),
        (catalog::ancient_greenwarden(), 5, 7, CreatureType::Elemental),
        (catalog::iceberg_cancrix(), 0, 4, CreatureType::Crab),
        (catalog::mirrorshell_crab(), 5, 7, CreatureType::Crab),
        (catalog::shore_keeper(), 0, 3, CreatureType::Trilobite),
        (catalog::mole_man_moloid_master(), 1, 1, CreatureType::Villain),
        (catalog::scuttling_sentinel(), 3, 2, CreatureType::Elf),
        (catalog::sakashima_of_a_thousand_faces(), 3, 1, CreatureType::Rogue),
        (catalog::chameleon_master_of_disguise(), 2, 3, CreatureType::Shapeshifter),
        (catalog::masked_vandal(), 1, 3, CreatureType::Shapeshifter),
        (catalog::loki_lord_of_misrule(), 3, 4, CreatureType::God),
        (catalog::changeling_wayfinder(), 1, 2, CreatureType::Shapeshifter),
        (catalog::crustacean_commando(), 0, 3, CreatureType::Soldier),
        (catalog::realmwalker(), 2, 3, CreatureType::Shapeshifter),
    ];
    for (def, p, t, ty) in table {
        assert_eq!((def.power, def.toughness), (p, t), "{}", def.name);
        assert!(def.subtypes.creature_types.contains(&ty), "{} is a {ty:?}", def.name);
    }
    assert!(catalog::iceberg_cancrix().supertypes.contains(&Supertype::Snow));
}

/// The three pod duals enter tapped in a duel and untapped with two or more
/// opponents.
#[test]
fn cmdr_homer_pod_duals_enter_untapped_only_with_two_opponents() {
    for make in [catalog::rejuvenating_springs, catalog::undergrowth_stadium, catalog::morphic_pool] {
        let mut duel = main_phase(2);
        let a = play_land(&mut duel, 0, make());
        assert!(duel.battlefield_find(a).unwrap().tapped, "{} tapped in a duel", make().name);
        let mut pod = main_phase(3);
        let b = play_land(&mut pod, 0, make());
        assert!(!pod.battlefield_find(b).unwrap().tapped, "{} untapped in a pod", make().name);
        assert_eq!(make().activated_abilities.len(), 2);
    }
}

/// The three copy sorceries make a non-legendary token of a legendary creature;
/// Irenicus's also grants flying.
#[test]
fn cmdr_homer_copy_sorceries_make_nonlegendary_tokens() {
    for (make, flying) in [
        (catalog::multiversal_recruitment as fn() -> CardDefinition, false),
        (catalog::irenicuss_vile_duplication, true),
        (catalog::quantum_misalignment, false),
    ] {
        let mut g = main_phase(2);
        let homer = g.add_card_to_battlefield(0, catalog::homer_the_hermit());
        let spell = g.add_card_to_hand(0, make());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(4);
        cast_at(&mut g, spell, Target::Permanent(homer));
        let token = g
            .battlefield
            .iter()
            .find(|c| c.is_token && c.definition.name == "Homer, the Hermit")
            .map(|c| c.id)
            .expect("token copy");
        let cp = g.computed_permanent(token).unwrap();
        assert!(!cp.supertypes().contains(&Supertype::Legendary), "{}", make().name);
        assert_eq!(cp.keywords().contains(&Keyword::Flying), flying, "{}", make().name);
        assert_eq!(on_bf(&g, "Homer, the Hermit"), 2, "legend rule spares both");
    }
}

// ── Creatures ────────────────────────────────────────────────────────────────

/// Homer's landfall mills its targets twice the sea-life count (Homer + two
/// Crabs → six each).
#[test]
fn homer_landfall_mills_targets_twice_the_sea_life_count() {
    let mut g = main_phase(4);
    g.add_card_to_battlefield(0, catalog::homer_the_hermit());
    g.add_card_to_battlefield(0, catalog::shorecomber_crab());
    g.add_card_to_battlefield(0, catalog::purple_crystal_crab());
    play_land(&mut g, 0, catalog::forest());
    let milled: Vec<usize> = (0..4).map(|p| gy(&g, p)).collect();
    assert!(milled.iter().any(|&n| n == 6), "a target milled 6: {milled:?}");
    assert!(milled.iter().all(|&n| n == 0 || n == 6), "only whole mills: {milled:?}");
}

/// Rikala's tide ticks up each upkeep, pumps the team, feeds an extra {U} at
/// three, and resets at four.
#[test]
fn rikala_tide_counters_pump_add_blue_and_reset() {
    let mut g = main_phase(2);
    let rikala = g.add_card_to_battlefield(0, catalog::rikala_homarid_king());
    let crab = g.add_card_to_battlefield(0, catalog::shorecomber_crab());
    let island = g.add_card_to_battlefield(0, catalog::island());
    for _ in 0..3 {
        enter_step(&mut g, TurnStep::Untap);
    }
    assert_eq!(g.battlefield_find(rikala).unwrap().counter_count(CounterType::Tide), 3);
    assert_eq!(pt(&g, crab), (3, 4));
    g.step = TurnStep::PreCombatMain;
    g.battlefield_find_mut(island).unwrap().tapped = false;
    activate(&mut g, island, 0, None);
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 2, "Island + Rikala's extra");
    enter_step(&mut g, TurnStep::Untap);
    assert_eq!(g.battlefield_find(rikala).unwrap().counter_count(CounterType::Tide), 0);
    assert_eq!(pt(&g, crab), (0, 4));
}

/// Gandalf blinks your lands tapped on entry, and landfall draws and grows him.
#[test]
fn gandalf_blinks_lands_and_their_landfall_feeds_him() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::island());
    let gandalf = g.add_card_to_hand(0, catalog::gandalf_shadows_foe());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(5);
    let hand = g.players[0].hand.len();
    cast(&mut g, gandalf);
    let islands: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Island").collect();
    assert_eq!(islands.len(), 2);
    assert!(islands.iter().all(|c| c.tapped), "returned tapped");
    // CR 603.2c — the two lands re-enter in one batch and each is its own
    // landfall: two draws, two counters.
    let counters = g.battlefield_find(gandalf).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, 2);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2);
    // A later land drop is its own landfall.
    play_land(&mut g, 0, catalog::island());
    assert_eq!(
        g.battlefield_find(gandalf).unwrap().counter_count(CounterType::PlusOnePlusOne),
        counters + 1
    );
}

/// Uchuulon counts Crabs/Oozes/Horrors for power and, at your end step,
/// exiles an opponent's creature card to copy itself.
#[test]
fn uchuulon_eats_a_graveyard_creature_to_copy_itself() {
    let mut g = main_phase(2);
    let uch = g.add_card_to_battlefield(0, catalog::uchuulon());
    g.add_card_to_battlefield(0, catalog::shorecomber_crab());
    assert_eq!(pt(&g, uch), (2, 4));
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    enter_step(&mut g, TurnStep::PostCombatMain);
    assert_eq!(on_bf(&g, "Uchuulon"), 2);
    assert_eq!(gy(&g, 1), 0);
    assert_eq!(pt(&g, uch), (3, 4), "the token is a third Crab/Ooze/Horror");
}

/// Mirelurk Queen hands out rad counters and draws once a turn off nonland
/// mills.
#[test]
fn mirelurk_queen_draws_once_per_turn_off_nonland_mills() {
    let mut g = main_phase(2);
    let queen = g.add_card_to_hand(0, catalog::mirelurk_queen());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, queen);
    let rads: u32 = g.players.iter().map(|p| p.rad_counters).sum();
    assert_eq!(rads, 2);
    let hand = g.players[0].hand.len();
    // Library tops are Forests; seat a nonland on top of seat 1's library.
    g.players[1].library.clear();
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    for _ in 0..2 {
        mill_seat(&mut g, 1, 1);
    }
    assert_eq!(g.players[0].hand.len(), hand + 1, "once each turn");
    assert_eq!(g.battlefield_find(queen).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Charix trades toughness for power per Island.
#[test]
fn charix_pumps_by_islands() {
    let mut g = main_phase(2);
    let charix = g.add_card_to_battlefield(0, catalog::charix_the_raging_isle());
    g.add_card_to_battlefield(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::island());
    g.players[0].mana_pool.add_colorless(3);
    activate(&mut g, charix, 0, None);
    assert_eq!(pt(&g, charix), (2, 15));
}

/// Purple Pentapus climbs back tapped by tapping another creature.
#[test]
fn purple_pentapus_returns_from_the_graveyard_tapped() {
    let mut g = main_phase(2);
    let crab = g.add_card_to_battlefield(0, catalog::shorecomber_crab());
    g.clear_sickness(crab);
    let pent = g.add_card_to_graveyard(0, catalog::purple_pentapus());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, pent, 0, None);
    let back = g.battlefield_find(pent).expect("returned");
    assert!(back.tapped);
    assert!(g.battlefield_find(crab).unwrap().tapped, "tapped as the cost");
}

/// Aesi grants a second land drop and draws on landfall.
#[test]
fn aesi_extra_land_and_landfall_draw() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::aesi_tyrant_of_gyre_strait());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    play_land(&mut g, 0, catalog::forest());
    play_land(&mut g, 0, catalog::island());
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

/// Chomping Changeling eats an artifact on entry.
#[test]
fn chomping_changeling_destroys_an_artifact() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(1, catalog::altar_of_the_brood());
    let chomp = g.add_card_to_hand(0, catalog::chomping_changeling());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, chomp);
    assert_eq!(on_bf(&g, "Altar of the Brood"), 0);
}

/// Roaming Throne becomes the chosen type and doubles that type's triggers.
#[test]
fn roaming_throne_doubles_the_chosen_types_triggers() {
    let mut g = main_phase(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Crab)]));
    let throne = g.add_card_to_hand(0, catalog::roaming_throne());
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, throne);
    let cp = g.computed_permanent(throne).unwrap();
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Crab));
    // A Crab's own ETB (Crustacean Commando's Mutagen) triggers twice.
    let cc = g.add_card_to_hand(0, catalog::crustacean_commando());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, cc);
    assert_eq!(on_bf(&g, "Mutagen"), 2, "the Crab's trigger fired twice");
}

/// Spiny Starfish leaves a Starfish behind for its regeneration.
#[test]
fn spiny_starfish_spawns_after_regenerating() {
    let mut g = main_phase(2);
    let star = g.add_card_to_battlefield(0, catalog::spiny_starfish());
    g.players[0].mana_pool.add(Color::Blue, 1);
    activate(&mut g, star, 0, None);
    destroy(&mut g, star);
    assert!(g.battlefield_find(star).is_some(), "regenerated");
    enter_step(&mut g, TurnStep::PostCombatMain);
    assert_eq!(on_bf(&g, "Starfish"), 1);
}

/// Ancient Greenwarden doubles landfall and plays lands from the graveyard.
#[test]
fn ancient_greenwarden_doubles_landfall_from_the_graveyard() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::ancient_greenwarden());
    g.add_card_to_battlefield(0, catalog::druid_class());
    g.add_card_to_battlefield(0, catalog::altar_of_the_brood());
    let forest = g.add_card_to_graveyard(0, catalog::forest());
    g.perform_action(GameAction::PlayLandFromGraveyard(forest)).expect("land from graveyard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(forest).is_some());
    assert_eq!(g.players[0].life, 22, "Druid Class landfall fired twice");
    assert_eq!(gy(&g, 1), 2, "Altar's permanent-entered trigger off a land fired twice");
}

/// Iceberg Cancrix mills two when another snow permanent enters.
#[test]
fn iceberg_cancrix_mills_on_snow_entries() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::iceberg_cancrix());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    play_land(&mut g, 0, catalog::island());
    assert_eq!(gy(&g, 0) + gy(&g, 1), 0, "a plain Island is not snow");
    play_land_extra(&mut g, catalog::snow_covered_island());
    assert_eq!(gy(&g, 0) + gy(&g, 1), 2);
}

fn play_land_extra(g: &mut GameState, def: CardDefinition) {
    g.players[0].extra_land_plays += 1;
    play_land(g, 0, def);
}

/// Mirrorshell Crab's channel counters a spell whose caster can't pay {3}.
#[test]
fn mirrorshell_crab_channel_taxes_a_spell() {
    let mut g = main_phase(2);
    let crab = g.add_card_to_hand(0, catalog::mirrorshell_crab());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: crab,
        ability_index: 0,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("channel");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "Bolt was countered");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == crab), "discarded");
}

/// Shore Keeper cashes in for three cards.
#[test]
fn shore_keeper_draws_three() {
    let mut g = main_phase(2);
    let keeper = g.add_card_to_battlefield(0, catalog::shore_keeper());
    g.clear_sickness(keeper);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(7);
    let hand = g.players[0].hand.len();
    activate(&mut g, keeper, 0, None);
    assert_eq!(g.players[0].hand.len(), hand + 3);
    assert!(g.battlefield_find(keeper).is_none());
}

/// Mole Man makes a Moloid per land and lets lands come back from the yard.
#[test]
fn mole_man_makes_moloids_and_replays_lands() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::mole_man_moloid_master());
    let forest = g.add_card_to_graveyard(0, catalog::forest());
    g.perform_action(GameAction::PlayLandFromGraveyard(forest)).expect("land from graveyard");
    drain_stack(&mut g);
    assert_eq!(on_bf(&g, "Moloid"), 1);
}

/// Scuttling Sentinel makes another creature a hexproof blue Crab with a
/// counter.
#[test]
fn scuttling_sentinel_crabs_up_an_ally() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sent = g.add_card_to_hand(0, catalog::scuttling_sentinel());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, sent);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Crab));
    assert!(cp.keywords().contains(&Keyword::Hexproof));
    assert_eq!(cp.colors.to_vec(), vec![Color::Blue]);
}

/// Sakashima copies your legendary commander and keeps the legend rule off.
#[test]
fn sakashima_copies_a_legend_and_both_survive() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::homer_the_hermit());
    let sak = g.add_card_to_hand(0, catalog::sakashima_of_a_thousand_faces());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, sak);
    assert_eq!(on_bf(&g, "Homer, the Hermit"), 2);
}

/// Chameleon copies a creature but keeps his own name.
#[test]
fn chameleon_copies_but_keeps_his_name() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::mirrorshell_crab());
    let cham = g.add_card_to_hand(0, catalog::chameleon_master_of_disguise());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, cham);
    let c = g.battlefield_find(cham).unwrap();
    assert_eq!(c.definition.name, "Chameleon, Master of Disguise");
    assert_eq!(pt(&g, cham), (5, 7));
}

/// Cryptic Trilobite enters with X counters and turns one into {C}{C}.
#[test]
fn cryptic_trilobite_banks_counters_for_ability_mana() {
    let mut g = main_phase(2);
    let tri = g.add_card_to_hand(0, catalog::cryptic_trilobite());
    g.players[0].mana_pool.add_colorless(4);
    cast_x(&mut g, tri, 2);
    assert_eq!(g.battlefield_find(tri).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    activate(&mut g, tri, 0, None);
    assert_eq!(g.battlefield_find(tri).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.players[0].mana_pool.restricted_total(), 2, "two colorless for abilities only");
}

/// Masked Vandal trades a graveyard creature for an opponent's artifact.
#[test]
fn masked_vandal_exiles_an_opposing_artifact() {
    let mut g = main_phase(2);
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::altar_of_the_brood());
    let vandal = g.add_card_to_hand(0, catalog::masked_vandal());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, vandal);
    assert_eq!(on_bf(&g, "Altar of the Brood"), 0);
    assert!(g.exile.iter().any(|c| c.definition.name == "Altar of the Brood"));
}

/// Loki turns your other creatures into non-legendary copies of the target.
#[test]
fn loki_makes_copies_of_the_chosen_creature() {
    let mut g = main_phase(2);
    let loki = g.add_card_to_battlefield(0, catalog::loki_lord_of_misrule());
    g.clear_sickness(loki);
    let charix = g.add_card_to_battlefield(0, catalog::charix_the_raging_isle());
    let crab = g.add_card_to_battlefield(0, catalog::shorecomber_crab());
    g.players[0].mana_pool.add(Color::Blue, 1);
    activate(&mut g, loki, 0, Some(Target::Permanent(charix)));
    let cp = g.computed_permanent(crab).unwrap();
    assert_eq!(cp.toughness, 17);
    assert!(!cp.supertypes().contains(&Supertype::Legendary));
    assert!(g.battlefield_find(charix).is_some(), "the original stays");
}

/// Changeling Wayfinder tutors a basic land to hand.
#[test]
fn changeling_wayfinder_fetches_a_basic() {
    let mut g = main_phase(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let way = g.add_card_to_hand(0, catalog::changeling_wayfinder());
    g.players[0].mana_pool.add_colorless(3);
    let hand = g.players[0].hand.len();
    cast(&mut g, way);
    assert_eq!(g.players[0].hand.len(), hand, "cast one, fetched one");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"));
}

/// Crustacean Commando's Mutagen grows a creature.
#[test]
fn crustacean_commando_mutagen_adds_a_counter() {
    let mut g = main_phase(2);
    let cc = g.add_card_to_hand(0, catalog::crustacean_commando());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, cc);
    let mutagen = g.battlefield.iter().find(|c| c.definition.name == "Mutagen").unwrap().id;
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, mutagen, 0, Some(Target::Permanent(cc)));
    assert_eq!(g.battlefield_find(cc).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Realmwalker casts creatures of the chosen type off the top.
#[test]
fn realmwalker_casts_the_chosen_type_from_the_top() {
    let mut g = main_phase(2);
    g.players[0].library.clear();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Crab)]));
    let rw = g.add_card_to_hand(0, catalog::realmwalker());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, rw);
    let crab = g.add_card_to_library(0, catalog::shorecomber_crab());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, crab);
    assert!(g.battlefield_find(crab).is_some(), "cast from the library top");
    let bears = g.players[0].library[0].id;
    g.players[0].mana_pool.add(Color::Green, 2);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .is_err(),
        "a Bear is not the chosen type"
    );
}

// ── Instants and sorceries ───────────────────────────────────────────────────

/// Through the Forest Gate drops every land in the top twenty, tapped.
#[test]
fn through_the_forest_gate_puts_lands_in_play() {
    let mut g = main_phase(2);
    g.players[0].library.clear();
    for i in 0..24 {
        if i % 2 == 0 {
            g.add_card_to_library(0, catalog::forest());
        } else {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
    }
    let gate = g.add_card_to_hand(0, catalog::through_the_forest_gate());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(6);
    cast(&mut g, gate);
    let forests: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Forest").collect();
    assert_eq!(forests.len(), 10);
    assert!(forests.iter().all(|c| c.tapped));
    assert_eq!(g.players[0].life, 28);
    assert_eq!(g.players[0].library.len(), 14);
}

/// Cruel Calculations draws per card that hit the target's graveyard.
#[test]
fn cruel_calculations_draws_per_milled_card() {
    let mut g = main_phase(2);
    mill_seat(&mut g, 1, 3);
    let cc = g.add_card_to_hand(0, catalog::cruel_calculations());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len();
    cast_at(&mut g, cc, Target::Player(1));
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3);
}

/// Entish Restoration fetches three with a big creature, two without.
#[test]
fn entish_restoration_scales_with_a_four_power_creature() {
    for (big, fetched) in [(false, 2), (true, 3)] {
        let mut g = main_phase(2);
        g.add_card_to_battlefield(0, catalog::island());
        if big {
            g.add_card_to_battlefield(0, catalog::mirelurk_queen());
        }
        let er = g.add_card_to_hand(0, catalog::entish_restoration());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, er);
        assert_eq!(on_bf(&g, "Island"), 0, "the land was sacrificed");
        assert_eq!(on_bf(&g, "Forest"), fetched);
    }
}

/// Kitsune's Technique mills half the target's library, rounded up.
#[test]
fn kitsunes_technique_mills_half_rounded_up() {
    let mut g = main_phase(2);
    g.add_card_to_library(1, catalog::forest());
    let kt = g.add_card_to_hand(0, catalog::kitsunes_technique());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    cast_at(&mut g, kt, Target::Player(1));
    assert_eq!(gy(&g, 1), 16, "31 cards → 16 milled");
}

/// Will of the Sultai (no commander) — one mode: mill three, lands come home.
#[test]
fn will_of_the_sultai_mill_mode_returns_lands() {
    let mut g = main_phase(2);
    g.add_card_to_graveyard(0, catalog::island());
    let will = g.add_card_to_hand(0, catalog::will_of_the_sultai());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: will,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("cast mode 0");
    drain_stack(&mut g);
    // Seat 0 milled three Forests; they and the Island all return tapped.
    assert_eq!(on_bf(&g, "Forest"), 3);
    assert_eq!(on_bf(&g, "Island"), 1);
    assert!(g.battlefield.iter().filter(|c| c.definition.is_land()).all(|c| c.tapped));
}

/// Breach the Multiverse mills everyone ten and steals a creature from each
/// graveyard, all turned Phyrexian.
#[test]
fn breach_the_multiverse_reanimates_one_per_player() {
    let mut g = main_phase(2);
    g.add_card_to_graveyard(0, catalog::shorecomber_crab());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let breach = g.add_card_to_hand(0, catalog::breach_the_multiverse());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(5);
    cast(&mut g, breach);
    let mine: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.is_creature())
        .map(|c| c.id)
        .collect();
    assert_eq!(mine.len(), 2);
    for id in mine {
        assert!(g.computed_permanent(id).unwrap().subtypes().creature_types.contains(&CreatureType::Phyrexian));
    }
    assert_eq!(gy(&g, 1), 10);
}

/// Nanogene Conversion turns every other creature into a copy.
#[test]
fn nanogene_conversion_copies_onto_every_other_creature() {
    let mut g = main_phase(2);
    let crab = g.add_card_to_battlefield(0, catalog::mirrorshell_crab());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ng = g.add_card_to_hand(0, catalog::nanogene_conversion());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast_at(&mut g, ng, Target::Permanent(crab));
    assert_eq!(pt(&g, bear), (5, 7));
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 1);
}

/// Formless Genesis makes an X/X deathtouch changeling, X = lands in yard, and
/// retraces.
#[test]
fn formless_genesis_scales_with_graveyard_lands() {
    let mut g = main_phase(2);
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::forest());
    }
    let fg = g.add_card_to_hand(0, catalog::formless_genesis());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, fg);
    let tok = g.battlefield.iter().find(|c| c.definition.name == "Shapeshifter").unwrap().id;
    assert_eq!(pt(&g, tok), (3, 3));
    assert!(g.computed_permanent(tok).unwrap().keywords().contains(&Keyword::Deathtouch));
    assert!(catalog::formless_genesis().card_types.contains(&CardType::Kindred));
}

/// Open the Way reveals until X lands and drops them tapped.
#[test]
fn open_the_way_puts_x_lands_onto_the_battlefield() {
    let mut g = main_phase(2);
    let otw = g.add_card_to_hand(0, catalog::open_the_way());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast_x(&mut g, otw, 2);
    assert_eq!(on_bf(&g, "Forest"), 2);
}

/// Afterlife from the Loam brings back a creature from each graveyard as a
/// Zombie.
#[test]
fn afterlife_from_the_loam_reanimates_zombies() {
    let mut g = main_phase(2);
    g.add_card_to_graveyard(0, catalog::shorecomber_crab());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let al = g.add_card_to_hand(0, catalog::afterlife_from_the_loam());
    g.players[0].mana_pool.add(Color::Black, 3);
    g.players[0].mana_pool.add_colorless(5);
    cast(&mut g, al);
    let bear = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").expect("stolen");
    assert_eq!(bear.controller, 0);
    let cp = g.computed_permanent(bear.id).unwrap();
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Zombie));
    assert_eq!(on_bf(&g, "Shorecomber Crab"), 1);
}

/// Reshape the Earth puts ten lands onto the battlefield.
#[test]
fn reshape_the_earth_fetches_ten_lands() {
    let mut g = main_phase(2);
    let rte = g.add_card_to_hand(0, catalog::reshape_the_earth());
    g.players[0].mana_pool.add(Color::Green, 3);
    g.players[0].mana_pool.add_colorless(6);
    cast(&mut g, rte);
    assert_eq!(on_bf(&g, "Forest"), 10);
}

// ── Artifacts ────────────────────────────────────────────────────────────────

/// Maskwood Nexus gives your creatures changeling and mints changelings that
/// Homer counts.
#[test]
fn maskwood_nexus_grants_changeling_and_mints_shapeshifters() {
    let mut g = main_phase(2);
    let nexus = g.add_card_to_battlefield(0, catalog::maskwood_nexus());
    g.add_card_to_battlefield(0, catalog::homer_the_hermit());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Changeling));
    g.players[0].mana_pool.add_colorless(3);
    activate(&mut g, nexus, 0, None);
    assert_eq!(on_bf(&g, "Shapeshifter"), 1);
    play_land(&mut g, 0, catalog::forest());
    let milled: usize = (0..2).map(|p| gy(&g, p)).max().unwrap();
    assert!(milled >= 4, "Homer and the changeling token are Crabs: {milled}");
}

/// Altar of the Brood mills each opponent per permanent you land.
#[test]
fn altar_of_the_brood_mills_each_opponent() {
    let mut g = main_phase(3);
    g.add_card_to_battlefield(0, catalog::altar_of_the_brood());
    play_land(&mut g, 0, catalog::forest());
    assert_eq!((gy(&g, 0), gy(&g, 1), gy(&g, 2)), (0, 1, 1));
}

/// Exploration Broodship becomes a 4/4 flier at eight charge counters.
#[test]
fn exploration_broodship_stations_into_a_flier() {
    let mut g = main_phase(2);
    let ship = g.add_card_to_battlefield(0, catalog::exploration_broodship());
    let crab = g.add_card_to_battlefield(0, catalog::mirrorshell_crab());
    g.clear_sickness(crab);
    g.battlefield_find_mut(ship).unwrap().add_counters(CounterType::Charge, 3);
    activate(&mut g, ship, 0, None);
    assert_eq!(g.battlefield_find(ship).unwrap().counter_count(CounterType::Charge), 8);
    let cp = g.computed_permanent(ship).unwrap();
    assert!(cp.card_types().contains(&CardType::Creature));
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.keywords().contains(&Keyword::Flying));
    assert_eq!(g.max_lands_per_turn(0), 2, "the {{3+}} land drop this turn");
}

/// Exploration Broodship {8+}: once during each of your turns, cast a
/// permanent spell from your graveyard by sacrificing a land (the chosen one)
/// in addition to its other costs. Below eight counters, or a second time in
/// the turn, the graveyard card can't be cast.
#[test]
fn exploration_broodship_casts_a_permanent_from_the_graveyard_for_a_land() {
    let mut g = main_phase(2);
    let ship = g.add_card_to_battlefield(0, catalog::exploration_broodship());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let island = g.add_card_to_battlefield(0, catalog::island());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bears2 = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let cast = |g: &mut GameState, card_id| {
        g.perform_action(GameAction::CastSpell {
            card_id,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    g.players[0].mana_pool.add(Color::Green, 4);
    g.battlefield_find_mut(ship).unwrap().add_counters(CounterType::Charge, 7);
    assert!(cast(&mut g, bears).is_err(), "seven counters: the {{8+}} band is off");
    g.battlefield_find_mut(ship).unwrap().add_counters(CounterType::Charge, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![island])]));
    cast(&mut g, bears).expect("cast from the graveyard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_some(), "the Bears resolved");
    assert!(g.battlefield_find(island).is_none(), "the chosen land was sacrificed");
    assert!(g.battlefield_find(forest).is_some(), "the other land stays");
    assert!(cast(&mut g, bears2).is_err(), "once each turn");
}

/// Firdoch Core taps for any color and animates into a 4/4.
#[test]
fn firdoch_core_animates() {
    let mut g = main_phase(2);
    let core = g.add_card_to_battlefield(0, catalog::firdoch_core());
    g.players[0].mana_pool.add_colorless(4);
    activate(&mut g, core, 1, None);
    let cp = g.computed_permanent(core).unwrap();
    assert!(cp.card_types().contains(&CardType::Creature));
    assert_eq!((cp.power, cp.toughness), (4, 4));
}

// ── Enchantments ─────────────────────────────────────────────────────────────

/// Black Market Connections' default picks: a Treasure and a card for 3 life.
#[test]
fn black_market_connections_sells_and_buys() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::black_market_connections());
    let hand = g.players[0].hand.len();
    enter_step(&mut g, TurnStep::Draw);
    assert_eq!(on_bf(&g, "Treasure"), 1);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.players[0].life, 17);
}

/// Arcane Adaptation adds the chosen type to your creatures.
#[test]
fn arcane_adaptation_types_your_creatures() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Crab)]));
    let aa = g.add_card_to_hand(0, catalog::arcane_adaptation());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, aa);
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Crab));
}

/// Fraying Sanity doubles whatever hit the cursed player's graveyard.
#[test]
fn fraying_sanity_mills_the_turns_graveyard_count() {
    let mut g = main_phase(2);
    let fs = g.add_card_to_hand(0, catalog::fraying_sanity());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_at(&mut g, fs, Target::Player(1));
    mill_seat(&mut g, 1, 2);
    enter_step(&mut g, TurnStep::PostCombatMain);
    assert_eq!(gy(&g, 1), 4);
}

/// Druid Class: landfall life at level 1; level 2 opens a second land drop.
#[test]
fn druid_class_gains_life_and_levels_into_land_drops() {
    let mut g = main_phase(2);
    let class = g.add_card_to_hand(0, catalog::druid_class());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, class);
    play_land(&mut g, 0, catalog::forest());
    assert_eq!(g.players[0].life, 21);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, class, 0, None);
    play_land(&mut g, 0, catalog::forest());
    assert_eq!(g.players[0].life, 22, "second land drop this turn");
}

/// Virtue of Knowledge doubles ETB triggers; its adventure copies an ability.
#[test]
fn virtue_of_knowledge_doubles_etb_triggers() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::virtue_of_knowledge());
    let cc = g.add_card_to_hand(0, catalog::crustacean_commando());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, cc);
    assert_eq!(on_bf(&g, "Mutagen"), 2);
    assert!(catalog::virtue_of_knowledge().adventure.is_some());
}

/// Memory Erosion mills an opponent two per spell they cast.
#[test]
fn memory_erosion_mills_casting_opponents() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::memory_erosion());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, bolt, Target::Player(0));
    assert_eq!(gy(&g, 1), 3, "two milled plus the Bolt");
}

/// Rites of Flourishing: an extra draw in each draw step, an extra land drop.
#[test]
fn rites_of_flourishing_extra_draw_and_land() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::rites_of_flourishing());
    let hand = g.players[0].hand.len();
    g.turn_number = 3; // past the starting player's skipped first draw
    enter_step(&mut g, TurnStep::Untap);
    assert_eq!(g.max_lands_per_turn(0), 2);
    enter_step(&mut g, TurnStep::Upkeep);
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

// ── Lands ────────────────────────────────────────────────────────────────────

/// Hobbit Hole cracks for a tapped basic.
#[test]
fn hobbit_hole_fetches_a_tapped_basic() {
    let mut g = main_phase(2);
    let hole = g.add_card_to_battlefield(0, catalog::hobbit_hole());
    activate(&mut g, hole, 0, None);
    assert!(g.battlefield_find(hole).is_none());
    let f = g.battlefield.iter().find(|c| c.definition.name == "Forest").expect("fetched");
    assert!(f.tapped);
}

/// Drownyard Temple climbs back from the graveyard for {3}.
#[test]
fn drownyard_temple_returns_from_the_graveyard() {
    let mut g = main_phase(2);
    let temple = g.add_card_to_graveyard(0, catalog::drownyard_temple());
    g.players[0].mana_pool.add_colorless(3);
    activate(&mut g, temple, 1, None);
    assert!(g.battlefield_find(temple).unwrap().tapped);
}

/// Riveteers Overlook sacrifices itself for a basic Forest and a life.
#[test]
fn riveteers_overlook_cracks_for_a_basic() {
    let mut g = main_phase(2);
    play_land(&mut g, 0, catalog::riveteers_overlook());
    assert_eq!(on_bf(&g, "Riveteers Overlook"), 0);
    assert_eq!(on_bf(&g, "Forest"), 1);
    assert_eq!(g.players[0].life, 21);
}
