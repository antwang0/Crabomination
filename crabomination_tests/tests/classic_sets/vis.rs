//! Visions (VIS) — `catalog::sets::vis`.

use crabomination::card::{CardDefinition, CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn ready(g: &mut GameState, seat: usize, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
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

/// The Karoo cycle enters tapped and costs an untapped basic of its colour.
#[test]
fn karoo_lands_cost_an_untapped_basic() {
    for (land, basic) in [
        (catalog::karoo_land as fn() -> CardDefinition, catalog::plains as fn() -> CardDefinition),
        (catalog::coral_atoll, catalog::island),
        (catalog::everglades, catalog::swamp),
        (catalog::dormant_volcano, catalog::mountain),
        (catalog::jungle_basin, catalog::forest),
    ] {
        let name = land().name;
        let mut g = two_player_game();
        let basic_id = ready(&mut g, 0, basic());
        let id = g.add_card_to_battlefield(0, land());
        let etb = land().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_ability(id, 0, None);
        g.resolve_effect(&etb, &ctx).expect("etb");
        drain_stack(&mut g);
        assert!(g.battlefield_find(id).is_some(), "{name} stayed");
        assert!(g.battlefield_find(basic_id).is_none(), "{name} bounced its basic");
        assert!(g.players[0].hand.iter().any(|c| c.id == basic_id));
    }
}

/// With no untapped basic to return, the Karoo land is sacrificed.
#[test]
fn karoo_land_is_sacrificed_without_a_basic() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::karoo_land());
    let etb = catalog::karoo_land().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_ability(id, 0, None);
    g.resolve_effect(&etb, &ctx).expect("etb");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none());
}

/// Lichenthrope takes damage as -1/-1 counters and heals one each upkeep.
#[test]
fn lichenthrope_takes_damage_as_counters() {
    let mut g = two_player_game();
    let lichen = ready(&mut g, 1, catalog::lichenthrope());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(lichen))).expect("bolt");
    drain_stack(&mut g);
    let c = g.battlefield_find(lichen).expect("alive");
    assert_eq!(c.damage, 0, "no marked damage");
    assert_eq!(c.counter_count(CounterType::MinusOneMinusOne), 3);
    let cp = g.computed_permanent(lichen).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
}

/// Tar Pit Warrior dies to being targeted at all.
#[test]
fn tar_pit_warrior_dies_to_any_targeting() {
    let mut g = two_player_game();
    let warrior = ready(&mut g, 1, catalog::tar_pit_warrior());
    let clasp = g.add_card_to_hand(0, catalog::sun_clasp());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, clasp, Some(Target::Permanent(warrior))).expect("enchant it");
    drain_stack(&mut g);
    assert!(g.battlefield_find(warrior).is_none());
}

/// Goblin Swine-Rider blows up the whole combat when it's blocked.
#[test]
fn goblin_swine_rider_sweeps_the_combat() {
    let mut g = two_player_game();
    let rider = ready(&mut g, 0, catalog::goblin_swine_rider());
    let partner = ready(&mut g, 0, catalog::infantry_veteran());
    let blocker = ready(&mut g, 1, catalog::jamuraan_lion());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: rider, target: AttackTarget::Player(1) },
        Attack { attacker: partner, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, rider)])).expect("block");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rider).is_none());
    assert!(g.battlefield_find(partner).is_none(), "the other attacker too");
    assert!(g.battlefield_find(blocker).is_none());
}

/// Crypt Rats scales its sweep with black mana.
#[test]
fn crypt_rats_sweeps_for_x() {
    let mut g = two_player_game();
    let rats = ready(&mut g, 0, catalog::crypt_rats());
    let victim = ready(&mut g, 1, catalog::archangel());
    g.players[0].mana_pool.add(Color::Black, 3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: rats,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("sweep");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rats).is_none(), "it kills itself too");
    assert_eq!(g.battlefield_find(victim).unwrap().damage, 3);
    assert_eq!(g.players[0].life, 17);
    assert_eq!(g.players[1].life, 17);
}

/// Helm of Awakening discounts every spell on the table by {1}.
#[test]
fn helm_of_awakening_discounts_everyone() {
    let mut g = two_player_game();
    ready(&mut g, 0, catalog::helm_of_awakening());
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{1}{G} minus {1} is {G}");
}

/// Aku Djinn grows the opposition, not your own board.
#[test]
fn aku_djinn_feeds_only_the_opposition() {
    let mut g = two_player_game();
    let djinn = ready(&mut g, 0, catalog::aku_djinn());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let upkeep = catalog::aku_djinn().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_ability(djinn, 0, None);
    g.resolve_effect(&upkeep, &ctx).expect("upkeep");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(theirs).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Blanket of Night makes every land a Swamp — including the opponent's.
#[test]
fn blanket_of_night_swamps_the_table() {
    let mut g = two_player_game();
    ready(&mut g, 0, catalog::blanket_of_night());
    let forest = ready(&mut g, 1, catalog::forest());
    let cp = g.computed_permanent(forest).unwrap();
    assert!(cp.subtypes.land_types.contains(&crabomination::card::LandType::Swamp));
    assert!(cp.subtypes.land_types.contains(&crabomination::card::LandType::Forest), "still a Forest");
}

/// Tremor spares the fliers.
#[test]
fn tremor_spares_fliers() {
    let mut g = two_player_game();
    let grounded = ready(&mut g, 1, catalog::infantry_veteran());
    let flier = ready(&mut g, 1, catalog::freewind_falcon());
    let tremor = g.add_card_to_hand(0, catalog::tremor());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, tremor, None).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(grounded).is_none());
    assert!(g.battlefield_find(flier).is_some());
}

/// Retribution of the Meek leaves the small creatures alone.
#[test]
fn retribution_of_the_meek_kills_only_the_big() {
    let mut g = two_player_game();
    let big = ready(&mut g, 1, catalog::archangel());
    let small = ready(&mut g, 1, catalog::jamuraan_lion());
    let sweep = g.add_card_to_hand(0, catalog::retribution_of_the_meek());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, sweep, None).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_none());
    assert!(g.battlefield_find(small).is_some(), "power 3");
}

/// Teferi's Honor Guard blinks itself out of a removal spell.
#[test]
fn teferis_honor_guard_phases_itself_out() {
    let mut g = two_player_game();
    let guard = ready(&mut g, 0, catalog::teferis_honor_guard());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, guard, 0, None).expect("phase out");
    drain_stack(&mut g);
    assert!(g.battlefield_find(guard).is_none());
    assert!(g.phased_out.iter().any(|c| c.id == guard));
}

/// Army Ants trades a land for a land.
#[test]
fn army_ants_trades_lands() {
    let mut g = two_player_game();
    let ants = ready(&mut g, 0, catalog::army_ants());
    let mine = ready(&mut g, 0, catalog::forest());
    let theirs = ready(&mut g, 1, catalog::island());
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, ants, 0, Some(Target::Permanent(theirs))).expect("trade");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none(), "the cost");
    assert!(g.battlefield_find(theirs).is_none());
}

/// Summer Bloom buys three extra land drops.
#[test]
fn summer_bloom_grants_three_land_drops() {
    let mut g = two_player_game();
    let bloom = g.add_card_to_hand(0, catalog::summer_bloom());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let lands: Vec<CardId> = (0..4).map(|_| g.add_card_to_hand(0, catalog::forest())).collect();
    cast(&mut g, bloom, None).expect("cast");
    drain_stack(&mut g);
    for land in &lands {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::PlayLand(*land)).expect("land drop");
    }
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_land()).count(), 4);
}

/// A `..Default::default()` sanity pass over the wave's simpler bodies.
#[test]
fn vis_stat_lines_match_print() {
    for (def, p, t) in [
        (catalog::archangel(), 5, 5),
        (catalog::tempest_drake(), 2, 2),
        (catalog::freewind_falcon(), 1, 1),
        (catalog::fallen_askari(), 2, 2),
        (catalog::suqata_lancer(), 2, 2),
        (catalog::king_cheetah(), 3, 2),
        (catalog::jamuraan_lion(), 3, 1),
        (catalog::daraja_griffin(), 2, 2),
        (catalog::wake_of_vultures(), 3, 1),
        (catalog::stampeding_wildebeests(), 5, 4),
        (catalog::aku_djinn(), 5, 6),
        (catalog::bull_elephant(), 4, 4),
        (catalog::firestorm_hellkite(), 6, 6),
        (catalog::lichenthrope(), 5, 5),
        (catalog::talruum_champion(), 3, 3),
        (catalog::zhalfirin_crusader(), 2, 2),
        (catalog::shimmering_efreet(), 2, 2),
    ] {
        assert_eq!((def.power, def.toughness), (p, t), "{}", def.name);
    }
    assert!(catalog::fallen_askari().keywords.contains(&Keyword::CantBlock));
    assert!(catalog::shimmering_efreet().keywords.contains(&Keyword::Phasing));
}
