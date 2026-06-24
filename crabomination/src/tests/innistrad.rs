//! Functionality tests for the MID/VOW cards in
//! `catalog::sets::decks::innistrad`.

use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::effects::EffectContext;
use crate::game::types::{Attack, AttackTarget};
use crate::game::*;
use crate::mana::Color;
use crate::TurnStep;

fn ctx0(g: &GameState) -> EffectContext {
    let _ = g;
    EffectContext::for_ability(crate::card::CardId(0), 0, None)
}

fn count_named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

// ── White ──────────────────────────────────────────────────────────────────

/// Unruly Mob grows when another creature you control dies.
#[test]
fn unruly_mob_grows_on_friendly_death() {
    let mut g = two_player_game();
    let mob = g.add_card_to_battlefield(0, catalog::unruly_mob());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(fodder).unwrap().damage = 2; // lethal → CreatureDied
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let r = g.battlefield_find(mob).unwrap();
    assert_eq!(r.counters.get(&CounterType::PlusOnePlusOne).copied(), Some(1));
}

/// Clarion Cathars mints a 1/1 Human on entry.
#[test]
fn clarion_cathars_makes_human() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::clarion_cathars());
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Human"), 1);
}

/// Homestead Courage adds a counter and grants vigilance.
#[test]
fn homestead_courage_counter_and_vigilance() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::homestead_courage().effect, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords.contains(&Keyword::Vigilance));
}

/// Flare of Faith pumps +3/+3 and grants indestructible to a Human.
#[test]
fn flare_of_faith_human_bonus() {
    let mut g = two_player_game();
    let human = g.add_card_to_battlefield(0, catalog::clarion_cathars()); // Human Knight 3/3
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(human)];
    g.resolve_effect(&catalog::flare_of_faith().effect, &ctx).unwrap();
    let cp = g.computed_permanent(human).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 6));
    assert!(cp.keywords.contains(&Keyword::Indestructible));
}

/// Flare of Faith on a non-Human is only +2/+2 with no indestructible.
#[test]
fn flare_of_faith_nonhuman() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::flare_of_faith().effect, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(!cp.keywords.contains(&Keyword::Indestructible));
}

/// Ritual of Hope pumps +2/+1 with coven active.
#[test]
fn ritual_of_hope_coven() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
    g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4 → coven active
    g.resolve_effect(&catalog::ritual_of_hope().effect, &ctx0(&g)).unwrap();
    let cp = g.computed_permanent(a).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3), "+2/+1 under coven");
}

/// Sunset Revelry gains 4 life when an opponent has more life.
#[test]
fn sunset_revelry_lifegain() {
    let mut g = two_player_game();
    g.players[0].life = 10;
    g.players[1].life = 20;
    g.resolve_effect(&catalog::sunset_revelry().effect, &ctx0(&g)).unwrap();
    assert_eq!(g.players[0].life, 14);
}

/// Valorous Stance mode 2 destroys a high-toughness creature.
#[test]
fn valorous_stance_destroys_tough() {
    use crate::card::Effect;
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(big)];
    let Effect::ChooseMode(modes) = catalog::valorous_stance().effect else { panic!() };
    g.resolve_effect(&modes[1], &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_none(), "4-toughness creature destroyed");
}

/// Sanctify destroys an artifact and gains 3 life.
#[test]
fn sanctify_destroy_and_gain() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    let art = g.add_card_to_battlefield(1, catalog::pristine_talisman());
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(art)];
    g.resolve_effect(&catalog::sanctify().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none());
    assert_eq!(g.players[0].life, 23);
}

/// Traveling Minister pumps and gains 1 life.
#[test]
fn traveling_minister_pump_and_gain() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::traveling_minister().activated_abilities[0].effect, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 2));
    assert_eq!(g.players[0].life, 21);
}

/// Resistance Squad draws when you control another Human.
#[test]
fn resistance_squad_draws_with_human() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::clarion_cathars()); // a Human
    g.add_card_to_library(0, catalog::island());
    let hand_before = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::resistance_squad());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

/// Heron of Hope adds 1 to each life-gain event.
#[test]
fn heron_of_hope_lifegain_bonus() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::heron_of_hope());
    g.players[0].life = 20;
    g.adjust_life(0, 3);
    assert_eq!(g.players[0].life, 24, "gained 3 + 1");
}

// ── Blue ───────────────────────────────────────────────────────────────────

/// Larder Zombie taps three creatures to surveil.
#[test]
fn larder_zombie_taps_three() {
    let mut g = two_player_game();
    let z = g.add_card_to_battlefield(0, catalog::larder_zombie());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.perform_action(GameAction::ActivateAbility {
        card_id: z,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    // Three creatures (z + a + b) tapped to pay the cost.
    assert!(g.battlefield_find(z).unwrap().tapped);
    assert!(g.battlefield_find(a).unwrap().tapped);
    assert!(g.battlefield_find(b).unwrap().tapped);
}

/// Startle shrinks a creature, mints a decayed Zombie, and draws.
#[test]
fn startle_full_effect() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.add_card_to_library(0, catalog::island());
    let hand_before = g.players[0].hand.len();
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(foe)];
    g.resolve_effect(&catalog::startle().effect, &ctx).unwrap();
    assert_eq!(g.computed_permanent(foe).unwrap().power, 2, "-2/-0");
    assert_eq!(count_named(&g, 0, "Zombie"), 1);
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

/// Organ Hoarder digs three and keeps one.
#[test]
fn organ_hoarder_dig() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand_before = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::organ_hoarder());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    assert_eq!(g.players[0].graveyard.len(), 2, "rest milled");
}

/// Dissipate counters a spell to exile rather than the graveyard.
#[test]
fn dissipate_exiles() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .unwrap();
    let dis = g.add_card_to_hand(0, catalog::dissipate());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: dis,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().all(|c| c.id != bolt), "not in graveyard");
    assert!(g.exile.iter().any(|c| c.id == bolt), "exiled instead");
}

/// Vivisection sacrifices a creature and draws three.
#[test]
fn vivisection_sac_and_draw() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand_before = g.players[0].hand.len();
    g.resolve_effect(&catalog::vivisection().effect, &ctx0(&g)).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "creature sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 3);
}

// ── Black ──────────────────────────────────────────────────────────────────

/// Novice Occultist draws and loses 1 life on death.
#[test]
fn novice_occultist_death_value() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    g.add_card_to_library(0, catalog::island());
    let occ = g.add_card_to_battlefield(0, catalog::novice_occultist());
    let hand_before = g.players[0].hand.len();
    g.battlefield_find_mut(occ).unwrap().damage = 2; // lethal
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    assert_eq!(g.players[0].life, 19);
}

/// Siege Zombie drains each opponent for tapping three.
#[test]
fn siege_zombie_drains() {
    let mut g = two_player_game();
    let z = g.add_card_to_battlefield(0, catalog::siege_zombie());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[1].life = 20;
    g.perform_action(GameAction::ActivateAbility {
        card_id: z,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19);
}

/// Blood Pact makes the target draw two and lose 2.
#[test]
fn blood_pact_draw_lose() {
    let mut g = two_player_game();
    g.players[1].life = 20;
    for _ in 0..2 {
        g.add_card_to_library(1, catalog::island());
    }
    let hand_before = g.players[1].hand.len();
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Player(1)];
    g.resolve_effect(&catalog::blood_pact().effect, &ctx).unwrap();
    assert_eq!(g.players[1].hand.len(), hand_before + 2);
    assert_eq!(g.players[1].life, 18);
}

/// Bat Whisperer makes a Bat when an opponent lost life this turn.
#[test]
fn bat_whisperer_makes_bat() {
    let mut g = two_player_game();
    g.players[1].lost_life_this_turn = true;
    g.move_card_to_battlefield_for_test(0, catalog::bat_whisperer());
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Bat"), 1);
}

/// Arrogant Outlaw drains 2 when an opponent lost life this turn.
#[test]
fn arrogant_outlaw_drain() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    g.players[1].life = 20;
    g.players[1].lost_life_this_turn = true;
    g.move_card_to_battlefield_for_test(0, catalog::arrogant_outlaw());
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
    assert_eq!(g.players[0].life, 22);
}

/// Eaten Alive sacrifices a creature and exiles the target.
#[test]
fn eaten_alive_exiles() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel());
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(foe)];
    g.resolve_effect(&catalog::eaten_alive().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed");
    assert!(g.exile.iter().any(|c| c.id == foe), "target exiled");
}

/// Gluttonous Guest mints a Blood on entry and gains life when one is sacked.
#[test]
fn gluttonous_guest_blood_and_lifegain() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    g.move_card_to_battlefield_for_test(0, catalog::gluttonous_guest());
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Blood"), 1);
    let blood = g.battlefield.iter().find(|c| c.definition.name == "Blood").unwrap().id;
    let _ = blood;
    let evs = g
        .resolve_effect(
            &crate::card::Effect::Sacrifice {
                who: crate::card::Selector::You,
                count: crate::card::Value::Const(1),
                filter: crate::card::SelectionRequirement::HasArtifactSubtype(
                    crate::card::ArtifactSubtype::Blood,
                ),
            },
            &ctx0(&g),
        )
        .unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 21, "1 life from sacking Blood");
}

/// Demonic Bargain exiles 13 and tutors a card.
#[test]
fn demonic_bargain_tutors() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    for _ in 0..14 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_library(0, catalog::serra_angel());
    let target = g.players[0].library.last().map(|c| c.id).unwrap();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target))]));
    let hand_before = g.players[0].hand.len();
    g.resolve_effect(&catalog::demonic_bargain().effect, &ctx0(&g)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.exile.len(), 13, "13 exiled");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "tutored one card");
}

// ── Green ──────────────────────────────────────────────────────────────────

/// Timberland Guide puts a counter on a target on entry.
#[test]
fn timberland_guide_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::timberland_guide().triggered_abilities[0].effect, &ctx).unwrap();
    assert_eq!(
        g.battlefield_find(bear).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(1)
    );
}

/// Pestilent Wolf grants itself deathtouch.
#[test]
fn pestilent_wolf_deathtouch() {
    let mut g = two_player_game();
    let wolf = g.add_card_to_battlefield(0, catalog::pestilent_wolf());
    let mut ctx = EffectContext::for_ability(wolf, 0, None);
    g.resolve_effect(&catalog::pestilent_wolf().activated_abilities[0].effect, &ctx).unwrap();
    assert!(g.computed_permanent(wolf).unwrap().keywords.contains(&Keyword::Deathtouch));
}

/// Brood Weaver leaves a Spider behind.
#[test]
fn brood_weaver_dies_to_spider() {
    let mut g = two_player_game();
    let w = g.add_card_to_battlefield(0, catalog::brood_weaver());
    g.battlefield_find_mut(w).unwrap().damage = 4; // lethal vs 2/4
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Spider"), 1);
}

/// Clear Shot fights one-sidedly using the pumped power.
#[test]
fn clear_shot_one_sided_fight() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 3/3
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(mine), Target::Permanent(foe)];
    g.resolve_effect(&catalog::clear_shot().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "took 3 damage, dead");
}

/// Crushing Canopy mode 1 destroys a flyer.
#[test]
fn crushing_canopy_kills_flyer() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let flyer = g.add_card_to_battlefield(1, catalog::serra_angel()); // flying
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(flyer)];
    g.resolve_effect(&catalog::crushing_canopy().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(flyer).is_none());
}

/// Rural Recruit makes a 3/1 Boar.
#[test]
fn rural_recruit_makes_boar() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::rural_recruit());
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Boar"), 1);
}

/// Hamlet Vanguard enters with 2 counters per other Human.
#[test]
fn hamlet_vanguard_scales_with_humans() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::clarion_cathars()); // a Human
    let hv = g.move_card_to_battlefield_for_test(0, catalog::hamlet_vanguard());
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(hv).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(2)
    );
}

/// Splendid Reclamation returns lands from the graveyard tapped.
#[test]
fn splendid_reclamation_returns_lands() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::island());
    g.add_card_to_graveyard(0, catalog::forest());
    g.resolve_effect(&catalog::splendid_reclamation().effect, &ctx0(&g)).unwrap();
    drain_stack(&mut g);
    let lands = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    assert_eq!(lands, 2, "two lands returned");
    assert!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).all(|c| c.tapped));
}

// ── Red ────────────────────────────────────────────────────────────────────

/// Voldaren Stinger has first strike only while attacking.
#[test]
fn voldaren_stinger_conditional_first_strike() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::voldaren_stinger());
    g.clear_sickness(s);
    assert!(
        !g.computed_permanent(s).unwrap().keywords.contains(&Keyword::FirstStrike),
        "no first strike at rest"
    );
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: s,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    assert!(
        g.computed_permanent(s).unwrap().keywords.contains(&Keyword::FirstStrike),
        "first strike while attacking"
    );
}

/// Daybreak Combatants pumps a target on entry.
#[test]
fn daybreak_combatants_pump() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::daybreak_combatants().triggered_abilities[0].effect, &ctx).unwrap();
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4);
}

/// Neonate's Rush pings creature + controller and draws.
#[test]
fn neonates_rush_pings() {
    let mut g = two_player_game();
    g.players[1].life = 20;
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let hand_before = g.players[0].hand.len();
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(foe)];
    g.resolve_effect(&catalog::neonates_rush().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "controller pinged");
    assert_eq!(g.battlefield_find(foe).unwrap().damage, 1);
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

/// Rending Flame deals 2 extra to a Spirit's controller.
#[test]
fn rending_flame_spirit_bonus() {
    let mut g = two_player_game();
    g.players[1].life = 20;
    let mut def = catalog::grizzly_bears();
    def.subtypes.creature_types = vec![CreatureType::Spirit];
    let sp = g.add_card_to_battlefield(1, def);
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(sp)];
    g.resolve_effect(&catalog::rending_flame().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "spirit controller took 2");
}

/// End the Festivities pings each opponent and their board.
#[test]
fn end_the_festivities_sweeps_opponent() {
    let mut g = two_player_game();
    g.players[1].life = 20;
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.resolve_effect(&catalog::end_the_festivities().effect, &ctx0(&g)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19);
    assert_eq!(g.battlefield_find(foe).unwrap().damage, 1);
}

/// Belligerent Guest's combat-damage trigger mints a Blood token.
#[test]
fn belligerent_guest_blood_on_hit() {
    let mut g = two_player_game();
    let bg = g.add_card_to_battlefield(0, catalog::belligerent_guest());
    let mut ctx = EffectContext::for_ability(bg, 0, None);
    g.resolve_effect(&catalog::belligerent_guest().triggered_abilities[0].effect, &ctx).unwrap();
    assert_eq!(count_named(&g, 0, "Blood"), 1);
}

/// Frenzied Devils pumps itself on a noncreature spell.
#[test]
fn frenzied_devils_pumps_on_noncreature() {
    let mut g = two_player_game();
    let fd = g.add_card_to_battlefield(0, catalog::frenzied_devils());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(fd).unwrap().power, 5, "+2/+2 from noncreature cast");
}
