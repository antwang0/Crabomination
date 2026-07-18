//! Functionality tests for `catalog::sets::decks::recent265` (DMU/SNC/MID/NEO).

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::{drain_stack, two_player_game};

fn kw(g: &crabomination::game::GameState, id: crabomination::card::CardId, k: Keyword) -> bool {
    g.computed_permanent(id).is_some_and(|cp| cp.keywords.contains(&k))
}

/// A resolution context whose source (and kicked flag) is `id`.
fn src_ctx(controller: usize, id: crabomination::card::CardId, kicked: bool) -> EffectContext {
    let mut ctx = EffectContext::for_spell(controller, None, 0, 0);
    ctx.source = Some(id);
    ctx.kicked = kicked;
    ctx
}

/// Bonebreaker Giant is a vanilla 4/4.
#[test]
fn bonebreaker_giant_is_a_vanilla_4_4() {
    let d = catalog::bonebreaker_giant();
    assert_eq!((d.power, d.toughness), (4, 4));
    assert!(d.keywords.is_empty() && d.triggered_abilities.is_empty());
}

/// Gnottvold Recluse has reach.
#[test]
fn gnottvold_recluse_has_reach() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::gnottvold_recluse());
    assert!(kw(&g, id, Keyword::Reach));
}

/// Deathbloom Gardener taps for any color.
#[test]
fn deathbloom_gardener_makes_mana() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::deathbloom_gardener());
    g.clear_sickness(id);
    assert!(kw(&g, id, Keyword::Deathtouch));
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0,
        target: None, additional_targets: vec![], x_value: None,
    }).unwrap();
    let total: u32 = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green]
        .iter().map(|c| g.players[0].mana_pool.amount(*c)).sum::<u32>()
        + g.players[0].mana_pool.colorless_amount();
    assert_eq!(total, 1, "produced one mana");
}

/// Battlefly Swarm buys deathtouch with {B}.
#[test]
fn battlefly_swarm_grants_deathtouch() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::battlefly_swarm());
    assert!(kw(&g, id, Keyword::Flying));
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(kw(&g, id, Keyword::Deathtouch));
}

/// Duct Crawler stops a creature from blocking it this turn.
#[test]
fn duct_crawler_locks_a_blocker() {
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let crawler = g.add_card_to_battlefield(0, catalog::duct_crawler());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(crawler);
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: crawler, ability_index: 0,
        target: Some(Target::Permanent(blocker)), additional_targets: vec![], x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: crawler, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(blocker, crawler)])).is_err(),
        "the locked creature can't block the crawler"
    );
}

/// Charismatic Vanguard pumps the whole team +1/+1.
#[test]
fn charismatic_vanguard_pumps_team() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let van = g.add_card_to_battlefield(0, catalog::charismatic_vanguard());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: van, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    let cp = g.computed_permanent(ally).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "ally pumped to 3/3");
}

/// Cabaretti Initiate buys double strike with its hybrid ability.
#[test]
fn cabaretti_initiate_gains_double_strike() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::cabaretti_initiate());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(kw(&g, id, Keyword::DoubleStrike));
}

/// Serpent-Blade Assailant's Backup puts a counter on and grants deathtouch.
#[test]
fn serpent_blade_backup_buffs_ally() {
    let mut g = two_player_game();
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let effect = catalog::serpent_blade_assailant().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(ally)), 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(
        g.battlefield_find(ally).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "+1/+1 counter placed"
    );
    assert!(kw(&g, ally, Keyword::Deathtouch), "granted deathtouch");
}

/// Rhox Pikemaster gives other Soldiers first strike.
#[test]
fn rhox_pikemaster_soldier_anthem() {
    let mut g = two_player_game();
    let _rhox = g.add_card_to_battlefield(0, catalog::rhox_pikemaster());
    let soldier = g.add_card_to_battlefield(0, catalog::conscripted_infantry()); // Soldier
    let nonsoldier = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(kw(&g, soldier, Keyword::FirstStrike), "other Soldier gets first strike");
    assert!(!kw(&g, nonsoldier, Keyword::FirstStrike), "non-Soldier unaffected");
}

/// Witty Roastmaster pings each opponent when another creature enters.
#[test]
fn witty_roastmaster_pings_on_creature_etb() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::witty_roastmaster());
    let before = g.players[1].life;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a creature");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 1, "1 damage to the opponent");
}

/// Yavimaya Iconoclast pumps itself when kicked.
#[test]
fn yavimaya_iconoclast_kicked_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::yavimaya_iconoclast());
    let effect = catalog::yavimaya_iconoclast().triggered_abilities[0].effect.clone();
    let ctx = src_ctx(0, id, true);
    g.resolve_effect(&effect, &ctx).unwrap();
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3), "+1/+1 when kicked");
    assert!(cp.keywords.contains(&Keyword::Haste));
}

/// Vineshaper Prodigy digs three when kicked.
#[test]
fn vineshaper_prodigy_kicked_digs() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::vineshaper_prodigy());
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let hand_before = g.players[0].hand.len();
    let effect = catalog::vineshaper_prodigy().triggered_abilities[0].effect.clone();
    let ctx = src_ctx(0, id, true);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "took one card to hand");
}

/// Shield-Wall Sentinel has defender and can tutor a defender to hand.
#[test]
fn shield_wall_sentinel_tutors_defender() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let wall = g.add_card_to_library(0, catalog::wall_of_omens());
    let sentinel_def = catalog::shield_wall_sentinel();
    assert!(sentinel_def.keywords.contains(&Keyword::Defender));
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(wall)),
    ]));
    let effect = sentinel_def.triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.players[0].hand.iter().any(|c| c.id == wall), "defender fetched to hand");
}

/// Kami of Industry reanimates a cheap artifact, hastes it, and sacs it at end.
#[test]
fn kami_of_industry_reanimates_artifact() {
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    let relic = g.add_card_to_graveyard(0, catalog::mind_stone()); // MV 2 artifact
    let effect = catalog::kami_of_industry().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(relic)), 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.battlefield_find(relic).is_some(), "artifact reanimated");
    assert!(kw(&g, relic, Keyword::Haste), "gained haste");
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(relic).is_none(), "sacrificed at the end step");
}

/// Wingmantle Chaplain mints a Bird per defender on ETB.
#[test]
fn wingmantle_chaplain_makes_birds_per_defender() {
    let mut g = two_player_game();
    // Two defenders already out (plus the Chaplain would be a third once it enters).
    g.add_card_to_battlefield(0, catalog::wall_of_omens());
    g.add_card_to_battlefield(0, catalog::shield_wall_sentinel());
    let chaplain = g.add_card_to_battlefield(0, catalog::wingmantle_chaplain());
    let effect = catalog::wingmantle_chaplain().triggered_abilities[0].effect.clone();
    let ctx = src_ctx(0, chaplain, false);
    g.resolve_effect(&effect, &ctx).unwrap();
    let birds = g.battlefield.iter().filter(|c| c.definition.name == "Bird").count();
    assert_eq!(birds, 3, "one Bird per defender (two walls + the Chaplain)");
}
