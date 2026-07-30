//! Champions of Kamigawa gap wave — divinity counters, the slow duals, Konda's
//! Banner's shared-trait anthems, and the legends.

use crabomination::card::{CounterType, Keyword};
use crabomination::game::types::{Attack, AttackTarget};
use crabomination::catalog;
use crabomination::game::two_player_game;
use crabomination::game::*;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Activate ability `idx` on `id` and drain the stack.
macro_rules! act {
    ($g:ident, $id:expr, $idx:expr, $tgt:expr) => {{
        $g.perform_action(GameAction::ActivateAbility {
            card_id: $id,
            ability_index: $idx,
            target: $tgt,
            additional_targets: Vec::new(),
            x_value: None,
        })
        .expect("activate ability");
        drain_stack(&mut $g);
    }};
}

// ── Printed stats ────────────────────────────────────────────────────────────

#[test]
fn chk2_printed_stats_and_keywords() {
    let rows = [
        ("Azami, Lady of Scrolls", catalog::azami_lady_of_scrolls(), (0, 2), vec![]),
        ("Takeno, Samurai General", catalog::takeno_samurai_general(), (3, 3), vec![Keyword::Bushido(2)]),
        ("The Unspeakable", catalog::the_unspeakable(), (6, 7), vec![Keyword::Flying, Keyword::Trample]),
        ("Myojin of Life's Web", catalog::myojin_of_lifes_web(), (8, 8), vec![]),
        ("Shimatsu the Bloodcloaked", catalog::shimatsu_the_bloodcloaked(), (0, 0), vec![]),
        ("Ragged Veins", catalog::ragged_veins(), (0, 0), vec![Keyword::Flash]),
    ];
    for (name, d, pt, kws) in rows {
        assert_eq!((d.power, d.toughness), pt, "{name} P/T");
        for k in kws {
            assert!(d.keywords.contains(&k), "{name} missing a printed keyword");
        }
    }
    assert!(
        catalog::candles_glow()
            .keywords
            .iter()
            .any(|k| matches!(k, Keyword::Splice(..))),
        "Candles' Glow splices"
    );
}

// ── The Myojin cycle ─────────────────────────────────────────────────────────

/// CR 122.1 — a Myojin cast from hand enters with a divinity counter and is
/// indestructible; the counter pays for its one-shot.
#[test]
fn myojin_enters_with_divinity_only_when_cast_from_hand() {
    use crabomination::mana::Color::*;
    let mut g = two_player_game();
    let m = g.add_card_to_hand(0, catalog::myojin_of_cleansing_fire());
    g.players[0].mana_pool.add(White, 3);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: m,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Myojin");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(m).unwrap().counter_count(CounterType::Divinity),
        1,
        "cast from hand → one divinity counter"
    );
    assert!(
        g.computed_permanent(m)
            .unwrap()
            .keywords
            .contains(&Keyword::Indestructible),
        "indestructible while the counter is there"
    );

    // Reanimated (not cast from hand) — no counter, no indestructible.
    let mut g2 = two_player_game();
    let r = g2.add_card_to_battlefield(0, catalog::myojin_of_cleansing_fire());
    assert_eq!(
        g2.battlefield_find(r).unwrap().counter_count(CounterType::Divinity),
        0
    );
}

/// Removing the divinity counter fires the wrath and drops indestructible.
#[test]
fn myojin_removes_its_counter_to_wrath() {
    let mut g = two_player_game();
    let m = g.add_card_to_battlefield(0, catalog::myojin_of_cleansing_fire());
    g.battlefield_find_mut(m).unwrap().add_counters(CounterType::Divinity, 1);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    act!(g, m, 0, None);
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "all other creatures destroyed");
    assert!(g.battlefield_find(m).is_some(), "Myojin spared itself");
    assert!(
        !g.computed_permanent(m).unwrap().keywords.contains(&Keyword::Indestructible),
        "counter spent → no more indestructible"
    );
}

// ── Lands ────────────────────────────────────────────────────────────────────

/// CR 605.1a / 502.3 — the coloured tap is still a mana ability, and it keeps
/// the land down through the next untap step.
#[test]
fn slow_dual_colored_tap_skips_the_next_untap() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::waterveil_cavern());
    act!(g, land, 1, None);
    assert_eq!(g.players[0].mana_pool.total(), 1, "one coloured mana");
    assert!(g.battlefield_find(land).unwrap().tapped);
    assert!(g.battlefield_find(land).unwrap().skip_next_untap);
    g.do_untap();
    assert!(g.battlefield_find(land).unwrap().tapped, "stayed down this untap");
    g.do_untap();
    assert!(!g.battlefield_find(land).unwrap().tapped, "untaps the turn after");
}

/// Untaidake's mana is legendary-only.
#[test]
fn untaidake_mana_only_pays_for_legendary_spells() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::untaidake_the_cloud_keeper());
    g.battlefield_find_mut(land).unwrap().tapped = false;
    act!(g, land, 0, None);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bear,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "nonlegendary spell can't spend it"
    );
}

/// Forbidden Orchard gifts a Spirit whenever it is tapped for mana.
#[test]
fn forbidden_orchard_gifts_a_spirit_on_tap() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forbidden_orchard());
    act!(g, land, 0, None);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 1 && c.is_token).count(),
        1,
        "the opponent got the Spirit"
    );
}

// ── Statics ──────────────────────────────────────────────────────────────────

/// CR 601 — Dosan stops everyone (both seats) from casting off-turn.
#[test]
fn dosan_locks_off_turn_casts_for_both_seats() {
    use crabomination::mana::Color::*;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dosan_the_falling_leaf());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Red, 1);
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
        "opponent can't cast on your turn"
    );
}

/// CR 502.3 — Imi Statue caps each player at one artifact untap per step.
#[test]
fn imi_statue_caps_artifact_untaps_at_one() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::imi_statue());
    let a = g.add_card_to_battlefield(0, catalog::sol_ring());
    let b = g.add_card_to_battlefield(0, catalog::sol_ring());
    for id in [a, b] {
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    g.do_untap();
    let still_tapped = [a, b]
        .iter()
        .filter(|id| g.battlefield_find(**id).unwrap().tapped)
        .count();
    assert_eq!(still_tapped, 1, "exactly one artifact stayed tapped");
}

/// Night of Souls' Betrayal shrinks every creature, both sides.
#[test]
fn night_of_souls_betrayal_shrinks_all_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::night_of_souls_betrayal());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for id in [mine, theirs] {
        let cp = g.computed_permanent(id).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 1));
    }
}

/// Takeno pumps each other Samurai by its own bushido rating.
#[test]
fn takeno_pumps_each_samurai_by_its_bushido() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::takeno_samurai_general());
    let one = g.add_card_to_battlefield(0, catalog::devoted_retainer()); // bushido 1
    let two = g.add_card_to_battlefield(0, catalog::samurai_enforcers()); // bushido 2
    assert_eq!(
        {
            let cp = g.computed_permanent(one).unwrap();
            (cp.power, cp.toughness)
        },
        (2, 2),
        "1/1 + bushido 1"
    );
    assert_eq!(
        {
            let cp = g.computed_permanent(two).unwrap();
            (cp.power, cp.toughness)
        },
        (6, 6),
        "4/4 + bushido 2"
    );
}

// ── Equipment ────────────────────────────────────────────────────────────────

/// CR 301.5c — Konda's Banner attaches only to a legendary creature, and pumps
/// every creature sharing the host's colour or a creature type.
#[test]
fn kondas_banner_only_equips_legends_and_shares_traits() {
    use crabomination::mana::Color::*;
    let mut g = two_player_game();
    let banner = g.add_card_to_battlefield(0, catalog::kondas_banner());
    let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green Bear
    let konda = g.add_card_to_battlefield(0, catalog::konda_lord_of_eiganjo()); // legendary white Human Samurai
    g.players[0].mana_pool.add_colorless(2);
    assert!(
        g.perform_action(GameAction::Equip { equipment: banner, target: plain }).is_err(),
        "can't attach to a nonlegendary creature"
    );
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Equip { equipment: banner, target: konda })
        .expect("attaches to a legend");
    let human = g.add_card_to_battlefield(0, catalog::devoted_retainer()); // white Human Samurai
    let bear = g.computed_permanent(plain).unwrap();
    assert_eq!((bear.power, bear.toughness), (2, 2), "green Bear shares nothing");
    let h = g.computed_permanent(human).unwrap();
    // White Human Samurai: +1/+1 from the colour anthem and +1/+1 from the
    // creature-type anthem on a printed 1/1.
    assert_eq!((h.power, h.toughness), (3, 3));
    let _ = White;
}

/// CR 702.6e — Hankyu's granted lines bank aim counters on the Equipment and
/// spend them all as one shot.
#[test]
fn hankyu_banks_aim_counters_on_the_equipment() {
    let mut g = two_player_game();
    let bow = g.add_card_to_battlefield(0, catalog::hankyu());
    let shooter = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(shooter).unwrap().summoning_sick = false;
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::Equip { equipment: bow, target: shooter }).expect("equip");
    let printed = catalog::grizzly_bears().activated_abilities.len();
    act!(g, shooter, printed, None); // granted "{T}: aim counter"
    assert_eq!(
        g.battlefield_find(bow).unwrap().counter_count(CounterType::Aim),
        1,
        "counter landed on Hankyu, not the creature"
    );
    g.battlefield_find_mut(shooter).unwrap().tapped = false;
    act!(g, shooter, printed + 1, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 19, "one aim counter → 1 damage");
    assert_eq!(
        g.battlefield_find(bow).unwrap().counter_count(CounterType::Aim),
        0
    );
}

/// General's Kabuto blanks combat damage aimed at its host.
#[test]
fn generals_kabuto_prevents_combat_damage_to_the_host() {
    let mut g = two_player_game();
    let hat = g.add_card_to_battlefield(0, catalog::generals_kabuto());
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Equip { equipment: hat, target: host }).expect("equip");
    assert!(
        g.permanent_prevents_all_combat_damage_to_self(host),
        "host is fogged"
    );
    assert!(
        g.computed_permanent(host).unwrap().keywords.contains(&Keyword::Shroud),
        "and has shroud"
    );
}

// ── Spells ───────────────────────────────────────────────────────────────────

/// CR 119.7 — Reverse the Sands exchanges the two life totals.
#[test]
fn reverse_the_sands_swaps_life_totals() {
    use crabomination::mana::Color::*;
    let mut g = two_player_game();
    g.players[0].life = 4;
    g.players[1].life = 27;
    let s = g.add_card_to_hand(0, catalog::reverse_the_sands());
    g.players[0].mana_pool.add(White, 2);
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpell {
        card_id: s,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!((g.players[0].life, g.players[1].life), (27, 4));
}

/// Soulblast sacrifices the whole board as a cost and throws its total power.
#[test]
fn soulblast_throws_the_teams_total_power() {
    use crabomination::mana::Color::*;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::soulblast());
    g.players[0].mana_pool.add(Red, 3);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: s,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 16, "2 + 2 power thrown");
    assert!(
        !g.battlefield.iter().any(|c| c.controller == 0 && c.definition.is_creature()),
        "board sacrificed as a cost"
    );
}

/// Candles' Glow soaks 3 damage and refunds the prevented amount as life.
#[test]
fn candles_glow_prevents_and_gains() {
    use crabomination::mana::Color::*;
    let mut g = two_player_game();
    let s = g.add_card_to_hand(0, catalog::candles_glow());
    g.players[0].mana_pool.add(White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: s,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 23, "3 prevented, 3 gained");
}

/// Thoughtbind only answers cheap spells.
#[test]
fn thoughtbind_counters_only_mv_four_or_less() {
    use crabomination::mana::Color::*;
    let mut g = two_player_game();
    let big = g.add_card_to_hand(1, catalog::soulblast()); // instant, mana value 6
    g.players[1].mana_pool.add(Red, 3);
    g.players[1].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: big,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the fatty");
    let bind = g.add_card_to_hand(0, catalog::thoughtbind());
    g.players[0].mana_pool.add(Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bind,
            target: Some(Target::Permanent(big)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "mana value 8 is out of range"
    );
}

/// Blood Speaker returns itself from the graveyard when a Demon lands.
#[test]
fn blood_speaker_returns_when_a_demon_enters() {
    let mut g = two_player_game();
    let bs = g.add_card_to_graveyard(0, catalog::blood_speaker());
    let demon = g.add_card_to_battlefield(0, catalog::shimatsu_the_bloodcloaked());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: demon }]);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bs), "Blood Speaker came back");
}

/// Kusari-Gama splashes the blocker damage across the rest of the defending
/// player's board.
#[test]
fn kusari_gama_splashes_onto_the_other_defenders() {
    let mut g = two_player_game();
    let gama = g.add_card_to_battlefield(0, catalog::kusari_gama());
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bystander = g.add_card_to_battlefield(1, catalog::lantern_kami());
    g.clear_sickness(host);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::Equip { equipment: gama, target: host }).expect("equip");
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: host,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, host)])).expect("block");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(bystander).is_none(), "the 1/1 bystander took 2");
}

/// Oathkeeper exiles its host when the Equipment dies, and reanimates a Samurai
/// host that dies while equipped.
#[test]
fn oathkeeper_reanimates_a_samurai_and_exiles_on_its_own_death() {
    let mut g = two_player_game();
    let sword = g.add_card_to_battlefield(0, catalog::oathkeeper_takenos_daisho());
    let host = g.add_card_to_battlefield(0, catalog::devoted_retainer()); // Samurai
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Equip { equipment: sword, target: host }).expect("equip");
    let cp = g.computed_permanent(host).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 2), "+3/+1");
    let events = g.remove_to_graveyard_with_triggers(host);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Devoted Retainer"),
        "the Samurai came back"
    );
}

/// Shell of the Last Kappa banks the spell it eats and casts it back for free.
#[test]
fn shell_of_the_last_kappa_eats_and_recasts() {
    use crabomination::mana::Color::*;
    let mut g = two_player_game();
    let shell = g.add_card_to_battlefield(0, catalog::shell_of_the_last_kappa());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt at the Shell's controller");
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    act!(g, shell, 0, Some(Target::Permanent(bolt)));
    assert_eq!(g.players[0].life, 20, "the bolt never resolved");
    assert!(
        g.exile.iter().any(|c| c.id == bolt && c.exiled_with == Some(shell)),
        "banked under the Shell"
    );
}

/// Junkyo Bell's pump is paid for with the creature at the next end step.
#[test]
fn junkyo_bell_pumps_then_kills_its_target() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::junkyo_bell());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_step_triggers(crabomination::game::TurnStep::Upkeep);
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 for the lone creature");
    g.fire_step_triggers(crabomination::game::TurnStep::End);
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "sacrificed at end of turn");
}
