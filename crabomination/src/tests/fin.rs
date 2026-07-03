//! Functionality tests for the `catalog::sets::fin` (Final Fantasy) batch.

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::game::types::TurnStep;
use crate::game::*;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Iron Giant ships as a 6/6 with vigilance, reach, and trample.
#[test]
fn iron_giant_keywords() {
    let g = catalog::iron_giant();
    assert_eq!((g.power, g.toughness), (6, 6));
    for kw in [Keyword::Vigilance, Keyword::Reach, Keyword::Trample] {
        assert!(g.keywords.contains(&kw), "Iron Giant has {kw:?}");
    }
}

/// Sazh's Chocobo grows with a +1/+1 counter on landfall.
#[test]
fn sazhs_chocobo_grows_on_landfall() {
    let mut g = two_player_game();
    let bird = g.add_card_to_battlefield(0, catalog::sazhs_chocobo());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bird).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "landfall added a +1/+1 counter"
    );
}

/// Sephiroth's Intervention destroys a creature and gains 2 life.
#[test]
fn sephiroths_intervention_kills_and_gains() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::sephiroths_intervention());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Sephiroth's Intervention");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "creature destroyed");
    assert_eq!(g.players[0].life, 22, "gained 2 life");
}

/// Cactuar bounces itself at end step, but not the turn it enters.
#[test]
fn cactuar_bounces_at_end_step_unless_fresh() {
    let mut g = two_player_game();
    // Freshly entered this turn → stays.
    let fresh = g.add_card_to_battlefield(0, catalog::cactuar());
    let t = g.turn_number;
    g.battlefield_find_mut(fresh).unwrap().entered_turn = Some(t);
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(fresh).is_some(), "fresh Cactuar stays");

    // A Cactuar that didn't enter this turn returns to hand.
    let mut g2 = two_player_game();
    let old = g2.add_card_to_battlefield(0, catalog::cactuar());
    g2.battlefield_find_mut(old).unwrap().entered_turn = None;
    advance_to(&mut g2, TurnStep::End);
    drain_stack(&mut g2);
    assert!(g2.battlefield_find(old).is_none(), "old Cactuar bounced");
    assert!(g2.players[0].hand.iter().any(|c| c.id == old), "returned to hand");
}

/// Magitek Armor enters as a Crew-1 Vehicle and mints a Hero token.
#[test]
fn magitek_armor_makes_a_hero() {
    let armor = catalog::magitek_armor();
    assert!(armor.keywords.contains(&Keyword::Crew(1)));
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::magitek_armor());
    drain_stack(&mut g);
    let heroes = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Hero").count();
    assert_eq!(heroes, 1, "one Hero token");
}

/// Chocobo Racetrack makes a Bird token on landfall.
#[test]
fn chocobo_racetrack_makes_bird_on_landfall() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::chocobo_racetrack());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    let birds = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Racetrack Bird").count();
    assert_eq!(birds, 1, "one Bird token from landfall");
}

/// Malboro's Bad Breath ETB discards, drains, and exiles three from each opponent.
#[test]
fn malboro_bad_breath() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_hand(1, catalog::grizzly_bears()); }
    for _ in 0..5 {
        let id = g.next_id();
        g.players[1].library.push(crate::card::CardInstance::new(id, catalog::grizzly_bears(), 1));
    }
    let hand_before = g.players[1].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::malboro());
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand_before - 1, "opponent discarded one");
    assert_eq!(g.players[1].life, 18, "opponent lost 2 life");
    assert_eq!(g.exile.iter().filter(|c| c.owner == 1).count(), 3, "top 3 exiled");
}

/// Sephiroth, Planet's Heir shrinks opponents on ETB and grows on their deaths.
#[test]
fn sephiroth_planets_heir_etb_and_death_growth() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → dies to -2/-2
    let seph = g.move_card_to_battlefield_for_test(0, catalog::sephiroth_planets_heir());
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "opponent 2/2 died to -2/-2");
    assert_eq!(
        g.battlefield_find(seph).unwrap().counter_count(crate::card::CounterType::PlusOnePlusOne),
        1,
        "Sephiroth grew when the opponent's creature died"
    );
}

/// Aerith grows on lifegain (lifelink) and scatters her counters to legends on death.
#[test]
fn aerith_lifegain_and_death_distribution() {
    use crate::card::CounterType;
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let aerith = g.add_card_to_battlefield(0, catalog::aerith_gainsborough());
    let legend = g.add_card_to_battlefield(0, catalog::atraxa_praetors_voice());
    g.clear_sickness(aerith);
    // Attack: lifelink gains 2 → one lifegain event → one +1/+1 counter.
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: aerith, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(
        g.battlefield_find(aerith).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "lifelink lifegain grew Aerith"
    );
    // Aerith (now 3/3) dies to lethal damage → her counter lands on the legend.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(aerith)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt Aerith");
    drain_stack(&mut g);
    assert!(g.battlefield_find(aerith).is_none(), "Aerith died");
    assert_eq!(
        g.battlefield_find(legend).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "death distributed Aerith's counter onto the legend"
    );
}

/// Phoenix Down reanimates a small creature from your graveyard.
#[test]
fn phoenix_down_reanimates() {
    let mut g = two_player_game();
    let down = g.add_card_to_battlefield(0, catalog::phoenix_down());
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: down, ability_index: 0,
        target: Some(Target::Permanent(dead)), additional_targets: vec![], x_value: None,
    }).expect("activate Phoenix Down");
    drain_stack(&mut g);
    let r = g.battlefield_find(dead).expect("creature reanimated");
    assert!(r.tapped, "enters tapped");
    assert!(g.exile.iter().any(|c| c.definition.name == "Phoenix Down"), "Phoenix Down exiled itself");
}

/// Tifa doubles her power until end of turn on landfall (and it expires).
#[test]
fn tifa_doubles_power_on_landfall() {
    let mut g = two_player_game();
    let tifa = g.add_card_to_battlefield(0, catalog::tifa_lockhart());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(tifa).unwrap().power(), 2, "1 → doubled to 2");
}

/// Feather of Flight draws on entry and grants +1/+0 and flying.
#[test]
fn feather_of_flight_draws_and_grants_flying() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.add_card_to_library(0, catalog::island());
    let hand0 = g.players[0].hand.len();
    let feather = g.add_card_to_hand(0, catalog::feather_of_flight());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.cast_spell(feather, Some(Target::Permanent(bear)), vec![], None, None)
        .expect("cast the Aura");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew a card on entry");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3, "enchanted creature gets +1/+0");
    assert!(cp.keywords.contains(&Keyword::Flying), "and flying");
}

/// Vivi grows and pings each opponent when you cast a noncreature spell.
#[test]
fn vivi_grows_and_pings_on_noncreature_spell() {
    let mut g = two_player_game();
    let vivi = g.add_card_to_battlefield(0, catalog::vivi_ornitier());
    let life1 = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.cast_spell(bolt, Some(Target::Player(1)), vec![], None, None).ok();
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(vivi).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "Vivi got a +1/+1 counter"
    );
    assert!(g.players[1].life < life1, "each opponent took at least Vivi's ping");
}

/// Barret pings the defending player for each equipped creature he controls.
#[test]
fn barret_wallace_pings_per_equipped_creature() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let barret = g.add_card_to_battlefield(0, catalog::barret_wallace());
    // Give a second creature an Equipment so it counts as equipped.
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sword = g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(ally);
    g.clear_sickness(barret);
    let life1 = g.players[1].life;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: barret,
        target: AttackTarget::Player(1),
    }])).expect("Barret attacks");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 1, "one equipped creature → 1 damage on attack");
}

/// Squall grants double strike when a creature attacks alone and reanimates on
/// combat damage.
#[test]
fn squall_attacks_alone_and_reanimates() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let squall = g.add_card_to_battlefield(0, catalog::squall_seed_mercenary());
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2 permanent
    g.clear_sickness(squall);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: squall,
        target: AttackTarget::Player(1),
    }])).expect("Squall attacks alone");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(squall).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "attacks-alone granted double strike"
    );
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert!(g.battlefield_find(dead).is_some(), "combat damage reanimated the graveyard permanent");
}

/// White Mage's Staff mints a Hero and equips it, granting +1/+1.
#[test]
fn white_mages_staff_job_select() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::white_mages_staff());
    drain_stack(&mut g);
    let hero = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Hero")
        .expect("Hero token minted");
    let hid = hero.id;
    let cp = g.computed_permanent(hid).unwrap();
    assert_eq!(cp.power, 2, "Hero is 1/1 + equip +1/+1");
    assert!(
        cp.subtypes.creature_types.contains(&crate::card::CreatureType::Cleric),
        "equipped creature is a Cleric"
    );
}
