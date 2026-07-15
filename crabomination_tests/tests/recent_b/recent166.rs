//! Functionality tests for `catalog::sets::decks::recent166` — EOE/TLA staples
//! and the impulse-until-nonland primitive.

use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::mana::Color;

/// Territorial Bruntar's landfall exiles leading lands and grants a pay-own-cost
/// impulse on the first nonland card.
#[test]
fn territorial_bruntar_landfall_impulses_first_nonland() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::territorial_bruntar());
    // Library top → bottom: Mountain (land), Grizzly Bears (nonland spell).
    g.add_card_to_library(0, catalog::mountain());
    let spell = g.add_card_to_library(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    let s = g.exile.iter().find(|c| c.id == spell).expect("nonland impulsed to exile");
    assert!(s.may_play_until.is_some(), "castable this turn");
    // Pay-own-cost impulse: the granted cost equals the card's real cost.
    assert!(s.granted_alt_cast_cost_eot.is_some(), "impulse pays the card's own cost, not free");
}

/// Solstice Revelations grants a free cast when the nonland's MV is below your
/// Mountain count, else it goes to hand.
#[test]
fn solstice_revelations_free_below_mountains_else_hand() {
    // Enough Mountains: Grizzly Bears (MV 2) < 3 Mountains → free may-play.
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    let spell = g.add_card_to_library(0, catalog::grizzly_bears());
    let cast = g.add_card_to_hand(0, catalog::solstice_revelations());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.cast_spell(cast, None, vec![], None, None).expect("cast Solstice");
    drain_stack(&mut g);
    let s = g.exile.iter().find(|c| c.id == spell).expect("nonland impulsed");
    assert!(s.may_play_until.is_some(), "free may-play granted");
    assert!(s.granted_alt_cast_cost_eot.is_none(), "cast without paying its mana cost");

    // Too few Mountains: MV 2 not below 0 → put into hand instead.
    let mut g2 = two_player_game();
    let spell2 = g2.add_card_to_library(0, catalog::grizzly_bears());
    let cast2 = g2.add_card_to_hand(0, catalog::solstice_revelations());
    g2.players[0].mana_pool.add(Color::Red, 1);
    g2.players[0].mana_pool.add_colorless(2);
    g2.priority.player_with_priority = 0;
    g2.step = TurnStep::PreCombatMain;
    g2.cast_spell(cast2, None, vec![], None, None).expect("cast Solstice");
    drain_stack(&mut g2);
    assert!(g2.players[0].hand.iter().any(|c| c.id == spell2), "put into hand, no free cast");
}

/// White Lotus Hideout taps for colorless and, restricted, for any color.
#[test]
fn white_lotus_hideout_taps_for_mana() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::white_lotus_hideout());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("{T}: Add {C}");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1, "one colorless produced");
}

/// Jasmine Dragon Tea Shop's {5},{T} mints a 1/1 Ally token.
#[test]
fn jasmine_dragon_tea_shop_makes_ally() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::jasmine_dragon_tea_shop());
    g.players[0].mana_pool.add_colorless(5);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("{5},{T}: make Ally");
    drain_stack(&mut g);
    let allies = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Ally").count();
    assert_eq!(allies, 1, "one Ally token");
}

/// Secret Tunnel's {4},{T} makes a creature you control unblockable.
#[test]
fn secret_tunnel_grants_unblockable() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::secret_tunnel());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(4);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    })
    .expect("{4},{T}: unblockable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().has_keyword(&Keyword::Unblockable), "bear can't be blocked");
}

/// Planetarium's scry fires its once-per-turn impulse on the top card.
#[test]
fn planetarium_impulses_top_on_scry() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(0, catalog::planetarium_of_wan_shi_tong());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: art, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("{1},{T}: Scry 2");
    drain_stack(&mut g);
    assert!(
        g.exile.iter().any(|c| c.controller == 0 && c.may_play_until.is_some()),
        "scry triggered an impulse of the top card",
    );
}

/// Phoenix Fleet Airship copies itself at end step if you sacrificed a permanent.
#[test]
fn phoenix_fleet_airship_copies_after_sacrifice() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::phoenix_fleet_airship());
    g.active_player_idx = 0;
    // Record a sacrifice this turn.
    g.players[0].permanents_sacrificed_this_turn = 1;
    let before = g.battlefield.iter().filter(|c| c.definition.name == "Phoenix Fleet Airship").count();
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let after = g.battlefield.iter().filter(|c| c.definition.name == "Phoenix Fleet Airship").count();
    assert_eq!(after, before + 1, "a token copy was minted");
}

/// Firebender Ascension's ETB mints a firebending 2/2 Soldier.
#[test]
fn firebender_ascension_makes_soldier() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::firebender_ascension());
    drain_stack(&mut g);
    let tok = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Soldier").expect("Soldier token");
    assert_eq!((tok.power(), tok.toughness()), (2, 2));
    assert!(tok.definition.keywords.contains(&Keyword::Firebending(1)), "has firebending 1");
}

/// Ragost sacrifices a Food to burn each opponent for 3.
#[test]
fn ragost_sacrifices_food_for_damage() {
    let mut g = two_player_game();
    let ragost = g.add_card_to_battlefield(0, catalog::ragost_deft_gastronaut());
    g.clear_sickness(ragost);
    g.add_token_to_battlefield(0, &crabomination_base::tokens::food_token());
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ragost, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("{1},{T},Sac Food: 3 to each opponent");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "opponent took 3");
}

/// Ragost untaps at end step if you gained life this turn.
#[test]
fn ragost_untaps_when_life_gained() {
    let mut g = two_player_game();
    let ragost = g.add_card_to_battlefield(0, catalog::ragost_deft_gastronaut());
    g.battlefield_find_mut(ragost).unwrap().tapped = true;
    g.active_player_idx = 0;
    g.players[0].life_gained_this_turn = 2;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(ragost).unwrap().tapped, "Ragost untapped");
}

/// Invasion Submersible's exhaust animation turns it into a 3/3 artifact creature.
#[test]
fn invasion_submersible_exhaust_animates() {
    let mut g = two_player_game();
    let sub = g.add_card_to_battlefield(0, catalog::invasion_submersible());
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sub, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("exhaust {3}: animate");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(sub).unwrap().card_types.contains(&CardType::Creature),
        "became an artifact creature",
    );
    assert_eq!(
        g.battlefield_find(sub).unwrap().counter_count(CounterType::PlusOnePlusOne),
        3,
        "three +1/+1 counters",
    );
}

// ── Aetherdrift (DFT) wave ─────────────────────────────────────────────────

/// Gloryheath Lynx tutors a basic Plains to hand when it attacks while saddled.
#[test]
fn gloryheath_lynx_saddled_attack_tutors_plains() {
    let mut g = two_player_game();
    let lynx = g.add_card_to_battlefield(0, catalog::gloryheath_lynx());
    g.clear_sickness(lynx);
    g.battlefield_find_mut(lynx).unwrap().saddled = true;
    let plains = g.add_card_to_library(0, catalog::plains());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(plains))]));
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: lynx, target: AttackTarget::Player(1),
    }]))
    .unwrap();
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Plains"), "tutored a Plains");
}

/// Guardian Sunmare puts a cheap nonland permanent onto the battlefield on a
/// saddled attack.
#[test]
fn guardian_sunmare_saddled_attack_cheats_permanent() {
    let mut g = two_player_game();
    let mare = g.add_card_to_battlefield(0, catalog::guardian_sunmare());
    g.clear_sickness(mare);
    g.battlefield_find_mut(mare).unwrap().saddled = true;
    let bears = g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2 ≤ 3
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bears))]));
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: mare, target: AttackTarget::Player(1),
    }]))
    .unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"), "cheated Bears in");
}

/// Guidelight Optimizer taps for one blue mana.
#[test]
fn guidelight_optimizer_taps_for_blue() {
    let mut g = two_player_game();
    let opt = g.add_card_to_battlefield(0, catalog::guidelight_optimizer());
    g.clear_sickness(opt);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: opt, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("{T}: Add {U}");
    // The mana is spend-restricted (artifact spells/abilities only), so it
    // lands in the restricted pool rather than the open blue slot.
    assert_eq!(g.players[0].mana_pool.restricted_total(), 1, "one restricted blue produced");
}

/// Grim Bauble's ETB shrinks an opposing creature by -2/-2.
#[test]
fn grim_bauble_etb_shrinks_opponent_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::grim_bauble());
    drain_stack(&mut g);
    // The 2/2 becomes a 0/0 and dies to state-based actions.
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "shrunk bear died");
}

/// Gastal Raider gets +1/+1 and menace only at max speed.
#[test]
fn gastal_raider_grows_at_max_speed() {
    let mut g = two_player_game();
    let raider = g.add_card_to_battlefield(0, catalog::gastal_raider());
    assert_eq!(g.computed_permanent(raider).unwrap().power, 2, "base 2/1 before max speed");
    g.players[0].speed = 4;
    let cp = g.computed_permanent(raider).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 2), "max speed → 3/2");
    assert!(cp.keywords.contains(&Keyword::Menace), "gains menace at max speed");
}

/// Basri makes a lifelinking Cat token with its activated ability.
#[test]
fn basri_makes_cat_token() {
    let mut g = two_player_game();
    let basri = g.add_card_to_battlefield(0, catalog::basri_tomorrows_champion());
    g.clear_sickness(basri);
    g.players[0].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: basri, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("{W},{T}: make Cat");
    drain_stack(&mut g);
    let cat = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Cat").expect("Cat token");
    assert!(cat.definition.keywords.contains(&Keyword::Lifelink), "Cat has lifelink");
}

/// Broodheart Engine's sac ability reanimates a creature from your graveyard.
#[test]
fn broodheart_engine_reanimates_from_graveyard() {
    let mut g = two_player_game();
    let engine = g.add_card_to_battlefield(0, catalog::broodheart_engine());
    g.clear_sickness(engine);
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: engine, ability_index: 0, target: Some(Target::Permanent(dead)),
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("sac: reanimate");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == dead), "Bears back on the battlefield");
}

/// Amonkhet Raceway's max-speed ability grants haste.
#[test]
fn amonkhet_raceway_max_speed_grants_haste() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::amonkhet_raceway());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].speed = 4;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("max speed {T}: grant haste");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().has_keyword(&Keyword::Haste), "bear gained haste");
}

/// Fang-Druid Summoner tutors a creature card to hand on ETB.
#[test]
fn fang_druid_summoner_tutors_creature() {
    let mut g = two_player_game();
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bears))]));
    g.move_card_to_battlefield_for_test(0, catalog::fang_druid_summoner());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"), "tutored a creature");
}

// ── DFT legends wave ────────────────────────────────────────────────────────

/// Caradora's static adds an extra +1/+1 counter to placements on your creatures.
#[test]
fn caradora_adds_extra_counter() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::caradora_heart_of_alacria());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.resolve_effect(
        &crabomination::effect::Effect::AddCounter {
            what: crabomination::effect::Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: crabomination::effect::Value::ONE,
        },
        &crabomination::game::effects::EffectContext::for_ability(bear, 0, None),
    )
    .unwrap();
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "one counter becomes two",
    );
}

/// Far Fortune pings each opponent when you attack.
#[test]
fn far_fortune_pings_on_attack() {
    let mut g = two_player_game();
    let boss = g.add_card_to_battlefield(0, catalog::far_fortune_end_boss());
    g.clear_sickness(boss);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: boss, target: AttackTarget::Player(1),
    }]))
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "opponent pinged for 1 on attack");
}

/// Hazoret makes a small creature unblockable.
#[test]
fn hazoret_makes_small_creature_unblockable() {
    let mut g = two_player_game();
    let haz = g.add_card_to_battlefield(0, catalog::hazoret_godseeker());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: haz, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("{1},{T}: unblockable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().has_keyword(&Keyword::Unblockable), "small creature unblockable");
}

/// Aatchik's ETB makes one Insect per artifact/creature card in your graveyard.
#[test]
fn aatchik_makes_insects_from_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // not counted
    g.move_card_to_battlefield_for_test(0, catalog::aatchik_emerald_radian());
    drain_stack(&mut g);
    let insects = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Insect").count();
    assert_eq!(insects, 2, "two creature cards → two Insects");
}

/// Aatchik grows and drains when another Insect you control dies.
#[test]
fn aatchik_grows_when_insect_dies() {
    use crabomination::card::{CardType, CreatureType, Subtypes, TokenDefinition};
    let mut g = two_player_game();
    let aatchik = g.add_card_to_battlefield(0, catalog::aatchik_emerald_radian());
    let insect = TokenDefinition {
        name: "Insect".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        ..Default::default()
    };
    let bug = g.add_token_to_battlefield(0, &insect);
    g.dispatch_triggers_for_events(&[GameEvent::CreatureDied { card_id: bug }]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(aatchik).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "grew");
    assert_eq!(g.players[1].life, 19, "opponent drained 1");
}

/// Fearless Swashbuckler grants haste to your Vehicles.
#[test]
fn fearless_swashbuckler_gives_vehicles_haste() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::fearless_swashbuckler());
    let sub = g.add_card_to_battlefield(0, catalog::invasion_submersible());
    assert!(
        g.computed_permanent(sub).unwrap().keywords.contains(&Keyword::Haste),
        "Vehicle has haste from the Swashbuckler",
    );
}

// ── DFT Vehicles wave ───────────────────────────────────────────────────────

/// Gastal Thrillroller animates itself on ETB.
#[test]
fn gastal_thrillroller_enters_as_creature() {
    let mut g = two_player_game();
    let v = g.move_card_to_battlefield_for_test(0, catalog::gastal_thrillroller());
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(v).unwrap().card_types.contains(&CardType::Creature),
        "Vehicle is a creature on entry",
    );
}

/// Gastal Thrillroller returns from the graveyard with a finality counter.
#[test]
fn gastal_thrillroller_recurs_from_graveyard() {
    let mut g = two_player_game();
    let v = g.add_card_to_graveyard(0, catalog::gastal_thrillroller());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // a card to discard
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: v, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("gy recur");
    drain_stack(&mut g);
    let back = g.battlefield_find(v).expect("back on battlefield");
    assert_eq!(back.counter_count(CounterType::Finality), 1, "with a finality counter");
}

/// Apocalypse Runner grants lifelink + unblockable to a small creature.
#[test]
fn apocalypse_runner_buffs_small_creature() {
    let mut g = two_player_game();
    let v = g.add_card_to_battlefield(0, catalog::apocalypse_runner());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: v, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("{T}: buff");
    drain_stack(&mut g);
    let cp = g.battlefield_find(bear).unwrap();
    assert!(cp.has_keyword(&Keyword::Lifelink) && cp.has_keyword(&Keyword::Unblockable), "buffed");
}

/// Wingshield Agent enters with a shield counter.
#[test]
fn wingshield_agent_enters_with_shield() {
    let mut g = two_player_game();
    let w = g.move_card_to_battlefield_for_test(0, catalog::wingshield_agent());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(w).unwrap().counter_count(CounterType::Shield), 1, "one shield counter");
}

/// Country Roads sacrifices for a Pilot token.
#[test]
fn country_roads_makes_pilot() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::country_roads());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("sac for Pilot");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Pilot").count(),
        1,
        "one Pilot token",
    );
}

/// Guidelight Pathmaker tutors an artifact to hand on ETB.
#[test]
fn guidelight_pathmaker_tutors_artifact() {
    let mut g = two_player_game();
    let art = g.add_card_to_library(0, catalog::grim_bauble());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(art))]));
    g.move_card_to_battlefield_for_test(0, catalog::guidelight_pathmaker());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == art), "tutored an artifact");
}

/// Voyager Glidecar's tap-three ability animates it with flying + a counter.
#[test]
fn voyager_glidecar_animates_with_crew() {
    let mut g = two_player_game();
    let v = g.add_card_to_battlefield(0, catalog::voyager_glidecar());
    let crew: Vec<Target> = (0..3).map(|_| {
        let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(c);
        Target::Permanent(c)
    }).collect();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: v, ability_index: 0, target: None, additional_targets: crew, x_value: None,
    })
    .expect("tap 3: animate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(v).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature) && cp.keywords.contains(&Keyword::Flying), "animated flyer");
    assert_eq!(g.battlefield_find(v).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "one +1/+1 counter");
}

/// Kickoff Celebrations' ETB loots: discard one, draw two.
#[test]
fn kickoff_celebrations_loots_on_etb() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::grizzly_bears()); // the card to discard
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let before = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::kickoff_celebrations());
    drain_stack(&mut g);
    // Discard 1, draw 2 → net +1 card in hand.
    assert_eq!(g.players[0].hand.len(), before + 1, "looted: -1 discard, +2 draw");
}
