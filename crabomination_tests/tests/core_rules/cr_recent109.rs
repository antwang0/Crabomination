//! CR 707.10 — a copy of a spell is not cast. Magecraft says "whenever you
//! cast OR COPY an instant or sorcery spell", so it fires for the copy too
//! (`EventSpec::or_copy`, run by the dispatcher once per `SpellsCopied`),
//! while a plain "whenever you cast" trigger still does not.

use crabomination::catalog;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;
use crabomination::TurnStep;

/// Bolt, then Reverberate copying it: returns how many of `name` seat 0 has.
fn bolt_and_copy(listener: crabomination::card::CardDefinition, name: &str) -> usize {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, listener);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let rev = g.add_card_to_hand(0, catalog::reverberate());
    g.players[0].mana_pool.add(Color::Red, 3);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt");
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: rev,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Reverberate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 14, "the Bolt and its copy both hit");
    g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == name).count()
}

#[test]
fn cr_707_10_magecraft_fires_for_a_copy() {
    // Bolt cast, Reverberate cast, the copy: three.
    assert_eq!(bolt_and_copy(catalog::storm_kiln_artist(), "Treasure"), 3);
}

#[test]
fn cr_707_10_a_cast_trigger_ignores_the_copy() {
    // "Whenever you cast an instant or sorcery spell": the two casts only.
    assert_eq!(bolt_and_copy(catalog::young_pyromancer(), "Elemental"), 2);
}

/// CR 603.2 — "Whenever THIS creature becomes the target of a spell or
/// ability an opponent controls" (Mossdog): an opponent's Giant Growth on
/// another of your creatures doesn't grow it; one on Mossdog does.
#[test]
fn cr_603_2_mossdog_counts_only_itself_being_targeted() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let dog = g.add_card_to_battlefield(0, catalog::mossdog());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let counters = |g: &GameState| g.battlefield_find(dog).unwrap().counter_count(CounterType::PlusOnePlusOne);
    let growth_on = |g: &mut GameState, target| {
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.step = TurnStep::PreCombatMain;
        let spell = g.add_card_to_hand(1, catalog::giant_growth());
        g.players[1].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(target)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("Giant Growth");
        drain_stack(g);
    };
    growth_on(&mut g, bear);
    assert_eq!(counters(&g), 0, "another creature was targeted");
    growth_on(&mut g, dog);
    assert_eq!(counters(&g), 1, "Mossdog itself was targeted");
}

/// CR 305.1 / 601 — "you may CAST spells from among them" (Apex of Power)
/// is no land drop: an exiled Forest is marked cast-only, and playing it is
/// refused, while an exiled spell stays castable.
#[test]
fn cr_305_1_a_cast_permission_does_not_play_a_land() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let forest = g.add_card_to_library(0, catalog::forest());
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    let apex = catalog::apex_of_power();
    g.resolve_effect(&apex.effect, &crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0))
        .expect("resolve");
    let perm = |id| g.exile.iter().find(|c| c.id == id).and_then(|c| c.may_play_until);
    assert!(perm(forest).is_some_and(|p| p.cast_only), "the Forest is cast-only");
    assert!(perm(bolt).is_some(), "the Bolt is castable");
    assert!(g.perform_action(GameAction::PlayLand(forest)).is_err(), "no land drop from a cast permission");
}

/// CR 500.4 — mana kept "as steps and phases end" (Savage Ventmaw) survives a step
/// only while unspent: three kept, two spent, one crosses the step (the
/// whole three used to be re-seeded at every step change), and none of it
/// comes back after being spent too.
#[test]
fn cr_500_4_kept_mana_is_not_restored_once_spent() {
    use crabomination::effect::{Effect, PlayerRef, Value};
    use crabomination::mana::{cost, generic, Color};
    let mut g = two_player_game();
    let keep = Effect::AddManaKeptThisTurnCount { who: PlayerRef::You, color: Color::Green, amount: Value::Const(3) };
    g.resolve_effect(&keep, &crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0)).unwrap();
    g.players[0].mana_pool.pay(&cost(&[generic(2)])).expect("pay two");
    g.empty_mana_pools();
    assert_eq!(g.players[0].mana_pool.total(), 1, "only the unspent one kept");
    g.players[0].mana_pool.pay(&cost(&[generic(1)])).expect("pay the last");
    g.empty_mana_pools();
    assert_eq!(g.players[0].mana_pool.total(), 0);
}

/// Klauth's kept mana is "only to cast spells" (`SpendRestriction::SpellsOnly`):
/// it floats as restricted mana, and an unspent pip survives the step.
#[test]
fn cr_106_6_klauth_mana_is_for_spells_and_kept() {
    use crabomination::effect::{Effect, PlayerRef, Value};
    let mut g = two_player_game();
    let keep = Effect::AddManaKeptThisTurnAnyColors { who: PlayerRef::You, amount: Value::Const(2) };
    g.resolve_effect(&keep, &crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0)).unwrap();
    assert_eq!((g.players[0].mana_pool.total(), g.players[0].mana_pool.restricted_total()), (0, 2));
    g.empty_mana_pools();
    assert_eq!(g.players[0].mana_pool.restricted_total(), 2, "kept across the step");
}

/// CR 614.9 — Heroic Sacrifice redirects damage to "you and CREATURES you
/// control": a Bolt at you goes to the chosen creature, a Bolt at your
/// planeswalker doesn't (Gideon's Sacrifice's "permanents" would take it).
#[test]
fn cr_614_9_heroic_sacrifice_covers_you_and_creatures_only() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let pw = g.add_card_to_battlefield(0, catalog::professor_onyx());
    let hs = catalog::heroic_sacrifice();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(wurm)), 0, 0);
    g.resolve_effect(&hs.effect, &ctx).expect("resolve");
    let bolt_at = |g: &mut GameState, target| {
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.step = TurnStep::PreCombatMain;
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(target), additional_targets: vec![], mode: None, x_value: None })
            .expect("Bolt");
        drain_stack(g);
    };
    let (life, loyalty) = (g.players[0].life, g.battlefield_find(pw).unwrap().counter_count(CounterType::Loyalty));
    bolt_at(&mut g, Target::Player(0));
    assert_eq!(g.players[0].life, life, "redirected to the Wurm");
    assert_eq!(g.battlefield_find(wurm).unwrap().damage, 3);
    bolt_at(&mut g, Target::Permanent(pw));
    assert_eq!(g.battlefield_find(pw).unwrap().counter_count(CounterType::Loyalty), loyalty - 3, "a planeswalker isn't covered");
}

/// CR 106.6 — "Spend this mana only to …" riders are restricted mana:
/// Sliver Hive's colored mana (Sliver spells) and Cultivator Drone's {C}
/// (colorless spells and abilities) float restricted, not free.
#[test]
fn cr_106_6_printed_spend_restrictions_float_restricted() {
    for (def, ability) in [(catalog::sliver_hive(), 1usize), (catalog::cultivator_drone(), 0)] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        g.clear_sickness(id);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: ability,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("tap for mana");
        assert_eq!(g.players[0].mana_pool.restricted_total(), 1);
        assert_eq!(g.players[0].mana_pool.total(), 0);
    }
}

/// Vampiric Embrace — "Whenever a creature dealt damage by enchanted creature
/// this turn dies, put a +1/+1 counter on that creature": the enchanted Wurm
/// kills a blocking Hawk and grows (the trigger was missing).
#[test]
fn vampiric_embrace_grows_the_host_off_a_kill() {
    use crabomination::card::CounterType;
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let aura = g.add_card_to_battlefield(0, catalog::vampiric_embrace());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(wurm);
    // A flyer: the Embrace gives the Wurm flying.
    let bear = g.add_card_to_battlefield(1, catalog::suntail_hawk());
    g.clear_sickness(wurm);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: wurm, target: AttackTarget::Player(1) }]).expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(bear, wurm)])).expect("block");
    for _ in 0..30 {
        if g.step == TurnStep::EndCombat || g.step == TurnStep::PostCombatMain {
            break;
        }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "the Hawk died");
    assert_eq!(g.battlefield_find(wurm).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// CR 602.5b — "Equip {0}. Activate only once each turn." (Leather Armor)
/// and Dark Knight's Greatsword's "Equip—Pay 3 life. Activate only once each
/// turn.": the second equip in a turn is refused, and the Greatsword's equip
/// costs life, not mana.
#[test]
fn equip_once_each_turn() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let armor = g.add_card_to_battlefield(0, catalog::leather_armor());
    g.perform_action(GameAction::Equip { equipment: armor, target: a }).expect("first equip");
    assert!(g.perform_action(GameAction::Equip { equipment: armor, target: b }).is_err());
    assert_eq!(g.battlefield_find(armor).unwrap().attached_to, Some(a));

    let sword = g.add_card_to_battlefield(0, catalog::dark_knights_greatsword());
    let life = g.players[0].life;
    g.perform_action(GameAction::Equip { equipment: sword, target: b }).expect("pay 3 life");
    assert_eq!(g.players[0].life, life - 3);
    assert!(g.perform_action(GameAction::Equip { equipment: sword, target: a }).is_err());
}

/// CR 602.5b — Luxurious Locomotive's "Crew 1. Activate only once each
/// turn." caps the printed crew only: a second crew is refused, but Kotori's
/// granted crew 2 still animates it.
#[test]
fn crew_once_each_turn_caps_the_printed_crew_only() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let loco = g.add_card_to_battlefield(0, catalog::luxurious_locomotive());
    let c1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::Crew { vehicle: loco, crew_creatures: vec![c1] }).expect("crew");
    assert!(g.perform_action(GameAction::Crew { vehicle: loco, crew_creatures: vec![c2] }).is_err());
    g.add_card_to_battlefield(0, catalog::kotori_pilot_prodigy());
    g.perform_action(GameAction::Crew { vehicle: loco, crew_creatures: vec![c2] })
        .expect("Kotori's crew 2 is a separate ability");
}

/// CR 602.5b — Phyrexian Battleflies' "{B}: +1/+0. Activate no more than
/// twice each turn." pumps twice and refuses a third (it was an uncapped copy
/// plus a once-per-turn copy, so it pumped without limit).
#[test]
fn phyrexian_battleflies_pumps_at_most_twice() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let flies = g.add_card_to_battlefield(0, catalog::phyrexian_battleflies());
    assert_eq!(g.battlefield_find(flies).unwrap().definition.activated_abilities.len(), 1);
    g.players[0].mana_pool.add(Color::Black, 3);
    let pump = GameAction::ActivateAbility {
        card_id: flies, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None, mode: None,
    };
    for _ in 0..2 {
        g.perform_action(pump.clone()).expect("pump");
        drain_stack(&mut g);
    }
    assert!(g.perform_action(pump).is_err(), "a third pump");
    assert_eq!(g.computed_permanent(flies).unwrap().power, 2);
}

/// CR 602.2b / 602.5b — "Remove a counter from this" is a COST, paid as the
/// ability is activated: Spike Feeder with two +1/+1 counters can gain life
/// twice while both activations wait on the stack, and not a third time
/// (the removal used to happen on resolution, behind a condition that read
/// the counters still there — so a one-counter creature activated forever).
#[test]
fn a_remove_counter_cost_is_paid_on_activation() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let feeder = g.add_card_to_battlefield(0, catalog::spike_feeder());
    g.battlefield_find_mut(feeder).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let gain = GameAction::ActivateAbility {
        card_id: feeder, ability_index: 1, target: None,
        additional_targets: Vec::new(), x_value: None, mode: None,
    };
    g.perform_action(gain.clone()).expect("first");
    assert_eq!(g.battlefield_find(feeder).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    g.priority.player_with_priority = 0;
    g.perform_action(gain.clone()).expect("second, holding priority");
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(gain).is_err(), "no counter left to pay with");
    let life = g.players[0].life;
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 4);
}

/// CR 702.16b / 608.2b — protection from a colour granted in response makes
/// the spell's target illegal as it resolves: Mother of Runes answers a
/// Murder aimed at a Bear, the Murder fizzles into the graveyard and the Bear
/// lives. (The resolution re-check read Shroud and Hexproof, never
/// protection, so the Bear died with protection from black; the colour pick
/// also named the densest opposing colour rather than the Murder's.)
#[test]
fn protection_granted_in_response_fizzles_the_spell() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    let mother = g.add_card_to_battlefield(0, catalog::mother_of_runes());
    g.clear_sickness(mother);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // A red board on the other side: the densest opposing colour isn't black.
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::goblin_guide());
    }
    let murder = g.add_card_to_hand(1, catalog::murder());
    g.players[1].mana_pool.add(Color::Black, 3);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: murder, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Murder");
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: mother, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("Mother of Runes");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "Murder fizzled");
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Protection(Color::Black)));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == murder));
}

/// CR 608.2b / 702.11b / 702.16b — an ability's sole target that gains
/// hexproof or protection from its source in response is illegal as the
/// ability resolves: Prodigal Sorcerer's ping at a Bear does nothing after
/// Blossoming Defense, or after Mother of Runes names blue. (The ability
/// path re-read only the target filter.)
#[test]
fn an_ability_fizzles_when_its_target_gains_hexproof_or_protection() {
    for answer in ["hexproof", "protection"] {
        let mut g = two_player_game();
        g.active_player_idx = 1;
        g.step = TurnStep::PreCombatMain;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let sorcerer = g.add_card_to_battlefield(1, catalog::prodigal_sorcerer());
        g.clear_sickness(sorcerer);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::ActivateAbility {
            card_id: sorcerer, ability_index: 0, target: Some(Target::Permanent(bear)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        })
        .expect("ping");
        g.priority.player_with_priority = 0;
        if answer == "hexproof" {
            let defense = g.add_card_to_hand(0, catalog::blossoming_defense());
            g.players[0].mana_pool.add(Color::Green, 1);
            g.perform_action(GameAction::CastSpell {
                card_id: defense, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
            })
            .expect("Blossoming Defense");
        } else {
            let mother = g.add_card_to_battlefield(0, catalog::mother_of_runes());
            g.clear_sickness(mother);
            g.perform_action(GameAction::ActivateAbility {
                card_id: mother, ability_index: 0, target: Some(Target::Permanent(bear)),
                additional_targets: Vec::new(), x_value: None, mode: None,
            })
            .expect("Mother of Runes");
        }
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(bear).map(|c| c.damage), Some(0), "{answer}: the ping fizzled");
    }
}

/// CR 608.2b / 702.16b — an Aura spell whose target gains protection from its
/// colour in response doesn't resolve: Pacifism at a Bear, Gods Willing
/// naming white, and the Pacifism goes to the graveyard instead of the
/// battlefield.
#[test]
fn an_aura_spell_fizzles_on_protection_gained_in_response() {
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pacifism = g.add_card_to_hand(1, catalog::pacifism());
    g.players[1].mana_pool.add(Color::White, 2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: pacifism, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Pacifism");
    g.priority.player_with_priority = 0;
    let willing = g.add_card_to_hand(0, catalog::gods_willing());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: willing, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Gods Willing");
    let events = drain_stack(&mut g);
    // Not merely swept by CR 704.5m afterwards: it never entered.
    assert!(!events.iter().any(|e| matches!(e, GameEvent::PermanentEntered { card_id } if *card_id == pacifism)));
    assert!(g.battlefield_find(pacifism).is_none());
    assert!(g.players[1].graveyard.iter().any(|c| c.id == pacifism));
}

/// CR 702.16b — protection stops spells of that colour from targeting the
/// permanent whoever controls them: its own controller can't aim a white
/// Gods Willing at a Bear that already has protection from white. (Unlike
/// hexproof, CR 702.11b, there is no "your opponents" clause; the cast gate
/// used to skip the caster's own permanents.)
#[test]
fn protection_stops_your_own_spells_too() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let mut bear = catalog::grizzly_bears();
    bear.keywords.push(Keyword::Protection(Color::White));
    let bear = g.add_card_to_battlefield(0, bear);
    let willing = g.add_card_to_hand(0, catalog::gods_willing());
    g.players[0].mana_pool.add(Color::White, 1);
    assert!(g
        .perform_action(GameAction::CastSpell {
            card_id: willing, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        })
        .is_err());
}

/// CR 702.11e / 608.2b — hexproof from a colour gained in response makes the
/// spell's target illegal as it resolves: a Murder at a Bear, and a Sign in
/// Blood at its controller, both fizzle after Veil of Summer ("you and
/// permanents you control gain hexproof from blue and from black"). The
/// colour-hexproof check ran at cast only.
#[test]
fn veil_of_summer_in_response_fizzles_the_spell() {
    for at_player in [false, true] {
        let mut g = two_player_game();
        g.active_player_idx = 1;
        g.step = TurnStep::PreCombatMain;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::forest());
        }
        let (spell, target) = if at_player {
            (g.add_card_to_hand(1, catalog::sign_in_blood()), Target::Player(0))
        } else {
            (g.add_card_to_hand(1, catalog::murder()), Target::Permanent(bear))
        };
        g.players[1].mana_pool.add(Color::Black, 3);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(target), additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast");
        g.priority.player_with_priority = 0;
        let veil = g.add_card_to_hand(0, catalog::veil_of_summer());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: veil, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("Veil of Summer");
        let life = g.players[0].life;
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_some(), "at_player {at_player}");
        assert_eq!(g.players[0].life, life, "at_player {at_player}");
        // Veil's own draw only (an opponent cast a black spell this turn),
        // not Sign in Blood's two.
        assert_eq!(g.players[0].library.len(), 2, "at_player {at_player}");
    }
}

/// CR 702.16b — protection blocks every target slot of a spell, not only the
/// first: Reckless Spite ("destroy two target nonblack creatures") can't name
/// a creature with protection from black second. (The cast gate read slot 0
/// only.)
#[test]
fn protection_blocks_a_later_target_slot() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::PreCombatMain;
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut warded = catalog::grizzly_bears();
    warded.keywords.push(Keyword::Protection(Color::Black));
    let b = g.add_card_to_battlefield(0, warded);
    let spite = g.add_card_to_hand(1, catalog::reckless_spite());
    g.players[1].mana_pool.add(Color::Black, 3);
    g.players[1].mana_pool.add_colorless(1);
    let cast = GameAction::CastSpell {
        card_id: spite,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    };
    assert!(g.perform_action(cast).is_err());
    assert!(g.players[1].hand.iter().any(|c| c.id == spite), "back in hand");
}
