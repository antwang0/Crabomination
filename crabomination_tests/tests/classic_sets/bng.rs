//! Functionality tests for Born of the Gods (BNG) — `catalog::sets::bng`.

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

/// Cast `card` from seat 0's hand at `target` with `colorless` generic plus
/// the listed colored mana.
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

/// Stat / keyword lines for the BNG bodies.
#[test]
fn bng_stat_lines() {
    let table: &[(fn() -> crabomination::card::CardDefinition, i32, i32, &[Keyword])] = &[
        (catalog::akroan_skyguard, 1, 1, &[Keyword::Flying]),
        (catalog::chorus_of_the_tides, 3, 2, &[Keyword::Flying]),
        (catalog::cyclops_of_one_eyed_pass, 5, 2, &[]),
        (catalog::deepwater_hypnotist, 2, 1, &[]),
        (catalog::elite_skirmisher, 3, 1, &[]),
        (catalog::forsaken_drifters, 4, 2, &[]),
        (catalog::great_hart, 2, 4, &[]),
        (catalog::griffin_dreamfinder, 1, 4, &[Keyword::Flying]),
        (catalog::impetuous_sunchaser, 1, 1, &[Keyword::Flying, Keyword::Haste, Keyword::MustAttack]),
        (catalog::kragma_butcher, 2, 3, &[]),
        (catalog::loyal_pegasus, 2, 1, &[Keyword::Flying, Keyword::CantAttackOrBlockAlone]),
        (catalog::marshmist_titan, 4, 5, &[]),
        (catalog::nyxborn_eidolon, 2, 1, &[]),
    ];
    for (f, p, t, kws) in table {
        let d = f();
        assert_eq!((d.power, d.toughness), (*p, *t), "{}", d.name);
        for kw in *kws {
            assert!(d.keywords.contains(kw), "{} lacks {:?}", d.name, kw);
        }
    }
}

/// Heroic triggers: a counter for the Skyguard, a scry for the Chorus.
#[test]
fn bng_heroic_payoffs() {
    let mut g = main_phase();
    let sky = g.add_card_to_battlefield(0, catalog::akroan_skyguard());
    cast(&mut g, catalog::mortals_ardor(), Some(Target::Permanent(sky)), 0, &[(Color::White, 1)]);
    assert_eq!(g.battlefield_find(sky).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.computed_permanent(sky).unwrap().keywords.contains(&Keyword::Lifelink));

    let chorus = g.add_card_to_battlefield(0, catalog::chorus_of_the_tides());
    g.add_card_to_library(0, catalog::great_hart());
    cast(
        &mut g,
        catalog::mortals_resolve(),
        Some(Target::Permanent(chorus)),
        1,
        &[(Color::Green, 1)],
    );
    assert!(g.computed_permanent(chorus).unwrap().keywords.contains(&Keyword::Indestructible));
}

/// Inspired fires when the creature untaps: the Butcher pumps itself, the
/// Hypnotist shrinks an opposing creature.
#[test]
fn bng_inspired_triggers_on_untap() {
    let mut g = main_phase();
    let butcher = g.add_card_to_battlefield(0, catalog::kragma_butcher());
    g.battlefield_find_mut(butcher).unwrap().tapped = true;
    for _ in 0..40 {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
        if g.active_player_idx == 0 && !g.battlefield_find(butcher).unwrap().tapped {
            break;
        }
    }
    assert_eq!(g.computed_permanent(butcher).map(|c| c.power), Some(4), "+2/+0 on untap");
}

/// Asphyxiate only hits untapped creatures; Excoriate only tapped ones.
#[test]
fn bng_tap_state_removal() {
    let mut g = main_phase();
    let tapped = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(tapped).unwrap().tapped = true;
    let untapped = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let asphyxiate = g.add_card_to_hand(0, catalog::asphyxiate());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: asphyxiate,
            target: Some(Target::Permanent(tapped)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "a tapped creature is an illegal Asphyxiate target"
    );
    g.perform_action(GameAction::CastSpell {
        card_id: asphyxiate,
        target: Some(Target::Permanent(untapped)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("untapped is legal");
    drain_stack(&mut g);
    assert!(g.battlefield_find(untapped).is_none(), "destroyed");

    cast(&mut g, catalog::excoriate(), Some(Target::Permanent(tapped)), 3, &[(Color::White, 1)]);
    assert!(g.battlefield_find(tapped).is_none(), "the tapped one was exiled");
}

/// Eye Gouge shrinks anything but outright kills a Cyclops.
#[test]
fn eye_gouge_executes_cyclopes() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    cast(&mut g, catalog::eye_gouge(), Some(Target::Permanent(bear)), 0, &[(Color::Black, 1)]);
    assert_eq!(g.computed_permanent(bear).map(|c| (c.power, c.toughness)), Some((1, 1)));

    let cyclops = g.add_card_to_battlefield(1, catalog::cyclops_of_one_eyed_pass());
    cast(&mut g, catalog::eye_gouge(), Some(Target::Permanent(cyclops)), 0, &[(Color::Black, 1)]);
    assert!(g.battlefield_find(cyclops).is_none(), "the Cyclops was destroyed");
}

/// Fall of the Hammer swings your creature's power at another creature.
#[test]
fn fall_of_the_hammer_hits_for_power() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::cyclops_of_one_eyed_pass());
    let theirs = g.add_card_to_battlefield(1, catalog::great_hart());
    let spell = g.add_card_to_hand(0, catalog::fall_of_the_hammer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "5 damage killed the 2/4");
}

/// Bolt of Keranos burns and scries; Hold at Bay soaks the next 7 damage.
#[test]
fn bng_burn_and_prevention() {
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::great_hart());
    let life = g.players[1].life;
    cast(&mut g, catalog::bolt_of_keranos(), Some(Target::Player(1)), 1, &[(Color::Red, 2)]);
    assert_eq!(g.players[1].life, life - 3);

    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    cast(&mut g, catalog::hold_at_bay(), Some(Target::Permanent(bear)), 1, &[(Color::White, 1)]);
    let mut events = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(bear),
        2,
        None,
        &mut events,
    );
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0, "prevented");
}

/// Nullify counters a creature spell.
#[test]
fn nullify_counters_creature_spells() {
    let mut g = main_phase();
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bear on the stack");
    g.priority.player_with_priority = 0;
    let nullify = g.add_card_to_hand(0, catalog::nullify());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: nullify,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("counter it");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "countered");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
}

/// Glimpse the Sun God taps X creatures.
#[test]
fn glimpse_the_sun_god_taps_x() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::glimpse_the_sun_god());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast for X=2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped);
}

/// Marshmist Titan's devotion discount pays for its generic pips.
#[test]
fn marshmist_titan_devotion_discount() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::nyxborn_eidolon());
    }
    let titan = g.add_card_to_hand(0, catalog::marshmist_titan());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: titan,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("three black pips shaved {3} off");
    drain_stack(&mut g);
    assert!(g.battlefield_find(titan).is_some());
}

/// Felhide Brawler can't block without another Minotaur.
#[test]
fn felhide_brawler_needs_a_friend() {
    let mut g = main_phase();
    let brawler = g.add_card_to_battlefield(0, catalog::felhide_brawler());
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(!g.blocker_can_block_attacker(brawler, attacker), "alone it can't block");
    g.add_card_to_battlefield(0, catalog::kragma_butcher());
    assert!(g.blocker_can_block_attacker(brawler, attacker), "another Minotaur unlocks it");
}

/// Forsaken Drifters mills four when it dies; Griffin Dreamfinder rebuys an
/// enchantment.
#[test]
fn bng_graveyard_value() {
    let mut g = main_phase();
    let drifters = g.add_card_to_battlefield(0, catalog::forsaken_drifters());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::great_hart());
    }
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    let murder = g.add_card_to_hand(0, catalog::murder());
    g.perform_action(GameAction::CastSpell {
        card_id: murder,
        target: Some(Target::Permanent(drifters)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("kill it");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), 2, "milled four");

    let aura = g.add_card_to_graveyard(0, catalog::nyxborn_eidolon());
    let griffin = g.add_card_to_hand(0, catalog::griffin_dreamfinder());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: griffin,
        target: Some(Target::Permanent(aura)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the Griffin");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == aura), "enchantment card returned");
}

/// Stat lines for the second BNG wave.
#[test]
fn bng_wave2_stat_lines() {
    let table: &[(fn() -> crabomination::card::CardDefinition, i32, i32)] = &[
        (catalog::akroan_phalanx, 3, 3),
        (catalog::ashioks_adept, 1, 3),
        (catalog::graverobber_spider, 2, 4),
        (catalog::nyxborn_shieldmate, 1, 2),
        (catalog::nyxborn_triton, 2, 3),
        (catalog::nyxborn_wolf, 3, 1),
    ];
    for (f, p, t) in table {
        let d = f();
        assert_eq!((d.power, d.toughness), (*p, *t), "{}", d.name);
        assert!(d.bestow.is_some() || !d.name.starts_with("Nyxborn"), "{} bestows", d.name);
    }
}

/// Ephara's Radiance grants its host a tap-for-life ability.
#[test]
fn granting_auras_hand_over_their_ability() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().summoning_sick = false;
    cast(&mut g, catalog::epharas_radiance(), Some(Target::Permanent(bear)), 0, &[(Color::White, 1)]);
    let life = g.players[0].life;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("granted ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3);
    assert!(g.battlefield_find(bear).unwrap().tapped, "the tap cost hit the host");
}

/// Gorgon's Head hands the equipped creature deathtouch.
#[test]
fn gorgons_head_grants_deathtouch() {
    let mut g = main_phase();
    let head = g.add_card_to_battlefield(0, catalog::gorgons_head());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Equip { equipment: head, target: bear }).expect("equip");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Deathtouch));
}

/// Graverobber Spider scales with the creature cards in your graveyard.
#[test]
fn graverobber_spider_scales_with_the_yard() {
    let mut g = main_phase();
    let spider = g.add_card_to_battlefield(0, catalog::graverobber_spider());
    g.battlefield_find_mut(spider).unwrap().summoning_sick = false;
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::great_hart());
    }
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: spider,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(spider).map(|c| (c.power, c.toughness)), Some((5, 7)));
}

/// Ashiok's Adept's heroic strips a card from each opponent.
#[test]
fn ashioks_adept_heroic_discards() {
    let mut g = main_phase();
    let adept = g.add_card_to_battlefield(0, catalog::ashioks_adept());
    g.add_card_to_hand(1, catalog::great_hart());
    cast(&mut g, catalog::mortals_ardor(), Some(Target::Permanent(adept)), 0, &[(Color::White, 1)]);
    assert!(g.players[1].hand.is_empty(), "the opponent discarded");
}

// ── Wave 3 (bng2) ────────────────────────────────────────────────────────────

/// Stat / keyword lines for the wave-3 bodies.
#[test]
fn bng2_stat_lines() {
    let table: &[(fn() -> crabomination::card::CardDefinition, i32, i32, &[Keyword])] = &[
        (catalog::swordwise_centaur, 3, 2, &[]),
        (catalog::oreskos_sun_guide, 2, 2, &[]),
        (catalog::sphinxs_disciple, 2, 2, &[Keyword::Flying]),
        (catalog::setessan_oathsworn, 1, 1, &[]),
        (catalog::vanguard_of_brimaz, 2, 2, &[Keyword::Vigilance]),
        (catalog::black_oak_of_odunos, 0, 5, &[Keyword::Defender]),
        (catalog::forlorn_pseudamma, 2, 1, &[Keyword::Intimidate]),
        (catalog::pheres_band_raiders, 5, 5, &[]),
        (catalog::eater_of_hope, 6, 4, &[Keyword::Flying]),
        (catalog::silent_sentinel, 4, 6, &[Keyword::Flying]),
        (catalog::fate_unraveler, 3, 4, &[]),
        (catalog::tromokratis, 8, 8, &[Keyword::CantBeBlockedUnlessAllBlock]),
    ];
    for (f, p, t, kws) in table {
        let d = f();
        assert_eq!((d.power, d.toughness), (*p, *t), "{}", d.name);
        for kw in *kws {
            assert!(d.keywords.contains(kw), "{} lacks {:?}", d.name, kw);
        }
    }
}

/// Inspired token payoffs: the Raiders mint a 3/3 Centaur enchantment creature
/// when the {2}{G} is paid.
#[test]
fn pheres_band_raiders_inspired_mints_a_centaur() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    let raiders = g.add_card_to_battlefield(0, catalog::pheres_band_raiders());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ctx = crabomination::game::effects::EffectContext::for_ability(raiders, 0, None);
    g.resolve_effect(&catalog::pheres_band_raiders().triggered_abilities[0].effect, &ctx)
        .expect("inspired body");
    drain_stack(&mut g);
    let token = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Centaur")
        .expect("Centaur token minted");
    assert_eq!((token.definition.power, token.definition.toughness), (3, 3));
    assert!(
        token.definition.card_types.contains(&crabomination::card::CardType::Enchantment),
        "the token is an enchantment creature"
    );
}

/// Fate Unraveler pings the opponent on each of their draws, not on yours.
#[test]
fn fate_unraveler_pings_opponent_draws() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::fate_unraveler());
    g.add_card_to_library(1, catalog::great_hart());
    g.add_card_to_library(0, catalog::great_hart());
    let life = g.players[1].life;
    let mut events = Vec::new();
    g.draw_one(1, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1);
    let mut events = Vec::new();
    g.draw_one(0, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "your own draw doesn't trigger");
}

/// Pain Seer's inspired trade: the top card lands in hand and costs its mana
/// value in life.
#[test]
fn pain_seer_inspired_reveals_for_life() {
    let mut g = main_phase();
    let seer = g.add_card_to_battlefield(0, catalog::pain_seer());
    g.battlefield_find_mut(seer).unwrap().tapped = true;
    g.add_card_to_library(0, catalog::great_hart()); // {3}{W} — MV 4
    let life = g.players[0].life;
    for _ in 0..40 {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
        if g.active_player_idx == 0 && !g.battlefield_find(seer).unwrap().tapped {
            break;
        }
    }
    assert_eq!(g.players[0].life, life - 4, "lost life equal to the revealed mana value");
}

/// Lightning Volley's grant expires at cleanup (the EOT-duration activated
/// ability grant).
#[test]
fn lightning_volley_grant_expires() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().summoning_sick = false;
    cast(&mut g, catalog::lightning_volley(), None, 3, &[(Color::Red, 1)]);
    assert_eq!(g.granted_abilities_for(bear).len(), 1, "the tap-ping is granted");
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1);

    for _ in 0..40 {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
        if g.step == TurnStep::PreCombatMain && g.active_player_idx == 1 {
            break;
        }
    }
    assert!(g.granted_abilities_for(bear).is_empty(), "the grant expired at cleanup");
}

/// Kraken of the Straits: small creatures can't block it once you have enough
/// Islands.
#[test]
fn kraken_of_the_straits_gates_blockers_by_island_count() {
    let mut g = main_phase();
    let kraken = g.add_card_to_battlefield(0, catalog::kraken_of_the_straits());
    g.battlefield_find_mut(kraken).unwrap().summoning_sick = false;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::island());
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::Attack {
        attacker: kraken,
        target: crabomination::game::AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(bear, kraken)])).is_err(),
        "a 2-power blocker can't block under three Islands"
    );
}

/// Tromokratis: hexproof off-combat, and blocking it requires every able
/// defender.
#[test]
fn tromokratis_hexproof_and_gang_block() {
    let mut g = main_phase();
    let kraken = g.add_card_to_battlefield(0, catalog::tromokratis());
    g.battlefield_find_mut(kraken).unwrap().summoning_sick = false;
    assert!(
        g.computed_permanent(kraken).unwrap().keywords.contains(&Keyword::Hexproof),
        "hexproof while not in combat"
    );
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::Attack {
        attacker: kraken,
        target: crabomination::game::AttackTarget::Player(1),
    }]))
    .expect("attack");
    assert!(
        !g.computed_permanent(kraken).unwrap().keywords.contains(&Keyword::Hexproof),
        "no hexproof while attacking"
    );
    g.step = TurnStep::DeclareBlockers;
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(a, kraken)])).is_err(),
        "one blocker isn't all of them"
    );
    g.perform_action(GameAction::DeclareBlockers(vec![(a, kraken), (b, kraken)]))
        .expect("both defenders block");
}

/// Heroes' Podium's anthem counts only the *other* legendary creatures.
#[test]
fn heroes_podium_counts_other_legends() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::heroes_podium());
    let one = g.add_card_to_battlefield(0, catalog::tromokratis());
    assert_eq!(g.computed_permanent(one).map(|c| c.power), Some(8), "alone: no bonus");
    let two = g.add_card_to_battlefield(0, catalog::tromokratis());
    assert_eq!(g.computed_permanent(one).map(|c| c.power), Some(9), "+1 for the other legend");
    assert_eq!(g.computed_permanent(two).map(|c| c.power), Some(9));
}

/// Astral Cornucopia enters with X charge counters and taps for that much
/// mana of one chosen color.
#[test]
fn astral_cornucopia_scales_with_charge_counters() {
    let mut g = main_phase();
    let id = g.add_card_to_hand(0, catalog::astral_cornucopia());
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast for X=2");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(id).unwrap().counter_count(CounterType::Charge),
        2,
        "two charge counters"
    );
    g.battlefield_find_mut(id).unwrap().summoning_sick = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("tap for mana");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 2, "two mana of the chosen color");
}

/// Whelming Wave spares the sea monsters.
#[test]
fn whelming_wave_spares_krakens() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let kraken = g.add_card_to_battlefield(0, catalog::kraken_of_the_straits());
    cast(&mut g, catalog::whelming_wave(), None, 2, &[(Color::Blue, 2)]);
    assert!(g.battlefield_find(bear).is_none(), "the Bears bounced");
    assert!(g.battlefield_find(kraken).is_some(), "the Kraken stayed");
}

/// Unravel the Aether shuffles the target back into its owner's library.
#[test]
fn unravel_the_aether_shuffles_away() {
    let mut g = main_phase();
    let lyre = g.add_card_to_battlefield(1, catalog::siren_song_lyre());
    cast(
        &mut g,
        catalog::unravel_the_aether(),
        Some(Target::Permanent(lyre)),
        1,
        &[(Color::Green, 1)],
    );
    assert!(g.battlefield_find(lyre).is_none());
    assert!(g.players[1].library.iter().any(|c| c.id == lyre), "back in the library");
}

/// Gild exiles the creature and leaves a Gold token behind.
#[test]
fn gild_exiles_and_pays_gold() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    cast(&mut g, catalog::gild(), Some(Target::Permanent(bear)), 3, &[(Color::Black, 1)]);
    assert!(g.battlefield_find(bear).is_none(), "exiled");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Gold"), "Gold token");
}

/// Scourge of Skola Vale enters with two counters and eats a creature for its
/// toughness.
#[test]
fn scourge_of_skola_vale_eats_for_toughness() {
    let mut g = main_phase();
    let hydra = cast(&mut g, catalog::scourge_of_skola_vale(), None, 2, &[(Color::Green, 1)]);
    g.battlefield_find_mut(hydra).unwrap().summoning_sick = false;
    assert_eq!(g.battlefield_find(hydra).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    g.add_card_to_battlefield(0, catalog::great_hart()); // 2/4
    g.perform_action(GameAction::ActivateAbility {
        card_id: hydra,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("sac for counters");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(hydra).unwrap().counter_count(CounterType::PlusOnePlusOne),
        6,
        "two starting plus the Hart's toughness"
    );
}

/// Stratus Walk draws, grants flying, and limits the host to blocking fliers.
#[test]
fn stratus_walk_grants_flight_and_restricts_blocks() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    g.add_card_to_library(0, catalog::great_hart());
    cast(&mut g, catalog::stratus_walk(), Some(Target::Permanent(bear)), 1, &[(Color::Blue, 1)]);
    assert_eq!(g.players[0].hand.len(), hand + 1, "ETB draw");
    let kws = g.computed_permanent(bear).unwrap().keywords.clone();
    assert!(kws.contains(&Keyword::Flying));
    assert!(kws.contains(&Keyword::CanBlockOnlyFlying));
}

/// Raised by Wolves mints two Wolves and scales the host by the Wolf count.
#[test]
fn raised_by_wolves_scales_with_wolves() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    cast(
        &mut g,
        catalog::raised_by_wolves(),
        Some(Target::Permanent(bear)),
        3,
        &[(Color::Green, 2)],
    );
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Wolf").count(),
        2,
        "two Wolf tokens"
    );
    assert_eq!(g.computed_permanent(bear).map(|c| (c.power, c.toughness)), Some((4, 4)));
}

/// Sanguimancy and Skyreaping both read devotion.
#[test]
fn bng2_devotion_payoffs() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forlorn_pseudamma()); // {3}{B}
    }
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::great_hart());
    }
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    cast(&mut g, catalog::sanguimancy(), None, 4, &[(Color::Black, 1)]);
    assert_eq!(g.players[0].hand.len(), hand + 3, "drew devotion-to-black cards");
    assert_eq!(g.players[0].life, life - 3);
}

/// Thassa's Rebuff taxes by your devotion to blue.
#[test]
fn thassas_rebuff_taxes_by_devotion() {
    let mut g = main_phase();
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::sphinxs_disciple()); // {3}{U}{U}
    }
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    g.priority.player_with_priority = 0;
    let rebuff = g.add_card_to_hand(0, catalog::thassas_rebuff());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: rebuff,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("counter");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "unpaid tax — countered");
}

// ── Wave 4 (bng3) ────────────────────────────────────────────────────────────

/// The Fated cycle scries only on your own turn.
#[test]
fn fated_cycle_scries_only_on_your_turn() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::great_hart());
    cast(
        &mut g,
        catalog::fated_conflagration(),
        Some(Target::Permanent(bear)),
        1,
        &[(Color::Red, 3)],
    );
    assert!(g.battlefield_find(bear).is_none(), "5 damage killed it");

    // Same spell on the opponent's turn: no scry, so the library is untouched.
    g.active_player_idx = 1;
    let other = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let lib = g.players[0].library.len();
    cast(
        &mut g,
        catalog::fated_conflagration(),
        Some(Target::Permanent(other)),
        1,
        &[(Color::Red, 3)],
    );
    assert_eq!(g.players[0].library.len(), lib, "no scry off-turn");
}

/// Fated Retribution wipes creatures and planeswalkers alike.
#[test]
fn fated_retribution_wipes_the_board() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::great_hart());
    g.add_card_to_library(0, catalog::great_hart());
    cast(&mut g, catalog::fated_retribution(), None, 4, &[(Color::White, 3)]);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(mine).is_none());
}

/// Fated Return reanimates with indestructible attached.
#[test]
fn fated_return_reanimates_indestructible() {
    let mut g = main_phase();
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::great_hart());
    cast(&mut g, catalog::fated_return(), Some(Target::Permanent(dead)), 4, &[(Color::Black, 3)]);
    let c = g.battlefield_find(dead).expect("reanimated");
    assert_eq!(c.controller, 0, "under your control");
    assert!(g.computed_permanent(dead).unwrap().keywords.contains(&Keyword::Indestructible));
}

/// Tribute declined (the AutoDecider's default) fires the "if tribute wasn't
/// paid" half; the Archon mints its Birds.
#[test]
fn ornitharch_tribute_declined_mints_birds() {
    let mut g = main_phase();
    cast(&mut g, catalog::ornitharch(), None, 3, &[(Color::White, 2)]);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Bird").count(),
        2,
        "tribute declined → two Birds"
    );
}

/// Tribute paid suppresses the ETB half and leaves the counters instead.
#[test]
fn ornitharch_tribute_paid_adds_counters() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let id = cast(&mut g, catalog::ornitharch(), None, 3, &[(Color::White, 2)]);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Bird").count(), 0);
}

/// Bestow: Ghostblade Eidolon cast for its bestow cost is an Aura that hands
/// over +1/+1 and double strike.
#[test]
fn ghostblade_eidolon_bestows_double_strike() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::ghostblade_eidolon());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastBestow {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bestow");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords.contains(&Keyword::DoubleStrike));
}

/// Everflame Eidolon pumps itself as a creature and the host as an Aura.
#[test]
fn everflame_eidolon_pumps_the_right_body() {
    let mut g = main_phase();
    let id = cast(&mut g, catalog::everflame_eidolon(), None, 1, &[(Color::Red, 1)]);
    g.battlefield_find_mut(id).unwrap().summoning_sick = false;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("pump self");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(id).map(|c| c.power), Some(2), "as a creature it pumps itself");
}

/// Eidolon of Countless Battles scales off creatures and Auras you control.
#[test]
fn eidolon_of_countless_battles_scales() {
    let mut g = main_phase();
    let id = cast(&mut g, catalog::eidolon_of_countless_battles(), None, 1, &[(Color::White, 2)]);
    // Itself is one creature.
    assert_eq!(g.computed_permanent(id).map(|c| (c.power, c.toughness)), Some((1, 1)));
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(id).map(|c| (c.power, c.toughness)), Some((2, 2)));
}

/// Ragemonger strips {B}{R} off Minotaur spells.
#[test]
fn ragemonger_discounts_minotaurs_colored() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::ragemonger());
    let butcher = g.add_card_to_hand(0, catalog::kragma_butcher()); // {2}{R} Minotaur
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: butcher,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{2} covers {2}{R} minus {B}{R}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(butcher).is_some());
}

/// Pillar of War can attack once it's enchanted.
#[test]
fn pillar_of_war_attacks_while_enchanted() {
    let mut g = main_phase();
    let pillar = g.add_card_to_battlefield(0, catalog::pillar_of_war());
    g.battlefield_find_mut(pillar).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    let attack = vec![crabomination::game::Attack {
        attacker: pillar,
        target: crabomination::game::AttackTarget::Player(1),
    }];
    assert!(
        g.perform_action(GameAction::DeclareAttackers(attack.clone())).is_err(),
        "defender keeps it home"
    );
    g.step = TurnStep::PreCombatMain;
    cast(
        &mut g,
        catalog::weight_of_the_underworld(),
        Some(Target::Permanent(pillar)),
        3,
        &[(Color::Black, 1)],
    );
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(attack)).expect("enchanted, so it can attack");
}

/// Floodtide Serpent's attack cost bounces one of your enchantments.
#[test]
fn floodtide_serpent_pays_by_bouncing_an_enchantment() {
    let mut g = main_phase();
    let serpent = g.add_card_to_battlefield(0, catalog::floodtide_serpent());
    g.battlefield_find_mut(serpent).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    let attack = vec![crabomination::game::Attack {
        attacker: serpent,
        target: crabomination::game::AttackTarget::Player(1),
    }];
    assert!(
        g.perform_action(GameAction::DeclareAttackers(attack.clone())).is_err(),
        "no enchantment to return"
    );
    let ench = g.add_card_to_battlefield(0, catalog::fate_unraveler());
    g.perform_action(GameAction::DeclareAttackers(attack)).expect("cost paid");
    assert!(g.battlefield_find(ench).is_none(), "the enchantment bounced");
}

/// Acolyte's Reward prevents by devotion and throws the prevented damage back.
#[test]
fn acolytes_reward_prevents_and_reflects() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::vanguard_of_brimaz()); // {W}{W}
    }
    let mine = g.add_card_to_battlefield(0, catalog::great_hart());
    let spell = g.add_card_to_hand(0, catalog::acolytes_reward());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Player(1)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let life = g.players[1].life;
    let mut events = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(mine),
        4,
        None,
        &mut events,
    );
    assert_eq!(g.battlefield_find(mine).unwrap().damage, 0, "6 devotion soaked all 4");
    assert_eq!(g.players[1].life, life - 4, "the prevented damage was reflected");
}

/// Arbiter of the Ideal cheats the revealed permanent in as an enchantment
/// with a manifestation counter.
#[test]
fn arbiter_of_the_ideal_cheats_in_an_enchantment() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    let arbiter = g.add_card_to_battlefield(0, catalog::arbiter_of_the_ideal());
    let top = g.add_card_to_library(0, catalog::great_hart());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ctx = crabomination::game::effects::EffectContext::for_ability(arbiter, 0, None);
    g.resolve_effect(&catalog::arbiter_of_the_ideal().triggered_abilities[0].effect, &ctx)
        .expect("inspired body");
    drain_stack(&mut g);
    assert!(g.battlefield_find(top).is_some(), "put onto the battlefield");
    assert_eq!(g.battlefield_find(top).unwrap().counter_count(CounterType::Manifestation), 1);
    assert!(
        g.computed_permanent(top)
            .unwrap()
            .card_types
            .contains(&crabomination::card::CardType::Enchantment),
        "an enchantment in addition to its other types"
    );
}

/// Champion of Stray Souls trades X creatures for X reanimations.
#[test]
fn champion_of_stray_souls_reanimates_x() {
    let mut g = main_phase();
    let champ = g.add_card_to_battlefield(0, catalog::champion_of_stray_souls());
    g.battlefield_find_mut(champ).unwrap().summoning_sick = false;
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let dead = g.add_card_to_graveyard(0, catalog::great_hart());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: champ,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: Some(1), mode: None,
    })
    .expect("sac one, return one");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed");
    assert!(g.battlefield_find(dead).is_some(), "reanimated");
}

/// Vortex Elemental shuffles itself and its blockers away.
#[test]
fn vortex_elemental_shuffles_the_combat_away() {
    let mut g = main_phase();
    let vortex = g.add_card_to_battlefield(0, catalog::vortex_elemental());
    g.battlefield_find_mut(vortex).unwrap().summoning_sick = false;
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::Attack {
        attacker: vortex,
        target: crabomination::game::AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, vortex)])).expect("block");
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: vortex,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("shuffle away");
    drain_stack(&mut g);
    assert!(g.battlefield_find(vortex).is_none());
    assert!(g.battlefield_find(blocker).is_none(), "the blocker went too");
}

/// Satyr Firedancer forwards instant/sorcery damage onto an opposing creature.
#[test]
fn satyr_firedancer_forwards_spell_damage() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::satyr_firedancer());
    let target = g.add_card_to_battlefield(1, catalog::great_hart());
    g.add_card_to_library(0, catalog::great_hart());
    let bolt = g.add_card_to_hand(0, catalog::bolt_of_keranos());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![Target::Permanent(target)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target).map(|c| c.damage), Some(3), "3 forwarded");
}

/// "That player controls" — a spell that hits its own caster forwards onto the
/// caster's creature, not the opponent's.
#[test]
fn satyr_firedancer_forwards_to_the_damaged_players_creature() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::satyr_firedancer());
    let mine = g.add_card_to_battlefield(0, catalog::great_hart());
    let theirs = g.add_card_to_battlefield(1, catalog::great_hart());
    g.add_card_to_library(0, catalog::great_hart());
    let bolt = g.add_card_to_hand(0, catalog::bolt_of_keranos());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(mine).map(|c| c.damage), Some(3));
    assert_eq!(g.battlefield_find(theirs).map(|c| c.damage), Some(0));
}

/// Kiora's +1 locks a permanent out of dealing and taking damage.
#[test]
fn kiora_plus_one_locks_damage_both_ways() {
    let mut g = main_phase();
    let kiora = g.add_card_to_battlefield(0, catalog::kiora_the_crashing_wave());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::great_hart());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: kiora,
        ability_index: 0,
        target: Some(Target::Permanent(theirs)),
        x_value: None,
    })
    .expect("+1");
    drain_stack(&mut g);
    let mut events = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(theirs),
        3,
        None,
        &mut events,
    );
    assert_eq!(g.battlefield_find(theirs).unwrap().damage, 0, "damage to it is prevented");
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(mine),
        3,
        Some(theirs),
        &mut events,
    );
    assert_eq!(g.battlefield_find(mine).unwrap().damage, 0, "damage by it is prevented too");
}

/// Mindreaver's sac only counters a spell it already exiled a copy of.
#[test]
fn mindreaver_counters_only_named_spells() {
    let mut g = main_phase();
    let reaver = g.add_card_to_battlefield(0, catalog::mindreaver());
    g.add_card_to_library(1, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_ability(reaver, 0, None);
    g.resolve_effect(
        &crabomination::effect::Effect::ExileTopOfLibrary {
            who: crabomination::effect::Selector::Player(crabomination::effect::PlayerRef::Seat(1)),
            amount: crabomination::effect::Value::Const(1),
            link_to_source: true,
            face_down: false,
        },
        &ctx,
    )
    .expect("exile the top card with Mindreaver");
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: reaver,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("counter it");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "name matched → countered");
}

/// Perplexing Chimera swaps itself for the opponent's spell.
#[test]
fn perplexing_chimera_swaps_for_the_spell() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    let chimera = g.add_card_to_battlefield(0, catalog::perplexing_chimera());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).map(|c| c.controller),
        Some(0),
        "the spell resolved under your control"
    );
    assert_eq!(
        g.battlefield_find(chimera).map(|c| c.controller),
        Some(1),
        "the Chimera went the other way"
    );
}

/// Whims of the Fates sacrifices roughly a third of each board.
#[test]
fn whims_of_the_fates_sacrifices_a_pile() {
    let mut g = main_phase();
    for _ in 0..6 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
    }
    cast(&mut g, catalog::whims_of_the_fates(), None, 5, &[(Color::Red, 1)]);
    for seat in 0..2 {
        let left = g.battlefield.iter().filter(|c| c.controller == seat).count();
        assert!((3..6).contains(&left), "seat {seat} kept {left} of 6");
    }
}
