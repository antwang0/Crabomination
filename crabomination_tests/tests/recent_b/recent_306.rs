//! Multi-block batch (CR 509.1g) + the block-count payoffs.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, Target};
use crabomination::game::{drain_stack, two_player_game, GameAction, GameState, TurnStep};
use crabomination::mana::Color;

/// Seat 1 attacks with `n` bears; seat 0 is the defender at declare-blockers.
fn combat_with_attackers(g: &mut GameState, n: usize) -> Vec<crabomination::game::CardId> {
    let attackers: Vec<_> =
        (0..n).map(|_| g.add_card_to_battlefield(1, catalog::grizzly_bears())).collect();
    g.attacking = attackers
        .iter()
        .map(|&a| Attack { attacker: a, target: AttackTarget::Player(0) })
        .collect();
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareBlockers;
    attackers
}

#[test]
fn a_plain_blocker_cannot_block_two_attackers() {
    let mut g = two_player_game();
    let atk = combat_with_attackers(&mut g, 2);
    let wall = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, atk[0]), (wall, atk[1])]))
        .expect_err("no CanBlockAdditional → the second assignment is illegal");
}

#[test]
fn can_block_additional_allows_exactly_two() {
    let mut g = two_player_game();
    let atk = combat_with_attackers(&mut g, 3);
    let brigade = g.add_card_to_battlefield(0, catalog::foriysian_brigade());
    g.perform_action(GameAction::DeclareBlockers(vec![(brigade, atk[0]), (brigade, atk[1])]))
        .expect("block an additional creature");
    assert_eq!(g.attackers_blocked_by(brigade), [atk[0], atk[1]]);
    // A third would exceed the 1 + 1 allowance.
    g.perform_action(GameAction::DeclareBlockers(vec![(brigade, atk[2])]))
        .expect_err("allowance is two, not any number");
}

#[test]
fn can_block_any_number_takes_all_comers() {
    let mut g = two_player_game();
    let atk = combat_with_attackers(&mut g, 3);
    let guard = g.add_card_to_battlefield(0, catalog::palace_guard());
    g.perform_action(GameAction::DeclareBlockers(
        atk.iter().map(|&a| (guard, a)).collect(),
    ))
    .expect("Palace Guard blocks any number");
    assert_eq!(g.attackers_blocked_by(guard).len(), 3);
    // Every attacker is blocked, so none goes unblocked.
    for &a in &atk {
        assert_eq!(g.blockers_of(a), vec![guard]);
    }
}

#[test]
fn the_same_attacker_cannot_be_blocked_twice_by_one_creature() {
    let mut g = two_player_game();
    let atk = combat_with_attackers(&mut g, 1);
    let guard = g.add_card_to_battlefield(0, catalog::palace_guard());
    g.perform_action(GameAction::DeclareBlockers(vec![(guard, atk[0]), (guard, atk[0])]))
        .expect_err("one blocker can't block the same attacker twice");
}

/// CR 510.1d — a blocker facing several attackers divides its damage, assigning
/// lethal in order rather than dealing full power to each.
#[test]
fn cr_510_1d_multi_blocker_divides_its_combat_damage() {
    let mut g = two_player_game();
    let atk = combat_with_attackers(&mut g, 2); // two 2/2 bears
    let guard = g.add_card_to_battlefield(0, catalog::palace_guard()); // 1/4
    g.perform_action(GameAction::DeclareBlockers(vec![(guard, atk[0]), (guard, atk[1])]))
        .expect("blocks both");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    let dmg: Vec<u32> =
        atk.iter().map(|&a| g.battlefield_find(a).map(|c| c.damage).unwrap_or(0)).collect();
    assert_eq!(dmg.iter().sum::<u32>(), 1, "1 power split, not 1 to each");
}




#[test]
fn high_ground_grants_the_team_an_extra_block() {
    let mut g = two_player_game();
    let atk = combat_with_attackers(&mut g, 2);
    g.add_card_to_battlefield(0, catalog::high_ground());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::DeclareBlockers(vec![(bear, atk[0]), (bear, atk[1])]))
        .expect("High Ground makes a vanilla bear a double blocker");
}


#[test]
fn echo_circlet_lets_the_equipped_creature_block_twice() {
    let mut g = two_player_game();
    let atk = combat_with_attackers(&mut g, 2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let circlet = g.add_card_to_battlefield(0, catalog::echo_circlet());
    g.battlefield_find_mut(circlet).unwrap().attached_to = Some(bear);
    g.perform_action(GameAction::DeclareBlockers(vec![(bear, atk[0]), (bear, atk[1])]))
        .expect("equipped creature blocks an additional creature");
}

#[test]
fn kembas_legion_scales_with_attached_equipment() {
    let mut g = two_player_game();
    let atk = combat_with_attackers(&mut g, 3);
    let kemba = g.add_card_to_battlefield(0, catalog::kembas_legion());
    // Bare: one block only.
    g.perform_action(GameAction::DeclareBlockers(vec![(kemba, atk[0]), (kemba, atk[1])]))
        .expect_err("no Equipment attached → the usual single block");
    let shield = g.add_card_to_battlefield(0, catalog::vanguards_shield());
    g.battlefield_find_mut(shield).unwrap().attached_to = Some(kemba);
    // Shield itself grants +1, Kemba's static adds another per Equipment.
    g.perform_action(GameAction::DeclareBlockers(
        atk.iter().map(|&a| (kemba, a)).collect(),
    ))
    .expect("1 + 1 (Shield) + 1 (per-Equipment) = 3");
}

#[test]
fn avatar_of_hope_costs_six_less_at_three_life() {
    let mut g = two_player_game();
    let aoh = g.add_card_to_hand(0, catalog::avatar_of_hope());
    g.players[0].life = 3;
    g.players[0].mana_pool.add(Color::White, 2);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: aoh,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{6}{W}{W} reduced to {W}{W}");
}

#[test]
fn avatar_of_hope_is_full_price_above_three_life() {
    let mut g = two_player_game();
    let aoh = g.add_card_to_hand(0, catalog::avatar_of_hope());
    g.players[0].life = 4;
    g.players[0].mana_pool.add(Color::White, 2);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: aoh,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect_err("no discount at 4 life");
}

#[test]
fn blaze_of_glory_only_casts_before_blockers() {
    let mut g = two_player_game();
    let atk = combat_with_attackers(&mut g, 1);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bog = g.add_card_to_hand(0, catalog::blaze_of_glory());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::DeclareBlockers(vec![(bear, atk[0])])).expect("blocks");
    g.perform_action(GameAction::CastSpell {
        card_id: bog,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect_err("blockers already declared");
}

/// Table-driven: every card in the batch that just carries the keyword.
#[test]
fn multi_block_keywords_are_printed_as_expected() {
    type Case = (fn() -> crabomination::card::CardDefinition, Keyword);
    let cases: &[Case] = &[
        (catalog::palace_guard, Keyword::CanBlockAnyNumber),
        (catalog::wall_of_glare, Keyword::CanBlockAnyNumber),
        (catalog::avatar_of_hope, Keyword::CanBlockAnyNumber),
        (catalog::ironfist_crusher, Keyword::CanBlockAnyNumber),
        (catalog::foriysian_brigade, Keyword::CanBlockAdditional(1)),
        (catalog::foriysian_interceptor, Keyword::CanBlockAdditional(1)),
        (catalog::selesnya_sagittars, Keyword::CanBlockAdditional(1)),
        (catalog::spike_tailed_ceratops, Keyword::CanBlockAdditional(1)),
        (catalog::two_headed_giant_of_foriys, Keyword::CanBlockAdditional(1)),
        (catalog::ghastbark_twins, Keyword::CanBlockAdditional(1)),
        (catalog::night_market_guard, Keyword::CanBlockAdditional(1)),
        (catalog::two_headed_dragon, Keyword::CanBlockAdditional(1)),
        (catalog::trueheart_duelist, Keyword::CanBlockAdditional(1)),
    ];
    for (factory, kw) in cases {
        let def = factory();
        assert!(def.keywords.contains(kw), "{} is missing {kw:?}", def.name);
    }
}



/// …but it won't over-extend into a swing that kills it.
#[test]
fn bot_multi_block_stops_before_the_blocker_would_die() {
    let mut g = two_player_game();
    combat_with_attackers(&mut g, 3); // three 2/2 bears, 6 power total
    let guard = g.add_card_to_battlefield(0, catalog::palace_guard()); // 1/4
    let blocks = crabomination::server::bot::pick_blocks_for_test(&g, 0);
    let taken = blocks.iter().filter(|(b, _)| *b == guard).count();
    assert!(taken <= 2, "4 toughness soaks at most two 2/2s, got {taken}");
}
