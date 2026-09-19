//! Commander: the Teval, the Balanced Scale batch (`decks::cmdr_teval`).

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
    let id = g.add_card_to_battlefield(0, def);
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

    // Not on an opponent's turn.
    let other = g.add_card_to_graveyard(0, catalog::llanowar_elves());
    let fodder: Vec<CardId> =
        (0..3).map(|_| g.add_card_to_graveyard(0, catalog::forest())).collect();
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

/// Steward of the Harvest exiles up to three lands and wields their abilities.
#[test]
fn steward_of_the_harvest_borrows_exiled_land_abilities() {
    let mut g = main_phase();
    let drownyard = g.add_card_to_graveyard(0, catalog::nephalia_drownyard());
    let steward = etb(&mut g, catalog::steward_of_the_harvest());
    assert!(in_exile(&g, drownyard));
    let granted = g.granted_abilities_for(steward);
    assert_eq!(granted.len(), 2, "{{T}}: Add {{C}} and the mill ability");
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

/// Welcome the Dead counts the discard and anything else binned this turn.
#[test]
fn welcome_the_dead_makes_zombies_for_cards_binned_this_turn() {
    let mut g = main_phase();
    stock_libraries(&mut g, 5);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].cards_to_graveyard_this_turn = 2;
    let spell = g.add_card_to_hand(0, catalog::welcome_the_dead());
    cast_with(&mut g, spell, None, None);
    assert_eq!(g.players[0].life, 18);
    let zombies: Vec<_> =
        g.battlefield.iter().filter(|c| c.definition.name == "Zombie Druid").collect();
    assert_eq!(zombies.len(), 3, "two earlier plus the discard");
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
    assert!(g.computed_permanent(back).unwrap().subtypes().creature_types.contains(&CreatureType::Zombie));
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
    let none: Setup = |_| {};
    let cases: Vec<(fn() -> CardDefinition, Setup, bool)> = vec![
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
    for factory in [catalog::the_black_gate as fn() -> CardDefinition] {
        let mut g = main_phase();
        let id = play(&mut g, factory());
        assert!(!g.battlefield_find(id).unwrap().tapped);
        assert_eq!(g.players[0].life, 17);
    }
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

    // Witch's Clinic: lifelink on a legendary creature.
    let mut g = main_phase();
    let clinic = g.add_card_to_battlefield(0, catalog::witchs_clinic());
    let konrad = g.add_card_to_battlefield(0, catalog::syr_konrad_the_grim());
    activate(&mut g, clinic, 1, Some(Target::Permanent(konrad)));
    assert!(has_keyword(&g, konrad, Keyword::Lifelink));

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

    // Sunken Ruins: {U/B}, {T} → two mana.
    let mut g = main_phase();
    let ruins = g.add_card_to_battlefield(0, catalog::sunken_ruins());
    g.players[0].mana_pool.add(Color::Blue, 1);
    tap(&mut g, ruins, 3).expect("filter");
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 2);
}
