//! CR conformance for this run's engine work:
//! - CR 702.22d/j — "bands with other [quality]": band legality without
//!   banding, and the defender assigning a blocked creature's damage.
//! - CR 805.4b/4c — the shared team turns option: every teammate draws in the
//!   team's draw step and gets their own land drop on the team's turn.
//! - CR 607.2 — a linked imprint ability reads only its own source's exiles.

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn at(attacker: CardId, target: AttackTarget) -> Attack {
    Attack { attacker, target }
}

fn to_declare_attackers(g: &mut GameState) {
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
}

// ── CR 702.22 — bands with other ────────────────────────────────────────────

/// CR 702.22d — legendary creatures band together on the strength of the
/// land's grant, with no member carrying plain banding.
#[test]
fn cr_702_22d_bands_with_other_needs_no_plain_banding() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::adventurers_guildhouse());
    let a = g.add_card_to_battlefield(0, catalog::kamahl_fist_of_krosa());
    let b = g.add_card_to_battlefield(0, catalog::kamahl_fist_of_krosa());
    for id in [a, b] {
        g.clear_sickness(id);
    }
    assert!(g.computed_permanent(a).unwrap().keywords.iter().any(|k| {
        matches!(k, Keyword::BandsWithOther(_))
    }));
    to_declare_attackers(&mut g);
    g.perform_action(GameAction::DeclareAttackersBanded {
        attacks: vec![at(a, AttackTarget::Player(1)), at(b, AttackTarget::Player(1))],
        bands: vec![vec![a, b]],
    })
    .expect("two green legends band without banding");
}

/// CR 702.22d — the grant is colour-scoped, so an off-colour legend can't join.
#[test]
fn cr_702_22d_band_quality_must_cover_every_member() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::adventurers_guildhouse());
    let legend = g.add_card_to_battlefield(0, catalog::kamahl_fist_of_krosa());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [legend, bear] {
        g.clear_sickness(id);
    }
    to_declare_attackers(&mut g);
    assert!(
        g.perform_action(GameAction::DeclareAttackersBanded {
            attacks: vec![
                at(legend, AttackTarget::Player(1)),
                at(bear, AttackTarget::Player(1)),
            ],
            bands: vec![vec![legend, bear]],
        })
        .is_err(),
        "the Bear is not a legendary creature"
    );
}

/// CR 702.22j — two band-quality blockers hand the attacker's damage division
/// to the defending player.
#[test]
fn cr_702_22j_defender_divides_damage_against_a_quality_band() {
    use crabomination::decision::Decision;
    use crabomination::game::types::ResumeContext;
    let mut g = main_phase();
    g.players[1].wants_ui = true;
    g.add_card_to_battlefield(1, catalog::adventurers_guildhouse());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let x = g.add_card_to_battlefield(1, catalog::kamahl_fist_of_krosa());
    let y = g.add_card_to_battlefield(1, catalog::kamahl_fist_of_krosa());
    g.clear_sickness(attacker);
    to_declare_attackers(&mut g);
    g.perform_action(GameAction::DeclareAttackers(vec![at(attacker, AttackTarget::Player(1))]))
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(x, attacker), (y, attacker)]))
        .expect("gang block");
    drain_stack(&mut g);
    while g.pending_decision.is_none() {
        g.perform_action(GameAction::PassPriority).expect("pass");
        assert!(!g.is_game_over());
    }
    let pd = g.pending_decision.as_ref().expect("combat suspends on ordering");
    assert!(
        matches!(pd.resume, ResumeContext::CombatDamage { player: 1, .. }),
        "the defending player, not the attacker, divides it: {:?}",
        pd.resume
    );
    assert!(matches!(pd.decision, Decision::CombatDamageOrder { .. }));
}

/// CR 702.22 — Tolaria strips both banding and every "bands with other".
#[test]
fn cr_702_22_tolaria_strips_all_band_abilities() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::adventurers_guildhouse());
    let tolaria = g.add_card_to_battlefield(0, catalog::tolaria());
    let legend = g.add_card_to_battlefield(0, catalog::kamahl_fist_of_krosa());
    g.step = TurnStep::Upkeep;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tolaria,
        ability_index: 1,
        target: Some(Target::Permanent(legend)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(
        !g.computed_permanent(legend)
            .unwrap()
            .keywords
            .iter()
            .any(|k| matches!(k, Keyword::BandsWithOther(_))),
        "the grant is gone for the turn"
    );
}

// ── CR 805 — shared team turns ──────────────────────────────────────────────

fn two_headed_giant() -> GameState {
    let mut g = game_with_format(crabomination::format::Format::TwoHeadedGiant, 4);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// CR 805.4b — each player on the active team draws in the team's draw step.
#[test]
fn cr_805_4b_every_teammate_draws_in_the_shared_draw_step() {
    let mut g = two_headed_giant();
    for seat in 0..4 {
        for _ in 0..5 {
            g.add_card_to_library(seat, catalog::grizzly_bears());
        }
        g.players[seat].hand.clear();
    }
    g.step = TurnStep::Upkeep;
    let _ = g.advance_step(Vec::new());
    assert_eq!(g.step, TurnStep::Draw);
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[1].hand.len(), 1, "the teammate drew too");
    assert_eq!(g.players[2].hand.len(), 0, "the other team did not");
}

/// CR 805.4c — a teammate plays their own land on the team's turn.
#[test]
fn cr_805_4c_teammate_gets_their_own_land_drop() {
    let mut g = two_headed_giant();
    let mine = g.add_card_to_hand(0, catalog::forest());
    let theirs = g.add_card_to_hand(1, catalog::forest());
    g.perform_action(GameAction::PlayLand(mine)).expect("active player's land");
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::PlayLand(theirs)).expect("teammate's land");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_land()).count(), 2);
}

/// CR 805.4c — the shared turn does not extend to an opposing team.
#[test]
fn cr_805_4c_opposing_team_still_cannot_play_a_land() {
    let mut g = two_headed_giant();
    let theirs = g.add_card_to_hand(2, catalog::forest());
    g.priority.player_with_priority = 2;
    assert!(g.perform_action(GameAction::PlayLand(theirs)).is_err());
}

// ── CR 607 — linked abilities ───────────────────────────────────────────────

/// CR 607.2 — each Myr Welder's static reads only the cards exiled with *it*.
#[test]
fn cr_607_2_imprint_static_reads_only_its_own_exiles() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(0, catalog::myr_welder());
    let b = g.add_card_to_battlefield(0, catalog::myr_welder());
    for id in [a, b] {
        g.clear_sickness(id);
    }
    let web = g.add_card_to_graveyard(0, catalog::decimator_web());
    let base = g.granted_abilities_for(b).len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: a,
        ability_index: 0,
        target: Some(Target::Permanent(web)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("imprint");
    drain_stack(&mut g);
    assert_eq!(g.granted_abilities_for(a).len(), base + 1);
    assert_eq!(g.granted_abilities_for(b).len(), base, "the other Welder is unaffected");
}
