//! Commander: one-card gaps of EDHREC ranks 5501-6500
//! (`decks::cmdr_rank6k_a`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::effect::Effect;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState) {
    for c in [
        Color::White,
        Color::Blue,
        Color::Black,
        Color::Red,
        Color::Green,
    ] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn act(g: &mut GameState, action: GameAction) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(action).expect("action");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    act(
        g,
        GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        },
    );
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    act(
        g,
        GameAction::ActivateAbility {
            card_id: id,
            ability_index: index,
            target,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        },
    );
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne)
}

fn kill(g: &mut GameState, id: CardId) {
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(g, murder, Some(Target::Permanent(id)));
}

fn resolve_at(
    g: &mut GameState,
    effect: &Effect,
    source: CardId,
    target: Option<Target>,
    mode: usize,
) {
    let mut ctx = EffectContext::for_spell(0, target, mode, 0);
    ctx.source = Some(source);
    let evs = g.resolve_effect(effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn declare(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attacks
            .into_iter()
            .map(|(attacker, p)| Attack {
                attacker,
                target: AttackTarget::Player(p),
            })
            .collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn finish_combat(g: &mut GameState) {
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![]))
        .expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
}
/// CR 402.2 — Ms. Marvel waives the hand cap and draws when you target your
/// creature.
#[test]
fn ms_marvel_draws_on_a_targeted_ally() {
    let mut g = pod(2);
    let mm = ready(&mut g, 0, catalog::ms_marvel_kamala_khan());
    g.add_card_to_library(0, catalog::island());
    let growth = g.add_card_to_hand(0, catalog::giant_growth());
    cast(&mut g, growth, Some(Target::Permanent(mm)));
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(
        pt(&g, mm),
        (1 + 3, 7),
        "base power = hand size (1), +3 Giant Growth"
    );
    // The gained ability keeps reading the hand.
    g.add_card_to_hand(0, catalog::island());
    assert_eq!(pt(&g, mm), (2 + 3, 7));
}

/// CR 701.21a — She-Hulk eats a land for a card and a counter.
#[test]
fn she_hulk_sacrifices_a_land() {
    let mut g = pod(2);
    let sh = ready(&mut g, 0, catalog::she_hulk_jennifer_walters());
    ready(&mut g, 0, catalog::mountain());
    g.add_card_to_library(0, catalog::island());
    activate(&mut g, sh, 0, None);
    assert_eq!(plus(&g, sh), 1);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// CR 301.5 — the Daggers are an indestructible +2/+2.
#[test]
fn vibranium_daggers_pump() {
    let mut g = pod(2);
    let d = ready(&mut g, 0, catalog::vibranium_energy_daggers());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    act(
        &mut g,
        GameAction::Equip {
            equipment: d,
            target: bear,
        },
    );
    assert_eq!(pt(&g, bear), (4, 4));
    assert!(
        g.computed_permanent(d)
            .unwrap()
            .keywords()
            .contains(&Keyword::Indestructible)
    );
}

/// CR 305.6 — Ice Tunnel is a snow Island Swamp that enters tapped.
#[test]
fn ice_tunnel_enters_tapped() {
    let mut g = pod(2);
    let land = g.add_card_to_hand(0, catalog::ice_tunnel());
    g.perform_action(GameAction::PlayLand(land)).expect("land");
    assert!(g.battlefield_find(land).unwrap().tapped);
}

/// CR 120.3 — Red Guardian destroys an opposing creature that dealt damage this
/// turn, and can't target one that didn't.
#[test]
fn red_guardian_punishes_a_damage_dealer() {
    let mut g = pod(2);
    let hitter = ready(&mut g, 1, catalog::grizzly_bears());
    let idle = ready(&mut g, 1, catalog::grizzly_bears());
    let rg = ready(&mut g, 0, catalog::red_guardian_super_soldier());
    let etb = catalog::red_guardian_super_soldier().triggered_abilities[0]
        .effect
        .clone();
    let bolt = Effect::DealDamage {
        to: crabomination::effect::Selector::Target(0),
        amount: crabomination::effect::Value::Const(1),
    };
    let mut ctx = EffectContext::for_spell(1, Some(Target::Player(0)), 0, 0);
    ctx.source = Some(hitter);
    g.resolve_effect(&bolt, &ctx).unwrap();
    resolve_at(&mut g, &etb, rg, Some(Target::Permanent(hitter)), 0);
    assert!(g.battlefield_find(hitter).is_none());
    let filter = crabomination::card::SelectionRequirement::DealtDamageThisTurnAsSource;
    assert!(
        !g.evaluate_requirement_static(&filter, &Target::Permanent(idle), 0, Some(rg)),
        "it dealt no damage"
    );
}

/// CR 611.3a — Victor Mancha replays a graveyard card while he stays.
#[test]
fn victor_mancha_replays_a_graveyard_card() {
    let mut g = pod(2);
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    let vm = ready(&mut g, 0, catalog::victor_mancha_runaway());
    let etb = catalog::victor_mancha_runaway().triggered_abilities[0]
        .effect
        .clone();
    resolve_at(&mut g, &etb, vm, Some(Target::Permanent(ring)), 0);
    flood(&mut g);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: ring,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from exile");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_some());
}

/// CR 611.2b — "for as long as you control Victor Mancha": a steal ends the
/// grant, and it stays ended when Victor comes back.
#[test]
fn victor_mancha_grant_ends_when_control_is_lost() {
    use crabomination::effect::{Duration, Selector};
    let mut g = pod(2);
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    let vm = ready(&mut g, 0, catalog::victor_mancha_runaway());
    let etb = catalog::victor_mancha_runaway().triggered_abilities[0].effect.clone();
    resolve_at(&mut g, &etb, vm, Some(Target::Permanent(ring)), 0);
    let granted = |g: &GameState| g.exile.iter().any(|c| c.id == ring && c.may_play_until.is_some());
    assert!(granted(&g));
    let steal = |g: &mut GameState, to: usize| {
        let ctx = EffectContext::for_spell(to, Some(Target::Permanent(vm)), 0, 0);
        let evs = g
            .resolve_effect(&Effect::GainControl { what: Selector::Target(0), to: None, duration: Duration::Permanent }, &ctx)
            .unwrap();
        g.dispatch_triggers_for_events(&evs);
    };
    steal(&mut g, 1);
    assert_eq!(g.battlefield_find(vm).unwrap().controller, 1);
    assert!(!granted(&g), "seat 0 no longer controls Victor");
    steal(&mut g, 0);
    assert!(!granted(&g), "control coming back does not restart it");
}

/// CR 701.14a — Yoshimaru's entry makes a friendly creature fight.
#[test]
fn yoshimaru_starts_a_fight() {
    let mut g = pod(2);
    let angel = ready(&mut g, 0, catalog::serra_angel());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let y = ready(&mut g, 0, catalog::yoshimaru_scrappy_stray());
    let etb = catalog::yoshimaru_scrappy_stray().triggered_abilities[0]
        .effect
        .clone();
    let mut ctx = EffectContext::for_spell(0, Some(Target::Permanent(angel)), 0, 0);
    ctx.targets.push(Target::Permanent(bear));
    ctx.source = Some(y);
    let evs = g.resolve_effect(&etb, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    activate(&mut g, y, 0, Some(Target::Permanent(angel)));
    assert_eq!(plus(&g, angel), 1);
}

/// CR 113.6 — Skill Borrower wields the top card's activated abilities.
#[test]
fn skill_borrower_borrows_from_the_top() {
    let mut g = pod(2);
    let sb = ready(&mut g, 0, catalog::skill_borrower());
    assert!(g.granted_abilities_for(sb).is_empty());
    g.add_card_to_library(0, catalog::llanowar_elves());
    assert!(!g.granted_abilities_for(sb).is_empty());
}

/// CR 303.4 — Cartouche of Ambition shrinks a foe and buffs its host.
#[test]
fn cartouche_of_ambition_buffs_and_shrinks() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let foe = ready(&mut g, 1, catalog::grizzly_bears());
    let c = g.add_card_to_hand(0, catalog::cartouche_of_ambition());
    cast(&mut g, c, Some(Target::Permanent(bear)));
    assert_eq!(pt(&g, bear), (3, 3));
    assert!(
        g.computed_permanent(bear)
            .unwrap()
            .keywords()
            .contains(&Keyword::Lifelink)
    );
    assert_eq!(pt(&g, foe), (1, 1), "the entry's -1/-1 counter");
}

/// CR 610.3 — Henchbots holds a tapped foe until it leaves.
#[test]
fn henchbots_holds_a_tapped_creature() {
    let mut g = pod(2);
    let foe = ready(&mut g, 1, catalog::grizzly_bears());
    g.battlefield_find_mut(foe).unwrap().tapped = true;
    let hb = ready(&mut g, 0, catalog::henchbots());
    let etb = catalog::henchbots().triggered_abilities[0].effect.clone();
    resolve_at(&mut g, &etb, hb, Some(Target::Permanent(foe)), 0);
    assert!(g.battlefield_find(foe).is_none());
    kill(&mut g, hb);
    assert!(
        g.battlefield
            .iter()
            .any(|c| c.definition.name == "Grizzly Bears" && c.controller == 1)
    );
}

/// CR 510.1c — Rasaad's creatures deal combat damage by toughness.
#[test]
fn rasaad_hits_with_toughness() {
    let mut g = pod(2);
    let r = ready(&mut g, 0, catalog::rasaad_yn_bashir());
    declare(&mut g, vec![(r, 1)]);
    finish_combat(&mut g);
    assert_eq!(g.players[1].life, 17);
}
