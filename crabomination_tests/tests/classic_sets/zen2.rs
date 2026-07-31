//! Zendikar gap closure — Traps, Allies, landfall and kicker.

use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
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

fn cast_alt(g: &mut GameState, id: CardId, target: Option<Target>) -> bool {
    let ok = g
        .perform_action(GameAction::CastSpellAlternative {
            card_id: id,
            pitch_card: None,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_ok();
    drain_stack(g);
    ok
}

/// Put `n` of seat 1's bears into the attack, and return their ids.
fn attackers(g: &mut GameState, n: usize) -> Vec<CardId> {
    let ids: Vec<CardId> =
        (0..n).map(|_| g.add_card_to_battlefield(1, catalog::grizzly_bears())).collect();
    g.attacking =
        ids.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(0) }).collect();
    ids
}

/// Yes to every "you may" the batch's Rally / landfall triggers ask.
fn always_yes(g: &mut GameState) {
    g.decider = Box::new(ScriptedDecider::new(
        std::iter::repeat_with(|| DecisionAnswer::Bool(true)).take(8),
    ));
}

// ── Traps ───────────────────────────────────────────────────────────────────

/// The board-state Traps only unlock their cheap cost once their gate is met.
#[test]
fn zen_trap_gates() {
    // Arrow Volley Trap wants four attackers.
    let volley = |n: usize| {
        let mut g = two_player_game();
        attackers(&mut g, n);
        let trap = g.add_card_to_hand(0, catalog::arrow_volley_trap());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast_alt(&mut g, trap, None)
    };
    assert!(!volley(3), "three attackers isn't enough");
    assert!(volley(4), "four unlocks the cheap white cost");

    // Pitfall Trap wants exactly one.
    let pitfall = |n: usize| {
        let mut g = two_player_game();
        let ids = attackers(&mut g, n);
        let trap = g.add_card_to_hand(0, catalog::pitfall_trap());
        g.players[0].mana_pool.add(Color::White, 1);
        cast_alt(&mut g, trap, Some(Target::Permanent(ids[0])))
    };
    assert!(pitfall(1), "exactly one attacker");
    assert!(!pitfall(2), "two is too many");

    // Lethargy Trap wants three.
    let mut g = two_player_game();
    let ids = attackers(&mut g, 3);
    let trap = g.add_card_to_hand(0, catalog::lethargy_trap());
    g.players[0].mana_pool.add(Color::Blue, 1);
    assert!(cast_alt(&mut g, trap, None));
    assert_eq!(g.computed_permanent(ids[0]).unwrap().power, -1, "2/2 minus 3/0");
}

/// Inferno Trap unlocks once two creatures have connected with you.
#[test]
fn inferno_trap_counts_the_creatures_that_hit_you() {
    let try_alt = |hits: usize| {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        for _ in 0..hits {
            let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
            g.players[0].creatures_that_damaged_me_this_turn.push(a);
        }
        let trap = g.add_card_to_hand(0, catalog::inferno_trap());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast_alt(&mut g, trap, Some(Target::Permanent(victim)))
    };
    assert!(!try_alt(1), "one attacker isn't enough");
    assert!(try_alt(2), "two unlocks the cheap red cost");
}

/// Lavaball Trap unlocks once an opponent has landed two lands this turn.
#[test]
fn lavaball_trap_watches_land_drops() {
    let try_alt = |lands: u32| {
        let mut g = two_player_game();
        g.players[1].lands_entered_this_turn = lands;
        let a = g.add_card_to_battlefield(1, catalog::mountain());
        let b = g.add_card_to_battlefield(1, catalog::mountain());
        let trap = g.add_card_to_hand(0, catalog::lavaball_trap());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpellAlternative {
            card_id: trap,
            pitch_card: None,
            target: Some(Target::Permanent(a)),
            additional_targets: vec![Target::Permanent(b)],
            mode: None,
            x_value: None,
        })
        .is_ok()
    };
    assert!(!try_alt(1), "one land isn't enough");
    assert!(try_alt(2), "two unlocks the cheap red cost");
}

/// Mindbreak Trap is free once an opponent has cast three spells, and exiles
/// the spell rather than countering it into the graveyard.
#[test]
fn mindbreak_trap_exiles_the_spell() {
    let mut g = two_player_game();
    g.players[1].spells_cast_this_turn = 3;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    let trap = g.add_card_to_hand(0, catalog::mindbreak_trap());
    g.priority.player_with_priority = 0;
    assert!(cast_alt(&mut g, trap, Some(Target::Permanent(bolt))), "free with three spells cast");
    assert!(g.exile.iter().any(|c| c.id == bolt), "the Bolt is exiled, not countered to the yard");
    assert_eq!(g.players[0].life, 20);
}

/// Baloth Cage Trap and Needlebite Trap read their opponent-side gates.
#[test]
fn zen_opponent_gated_traps() {
    let mut g = two_player_game();
    g.players[1].artifacts_entered_this_turn = 1;
    let trap = g.add_card_to_hand(0, catalog::baloth_cage_trap());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(cast_alt(&mut g, trap, None), "their artifact turned it on");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Beast"), "a 4/4 Beast");

    let mut g = two_player_game();
    g.players[1].life_gained_this_turn = 1;
    let trap = g.add_card_to_hand(0, catalog::needlebite_trap());
    g.players[0].mana_pool.add(Color::Black, 1);
    assert!(cast_alt(&mut g, trap, Some(Target::Player(1))), "their life gain turned it on");
    assert_eq!((g.players[0].life, g.players[1].life), (25, 15));
}

// ── Allies ──────────────────────────────────────────────────────────────────

/// The Rally counter-growers grow off each Ally entering, including their own.
#[test]
fn zen_rally_counters() {
    let mut g = two_player_game();
    always_yes(&mut g);
    let blade = g.add_card_to_battlefield(0, catalog::kazandu_blademaster());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: blade }]);
    drain_stack(&mut g);
    let sell = g.add_card_to_battlefield(0, catalog::nimana_sell_sword());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: sell }]);
    drain_stack(&mut g);
    let n = |id| g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(n(blade), 2, "its own ETB plus the Sell-Sword's");
    assert_eq!(n(sell), 1);
}

/// Kazuul Warlord's Rally grows every Ally at once.
#[test]
fn kazuul_warlord_grows_the_team() {
    let mut g = two_player_game();
    always_yes(&mut g);
    // A Rally Ally without its own counter trigger, so the count is the
    // Warlord's alone.
    let other = g.add_card_to_battlefield(0, catalog::joraga_bard());
    let lord = g.add_card_to_battlefield(0, catalog::kazuul_warlord());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: lord }]);
    drain_stack(&mut g);
    for id in [lord, other] {
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }
}

/// Murasa Pyromancer and Hagra Diabolist scale off your Ally count.
#[test]
fn zen_ally_count_payoffs() {
    let mut g = two_player_game();
    always_yes(&mut g);
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.add_card_to_battlefield(0, catalog::nimana_sell_sword());
    let pyro = g.add_card_to_battlefield(0, catalog::murasa_pyromancer());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: pyro }]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(victim).map(|c| c.damage), Some(2), "two Allies, two damage");

    let mut g = two_player_game();
    always_yes(&mut g);
    g.add_card_to_battlefield(0, catalog::nimana_sell_sword());
    let dia = g.add_card_to_battlefield(0, catalog::hagra_diabolist());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: dia }]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "two Allies, two life");
}

/// Highland Berserker's Rally hands first strike to your Allies.
#[test]
fn highland_berserker_grants_first_strike() {
    let mut g = two_player_game();
    always_yes(&mut g);
    let other = g.add_card_to_battlefield(0, catalog::nimana_sell_sword());
    let zerk = g.add_card_to_battlefield(0, catalog::highland_berserker());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: zerk }]);
    drain_stack(&mut g);
    assert!(g.computed_permanent(other).unwrap().keywords.contains(&Keyword::FirstStrike));
}

// ── Landfall / kicker ───────────────────────────────────────────────────────

/// Ob Nixilis drains for 3 and grows by three counters on a land drop.
#[test]
fn ob_nixilis_the_fallen_on_landfall() {
    let mut g = two_player_game();
    always_yes(&mut g);
    let ob = g.add_card_to_battlefield(0, catalog::ob_nixilis_the_fallen());
    let land = g.add_card_to_hand(0, catalog::swamp());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(land)).expect("play");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
    assert_eq!(g.battlefield_find(ob).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}

/// Ior Ruin Expedition banks quest counters and cashes three in for two cards.
#[test]
fn ior_ruin_expedition_cashes_out() {
    let mut g = two_player_game();
    always_yes(&mut g);
    let exp = g.add_card_to_battlefield(0, catalog::ior_ruin_expedition());
    g.battlefield_find_mut(exp).unwrap().add_counters(CounterType::Quest, 3);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let before = g.players[0].hand.len();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: exp,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("cash out");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 2);
    assert!(g.battlefield_find(exp).is_none(), "and it sacrificed itself");
}

/// The ZEN kicker creatures only fire their rider when kicked.
#[test]
fn zen_kicked_etb_riders() {
    let ruinblaster = |kicked: bool| {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(1, catalog::akoum_refuge());
        let goblin = g.add_card_to_hand(0, catalog::goblin_ruinblaster());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(if kicked {
            GameAction::CastSpellKicked {
                card_id: goblin,
                target: Some(Target::Permanent(land)),
                additional_targets: vec![],
                mode: None,
                x_value: None,
            }
        } else {
            GameAction::CastSpell {
                card_id: goblin,
                target: None,
                additional_targets: vec![],
                mode: None,
                x_value: None,
            }
        })
        .expect("cast");
        drain_stack(&mut g);
        g.battlefield_find(land).is_none()
    };
    assert!(!ruinblaster(false), "unkicked, the land lives");
    assert!(ruinblaster(true), "kicked, the nonbasic dies");
}

/// Bold Defense pumps +1/+1, or +2/+2 with first strike when kicked.
#[test]
fn bold_defense_scales_with_the_kicker() {
    let run = |kicked: bool| {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::bold_defense());
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(5);
        g.perform_action(if kicked {
            GameAction::CastSpellKicked {
                card_id: spell,
                target: None,
                additional_targets: vec![],
                mode: None,
                x_value: None,
            }
        } else {
            GameAction::CastSpell {
                card_id: spell,
                target: None,
                additional_targets: vec![],
                mode: None,
                x_value: None,
            }
        })
        .expect("cast");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        (cp.power, cp.keywords.contains(&Keyword::FirstStrike))
    };
    assert_eq!(run(false), (3, false));
    assert_eq!(run(true), (4, true));
}

/// Conqueror's Pledge doubles its token count when kicked.
#[test]
fn conquerors_pledge_doubles_when_kicked() {
    let mut g = two_player_game();
    let pledge = g.add_card_to_hand(0, catalog::conquerors_pledge());
    g.players[0].mana_pool.add(Color::White, 3);
    g.players[0].mana_pool.add_colorless(8);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: pledge,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast kicked");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Kor Soldier").count(),
        12
    );
}

// ── Lands and statics ───────────────────────────────────────────────────────

/// The Refuge cycle enters tapped, gains 1 life, and taps for either colour.
#[test]
fn zen_refuges_enter_tapped_and_gain_life() {
    let mut g = two_player_game();
    let land = g.add_card_to_hand(0, catalog::akoum_refuge());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(land)).expect("play");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).unwrap().tapped);
    assert_eq!(g.players[0].life, 21);
    let def = catalog::akoum_refuge();
    assert_eq!(def.activated_abilities.len(), 2, "one ability per colour");
}

/// Armament Master pumps the other Kor by the Equipment on itself.
#[test]
fn armament_master_scales_with_its_own_equipment() {
    let mut g = two_player_game();
    let master = g.add_card_to_battlefield(0, catalog::armament_master());
    let other = g.add_card_to_battlefield(0, catalog::cliff_threader());
    assert_eq!(g.computed_permanent(other).unwrap().power, 2, "no Equipment yet");
    let torch = g.add_card_to_battlefield(0, catalog::blazing_torch());
    g.battlefield_find_mut(torch).unwrap().attached_to = Some(master);
    assert_eq!(g.computed_permanent(other).unwrap().power, 4, "+2/+2 per Equipment");
    assert_eq!(g.computed_permanent(master).unwrap().power, 2, "but not itself");
}

/// Mindless Null can't block until you control a Vampire.
#[test]
fn mindless_null_needs_a_vampire() {
    let mut g = two_player_game();
    let null = g.add_card_to_battlefield(0, catalog::mindless_null());
    assert!(g.computed_permanent(null).unwrap().keywords.contains(&Keyword::CantBlock));
    g.add_card_to_battlefield(0, catalog::vampire_nighthawk());
    assert!(!g.computed_permanent(null).unwrap().keywords.contains(&Keyword::CantBlock));
}

/// Nissa's Chosen goes to the bottom of its owner's library instead of dying.
#[test]
fn nissas_chosen_goes_to_the_library_bottom() {
    let mut g = two_player_game();
    let elf = g.add_card_to_battlefield(0, catalog::nissas_chosen());
    let lib = g.players[0].library.len();
    let mut ev = vec![];
    g.destroy_permanent(elf, false, &mut ev);
    assert!(g.players[0].graveyard.iter().all(|c| c.id != elf), "not in the graveyard");
    assert_eq!(g.players[0].library.len(), lib + 1);
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(elf), "on the bottom");
}

/// Guul Draz Specter swells while an opponent is hellbent.
#[test]
fn guul_draz_specter_grows_on_an_empty_hand() {
    let mut g = two_player_game();
    let spec = g.add_card_to_battlefield(0, catalog::guul_draz_specter());
    g.add_card_to_hand(1, catalog::lightning_bolt());
    assert_eq!(g.computed_permanent(spec).unwrap().power, 2);
    g.players[1].hand.clear();
    assert_eq!(g.computed_permanent(spec).unwrap().power, 5);
}

/// Mire Blight destroys the enchanted creature the moment it takes damage.
#[test]
fn mire_blight_kills_on_any_damage() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
    let blight = g.add_card_to_hand(0, catalog::mire_blight());
    g.players[0].mana_pool.add(Color::Black, 1);
    cast(&mut g, blight, Some(Target::Permanent(victim)));
    let mut ev = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(victim),
        1,
        None,
        &mut ev,
    );
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "one point of damage is lethal");
}

/// Magma Rift's additional cost sacrifices a land.
#[test]
fn magma_rift_eats_a_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::mountain());
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
    let rift = g.add_card_to_hand(0, catalog::magma_rift());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, rift, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(land).is_none(), "the land is sacrificed");
    assert!(g.battlefield_find(victim).is_none(), "5 damage kills the 4/4");
}

/// Feast of Blood is castable only with two Vampires out.
#[test]
fn feast_of_blood_needs_two_vampires() {
    let try_cast = |vampires: usize| {
        let mut g = two_player_game();
        for _ in 0..vampires {
            g.add_card_to_battlefield(0, catalog::vampire_nighthawk());
        }
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let feast = g.add_card_to_hand(0, catalog::feast_of_blood());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: feast,
            target: Some(Target::Permanent(victim)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_ok()
    };
    assert!(!try_cast(1), "one Vampire isn't enough");
    assert!(try_cast(2), "two is");
}

/// The vanilla-ish ZEN gap creatures ship with their printed stats and keywords.
#[test]
fn zen2_stat_lines() {
    let table: &[(crabomination::card::CardDefinition, i32, i32, Option<Keyword>)] = &[
        (catalog::bladetusk_boar(), 3, 2, Some(Keyword::Intimidate)),
        (catalog::caravan_hurda(), 1, 5, Some(Keyword::Lifelink)),
        (catalog::giant_scorpion(), 1, 3, Some(Keyword::Deathtouch)),
        (catalog::bog_tatters(), 4, 2, None),
        (catalog::hedron_scrabbler(), 1, 1, None),
        (catalog::molten_ravager(), 0, 4, None),
        (catalog::joraga_bard(), 1, 4, None),
        (catalog::makindi_shieldmate(), 0, 3, Some(Keyword::Defender)),
    ];
    for (def, p, t, kw) in table {
        assert_eq!((def.power, def.toughness), (*p, *t), "{}", def.name);
        if let Some(k) = kw {
            assert!(def.keywords.contains(k), "{} has {k:?}", def.name);
        }
    }
    assert!(catalog::hedron_scrabbler().card_types.contains(&CardType::Artifact));
}
