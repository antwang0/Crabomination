//! Functionality tests for Magic 2015 (M15) — `catalog::sets::m15`.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn cast(
    g: &mut GameState,
    def: crabomination::card::CardDefinition,
    target: Option<Target>,
    colorless: u32,
    colors: &[(Color, u32)],
) -> CardId {
    let id = g.add_card_to_hand(0, def);
    g.players[0].mana_pool.add_colorless(colorless);
    for (c, n) in colors {
        g.players[0].mana_pool.add(*c, *n);
    }
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
    id
}

fn activate(g: &mut GameState, card: CardId, idx: usize, target: Option<Target>) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: card,
        ability_index: idx,
        target,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Stat / keyword lines for the M15 bodies.
#[test]
fn m15_stat_lines() {
    let table: &[(fn() -> crabomination::card::CardDefinition, i32, i32, &[Keyword])] = &[
        (catalog::geist_of_the_moors, 3, 1, &[Keyword::Flying]),
        (catalog::sungrace_pegasus, 1, 2, &[Keyword::Flying, Keyword::Lifelink]),
        (catalog::krenkos_enforcer, 2, 2, &[Keyword::Intimidate]),
        (catalog::xathrid_slyblade, 2, 1, &[Keyword::Hexproof]),
        (catalog::witchs_familiar, 2, 3, &[]),
        (catalog::nimbus_of_the_isles, 3, 3, &[Keyword::Flying]),
        (catalog::netcaster_spider, 2, 3, &[Keyword::Reach]),
        (catalog::phytotitan, 7, 2, &[]),
        (catalog::siege_dragon, 5, 5, &[Keyword::Flying]),
        (catalog::will_forged_golem, 4, 4, &[Keyword::Convoke]),
        (catalog::living_totem, 2, 3, &[Keyword::Convoke]),
        (catalog::soul_of_theros, 6, 6, &[Keyword::Vigilance]),
        (catalog::soul_of_ravnica, 6, 6, &[Keyword::Flying]),
        (catalog::soul_of_innistrad, 6, 6, &[Keyword::Deathtouch]),
        (catalog::soul_of_shandalar, 6, 6, &[Keyword::FirstStrike]),
        (catalog::soul_of_zendikar, 6, 6, &[Keyword::Reach]),
        (catalog::soul_of_new_phyrexia, 6, 6, &[Keyword::Trample]),
        (catalog::miners_bane, 6, 3, &[]),
        (catalog::wall_of_limbs, 0, 3, &[Keyword::Defender]),
    ];
    for (f, p, t, kws) in table {
        let d = f();
        assert_eq!((d.power, d.toughness), (*p, *t), "{}", d.name);
        for kw in *kws {
            assert!(d.keywords.contains(kw), "{} lacks {:?}", d.name, kw);
        }
    }
}

/// CR 702.51 Convoke — tapping creatures pays the generic and colored pips.
#[test]
fn convoke_taps_creatures_for_the_cost() {
    let mut g = main_phase();
    let helpers: Vec<_> = (0..6)
        .map(|_| g.add_card_to_battlefield(0, catalog::sungrace_pegasus()))
        .collect();
    for h in &helpers {
        g.clear_sickness(*h);
    }
    let spirits = g.add_card_to_hand(0, catalog::triplicate_spirits());
    g.perform_action(GameAction::CastSpellConvoke {
        card_id: spirits,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        convoke_creatures: helpers.clone(),
    })
    .expect("all six paid by convoke, including the two {W} pips");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Spirit").count(),
        3
    );
    assert!(helpers.iter().all(|h| g.battlefield_find(*h).unwrap().tapped), "all tapped");
}

/// The Paragon cycle anthems only its own color and grants only to it.
#[test]
fn paragon_lords_its_own_color() {
    let mut g = main_phase();
    let paragon = g.add_card_to_battlefield(0, catalog::paragon_of_gathering_mists());
    g.clear_sickness(paragon);
    let blue = g.add_card_to_battlefield(0, catalog::nimbus_of_the_isles()); // 3/3 blue
    let green = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(blue).map(|c| (c.power, c.toughness)), Some((4, 4)));
    assert_eq!(g.computed_permanent(green).map(|c| (c.power, c.toughness)), Some((2, 2)));
    assert_eq!(
        g.computed_permanent(paragon).map(|c| (c.power, c.toughness)),
        Some((2, 2)),
        "other creatures only"
    );
}

/// The Soul cycle's ability works from the graveyard for an exile cost.
#[test]
fn soul_of_zendikar_works_from_the_graveyard() {
    let mut g = main_phase();
    let soul = g.add_card_to_graveyard(0, catalog::soul_of_zendikar());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Green, 2);
    activate(&mut g, soul, 1, None);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Beast"));
    assert!(
        g.players[0].graveyard.iter().all(|c| c.id != soul),
        "the Soul exiled itself as a cost"
    );
}

/// Soul of Ravnica draws one per color among your permanents.
#[test]
fn soul_of_ravnica_draws_per_color() {
    let mut g = main_phase();
    let soul = g.add_card_to_battlefield(0, catalog::soul_of_ravnica());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green
    g.add_card_to_battlefield(0, catalog::sungrace_pegasus()); // white
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let hand = g.players[0].hand.len();
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(Color::Blue, 2);
    activate(&mut g, soul, 0, None);
    assert_eq!(g.players[0].hand.len(), hand + 3, "blue, green, white");
}

/// The land-type cycle grows only while you control the gating land.
#[test]
fn sunblade_elf_grows_beside_a_plains() {
    let mut g = main_phase();
    let elf = g.add_card_to_battlefield(0, catalog::sunblade_elf());
    assert_eq!(g.computed_permanent(elf).map(|c| (c.power, c.toughness)), Some((1, 1)));
    g.add_card_to_battlefield(0, catalog::plains());
    assert_eq!(g.computed_permanent(elf).map(|c| (c.power, c.toughness)), Some((2, 2)));
}

/// Kalonian Twingrove and its token both count Forests.
#[test]
fn kalonian_twingrove_counts_forests() {
    let mut g = main_phase();
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let tree = g.add_card_to_battlefield(0, catalog::kalonian_twingrove());
    g.fire_self_etb_triggers(tree, 0);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(tree).map(|c| (c.power, c.toughness)), Some((4, 4)));
    let token = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Treefolk Warrior" && c.is_token)
        .expect("token");
    assert_eq!(g.computed_permanent(token.id).map(|c| (c.power, c.toughness)), Some((4, 4)));
}

/// Aeronaut Tinkerer only flies with an artifact out.
#[test]
fn aeronaut_tinkerer_needs_an_artifact() {
    let mut g = main_phase();
    let tinkerer = g.add_card_to_battlefield(0, catalog::aeronaut_tinkerer());
    assert!(!g.computed_permanent(tinkerer).unwrap().keywords.contains(&Keyword::Flying));
    g.add_card_to_battlefield(0, catalog::sacred_armory());
    assert!(g.computed_permanent(tinkerer).unwrap().keywords.contains(&Keyword::Flying));
}

/// Scrapyard Mongrel's artifact clause pumps and tramples together.
#[test]
fn scrapyard_mongrel_reads_your_artifacts() {
    let mut g = main_phase();
    let dog = g.add_card_to_battlefield(0, catalog::scrapyard_mongrel());
    assert_eq!(g.computed_permanent(dog).map(|c| (c.power, c.toughness)), Some((3, 3)));
    g.add_card_to_battlefield(0, catalog::tyrants_machine());
    let cp = g.computed_permanent(dog).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 3));
    assert!(cp.keywords.contains(&Keyword::Trample));
}

/// Undergrowth Scavenger sizes itself off every graveyard.
#[test]
fn undergrowth_scavenger_counts_all_graveyards() {
    let mut g = main_phase();
    for seat in [0, 1, 1] {
        g.add_card_to_graveyard(seat, catalog::grizzly_bears());
    }
    let scav = cast(&mut g, catalog::undergrowth_scavenger(), None, 3, &[(Color::Green, 1)]);
    assert_eq!(g.battlefield_find(scav).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}

/// Phytotitan comes back tapped at the next upkeep.
#[test]
fn phytotitan_returns_at_your_upkeep() {
    let mut g = main_phase();
    let titan = g.add_card_to_battlefield(0, catalog::phytotitan());
    let mut evs = Vec::new();
    g.destroy_permanent(titan, false, &mut evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(titan).is_none());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let back = g.battlefield_find(titan).expect("returned");
    assert!(back.tapped);
}

/// Resolute Archangel resets you to your starting life, never down from it.
#[test]
fn resolute_archangel_restores_starting_life() {
    let mut g = main_phase();
    g.players[0].life = 4;
    let angel = g.add_card_to_battlefield(0, catalog::resolute_archangel());
    g.fire_self_etb_triggers(angel, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, g.players[0].starting_life);
}

/// Hunter's Ambush fogs nongreen attackers but lets green damage through.
#[test]
fn hunters_ambush_spares_green_damage() {
    let mut g = main_phase();
    let green = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green 2/2
    let white = g.add_card_to_battlefield(1, catalog::sungrace_pegasus()); // white 1/2
    for c in [green, white] {
        g.clear_sickness(c);
    }
    cast(&mut g, catalog::hunters_ambush(), None, 2, &[(Color::Green, 1)]);
    assert!(g.prevent_combat_damage_this_turn);
    let life = g.players[0].life;
    use crabomination::game::types::{Attack, AttackTarget};
    g.attacking = vec![
        Attack { attacker: green, target: AttackTarget::Player(0) },
        Attack { attacker: white, target: AttackTarget::Player(0) },
    ];
    g.active_player_idx = 1;
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2, "only the green attacker connected");
}

/// Chronostutter buries a creature second from the top.
#[test]
fn chronostutter_buries_second_from_the_top() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let victim = g.add_card_to_battlefield(1, catalog::oreskos_swiftclaw());
    cast(
        &mut g,
        catalog::chronostutter(),
        Some(Target::Permanent(victim)),
        5,
        &[(Color::Blue, 1)],
    );
    assert_eq!(g.players[1].library.iter().position(|c| c.id == victim), Some(1));
}

/// Perilous Vault exiles every nonland permanent, itself included.
#[test]
fn perilous_vault_sweeps_the_board() {
    let mut g = main_phase();
    let vault = g.add_card_to_battlefield(0, catalog::perilous_vault());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.clear_sickness(vault);
    g.players[0].mana_pool.add_colorless(5);
    activate(&mut g, vault, 0, None);
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none());
    assert!(g.battlefield_find(land).is_some(), "lands survive");
}

/// Military Intelligence needs two attackers.
#[test]
fn military_intelligence_wants_a_crew() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::military_intelligence());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for c in [a, b] {
        g.clear_sickness(c);
    }
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.step = TurnStep::DeclareAttackers;
    let hand = g.players[0].hand.len();
    g.declare_attackers(vec![
        crabomination::game::types::Attack {
            attacker: a,
            target: crabomination::game::types::AttackTarget::Player(1),
        },
        crabomination::game::types::Attack {
            attacker: b,
            target: crabomination::game::types::AttackTarget::Player(1),
        },
    ])
    .expect("swing with two");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Blastfire Bolt takes the Equipment with the creature.
#[test]
fn blastfire_bolt_strips_equipment() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let plate = g.add_card_to_battlefield(1, catalog::brawlers_plate());
    g.battlefield_find_mut(plate).unwrap().attached_to = Some(bear);
    cast(
        &mut g,
        catalog::blastfire_bolt(),
        Some(Target::Permanent(bear)),
        5,
        &[(Color::Red, 1)],
    );
    assert!(g.battlefield_find(bear).is_none(), "5 damage killed the 2/2");
    assert!(g.battlefield_find(plate).is_none(), "its Equipment went too");
}

/// In Garruk's Wake spares your own board.
#[test]
fn in_garruks_wake_spares_your_side() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    cast(&mut g, catalog::in_garruks_wake(), None, 7, &[(Color::Black, 2)]);
    assert!(g.battlefield_find(mine).is_some() && g.battlefield_find(theirs).is_none());
}

/// Wall of Limbs grows on life gain and drains for its power.
#[test]
fn wall_of_limbs_grows_and_drains() {
    let mut g = main_phase();
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_limbs());
    for _ in 0..3 {
        cast(&mut g, catalog::meditation_puzzle(), None, 3, &[(Color::White, 2)]);
    }
    assert_eq!(g.battlefield_find(wall).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    let life = g.players[1].life;
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(Color::Black, 2);
    activate(&mut g, wall, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 3);
}

/// Seraph of the Masses scales with your board.
#[test]
fn seraph_of_the_masses_counts_the_team() {
    let mut g = main_phase();
    let seraph = g.add_card_to_battlefield(0, catalog::seraph_of_the_masses());
    assert_eq!(g.computed_permanent(seraph).map(|c| (c.power, c.toughness)), Some((1, 1)));
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(seraph).map(|c| (c.power, c.toughness)), Some((3, 3)));
}

/// Siege Dragon clears Walls on entry.
#[test]
fn siege_dragon_eats_walls() {
    let mut g = main_phase();
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_limbs());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dragon = g.add_card_to_battlefield(0, catalog::siege_dragon());
    g.fire_self_etb_triggers(dragon, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(wall).is_none() && g.battlefield_find(bear).is_some());
}

/// Ensoul Artifact animates its host as a 5/5 that keeps being an artifact.
#[test]
fn ensoul_artifact_makes_a_five_five() {
    let mut g = main_phase();
    let armory = g.add_card_to_battlefield(0, catalog::sacred_armory());
    cast(
        &mut g,
        catalog::ensoul_artifact(),
        Some(Target::Permanent(armory)),
        1,
        &[(Color::Blue, 1)],
    );
    let cp = g.computed_permanent(armory).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
    assert!(cp.card_types.contains(&crabomination::card::CardType::Creature));
    assert!(cp.card_types.contains(&crabomination::card::CardType::Artifact));
}

/// Burning Anger turns the host into a repeatable cannon.
#[test]
fn burning_anger_arms_the_host() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    cast(&mut g, catalog::burning_anger(), Some(Target::Permanent(bear)), 4, &[(Color::Red, 1)]);
    let life = g.players[1].life;
    activate(&mut g, bear, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 2, "power 2 = 2 damage");
}

/// Brood Keeper mints a Dragon whenever an Aura lands on it.
#[test]
fn brood_keeper_hatches_on_each_aura() {
    let mut g = main_phase();
    let keeper = g.add_card_to_battlefield(0, catalog::brood_keeper());
    cast(
        &mut g,
        catalog::marked_by_honor(),
        Some(Target::Permanent(keeper)),
        3,
        &[(Color::White, 1)],
    );
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Dragon").count(),
        1
    );
}

/// Necromancer's Stockpile pays off extra for a discarded Zombie.
#[test]
fn necromancers_stockpile_rewards_zombies() {
    let mut g = main_phase();
    let pile = g.add_card_to_battlefield(0, catalog::necromancers_stockpile());
    g.add_card_to_hand(0, catalog::carrion_crow()); // Zombie Bird
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    activate(&mut g, pile, 0, None);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Zombie").count(),
        1,
        "the discarded Zombie minted a token"
    );
}

/// Avarice Amulet's upkeep draw rides the equipped creature.
#[test]
fn avarice_amulet_draws_each_upkeep() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let amulet = g.add_card_to_battlefield(0, catalog::avarice_amulet());
    g.battlefield_find_mut(amulet).unwrap().attached_to = Some(bear);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 2));
    assert!(cp.keywords.contains(&Keyword::Vigilance));
    let hand = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}
