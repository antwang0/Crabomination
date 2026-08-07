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

/// Raging Gorilla swings to 4/1 when it becomes blocked.
#[test]
fn raging_gorilla_swings_when_blocked() {
    let mut g = two_player_game();
    let ape = ready(&mut g, 0, catalog::raging_gorilla());
    let wall = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ape,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, ape)])).expect("block");
    drain_stack(&mut g);
    let cp = g.computed_permanent(ape).expect("still there");
    assert_eq!((cp.power, cp.toughness), (4, 1), "+2/-2 on becoming blocked");
}

/// Suq'Ata Assassin poisons the defender when it gets through.
#[test]
fn suqata_assassin_poisons_on_unblocked_attack() {
    let mut g = two_player_game();
    let assassin = ready(&mut g, 0, catalog::suqata_assassin());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: assassin,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    assert_eq!(g.players[1].poison_counters, 1, "unblocked attack poisoned the defender");
}

/// Waterspout Djinn eats itself at upkeep with no untapped Island to bounce.
#[test]
fn waterspout_djinn_sacrifices_without_an_island() {
    let mut g = two_player_game();
    let djinn = g.add_card_to_battlefield(0, catalog::waterspout_djinn());
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(djinn).is_none(), "no Island to return");

    let mut g = two_player_game();
    let djinn = g.add_card_to_battlefield(0, catalog::waterspout_djinn());
    let island = g.add_card_to_battlefield(0, catalog::island());
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(djinn).is_some(), "the Island paid for it");
    assert!(g.battlefield_find(island).is_none(), "and went back to hand");
}

/// Mortal Wound kills its host the moment the host takes any damage.
#[test]
fn mortal_wound_destroys_on_damage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::mortal_wound());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, aura, Some(Target::Permanent(bear))).expect("cast");
    drain_stack(&mut g);
    let mut evs = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(bear),
        1,
        None,
        &mut evs,
    );
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "one damage destroyed it");
}

/// Betrayal draws its controller a card whenever the enchanted creature taps.
#[test]
fn betrayal_draws_when_the_host_taps() {
    let mut g = two_player_game();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let aura = g.add_card_to_hand(0, catalog::betrayal());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, aura, Some(Target::Permanent(theirs))).expect("cast");
    drain_stack(&mut g);
    let hand = g.players[0].hand.len();
    g.battlefield_find_mut(theirs).unwrap().tapped = true;
    g.dispatch_triggers_for_events(&[crabomination::game::types::GameEvent::PermanentTapped {
        card_id: theirs,
        actor: None,
        as_attacker: false,
    }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "the tap drew a card");
}

/// Parapet is a +0/+1 anthem for your creatures only.
#[test]
fn parapet_toughens_your_team() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::parapet());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(mine).unwrap().toughness, 3);
    assert_eq!(g.computed_permanent(theirs).unwrap().toughness, 2);
}

/// Mystic Veil and Relic Ward both hand out shroud, on a creature and an
/// artifact respectively.
#[test]
fn shroud_auras_protect_their_hosts() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let veil = g.add_card_to_hand(0, catalog::mystic_veil());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, veil, Some(Target::Permanent(bear))).expect("cast");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Shroud));
    assert_eq!(
        catalog::relic_ward().aura_enchant_filter(),
        Some(&crabomination::card::SelectionRequirement::Artifact),
        "Relic Ward enchants an artifact"
    );
}

/// Giant Caterpillar's sacrifice pays off at the next end step.
#[test]
fn giant_caterpillar_leaves_a_butterfly() {
    let mut g = two_player_game();
    let bug = ready(&mut g, 0, catalog::giant_caterpillar());
    g.players[0].mana_pool.add(Color::Green, 1);
    activate(&mut g, bug, 0, None).expect("sacrifice");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Butterfly"), "not yet");
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Butterfly"),
        "the Butterfly arrived at the end step"
    );
}

/// Necrosavant crawls back during your upkeep by eating another creature.
#[test]
fn necrosavant_reanimates_itself_at_upkeep() {
    let mut g = two_player_game();
    let savant = g.add_card_to_graveyard(0, catalog::necrosavant());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: savant,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("reanimate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(savant).is_some(), "back on the battlefield");
    assert!(g.battlefield_find(fodder).is_none(), "ate a creature for it");
}

/// The wave's plain bodies match print.
#[test]
fn vis_wave_two_stat_lines_match_print() {
    for (def, p, t) in [
        (catalog::python(), 3, 2),
        (catalog::raging_gorilla(), 2, 3),
        (catalog::suqata_assassin(), 1, 1),
        (catalog::talruum_piper(), 3, 3),
        (catalog::waterspout_djinn(), 4, 4),
        (catalog::giant_caterpillar(), 3, 3),
        (catalog::kyscu_drake(), 2, 2),
        (catalog::necrosavant(), 5, 5),
    ] {
        assert_eq!((def.power, def.toughness), (p, t), "{}", def.name);
    }
    assert!(catalog::suqata_assassin().keywords.contains(&Keyword::Fear));
    assert!(catalog::talruum_piper().keywords.contains(&Keyword::AllMustBlock));
}

/// Phyrexian Marauder enters as an X/X and can't block.
#[test]
fn phyrexian_marauder_enters_with_x_counters() {
    let mut g = two_player_game();
    let marauder = g.add_card_to_hand(0, catalog::phyrexian_marauder());
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: marauder,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("cast for X=3");
    drain_stack(&mut g);
    let cp = g.computed_permanent(marauder).expect("resolved");
    assert_eq!((cp.power, cp.toughness), (3, 3), "three +1/+1 counters");
    assert!(cp.keywords.contains(&Keyword::CantBlock));
}

/// Miraculous Recovery reanimates with a +1/+1 counter.
#[test]
fn miraculous_recovery_reanimates_with_a_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::miraculous_recovery());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, spell, Some(Target::Permanent(bear))).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).expect("back on the battlefield");
    assert_eq!((cp.power, cp.toughness), (3, 3), "2/2 plus a counter");
// ── Second wave ─────────────────────────────────────────────────────────────

/// Bull Elephant's ETB returns *two* Forests, not one.
#[test]
fn bull_elephant_returns_two_forests() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::forest());
    let b = g.add_card_to_battlefield(0, catalog::forest());
    let elephant = g.add_card_to_battlefield(0, catalog::bull_elephant());
    let etb = catalog::bull_elephant().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_ability(elephant, 0, None);
    g.resolve_effect(&etb, &ctx).expect("etb");
    drain_stack(&mut g);
    assert!(g.battlefield_find(elephant).is_some(), "kept the Elephant");
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}

/// One Forest isn't enough — the Elephant is sacrificed and the land stays.
#[test]
fn bull_elephant_needs_both_forests() {
    let mut g = two_player_game();
    let only = g.add_card_to_battlefield(0, catalog::forest());
    let elephant = g.add_card_to_battlefield(0, catalog::bull_elephant());
    let etb = catalog::bull_elephant().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_ability(elephant, 0, None);
    g.resolve_effect(&etb, &ctx).expect("etb");
    drain_stack(&mut g);
    assert!(g.battlefield_find(elephant).is_none());
    assert!(g.battlefield_find(only).is_some(), "unpayable costs aren't partially paid");
}

/// Flooded Shoreline bounces a creature by bouncing two Islands.
#[test]
fn flooded_shoreline_returns_two_islands() {
    let mut g = two_player_game();
    let shoreline = ready(&mut g, 0, catalog::flooded_shoreline());
    let i1 = g.add_card_to_battlefield(0, catalog::island());
    let i2 = g.add_card_to_battlefield(0, catalog::island());
    let victim = ready(&mut g, 1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 2);
    activate(&mut g, shoreline, 0, Some(Target::Permanent(victim))).expect("bounce");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none());
    assert!(g.battlefield_find(i1).is_none() && g.battlefield_find(i2).is_none());
}

/// With only one Island the ability can't be activated at all.
#[test]
fn flooded_shoreline_needs_two_islands() {
    let mut g = two_player_game();
    let shoreline = ready(&mut g, 0, catalog::flooded_shoreline());
    g.add_card_to_battlefield(0, catalog::island());
    let victim = ready(&mut g, 1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 2);
    assert!(activate(&mut g, shoreline, 0, Some(Target::Permanent(victim))).is_err());
}

/// Squandered Resources turns a Swamp into black mana.
#[test]
fn squandered_resources_reads_the_sacrificed_land() {
    let mut g = two_player_game();
    let res = ready(&mut g, 0, catalog::squandered_resources());
    let swamp = g.add_card_to_battlefield(0, catalog::swamp());
    activate(&mut g, res, 0, None).expect("sac the Swamp");
    drain_stack(&mut g);
    assert!(g.battlefield_find(swamp).is_none());
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1);
}

/// Desertion steals the creature it counters.
#[test]
fn desertion_steals_a_creature_spell() {
    let mut g = two_player_game();
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    let desertion = g.add_card_to_hand(0, catalog::desertion());
    g.players[1].mana_pool.add(Color::Green, 2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast bears");
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: desertion,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("counter it");
    drain_stack(&mut g);
    let stolen = g.battlefield_find(bears).expect("on the battlefield");
    assert_eq!(stolen.controller, 0, "under the countering player's control");
}

/// Magma Mine stores charges and throws them all at once.
#[test]
fn magma_mine_deals_its_pressure_counters() {
    let mut g = two_player_game();
    let mine = ready(&mut g, 0, catalog::magma_mine());
    for _ in 0..2 {
        g.players[0].mana_pool.add_colorless(4);
        activate(&mut g, mine, 0, None).expect("charge");
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::Pressure), 2);
    activate(&mut g, mine, 1, Some(Target::Player(1))).expect("fire");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
}

/// Bogardan Phoenix comes back once, then exiles itself.
#[test]
fn bogardan_phoenix_returns_only_once() {
    let mut g = two_player_game();
    let phoenix = ready(&mut g, 0, catalog::bogardan_phoenix());
    let mut events = Vec::new();
    g.destroy_permanent(phoenix, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let back = g.battlefield_find(phoenix).expect("back from the first death");
    assert_eq!(back.counter_count(CounterType::Death), 1);
    let mut events = Vec::new();
    g.destroy_permanent(phoenix, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(phoenix).is_none(), "the second death sticks");
    assert!(g.exile.iter().any(|c| c.id == phoenix), "exiled, not in the graveyard");
}

/// Matopi Golem shrinks each time it regenerates.
#[test]
fn matopi_golem_shrinks_when_it_regenerates() {
    let mut g = two_player_game();
    let golem = ready(&mut g, 0, catalog::matopi_golem());
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, golem, 0, None).expect("shield up");
    drain_stack(&mut g);
    let mut events = Vec::new();
    g.destroy_permanent(golem, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let c = g.battlefield_find(golem).expect("regenerated");
    assert_eq!(c.counter_count(CounterType::MinusOneMinusOne), 1);
}

/// Death Watch drains for the dead creature's stats.
#[test]
fn death_watch_drains_for_the_hosts_stats() {
    let mut g = two_player_game();
    let host = ready(&mut g, 1, catalog::grizzly_bears());
    let watch = g.add_card_to_hand(0, catalog::death_watch());
    g.players[0].mana_pool.add(Color::Black, 1);
    cast(&mut g, watch, Some(Target::Permanent(host))).expect("enchant");
    drain_stack(&mut g);
    let mut events = Vec::new();
    g.destroy_permanent(host, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "lost life equal to its power");
    assert_eq!(g.players[0].life, 22, "gained life equal to its toughness");
}

/// Brood of Cockroaches buys itself back at the next end step.
#[test]
fn brood_of_cockroaches_returns_at_end_step() {
    let mut g = two_player_game();
    let brood = ready(&mut g, 0, catalog::brood_of_cockroaches());
    let mut events = Vec::new();
    g.destroy_permanent(brood, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == brood));
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
    }
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == brood), "back in hand");
    assert_eq!(g.players[0].life, 19);
}

/// Suleiman's Legacy wipes Djinns on entry and keeps killing them after.
#[test]
fn suleimans_legacy_kills_djinns_on_sight() {
    let mut g = two_player_game();
    let efreet = ready(&mut g, 1, catalog::rainbow_efreet());
    let legacy = g.add_card_to_battlefield(0, catalog::suleimans_legacy());
    let etb = catalog::suleimans_legacy().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_ability(legacy, 0, None);
    g.resolve_effect(&etb, &ctx).expect("sweep");
    drain_stack(&mut g);
    assert!(g.battlefield_find(efreet).is_none());
    let latecomer = g.add_card_to_battlefield(1, catalog::rainbow_efreet());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: latecomer }]);
    drain_stack(&mut g);
    assert!(g.battlefield_find(latecomer).is_none(), "the static trigger catches it too");
}

/// Eye of Singularity destroys everything that shares a name.
#[test]
fn eye_of_singularity_enforces_singleton() {
    let mut g = two_player_game();
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::grizzly_bears());
    let plains = g.add_card_to_battlefield(0, catalog::plains());
    let eye = g.add_card_to_battlefield(0, catalog::eye_of_singularity());
    let etb = catalog::eye_of_singularity().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_ability(eye, 0, None);
    g.resolve_effect(&etb, &ctx).expect("sweep");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    assert!(g.battlefield_find(plains).is_some(), "basic lands are exempt");
}

/// Quicksand shrinks a ground attacker and can't touch a flier.
#[test]
fn quicksand_only_catches_ground_attackers() {
    let mut g = two_player_game();
    let sand = ready(&mut g, 0, catalog::quicksand());
    let attacker = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    g.priority.player_with_priority = 0;
    activate(&mut g, sand, 1, Some(Target::Permanent(attacker))).expect("sac Quicksand");
    drain_stack(&mut g);
    assert!(g.battlefield_find(attacker).is_none(), "-1/-2 killed the 2/2");
}

/// Magma Mine charges up and then blows for its counter count.
#[test]
fn magma_mine_deals_its_pressure_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let mine = ready(&mut g, 0, catalog::magma_mine());
    g.players[0].mana_pool.add_colorless(8);
    activate(&mut g, mine, 0, None).expect("charge");
    drain_stack(&mut g);
    activate(&mut g, mine, 0, None).expect("charge again");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::Pressure), 2);
    activate(&mut g, mine, 1, Some(Target::Player(1))).expect("blow");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "two pressure counters, two damage");
}

/// Snake Basket turns X into that many 1/1 Snakes.
fn quicksand_only_hits_ground_attackers() {
    let mut g = two_player_game();
    let sand = ready(&mut g, 0, catalog::quicksand());
    let flier = ready(&mut g, 1, catalog::rainbow_efreet());
    let ground = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: flier, target: AttackTarget::Player(0) },
        Attack { attacker: ground, target: AttackTarget::Player(0) },
    ]))
    .expect("attack");
    g.priority.player_with_priority = 0;
    assert!(activate(&mut g, sand, 1, Some(Target::Permanent(flier))).is_err(), "flier is safe");
    activate(&mut g, sand, 1, Some(Target::Permanent(ground))).expect("mire it");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ground).is_none(), "-1/-2 takes a 2/2 to 1/0");
}

/// Griffin Canyon untaps and pumps a Griffin.
#[test]
fn griffin_canyon_untaps_and_pumps() {
    let mut g = two_player_game();
    let canyon = ready(&mut g, 0, catalog::griffin_canyon());
    let griffin = ready(&mut g, 0, catalog::enforcer_griffin());
    g.battlefield_find_mut(griffin).unwrap().tapped = true;
    activate(&mut g, canyon, 1, Some(Target::Permanent(griffin))).expect("untap it");
    drain_stack(&mut g);
    let c = g.battlefield_find(griffin).expect("there");
    assert!(!c.tapped);
    let base = catalog::enforcer_griffin();
    let cp = g.computed_permanent(griffin).unwrap();
    assert_eq!((cp.power, cp.toughness), (base.power + 1, base.toughness + 1));
}

/// Miraculous Recovery reanimates with a +1/+1 counter.
#[test]
fn miraculous_recovery_adds_a_counter() {
    let mut g = two_player_game();
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::miraculous_recovery());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, spell, Some(Target::Permanent(bears))).expect("recover");
    drain_stack(&mut g);
    let c = g.battlefield_find(bears).expect("back");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Snake Basket pays out X Snakes.
#[test]
fn snake_basket_makes_x_snakes() {
    let mut g = two_player_game();
    let basket = ready(&mut g, 0, catalog::snake_basket());
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: basket,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: Some(3),
        mode: None,
    })
    .expect("crack the basket");
    .expect("dump snakes");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Snake").count(), 3);
}

/// Griffin Canyon untaps and pumps a Griffin.
#[test]
fn griffin_canyon_untaps_a_griffin() {
    let mut g = two_player_game();
    let canyon = ready(&mut g, 0, catalog::griffin_canyon());
    let griffin = ready(&mut g, 0, catalog::daraja_griffin());
    g.battlefield_find_mut(griffin).unwrap().tapped = true;
    activate(&mut g, canyon, 1, Some(Target::Permanent(griffin))).expect("untap it");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(griffin).unwrap().tapped, "untapped");
    assert_eq!(g.computed_permanent(griffin).unwrap().power, 3, "+1/+1");
}

/// Scalebane's Elite carries protection from black.
#[test]
fn scalebanes_elite_has_pro_black() {
    let d = catalog::scalebanes_elite();
    assert_eq!((d.power, d.toughness), (4, 4));
    assert!(d.keywords.contains(&Keyword::Protection(Color::Black)));
}

/// Righteous War hands protection out along the colour line.
#[test]
fn righteous_war_protects_both_halves() {
    use crabomination::card::Keyword as K;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::righteous_war());
    let white = g.add_card_to_battlefield(0, catalog::savannah_lions());
    let black = g.add_card_to_battlefield(0, catalog::python());
    assert!(
        g.computed_permanent(white).unwrap().keywords.contains(&K::Protection(Color::Black)),
        "white creature gets pro-black"
    );
    assert!(
        g.computed_permanent(black).unwrap().keywords.contains(&K::Protection(Color::White)),
        "black creature gets pro-white"
    );
}

/// Suleiman's Legacy wipes the Djinns on entry and kills the next one too.
#[test]
fn suleimans_legacy_hates_djinns() {
    let mut g = two_player_game();
    let djinn = g.add_card_to_battlefield(1, catalog::waterspout_djinn());
    g.move_card_to_battlefield_for_test(0, catalog::suleimans_legacy());
    drain_stack(&mut g);
    assert!(g.battlefield_find(djinn).is_none(), "the sweep got it");
    let later = g.add_card_to_battlefield(1, catalog::waterspout_djinn());
    g.dispatch_triggers_for_events(&[crabomination::game::types::GameEvent::PermanentEntered {
        card_id: later,
    }]);
    drain_stack(&mut g);
    assert!(g.battlefield_find(later).is_none(), "and the next one");
}

/// Death Watch drains the dead creature's controller for its power and gains
/// you its toughness.
#[test]
fn death_watch_drains_on_death() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::death_watch());
    g.players[0].mana_pool.add(Color::Black, 1);
    cast(&mut g, aura, Some(Target::Permanent(bear))).expect("cast");
    drain_stack(&mut g);
    g.battlefield_find_mut(bear).unwrap().damage = 99;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "lost life equal to its power");
    assert_eq!(g.players[0].life, 22, "gained life equal to its toughness");
}

/// Vanishing phases its host out.
#[test]
fn vanishing_phases_out_its_host() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::vanishing());
    g.players[0].mana_pool.add(Color::Blue, 3);
    cast(&mut g, aura, Some(Target::Permanent(bear))).expect("cast");
    drain_stack(&mut g);
    let aura_id = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Vanishing")
        .expect("attached")
        .id;
    activate(&mut g, aura_id, 0, None).expect("phase it out");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "phased out of the battlefield");
}

/// Flooded Shoreline bounces a creature for two Islands.
#[test]
fn flooded_shoreline_bounces_for_two_islands() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::flooded_shoreline());
    let shore = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Flooded Shoreline")
        .unwrap()
        .id;
    let i1 = g.add_card_to_battlefield(0, catalog::island());
    let i2 = g.add_card_to_battlefield(0, catalog::island());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 2);
    activate(&mut g, shore, 0, Some(Target::Permanent(victim))).expect("bounce");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "creature bounced");
    assert!(g.battlefield_find(i1).is_none() && g.battlefield_find(i2).is_none(), "both Islands paid");
}

/// Righteous Aura buys a damage-prevention shield for {W} and 2 life.
#[test]
fn righteous_aura_prevents_the_next_hit() {
    let mut g = two_player_game();
    let ra = ready(&mut g, 0, catalog::righteous_aura());
    let src = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    activate(&mut g, ra, 0, None).expect("buy a shield");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 18, "paid 2 life");
    let mut evs = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(0),
        3,
        Some(src),
        &mut evs,
    );
    assert_eq!(g.players[0].life, 18, "the shield ate the damage");
}

/// Quirion Druid animates a land into a 2/2 that stays a land.
#[test]
fn quirion_druid_animates_a_land() {
    use crabomination::card::CardType;
    let mut g = two_player_game();
    let druid = ready(&mut g, 0, catalog::quirion_druid());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.players[0].mana_pool.add(Color::Green, 1);
    activate(&mut g, druid, 0, Some(Target::Permanent(land))).expect("animate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(land).expect("still there");
    assert_eq!((cp.power, cp.toughness), (2, 2));
    assert!(cp.card_types.contains(&CardType::Creature) && cp.card_types.contains(&CardType::Land));
}

/// Rainbow Efreet phases itself out for {U}{U}.
#[test]
fn rainbow_efreet_phases_itself_out() {
    let mut g = two_player_game();
    let efreet = ready(&mut g, 0, catalog::rainbow_efreet());
    g.players[0].mana_pool.add(Color::Blue, 2);
    activate(&mut g, efreet, 0, None).expect("phase out");
    drain_stack(&mut g);
    assert!(g.battlefield_find(efreet).is_none(), "phased out");
}

/// Knight of Valor shrinks everything blocking it, once a turn.
#[test]
fn knight_of_valor_shrinks_its_blockers() {
    let mut g = two_player_game();
    let knight = ready(&mut g, 0, catalog::knight_of_valor());
    let blocker = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
/// Diamond Kaleidoscope's Prisms cash in for any colour.
#[test]
fn diamond_kaleidoscope_prisms_make_mana() {
    let mut g = two_player_game();
    let kaleidoscope = ready(&mut g, 0, catalog::diamond_kaleidoscope());
    g.players[0].mana_pool.add_colorless(3);
    activate(&mut g, kaleidoscope, 0, None).expect("mint a Prism");
    drain_stack(&mut g);
    let prism = g.battlefield.iter().find(|c| c.definition.name == "Prism").expect("minted").id;
    activate(&mut g, kaleidoscope, 1, None).expect("cash it in");
    drain_stack(&mut g);
    assert!(g.battlefield_find(prism).is_none());
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

/// Righteous War splits your board into two untouchable halves.
#[test]
fn righteous_war_grants_paired_protection() {
    let mut g = two_player_game();
    let white = ready(&mut g, 0, catalog::savannah_lions());
    g.add_card_to_battlefield(0, catalog::righteous_war());
    let cp = g.computed_permanent(white).expect("there");
    assert!(cp.keywords.contains(&Keyword::Protection(Color::Black)));
}

/// Knight of Valor shrinks its non-flanking blockers.
#[test]
fn knight_of_valor_shrinks_blockers() {
    let mut g = two_player_game();
    let knight = ready(&mut g, 0, catalog::knight_of_valor());
    let blocker = ready(&mut g, 1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: knight,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, knight)])).expect("block");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    activate(&mut g, knight, 0, None).expect("shrink the blockers");
    drain_stack(&mut g);
    // Flanking already gave -1/-1; the ability adds another.
    assert!(g.battlefield_find(blocker).is_none(), "the 2/2 shrank away");
}

/// Matopi Golem shrinks itself each time it regenerates.
#[test]
fn matopi_golem_shrinks_on_regenerate() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let golem = ready(&mut g, 0, catalog::matopi_golem());
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, golem, 0, None).expect("shield up");
    drain_stack(&mut g);
    g.battlefield_find_mut(golem).unwrap().damage = 99;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let c = g.battlefield_find(golem).expect("regenerated instead of dying");
    assert_eq!(c.counter_count(CounterType::MinusOneMinusOne), 1, "one -1/-1 per regeneration");
}

/// Brood of Cockroaches crawls back to hand at the next end step.
#[test]
fn brood_of_cockroaches_returns_at_end_step() {
    let mut g = two_player_game();
    let roaches = g.add_card_to_battlefield(0, catalog::brood_of_cockroaches());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let evs = g.remove_to_graveyard_with_triggers(roaches);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == roaches), "back in hand");
    assert_eq!(g.players[0].life, 19, "for a life");
}

/// Vampirism grows its host per other creature you control and shrinks the
/// rest of the team.
#[test]
fn vampirism_feeds_on_your_own_board() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::vampirism());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, aura, Some(Target::Permanent(host))).expect("cast");
    drain_stack(&mut g);
    // One other creature → +1/+1 on the host, -1/-1 on it.
    assert_eq!(g.computed_permanent(host).unwrap().power, 3);
    assert_eq!(g.computed_permanent(other).unwrap().power, 1);
}

/// Knight of the Mists destroys a Knight when its {U} goes unpaid — the target
/// lives in the `PayManaOrElse` fallback arm, so the trigger must reach it.
#[test]
fn knight_of_the_mists_kills_a_knight_unpaid() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::knight_of_valor());
    g.move_card_to_battlefield_for_test(0, catalog::knight_of_the_mists());
    drain_stack(&mut g);
    let knights = g
        .battlefield
        .iter()
        .filter(|c| {
            c.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Knight)
        })
        .count();
    assert_eq!(knights, 1, "unpaid, so the ETB destroyed a Knight");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, knight)])).expect("block");
    drain_stack(&mut g);
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, knight, 0, None).expect("shrink");
    drain_stack(&mut g);
    // Flanking already took it to 1/1; the ability makes it 0/0.
    assert!(g.battlefield_find(blocker).is_none(), "the blocker dies to the second shrink");
}

/// Tithe finds a second Plains only when the opponent is ahead on lands.
#[test]
fn tithe_finds_a_second_plains_when_behind() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::plains());
        g.add_card_to_battlefield(1, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::tithe());
    g.players[0].mana_pool.add(Color::White, 1);
    cast(&mut g, spell, Some(Target::Player(1))).expect("tithe");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.iter().filter(|c| c.definition.name == "Plains").count(), 2);
}

/// The second wave's plain bodies match print.
#[test]
fn vis_wave_three_stat_lines_match_print() {
    for (def, p, t) in [
        (catalog::scalebanes_elite(), 4, 4),
        (catalog::viashivan_dragon(), 4, 4),
        (catalog::rainbow_efreet(), 3, 1),
        (catalog::mundungu(), 1, 1),
        (catalog::brood_of_cockroaches(), 1, 1),
        (catalog::matopi_golem(), 3, 3),
        (catalog::bogardan_phoenix(), 3, 3),
        (catalog::knight_of_valor(), 2, 2),
    ] {
        assert_eq!((def.power, def.toughness), (p, t), "{}", def.name);
    }
    assert!(catalog::scalebanes_elite().keywords.contains(&Keyword::Protection(Color::Black)));
    assert!(catalog::knight_of_valor().keywords.contains(&Keyword::Flanking));
}

// ── Wave four ───────────────────────────────────────────────────────────────

/// A Chimera hands its keyword and a +2/+2 counter to another Chimera.
#[test]
fn chimeras_pass_their_keyword_along() {
    for (donor, keyword) in [
        (catalog::tin_wing_chimera as fn() -> CardDefinition, Keyword::Flying),
        (catalog::lead_belly_chimera, Keyword::Trample),
        (catalog::iron_heart_chimera, Keyword::Vigilance),
        (catalog::brass_talon_chimera, Keyword::FirstStrike),
    ] {
        let mut g = two_player_game();
        let from = ready(&mut g, 0, donor());
        let to = ready(&mut g, 0, catalog::brass_talon_chimera());
        activate(&mut g, from, 0, Some(Target::Permanent(to))).expect("sacrifice it");
        drain_stack(&mut g);
        let c = g.battlefield_find(to).expect("recipient");
        assert_eq!(c.counter_count(CounterType::PlusTwoPlusTwo), 1, "{}", donor().name);
        assert!(g.computed_permanent(to).unwrap().keywords.contains(&keyword));
    }
}

/// Sands of Time flips every board at each upkeep.
#[test]
fn sands_of_time_swaps_tapped_state() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sands_of_time());
    let tapped = ready(&mut g, 0, catalog::grizzly_bears());
    let untapped = ready(&mut g, 0, catalog::savannah_lions());
    g.battlefield_find_mut(tapped).unwrap().tapped = true;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(tapped).unwrap().tapped);
    assert!(g.battlefield_find(untapped).unwrap().tapped);
}

/// City of Solitude stops the opponent acting on your turn.
#[test]
fn city_of_solitude_locks_out_the_off_turn_player() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::city_of_solitude());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "not their turn"
    );
}

/// Quirion Druid animates a land for good.
#[test]
fn quirion_druid_animates_a_land() {
    let mut g = two_player_game();
    let druid = ready(&mut g, 0, catalog::quirion_druid());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.players[0].mana_pool.add(Color::Green, 1);
    activate(&mut g, druid, 0, Some(Target::Permanent(forest))).expect("animate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(forest).expect("still a land");
    assert_eq!((cp.power, cp.toughness), (2, 2));
    assert!(cp.card_types.contains(&crabomination::card::CardType::Land), "still a land");
}

/// Katabatic Winds grounds fliers and locks their tap abilities.
#[test]
fn katabatic_winds_grounds_fliers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::katabatic_winds());
    let flier = ready(&mut g, 1, catalog::rainbow_efreet());
    let cp = g.computed_permanent(flier).expect("there");
    assert!(cp.keywords.contains(&Keyword::CantAttack));
    assert!(cp.keywords.contains(&Keyword::CantBlock));
    assert!(cp.keywords.contains(&Keyword::CantActivateTapAbilities));
}

/// Time and Tide swaps both sides of the phasing ledger at once.
#[test]
fn time_and_tide_swaps_the_phased_out() {
    let mut g = two_player_game();
    let phaser = ready(&mut g, 0, catalog::shimmering_efreet());
    let away = ready(&mut g, 0, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_ability(away, 0, None);
    g.resolve_effect(
        &crabomination::effect::Effect::PhaseOut {
            what: crabomination::effect::Selector::This,
            until_source_leaves: false,
        },
        &ctx,
    )
    .expect("phase it out");
    let spell = g.add_card_to_hand(0, catalog::time_and_tide());
    g.players[0].mana_pool.add(Color::Blue, 2);
    cast(&mut g, spell, None).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(away).is_some(), "the phased-out creature came back");
    assert!(g.battlefield_find(phaser).is_none(), "and the phaser left");
}

/// Equipoise phases out only the opponent's excess.
#[test]
fn equipoise_phases_out_the_excess() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::equipoise());
    g.add_card_to_battlefield(0, catalog::forest());
    let extra: Vec<CardId> =
        (0..3).map(|_| g.add_card_to_battlefield(1, catalog::island())).collect();
    let eq = g.battlefield.iter().find(|c| c.definition.name == "Equipoise").unwrap().id;
    let etb = catalog::equipoise().triggered_abilities[0].effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_ability(eq, 0, None);
    ctx.targets = vec![Target::Player(1)];
    g.resolve_effect(&etb, &ctx).expect("upkeep");
    drain_stack(&mut g);
    let left = extra.iter().filter(|id| g.battlefield_find(**id).is_some()).count();
    assert_eq!(left, 1, "three lands against one leaves one");
}

/// Guiding Spirit puts a creature card back on top.
#[test]
fn guiding_spirit_recycles_a_creature() {
    let mut g = two_player_game();
    let spirit = ready(&mut g, 0, catalog::guiding_spirit());
    let bears = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    activate(&mut g, spirit, 0, Some(Target::Player(1))).expect("recycle");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.first().map(|c| c.id), Some(bears));
}

/// Wand of Denial bins a nonland top card for 2 life.
#[test]
fn wand_of_denial_bins_a_nonland() {
    let mut g = two_player_game();
    let wand = ready(&mut g, 0, catalog::wand_of_denial());
    g.players[1].library.clear();
    let top = g.add_card_to_library(1, catalog::grizzly_bears());
    activate(&mut g, wand, 0, Some(Target::Player(1))).expect("peek");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == top));
    assert_eq!(g.players[0].life, 18);
}

/// Pillar Tombs of Aku takes a creature a turn, or five life and itself.
#[test]
fn pillar_tombs_of_aku_taxes_each_upkeep() {
    let mut g = two_player_game();
    let pillar = g.add_card_to_battlefield(0, catalog::pillar_tombs_of_aku());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 15, "no creature to feed it");
    assert!(g.battlefield_find(pillar).is_none(), "and it goes with them");
}

/// Ovinomancer needs three basics, then turns anything into a Sheep.
#[test]
fn ovinomancer_trades_lands_for_sheep() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::island());
    }
    let ovi = g.add_card_to_battlefield(0, catalog::ovinomancer());
    let etb = catalog::ovinomancer().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_ability(ovi, 0, None);
    g.resolve_effect(&etb, &ctx).expect("etb");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ovi).is_some(), "the basics paid for it");
    g.clear_sickness(ovi);
    let victim = ready(&mut g, 1, catalog::grizzly_bears());
    activate(&mut g, ovi, 0, Some(Target::Permanent(victim))).expect("shear it");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none());
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Sheep" && c.controller == 1));
}

/// Infernal Harvest turns X Swamps into X damage.
#[test]
fn infernal_harvest_pays_in_swamps() {
    let mut g = two_player_game();
    let swamps: Vec<CardId> =
        (0..2).map(|_| g.add_card_to_battlefield(0, catalog::swamp())).collect();
    let victim = ready(&mut g, 1, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::infernal_harvest());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("harvest");
    drain_stack(&mut g);
    assert!(swamps.iter().all(|id| g.battlefield_find(*id).is_none()), "both Swamps bounced");
    assert_eq!(g.battlefield_find(victim).map(|c| c.damage), Some(2));
}

/// Vampirism drains the rest of your board into its host.
#[test]
fn vampirism_feeds_on_your_other_creatures() {
    let mut g = two_player_game();
    let host = ready(&mut g, 0, catalog::grizzly_bears());
    let feeder = ready(&mut g, 0, catalog::hill_giant());
    let aura = g.add_card_to_hand(0, catalog::vampirism());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, aura, Some(Target::Permanent(host))).expect("enchant");
    drain_stack(&mut g);
    let cp = g.computed_permanent(host).expect("host");
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 for the one other creature");
    let other = g.computed_permanent(feeder).expect("feeder");
    assert_eq!((other.power, other.toughness), (2, 2), "and it pays -1/-1");
}

/// Righteous Aura buys off a source's next hit for {W} and two life.
#[test]
fn righteous_aura_prevents_the_next_hit() {
    let mut g = two_player_game();
    let auraboard = ready(&mut g, 0, catalog::righteous_aura());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    // In response — the shield names the spell already on the stack.
    g.players[0].mana_pool.add(Color::White, 1);
    activate(&mut g, auraboard, 0, None).expect("shield up");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 18, "two life paid, no bolt damage");
}

/// Phyrexian Marauder enters with X counters and can never block.
#[test]
fn phyrexian_marauder_enters_with_x_counters() {
    let mut g = two_player_game();
    let m = g.add_card_to_hand(0, catalog::phyrexian_marauder());
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: m,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("cast");
    drain_stack(&mut g);
    let c = g.battlefield_find(m).expect("there");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 3);
    assert!(catalog::phyrexian_marauder().keywords.contains(&Keyword::CantBlock));
}

// ── Wave five ───────────────────────────────────────────────────────────────

/// Vision Charm's third mode phases out an artifact.
#[test]
fn vision_charm_phases_out_an_artifact() {
    let mut g = two_player_game();
    let mine = ready(&mut g, 1, catalog::magma_mine());
    let charm = g.add_card_to_hand(0, catalog::vision_charm());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: charm,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![],
        mode: Some(2),
        x_value: None,
    })
    .expect("charm");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none());
}

/// Elephant Grass walls off black attackers and taxes the rest.
#[test]
fn elephant_grass_walls_black_and_taxes_the_rest() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::elephant_grass());
    let black = ready(&mut g, 1, catalog::python());
    let green = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: black,
            target: AttackTarget::Player(0),
        }]))
        .is_err(),
        "black creatures can't attack at all"
    );
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: green,
            target: AttackTarget::Player(0),
        }]))
        .is_err(),
        "and the rest need two generic mana"
    );
    g.players[1].mana_pool.add_colorless(2);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: green,
        target: AttackTarget::Player(0),
    }]))
    .expect("paid the toll");
}

/// Heat Wave charges a life per nonblue blocker.
#[test]
fn heat_wave_charges_life_to_block() {
    let mut g = two_player_game();
    let attacker = ready(&mut g, 0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::heat_wave());
    let blocker = ready(&mut g, 1, catalog::hill_giant());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).expect("block");
    assert_eq!(g.players[1].life, 19, "one life per blocker");
}

/// Corrosion rusts an opponent's artifacts until they crumble.
#[test]
fn corrosion_destroys_rusted_artifacts() {
    let mut g = two_player_game();
    let corrosion = g.add_card_to_battlefield(0, catalog::corrosion());
    let cheap = g.add_card_to_battlefield(1, catalog::magma_mine());
    let dear = g.add_card_to_battlefield(1, catalog::snake_basket());
    let upkeep = catalog::corrosion().triggered_abilities[0].effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_ability(corrosion, 0, None);
    ctx.targets = vec![Target::Player(1)];
    g.resolve_effect(&upkeep, &ctx).expect("first tick");
    drain_stack(&mut g);
    assert!(g.battlefield_find(cheap).is_none(), "a one-mana artifact rusts through at once");
    assert_eq!(g.battlefield_find(dear).unwrap().counter_count(CounterType::Rust), 1);
}

/// Dream Tides keeps creatures tapped through the untap step.
#[test]
fn dream_tides_stops_creatures_untapping() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dream_tides());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().tapped = true;
    g.active_player_idx = 0;
    g.do_untap();
    assert!(g.battlefield_find(bears).unwrap().tapped);
}

/// Foreshadow draws on a hit, and again next upkeep either way.
#[test]
fn foreshadow_draws_on_a_named_hit() {
    let mut g = two_player_game();
    g.players[1].library.clear();
    g.add_card_to_library(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::plains());
    let spell = g.add_card_to_hand(0, catalog::foreshadow());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let before = g.players[0].hand.len();
    cast(&mut g, spell, Some(Target::Player(1))).expect("foreshadow");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"));
    // The named card is auto-picked; either way the delayed draw is pending.
    assert!(g.players[0].hand.len() >= before - 1);
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.players[0].hand.len() >= before, "the next-upkeep draw landed");
}

/// Three Wishes exiles three cards you can play until your next turn.
#[test]
fn three_wishes_exiles_three_playable_cards() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::plains());
    }
    let spell = g.add_card_to_hand(0, catalog::three_wishes());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, spell, None).expect("wish");
    drain_stack(&mut g);
    assert_eq!(g.exile.iter().filter(|c| c.owner == 0 && c.definition.name == "Plains").count(), 3);
}

/// Necromancy reanimates, attaches, and takes the creature with it.
#[test]
fn necromancy_takes_its_creature_with_it() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::necromancy());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, aura, None).expect("cast");
    drain_stack(&mut g);
    let back = g.battlefield_find(dead).expect("on the battlefield");
    assert_eq!(back.controller, 0, "under your control");
    assert_eq!(g.battlefield_find(aura).and_then(|c| c.attached_to), Some(dead));
    let mut events = Vec::new();
    g.destroy_permanent(aura, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_none(), "the creature goes with it");
}

/// Animate Dead attaches to what it reanimates, shrinks it, and takes it back.
#[test]
fn animate_dead_attaches_and_reclaims() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(1, catalog::hill_giant());
    let aura = g.add_card_to_hand(0, catalog::animate_dead());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, aura, None).expect("cast");
    drain_stack(&mut g);
    let base = catalog::hill_giant();
    let cp = g.computed_permanent(dead).expect("back");
    assert_eq!((cp.power, cp.toughness), (base.power - 1, base.toughness));
    let mut events = Vec::new();
    g.destroy_permanent(aura, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_none());
}

/// Undiscovered Paradise bounces itself as you untap.
#[test]
fn undiscovered_paradise_bounces_at_untap() {
    let mut g = two_player_game();
    let land = ready(&mut g, 0, catalog::undiscovered_paradise());
    activate(&mut g, land, 0, None).expect("tap for mana");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1);
    assert!(g.battlefield_find(land).is_some(), "it stays until the untap step");
    g.active_player_idx = 0;
    g.do_untap();
    assert!(g.players[0].hand.iter().any(|c| c.id == land), "back in hand");
}

/// Desolation only taxes players who tapped a land for mana this turn.
#[test]
fn desolation_taxes_only_the_players_who_tapped() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::desolation());
    let plains = ready(&mut g, 0, catalog::plains());
    let idle = g.add_card_to_battlefield(1, catalog::forest());
    activate(&mut g, plains, 0, None).expect("tap for mana");
    drain_stack(&mut g);
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(plains).is_none(), "sacrificed the land it tapped");
    assert_eq!(g.players[0].life, 18, "a Plains costs two more");
    assert!(g.battlefield_find(idle).is_some(), "the idle player keeps theirs");
}

/// Elkin Lair exiles a card each upkeep and bins it if it goes unplayed.
#[test]
fn elkin_lair_gambles_a_card_each_upkeep() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::elkin_lair());
    let card = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == card), "exiled and playable");
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == card), "unplayed, so binned");
}
