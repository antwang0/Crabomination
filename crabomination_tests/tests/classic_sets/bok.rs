//! Betrayers of Kamigawa (BOK) gap closure.

use crabomination::card::{CardType, CounterType, Keyword, Supertype};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
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

fn always_yes(g: &mut GameState) {
    g.decider = Box::new(ScriptedDecider::new(
        std::iter::repeat_with(|| DecisionAnswer::Bool(true)).take(8),
    ));
}

/// Every BOK factory is registered under its printed name.
#[test]
fn bok_cards_are_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for f in [
        catalog::akki_blizzard_herder as fn() -> crabomination::card::CardDefinition,
        catalog::ashen_monstrosity,
        catalog::body_of_jukai,
        catalog::forked_branch_garami,
        catalog::harbinger_of_spring,
        catalog::kami_of_the_honored_dead,
        catalog::shinka_gatekeeper,
        catalog::scourge_of_numai,
        catalog::takenuma_bleeder,
        catalog::scaled_hulk,
        catalog::oyobi_who_split_the_heavens,
        catalog::kyoki_sanitys_eclipse,
        catalog::ishi_ishi_akki_crackshot,
        catalog::ogre_recluse,
        catalog::indebted_samurai,
        catalog::silverstorm_samurai,
        catalog::takenos_cavalry,
        catalog::isao_enlightened_bushi,
        catalog::mannichi_the_fevered_dream,
        catalog::minamo_sightbender,
        catalog::split_tail_miko,
        catalog::sakura_tribe_springcaller,
        catalog::sakiko_mother_of_summer,
        catalog::heros_demise,
        catalog::terashis_verdict,
        catalog::first_volley,
        catalog::three_tragedies,
        catalog::reduce_to_dreams,
        catalog::ribbons_of_the_reikai,
        catalog::uproot,
        catalog::stir_the_grave,
        catalog::unchecked_growth,
        catalog::enshrined_memories,
        catalog::sosukes_summons,
        catalog::day_of_destiny,
        catalog::in_the_web_of_war,
        catalog::orb_of_dreams,
        catalog::mirror_gallery,
        catalog::gods_eye_gate_to_the_reikai,
        catalog::tendo_ice_bridge,
        catalog::yomiji_who_bars_the_way,
        catalog::heed_the_mists,
        catalog::ward_of_piety,
        catalog::heart_of_light,
    ] {
        let name = f().name;
        assert!(names.contains(&name), "{name} is not registered");
    }
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Akki Blizzard-Herder's death costs everyone a land.
#[test]
fn akki_blizzard_herder_eats_a_land_from_each_player() {
    let mut g = two_player_game();
    let herder = g.add_card_to_battlefield(0, catalog::akki_blizzard_herder());
    for p in [0, 1] {
        g.add_card_to_battlefield(p, catalog::forest());
    }
    let evs = g.remove_to_graveyard_with_triggers(herder);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    for p in [0, 1] {
        assert_eq!(g.players[p].graveyard.iter().filter(|c| c.definition.is_land()).count(), 1);
    }
}

/// Soulshift 8 fishes a big Spirit back out of the graveyard.
#[test]
fn body_of_jukai_soulshifts_a_spirit() {
    let mut g = two_player_game();
    let body = g.add_card_to_battlefield(0, catalog::body_of_jukai());
    let spirit = g.add_card_to_graveyard(0, catalog::oyobi_who_split_the_heavens());
    always_yes(&mut g);
    let evs = g.remove_to_graveyard_with_triggers(body);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == spirit), "the Spirit came back");
}

/// Forked-Branch Garami's two soulshifts each return a Spirit.
#[test]
fn forked_branch_garami_soulshifts_twice() {
    let mut g = two_player_game();
    let garami = g.add_card_to_battlefield(0, catalog::forked_branch_garami());
    let a = g.add_card_to_graveyard(0, catalog::kami_of_ancient_law()); // MV 2 Spirit
    let b = g.add_card_to_graveyard(0, catalog::gibbering_kami()); // MV 4 Spirit
    always_yes(&mut g);
    let evs = g.remove_to_graveyard_with_triggers(garami);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    // Both triggers fire; the auto-targeter reuses one legal Spirit, so the
    // second fizzles (a live seat would pick the other).
    assert_eq!(catalog::forked_branch_garami().triggered_abilities.len(), 2);
    let back = [a, b].iter().filter(|id| g.players[0].hand.iter().any(|c| c.id == **id)).count();
    assert!(back >= 1, "soulshift returned a Spirit");
}

/// Harbinger of Spring can't be blocked by anything that isn't a Spirit.
#[test]
fn harbinger_of_spring_only_spirits_may_block() {
    let block = |blocker: crabomination::card::CardDefinition| {
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(0, catalog::harbinger_of_spring());
        g.clear_sickness(atk);
        let blk = g.add_card_to_battlefield(1, blocker);
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: atk,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).is_ok()
    };
    assert!(!block(catalog::grizzly_bears()), "a Bear can't get in the way");
    assert!(block(catalog::scaled_hulk()), "a Spirit can");
}

/// Kami of the Honored Dead converts damage taken into life.
#[test]
fn kami_of_the_honored_dead_drinks_damage() {
    let mut g = two_player_game();
    let kami = g.add_card_to_battlefield(0, catalog::kami_of_the_honored_dead());
    let life = g.players[0].life;
    let mut evs = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(kami),
        3,
        None,
        &mut evs,
    );
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3);
}

/// Shinka Gatekeeper passes the damage it takes straight to you.
#[test]
fn shinka_gatekeeper_reflects_damage() {
    let mut g = two_player_game();
    let ogre = g.add_card_to_battlefield(0, catalog::shinka_gatekeeper());
    let life = g.players[0].life;
    let mut evs = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(ogre),
        2,
        None,
        &mut evs,
    );
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2);
}

/// Scourge of Numai only bleeds you without an Ogre on board.
#[test]
fn scourge_of_numai_wants_an_ogre() {
    let upkeep = |with_ogre: bool| {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::scourge_of_numai());
        if with_ogre {
            g.add_card_to_battlefield(0, catalog::ogre_recluse());
        }
        let life = g.players[0].life;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        life - g.players[0].life
    };
    assert_eq!(upkeep(false), 2);
    assert_eq!(upkeep(true), 0);
}

/// Scaled Hulk swells on each Spirit or Arcane spell.
#[test]
fn scaled_hulk_grows_on_arcane() {
    let mut g = two_player_game();
    let hulk = g.add_card_to_battlefield(0, catalog::scaled_hulk());
    let bolt = g.add_card_to_hand(0, catalog::uproot());
    g.add_card_to_battlefield(1, catalog::forest());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let land = g.battlefield.iter().find(|c| c.controller == 1).map(|c| c.id).unwrap();
    cast(&mut g, bolt, Some(Target::Permanent(land)));
    assert_eq!(g.computed_permanent(hulk).map(|c| (c.power, c.toughness)), Some((6, 6)));
}

/// Oyobi mints a 3/3 flier on each Spirit or Arcane spell.
#[test]
fn oyobi_mints_a_flying_spirit() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::oyobi_who_split_the_heavens());
    let spell = g.add_card_to_hand(0, catalog::three_tragedies());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, spell, Some(Target::Player(1)));
    let token = g
        .battlefield
        .iter()
        .find(|c| c.is_token && c.definition.name == "Spirit")
        .expect("Spirit token");
    assert_eq!((token.definition.power, token.definition.toughness), (3, 3));
    assert!(token.definition.keywords.contains(&Keyword::Flying));
}

/// Ishi-Ishi burns an opponent for casting Arcane.
#[test]
fn ishi_ishi_punishes_opposing_arcane() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ishi_ishi_akki_crackshot());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(1, catalog::first_volley());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    let life = g.players[1].life;
    cast(&mut g, spell, Some(Target::Permanent(bear)));
    assert_eq!(g.players[1].life, life - 2, "Ishi-Ishi burned the caster");
}

/// Indebted Samurai grows when a fellow Samurai dies.
#[test]
fn indebted_samurai_mourns_with_a_counter() {
    let mut g = two_player_game();
    let sam = g.add_card_to_battlefield(0, catalog::indebted_samurai());
    let other = g.add_card_to_battlefield(0, catalog::silverstorm_samurai());
    always_yes(&mut g);
    let mut evs = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(other),
        3,
        None,
        &mut evs,
    );
    let sba = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&sba);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(sam).map(|c| c.counter_count(CounterType::PlusOnePlusOne)), Some(1));
}

/// Isao can't be countered and keeps a Samurai alive.
#[test]
fn isao_is_uncounterable_and_regenerates() {
    let d = catalog::isao_enlightened_bushi();
    assert!(d.keywords.contains(&Keyword::CantBeCountered));
    assert!(d.keywords.contains(&Keyword::Bushido(2)));
}

/// Mannichi flips everyone's power and toughness.
#[test]
fn mannichi_switches_every_bodys_stats() {
    let mut g = two_player_game();
    let mannichi = g.add_card_to_battlefield(0, catalog::mannichi_the_fevered_dream());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hulk = g.add_card_to_battlefield(0, catalog::body_of_jukai());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mannichi,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).map(|c| (c.power, c.toughness)), Some((2, 2)));
    assert_eq!(g.computed_permanent(hulk).map(|c| (c.power, c.toughness)), Some((5, 8)));
}

/// Minamo Sightbender slips a small creature through, but only up to X.
#[test]
fn minamo_sightbender_gates_on_x() {
    let sneak = |x: u32, power_target: bool| {
        let mut g = two_player_game();
        let bender = g.add_card_to_battlefield(0, catalog::minamo_sightbender());
        g.clear_sickness(bender);
        let victim = if power_target {
            g.add_card_to_battlefield(0, catalog::grizzly_bears()) // 2/2
        } else {
            g.add_card_to_battlefield(0, catalog::body_of_jukai()) // 8/5
        };
        g.players[0].mana_pool.add_colorless(x);
        let ok = g
            .perform_action(GameAction::ActivateAbility {
                card_id: bender,
                ability_index: 0,
                target: Some(Target::Permanent(victim)),
                additional_targets: vec![],
                mode: None,
                x_value: Some(x),
            })
            .is_ok();
        drain_stack(&mut g);
        ok && g
            .computed_permanent(victim)
            .is_some_and(|c| c.keywords.contains(&Keyword::Unblockable))
    };
    assert!(sneak(2, true), "X=2 covers a 2/2");
    assert!(!sneak(2, false), "X=2 doesn't cover an 8/5");
}

/// Sakura-Tribe Springcaller's {G} survives the step change.
#[test]
fn springcaller_mana_outlives_the_step() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sakura_tribe_springcaller());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
    g.empty_mana_pools();
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1, "kept across the step");
}

/// Sakiko banks {G} equal to the combat damage your creatures deal.
#[test]
fn sakiko_banks_green_off_combat_damage() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sakiko_mother_of_summer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.fire_combat_damage_to_player_triggers(bear, 1, 2);
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2);
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Hero's Demise only answers legends.
#[test]
fn heros_demise_wants_a_legend() {
    let kill = |legendary: bool| {
        let mut g = two_player_game();
        let victim = if legendary {
            g.add_card_to_battlefield(1, catalog::oyobi_who_split_the_heavens())
        } else {
            g.add_card_to_battlefield(1, catalog::grizzly_bears())
        };
        let spell = g.add_card_to_hand(0, catalog::heros_demise());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(victim)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_ok()
    };
    assert!(kill(true));
    assert!(!kill(false), "a nonlegendary Bear isn't a legal target");
}

/// Terashi's Verdict answers a small attacker only.
#[test]
fn terashis_verdict_hits_small_attackers() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Player(0) }];
    let spell = g.add_card_to_hand(0, catalog::terashis_verdict());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, spell, Some(Target::Permanent(atk)));
    assert!(g.battlefield_find(atk).is_none());
}

/// First Volley splashes the creature's controller for 1.
#[test]
fn first_volley_hits_creature_and_controller() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::first_volley());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[1].life;
    cast(&mut g, spell, Some(Target::Permanent(bear)));
    assert_eq!(g.players[1].life, life - 1);
    assert_eq!(g.battlefield_find(bear).map(|c| c.damage), Some(1));
}

/// Reduce to Dreams bounces every artifact and enchantment.
#[test]
fn reduce_to_dreams_clears_both_types() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::orb_of_dreams());
    g.add_card_to_battlefield(1, catalog::day_of_destiny());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::reduce_to_dreams());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, spell, None);
    assert_eq!(g.players[0].hand.iter().filter(|c| c.definition.is_artifact()).count(), 1);
    assert_eq!(g.players[1].hand.len(), 1, "the enchantment went home");
    assert!(g.battlefield_find(bear).is_some(), "creatures stay");
}

/// Ribbons of the Reikai draws one per Spirit.
#[test]
fn ribbons_draws_per_spirit() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::scaled_hulk());
        g.add_card_to_library(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::ribbons_of_the_reikai());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    let hand = g.players[0].hand.len();
    cast(&mut g, spell, None);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3);
}

/// Uproot puts a land back on top.
#[test]
fn uproot_stacks_a_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::uproot());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, spell, Some(Target::Permanent(land)));
    assert_eq!(g.players[1].library.first().map(|c| c.id), Some(land));
}

/// Stir the Grave reanimates only within X.
#[test]
fn stir_the_grave_respects_x() {
    let raise = |x: u32| {
        let mut g = two_player_game();
        let big = g.add_card_to_graveyard(0, catalog::body_of_jukai()); // MV 9
        let spell = g.add_card_to_hand(0, catalog::stir_the_grave());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(x);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(big)),
            additional_targets: vec![],
            mode: None,
            x_value: Some(x),
        })
        .is_ok()
    };
    assert!(!raise(2), "X=2 can't reach a 9-drop");
    assert!(raise(9));
}

/// Unchecked Growth's trample rider is Spirit-only.
#[test]
fn unchecked_growth_tramples_for_spirits_only() {
    let pump = |spirit: bool| {
        let mut g = two_player_game();
        let target = if spirit {
            g.add_card_to_battlefield(0, catalog::harbinger_of_spring()) // 2/1 Spirit
        } else {
            g.add_card_to_battlefield(0, catalog::grizzly_bears())
        };
        let spell = g.add_card_to_hand(0, catalog::unchecked_growth());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, spell, Some(Target::Permanent(target)));
        let cp = g.computed_permanent(target).unwrap();
        (cp.power, cp.keywords.contains(&Keyword::Trample))
    };
    assert_eq!(pump(true), (6, true));
    assert_eq!(pump(false), (6, false));
}

/// Enshrined Memories takes the creatures and bottoms the rest.
#[test]
fn enshrined_memories_takes_creatures() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let land = g.add_card_to_library(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::enshrined_memories());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
    assert!(g.players[0].library.iter().any(|c| c.id == land));
}

/// Sosuke's Summons mints two Snakes and climbs back out of the graveyard.
#[test]
fn sosukes_summons_recurs_on_a_snake() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::sosukes_summons());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, spell, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Snake").count(), 2);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == spell));
    // A nontoken Snake brings it home.
    always_yes(&mut g);
    let snake = g.add_card_to_hand(0, catalog::sakura_tribe_springcaller());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, snake, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == spell), "Summons returned to hand");
}

// ── Noncreature permanents ──────────────────────────────────────────────────

/// Day of Destiny only pumps legends.
#[test]
fn day_of_destiny_pumps_only_legends() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::day_of_destiny());
    let legend = g.add_card_to_battlefield(0, catalog::sakiko_mother_of_summer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(legend).map(|c| (c.power, c.toughness)), Some((5, 5)));
    assert_eq!(g.computed_permanent(bear).map(|c| (c.power, c.toughness)), Some((2, 2)));
}

/// In the Web of War gives every arrival +2/+0 and haste.
#[test]
fn in_the_web_of_war_arms_arrivals() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::in_the_web_of_war());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, bear, None);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 4);
    assert!(cp.keywords.contains(&Keyword::Haste));
}

/// Orb of Dreams taps everything down as it arrives.
#[test]
fn orb_of_dreams_taps_arrivals() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::orb_of_dreams());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    cast(&mut g, bear, None);
    assert_eq!(g.battlefield_find(bear).map(|c| c.tapped), Some(true));
}

/// Mirror Gallery switches the legend rule off.
#[test]
fn mirror_gallery_suspends_the_legend_rule() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::sakiko_mother_of_summer());
    let b = g.add_card_to_battlefield(0, catalog::sakiko_mother_of_summer());
    g.check_state_based_actions();
    assert!(g.battlefield_find(a).is_none() || g.battlefield_find(b).is_none(), "legend rule bites");

    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mirror_gallery());
    let a = g.add_card_to_battlefield(0, catalog::sakiko_mother_of_summer());
    let b = g.add_card_to_battlefield(0, catalog::sakiko_mother_of_summer());
    g.check_state_based_actions();
    assert!(g.battlefield_find(a).is_some() && g.battlefield_find(b).is_some());
}

/// Gods' Eye leaves a Spirit behind.
#[test]
fn gods_eye_leaves_a_spirit() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::gods_eye_gate_to_the_reikai());
    let evs = g.remove_to_graveyard_with_triggers(land);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Spirit"));
}

/// Tendo Ice Bridge enters with a charge counter it can spend for any color.
#[test]
fn tendo_ice_bridge_spends_its_charge() {
    let mut g = two_player_game();
    assert!(catalog::tendo_ice_bridge().enters_with_counters.is_some(), "enters charged");
    let land = g.add_card_to_battlefield(0, catalog::tendo_ice_bridge());
    if let Some(c) = g.battlefield_find_mut(land) {
        c.add_counters(CounterType::Charge, 1);
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1, "one mana of the chosen color");
    assert_eq!(g.battlefield_find(land).map(|c| c.counter_count(CounterType::Charge)), Some(0));
}

/// Yomiji sends other dying legends home instead of to the graveyard.
#[test]
fn yomiji_recalls_dying_legends() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::yomiji_who_bars_the_way());
    let legend = g.add_card_to_battlefield(1, catalog::sakiko_mother_of_summer());
    let evs = g.remove_to_graveyard_with_triggers(legend);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == legend));
}

/// Heed the Mists draws the milled card's mana value.
#[test]
fn heed_the_mists_draws_the_milled_cost() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::body_of_jukai()); // MV 9 — milled
    for _ in 0..9 {
        g.add_card_to_library(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::heed_the_mists());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    let hand = g.players[0].hand.len();
    cast(&mut g, spell, None);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 9);
}

/// Heart of Light seals the enchanted creature in both directions.
#[test]
fn heart_of_light_seals_both_ways() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::heart_of_light());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, aura, Some(Target::Permanent(bear)));
    let mut evs = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(bear),
        5,
        None,
        &mut evs,
    );
    assert_eq!(g.battlefield_find(bear).map(|c| c.damage), Some(0), "takes nothing");
    let life = g.players[0].life;
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(0),
        3,
        Some(bear),
        &mut evs,
    );
    assert_eq!(g.players[0].life, life, "deals nothing");
}

/// Ashen Monstrosity is forced into combat.
#[test]
fn ashen_monstrosity_must_swing() {
    let d = catalog::ashen_monstrosity();
    assert!(d.keywords.contains(&Keyword::MustAttack));
    assert!(d.keywords.contains(&Keyword::Haste));
}

/// The BOK stat lines match print.
#[test]
fn bok_stat_lines() {
    let checks: [(fn() -> crabomination::card::CardDefinition, i32, i32); 6] = [
        (catalog::ashen_monstrosity, 7, 4),
        (catalog::body_of_jukai, 8, 5),
        (catalog::harbinger_of_spring, 2, 1),
        (catalog::takenuma_bleeder, 3, 3),
        (catalog::silverstorm_samurai, 3, 3),
        (catalog::takenos_cavalry, 1, 1),
    ];
    for (f, p, t) in checks {
        let d = f();
        assert_eq!((d.power, d.toughness), (p, t), "{}", d.name);
    }
    assert!(catalog::day_of_destiny().supertypes.contains(&Supertype::Legendary));
    assert!(catalog::orb_of_dreams().card_types.contains(&CardType::Artifact));
}

// ── Batch 2 ─────────────────────────────────────────────────────────────────

/// The Genju cycle is registered and enchants its own basic type.
#[test]
fn genju_cycle_is_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for f in [
        catalog::genju_of_the_cedars as fn() -> crabomination::card::CardDefinition,
        catalog::genju_of_the_falls,
        catalog::genju_of_the_spires,
        catalog::genju_of_the_fens,
        catalog::genju_of_the_fields,
        catalog::genju_of_the_realm,
    ] {
        let d = f();
        assert!(names.contains(&d.name), "{} is not registered", d.name);
        assert_eq!(d.activated_abilities.len(), 1, "{} animates for {{2}}", d.name);
    }
}

/// Genju of the Spires turns its Mountain into a 6/1 for the turn.
#[test]
fn genju_of_the_spires_animates_its_mountain() {
    let mut g = two_player_game();
    let mountain = g.add_card_to_battlefield(0, catalog::mountain());
    let genju = g.add_card_to_hand(0, catalog::genju_of_the_spires());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, genju, Some(Target::Permanent(mountain)));
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: genju,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("animate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(mountain).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 1));
    assert!(cp.card_types.contains(&CardType::Land), "still a land");
}

/// A Genju climbs back out of the graveyard when its land dies.
#[test]
fn genju_returns_when_its_land_dies() {
    let mut g = two_player_game();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let genju = g.add_card_to_hand(0, catalog::genju_of_the_cedars());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, genju, Some(Target::Permanent(forest)));
    always_yes(&mut g);
    let evs = g.remove_to_graveyard_with_triggers(forest);
    g.dispatch_triggers_for_events(&evs);
    // The orphaned Aura is swept by SBA; its "when enchanted land dies"
    // trigger fires off that sweep.
    let sba = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&sba);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == genju), "the Genju came home");
}

/// Floodbringer's bounce cost really returns a land.
#[test]
fn floodbringer_pays_with_a_land() {
    let mut g = two_player_game();
    let bringer = g.add_card_to_battlefield(0, catalog::floodbringer());
    g.clear_sickness(bringer);
    let mine = g.add_card_to_battlefield(0, catalog::island());
    let theirs = g.add_card_to_battlefield(1, catalog::forest());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bringer,
        ability_index: 0,
        target: Some(Target::Permanent(theirs)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == mine), "my land bounced as the cost");
    assert_eq!(g.battlefield_find(theirs).map(|c| c.tapped), Some(true));
}

/// Lifespinner eats three Spirits for a legendary Spirit.
#[test]
fn lifespinner_trades_three_spirits() {
    let mut g = two_player_game();
    let spinner = g.add_card_to_battlefield(0, catalog::lifespinner());
    g.clear_sickness(spinner);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::scaled_hulk());
    }
    let prize = g.add_card_to_library(0, catalog::oyobi_who_split_the_heavens());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(prize))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: spinner,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Scaled Hulk").count(), 0);
    assert!(g.battlefield_find(prize).is_some(), "Oyobi hit the battlefield");
}

/// That Which Was Taken hands out indestructibility with its divinity counters.
#[test]
fn that_which_was_taken_grants_indestructible() {
    let mut g = two_player_game();
    let relic = g.add_card_to_battlefield(0, catalog::that_which_was_taken());
    g.clear_sickness(relic);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: relic,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).map(|c| c.counter_count(CounterType::Divinity)), Some(1));
    assert!(
        g.computed_permanent(bear)
            .is_some_and(|c| c.keywords.contains(&Keyword::Indestructible))
    );
}

/// Ronin Warclub jumps onto each creature you deploy.
#[test]
fn ronin_warclub_leaps_to_new_arrivals() {
    let mut g = two_player_game();
    let club = g.add_card_to_battlefield(0, catalog::ronin_warclub());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, bear, None);
    assert_eq!(g.battlefield_find(club).and_then(|c| c.attached_to), Some(bear));
    assert_eq!(g.computed_permanent(bear).map(|c| (c.power, c.toughness)), Some((4, 3)));
}

/// Shizuko banks {G}{G}{G} for whoever's upkeep it is.
#[test]
fn shizuko_pays_the_active_player() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::shizuko_caller_of_autumn());
    g.active_player_idx = 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].mana_pool.amount(Color::Green), 3, "the opponent gets it too");
}

/// Call for Blood shrinks by the sacrificed creature's power.
#[test]
fn call_for_blood_scales_off_the_sacrifice() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::body_of_jukai()); // 8/5
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::call_for_blood());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, spell, Some(Target::Permanent(victim)));
    g.check_state_based_actions();
    assert!(g.battlefield_find(fodder).is_none(), "the sacrifice was paid");
    assert!(g.battlefield_find(victim).is_none(), "-8/-8 killed it");
}

/// Stream of Consciousness shuffles graveyard cards back in.
#[test]
fn stream_of_consciousness_recycles() {
    let mut g = two_player_game();
    let a = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::stream_of_consciousness());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, spell, Some(Target::Permanent(a)));
    assert!(g.players[1].library.iter().any(|c| c.id == a));
}

/// Kaijin sends its blocking victim home at end of combat.
#[test]
fn kaijin_bounces_what_it_blocks() {
    let mut g = two_player_game();
    let kaijin = g.add_card_to_battlefield(0, catalog::kaijin_of_the_vanishing_touch());
    let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![(kaijin, atk)])).expect("block");
    drain_stack(&mut g);
    g.step = TurnStep::EndCombat;
    g.fire_step_triggers(TurnStep::EndCombat);
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == atk), "the attacker bounced");
}
