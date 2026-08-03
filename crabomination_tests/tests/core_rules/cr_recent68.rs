//! CR conformance for this run's engine work:
//! - CR 702.22 — attacking bands (declaration legality, block-as-a-unit,
//!   removal from combat).
//! - CR 612 — the creature-type text change: it reaches words used as token
//!   types and survives the stack → battlefield hop; it never touches a name.
//! - CR 313.2 — a vanguard card stays in the command zone.

use crabomination::card::{CardId, CreatureType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn to_declare_attackers(g: &mut GameState) {
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
}

fn at(attacker: CardId, target: AttackTarget) -> Attack {
    Attack { attacker, target }
}

// ── CR 702.22 — attacking bands ─────────────────────────────────────────────

/// CR 702.22h — a blocker assigned to one member of a band blocks every other
/// member too, so nothing in the band gets through.
#[test]
fn cr_702_22h_blocking_one_bandmate_blocks_the_whole_band() {
    let mut g = main_phase();
    let hero = g.add_card_to_battlefield(0, catalog::benalish_hero());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_stone());
    for id in [hero, bear] {
        g.clear_sickness(id);
    }
    to_declare_attackers(&mut g);
    g.perform_action(GameAction::DeclareAttackersBanded {
        attacks: vec![at(hero, AttackTarget::Player(1)), at(bear, AttackTarget::Player(1))],
        bands: vec![vec![hero, bear]],
    })
    .expect("declare band");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, hero)])).expect("block");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[1].life, 20, "the Bear was blocked along with its bandmate");
}

/// CR 702.22c — a band needs at least one member with banding and may hold at
/// most one member without it.
#[test]
fn cr_702_22c_band_requires_banding_and_at_most_one_without() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hero = g.add_card_to_battlefield(0, catalog::benalish_hero());
    for id in [a, b, hero] {
        g.clear_sickness(id);
    }
    to_declare_attackers(&mut g);
    let attacks: Vec<Attack> =
        [a, b, hero].iter().map(|&id| at(id, AttackTarget::Player(1))).collect();
    assert!(
        g.perform_action(GameAction::DeclareAttackersBanded {
            attacks: attacks.clone(),
            bands: vec![vec![a, b]],
        })
        .is_err(),
        "no member has banding"
    );
    assert!(
        g.perform_action(GameAction::DeclareAttackersBanded {
            attacks: attacks.clone(),
            bands: vec![vec![a, b, hero]],
        })
        .is_err(),
        "two members lack banding"
    );
    g.perform_action(GameAction::DeclareAttackersBanded {
        attacks,
        bands: vec![vec![a, hero]],
    })
    .expect("one banded + one unbanded is legal");
}

/// CR 702.22d — every creature in a band must attack the same defender.
#[test]
fn cr_702_22d_band_members_share_one_defender() {
    let mut g = main_phase();
    let hero = g.add_card_to_battlefield(0, catalog::benalish_hero());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pw = g.add_card_to_battlefield(1, catalog::jace_beleren());
    for id in [hero, bear] {
        g.clear_sickness(id);
    }
    to_declare_attackers(&mut g);
    assert!(
        g.perform_action(GameAction::DeclareAttackersBanded {
            attacks: vec![
                at(hero, AttackTarget::Player(1)),
                at(bear, AttackTarget::Planeswalker(pw)),
            ],
            bands: vec![vec![hero, bear]],
        })
        .is_err(),
        "the band is split across a player and a planeswalker"
    );
}

/// CR 702.22f — a creature removed from combat leaves its band, so blocking a
/// survivor no longer reaches it.
#[test]
fn cr_702_22f_removal_from_combat_leaves_the_band() {
    let mut g = main_phase();
    let hero = g.add_card_to_battlefield(0, catalog::benalish_hero());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pegasus = g.add_card_to_battlefield(0, catalog::mesa_pegasus());
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_stone());
    for id in [hero, bear, pegasus] {
        g.clear_sickness(id);
    }
    to_declare_attackers(&mut g);
    g.perform_action(GameAction::DeclareAttackersBanded {
        attacks: [hero, bear, pegasus]
            .iter()
            .map(|&id| at(id, AttackTarget::Player(1)))
            .collect(),
        bands: vec![vec![hero, bear, pegasus]],
    })
    .expect("declare band");
    // CR 506.4 — the bounced Pegasus is removed from combat, and from the band.
    let bounce = g.add_card_to_hand(1, catalog::unsummon());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bounce,
        target: Some(Target::Permanent(pegasus)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bounce");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, hero)])).expect("block");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[1].life, 20, "the two survivors are both blocked");
    assert!(g.players[0].hand.iter().any(|c| c.id == pegasus), "and the Pegasus left combat");
}

/// The server view surfaces each band, so a defender can see which attackers
/// one block would drag in (CR 702.22h).
#[test]
fn cr_702_22_bands_reach_the_server_view() {
    use crabomination::server::view::project;
    let mut g = main_phase();
    let hero = g.add_card_to_battlefield(0, catalog::benalish_hero());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [hero, bear] {
        g.clear_sickness(id);
    }
    to_declare_attackers(&mut g);
    g.perform_action(GameAction::DeclareAttackersBanded {
        attacks: vec![at(hero, AttackTarget::Player(1)), at(bear, AttackTarget::Player(1))],
        bands: vec![vec![hero, bear]],
    })
    .expect("declare band");
    assert_eq!(project(&g, 1).attack_bands, vec![vec![hero, bear]]);
}

// ── CR 612 — creature-type text change ──────────────────────────────────────

/// CR 612.2a — a creature-type word used to define a token is text, so the
/// rewrite changes the tokens the ability mints. The change is applied to the
/// spell and survives its resolution onto the battlefield.
#[test]
fn cr_612_2a_text_change_on_a_spell_reaches_its_token_types() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::CreatureTypePair(
        CreatureType::Goblin,
        CreatureType::Bear,
    )]));
    let chief = g.add_card_to_hand(0, catalog::beetleback_chief());
    let evo = g.add_card_to_hand(0, catalog::artificial_evolution());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: chief,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the Chief");
    // Respond, hitting the spell on the stack.
    g.perform_action(GameAction::CastSpell {
        card_id: evo,
        target: Some(Target::Permanent(chief)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Artificial Evolution");
    drain_stack(&mut g);
    let types = &g.battlefield_find(chief).unwrap().definition.subtypes.creature_types;
    assert!(types.contains(&CreatureType::Bear) && !types.contains(&CreatureType::Goblin));
    let tokens: Vec<_> = g.battlefield.iter().filter(|c| c.is_token).collect();
    assert_eq!(tokens.len(), 2);
    assert!(
        tokens
            .iter()
            .all(|t| t.definition.subtypes.creature_types.contains(&CreatureType::Bear)),
        "CR 612.2a — the token's type word was rewritten too"
    );
}

/// CR 612.4 — a token's subtypes come from the ability that made it, and a
/// text change applied to the token itself can rewrite them.
#[test]
fn cr_612_4_text_change_rewrites_a_tokens_own_subtypes() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::CreatureTypePair(
        CreatureType::Goblin,
        CreatureType::Zombie,
    )]));
    let chief = g.add_card_to_hand(0, catalog::beetleback_chief());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: chief,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let token = g.battlefield.iter().find(|c| c.is_token).unwrap().id;
    let evo = g.add_card_to_hand(0, catalog::artificial_evolution());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: evo,
        target: Some(Target::Permanent(token)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let types = &g.battlefield_find(token).unwrap().definition.subtypes.creature_types;
    assert_eq!(types, &vec![CreatureType::Zombie]);
}

// ── CR 313 — vanguard cards ─────────────────────────────────────────────────

/// CR 313.2 — a vanguard card can't be cast and can't leave the command zone,
/// and it keeps working from there.
#[test]
fn cr_313_2_vanguard_stays_in_the_command_zone() {
    let mut g = main_phase();
    let avatar = g.seat_vanguard(0, catalog::ashling_the_pilgrim_avatar());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: avatar,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "CR 313.2 — a vanguard card can't be cast"
    );
    g.remove_from_battlefield_to_exile(avatar);
    assert!(
        g.players[0].command.iter().any(|c| c.id == avatar),
        "CR 313.2 — it remains in the command zone"
    );
    assert!(g.exile.iter().all(|c| c.id != avatar));
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: avatar,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("still activates");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 1);
}
