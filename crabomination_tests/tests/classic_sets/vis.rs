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
