//! Functionality tests for the MID/VOW cards in
//! `catalog::sets::decks::innistrad`.

use crabomination::card::{CardType, CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget};
use crabomination::game::*;
use crabomination::mana::Color;
use crabomination::TurnStep;

fn ctx0(g: &GameState) -> EffectContext {
    let _ = g;
    EffectContext::for_ability(crabomination::card::CardId(0), 0, None)
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
    use crabomination::card::Effect;
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

/// Search Party Captain costs {1} less per creature you attacked with this turn.
#[test]
fn search_party_captain_cost_reduction() {
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    let spell = crabomination::card::CardInstance::new(g.next_id(), catalog::search_party_captain(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 0, "no attacks → full price");
    g.players[0].creatures_attacked_this_turn = 2;
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 2, "two attackers → 2 off");
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
        x_value: None, mode: None,
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
        x_value: None, mode: None,
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
            &crabomination::card::Effect::Sacrifice {
                who: crabomination::card::Selector::You,
                count: crabomination::card::Value::Const(1),
                filter: crabomination::card::SelectionRequirement::HasArtifactSubtype(
                    crabomination::card::ArtifactSubtype::Blood,
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
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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
    let ctx = EffectContext::for_ability(wolf, 0, None);
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
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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
    let ctx = EffectContext::for_ability(bg, 0, None);
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

// ── Batch 2 ──────────────────────────────────────────────────────────────────

/// Component Collector becomes day on entry when it's neither day nor night.
#[test]
fn component_collector_becomes_day() {
    use crabomination::game::types::DayNight;
    let mut g = two_player_game();
    assert_eq!(g.day_night, None);
    g.move_card_to_battlefield_for_test(0, catalog::component_collector());
    drain_stack(&mut g);
    assert_eq!(g.day_night, Some(DayNight::Day));
}

/// Stromkirk Bloodthief's end-step trigger grows a Vampire.
#[test]
fn stromkirk_bloodthief_counters_vampire() {
    let mut g = two_player_game();
    let sb = g.add_card_to_battlefield(0, catalog::stromkirk_bloodthief()); // a Vampire
    let mut ctx = EffectContext::for_ability(sb, 0, None);
    ctx.targets = vec![Target::Permanent(sb)];
    g.resolve_effect(&catalog::stromkirk_bloodthief().triggered_abilities[0].effect, &ctx).unwrap();
    assert_eq!(
        g.battlefield_find(sb).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(1)
    );
}

/// Voldaren Ambusher deals damage equal to your Vampire count when an opponent
/// lost life this turn.
#[test]
fn voldaren_ambusher_pings_for_vampires() {
    let mut g = two_player_game();
    g.players[1].lost_life_this_turn = true;
    let amb = g.add_card_to_battlefield(0, catalog::voldaren_ambusher()); // Vampire #1
    g.add_card_to_battlefield(0, catalog::voldaren_ambusher()); // Vampire #2
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel());
    let mut ctx = EffectContext::for_ability(amb, 0, None);
    ctx.targets = vec![Target::Permanent(foe)];
    g.resolve_effect(&catalog::voldaren_ambusher().triggered_abilities[0].effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(foe).unwrap().damage, 2, "2 Vampires → 2 damage");
}

/// Chill of the Grave taps a creature, locks its next untap, and draws.
#[test]
fn chill_of_the_grave_taps_locks_draws() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let hand_before = g.players[0].hand.len();
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(foe)];
    g.resolve_effect(&catalog::chill_of_the_grave().effect, &ctx).unwrap();
    assert!(g.battlefield_find(foe).unwrap().tapped, "tapped");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
}

/// Chill of the Grave costs {1} less if you control a Zombie.
#[test]
fn chill_of_the_grave_zombie_discount() {
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    let spell = crabomination::card::CardInstance::new(g.next_id(), catalog::chill_of_the_grave(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 0);
    g.add_card_to_battlefield(0, catalog::larder_zombie()); // a Zombie
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 1, "Zombie → {{1}} off");
}

/// Bloody Betrayal steals a creature, untaps it, and grants haste + Blood.
#[test]
fn bloody_betrayal_steals_and_hastes() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(foe).unwrap().tapped = true;
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(foe)];
    g.resolve_effect(&catalog::bloody_betrayal().effect, &ctx).unwrap();
    let c = g.battlefield_find(foe).unwrap();
    assert_eq!(c.controller, 0, "control stolen");
    assert!(!c.tapped, "untapped");
    assert!(g.computed_permanent(foe).unwrap().keywords.contains(&Keyword::Haste));
    assert_eq!(count_named(&g, 0, "Blood"), 1);
}

/// Wolf Strike fights one-sidedly; at night the attacker is pumped first.
#[test]
fn wolf_strike_night_pump_fight() {
    use crabomination::game::types::DayNight;
    let mut g = two_player_game();
    g.day_night = Some(DayNight::Night);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 4/2 at night
    let foe = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(mine), Target::Permanent(foe)];
    g.resolve_effect(&catalog::wolf_strike().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "4 damage at night kills the 3/3");
}

/// Fateful Absence destroys the target and gives its controller a Clue.
#[test]
fn fateful_absence_destroys_and_clues() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel());
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(foe)];
    g.resolve_effect(&catalog::fateful_absence().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "destroyed");
    assert_eq!(count_named(&g, 1, "Clue"), 1, "controller investigated");
}

/// Bloodline Culling mode 1 gives a creature -5/-5.
#[test]
fn bloodline_culling_minus_five() {
    use crabomination::card::Effect;
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(foe)];
    let Effect::ChooseMode(modes) = catalog::bloodline_culling().effect else { panic!() };
    g.resolve_effect(&modes[0], &ctx).unwrap();
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "-5/-5 kills the 4/4");
}

/// Gavony Dawnguard becomes day on entry.
#[test]
fn gavony_dawnguard_becomes_day() {
    use crabomination::game::types::DayNight;
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::gavony_dawnguard());
    drain_stack(&mut g);
    assert_eq!(g.day_night, Some(DayNight::Day));
}

/// Abandon the Post stops up to two creatures from blocking.
#[test]
fn abandon_the_post_cant_block() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::hill_giant());
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(a), Target::Permanent(b)];
    g.resolve_effect(&catalog::abandon_the_post().effect, &ctx).unwrap();
    assert!(g.computed_permanent(a).unwrap().keywords.contains(&Keyword::CantBlock));
    assert!(g.computed_permanent(b).unwrap().keywords.contains(&Keyword::CantBlock));
}

/// Blood Fountain mints a Blood token on entry and returns creatures from the
/// graveyard via its sacrifice ability.
#[test]
fn blood_fountain_etb_and_recur() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::blood_fountain());
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Blood"), 1, "ETB Blood");
    let bf = g.battlefield.iter().find(|c| c.definition.name == "Blood Fountain").map(|c| c.id).unwrap();
    let corpse = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let mut ctx = EffectContext::for_ability(bf, 0, None);
    ctx.targets = vec![Target::Permanent(corpse)];
    g.resolve_effect(&catalog::blood_fountain().activated_abilities[0].effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == corpse), "creature returned to hand");
}

// ── Batch 3 ──────────────────────────────────────────────────────────────────

/// Lightning Wolf grants itself first strike.
#[test]
fn lightning_wolf_first_strike() {
    let mut g = two_player_game();
    let w = g.add_card_to_battlefield(0, catalog::lightning_wolf());
    let ctx = EffectContext::for_ability(w, 0, None);
    g.resolve_effect(&catalog::lightning_wolf().activated_abilities[0].effect, &ctx).unwrap();
    assert!(g.computed_permanent(w).unwrap().keywords.contains(&Keyword::FirstStrike));
}

/// Ritual Guardian gains lifelink at combat when coven is active.
#[test]
fn ritual_guardian_coven_lifelink() {
    let mut g = two_player_game();
    let rg = g.add_card_to_battlefield(0, catalog::ritual_guardian()); // power 3
    let ctx = EffectContext::for_ability(rg, 0, None);
    g.resolve_effect(&catalog::ritual_guardian().triggered_abilities[0].effect, &ctx).unwrap();
    assert!(g.computed_permanent(rg).unwrap().keywords.contains(&Keyword::Lifelink));
}

/// Defend the Celestus distributes three +1/+1 counters across your creatures.
#[test]
fn defend_the_celestus_distributes() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(a)];
    g.resolve_effect(&catalog::defend_the_celestus().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(a).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(3),
        "all three counters land on the single target",
    );
}

/// Rot-Tide Gargantua's exploit makes each opponent sacrifice a creature.
#[test]
fn rot_tide_gargantua_edicts() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // accept exploit
    g.move_card_to_battlefield_for_test(0, catalog::rot_tide_gargantua());
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "opponent sacrificed their creature");
}

// ── MID/VOW Daybound werewolves + Disturb DFCs (2026-06 batch) ───────────────

use crabomination::game::types::DayNight;

/// Helper: transform a Daybound front to its Nightbound back by becoming night.
fn become_night(g: &mut GameState) {
    let mut ev = vec![];
    g.set_day_night(DayNight::Night, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

/// Tireless Hauler transforms into a 6/6 vigilant Dire-Strain Brawler at night.
#[test]
fn tireless_hauler_transforms_at_night() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::tireless_hauler());
    become_night(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert!(c.transformed, "flipped to back face");
    assert_eq!(c.definition.name, "Dire-Strain Brawler");
    assert_eq!((c.power(), c.toughness()), (6, 6));
    assert!(c.definition.keywords.contains(&Keyword::Vigilance));
    assert!(c.definition.keywords.contains(&Keyword::Nightbound));
}

/// Bird Admirer's back, Wing Shredder, is a 3/5 with reach.
#[test]
fn bird_admirer_back_has_reach() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::bird_admirer());
    become_night(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.power(), c.toughness()), (3, 5));
    assert!(c.definition.keywords.contains(&Keyword::Reach));
}

/// Hookhand Mariner's back can't be blocked by power-2-or-less creatures.
#[test]
fn hookhand_mariner_back_evasion() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::hookhand_mariner());
    become_night(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.power(), c.toughness()), (6, 4));
    assert!(c.definition.keywords.contains(&Keyword::CantBeBlockedByPowerAtMost(2)));
}

/// Fearful Villager keeps menace and grows to 4/3 as Fearsome Werewolf.
#[test]
fn fearful_villager_back_menace() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::fearful_villager());
    become_night(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.power(), c.toughness()), (4, 3));
    assert!(c.definition.keywords.contains(&Keyword::Menace));
}

/// Weary Prisoner's back, Wrathful Jailbreaker, must attack each combat.
#[test]
fn weary_prisoner_back_must_attack() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::weary_prisoner());
    become_night(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.power(), c.toughness()), (6, 6));
    assert!(c.definition.keywords.contains(&Keyword::MustAttack));
}

/// Suspicious Stowaway flips into the unblockable 2/1 Seafaring Werewolf.
#[test]
fn suspicious_stowaway_transforms() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::suspicious_stowaway());
    become_night(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.definition.name, "Seafaring Werewolf");
    assert_eq!((c.power(), c.toughness()), (2, 1));
    assert!(c.definition.keywords.contains(&Keyword::Unblockable));
}

/// Lambholt Raconteur pings each opponent when you cast a noncreature spell.
#[test]
fn lambholt_raconteur_pings_on_noncreature_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lambholt_raconteur());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    // 3 from Bolt + 1 from Raconteur's noncreature-cast trigger.
    assert_eq!(g.players[1].life, before - 4);
}

/// Spellrune Painter pumps itself when you cast an instant or sorcery.
#[test]
fn spellrune_painter_magecraft_pump() {
    let mut g = two_player_game();
    let painter = g.add_card_to_battlefield(0, catalog::spellrune_painter());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    let cp = g.computed_permanent(painter).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4), "+1/+1 until EOT");
}

/// Wolfkin Outcast costs {2} less to cast while you control a Wolf/Werewolf.
#[test]
fn wolfkin_outcast_cost_reduction() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lightning_wolf()); // a Wolf
    let id = g.add_card_to_hand(0, catalog::wolfkin_outcast());
    // {5}{G} reduced by {2} → {3}{G} = 4 mana.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for the reduced cost");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some(), "resolved onto the battlefield");
}

/// Galedrifter's disturb back is the 2/2 flying Waildrifter.
#[test]
fn galedrifter_disturb_back() {
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::galedrifter());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastDisturb {
        card_id: id, target: None, additional_targets: vec![],
    }).expect("disturb");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("on battlefield");
    assert_eq!(c.definition.name, "Waildrifter");
    assert_eq!((c.power(), c.toughness()), (2, 2));
    assert!(c.definition.keywords.contains(&Keyword::Flying));
}

/// Kindly Ancestor's disturb back is an Aura that grants lifelink (exercising
/// the disturb→Aura target plumbing).
#[test]
fn kindly_ancestor_disturb_aura_grants_lifelink() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_graveyard(0, catalog::kindly_ancestor());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastDisturb {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![],
    }).expect("disturb the Aura onto the bear");
    drain_stack(&mut g);
    let aura = g.battlefield_find(id).expect("Aura on battlefield");
    assert_eq!(aura.attached_to, Some(bear), "attached to the chosen creature");
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Lifelink));
}

/// Twinblade Geist's disturb Aura grants double strike.
#[test]
fn twinblade_geist_disturb_aura_double_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_graveyard(0, catalog::twinblade_geist());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastDisturb {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![],
    }).expect("disturb");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::DoubleStrike));
}

/// Mischievous Catgeist's disturb Aura attaches to the chosen creature.
#[test]
fn mischievous_catgeist_disturb_aura_attaches() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_graveyard(0, catalog::mischievous_catgeist());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastDisturb {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![],
    }).expect("disturb");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().attached_to, Some(bear));
}

/// Olivia's Midnight Ambush is -2/-2 by day, -13/-13 at night.
#[test]
fn olivias_midnight_ambush_day_and_night() {
    // Day: -2/-2 kills a 2/2.
    let mut g = two_player_game();
    g.day_night = Some(DayNight::Day);
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(foe)];
    g.resolve_effect(&catalog::olivias_midnight_ambush().effect, &ctx).unwrap();
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "-2/-2 kills the 2/2");
    // Night: -13/-13 kills a 4/4.
    let mut g = two_player_game();
    g.day_night = Some(DayNight::Night);
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(foe)];
    g.resolve_effect(&catalog::olivias_midnight_ambush().effect, &ctx).unwrap();
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "-13/-13 kills the 4/4 at night");
}

/// Moonrager's Slash deals 3, and costs {2} less to cast at night.
#[test]
fn moonragers_slash_damage_and_night_discount() {
    // Damage.
    let mut g = two_player_game();
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Player(1)];
    let before = g.players[1].life;
    g.resolve_effect(&catalog::moonragers_slash().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 3);
    // Night discount: {2}{R} → {R}, castable with a single red.
    let mut g = two_player_game();
    g.day_night = Some(DayNight::Night);
    let id = g.add_card_to_hand(0, catalog::moonragers_slash());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for {R} at night");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20 - 3);
}

/// Lunar Rejection bounces a Wolf/Werewolf and draws a card.
#[test]
fn lunar_rejection_bounces_wolf_and_draws() {
    let mut g = two_player_game();
    let wolf = g.add_card_to_battlefield(1, catalog::lightning_wolf());
    g.add_card_to_library(0, catalog::island());
    let hand_before = g.players[0].hand.len();
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(wolf)];
    g.resolve_effect(&catalog::lunar_rejection().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(wolf).is_none(), "wolf bounced");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
}

/// Shipwreck Sifters grows when you discard a card with disturb.
#[test]
fn shipwreck_sifters_grows_on_disturb_discard() {
    let mut g = two_player_game();
    let sifters = g.add_card_to_battlefield(0, catalog::shipwreck_sifters());
    let disturb_card = g.add_card_to_hand(0, catalog::galedrifter()); // has disturb, not a Spirit
    let mut ev = vec![];
    g.discard_card(0, disturb_card, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(sifters).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(1),
        "a discarded disturb card adds a +1/+1 counter",
    );
}

/// Cobbled Lancer requires exiling a creature card from your graveyard to cast.
#[test]
fn cobbled_lancer_additional_cost() {
    let mut g = two_player_game();
    let corpse = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::cobbled_lancer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable with a creature in the graveyard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some(), "resolved");
    assert!(g.exile.iter().any(|c| c.id == corpse), "exiled the graveyard creature as a cost");
}

/// Gryffwing Cavalry: paying {1}{W} when it attacks stops a creature blocking.
#[test]
fn gryffwing_cavalry_attack_pay_cant_block() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // pay {1}{W}
    let cav = catalog::gryffwing_cavalry();
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(foe)];
    // Ability 0 is Training; ability 1 is the attack "may pay" rider.
    g.resolve_effect(&cav.triggered_abilities[1].effect, &ctx).unwrap();
    assert!(g.computed_permanent(foe).unwrap().keywords.contains(&Keyword::CantBlock));
}

/// Ground Pounder's die-roll ability pumps itself by the rolled amount.
#[test]
fn ground_pounder_die_pump() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let gp = g.add_card_to_battlefield(0, catalog::ground_pounder());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(4)]));
    let ability = catalog::ground_pounder().activated_abilities[0].effect.clone();
    let ctx = EffectContext::for_ability(gp, 0, None);
    g.resolve_effect(&ability, &ctx).unwrap();
    let cp = g.computed_permanent(gp).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 6), "+4/+4 from a rolled 4");
}

/// Augur of Autumn lets you play lands from the top of your library.
#[test]
fn augur_of_autumn_plays_lands_from_top() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::augur_of_autumn());
    let top = g.next_id();
    g.players[0].add_to_library_top(top, catalog::forest());
    g.perform_action(GameAction::PlayLand(top)).expect("land playable off the top");
    assert!(g.battlefield_find(top).is_some(), "Forest entered from the library top");
}

/// Secrets of the Key investigates (creates a Clue) and has flashback.
#[test]
fn secrets_of_the_key_investigates() {
    let mut g = two_player_game();
    let ctx = ctx0(&g);
    g.resolve_effect(&catalog::secrets_of_the_key().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Clue"), 1, "investigated");
    assert!(catalog::secrets_of_the_key().keywords.iter().any(|k|
        matches!(k, Keyword::Flashback(_))));
}

/// Diregraf Rebirth reanimates a creature card from your graveyard.
#[test]
fn diregraf_rebirth_reanimates() {
    let mut g = two_player_game();
    let corpse = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(corpse)];
    g.resolve_effect(&catalog::diregraf_rebirth().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(corpse).is_some(), "returned to the battlefield");
    assert!(g.players[0].graveyard.iter().all(|c| c.id != corpse), "left the graveyard");
}

/// Edgar's Awakening reanimates a creature card from your graveyard.
#[test]
fn edgars_awakening_reanimates() {
    let mut g = two_player_game();
    let corpse = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(corpse)];
    g.resolve_effect(&catalog::edgars_awakening().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(corpse).is_some(), "returned to the battlefield");
}

/// Ceremonial Knife grants +1/+0 and a Blood-on-combat-damage trigger.
#[test]
fn ceremonial_knife_equips_and_makes_blood() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let knife = g.add_card_to_battlefield(0, catalog::ceremonial_knife());
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::Equip { equipment: knife, target: bear }).expect("equip");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "+1/+0");
    // The equipment grants a combat-damage Blood-making triggered ability.
    let bonus = catalog::ceremonial_knife().equipped_bonus.unwrap();
    assert_eq!(bonus.triggered_abilities.len(), 1, "grants the Blood-on-combat-damage trigger");
}

/// Lambholt Pacifist can't attack without a power-4+ creature, but can still
/// block, and transforms on a quiet turn.
#[test]
fn lambholt_pacifist_attack_gate_block_and_transform() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let pacifist = g.add_card_to_battlefield(0, catalog::lambholt_pacifist());
    g.clear_sickness(pacifist);
    g.step = TurnStep::DeclareAttackers;
    // No big creature → can't attack.
    assert!(!g.legal_attackers(0).contains(&pacifist));
    // Blocking is unaffected by the attack-only gate.
    let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    assert!(g.blocker_can_block_attacker(pacifist, atk), "attack-only gate doesn't stop blocking");
    // A power-4+ creature you control lifts the attack restriction.
    g.add_card_to_battlefield(0, catalog::rot_tide_gargantua());
    assert!(g.legal_attackers(0).contains(&pacifist));
    // Quiet last turn → transforms to Lambholt Butcher.
    g.step = TurnStep::Upkeep;
    g.spells_cast_last_turn = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let b = g.battlefield_find(pacifist).unwrap();
    assert_eq!(b.definition.name, "Lambholt Butcher");
    assert_eq!((b.power(), b.toughness()), (4, 4));
}

/// Restless Bloodseeker transforms by sacrificing two Blood tokens; the back
/// face drains each opponent for 2 and gains you 2.
#[test]
fn restless_bloodseeker_blood_transform_and_drain() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let seeker = g.add_card_to_battlefield(0, catalog::restless_bloodseeker());
    g.clear_sickness(seeker);
    g.add_token_to_battlefield(0, &crabomination::game::effects::blood_token());
    g.add_token_to_battlefield(0, &crabomination::game::effects::blood_token());
    g.perform_action(GameAction::ActivateAbility {
        card_id: seeker, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("sac two Blood to transform");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Blood"), 0, "both Blood sacrificed");
    let reveler = g.battlefield_find(seeker).unwrap();
    assert_eq!(reveler.definition.name, "Bloodsoaked Reveler");
    assert_eq!((reveler.power(), reveler.toughness()), (3, 3));
    // Back-face drain: {4}{B}, each opponent loses 2 and you gain 2.
    g.players[0].life = 20;
    g.players[1].life = 20;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: seeker, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("activate back-face drain");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "opponent lost 2");
    assert_eq!(g.players[0].life, 22, "you gained 2");
}

/// Duelcraft Trainer grants double strike to a target creature under Coven.
#[test]
fn duelcraft_trainer_grants_double_strike() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let trainer = g.add_card_to_battlefield(0, catalog::duelcraft_trainer()); // power 3
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
    g.add_card_to_battlefield(0, catalog::rot_tide_gargantua()); // power 5 → coven
    let mut ctx = EffectContext::for_ability(trainer, 0, Some(Target::Permanent(ally)));
    ctx.targets = vec![Target::Permanent(ally)];
    g.resolve_effect(&catalog::duelcraft_trainer().triggered_abilities[0].effect, &ctx).unwrap();
    assert!(g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::DoubleStrike));
}

/// Harvesttide Sentry gains a power-based evasion grant under Coven.
#[test]
fn harvesttide_sentry_evasion_under_coven() {
    let mut g = two_player_game();
    let sentry = g.add_card_to_battlefield(0, catalog::harvesttide_sentry());
    g.resolve_effect(
        &catalog::harvesttide_sentry().triggered_abilities[0].effect,
        &EffectContext::for_ability(sentry, 0, None),
    ).unwrap();
    assert!(g
        .computed_permanent(sentry)
        .unwrap()
        .keywords
        .contains(&Keyword::CantBeBlockedByPowerAtMost(2)));
}

/// Grizzly Ghoul enters with a +1/+1 counter per creature that died this turn.
#[test]
fn grizzly_ghoul_scales_with_deaths() {
    let mut g = two_player_game();
    g.players[0].creatures_died_this_turn = 2;
    let ghoul = g.add_card_to_battlefield(0, catalog::grizzly_ghoul());
    g.resolve_effect(
        &catalog::grizzly_ghoul().triggered_abilities[0].effect,
        &EffectContext::for_ability(ghoul, 0, None),
    ).unwrap();
    assert_eq!(g.battlefield_find(ghoul).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Crossroads Candleguide exiles a graveyard card and taps for any color.
#[test]
fn crossroads_candleguide_exile_and_mana() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let guide = g.add_card_to_battlefield(0, catalog::crossroads_candleguide());
    let mut ctx = EffectContext::for_ability(guide, 0, Some(Target::Permanent(dead)));
    ctx.targets = vec![Target::Permanent(dead)];
    g.resolve_effect(&catalog::crossroads_candleguide().triggered_abilities[0].effect, &ctx).unwrap();
    assert!(g.players[1].graveyard.iter().all(|c| c.id != dead), "graveyard card exiled");
    g.resolve_effect(
        &catalog::crossroads_candleguide().activated_abilities[0].effect,
        &EffectContext::for_ability(guide, 0, None),
    ).unwrap();
    assert!(g.players[0].mana_pool.total() >= 1, "added a mana");
}

/// Drownyard Amalgam mills three on entry.
#[test]
fn drownyard_amalgam_mills_three() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(1, catalog::island()); }
    let amalgam = g.add_card_to_battlefield(0, catalog::drownyard_amalgam());
    let mut ctx = EffectContext::for_ability(amalgam, 0, Some(Target::Player(1)));
    ctx.targets = vec![Target::Player(1)];
    g.resolve_effect(&catalog::drownyard_amalgam().triggered_abilities[0].effect, &ctx).unwrap();
    assert_eq!(g.players[1].graveyard.len(), 3, "milled three");
}

/// Firmament Sage draws when the day/night state flips.
#[test]
fn firmament_sage_draws_on_daynight_flip() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let sage = g.add_card_to_battlefield(0, catalog::firmament_sage());
    let _ = sage;
    g.day_night = Some(DayNight::Day);
    let hand0 = g.players[0].hand.len();
    become_night(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew on the day→night flip");
}

/// Candlegrove Witch gains flying at combat while Coven is active.
#[test]
fn candlegrove_witch_flies_with_coven() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let witch = g.add_card_to_battlefield(0, catalog::candlegrove_witch()); // power 2
    // Two more creatures with distinct powers → coven active (powers 2,1,5... ).
    g.add_card_to_battlefield(0, catalog::vampire_interloper()); // power 2 — same, need distinct
    g.add_card_to_battlefield(0, catalog::rot_tide_gargantua()); // power 5
    g.add_card_to_battlefield(0, catalog::lambholt_pacifist()); // power 3
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert!(g.computed_permanent(witch).unwrap().keywords.contains(&Keyword::Flying));
}

/// Rite of Oblivion exiles a nonland permanent (after its sacrifice cost).
#[test]
fn rite_of_oblivion_exiles_nonland() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, Some(Target::Permanent(foe)));
    ctx.targets = vec![Target::Permanent(foe)];
    g.resolve_effect(&catalog::rite_of_oblivion().effect, &ctx).unwrap();
    assert!(g.battlefield_find(foe).is_none(), "target exiled");
    assert!(g.exile.iter().any(|c| c.id == foe), "in exile");
}

/// Can't Stay Away reanimates a small creature with a finality counter.
#[test]
fn cant_stay_away_reanimates_with_finality() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
    let mut ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, Some(Target::Permanent(dead)));
    ctx.targets = vec![Target::Permanent(dead)];
    g.resolve_effect(&catalog::cant_stay_away().effect, &ctx).unwrap();
    let c = g.battlefield_find(dead).expect("reanimated to battlefield");
    assert_eq!(c.counter_count(CounterType::Finality), 1, "entered with a finality counter");
}

/// Blood Hypnotist's reflexive grant makes a target creature unable to block.
#[test]
fn blood_hypnotist_locks_a_blocker() {
    let mut g = two_player_game();
    let h = g.add_card_to_battlefield(0, catalog::blood_hypnotist());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut ctx = EffectContext::for_ability(h, 0, Some(Target::Permanent(foe)));
    ctx.targets = vec![Target::Permanent(foe)];
    g.resolve_effect(&catalog::blood_hypnotist().triggered_abilities[0].effect, &ctx).unwrap();
    assert!(g.computed_permanent(foe).unwrap().keywords.contains(&Keyword::CantBlock));
}

/// Grisly Ritual destroys a creature and makes two Blood.
#[test]
fn grisly_ritual_destroys_and_makes_blood() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, Some(Target::Permanent(bear)));
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::grisly_ritual().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "creature destroyed");
    assert_eq!(count_named(&g, 0, "Blood"), 2);
}

/// Dominating Vampire steals a small creature (MV gated by your Vampire count).
#[test]
fn dominating_vampire_steals_small_creature() {
    let mut g = two_player_game();
    let dv = g.add_card_to_battlefield(0, catalog::dominating_vampire()); // a Vampire
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2 ≤ 1 Vampire? no
    // One Vampire (the Dominating Vampire itself) → can only steal MV ≤ 1. A MV-2
    // bear is out of range, so control stays. Add a second Vampire to reach MV 2.
    g.add_card_to_battlefield(0, catalog::vampire_interloper());
    let mut ctx = EffectContext::for_ability(dv, 0, Some(Target::Permanent(bear)));
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::dominating_vampire().triggered_abilities[0].effect, &ctx).unwrap();
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0, "stole the bear (2 Vampires)");
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste));
}

/// Graf Reaver pings its controller at upkeep.
#[test]
fn graf_reaver_pings_you_at_upkeep() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    let reaver = g.add_card_to_battlefield(0, catalog::graf_reaver());
    g.resolve_effect(
        &catalog::graf_reaver().triggered_abilities[1].effect,
        &EffectContext::for_ability(reaver, 0, None),
    ).unwrap();
    assert_eq!(g.players[0].life, 19, "dealt 1 to you");
}

/// Blood Servitor makes a Blood token on entry.
#[test]
fn blood_servitor_makes_blood() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::blood_servitor());
    g.resolve_effect(
        &catalog::blood_servitor().triggered_abilities[0].effect,
        &EffectContext::for_ability(s, 0, None),
    ).unwrap();
    assert_eq!(count_named(&g, 0, "Blood"), 1);
}

/// Dreadfeast Demon sacrifices a creature at end step to copy itself.
#[test]
fn dreadfeast_demon_sacs_to_copy() {
    let mut g = two_player_game();
    let demon = g.add_card_to_battlefield(0, catalog::dreadfeast_demon());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // non-Demon fodder
    g.resolve_effect(
        &catalog::dreadfeast_demon().triggered_abilities[0].effect,
        &EffectContext::for_ability(demon, 0, None),
    ).unwrap();
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Dreadfeast Demon"), 2, "made a copy after the sacrifice");
}

/// Dying to Serve mints a tapped Zombie when you discard (once per turn).
#[test]
fn dying_to_serve_makes_tapped_zombie_on_discard() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dying_to_serve());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let evs = g.resolve_effect(
        &crabomination::card::Effect::Discard { who: crabomination::card::Selector::You, amount: crabomination::card::Value::Const(1), random: false },
        &ctx0(&g),
    ).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let z = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Zombie");
    assert!(z.is_some() && z.unwrap().tapped, "tapped Zombie minted");
}

/// Laid to Rest draws when a Human dies and gains life when a counter-bearing
/// creature dies.
#[test]
fn laid_to_rest_death_payoffs() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::laid_to_rest());
    g.add_card_to_library(0, catalog::island());
    let hand0 = g.players[0].hand.len();
    // Lambholt Pacifist is a Human; kill it → Laid to Rest draws.
    let lh = g.add_card_to_battlefield(0, catalog::lambholt_pacifist());
    g.battlefield_find_mut(lh).unwrap().damage = 99;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew when a Human died");
}

/// Laid to Rest's Human death payoff fires for a token Human too: token deaths
/// leave no graveyard card, so the type filter must read the die-snapshot (LKI).
#[test]
fn laid_to_rest_fires_on_human_token_death() {
    use crabomination::card::{CardType, Subtypes, TokenDefinition};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::laid_to_rest());
    g.add_card_to_library(0, catalog::island());
    let hand0 = g.players[0].hand.len();
    let human = TokenDefinition {
        name: "Human".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human],
            ..Default::default()
        },
        ..Default::default()
    };
    let tok = g.add_token_to_battlefield(0, &human);
    g.battlefield_find_mut(tok).unwrap().damage = 99;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew when a Human token died");
}

/// Rise of the Ants makes two 3/3 Insects and gains 2 life.
#[test]
fn rise_of_the_ants_makes_insects() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    g.resolve_effect(&catalog::rise_of_the_ants().effect, &ctx0(&g)).unwrap();
    assert_eq!(count_named(&g, 0, "Insect"), 2);
    assert_eq!(g.players[0].life, 22);
}

/// Soul-Guide Gryff exiles a card from a graveyard on entry.
#[test]
fn soul_guide_gryff_exiles_graveyard_card() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let gryff = g.add_card_to_battlefield(0, catalog::soul_guide_gryff());
    let mut ctx = EffectContext::for_ability(gryff, 0, Some(Target::Permanent(dead)));
    ctx.targets = vec![Target::Permanent(dead)];
    g.resolve_effect(&catalog::soul_guide_gryff().triggered_abilities[0].effect, &ctx).unwrap();
    assert!(g.players[1].graveyard.iter().all(|c| c.id != dead), "card exiled from graveyard");
}

/// Honored Heirloom taps for any color and exiles a graveyard card.
#[test]
fn honored_heirloom_mana_and_graveyard_exile() {
    let mut g = two_player_game();
    let heirloom = g.add_card_to_battlefield(0, catalog::honored_heirloom());
    g.resolve_effect(
        &catalog::honored_heirloom().activated_abilities[0].effect,
        &EffectContext::for_ability(heirloom, 0, None),
    ).unwrap();
    assert!(g.players[0].mana_pool.total() >= 1, "added a mana");
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let mut ctx = EffectContext::for_ability(heirloom, 0, Some(Target::Permanent(dead)));
    ctx.targets = vec![Target::Permanent(dead)];
    g.resolve_effect(&catalog::honored_heirloom().activated_abilities[1].effect, &ctx).unwrap();
    assert!(g.players[1].graveyard.iter().all(|c| c.id != dead), "graveyard card exiled");
}

/// Stuffed Bear animates into a 4/4 Bear.
#[test]
fn stuffed_bear_becomes_a_bear() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::stuffed_bear());
    g.resolve_effect(
        &catalog::stuffed_bear().activated_abilities[0].effect,
        &EffectContext::for_ability(bear, 0, None),
    ).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.card_types.contains(&crabomination::card::CardType::Creature));
}

/// Circle of Confinement exiles a small opposing creature until it leaves.
#[test]
fn circle_of_confinement_exiles_cheap_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let circle = g.add_card_to_battlefield(0, catalog::circle_of_confinement());
    let mut ctx = EffectContext::for_ability(circle, 0, Some(Target::Permanent(bear)));
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::circle_of_confinement().triggered_abilities[0].effect, &ctx).unwrap();
    assert!(g.battlefield_find(bear).is_none(), "cheap creature exiled");
}

/// Stolen Vitality pumps +3/+1 and grants trample on your turn.
#[test]
fn stolen_vitality_pumps_and_tramples_on_your_turn() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, Some(Target::Permanent(bear)));
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::stolen_vitality().effect, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 3), "+3/+1");
    assert!(cp.keywords.contains(&Keyword::Trample), "your turn → trample");
}

/// Gift of Fangs is a +2/+2 boon on a Vampire but a -2/-2 bane otherwise.
#[test]
fn gift_of_fangs_helps_vampires_hurts_others() {
    let mut g = two_player_game();
    let vamp = g.add_card_to_battlefield(0, catalog::vampire_interloper()); // 2/1 Vampire
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 non-Vampire
    let gf1 = g.add_card_to_battlefield(0, catalog::gift_of_fangs());
    let gf2 = g.add_card_to_battlefield(0, catalog::gift_of_fangs());
    let mut c1 = EffectContext::for_ability(gf1, 0, Some(Target::Permanent(vamp)));
    c1.targets = vec![Target::Permanent(vamp)];
    g.resolve_effect(&catalog::gift_of_fangs().effect, &c1).unwrap();
    let mut c2 = EffectContext::for_ability(gf2, 0, Some(Target::Permanent(bear)));
    c2.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::gift_of_fangs().effect, &c2).unwrap();
    assert_eq!(g.computed_permanent(vamp).unwrap().power, 4, "Vampire +2/+2");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 0, "non-Vampire -2/-2");
}

/// Silver Bolt deals 3 and destroys a Werewolf outright.
#[test]
fn silver_bolt_destroys_werewolf() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let wolf = g.add_card_to_battlefield(1, catalog::lambholt_pacifist()); // 3/3 Werewolf
    let bolt = g.add_card_to_battlefield(0, catalog::silver_bolt());
    let mut ctx = EffectContext::for_ability(bolt, 0, Some(Target::Permanent(wolf)));
    ctx.targets = vec![Target::Permanent(wolf)];
    g.resolve_effect(&catalog::silver_bolt().activated_abilities[0].effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(wolf).is_none(), "Werewolf destroyed despite surviving 3 damage");
}

/// Aim for the Head exiles a Zombie (mode 0).
#[test]
fn aim_for_the_head_exiles_zombie() {
    let mut g = two_player_game();
    let zombie = g.add_card_to_battlefield(1, catalog::diregraf_colossus());
    let mut ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, Some(Target::Permanent(zombie)));
    ctx.targets = vec![Target::Permanent(zombie)];
    ctx.mode = 0;
    g.resolve_effect(&catalog::aim_for_the_head().effect, &ctx).unwrap();
    assert!(g.battlefield_find(zombie).is_none(), "Zombie exiled");
}

/// Borrowed Time exiles an opponent's permanent until it leaves.
#[test]
fn borrowed_time_exiles_until_it_leaves() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bt = g.add_card_to_battlefield(0, catalog::borrowed_time());
    let mut ctx = EffectContext::for_ability(bt, 0, Some(Target::Permanent(bear)));
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::borrowed_time().triggered_abilities[0].effect, &ctx).unwrap();
    assert!(g.battlefield_find(bear).is_none(), "bear exiled");
}

/// Join the Dance makes two 1/1 white Humans and carries Flashback.
#[test]
fn join_the_dance_makes_two_humans() {
    let mut g = two_player_game();
    g.resolve_effect(&catalog::join_the_dance().effect, &ctx0(&g)).unwrap();
    assert_eq!(count_named(&g, 0, "Human"), 2);
    assert!(catalog::join_the_dance()
        .keywords
        .iter()
        .any(|k| matches!(k, Keyword::Flashback(_))));
}

/// No Way Out makes the opponent discard two and mints a decayed Zombie.
#[test]
fn no_way_out_discard_and_decayed_zombie() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let mut ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
    ctx.targets = vec![Target::Player(1)];
    g.resolve_effect(&catalog::no_way_out().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "opponent discarded both");
    let z = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.is_creature());
    assert!(z.unwrap().definition.keywords.contains(&Keyword::Decayed), "decayed Zombie");
}

/// Pack's Betrayal steals a creature with haste; scry 2 rider needs a Wolf.
#[test]
fn packs_betrayal_steals_with_haste() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, Some(Target::Permanent(bear)));
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::packs_betrayal().effect, &ctx).unwrap();
    let stolen = g.battlefield_find(bear).unwrap();
    assert_eq!(stolen.controller, 0, "gained control");
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste));
}

/// Dryad's Revival returns a graveyard card to hand and carries Flashback.
#[test]
fn dryads_revival_returns_from_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let mut ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, Some(Target::Permanent(bear)));
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::dryads_revival().effect, &ctx).unwrap();
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "card returned to hand");
    assert!(catalog::dryads_revival()
        .keywords
        .iter()
        .any(|k| matches!(k, Keyword::Flashback(_))));
}

/// Cathar's Call grants vigilance via its aura bonus and an end-step Human maker.
#[test]
fn cathars_call_grants_vigilance_and_token() {
    let bonus = catalog::cathars_call().equipped_bonus.unwrap();
    assert!(bonus.keywords.contains(&Keyword::Vigilance));
    assert_eq!(bonus.triggered_abilities.len(), 1, "end-step Human-maker");
}

/// Bleeding Edge shrinks a creature and amasses Zombies 2.
#[test]
fn bleeding_edge_shrinks_and_amasses() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let edge = g.add_card_to_hand(0, catalog::bleeding_edge());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: edge,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("cast Bleeding Edge");
    drain_stack(&mut g);
    // -2/-2 kills the 2/2 bear; an Army token (Amass 2) appears.
    assert!(g.battlefield_find(bear).is_none(), "bear died to -2/-2");
    let army = g.battlefield.iter().find(|c| {
        c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Army)
    });
    assert!(army.is_some(), "Amass minted an Army");
    assert_eq!(g.computed_permanent(army.unwrap().id).unwrap().power, 2, "Army has two counters");
}

/// Lantern Bearer's disturb back face is an Aura granting +1/+1 and flying.
#[test]
fn lantern_bearer_disturb_aura_grants_flying() {
    let bonus = catalog::lantern_bearer().back_face.unwrap().equipped_bonus.unwrap();
    assert_eq!((bonus.power, bonus.toughness), (1, 1));
    assert!(bonus.keywords.contains(&Keyword::Flying));
}

/// Catapult Fodder transforms once you control three defensive creatures, and
/// the back drains an opponent for a sacrificed creature's toughness.
#[test]
fn catapult_fodder_transforms_and_captain_drains() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let fodder = g.add_card_to_battlefield(0, catalog::catapult_fodder());
    g.clear_sickness(fodder); // 1/5, t>p
    let effect = catalog::catapult_fodder().triggered_abilities[0].effect.clone();
    // Only one toughness>power creature (the fodder) → no transform.
    g.resolve_effect(&effect, &EffectContext::for_ability(fodder, 0, None)).unwrap();
    assert_eq!(g.battlefield_find(fodder).unwrap().definition.name, "Catapult Fodder");
    // Add two more defensive creatures (Rot-Tide Gargantua is 5/4 = p>t, so use
    // two more high-toughness bodies).
    g.add_card_to_battlefield(0, catalog::catapult_fodder());
    g.add_card_to_battlefield(0, catalog::catapult_fodder());
    g.resolve_effect(&effect, &EffectContext::for_ability(fodder, 0, None)).unwrap();
    assert_eq!(g.battlefield_find(fodder).unwrap().definition.name, "Catapult Captain");
    // Captain's sac drain: sacrifice a 1/5 → opponent loses 5.
    g.players[1].life = 20;
    let victim = g.add_card_to_battlefield(0, catalog::catapult_fodder()); // 1/5
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: fodder, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("captain drain");
    drain_stack(&mut g);
    let _ = victim;
    assert_eq!(g.players[1].life, 15, "opponent loses 5 (sacrificed toughness)");
}

/// Voldaren Bloodcaster transforms when you create a Blood while controlling
/// five or more Blood tokens (exercises the new TokenCreated trigger).
#[test]
fn voldaren_bloodcaster_transforms_at_five_blood() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let caster = g.add_card_to_battlefield(0, catalog::voldaren_bloodcaster());
    // Four Blood already out; the fifth (minted now) crosses the threshold and
    // its TokenCreated trigger fires the transform.
    for _ in 0..4 {
        g.add_token_to_battlefield(0, &crabomination::game::effects::blood_token());
    }
    let evs = g
        .resolve_effect(
            &crabomination::card::Effect::CreateToken {
                who: crabomination::effect::PlayerRef::You,
                count: crabomination::card::Value::Const(1),
                definition: Box::new(crabomination_base::tokens::blood_token()),
            },
            &ctx0(&g),
        )
        .unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(caster).unwrap().definition.name, "Bloodbat Summoner");
}

/// Daring Sleuth transforms when you sacrifice a Clue; the back has prowess.
#[test]
fn daring_sleuth_transforms_on_clue_sacrifice() {
    use crabomination::card::{Effect, Selector, SelectionRequirement, Value};
    let mut g = two_player_game();
    let sleuth = g.add_card_to_battlefield(0, catalog::daring_sleuth());
    // Mint a Clue, then sacrifice it — the sacrifice fires the transform trigger.
    g.resolve_effect(&crabomination::effect::shortcut::investigate(1), &ctx0(&g)).unwrap();
    let evs = g
        .resolve_effect(
            &Effect::Sacrifice {
                who: Selector::You,
                count: Value::Const(1),
                filter: SelectionRequirement::HasArtifactSubtype(crabomination::card::ArtifactSubtype::Clue),
            },
            &ctx0(&g),
        )
        .unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let back = g.battlefield_find(sleuth).unwrap();
    assert_eq!(back.definition.name, "Bearer of Overwhelming Truths");
    assert!(back.definition.keywords.contains(&Keyword::Prowess));
}

/// Bloodsworn Squire's reflexive transform fires once four creature cards are
/// in your graveyard; the back's P/T scales with that count.
#[test]
fn bloodsworn_squire_transforms_with_full_graveyard() {
    let mut g = two_player_game();
    let squire = g.add_card_to_battlefield(0, catalog::bloodsworn_squire());
    let effect = catalog::bloodsworn_squire().activated_abilities[0].effect.clone();
    // Empty graveyard → ability resolves (indestructible + tap) but no transform.
    g.resolve_effect(&effect, &EffectContext::for_ability(squire, 0, None)).unwrap();
    assert_eq!(g.battlefield_find(squire).unwrap().definition.name, "Bloodsworn Squire");
    assert!(g.battlefield_find(squire).unwrap().tapped, "tapped itself");
    // Four creature cards in the graveyard → the reflexive clause transforms it.
    for _ in 0..4 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    g.resolve_effect(&effect, &EffectContext::for_ability(squire, 0, None)).unwrap();
    assert_eq!(g.battlefield_find(squire).unwrap().definition.name, "Bloodsworn Knight");
    assert_eq!(
        g.computed_permanent(squire).unwrap().power,
        4,
        "*/* = creature cards in your graveyard"
    );
}

/// Dormant Grove pumps a creature and transforms once it reaches toughness 6.
#[test]
fn dormant_grove_transforms_when_toughness_six() {
    let mut g = two_player_game();
    let grove = g.add_card_to_battlefield(0, catalog::dormant_grove());
    let big = g.add_card_to_battlefield(0, catalog::rot_tide_gargantua()); // 5/4
    let effect = catalog::dormant_grove().triggered_abilities[0].effect.clone();
    let mut ctx = EffectContext::for_ability(grove, 0, Some(Target::Permanent(big)));
    ctx.targets = vec![Target::Permanent(big)];
    // First counter: toughness 4→5, below 6, no transform.
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.battlefield_find(grove).unwrap().definition.name, "Dormant Grove");
    // Second counter: toughness 6 → transforms into Gnarled Grovestrider.
    g.resolve_effect(&effect, &ctx).unwrap();
    let strider = g.battlefield_find(grove).unwrap();
    assert_eq!(strider.definition.name, "Gnarled Grovestrider");
    assert!(strider.definition.keywords.contains(&Keyword::Vigilance));
}

/// Restless Bloodseeker mints a Blood at end step only if you gained life.
#[test]
fn restless_bloodseeker_end_step_blood_on_lifegain() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let seeker = g.add_card_to_battlefield(0, catalog::restless_bloodseeker());
    let _ = seeker;
    // No life gained → no Blood at end step.
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Blood"), 0, "no lifegain → no Blood");
    // Gain life, then the end-step trigger mints a Blood.
    g.adjust_life(0, 1);
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Blood"), 1, "lifegain → Blood");
}

// ── VOW/MID batch 17 ─────────────────────────────────────────────────────────

/// Unholy Officiant's {4}{W} ability puts a +1/+1 counter on it.
#[test]
fn unholy_officiant_counter() {
    let mut g = two_player_game();
    let off = g.add_card_to_battlefield(0, catalog::unholy_officiant());
    g.resolve_effect(
        &catalog::unholy_officiant().activated_abilities[0].effect,
        &EffectContext::for_ability(off, 0, None),
    ).unwrap();
    assert_eq!(
        g.battlefield_find(off).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(1)
    );
}

/// Nebelgast Beguiler's ability taps a target creature.
#[test]
fn nebelgast_beguiler_taps() {
    let mut g = two_player_game();
    let beg = g.add_card_to_battlefield(0, catalog::nebelgast_beguiler());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut ctx = EffectContext::for_ability(beg, 0, Some(Target::Permanent(bear)));
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::nebelgast_beguiler().activated_abilities[0].effect, &ctx).unwrap();
    assert!(g.battlefield_find(bear).unwrap().tapped, "target creature is tapped");
}

/// Vampire Slayer destroys a Vampire it damages.
#[test]
fn vampire_slayer_destroys_vampire() {
    let mut g = two_player_game();
    let slayer = g.add_card_to_battlefield(0, catalog::vampire_slayer());
    let vamp = g.add_card_to_battlefield(1, catalog::gluttonous_guest()); // Vampire 1/4
    let mut ctx = EffectContext::for_ability(slayer, 0, Some(Target::Permanent(vamp)));
    ctx.targets = vec![Target::Permanent(vamp)];
    g.resolve_effect(&catalog::vampire_slayer().triggered_abilities[0].effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(vamp).is_none(), "Vampire destroyed");
}

/// Odric's Outrider drops a +1/+1 counter on a creature when one dies.
#[test]
fn odrics_outrider_counter_on_death() {
    let mut g = two_player_game();
    let outrider = g.add_card_to_battlefield(0, catalog::odrics_outrider());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ctx = EffectContext::for_ability(outrider, 0, Some(Target::Permanent(bear)));
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::odrics_outrider().triggered_abilities[0].effect, &ctx).unwrap();
    assert_eq!(
        g.battlefield_find(bear).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(1)
    );
}

/// Syphon Essence counters a creature spell and mints a Blood token.
#[test]
fn syphon_essence_counters_and_makes_blood() {
    let mut g = two_player_game();
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add_colorless(1);
    g.players[1].mana_pool.add(Color::Green, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    let mut ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, Some(Target::Permanent(bears)));
    ctx.targets = vec![Target::Permanent(bears)];
    g.resolve_effect(&catalog::syphon_essence().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none(), "creature spell countered");
    assert_eq!(count_named(&g, 0, "Blood"), 1, "Blood token created");
}

/// Wanderlight Spirit can block only flyers.
#[test]
fn wanderlight_spirit_blocks_only_flying() {
    let mut g = two_player_game();
    let blk = g.add_card_to_battlefield(1, catalog::wanderlight_spirit());
    let binst = g.battlefield_find(blk).unwrap();
    let bcomp = g.computed_permanent(blk).unwrap();
    assert!(
        !crabomination::game::can_block_attacker_computed(binst, &bcomp, &[], &[], 2),
        "can't block a ground attacker"
    );
    assert!(
        crabomination::game::can_block_attacker_computed(binst, &bcomp, &[Keyword::Flying], &[], 2),
        "can block a flyer"
    );
}

/// Dreadlight Monstrosity's ability makes it unblockable for the turn.
#[test]
fn dreadlight_monstrosity_unblockable() {
    let mut g = two_player_game();
    let mon = g.add_card_to_battlefield(0, catalog::dreadlight_monstrosity());
    g.resolve_effect(
        &catalog::dreadlight_monstrosity().activated_abilities[0].effect,
        &EffectContext::for_ability(mon, 0, None),
    ).unwrap();
    assert!(g.computed_permanent(mon).unwrap().keywords.contains(&Keyword::Unblockable));
}

/// Skulking Killer gives -2/-2 only when the opponent has no other creatures.
#[test]
fn skulking_killer_minus_when_lonely() {
    let mut g = two_player_game();
    let killer = g.add_card_to_battlefield(0, catalog::skulking_killer());
    let lone = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // opponent's only creature
    let mut ctx = EffectContext::for_ability(killer, 0, Some(Target::Permanent(lone)));
    ctx.targets = vec![Target::Permanent(lone)];
    g.resolve_effect(&catalog::skulking_killer().triggered_abilities[0].effect, &ctx).unwrap();
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(lone).is_none(), "lone 2/2 dies to -2/-2");
    // With a second opponent creature, the condition fails — no shrink.
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut ctx = EffectContext::for_ability(killer, 0, Some(Target::Permanent(a)));
    ctx.targets = vec![Target::Permanent(a)];
    g.resolve_effect(&catalog::skulking_killer().triggered_abilities[0].effect, &ctx).unwrap();
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_some() && g.battlefield_find(b).is_some(),
        "no shrink when opponent has multiple creatures");
}

/// Pyre Spawn deals 3 on death.
#[test]
fn pyre_spawn_dies_deals_three() {
    let mut g = two_player_game();
    let spawn = g.add_card_to_battlefield(0, catalog::pyre_spawn());
    let mut ctx = EffectContext::for_ability(spawn, 0, Some(Target::Player(1)));
    ctx.targets = vec![Target::Player(1)];
    g.resolve_effect(&catalog::pyre_spawn().triggered_abilities[0].effect, &ctx).unwrap();
    assert_eq!(g.players[1].life, 17, "3 damage to opponent");
}

/// Sanguine Statuette mints a Blood on entry and animates on a Blood sacrifice.
#[test]
fn sanguine_statuette_etb_and_animate() {
    let mut g = two_player_game();
    let stat = g.move_card_to_battlefield_for_test(0, catalog::sanguine_statuette());
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Blood"), 1, "ETB Blood token");
    g.resolve_effect(
        &catalog::sanguine_statuette().triggered_abilities[1].effect,
        &EffectContext::for_ability(stat, 0, None),
    ).unwrap();
    let cp = g.computed_permanent(stat).unwrap();
    assert_eq!(cp.power, 3, "animated to 3/3");
    assert!(g.battlefield_find(stat).unwrap().definition.is_creature() || cp.card_types.contains(&CardType::Creature));
}

/// Runebound Wolf deals damage equal to the Wolves/Werewolves you control.
#[test]
fn runebound_wolf_damage_scales() {
    let mut g = two_player_game();
    let wolf = g.add_card_to_battlefield(0, catalog::runebound_wolf());
    g.add_card_to_battlefield(0, catalog::runebound_wolf()); // second Wolf → count 2
    let mut ctx = EffectContext::for_ability(wolf, 0, Some(Target::Player(1)));
    ctx.targets = vec![Target::Player(1)];
    g.resolve_effect(&catalog::runebound_wolf().activated_abilities[0].effect, &ctx).unwrap();
    assert_eq!(g.players[1].life, 18, "2 Wolves → 2 damage");
}

/// Into the Night turns it to night and loots (discard 0 → draw 1).
#[test]
fn into_the_night_night_and_draw() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    g.resolve_effect(&catalog::into_the_night().effect, &ctx0(&g)).unwrap();
    assert_eq!(g.day_night, Some(crabomination::game::types::DayNight::Night), "it becomes night");
    assert_eq!(g.players[0].hand.len(), before + 1, "discard 0, draw 1");
}

/// Cloaked Cadet draws when +1/+1 counters land on a Human you control.
#[test]
fn cloaked_cadet_draws_on_human_counter() {
    let mut g = two_player_game();
    let cadet = g.add_card_to_battlefield(0, catalog::cloaked_cadet());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    g.battlefield_find_mut(cadet).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.dispatch_triggers_for_events(&[GameEvent::CounterAdded {
        card_id: cadet, counter_type: CounterType::PlusOnePlusOne, count: 1,
    }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1, "drew off the Human counter trigger");
}

/// Flourishing Hunter gains life equal to the greatest toughness among OTHER
/// creatures (excluding itself).
#[test]
fn flourishing_hunter_gains_other_toughness() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rot_tide_gargantua()); // 5/4
    let before = g.players[0].life;
    let hunter = g.move_card_to_battlefield_for_test(0, catalog::flourishing_hunter()); // 6/6
    let _ = hunter;
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, before + 4, "gains 4 (other's toughness), not 6 (its own)");
}

/// Witch's Web pumps, grants reach, and untaps the target.
#[test]
fn witchs_web_pump_reach_untap() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let mut ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, Some(Target::Permanent(bear)));
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::witchs_web().effect, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 5, "+3/+3");
    assert!(cp.keywords.contains(&Keyword::Reach), "gains reach");
    assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped");
}

/// Vilespawn Spider makes one Insect per creature card in your graveyard.
#[test]
fn vilespawn_spider_makes_insects() {
    let mut g = two_player_game();
    let spider = g.add_card_to_battlefield(0, catalog::vilespawn_spider());
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    g.resolve_effect(
        &catalog::vilespawn_spider().activated_abilities[0].effect,
        &EffectContext::for_ability(spider, 0, None),
    ).unwrap();
    assert_eq!(count_named(&g, 0, "Insect"), 3, "one Insect per graveyard creature");
}

/// Lantern of the Lost exiles a graveyard card on entry, then can sweep all.
#[test]
fn lantern_of_the_lost_exiles() {
    let mut g = two_player_game();
    let lantern = g.add_card_to_battlefield(0, catalog::lantern_of_the_lost());
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let mut ctx = EffectContext::for_ability(lantern, 0, Some(Target::Permanent(dead)));
    ctx.targets = vec![Target::Permanent(dead)];
    g.resolve_effect(&catalog::lantern_of_the_lost().triggered_abilities[0].effect, &ctx).unwrap();
    assert!(g.players[1].graveyard.iter().all(|c| c.id != dead), "ETB exiled the gy card");
    // Sweep ability empties all graveyards.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.resolve_effect(
        &catalog::lantern_of_the_lost().activated_abilities[0].effect,
        &EffectContext::for_ability(lantern, 0, None),
    ).unwrap();
    assert!(g.players[0].graveyard.is_empty() && g.players[1].graveyard.is_empty(),
        "all graveyards exiled");
}

/// Nebelgast Intruder shrinks an opponent's creature by -2/-0.
#[test]
fn nebelgast_intruder_shrinks() {
    let mut g = two_player_game();
    let intruder = g.add_card_to_battlefield(0, catalog::nebelgast_intruder());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut ctx = EffectContext::for_ability(intruder, 0, Some(Target::Permanent(bear)));
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::nebelgast_intruder().triggered_abilities[0].effect, &ctx).unwrap();
    assert_eq!(g.computed_permanent(bear).unwrap().power, 0, "-2/-0 on a 2/2");
}

/// Candlelit Cavalry gains trample under Coven (three different powers).
#[test]
fn candlelit_cavalry_coven_trample() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let cav = g.add_card_to_battlefield(0, catalog::candlelit_cavalry()); // power 5
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
    let third = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(third).unwrap().add_counters(CounterType::PlusOnePlusOne, 1); // power 3
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert!(g.computed_permanent(cav).unwrap().keywords.contains(&Keyword::Trample),
        "Coven grants trample");
}

/// Purifying Dragon pings 1 on attack, or 2 to a Zombie.
#[test]
fn purifying_dragon_attack_ping() {
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(0, catalog::purifying_dragon());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut ctx = EffectContext::for_ability(dragon, 0, Some(Target::Permanent(bear)));
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::purifying_dragon().triggered_abilities[0].effect, &ctx).unwrap();
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 1, "1 to a non-Zombie");
    let zombie = g.add_card_to_battlefield(1, catalog::larder_zombie());
    let mut ctx = EffectContext::for_ability(dragon, 0, Some(Target::Permanent(zombie)));
    ctx.targets = vec![Target::Permanent(zombie)];
    g.resolve_effect(&catalog::purifying_dragon().triggered_abilities[0].effect, &ctx).unwrap();
    assert_eq!(g.battlefield_find(zombie).unwrap().damage, 2, "2 to a Zombie");
}

/// Obsessive Astronomer makes it day on entry when neither day nor night.
#[test]
fn obsessive_astronomer_makes_day() {
    let mut g = two_player_game();
    assert_eq!(g.day_night, None);
    g.move_card_to_battlefield_for_test(0, catalog::obsessive_astronomer());
    drain_stack(&mut g);
    assert_eq!(g.day_night, Some(crabomination::game::types::DayNight::Day), "becomes day on entry");
}

/// Hungry for More mints a hasty Vampire that sacrifices itself at end step.
#[test]
fn hungry_for_more_temp_token() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.resolve_effect(&catalog::hungry_for_more().effect, &ctx0(&g)).unwrap();
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Vampire"), 1, "token minted");
    let tok = g.battlefield.iter().find(|c| c.definition.name == "Vampire").unwrap();
    assert!(tok.definition.keywords.contains(&Keyword::Haste), "hasty");
    assert!(tok.definition.keywords.contains(&Keyword::Lifelink), "lifelink");
    // End step → the token sacrifices itself and ceases to exist.
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Vampire"), 0, "token gone at end step");
}

// ── VOW/MID batch 18 ─────────────────────────────────────────────────────────

/// Flip the Switch counters an unaffordable spell and mints a decayed Zombie.
#[test]
fn flip_the_switch_counters_and_makes_zombie() {
    let mut g = two_player_game();
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add_colorless(1);
    g.players[1].mana_pool.add(Color::Green, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    // Opponent has no mana to pay {4} → countered.
    let mut ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, Some(Target::Permanent(bears)));
    ctx.targets = vec![Target::Permanent(bears)];
    g.resolve_effect(&catalog::flip_the_switch().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none(), "spell countered");
    assert_eq!(count_named(&g, 0, "Zombie"), 1, "decayed Zombie minted");
}

/// Jack-o'-Lantern exiles a graveyard card and draws.
#[test]
fn jack_o_lantern_exile_and_draw() {
    let mut g = two_player_game();
    let lantern = g.add_card_to_battlefield(0, catalog::jack_o_lantern());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    let mut ctx = EffectContext::for_ability(lantern, 0, Some(Target::Permanent(dead)));
    ctx.targets = vec![Target::Permanent(dead)];
    g.resolve_effect(&catalog::jack_o_lantern().activated_abilities[0].effect, &ctx).unwrap();
    assert!(g.players[1].graveyard.iter().all(|c| c.id != dead), "exiled gy card");
    assert_eq!(g.players[0].hand.len(), before + 1, "drew a card");
}

/// Ominous Roost mints a fly-only-blocking Bird on entry.
#[test]
fn ominous_roost_etb_bird() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::ominous_roost());
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Bird"), 1, "ETB Bird");
    let bird = g.battlefield.iter().find(|c| c.definition.name == "Bird").unwrap();
    assert!(bird.definition.keywords.contains(&Keyword::CanBlockOnlyFlying));
}

/// Serpentine Ambush turns a creature into a 5/5 Serpent.
#[test]
fn serpentine_ambush_makes_serpent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, Some(Target::Permanent(bear)));
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::serpentine_ambush().effect, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "becomes 5/5");
    assert!(cp.subtypes.creature_types.contains(&CreatureType::Serpent), "is a Serpent");
}

/// Turn the Earth shuffles graveyard cards back and gains 2 life.
#[test]
fn turn_the_earth_recycles_and_gains() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let before_life = g.players[0].life;
    let mut ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, Some(Target::Permanent(dead)));
    ctx.targets = vec![Target::Permanent(dead)];
    g.resolve_effect(&catalog::turn_the_earth().effect, &ctx).unwrap();
    assert!(g.players[0].graveyard.iter().all(|c| c.id != dead), "shuffled out of graveyard");
    assert_eq!(g.players[0].life, before_life + 2, "gained 2 life");
}

/// Moonsilver Key fetches a basic land to hand.
#[test]
fn moonsilver_key_fetches_land() {
    let mut g = two_player_game();
    let key = g.add_card_to_battlefield(0, catalog::moonsilver_key());
    let land = g.add_card_to_library(0, catalog::forest());
    let before = g.players[0].hand.len();
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(Some(land)),
    ]));
    g.resolve_effect(
        &catalog::moonsilver_key().activated_abilities[0].effect,
        &EffectContext::for_ability(key, 0, None),
    ).unwrap();
    assert_eq!(g.players[0].hand.len(), before + 1, "fetched a card to hand");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"), "fetched the Forest");
}

/// Unblinking Observer taps for restricted blue mana.
#[test]
fn unblinking_observer_makes_mana() {
    let mut g = two_player_game();
    let obs = g.add_card_to_battlefield(0, catalog::unblinking_observer());
    g.resolve_effect(
        &catalog::unblinking_observer().activated_abilities[0].effect,
        &EffectContext::for_ability(obs, 0, None),
    ).unwrap();
    assert!(g.players[0].mana_pool.restricted_total() >= 1, "added restricted blue mana");
}

/// Duel for Dominance pumps under Coven, then fights.
#[test]
fn duel_for_dominance_coven_fight() {
    let mut g = two_player_game();
    // Coven: three creatures with powers 5, 2, 3.
    let mine = g.add_card_to_battlefield(0, catalog::candlelit_cavalry()); // 5/5
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let third = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(third).unwrap().add_counters(CounterType::PlusOnePlusOne, 1); // 3/3
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let mut ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
    ctx.targets = vec![Target::Permanent(mine), Target::Permanent(foe)];
    g.resolve_effect(&catalog::duel_for_dominance().effect, &ctx).unwrap();
    g.check_state_based_actions();
    drain_stack(&mut g);
    // Coven counter made the cavalry 6/6; it fought the 2/2 (which dies).
    assert_eq!(
        g.battlefield_find(mine).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(1),
        "coven +1/+1"
    );
    assert!(g.battlefield_find(foe).is_none(), "the opposing 2/2 died in the fight");
}

/// Rootcoil Creeper taps for one mana of any color.
#[test]
fn rootcoil_creeper_ramps() {
    let mut g = two_player_game();
    let creep = g.add_card_to_battlefield(0, catalog::rootcoil_creeper());
    g.resolve_effect(
        &catalog::rootcoil_creeper().activated_abilities[0].effect,
        &EffectContext::for_ability(creep, 0, None),
    ).unwrap();
    assert!(g.players[0].mana_pool.total() >= 1, "ramped one mana");
}

// ── VOW/MID batch 19 ─────────────────────────────────────────────────────────

/// Spiked Ripsaw grants +3/+3 and a sac-a-Forest-for-trample attack trigger.
#[test]
fn spiked_ripsaw_sac_forest_for_trample() {
    let mut g = two_player_game();
    assert_eq!(catalog::spiked_ripsaw().equipped_bonus.unwrap().power, 3, "+3/+3 bonus");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    let trig = catalog::spiked_ripsaw().equipped_bonus.unwrap().triggered_abilities[0].effect.clone();
    g.resolve_effect(&trig, &EffectContext::for_trigger(bear, 0, None, 0)).unwrap();
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample), "gained trample");
    assert!(g.battlefield_find(forest).is_none(), "Forest sacrificed");
}

/// Fleeting Spirit's first ability grants first strike; the second blinks it.
#[test]
fn fleeting_spirit_abilities() {
    let mut g = two_player_game();
    let spirit = g.add_card_to_battlefield(0, catalog::fleeting_spirit());
    g.resolve_effect(
        &catalog::fleeting_spirit().activated_abilities[0].effect,
        &EffectContext::for_ability(spirit, 0, None),
    ).unwrap();
    assert!(g.computed_permanent(spirit).unwrap().keywords.contains(&Keyword::FirstStrike));
    // Second ability exiles it (returns at the next end step).
    g.resolve_effect(
        &catalog::fleeting_spirit().activated_abilities[1].effect,
        &EffectContext::for_ability(spirit, 0, None),
    ).unwrap();
    assert!(g.battlefield_find(spirit).is_none(), "exiled by the blink ability");
}

/// Contortionist Troupe enters with X counters and a coven end-step counter.
#[test]
fn contortionist_troupe_x_counters_and_coven() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // Coven board: powers 5, 2, 3.
    let cav = g.add_card_to_battlefield(0, catalog::candlelit_cavalry()); // 5
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2
    let third = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(third).unwrap().add_counters(CounterType::PlusOnePlusOne, 1); // 3
    // Resolve the end-step coven trigger targeting the cavalry.
    let trig = catalog::contortionist_troupe().triggered_abilities[0].effect.clone();
    let mut ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, Some(Target::Permanent(cav)));
    ctx.targets = vec![Target::Permanent(cav)];
    g.step = TurnStep::End;
    g.resolve_effect(&trig, &ctx).unwrap();
    assert_eq!(
        g.battlefield_find(cav).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(1),
        "coven end-step counter"
    );
}

/// Crawling Infestation makes an Insect when a creature card hits your graveyard.
#[test]
fn crawling_infestation_spawns_insect() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.move_card_to_battlefield_for_test(0, catalog::crawling_infestation());
    drain_stack(&mut g);
    // A creature card entering the graveyard triggers the once-per-turn spawn.
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::CardPutIntoGraveyard {
        player: 0,
        card_id: dead,
        is_land: false,
    }]);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Insect"), 1, "creature-to-graveyard spawns an Insect");
}
