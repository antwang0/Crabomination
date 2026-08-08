//! Tests for the recent302 Dissension gap batch.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game, GameAction};
use crabomination::mana::Color;

#[test]
fn haazda_exonerator_sacs_to_destroy_an_aura() {
    let mut g = two_player_game();
    let ex = g.add_card_to_battlefield(0, catalog::haazda_exonerator());
    g.clear_sickness(ex);
    let aura = g.add_card_to_battlefield(1, catalog::pacifism());
    g.perform_action(GameAction::ActivateAbility {
        card_id: ex, ability_index: 0, target: Some(Target::Permanent(aura)),
        additional_targets: vec![], x_value: None, mode: None,
    }).expect("sac to destroy the Aura");
    drain_stack(&mut g);
    assert!(g.battlefield_find(aura).is_none(), "Aura destroyed");
    assert!(g.battlefield_find(ex).is_none(), "Exonerator sacrificed as a cost");
}

#[test]
fn ogre_gatecrasher_destroys_a_defender() {
    let mut g = two_player_game();
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_omens());
    let og = g.add_card_to_battlefield(0, catalog::ogre_gatecrasher());
    g.fire_self_etb_triggers(og, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(wall).is_none(), "defender destroyed on ETB");
}

#[test]
fn whiptail_moloch_shoots_your_own_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wm = g.add_card_to_battlefield(0, catalog::whiptail_moloch());
    g.fire_self_etb_triggers(wm, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "your own 2/2 took 3 and died");
}

#[test]
fn utvara_scalper_flies_and_must_attack() {
    let mut g = two_player_game();
    let us = g.add_card_to_battlefield(0, catalog::utvara_scalper());
    let kw = g.computed_permanent(us).unwrap().keywords.clone();
    assert!(kw.contains(&Keyword::Flying) && kw.contains(&Keyword::MustAttack));
}

#[test]
fn gnat_alley_creeper_dodges_flyers() {
    let mut g = two_player_game();
    let gc = g.add_card_to_battlefield(0, catalog::gnat_alley_creeper());
    assert!(g.computed_permanent(gc).unwrap().keywords.iter().any(|k| matches!(
        k,
        Keyword::CantBeBlockedBy(_)
    )));
}

#[test]
fn silkwing_scout_sacs_to_fetch_a_basic() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    let ss = g.add_card_to_battlefield(0, catalog::silkwing_scout());
    g.clear_sickness(ss);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let lands_before = g.battlefield.iter().filter(|c| c.definition.is_land()).count();
    g.perform_action(GameAction::ActivateAbility {
        card_id: ss, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("sac to fetch");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ss).is_none(), "sacrificed as a cost");
    let lands_after = g.battlefield.iter().filter(|c| c.definition.is_land()).count();
    assert_eq!(lands_after, lands_before + 1, "a basic entered the battlefield");
}

#[test]
fn vesper_ghoul_taps_and_bleeds_for_any_color() {
    let mut g = two_player_game();
    let vg = g.add_card_to_battlefield(0, catalog::vesper_ghoul());
    g.clear_sickness(vg);
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: vg, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("mana ability");
    assert_eq!(g.players[0].life, life - 1, "paid 1 life");
    assert!(g.players[0].mana_pool.total() >= 1, "produced a mana");
}

#[test]
fn patagia_viper_needs_blue_and_broods_snakes() {
    // Blue spent → sticks around; two Snakes made.
    let mut g = two_player_game();
    let pv = g.add_card_to_hand(0, catalog::patagia_viper());
    // {3}{G}: pay {G} with green, one generic pip with blue so {U} is spent.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: pv, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast with blue");
    drain_stack(&mut g);
    assert!(g.battlefield_find(pv).is_some(), "blue spent → stays");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Snake").count(), 2, "two Snakes");
}

#[test]
fn squealing_devil_pumps_and_needs_black() {
    // No black spent → sacrifices itself.
    let mut g = two_player_game();
    let sd = g.add_card_to_hand(0, catalog::squealing_devil());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: sd, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast without black");
    drain_stack(&mut g);
    assert!(g.battlefield_find(sd).is_none(), "no black spent → sacrificed");
    assert!(g.computed_permanent(sd).is_none());
}

#[test]
fn slaughterhouse_bouncer_shrinks_a_creature_when_hellbent() {
    use crabomination::effect::{Effect, Selector};
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    let bouncer = g.add_card_to_battlefield(0, catalog::slaughterhouse_bouncer());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(g.players[0].hand.is_empty(), "hellbent");
    // Kill the Bouncer; its death trigger fires with hand empty.
    let ctx = EffectContext::for_ability(bouncer, 0, Some(Target::Permanent(bouncer)));
    let evs = g.resolve_effect(&Effect::SacrificePermanent { what: Selector::Target(0) }, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    // Auto-target picks the only other creature (foe).
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "a 2/2 given -3/-3 dies");
}

#[test]
fn transguild_courier_is_all_colors() {
    let mut g = two_player_game();
    let tc = g.add_card_to_battlefield(0, catalog::transguild_courier());
    let colors = g.computed_permanent(tc).unwrap().colors.clone();
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        assert!(colors.contains(&c), "is {c:?}");
    }
}

#[test]
fn wakestone_gargoyle_lets_defenders_attack() {
    use crabomination::TurnStep;
    let mut g = two_player_game();
    let wg = g.add_card_to_battlefield(0, catalog::wakestone_gargoyle());
    g.clear_sickness(wg);
    g.active_player_idx = 0;
    assert!(g.computed_permanent(wg).unwrap().keywords.contains(&Keyword::Defender));
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wg, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("drop defender for the team");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    // It can now be declared as an attacker despite Defender.
    assert!(g.legal_attackers(0).contains(&wg), "defender lifted this turn");
}
