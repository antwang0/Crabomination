//! Functionality tests for `catalog::sets::decks::recent109` — the CR 104.3
//! can't-lose/can't-win cluster (Angel's Grace, Platinum Angel, Abyssal
//! Persecutor, Worship).

use crate::catalog;
use crate::game::effects::EntityRef;
use crate::game::*;

/// CR 104.3d — Platinum Angel's controller skips the life-loss SBA; the loss
/// resumes as soon as the Angel leaves.
#[test]
fn cr_104_3d_platinum_angel_blocks_loss_sbas() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(0, catalog::platinum_angel());
    g.players[0].life = 0;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    assert!(!g.players[0].eliminated, "can't lose at 0 life with the Angel");
    assert!(!g.is_game_over());
    g.remove_from_battlefield_to_graveyard_raw(angel);
    g.check_state_based_actions();
    assert!(g.players[0].eliminated, "loss SBA resumes once the Angel leaves");
}

/// CR 104.3d — an opponent's "you win the game" effect does nothing while
/// Platinum Angel's controller can't lose.
#[test]
fn cr_104_3d_platinum_angel_blocks_opponent_win_effect() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::platinum_angel());
    let ctx = crate::game::effects::EffectContext::for_spell(1, None, 0, 0);
    g.resolve_effect(
        &crate::effect::Effect::WinGame { who: crate::effect::PlayerRef::You },
        &ctx,
    )
    .unwrap();
    g.check_state_based_actions();
    assert!(!g.players[0].eliminated, "opponent can't win through the Angel");
    assert!(!g.is_game_over());
}

/// CR 104.3d — Abyssal Persecutor keeps its controller's opponents alive at
/// 0 life; killing it hands them the loss.
#[test]
fn cr_104_3d_abyssal_persecutor_keeps_opponents_alive() {
    let mut g = two_player_game();
    let demon = g.add_card_to_battlefield(0, catalog::abyssal_persecutor());
    g.players[1].life = -3;
    g.check_state_based_actions();
    assert!(!g.players[1].eliminated, "opponent can't lose under the Persecutor");
    g.remove_from_battlefield_to_graveyard_raw(demon);
    g.check_state_based_actions();
    assert!(g.players[1].eliminated);
    assert!(g.is_game_over());
}

/// Angel's Grace — can't lose this turn, and damage that would drop you
/// below 1 life drops you to 1; both wear off at the turn boundary.
#[test]
fn angels_grace_floors_damage_and_blocks_loss_this_turn() {
    let mut g = two_player_game();
    let grace = g.add_card_to_hand(0, catalog::angels_grace());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.step = crate::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: grace, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Angel's Grace");
    drain_stack(&mut g);
    g.players[0].life = 3;
    let mut evs = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(0), 9, None, &mut evs);
    assert_eq!(g.players[0].life, 1, "damage floored at 1 life");
    // A non-damage loss effect is also blocked this turn.
    let ctx = crate::game::effects::EffectContext::for_spell(1, None, 0, 0);
    g.resolve_effect(
        &crate::effect::Effect::LoseGame { who: crate::effect::PlayerRef::EachOpponent },
        &ctx,
    )
    .unwrap();
    assert!(!g.players[0].eliminated, "can't lose this turn");
    // The protections end at the turn boundary.
    g.do_untap();
    assert!(!g.players[0].cant_lose_this_turn);
    assert!(!g.players[0].damage_floor_this_turn);
}

/// Worship — with a creature, damage can't take its controller below 1;
/// without one the floor is off.
#[test]
fn worship_floors_damage_while_controlling_a_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::worship());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].life = 2;
    let mut evs = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(0), 7, None, &mut evs);
    assert_eq!(g.players[0].life, 1, "floored while a creature is out");
    g.remove_from_battlefield_to_graveyard_raw(bear);
    g.deal_damage_to_from(EntityRef::Player(0), 7, None, &mut evs);
    assert_eq!(g.players[0].life, -6, "no creature, no floor");
}

// ── CR 113.11 — "can't have or gain" (Archetypes) ─────────────────────────────

/// Archetype of Imagination: your team gains flying; opponents' creatures
/// lose printed flying and can't gain it even from a later grant.
#[test]
fn cr_113_11_archetype_strips_and_blocks_keyword() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::archetype_of_imagination());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    assert!(
        g.computed_permanent(bear).unwrap().keywords.contains(&crate::card::Keyword::Flying),
        "your creatures gain flying"
    );
    assert!(
        !g.computed_permanent(angel).unwrap().keywords.contains(&crate::card::Keyword::Flying),
        "opponent's printed flying is stripped"
    );
    // A grant with a later timestamp still loses to the can't-have.
    let ctx =
        crate::game::effects::EffectContext::for_spell(1, Some(Target::Permanent(angel)), 0, 0);
    g.resolve_effect(
        &crate::effect::Effect::GrantKeyword {
            what: crate::card::Selector::Target(0),
            keyword: crate::card::Keyword::Flying,
            duration: crate::effect::Duration::EndOfTurn,
        },
        &ctx,
    )
    .unwrap();
    assert!(
        !g.computed_permanent(angel).unwrap().keywords.contains(&crate::card::Keyword::Flying),
        "a later EOT grant can't restore the keyword"
    );
}

// ── CR 702.19i — trample over planeswalkers ──────────────────────────────────

/// An attacker with trample over planeswalkers (CR 702.19c) assigns lethal
/// to the planeswalker and the excess to its controller; plain trample
/// never spills past a planeswalker (CR 702.19f).
#[test]
fn cr_702_19c_trample_over_planeswalkers_spills_excess() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(
        0,
        crate::card::CardDefinition {
            name: "Wagon",
            card_types: vec![crate::card::CardType::Creature],
            power: 6,
            toughness: 6,
            keywords: vec![
                crate::card::Keyword::Trample,
                crate::card::Keyword::TrampleOverPlaneswalkers,
            ],
            ..Default::default()
        },
    );
    g.clear_sickness(atk);
    let pw = g.add_card_to_battlefield(1, catalog::teferi_time_raveler()); // loyalty 4
    let life = g.players[1].life;
    g.step = crate::game::types::TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![crate::game::types::Attack {
        attacker: atk,
        target: crate::game::types::AttackTarget::Planeswalker(pw),
    }]))
    .unwrap();
    g.step = crate::game::types::TurnStep::DeclareBlockers;
    g.step = crate::game::types::TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    assert!(
        g.battlefield_find(pw)
            .is_none_or(|c| c.counter_count(CounterType::Loyalty) == 0),
        "planeswalker took lethal loyalty damage"
    );
    assert_eq!(g.players[1].life, life - 2, "6 power − 4 loyalty spills 2 to the player");
}

/// CR 702.19f — plain trample assigns nothing past a planeswalker.
#[test]
fn cr_702_19f_plain_trample_does_not_spill_over_planeswalker() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(
        0,
        crate::card::CardDefinition {
            name: "Wagon",
            card_types: vec![crate::card::CardType::Creature],
            power: 6,
            toughness: 6,
            keywords: vec![crate::card::Keyword::Trample],
            ..Default::default()
        },
    );
    g.clear_sickness(atk);
    let pw = g.add_card_to_battlefield(1, catalog::teferi_time_raveler());
    let life = g.players[1].life;
    g.step = crate::game::types::TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![crate::game::types::Attack {
        attacker: atk,
        target: crate::game::types::AttackTarget::Planeswalker(pw),
    }]))
    .unwrap();
    g.step = crate::game::types::TurnStep::DeclareBlockers;
    g.step = crate::game::types::TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    assert_eq!(g.players[1].life, life, "no spill without the variant keyword");
}
