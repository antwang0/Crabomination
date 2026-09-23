//! Commander: the Teval, the Balanced Scale batch (`decks::cmdr_teval`), and
//! the Wretched Ranks precon's missing cards (`decks::cmdr_gisa`), at the end.

use crabomination::card::{CardDefinition, CardId, CardType, CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

/// Put `def` onto seat 0's battlefield and resolve its ETB triggers.
fn etb(g: &mut GameState, def: CardDefinition) -> CardId {
    // CR 614.12 — the entry runs its as-enters replacements first;
    // `add_card_to_battlefield` skips every one of them.
    let id = g.add_card_to_battlefield_entering(0, def);
    g.fire_self_etb_triggers(id, 0);
    drain_stack(g);
    id
}

fn cast_with(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
    flood(g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: x,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    flood(g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .unwrap_or_else(|e| panic!("activate: {e:?}"));
    drain_stack(g);
}

/// Seat 0 attacks seat 1 with `attacker`; no blocks; runs to end of combat.
fn swing(g: &mut GameState, attacker: CardId) {
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    drain_stack(g);
}

fn count_named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.name == name)
        .count()
}

fn on_battlefield(g: &GameState, id: CardId) -> bool {
    g.battlefield_find(id).is_some()
}

fn in_graveyard(g: &GameState, seat: usize, id: CardId) -> bool {
    g.players[seat].graveyard.iter().any(|c| c.id == id)
}

fn in_hand(g: &GameState, seat: usize, id: CardId) -> bool {
    g.players[seat].hand.iter().any(|c| c.id == id)
}

fn in_exile(g: &GameState, id: CardId) -> bool {
    g.exile.iter().any(|c| c.id == id)
}

fn has_keyword(g: &GameState, id: CardId, kw: Keyword) -> bool {
    g.computed_permanent(id).unwrap().keywords().contains(&kw)
}

/// Cast Raise Dead on `target` — one card leaving seat 0's graveyard.
fn raise(g: &mut GameState, target: CardId) {
    let spell = g.add_card_to_hand(0, catalog::raise_dead());
    cast_with(g, spell, Some(Target::Permanent(target)), None);
}

// ── Teval and the graveyard creatures ────────────────────────────────────────

/// Teval's attack mills three and brings a land back tapped; that land leaving
/// the graveyard is one batch, so exactly one Zombie Druid.
#[test]
fn teval_mills_returns_a_land_and_makes_one_zombie_per_batch() {
    let mut g = main_phase();
    let teval = g.add_card_to_battlefield(0, catalog::teval_the_balanced_scale());
    let forest = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::lightning_bolt());
    swing(&mut g, teval);
    assert!(g.players[0].library.is_empty(), "milled three");
    let land = g.battlefield_find(forest).expect("the land came back");
    assert!(land.tapped, "returned tapped");
    assert_eq!(count_named(&g, 0, "Zombie Druid"), 1);
    // A second batch (a creature card leaving) makes a second Zombie.
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    let bears = g.players[0].graveyard.iter().find(|c| c.definition.name == "Grizzly Bears").unwrap().id;
    raise(&mut g, bears);
    assert_eq!(count_named(&g, 0, "Zombie Druid"), 2);
}

/// Thranduil pumps other Elves, makes an Elf per land, and its adventure mills
/// four keeping up to two lands.
#[test]
fn thranduil_landfall_elves_and_silvan_rally() {
    let mut g = main_phase();
    let thranduil = g.add_card_to_battlefield(0, catalog::thranduil_sindarin_liege());
    let forest = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(forest)).expect("play land");
    drain_stack(&mut g);
    let elf = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Elf")
        .map(|c| c.id)
        .expect("landfall Elf");
    let cp = g.computed_permanent(elf).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "other Elves get +1/+1");
    let me = g.computed_permanent(thranduil).unwrap();
    assert_eq!((me.power, me.toughness), (2, 3), "not itself");

    let mut g = main_phase();
    let card = g.add_card_to_hand(0, catalog::thranduil_sindarin_liege());
    let a = g.add_card_to_library(0, catalog::island());
    let b = g.add_card_to_library(0, catalog::swamp());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastAdventure {
        card_id: card,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Silvan Rally");
    drain_stack(&mut g);
    assert!(in_hand(&g, 0, a) && in_hand(&g, 0, b), "both lands to hand");
    assert_eq!(g.players[0].graveyard.len(), 2, "the other two stay milled");
    assert!(in_exile(&g, card), "on an adventure");
}

/// Colossal Grave-Reaver's ETB mill of two creatures puts one onto the
/// battlefield; Sidisi's makes one Zombie for the batch.
#[test]
fn grave_reaver_and_sidisi_pay_off_creature_mills_once_per_batch() {
    let mut g = main_phase();
    let a = g.add_card_to_library(0, catalog::grizzly_bears());
    let b = g.add_card_to_library(0, catalog::llanowar_elves());
    g.add_card_to_library(0, catalog::forest());
    etb(&mut g, catalog::colossal_grave_reaver());
    let returned = [a, b].iter().filter(|id| on_battlefield(&g, **id)).count();
    assert_eq!(returned, 1, "one of them onto the battlefield");

    let mut g = main_phase();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::llanowar_elves());
    g.add_card_to_library(0, catalog::forest());
    etb(&mut g, catalog::sidisi_brood_tyrant());
    assert_eq!(g.players[0].graveyard.len(), 3);
    assert_eq!(count_named(&g, 0, "Zombie"), 1, "one token for the batch");
}

/// River Kelpie draws when a permanent enters from a graveyard and when a
/// spell is cast from a graveyard.
#[test]
fn river_kelpie_draws_off_graveyard_entries_and_casts() {
    let mut g = main_phase();
    stock_libraries(&mut g, 10);
    g.add_card_to_battlefield(0, catalog::river_kelpie());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    let reanimate = g.add_card_to_hand(0, catalog::reanimate());
    cast_with(&mut g, reanimate, Some(Target::Permanent(bears)), None);
    assert!(on_battlefield(&g, bears));
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew for the graveyard entry");

    let hand = g.players[0].hand.len();
    let welcome = g.add_card_to_graveyard(0, catalog::welcome_the_dead());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFlashback {
        card_id: welcome,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("flashback");
    drain_stack(&mut g);
    // +1 Kelpie, +2 Welcome the Dead, -1 discard.
    assert_eq!(g.players[0].hand.len(), hand + 2, "drew for the graveyard cast");
}

/// River Kelpie's cast trigger is "from a graveyard" only (bug fix: it read
/// "not from hand", so every commander cast from the command zone drew).
/// Breathless Knight's "entered from a graveyard" rider likewise ignores a
/// token (it read "not cast from hand" too).
#[test]
fn river_kelpie_ignores_a_command_zone_cast() {
    let mut g = main_phase();
    stock_libraries(&mut g, 10);
    g.add_card_to_battlefield(0, catalog::river_kelpie());
    let knight = g.add_card_to_battlefield(0, catalog::breathless_knight());
    let cmd = g.seat_commanders(0, vec![catalog::grizzly_bears()])[0];
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("cast the commander");
    drain_stack(&mut g);
    assert!(on_battlefield(&g, cmd));
    assert_eq!(g.players[0].hand.len(), hand, "a command-zone cast is not a graveyard cast");
    let knight_counters = |g: &GameState| {
        g.battlefield_find(knight).unwrap().counter_count(CounterType::PlusOnePlusOne)
    };
    assert_eq!(knight_counters(&g), 0, "nor did it enter from a graveyard");
    let spawn = g.add_card_to_hand(0, catalog::raise_the_alarm());
    cast_with(&mut g, spawn, None, None);
    assert_eq!(knight_counters(&g), 0, "tokens don't come from a graveyard");
}

/// Kotis lets a creature be cast from the graveyard by exiling three other
/// cards, and grows when it does.
#[test]
fn kotis_casts_a_creature_from_the_graveyard_and_grows() {
    let mut g = main_phase();
    let kotis = g.add_card_to_battlefield(0, catalog::kotis_sibsig_champion());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let fodder: Vec<CardId> =
        (0..3).map(|_| g.add_card_to_graveyard(0, catalog::forest())).collect();
    flood(&mut g, 0);
    g.perform_action(GameAction::CastEscape {
        card_id: bears,
        exile_cards: fodder.clone(),
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from the graveyard");
    drain_stack(&mut g);
    assert!(on_battlefield(&g, bears));
    assert!(fodder.iter().all(|id| in_exile(&g, *id)));
    assert_eq!(g.battlefield_find(kotis).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);

    // "Once during each of your turns": the grant is spent for this turn.
    let other = g.add_card_to_graveyard(0, catalog::llanowar_elves());
    let fodder: Vec<CardId> =
        (0..3).map(|_| g.add_card_to_graveyard(0, catalog::forest())).collect();
    flood(&mut g, 0);
    let again = |g: &mut GameState, fodder: &[CardId]| {
        g.perform_action(GameAction::CastEscape {
            card_id: other,
            exile_cards: fodder.to_vec(),
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    assert!(again(&mut g, &fodder).is_err(), "second graveyard cast in one turn");

    // Not on an opponent's turn either.
    g.players[0].graveyard_sac_cast_sources_this_turn.clear();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 0;
    assert!(g
        .perform_action(GameAction::CastEscape {
            card_id: other,
            exile_cards: fodder,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err());
}

/// Syr Konrad pings each opponent for a milled creature card and for a creature
/// card leaving your graveyard, and its activation mills everyone.
#[test]
fn syr_konrad_pings_for_graveyard_traffic() {
    let mut g = main_phase();
    let konrad = g.add_card_to_battlefield(0, catalog::syr_konrad_the_grim());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::forest());
    activate(&mut g, konrad, 0, None);
    assert_eq!(g.players[0].graveyard.len(), 1);
    assert_eq!(g.players[1].graveyard.len(), 1, "each player mills");
    assert_eq!(g.players[1].life, 19, "the milled creature card");
    let bears = g.players[0].graveyard[0].id;
    raise(&mut g, bears);
    assert_eq!(g.players[1].life, 18, "a creature card left your graveyard");
}

/// The Scarab God reanimates a graveyard creature as a 4/4 black Zombie and
/// drains for Zombies each upkeep.
#[test]
fn the_scarab_god_makes_zombie_copies_and_drains() {
    let mut g = main_phase();
    let god = g.add_card_to_battlefield(0, catalog::the_scarab_god());
    let bears = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    activate(&mut g, god, 0, Some(Target::Permanent(bears)));
    assert!(in_exile(&g, bears));
    let copy = g
        .battlefield
        .iter()
        .find(|c| c.controller == 0 && c.definition.name == "Grizzly Bears")
        .map(|c| c.id)
        .expect("token copy");
    let cp = g.computed_permanent(copy).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Zombie));
    assert_eq!(cp.colors.to_vec(), vec![Color::Black]);

    stock_libraries(&mut g, 5);
    g.step = TurnStep::Untap;
    advance_to(&mut g, TurnStep::Draw);
    assert_eq!(g.players[1].life, 19, "one Zombie → lose 1");
}

/// Shigeki bounces itself to dig a land onto the battlefield and bin the rest.
#[test]
fn shigeki_digs_a_land_and_bins_the_rest() {
    let mut g = main_phase();
    let shigeki = g.add_card_to_battlefield(0, catalog::shigeki_jukai_visionary());
    g.clear_sickness(shigeki);
    let land = g.add_card_to_library(0, catalog::forest());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    activate(&mut g, shigeki, 0, None);
    assert!(in_hand(&g, 0, shigeki), "returned to hand as a cost");
    assert!(g.battlefield_find(land).is_some_and(|c| c.tapped), "land in tapped");
    assert_eq!(g.players[0].graveyard.len(), 3);
}

/// Floral Evoker grows on landfall and pitches a creature to return a land.
#[test]
fn floral_evoker_landfall_and_land_recursion() {
    let mut g = main_phase();
    let evoker = g.add_card_to_battlefield(0, catalog::floral_evoker());
    let forest = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(forest)).expect("play land");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(evoker).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let dead = g.add_card_to_graveyard(0, catalog::swamp());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    activate(&mut g, evoker, 0, Some(Target::Permanent(dead)));
    assert!(g.battlefield_find(dead).is_some_and(|c| c.tapped));
}

/// Steward of the Harvest exiles up to three lands, and "creatures you
/// control" — not only the Steward — have their activated abilities.
#[test]
fn steward_of_the_harvest_borrows_exiled_land_abilities() {
    let mut g = main_phase();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let drownyard = g.add_card_to_graveyard(0, catalog::nephalia_drownyard());
    let steward = etb(&mut g, catalog::steward_of_the_harvest());
    assert!(in_exile(&g, drownyard));
    for id in [steward, bears] {
        assert_eq!(g.granted_abilities_for(id).len(), 2, "{{T}}: Add {{C}} and the mill ability");
    }
    assert!(g.granted_abilities_for(theirs).is_empty(), "an opponent's creature gets nothing");
    // The Bears activate Drownyard's "{1}{U}{B}, {T}: Target player mills
    // three cards."
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::forest());
    }
    let before = g.players[1].graveyard.len();
    g.battlefield_find_mut(bears).unwrap().summoning_sick = false;
    activate(&mut g, bears, 1, Some(Target::Player(1)));
    assert_eq!(g.players[1].graveyard.len(), before + 3);
    assert!(g.battlefield_find(bears).unwrap().tapped);
}

/// Diviner of Mist's attack mills four and casts a cheap sorcery free, exiling it.
#[test]
fn diviner_of_mist_casts_a_milled_sorcery_free() {
    let mut g = main_phase();
    let diviner = g.add_card_to_battlefield(0, catalog::diviner_of_mist());
    let div = g.add_card_to_library(0, catalog::divination());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    stock_libraries(&mut g, 5);
    let hand = g.players[0].hand.len();
    swing(&mut g, diviner);
    assert_eq!(g.players[0].hand.len(), hand + 2, "Divination resolved");
    assert!(in_exile(&g, div), "exiled instead of graveyard");
}

/// Lost Monarch's afflict (and the one it grants other Zombies) drains the
/// defending player when blocked.
#[test]
fn lost_monarch_afflicts_and_grants_afflict() {
    let mut g = main_phase();
    let monarch = g.add_card_to_battlefield(0, catalog::lost_monarch_of_ifnir());
    let corpse = g.add_card_to_battlefield(0, catalog::walking_corpse());
    let wall = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wall2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for id in [monarch, corpse] {
        g.clear_sickness(id);
    }
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![
        Attack { attacker: monarch, target: AttackTarget::Player(1) },
        Attack { attacker: corpse, target: AttackTarget::Player(1) },
    ])
    .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, monarch), (wall2, corpse)]))
        .expect("block");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 14, "afflict 3 twice");
}

/// Lord of Extinction counts every card in every graveyard.
#[test]
fn lord_of_extinction_counts_all_graveyards() {
    let mut g = main_phase();
    let lord = g.add_card_to_battlefield(0, catalog::lord_of_extinction());
    g.add_card_to_graveyard(0, catalog::forest());
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let cp = g.computed_permanent(lord).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
}

/// Lord of the Forsaken: "{B}, Sacrifice another creature: Target player
/// mills three cards."
#[test]
fn lord_of_the_forsaken_sacrifices_to_mill_a_target_player() {
    let mut g = main_phase();
    let lord = g.add_card_to_battlefield(0, catalog::lord_of_the_forsaken());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..4 {
        g.add_card_to_library(1, catalog::forest());
    }
    let before = g.players[1].graveyard.len();
    activate(&mut g, lord, 0, Some(Target::Player(1)));
    assert!(in_graveyard(&g, 0, bears), "the other creature is the cost");
    assert!(on_battlefield(&g, lord));
    assert_eq!(g.players[1].graveyard.len(), before + 3);
}

/// Lord of the Forsaken's "Pay 1 life: Add {C}. Spend this mana only to cast
/// a spell from your graveyard" — a CR 106.6 spend restriction keyed on the
/// zone the spell is cast from (CR 601.2a): a hand cast can't use it, a
/// flashback cast can.
#[test]
fn lord_of_the_forsaken_mana_only_funds_graveyard_casts() {
    let mut g = main_phase();
    let lord = g.add_card_to_battlefield(0, catalog::lord_of_the_forsaken());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let life = g.players[0].life;
    for _ in 0..2 {
        g.perform_action(GameAction::ActivateAbility {
            card_id: lord,
            ability_index: 1,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("pay 1 life: add {C}");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].life, life - 2);
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    let cast = |g: &mut GameState, card_id| {
        g.perform_action(GameAction::CastSpell {
            card_id,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    assert!(cast(&mut g, ring).is_err(), "a hand cast can't spend it");
    // Faithless Looting's flashback is {2}{R}: the two restricted {C} pay
    // the generic half.
    let looting = g.add_card_to_graveyard(0, catalog::faithless_looting());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastFlashback {
        card_id: looting,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("flashback funded by the graveyard-only mana");
    assert_eq!(g.players[0].mana_pool.total(), 0);
}

/// CR 106.7 — Exotic Orchard makes a color "a land an opponent controls could
/// produce": a nonbasic's mana abilities count, and with no such land (or
/// only another Orchard facing it, the rule's own example) it makes nothing.
#[test]
fn cr_106_7_exotic_orchard_reads_opponents_lands() {
    let tap = |g: &mut GameState, id| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("tap");
        drain_stack(g);
    };
    let mut g = main_phase();
    let orchard = g.add_card_to_battlefield(0, catalog::exotic_orchard());
    g.add_card_to_battlefield(1, catalog::exotic_orchard());
    tap(&mut g, orchard);
    assert_eq!(g.players[0].mana_pool.total(), 0, "Orchard facing Orchard makes nothing");

    let mut g = main_phase();
    let orchard = g.add_card_to_battlefield(0, catalog::exotic_orchard());
    g.add_card_to_battlefield(1, catalog::hinterland_harbor());
    tap(&mut g, orchard);
    let pool = &g.players[0].mana_pool;
    assert_eq!(pool.total(), 1);
    assert_eq!(pool.amount(Color::Green) + pool.amount(Color::Blue), 1, "only {{G}} or {{U}}");
}

/// Tormod makes one tapped Zombie per batch of cards leaving your graveyard.
#[test]
fn tormod_makes_a_tapped_zombie() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::tormod_the_desecrator());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    raise(&mut g, bears);
    let zombies: Vec<_> =
        g.battlefield.iter().filter(|c| c.definition.name == "Zombie").collect();
    assert_eq!(zombies.len(), 1);
    assert!(zombies[0].tapped);
}

/// Amphin Mutineer exiles an opposing creature and hands its controller a
/// 4/3 Salamander Warrior.
#[test]
fn amphin_mutineer_exiles_and_compensates() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mutineer = g.add_card_to_hand(0, catalog::amphin_mutineer());
    cast_with(&mut g, mutineer, None, None);
    drain_stack(&mut g);
    assert!(in_exile(&g, victim));
    assert_eq!(count_named(&g, 1, "Salamander Warrior"), 1);
}

// ── Instants and sorceries ───────────────────────────────────────────────────

/// Lethal Scheme destroys a creature; Midnight Tilling mills four and keeps a
/// permanent card.
#[test]
fn lethal_scheme_and_midnight_tilling() {
    let mut g = main_phase();
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let scheme = g.add_card_to_hand(0, catalog::lethal_scheme());
    cast_with(&mut g, scheme, Some(Target::Permanent(bears)), None);
    assert!(in_graveyard(&g, 1, bears));

    let mut g = main_phase();
    let keep = g.add_card_to_library(0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    let tilling = g.add_card_to_hand(0, catalog::midnight_tilling());
    cast_with(&mut g, tilling, None, None);
    assert!(in_hand(&g, 0, keep));
}

/// Welcome the Dead counts cards put into your graveyard from your hand or
/// library this turn — two milled earlier plus its own discard — and not a
/// creature that died.
#[test]
fn welcome_the_dead_makes_zombies_for_cards_binned_this_turn() {
    let mut g = main_phase();
    stock_libraries(&mut g, 5);
    g.add_card_to_library(0, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let mill = crabomination::effect::Effect::Mill {
        who: crabomination::effect::Selector::You,
        amount: crabomination::card::Value::Const(2),
    };
    g.resolve_effect(&mill, &ctx).expect("mill");
    let dies = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.remove_from_battlefield_to_graveyard_raw(dies);
    let spell = g.add_card_to_hand(0, catalog::welcome_the_dead());
    cast_with(&mut g, spell, None, None);
    assert_eq!(g.players[0].life, 18);
    let zombies: Vec<_> =
        g.battlefield.iter().filter(|c| c.definition.name == "Zombie Druid").collect();
    assert_eq!(zombies.len(), 3, "two milled plus the discard; the dead Elves don't count");
    assert!(zombies.iter().all(|z| z.tapped));
}

/// Necromantic Selection wipes the board and steals one of the dead as a Zombie.
#[test]
fn necromantic_selection_wipes_and_reanimates_one() {
    let mut g = main_phase();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let spell = g.add_card_to_hand(0, catalog::necromantic_selection());
    cast_with(&mut g, spell, None, None);
    let creatures: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.card_types.contains(&CardType::Creature))
        .collect();
    assert_eq!(creatures.len(), 1, "one comes back");
    assert_eq!(creatures[0].controller, 0, "under your control");
    let back = creatures[0].id;
    let cp = g.computed_permanent(back).unwrap();
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Zombie));
    assert!(cp.colors.contains(Color::Black), "a black Zombie in addition to green");
    assert!(cp.colors.contains(Color::Green));
    assert!(in_exile(&g, spell), "exiles itself");
    let _ = theirs;
}

/// Rise of the Witch-king: everyone sacrifices; you get another permanent back.
#[test]
fn rise_of_the_witch_king_returns_another_permanent() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let old = g.add_card_to_graveyard(0, catalog::llanowar_elves());
    let spell = g.add_card_to_hand(0, catalog::rise_of_the_witch_king());
    cast_with(&mut g, spell, None, None);
    assert!(in_graveyard(&g, 0, mine) && in_graveyard(&g, 1, theirs));
    assert!(on_battlefield(&g, old), "another permanent card returns");
}

/// Agadeem's Awakening returns at most one creature card per mana value ≤ X.
#[test]
fn agadeems_awakening_returns_one_per_mana_value() {
    let mut g = main_phase();
    let elves = g.add_card_to_graveyard(0, catalog::llanowar_elves());
    let elves2 = g.add_card_to_graveyard(0, catalog::llanowar_elves());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let big = g.add_card_to_graveyard(0, catalog::the_scarab_god());
    let spell = g.add_card_to_hand(0, catalog::agadeems_awakening());
    cast_with(&mut g, spell, None, Some(2));
    assert!(on_battlefield(&g, bears));
    assert_eq!([elves, elves2].iter().filter(|id| on_battlefield(&g, **id)).count(), 1);
    assert!(!on_battlefield(&g, big), "mana value above X");
    // The back face: a land that taps for {B}.
    let back = catalog::agadeems_awakening().back_face.unwrap();
    assert_eq!(back.name, "Agadeem, the Undercrypt");
    assert!(back.card_types.contains(&CardType::Land));
}

// ── Enchantments ─────────────────────────────────────────────────────────────

/// Dancing from Dark to Dawn: a creature spell's mana value in counters, and a
/// Bear on landfall.
#[test]
fn dancing_from_dark_to_dawn_counters_and_bears() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::dancing_from_dark_to_dawn());
    let host = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_with(&mut g, bears, None, None);
    let counters: u32 = [host, bears]
        .iter()
        .filter_map(|id| g.battlefield_find(*id))
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .sum();
    assert_eq!(counters, 2, "mana value 2");
    let forest = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(forest)).expect("play land");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Bear"), 1);
}

/// Teval's Judgment picks a fresh mode per batch, runs dry after three, and
/// resets the next turn (`Effect::ChooseUnchosenModeThisTurn`).
#[test]
fn tevals_judgment_modes_reset_each_turn() {
    let mut g = main_phase();
    stock_libraries(&mut g, 10);
    g.add_card_to_battlefield(0, catalog::tevals_judgment());
    let tally = |g: &GameState| {
        (
            count_named(g, 0, "Treasure"),
            count_named(g, 0, "Zombie Druid"),
        )
    };
    let mut dead: Vec<CardId> =
        (0..5).map(|_| g.add_card_to_graveyard(0, catalog::grizzly_bears())).collect();
    for _ in 0..4 {
        raise(&mut g, dead.pop().unwrap());
    }
    assert_eq!(tally(&g), (1, 1), "treasure and zombie once each; the fourth batch finds nothing");
    let raised_in_hand = g.players[0].hand.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    // Four Raise Deads + one draw.
    assert_eq!(g.players[0].hand.len(), raised_in_hand + 1, "drew exactly once");
    g.turn_number += 1;
    raise(&mut g, dead.pop().unwrap());
    assert_eq!(g.players[0].hand.len(), raised_in_hand + 3, "a new turn: draw again");
}

/// Crawling Sensation's Insect fires once a turn however many lands are binned.
#[test]
fn crawling_sensation_once_a_turn() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::crawling_sensation());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let tilling = g.add_card_to_hand(0, catalog::midnight_tilling());
    cast_with(&mut g, tilling, None, None);
    assert_eq!(count_named(&g, 0, "Insect"), 1);
}

/// Titans' Nest's mana: a card from the graveyard for a restricted {C}.
#[test]
fn titans_nest_exiles_for_restricted_colorless() {
    let mut g = main_phase();
    let nest = g.add_card_to_battlefield(0, catalog::titans_nest());
    let fuel = g.add_card_to_graveyard(0, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: nest,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(in_exile(&g, fuel));
    // A colored spell can use it; a colorless one can't.
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    assert!(g
        .perform_action(GameAction::CastSpell {
            card_id: ring,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err());
}

/// Reflections of Littjara copies spells of the chosen type.
#[test]
fn reflections_of_littjara_copies_the_chosen_type() {
    let mut g = main_phase();
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::CreatureType(CreatureType::Bear),
    ]));
    etb(&mut g, catalog::reflections_of_littjara());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_with(&mut g, bears, None, None);
    assert_eq!(count_named(&g, 0, "Grizzly Bears"), 2, "the card and a token copy");
}

/// Rogue Class exiles the damaged player's top card face down, and level 2
/// gives your creatures menace.
#[test]
fn rogue_class_exiles_on_hit_and_levels_into_menace() {
    let mut g = main_phase();
    let class = g.add_card_to_battlefield(0, catalog::rogue_class());
    g.battlefield_find_mut(class).unwrap().class_level = 1;
    let top = g.add_card_to_library(1, catalog::lightning_bolt());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    swing(&mut g, bears);
    let exiled = g.exile.iter().find(|c| c.id == top).expect("top card exiled");
    assert!(exiled.face_down && exiled.may_play_until.is_none(), "face down, not yet playable");
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    assert!(!has_keyword(&g, bears, Keyword::Menace));
    activate(&mut g, class, 0, None);
    assert!(has_keyword(&g, bears, Keyword::Menace));

    // Level 3: the next hit's card is playable.
    g.battlefield_find_mut(class).unwrap().class_level = 3;
    let second = g.add_card_to_library(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().tapped = false;
    swing(&mut g, bears);
    let exiled = g.exile.iter().find(|c| c.id == second).expect("exiled");
    assert_eq!(exiled.may_play_until.unwrap().player, 0, "playable at level 3");
}

// ── Planeswalker ─────────────────────────────────────────────────────────────

/// Ashiok's +1 splits the top two between exile and hand; −2 makes Nightmares.
#[test]
fn ashiok_plus_one_and_minus_two() {
    let mut g = main_phase();
    let ashiok = g.add_card_to_battlefield(0, catalog::ashiok_wicked_manipulator());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::the_scarab_god());
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: ashiok,
        ability_index: 0,
        target: None,
        x_value: None,
    })
    .expect("+1");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.exile.len(), 1);
    assert!(g.players[0].library.is_empty());

    let mut g = main_phase();
    let ashiok = g.add_card_to_battlefield(0, catalog::ashiok_wicked_manipulator());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: ashiok,
        ability_index: 1,
        target: None,
        x_value: None,
    })
    .expect("-2");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Nightmare"), 2);
}

/// Ashiok's static: a life payment becomes exiling that many cards off the top
/// of the library — Phyrexian mana and a "Pay 1 life" activation (Yawgmoth's
/// Bargain) alike. With too small a library the life is paid as usual, and the
/// replacement doesn't make an unaffordable payment affordable (CR 119.4).
#[test]
fn ashiok_life_payments_exile_the_library_instead() {
    fn with_ashiok(library: usize) -> GameState {
        let mut g = main_phase();
        g.add_card_to_battlefield(0, catalog::ashiok_wicked_manipulator());
        for _ in 0..library {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        g
    }
    let cast_growth = |g: &mut GameState, bears: CardId| {
        let growth = g.add_card_to_hand(0, catalog::mutagenic_growth());
        let r = g.perform_action(GameAction::CastSpell {
            card_id: growth,
            target: Some(Target::Permanent(bears)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        });
        drain_stack(g);
        r
    };

    // Phyrexian {G/P} paid with "life": two cards exiled, life untouched.
    let mut g = with_ashiok(5);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    cast_growth(&mut g, bears).expect("cast Mutagenic Growth for 2 life");
    assert_eq!(g.players[0].life, 20, "no life paid");
    assert_eq!(g.players[0].library.len(), 3);
    assert_eq!(g.exile.len(), 2, "two cards exiled instead");
    assert_eq!(g.computed_permanent(bears).unwrap().power, 4);

    // "Pay 1 life: Draw a card" — exile one, draw one.
    let mut g = with_ashiok(5);
    let bargain = g.add_card_to_battlefield(0, catalog::yawgmoths_bargain());
    let hand = g.players[0].hand.len();
    activate(&mut g, bargain, 0, None);
    assert_eq!(g.players[0].life, 20);
    assert_eq!(g.exile.len(), 1);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.players[0].library.len(), 3);

    // Library smaller than the payment: the life is paid.
    let mut g = with_ashiok(1);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    cast_growth(&mut g, bears).expect("cast");
    assert_eq!(g.players[0].life, 18);
    assert_eq!(g.players[0].library.len(), 1);
    assert!(g.exile.is_empty());

    // CR 119.4 — at 0 life the 1-life payment can't be made at all, Ashiok
    // or not: the replacement doesn't make a payment affordable.
    let mut g = with_ashiok(5);
    g.players[0].life = 0;
    let bargain = g.add_card_to_battlefield(0, catalog::yawgmoths_bargain());
    let r = g.perform_action(GameAction::ActivateAbility {
        card_id: bargain,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    });
    assert!(r.is_err(), "can't pay 1 life at 0");
    assert_eq!(g.players[0].library.len(), 5);

    // A fetch land's "pay 1 life" is a cost too, so Ashiok replaces it.
    let mut g = with_ashiok(5);
    g.add_card_to_library(0, catalog::island());
    let delta = g.add_card_to_battlefield(0, catalog::polluted_delta());
    activate(&mut g, delta, 0, None);
    assert_eq!(g.players[0].life, 20, "no life paid for the fetch");
    assert_eq!(g.exile.len(), 1, "one card exiled instead");
}

// ── Artifacts ────────────────────────────────────────────────────────────────

/// Winged Boots and Brotherhood Regalia grant their keywords; the Regalia's
/// legendary equip is {1}.
#[test]
fn equipment_grants() {
    let mut g = main_phase();
    let boots = g.add_card_to_battlefield(0, catalog::winged_boots());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: boots, target: bears }).expect("equip {1}");
    assert!(has_keyword(&g, bears, Keyword::Flying));

    let regalia = g.add_card_to_battlefield(0, catalog::brotherhood_regalia());
    let konrad = g.add_card_to_battlefield(0, catalog::syr_konrad_the_grim());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: regalia, target: konrad })
        .expect("equip legendary {1}");
    assert!(has_keyword(&g, konrad, Keyword::Unblockable));
    assert!(g.computed_permanent(konrad).unwrap().subtypes().creature_types.contains(&CreatureType::Assassin));
}

/// Conjurer's Closet blinks a creature at your end step.
#[test]
fn conjurers_closet_blinks_at_end_step() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::conjurers_closet());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().tapped = true;
    advance_to(&mut g, TurnStep::End);
    let back = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Grizzly Bears")
        .expect("returned");
    assert!(!back.tapped, "a new object");
}

/// Palantír ticks up and scries at your end step, then the opponent picks.
#[test]
fn palantir_of_orthanc_ticks_and_punishes_or_feeds() {
    let mut g = main_phase();
    let palantir = g.add_card_to_battlefield(0, catalog::palantir_of_orthanc());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let (hand, lib, life) = (g.players[0].hand.len(), g.players[0].library.len(), g.players[1].life);
    advance_to(&mut g, TurnStep::End);
    assert_eq!(g.battlefield_find(palantir).unwrap().counter_count(CounterType::Charge), 1);
    let drew = g.players[0].hand.len() == hand + 1;
    let milled = g.players[0].library.len() == lib - 1 && g.players[1].life == life - 2;
    assert!(drew ^ milled, "exactly one branch ran");
}

/// Midnight Clock's twelfth counter resets hand and graveyard into seven new cards.
#[test]
fn midnight_clock_strikes_twelve() {
    let mut g = main_phase();
    let clock = g.add_card_to_battlefield(0, catalog::midnight_clock());
    g.battlefield_find_mut(clock).unwrap().add_counters(CounterType::Charge, 11);
    stock_libraries(&mut g, 10);
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    activate(&mut g, clock, 1, None);
    assert!(in_exile(&g, clock), "exiled");
    assert_eq!(g.players[0].hand.len(), 7);
    assert!(g.players[0].graveyard.is_empty());
}

/// The Soul Stone, once harnessed, reanimates each upkeep.
#[test]
fn the_soul_stone_harnessed_reanimates() {
    let mut g = main_phase();
    let stone = g.add_card_to_battlefield(0, catalog::the_soul_stone());
    let fodder = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    activate(&mut g, stone, 1, None);
    assert!(in_exile(&g, fodder), "exiled as the cost");
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    stock_libraries(&mut g, 5);
    g.battlefield_find_mut(stone).unwrap().tapped = false;
    g.step = TurnStep::Untap;
    g.priority.player_with_priority = 0;
    advance_to(&mut g, TurnStep::Draw);
    assert!(on_battlefield(&g, bears));
}

/// Stonespeaker Crystal taps for two and cashes in to wipe opposing graveyards.
#[test]
fn stonespeaker_crystal_exiles_graveyards_and_draws() {
    let mut g = main_phase();
    stock_libraries(&mut g, 5);
    let crystal = g.add_card_to_battlefield(0, catalog::stonespeaker_crystal());
    let theirs = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    activate(&mut g, crystal, 1, None);
    assert!(in_exile(&g, theirs));
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

// ── Lands ────────────────────────────────────────────────────────────────────

fn play(g: &mut GameState, def: CardDefinition) -> CardId {
    let id = g.add_card_to_hand(0, def);
    g.players[0].lands_played_this_turn = 0;
    g.perform_action(GameAction::PlayLand(id)).expect("play land");
    drain_stack(g);
    id
}

/// The enters-tapped lands, with and without their untapped conditions met.
#[test]
fn cmdr_teval_lands_enter_tapped_unless() {
    type Setup = fn(&mut GameState);
    /// (land factory, board setup, does it enter tapped).
    type TapCase = (fn() -> CardDefinition, Setup, bool);
    let none: Setup = |_| {};
    let cases: Vec<TapCase> = vec![
        (catalog::haunted_mire, none, true),
        (catalog::contaminated_aquifer, none, true),
        (catalog::memorial_to_folly, none, true),
        (catalog::dakmor_salvage, none, true),
        (catalog::minas_morgul_dark_fortress, none, true),
        (catalog::turbulent_wetlands, none, true),
        (
            catalog::turbulent_wetlands,
            |g| {
                for _ in 0..8 {
                    g.add_card_to_battlefield(1, catalog::forest());
                }
            },
            false,
        ),
        (catalog::choked_estuary, none, true),
        (catalog::choked_estuary, |g| { g.add_card_to_hand(0, catalog::island()); }, false),
        (catalog::shipwreck_marsh, |g| { g.add_card_to_battlefield(0, catalog::island()); }, true),
        (
            catalog::shipwreck_marsh,
            |g| {
                g.add_card_to_battlefield(0, catalog::island());
                g.add_card_to_battlefield(0, catalog::swamp());
            },
            false,
        ),
        (catalog::sunken_ruins, none, false),
        (catalog::hidden_lair, none, false),
    ];
    for (factory, setup, tapped) in cases {
        let mut g = main_phase();
        setup(&mut g);
        let name = factory().name;
        let id = play(&mut g, factory());
        assert_eq!(g.battlefield_find(id).unwrap().tapped, tapped, "{name}");
    }
    // The pay-3-life lands: the default choice pays and stays untapped.
    // One card carries the clause today; the loop it used to sit in said
    // "more are coming" and clippy read it as the tautology it was.
    let mut g = main_phase();
    let id = play(&mut g, catalog::the_black_gate());
    assert!(!g.battlefield_find(id).unwrap().tapped);
    assert_eq!(g.players[0].life, 17);
}

/// The colorless utility lands' activations.
#[test]
fn cmdr_teval_utility_land_activations() {
    // Nephalia Drownyard: target player mills three.
    let mut g = main_phase();
    stock_libraries(&mut g, 5);
    let yard = g.add_card_to_battlefield(0, catalog::nephalia_drownyard());
    activate(&mut g, yard, 1, Some(Target::Player(1)));
    assert_eq!(g.players[1].graveyard.len(), 3);

    // Scavenger Grounds: exile all graveyards.
    let mut g = main_phase();
    let grounds = g.add_card_to_battlefield(0, catalog::scavenger_grounds());
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    activate(&mut g, grounds, 1, None);
    assert!(in_exile(&g, a) && in_exile(&g, b));

    // Witch's Clinic: lifelink on a commander — an opponent's too — and a
    // legend that isn't one is no target.
    let mut g = main_phase();
    let clinic = g.add_card_to_battlefield(0, catalog::witchs_clinic());
    let konrad = g.add_card_to_battlefield(0, catalog::syr_konrad_the_grim());
    let cmd = g.seat_commanders(1, vec![catalog::grizzly_bears()])[0];
    let card = g.players[1].command.pop().unwrap();
    g.battlefield.push(card);
    let clinic_fires = |g: &GameState, t| {
        g.would_accept(GameAction::ActivateAbility {
            card_id: clinic, ability_index: 1, target: Some(Target::Permanent(t)),
            additional_targets: vec![], x_value: None, mode: None,
        })
    };
    flood(&mut g, 0);
    assert!(clinic_fires(&g, cmd), "an opponent's commander");
    assert!(!clinic_fires(&g, konrad), "a legend that isn't a commander");
    activate(&mut g, clinic, 1, Some(Target::Permanent(cmd)));
    assert!(has_keyword(&g, cmd, Keyword::Lifelink));

    // Memorial to Folly: a creature card back to hand.
    let mut g = main_phase();
    let memorial = g.add_card_to_battlefield(0, catalog::memorial_to_folly());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    activate(&mut g, memorial, 1, Some(Target::Permanent(bears)));
    assert!(in_hand(&g, 0, bears) && in_graveyard(&g, 0, memorial));

    // Minas Morgul: a shadow counter and the Wraith type.
    let mut g = main_phase();
    let morgul = g.add_card_to_battlefield(0, catalog::minas_morgul_dark_fortress());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, morgul, 1, Some(Target::Permanent(bears)));
    assert!(has_keyword(&g, bears, Keyword::Shadow));
    assert!(g.computed_permanent(bears).unwrap().subtypes().creature_types.contains(&CreatureType::Wraith));

    // The Black Gate: unblockable while an opponent has the most life.
    let mut g = main_phase();
    let gate = g.add_card_to_battlefield(0, catalog::the_black_gate());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, gate, 1, Some(Target::Permanent(bears)));
    assert!(has_keyword(&g, bears, Keyword::Unblockable));
}

/// The Dimir mana lands' conditional abilities.
#[test]
fn cmdr_teval_dimir_mana_lands() {
    let tap = |g: &mut GameState, id: CardId, index: usize| {
        g.battlefield_find_mut(id).unwrap().tapped = false;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: index,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    // River of Tears: {U} before a land drop, {B} after.
    let mut g = main_phase();
    let river = g.add_card_to_battlefield(0, catalog::river_of_tears());
    g.players[0].lands_played_this_turn = 0;
    assert!(tap(&mut g, river, 1).is_err(), "no land played yet");
    tap(&mut g, river, 0).expect("{U}");
    g.players[0].lands_played_this_turn = 1;
    assert!(tap(&mut g, river, 0).is_err(), "played a land: only {{B}}");
    tap(&mut g, river, 1).expect("{B}");

    // Hidden Lair: colored only with a basic (or the turn it entered).
    let mut g = main_phase();
    let lair = g.add_card_to_battlefield(0, catalog::hidden_lair());
    g.battlefield_find_mut(lair).unwrap().entered_turn = None;
    assert!(tap(&mut g, lair, 1).is_err(), "no basic land");
    g.add_card_to_battlefield(0, catalog::swamp());
    tap(&mut g, lair, 1).expect("with a basic");

    // Sunken Ruins: {U/B}, {T} → two mana of the pair. One filter ability,
    // not three, so the payout is the decider's call and the count is what
    // the card promises.
    let mut g = main_phase();
    let ruins = g.add_card_to_battlefield(0, catalog::sunken_ruins());
    g.players[0].mana_pool.add(Color::Blue, 1);
    tap(&mut g, ruins, 1).expect("filter");
    let pool = &g.players[0].mana_pool;
    assert_eq!(pool.amount(Color::Blue) + pool.amount(Color::Black), 2);
}

// ── Wretched Ranks (FDC, Ghoulcaller Gisa) ─────────────────────────────────

fn zombies(g: &GameState, seat: usize) -> Vec<CardId> {
    g.battlefield
        .iter()
        .filter(|c| c.controller == seat && c.is_token && c.definition.name == "Zombie")
        .map(|c| c.id)
        .collect()
}

/// Army of the Damned and Necrotic Hex make their Zombies tapped; the Hex
/// takes six creatures from every player, the caster included.
#[test]
fn army_of_the_damned_and_necrotic_hex_make_tapped_zombies() {
    let mut g = main_phase();
    let army = g.add_card_to_hand(0, catalog::army_of_the_damned());
    cast_with(&mut g, army, None, None);
    let z = zombies(&g, 0);
    assert_eq!(z.len(), 13);
    assert!(z.iter().all(|&id| g.battlefield_find(id).unwrap().tapped));

    let mut g = main_phase();
    for _ in 0..8 {
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
    }
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hex = g.add_card_to_hand(0, catalog::necrotic_hex());
    cast_with(&mut g, hex, None, None);
    assert_eq!(count_named(&g, 1, "Grizzly Bears"), 2, "eight less six");
    assert_eq!(count_named(&g, 0, "Grizzly Bears"), 0, "the caster sacrifices too");
    let z = zombies(&g, 0);
    assert_eq!(z.len(), 6);
    assert!(z.iter().all(|&id| g.battlefield_find(id).unwrap().tapped));
}

/// Endless Ranks of the Dead adds half your Zombies, rounded down, each upkeep.
#[test]
fn endless_ranks_of_the_dead_adds_half_rounded_down() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::endless_ranks_of_the_dead());
    for _ in 0..5 {
        g.add_card_to_battlefield(0, catalog::razorlash_transmogrant());
    }
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(zombies(&g, 0).len(), 2, "5 / 2 = 2");
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(zombies(&g, 0).len(), 5, "7 / 2 = 3 more");
}

/// Infernal Idol taps for {B} and cashes in for two cards and 2 life.
#[test]
fn infernal_idol_draws_two_for_two_life() {
    let mut g = main_phase();
    let idol = g.add_card_to_battlefield(0, catalog::infernal_idol());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::swamp());
    }
    let (hand, life) = (g.players[0].hand.len(), g.players[0].life);
    activate(&mut g, idol, 1, None);
    assert_eq!(g.players[0].hand.len(), hand + 2);
    assert_eq!(g.players[0].life, life - 2);
    assert!(!on_battlefield(&g, idol), "sacrificed");
}

/// Josu Vess brings eight menacing Zombie Knights only when kicked.
#[test]
fn josu_vess_makes_knights_only_when_kicked() {
    let knights = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Zombie Knight").count();
    let mut g = main_phase();
    let josu = g.add_card_to_hand(0, catalog::josu_vess_lich_knight());
    cast_with(&mut g, josu, None, None);
    assert_eq!(knights(&g), 0);

    let mut g = main_phase();
    let josu = g.add_card_to_hand(0, catalog::josu_vess_lich_knight());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: josu,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("kicked");
    drain_stack(&mut g);
    assert_eq!(knights(&g), 8);
    let k = g.battlefield.iter().find(|c| c.definition.name == "Zombie Knight").unwrap().id;
    assert!(g.computed_permanent(k).unwrap().keywords().contains(&Keyword::Menace));
}

/// Liliana's Reaver: a connecting hit makes the player discard and gives
/// you a tapped Zombie.
#[test]
fn lilianas_reaver_discards_and_makes_a_zombie() {
    let mut g = main_phase();
    let reaver = g.add_card_to_battlefield(0, catalog::lilianas_reaver());
    g.add_card_to_hand(1, catalog::swamp());
    swing(&mut g, reaver);
    assert!(g.players[1].hand.is_empty(), "discarded");
    let z = zombies(&g, 0);
    assert_eq!(z.len(), 1);
    assert!(g.battlefield_find(z[0]).unwrap().tapped);
}

/// Open the Graves answers a nontoken creature's death, not a token's.
#[test]
fn open_the_graves_counts_only_nontoken_deaths() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::open_the_graves());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_with(&mut g, bolt, Some(Target::Permanent(bears)), None);
    let z = zombies(&g, 0);
    assert_eq!(z.len(), 1);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_with(&mut g, bolt, Some(Target::Permanent(z[0])), None);
    assert!(zombies(&g, 0).is_empty(), "a token's death makes nothing");
}

/// CR 102.2 — Razorlash Transmogrant returns from the graveyard with a
/// +1/+1 counter; the ability costs {4} less only while one opponent has four
/// nonbasic lands.
#[test]
fn cr_102_2_razorlash_transmogrant_discounts_against_nonbasics() {
    let pay_bb = |g: &mut GameState, id| {
        g.players[0].mana_pool.empty();
        g.players[0].mana_pool.add(Color::Black, 2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    let mut g = main_phase();
    let razor = g.add_card_to_graveyard(0, catalog::razorlash_transmogrant());
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::tundra());
    }
    assert!(pay_bb(&mut g, razor).is_err(), "three nonbasics: full price");
    g.add_card_to_battlefield(1, catalog::tundra());
    pay_bb(&mut g, razor).expect("four: {B}{B}");
    drain_stack(&mut g);
    let back = g.battlefield_find(razor).expect("returned");
    assert_eq!(back.counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.computed_permanent(razor).unwrap().keywords().contains(&Keyword::CantBlock));
}

/// Syphon Flesh: a Zombie for each creature actually sacrificed.
#[test]
fn syphon_flesh_counts_what_was_sacrificed() {
    let mut g = multi_player_game(4);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::syphon_flesh());
    cast_with(&mut g, spell, None, None);
    assert_eq!(zombies(&g, 0).len(), 2, "seat 3 had nothing to sacrifice");
    assert!(on_battlefield(&g, mine), "each OTHER player");
}
