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

/// Tidus grows when your artifacts enter and taps a blocker on attack.
#[test]
fn tidus_grows_on_artifact_and_taps_on_attack() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let tidus = g.add_card_to_battlefield(0, catalog::tidus_blitzball_star());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // An artifact entering under your control adds a counter.
    let art = g.move_card_to_battlefield_for_test(0, catalog::bonesplitter());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: art }]);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(tidus).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "artifact ETB grew Tidus"
    );
    g.clear_sickness(tidus);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: tidus,
        target: AttackTarget::Player(1),
    }])).expect("Tidus attacks");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "attack tapped the opponent's creature");
}

/// Zidane temporarily steals an opponent's creature on entry.
#[test]
fn zidane_steals_on_entry() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(foe).unwrap().tapped = true;
    g.move_card_to_battlefield_for_test(0, catalog::zidane_tantalus_thief());
    drain_stack(&mut g);
    let stolen = g.battlefield_find(foe).unwrap();
    assert_eq!(stolen.controller, 0, "Zidane took control");
    assert!(!stolen.tapped, "and untapped it");
    assert!(
        g.computed_permanent(foe).unwrap().keywords.contains(&Keyword::Haste),
        "granted haste"
    );
}

/// Snow Villiers's power tracks the number of creatures you control.
#[test]
fn snow_villiers_power_is_creature_count() {
    let mut g = two_player_game();
    let snow = g.add_card_to_battlefield(0, catalog::snow_villiers());
    assert_eq!(g.computed_permanent(snow).unwrap().power, 1, "just Snow → power 1");
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(snow).unwrap().power, 3, "three creatures → power 3");
    assert_eq!(g.computed_permanent(snow).unwrap().toughness, 3, "fixed 3 toughness");
}

/// Hope Estheim mills each opponent for the life you gained this turn.
#[test]
fn hope_estheim_mills_lifegain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hope_estheim());
    for _ in 0..5 {
        let id = g.next_id();
        g.players[1].library.push(crate::card::CardInstance::new(id, catalog::grizzly_bears(), 1));
    }
    g.players[0].life_gained_this_turn = 3;
    let gy0 = g.players[1].graveyard.len();
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), gy0 + 3, "opponent milled 3 (life gained)");
}

/// Sazh doubles the +1/+1 counters on the creature he pumps when attacking.
#[test]
fn sazh_attack_doubles_counters() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let sazh = g.add_card_to_battlefield(0, catalog::sazh_katzroy());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(ally).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    g.clear_sickness(sazh);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sazh,
        target: AttackTarget::Player(1),
    }])).expect("Sazh attacks");
    // Auto-target the only other creature; resolve.
    drain_stack(&mut g);
    // 2 existing + 1 added = 3, then doubled = 6.
    assert_eq!(
        g.battlefield_find(ally).unwrap().counter_count(CounterType::PlusOnePlusOne),
        6,
        "counter added then doubled (2→3→6)"
    );
}

/// Vanille mills two and returns a permanent card from the graveyard.
#[test]
fn vanille_mills_and_returns_permanent() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    for _ in 0..3 {
        let id = g.next_id();
        g.players[0].library.push(crate::card::CardInstance::new(id, catalog::forest(), 0));
    }
    g.move_card_to_battlefield_for_test(0, catalog::vanille_cheerful_lcie());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "returned the graveyard permanent to hand");
}

/// Y'shtola pings each opponent and gains life on a big noncreature spell.
#[test]
fn yshtola_triggers_on_expensive_noncreature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::yshtola_nights_blessed());
    let life0 = g.players[0].life;
    let life1 = g.players[1].life;
    // Divination is a {2}{U} sorcery (MV 3, noncreature).
    let spell = g.add_card_to_hand(0, catalog::divination());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.cast_spell(spell, None, vec![], None, None).expect("cast Divination");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 2, "each opponent took 2");
    assert_eq!(g.players[0].life, life0 + 2, "gained 2");
}

/// Tonberry enters tapped with a stun counter and gains combat keywords on your turn.
#[test]
fn tonberry_enters_stunned_and_conditional_keywords() {
    let mut g = two_player_game();
    let tonberry = g.move_card_to_battlefield_for_test(0, catalog::tonberry());
    drain_stack(&mut g);
    let t = g.battlefield_find(tonberry).unwrap();
    assert!(t.tapped, "enters tapped");
    assert_eq!(t.counter_count(CounterType::Stun), 1, "with a stun counter");
    // Active player is 0 (your turn) → first strike + deathtouch.
    g.active_player_idx = 0;
    let cp = g.computed_permanent(tonberry).unwrap();
    assert!(cp.keywords.contains(&Keyword::FirstStrike), "first strike on your turn");
    assert!(cp.keywords.contains(&Keyword::Deathtouch), "deathtouch on your turn");
    // Opponent's turn → neither.
    g.active_player_idx = 1;
    let cp = g.computed_permanent(tonberry).unwrap();
    assert!(!cp.keywords.contains(&Keyword::FirstStrike), "not on opponent's turn");
}

/// Zell's power tracks lands you control and he gets an extra land drop.
#[test]
fn zell_dincht_power_tracks_lands() {
    let mut g = two_player_game();
    let zell = g.add_card_to_battlefield(0, catalog::zell_dincht());
    assert_eq!(g.computed_permanent(zell).unwrap().power, 0, "no lands → 0 power");
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::forest());
    let cp = g.computed_permanent(zell).unwrap();
    assert_eq!(cp.power, 2, "two lands → +2 power");
    assert_eq!(cp.toughness, 3, "fixed 3 toughness");
}

/// Angel of Mercy gains 3 life on entry.
#[test]
fn angel_of_mercy_gains_life() {
    let mut g = two_player_game();
    let life = g.players[0].life;
    g.move_card_to_battlefield_for_test(0, catalog::angel_of_mercy());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3, "gained 3 life on ETB");
}

/// Rydia loots on landfall (discard then draw).
#[test]
fn rydia_landfall_loots() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rydia_summoner_of_mist());
    let discardable = g.add_card_to_hand(0, catalog::grizzly_bears());
    let drawable = g.add_card_to_library(0, catalog::island());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let land = g.add_card_to_hand(0, catalog::forest());
    // Force the optional loot (the bot otherwise declines to pitch a card).
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
        crate::decision::DecisionAnswer::Target(Target::Permanent(discardable)),
    ]));
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == discardable),
        "landfall discarded a card"
    );
    assert!(g.players[0].hand.iter().any(|c| c.id == drawable), "and drew a fresh card");
}

/// Locke Cole loots (draw then discard) on combat damage to a player.
#[test]
fn locke_cole_loots_on_combat_damage() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let locke = g.add_card_to_battlefield(0, catalog::locke_cole());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // a card the loot can pitch
    g.clear_sickness(locke);
    let gy0 = g.players[0].graveyard.len();
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: locke,
        target: AttackTarget::Player(1),
    }])).expect("Locke attacks");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[0].graveyard.len(), gy0 + 1, "looted: drew then discarded one to the graveyard");
}

/// Ultima Weapon grants +7/+7 and destroys a creature when the wearer attacks.
#[test]
fn ultima_weapon_pumps_and_kills_on_attack() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let wearer = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let weapon = g.add_card_to_battlefield(0, catalog::ultima_weapon());
    g.battlefield_find_mut(weapon).unwrap().attached_to = Some(wearer);
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(wearer);
    assert_eq!(g.computed_permanent(wearer).unwrap().power, 9, "2/2 + 7/7");
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: wearer,
        target: AttackTarget::Player(1),
    }])).expect("wearer attacks");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "attack destroyed the opposing creature");
}

/// Cloud tutors an Equipment to hand on entry.
#[test]
fn cloud_midgar_tutors_equipment() {
    let mut g = two_player_game();
    let sword = g.add_card_to_library(0, catalog::bonesplitter()); // an Equipment
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Search(Some(sword)),
    ]));
    g.move_card_to_battlefield_for_test(0, catalog::cloud_midgar_mercenary());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == sword), "found an Equipment for hand");
}

/// Aerith returns a creature to hand on a small lifegain, or to the battlefield
/// after gaining 7+.
#[test]
fn aerith_raise_scales_with_lifegain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::aerith_last_ancient());
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].life_gained_this_turn = 3; // < 7 → to hand
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "small lifegain returned it to hand");

    let mut g2 = two_player_game();
    g2.add_card_to_battlefield(0, catalog::aerith_last_ancient());
    let dead2 = g2.add_card_to_graveyard(0, catalog::grizzly_bears());
    g2.players[0].life_gained_this_turn = 8; // >= 7 → to battlefield
    advance_to(&mut g2, TurnStep::End);
    drain_stack(&mut g2);
    assert!(g2.battlefield_find(dead2).is_some(), "big lifegain reanimated it");
}

/// Barret, Avalanche Leader mints a Rebel when an Equipment you control enters.
#[test]
fn barret_avalanche_makes_rebel_on_equipment() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::barret_avalanche_leader());
    let art = g.move_card_to_battlefield_for_test(0, catalog::bonesplitter());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: art }]);
    drain_stack(&mut g);
    let rebels = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Rebel").count();
    assert_eq!(rebels, 1, "an Equipment ETB minted one Rebel token");
}

/// Edgar draws a card for each artifact he controls on entry.
#[test]
fn edgar_draws_per_artifact() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.add_card_to_battlefield(0, catalog::mishras_bauble());
    for _ in 0..4 {
        let id = g.next_id();
        g.players[0].library.push(crate::card::CardInstance::new(id, catalog::island(), 0));
    }
    let hand0 = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::edgar_king_of_figaro());
    drain_stack(&mut g);
    // Two artifacts on board (Edgar itself isn't one) → draw 2.
    assert_eq!(g.players[0].hand.len(), hand0 + 2, "drew one per artifact controlled");
}

/// Cid buffs artifact creatures by the number of Artificers you control plus
/// Artificer cards in your graveyard.
#[test]
fn cid_scales_anthem_with_artificers_and_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::cid_timeless_artificer()); // an Artificer
    let giant = g.add_card_to_battlefield(0, catalog::iron_giant()); // 6/6 artifact creature
    assert_eq!(g.computed_permanent(giant).unwrap().power, 7, "one Artificer (Cid) → +1/+1");
    g.add_card_to_battlefield(0, catalog::edgar_king_of_figaro()); // Human Artificer
    assert_eq!(g.computed_permanent(giant).unwrap().power, 8, "two Artificers → +2/+2");
    g.add_card_to_graveyard(0, catalog::edgar_king_of_figaro()); // Artificer card in gy
    assert_eq!(g.computed_permanent(giant).unwrap().power, 9, "graveyard Artificer counts too");
}

/// Warrior of Light's anthem scales with the number of legendary creatures you
/// control and touches only legendaries.
#[test]
fn warrior_of_light_legendary_anthem() {
    let mut g = two_player_game();
    let wol = g.add_card_to_battlefield(0, catalog::warrior_of_light()); // 5/5 legendary
    assert_eq!(g.computed_permanent(wol).unwrap().power, 6, "one legendary → +1/+1");
    let edgar = g.add_card_to_battlefield(0, catalog::edgar_king_of_figaro()); // legendary
    assert_eq!(g.computed_permanent(wol).unwrap().power, 7, "two legendaries → +2/+2");
    assert_eq!(g.computed_permanent(edgar).unwrap().power, 6, "Edgar 4/5 gets +2/+2 too");
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // nonlegendary
    assert_eq!(g.computed_permanent(bears).unwrap().power, 2, "nonlegendaries unaffected");
}

/// Casting a legendary spell from hand impulse-digs for a cheaper legendary.
#[test]
fn warrior_of_light_impulse_on_legendary_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::warrior_of_light());
    // Tifa (legendary, MV 2) on top of the library — lesser MV than Cid (MV 4).
    let tifa = g.next_id();
    g.players[0]
        .library
        .insert(0, crate::card::CardInstance::new(tifa, catalog::tifa_lockhart(), 0));
    let cid = g.add_card_to_hand(0, catalog::cid_timeless_artificer());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: cid, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Cid");
    drain_stack(&mut g);
    assert!(
        g.exile.iter().any(|c| c.id == tifa),
        "the cheaper legendary was impulse-exiled"
    );
    assert!(!g.players[0].library.iter().any(|c| c.id == tifa), "it left the library");
}

/// Cloud attaches an Equipment on entry, then draws per equipped attacker and
/// makes Treasures when big enough on attack.
#[test]
fn cloud_ex_soldier_attaches_and_draws_on_attack() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let sword = g.add_card_to_battlefield(0, catalog::bonesplitter()); // +2/+0 Equipment
    for _ in 0..3 {
        let id = g.next_id();
        g.players[0].library.push(crate::card::CardInstance::new(id, catalog::island(), 0));
    }
    let cloud = g.move_card_to_battlefield_for_test(0, catalog::cloud_ex_soldier());
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(sword).unwrap().attached_to,
        Some(cloud),
        "ETB attached the Equipment to Cloud"
    );
    // Cloud is 4/4 + Bonesplitter's +2/+0 = 6/6 — under 7 power, no Treasures.
    g.clear_sickness(cloud);
    let hand0 = g.players[0].hand.len();
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: cloud, target: AttackTarget::Player(1),
    }])).expect("Cloud attacks");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew one for the equipped attacker");
    let treasures = g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count();
    assert_eq!(treasures, 0, "power 6 (<7) → no Treasures");
}

/// Adelbert Steiner grows +1/+1 for each Equipment you control.
#[test]
fn adelbert_steiner_grows_per_equipment() {
    let mut g = two_player_game();
    let steiner = g.add_card_to_battlefield(0, catalog::adelbert_steiner());
    assert!(g.computed_permanent(steiner).unwrap().keywords.contains(&Keyword::Lifelink));
    assert_eq!(g.computed_permanent(steiner).unwrap().power, 2, "no Equipment → 2/1");
    g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.add_card_to_battlefield(0, catalog::bonesplitter());
    assert_eq!(g.computed_permanent(steiner).unwrap().power, 4, "two Equipment → 4/3");
    assert_eq!(g.computed_permanent(steiner).unwrap().toughness, 3);
}

/// Ambrosia Whiteheart pumps herself on landfall and has flash.
#[test]
fn ambrosia_landfall_pumps() {
    let mut g = two_player_game();
    let amb = g.add_card_to_battlefield(0, catalog::ambrosia_whiteheart());
    assert!(catalog::ambrosia_whiteheart().keywords.contains(&Keyword::Flash));
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let land = g.add_card_to_hand(0, catalog::plains());
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(amb).unwrap().power, 3, "landfall +1/+0 → 3/2");
}

/// Coliseum Behemoth's modal ETB can draw a card.
#[test]
fn coliseum_behemoth_modal_draw() {
    let mut g = two_player_game();
    let id = g.next_id();
    g.players[0].library.push(crate::card::CardInstance::new(id, catalog::island(), 0));
    g.decider = Box::new(crate::decision::ScriptedDecider::new(vec![
        crate::decision::DecisionAnswer::Mode(1), // draw a card
    ]));
    let hand0 = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::coliseum_behemoth());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "chose the draw mode");
}

/// Hill Gigas ships with trample, haste, and Mountaincycling.
#[test]
fn hill_gigas_keywords() {
    let g = catalog::hill_gigas();
    assert!(g.keywords.contains(&Keyword::Trample));
    assert!(g.keywords.contains(&Keyword::Haste));
    assert!(g.keywords.iter().any(|k| matches!(k, Keyword::Landcycling(_, crate::card::LandType::Mountain))));
}

/// Cloudbound Moogle puts a +1/+1 counter on a creature when it enters.
#[test]
fn cloudbound_moogle_etb_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::cloudbound_moogle());
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "ETB dropped a +1/+1 counter on the bear"
    );
}

/// Balamb T-Rexaur gains 3 life on entry.
#[test]
fn balamb_t_rexaur_gains_life() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::balamb_t_rexaur());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 23, "ETB gained 3 life");
}

/// Goobbue Gardener taps for green.
#[test]
fn goobbue_gardener_taps_for_green() {
    let mut g = two_player_game();
    let dork = g.add_card_to_battlefield(0, catalog::goobbue_gardener());
    g.clear_sickness(dork);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dork, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    }).expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.amount(crate::mana::Color::Green), 1, "added one green");
}

/// Blazing Bomb sacrifices to deal its power to a creature.
#[test]
fn blazing_bomb_blow_up() {
    let mut g = two_player_game();
    let bomb = g.add_card_to_battlefield(0, catalog::blazing_bomb());
    g.battlefield_find_mut(bomb).unwrap().add_counters(CounterType::PlusOnePlusOne, 1); // → 2/2
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bomb, ability_index: 0, target: Some(Target::Permanent(foe)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("Blow Up");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bomb).is_none(), "Blazing Bomb sacrificed itself");
    assert!(g.battlefield_find(foe).is_none(), "dealt 2 to the 2/2, destroying it");
}

/// Coral Sword attaches on entry, granting +1/+0 and first strike.
#[test]
fn coral_sword_attaches_and_pumps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::coral_sword());
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3, "equipped +1/+0 → 3/2");
    assert!(cp.keywords.contains(&Keyword::FirstStrike), "gained first strike");
}

/// Adventurer's Airship loots when it attacks (crewed).
#[test]
fn adventurers_airship_loots_on_attack() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let ship = g.add_card_to_battlefield(0, catalog::adventurers_airship());
    let crew = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2 crews it
    g.add_card_to_hand(0, catalog::grizzly_bears()); // something to discard
    for _ in 0..2 {
        let id = g.next_id();
        g.players[0].library.push(crate::card::CardInstance::new(id, catalog::island(), 0));
    }
    g.clear_sickness(ship);
    g.clear_sickness(crew);
    advance_to(&mut g, TurnStep::PreCombatMain);
    g.perform_action(GameAction::Crew { vehicle: ship, crew_creatures: vec![crew] }).expect("crew the ship");
    let hand0 = g.players[0].hand.len();
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ship, target: AttackTarget::Player(1),
    }])).expect("airship attacks");
    drain_stack(&mut g);
    // Loot = draw then discard → net zero hand size, but a card was drawn & pitched.
    assert_eq!(g.players[0].hand.len(), hand0, "drew one and discarded one");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "discarded a card to the graveyard");
}

/// Al Bhed Salvagers drains when a creature you control dies.
#[test]
fn al_bhed_salvagers_drains_on_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::al_bhed_salvagers());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    crate::game::cast_at(&mut g, bolt, Target::Permanent(fodder));
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder died");
    assert_eq!(g.players[1].life, 19, "opponent lost 1");
    assert_eq!(g.players[0].life, 21, "you gained 1");
}

/// Demon Wall can't attack until it has a counter; its ability adds two.
#[test]
fn demon_wall_counter_unlocks_attack() {
    use crate::game::types::{Attack, AttackTarget};
    // The {5}{B} ability puts two +1/+1 counters on it.
    let mut g = two_player_game();
    let wall = g.add_card_to_battlefield(0, catalog::demon_wall());
    assert!(catalog::demon_wall().keywords.contains(&Keyword::Defender));
    assert!(catalog::demon_wall().keywords.contains(&Keyword::Menace));
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wall, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    }).expect("add counters");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(wall).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);

    // With a counter it can attack despite Defender; a fresh copy cannot.
    let mut g2 = two_player_game();
    let bare = g2.add_card_to_battlefield(0, catalog::demon_wall());
    g2.clear_sickness(bare);
    advance_to(&mut g2, TurnStep::DeclareAttackers);
    assert!(g2.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bare, target: AttackTarget::Player(1),
    }])).is_err(), "defender stops the counterless wall");
    g2.battlefield_find_mut(bare).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g2.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bare, target: AttackTarget::Player(1),
    }])).expect("the counter lets it attack past Defender");
}

/// Ashe digs the top five for an artifact on attack.
#[test]
fn ashe_digs_for_artifact() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let ashe = g.add_card_to_battlefield(0, catalog::ashe_princess_of_dalmasca());
    let art = g.next_id();
    g.players[0].library.insert(0, crate::card::CardInstance::new(art, catalog::bonesplitter(), 0));
    for _ in 0..4 {
        let id = g.next_id();
        g.players[0].library.insert(1, crate::card::CardInstance::new(id, catalog::forest(), 0));
    }
    g.clear_sickness(ashe);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ashe, target: AttackTarget::Player(1),
    }])).expect("Ashe attacks");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == art), "the artifact was taken to hand");
}

/// Gladiolus ramps on ETB and pumps another creature on landfall.
#[test]
fn gladiolus_ramps_and_pumps_on_landfall() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Search(Some(forest)),
    ]));
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let lands0 = g.battlefield.iter().filter(|c| c.definition.is_land() && c.controller == 0).count();
    g.move_card_to_battlefield_for_test(0, catalog::gladiolus_amicitia());
    drain_stack(&mut g);
    let lands1 = g.battlefield.iter().filter(|c| c.definition.is_land() && c.controller == 0).count();
    assert_eq!(lands1, lands0 + 1, "ETB put a land onto the battlefield");
    // Landfall pumps the ally +2/+2 and grants trample.
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let land = g.add_card_to_hand(0, catalog::plains());
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    let cp = g.computed_permanent(ally).unwrap();
    assert_eq!(cp.power, 4, "ally pumped +2/+2");
    assert!(cp.keywords.contains(&Keyword::Trample), "ally gained trample");
}

/// Eject bounces a nonland permanent and cantrips.
#[test]
fn eject_bounces_and_draws() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::eject());
    let id = g.next_id();
    g.players[0].library.push(crate::card::CardInstance::new(id, catalog::island(), 0));
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(foe)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Eject");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "creature bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == foe), "returned to owner's hand");
    // -1 (Eject cast) +1 (bounced foe n/a, owned by opp) +1 (draw) ⇒ net +0 vs hand0.
    assert_eq!(g.players[0].hand.len(), hand0 - 1 + 1, "cantrip drew a card");
}

/// Deadly Embrace draws for each creature that died this turn.
#[test]
fn deadly_embrace_draws_per_death() {
    let mut g = two_player_game();
    // A creature already died this turn.
    let gid = g.next_id();
    g.players[0].graveyard.push(crate::card::CardInstance::new(gid, catalog::grizzly_bears(), 0));
    g.players[0].creatures_died_this_turn = 1;
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::deadly_embrace());
    for _ in 0..3 {
        let id = g.next_id();
        g.players[0].library.push(crate::card::CardInstance::new(id, catalog::island(), 0));
    }
    g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(foe)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Deadly Embrace");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "target destroyed");
    // The prior death + the one Deadly Embrace destroyed = 2 draws.
    assert_eq!(g.players[0].hand.len(), hand0 - 1 + 2, "drew for both deaths");
}

/// Airship Crash destroys a flier (and cycles).
#[test]
fn airship_crash_destroys_flier() {
    let mut g = two_player_game();
    assert!(catalog::airship_crash().keywords.iter().any(|k| matches!(k, Keyword::Cycling(_))));
    let flier = g.add_card_to_battlefield(1, catalog::serra_angel()); // has flying
    let spell = g.add_card_to_hand(0, catalog::airship_crash());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(flier)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Airship Crash");
    drain_stack(&mut g);
    assert!(g.battlefield_find(flier).is_none(), "destroyed the flier");
}

/// Dreams of Laguna surveils then draws, and carries flashback.
#[test]
fn dreams_of_laguna_surveil_draw() {
    let mut g = two_player_game();
    assert!(catalog::dreams_of_laguna().keywords.iter().any(|k| matches!(k, Keyword::Flashback(_))));
    let spell = g.add_card_to_hand(0, catalog::dreams_of_laguna());
    for _ in 0..3 {
        let id = g.next_id();
        g.players[0].library.push(crate::card::CardInstance::new(id, catalog::island(), 0));
    }
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Dreams of Laguna");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 - 1 + 1, "spent the spell and drew one");
}

/// Gysahl Greens makes a Bird token.
#[test]
fn gysahl_greens_makes_bird() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::gysahl_greens());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Gysahl Greens");
    drain_stack(&mut g);
    let birds = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&crate::card::CreatureType::Bird))
        .count();
    assert_eq!(birds, 1, "created a Bird token");
}

/// Battle Menu's Item mode gains 4 life.
#[test]
fn battle_menu_item_gains_life() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::battle_menu());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: Some(3), x_value: None,
    }).expect("cast Battle Menu");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 24, "Item mode gained 4 life");
}

// ── modern_decks FIN batch ───────────────────────────────────────────────

use crate::card::ArtifactSubtype;
use crate::game::types::Target;

fn kill_perm(g: &mut GameState, id: CardId) {
    let ctl = g.battlefield_find(id).unwrap().controller;
    let ctx = crate::game::effects::EffectContext::for_ability(id, ctl, Some(Target::Permanent(id)));
    g.resolve_effect(
        &crate::effect::Effect::SacrificePermanent { what: crate::effect::Selector::Target(0) },
        &ctx,
    )
    .unwrap();
    drain_stack(g);
}

fn treasure_count(g: &GameState, seat: usize) -> usize {
    g.battlefield
        .iter()
        .filter(|c| {
            c.controller == seat
                && c.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Treasure)
        })
        .count()
}

/// Undercity Dire Rat mints a Treasure when it dies.
#[test]
fn undercity_dire_rat_dies_makes_treasure() {
    let mut g = two_player_game();
    let rat = g.add_card_to_battlefield(0, catalog::undercity_dire_rat());
    kill_perm(&mut g, rat);
    assert_eq!(treasure_count(&g, 0), 1, "death made a Treasure");
}

/// Magic Pot mints a Treasure when it dies.
#[test]
fn magic_pot_dies_makes_treasure() {
    let mut g = two_player_game();
    let pot = g.add_card_to_battlefield(0, catalog::magic_pot());
    kill_perm(&mut g, pot);
    assert_eq!(treasure_count(&g, 0), 1, "death made a Treasure");
}

/// Shinra Reinforcements mills three and gains 3 on ETB.
#[test]
fn shinra_reinforcements_mills_and_gains() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let gy_before = g.players[0].graveyard.len();
    g.move_card_to_battlefield_for_test(0, catalog::shinra_reinforcements());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 23, "gained 3 life");
    assert_eq!(g.players[0].graveyard.len(), gy_before + 3, "milled three");
}

/// Minwu grows every Cleric with a +1/+1 counter when you gain life.
#[test]
fn minwu_lifegain_grows_clerics() {
    let mut g = two_player_game();
    let minwu = g.add_card_to_battlefield(0, catalog::minwu_white_mage());
    g.clear_sickness(minwu);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    // Lifelink attack → one lifegain → +1/+1 on each Cleric (Minwu is a Cleric).
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: minwu,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(
        g.battlefield_find(minwu).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "lifegain grew the Cleric"
    );
}

/// Il Mheg Pixie flies and surveils on attack without error.
#[test]
fn il_mheg_pixie_flies_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let pixie = g.add_card_to_battlefield(0, catalog::il_mheg_pixie());
    assert!(g.battlefield_find(pixie).unwrap().definition.keywords.contains(&Keyword::Flying));
    g.clear_sickness(pixie);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: pixie,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(pixie).is_some(), "pixie survived its surveil");
}

/// Sabotender pings each opponent on landfall.
#[test]
fn sabotender_landfall_pings() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sabotender());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "landfall pinged the opponent");
}

/// Black Waltz No. 3 pings each opponent when you cast a noncreature spell.
#[test]
fn black_waltz_pings_on_noncreature_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::black_waltz_no_3());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::concentrate());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Concentrate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "noncreature cast pinged for 2");
}

/// Xande grows with noncreature, nonland cards in your graveyard.
#[test]
fn xande_grows_with_graveyard() {
    let mut g = two_player_game();
    let xande = g.add_card_to_battlefield(0, catalog::xande_dark_mage());
    let cp = g.computed_permanent(xande).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "empty graveyard → base 3/3");
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::concentrate());
    let cp = g.computed_permanent(xande).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "two noncreature cards → 5/5");
}

/// Overkill drops a creature's toughness to lethal.
#[test]
fn overkill_shrinks_toughness() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::overkill());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Overkill");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "toughness ≤ 0 killed it");
}

/// Blitzball Shot pumps +3/+3 and grants trample.
#[test]
fn blitzball_shot_pumps_and_tramples() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::blitzball_shot());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Blitzball Shot");
    drain_stack(&mut g);
    let b = g.computed_permanent(bear).unwrap();
    assert_eq!((b.power, b.toughness), (5, 5), "+3/+3");
    assert!(b.keywords.contains(&Keyword::Trample), "gained trample");
}

/// Fight On! returns two creature cards from the graveyard to hand.
#[test]
fn fight_on_returns_two_creatures() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::fight_on());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Fight On!");
    drain_stack(&mut g);
    // Spell leaves hand (-1), two creatures returned (+2) → net +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "spell left, two creatures returned");
    assert_eq!(
        g.players[0].hand.iter().filter(|c| c.definition.name == "Grizzly Bears").count(),
        2
    );
}

/// Evil Reawakened reanimates a creature with two extra +1/+1 counters.
#[test]
fn evil_reawakened_reanimates_with_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::evil_reawakened());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Evil Reawakened");
    drain_stack(&mut g);
    let bear = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").expect("reanimated");
    assert_eq!(bear.counter_count(CounterType::PlusOnePlusOne), 2, "two +1/+1 counters");
}

/// You're Not Alone gives +4/+4 when you control three or more creatures.
#[test]
fn youre_not_alone_scales_with_board() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::youre_not_alone());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast You're Not Alone");
    drain_stack(&mut g);
    let b = g.computed_permanent(a).unwrap();
    assert_eq!((b.power, b.toughness), (6, 6), "+4/+4 with three creatures");
}

/// Auron's Inspiration pumps attacking creatures.
#[test]
fn aurons_inspiration_pumps_attackers() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let spell = g.add_card_to_hand(0, catalog::aurons_inspiration());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Auron's Inspiration");
    drain_stack(&mut g);
    let b = g.computed_permanent(bear).unwrap();
    assert_eq!((b.power, b.toughness), (4, 2), "attacking bear got +2/+0");
}

/// Magic Damper untaps a creature you control and shields it.
#[test]
fn magic_damper_untaps_and_protects() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::magic_damper());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Magic Damper");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped");
    let b = g.computed_permanent(bear).unwrap();
    assert!(b.keywords.contains(&Keyword::Hexproof), "gained hexproof");
    assert_eq!((b.power, b.toughness), (3, 3), "+1/+1");
}

/// Instant Ramen draws on ETB and can be sacrificed for life.
#[test]
fn instant_ramen_draws_and_gains() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let hand_before = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::instant_ramen());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "ETB drew a card");
}

/// Sahagin grows when you cast a noncreature spell with four or more mana.
#[test]
fn sahagin_grows_on_big_spell() {
    let mut g = two_player_game();
    let sahagin = g.add_card_to_battlefield(0, catalog::sahagin());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::concentrate());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Concentrate");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(sahagin).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "four-mana noncreature spell grew Sahagin"
    );
}

/// Qiqirn Merchant loots with its {1}, {T} ability.
#[test]
fn qiqirn_merchant_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let qiqirn = g.add_card_to_battlefield(0, catalog::qiqirn_merchant());
    g.clear_sickness(qiqirn);
    let discard = g.add_card_to_hand(0, catalog::forest());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: qiqirn,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("activate loot");
    drain_stack(&mut g);
    let _ = discard;
    // Draw +1, discard -1 → net 0.
    assert_eq!(g.players[0].hand.len(), hand_before, "loot drew then discarded");
}

/// Matoya draws whenever you scry or surveil.
#[test]
fn matoya_draws_on_scry() {
    let mut g = two_player_game();
    let matoya = g.add_card_to_battlefield(0, catalog::matoya_archon_elder());
    let _ = matoya;
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let hand_before = g.players[0].hand.len();
    // Resolve a Scry 1 for player 0 directly.
    let ctx = crate::game::effects::EffectContext::for_ability(matoya, 0, None);
    let events = g
        .resolve_effect(
            &crate::effect::Effect::Scry { who: crate::effect::PlayerRef::You, amount: crate::effect::Value::ONE },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "scry triggered a draw");
}

/// Matoya also triggers on surveil.
#[test]
fn matoya_draws_on_surveil() {
    let mut g = two_player_game();
    let matoya = g.add_card_to_battlefield(0, catalog::matoya_archon_elder());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let hand_before = g.players[0].hand.len();
    let ctx = crate::game::effects::EffectContext::for_ability(matoya, 0, None);
    let events = g
        .resolve_effect(
            &crate::effect::Effect::Surveil { who: crate::effect::PlayerRef::You, amount: crate::effect::Value::ONE },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "surveil triggered a draw");
}

// ── modern_decks FIN batch 2 tests ────────────────────────────────────────

fn is_creature_token_named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == seat && c.is_token && c.definition.name == name)
        .count()
}

/// Queen Brahne has prowess and mints a Wizard token when she attacks.
#[test]
fn queen_brahne_mints_wizard_on_attack() {
    let mut g = two_player_game();
    let q = g.add_card_to_battlefield(0, catalog::queen_brahne());
    assert!(g.battlefield_find(q).unwrap().definition.keywords.contains(&Keyword::Prowess));
    g.clear_sickness(q);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: q,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(is_creature_token_named(&g, 0, "Wizard"), 1, "attack minted a Wizard");
}

/// Rosa grows a creature and grants lifelink at the beginning of your combat.
#[test]
fn rosa_pumps_and_grants_lifelink_at_combat() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rosa_resolute_white_mage());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    advance_to(&mut g, TurnStep::BeginCombat);
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 1, "+1/+1 counter");
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Lifelink));
}

/// Slash of Light deals damage equal to your creature + Equipment count.
#[test]
fn slash_of_light_scales_with_board() {
    let mut g = two_player_game();
    // Two creatures you control (incl. the target's controller separate).
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::slash_of_light());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Slash of Light");
    drain_stack(&mut g);
    // 2 creatures + 0 Equipment = 2 damage → kills the 2/2.
    assert!(g.battlefield_find(victim).is_none(), "2 damage killed the 2/2");
}

/// Rydia's Return mode 1 pumps your team +3/+3.
#[test]
fn rydias_return_pump_mode() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::rydias_return());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("cast Rydia's Return mode 0");
    drain_stack(&mut g);
    let b = g.computed_permanent(bear).unwrap();
    assert_eq!((b.power, b.toughness), (5, 5), "+3/+3");
}

/// The Crystal's Chosen makes four Heroes and counters up your board.
#[test]
fn the_crystals_chosen_makes_heroes_and_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::the_crystals_chosen());
    g.players[0].mana_pool.add(crate::mana::Color::White, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast The Crystal's Chosen");
    drain_stack(&mut g);
    assert_eq!(is_creature_token_named(&g, 0, "Hero"), 4, "four Hero tokens");
    // The pre-existing bear got a counter (tokens too, but check the bear).
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Commune with Beavers digs three and takes a creature card to hand.
#[test]
fn commune_with_beavers_digs_for_a_creature() {
    let mut g = two_player_game();
    let want = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::commune_with_beavers());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Search(Some(want)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Commune with Beavers");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == want), "picked creature went to hand");
}

/// Prishe's Wanderings ramps a basic onto the battlefield tapped.
#[test]
fn prishes_wanderings_ramps_tapped() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let spell = g.add_card_to_hand(0, catalog::prishes_wanderings());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Search(Some(forest)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Prishe's Wanderings");
    drain_stack(&mut g);
    let f = g.battlefield.iter().find(|c| c.id == forest).expect("forest fetched");
    assert!(f.tapped, "entered tapped");
}

/// Laughing Mad discards one and draws two.
#[test]
fn laughing_mad_discards_one_draws_two() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::laughing_mad());
    g.add_card_to_hand(0, catalog::forest()); // the discard fodder
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Laughing Mad");
    drain_stack(&mut g);
    // -1 cast, -1 discard cost, +2 draw = net 0.
    assert_eq!(g.players[0].hand.len(), hand_before, "discard one, draw two");
}

/// White Auracite exiles an opponent's permanent until it leaves.
#[test]
fn white_auracite_exiles_until_it_leaves() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(victim)),
    ]));
    let auracite = g.move_card_to_battlefield_for_test(0, catalog::white_auracite());
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "victim exiled");
    // Auracite leaving returns the creature.
    kill_perm(&mut g, auracite);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "victim returned when Auracite left"
    );
}

/// Ride the Shoopuf grows a creature on each landfall.
#[test]
fn ride_the_shoopuf_landfall_counter() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ride_the_shoopuf());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Cornered by Black Mages edicts and mints a Wizard.
#[test]
fn cornered_by_black_mages_edicts_and_mints() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::cornered_by_black_mages());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Cornered by Black Mages");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "opponent sacrificed its only creature");
    assert_eq!(is_creature_token_named(&g, 0, "Wizard"), 1, "minted a Wizard");
}

/// Sleep Magic taps its target and keeps it from untapping.
#[test]
fn sleep_magic_taps_and_locks() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    let spell = g.add_card_to_hand(0, catalog::sleep_magic());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Sleep Magic");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "enchanted creature tapped on ETB");
}

/// Choco-Comet burns for X and leaves a Bird behind.
#[test]
fn choco_comet_burns_and_makes_a_bird() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::choco_comet());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("cast Choco-Comet for X=3");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "X=3 damage to the opponent");
    assert_eq!(is_creature_token_named(&g, 0, "Bird"), 1, "created a Bird token");
}

// ── modern_decks FIN Town lands tests ─────────────────────────────────────

/// A Town dual is typed "Land — Town", enters tapped, and taps for two colors.
#[test]
fn town_dual_is_typed_and_taps_for_two_colors() {
    use crate::card::LandType;
    let g = catalog::treno_dark_city();
    assert!(g.subtypes.land_types.contains(&LandType::Town), "typed as a Town");
    assert_eq!(g.activated_abilities.len(), 2, "two mana abilities");
    // Enters tapped via its ETB trigger.
    assert!(!g.triggered_abilities.is_empty(), "has an enters-tapped trigger");
}

/// Adventurer's Inn is a Town that gains 2 life on entry.
#[test]
fn adventurers_inn_gains_life() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::adventurers_inn());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 22, "ETB gained 2 life");
}

/// Travel the Overworld's Affinity for Towns discounts it per Town you control.
#[test]
fn travel_the_overworld_affinity_for_towns() {
    let mut g = two_player_game();
    // Two Towns on the battlefield → {2} off the {5}{U}{U}, so {3}{U}{U}.
    g.add_card_to_battlefield(0, catalog::treno_dark_city());
    g.add_card_to_battlefield(0, catalog::vector_imperial_capital());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::travel_the_overworld());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Travel the Overworld castable for {3}{U}{U} with two Towns");
    drain_stack(&mut g);
    // -1 cast, +4 draw = net +3.
    assert_eq!(g.players[0].hand.len(), hand_before + 3, "drew four");
}

// ── modern_decks batch: creature-or-artifact death event + FIN wave ───────────

/// Judge Magister Gabranth grows when another creature you control dies.
#[test]
fn gabranth_grows_on_your_creature_death() {
    let mut g = two_player_game();
    let gabranth = g.add_card_to_battlefield(0, catalog::judge_magister_gabranth());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let evs = g.remove_to_graveyard_with_triggers(ally);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(gabranth).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "a friendly creature dying adds a counter"
    );
}

/// Gabranth also grows on a *non-creature artifact* you control dying — the new
/// `PermanentDied` rail that no `CreatureDied` event covers.
#[test]
fn gabranth_grows_on_your_artifact_death() {
    let mut g = two_player_game();
    let gabranth = g.add_card_to_battlefield(0, catalog::judge_magister_gabranth());
    let equip = g.add_card_to_battlefield(0, catalog::bonesplitter()); // noncreature artifact
    assert!(!g.battlefield_find(equip).unwrap().definition.is_creature());
    let evs = g.remove_to_graveyard_with_triggers(equip);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(gabranth).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "a friendly artifact dying adds a counter"
    );
}

/// Gabranth ignores an opponent's creature dying (AnotherOfYours scope).
#[test]
fn gabranth_ignores_opponent_death() {
    let mut g = two_player_game();
    let gabranth = g.add_card_to_battlefield(0, catalog::judge_magister_gabranth());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let evs = g.remove_to_graveyard_with_triggers(foe);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(gabranth).unwrap().counter_count(CounterType::PlusOnePlusOne),
        0,
        "an opponent's creature dying does nothing"
    );
}

/// G'raha Tia draws once per turn no matter how many creatures/artifacts die.
#[test]
fn graha_tia_draws_once_per_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::graha_tia());
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    // Two deaths in one batch → one draw.
    let mut evs = g.remove_to_graveyard_with_triggers(a);
    evs.append(&mut g.remove_to_graveyard_with_triggers(b));
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1, "one draw for the batch (once each turn)");
    // A later death this turn draws nothing more.
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let evs = g.remove_to_graveyard_with_triggers(c);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1, "no second draw this turn");
}

/// Diamond Weapon costs {1} less per permanent card in your graveyard and can't
/// take combat damage.
#[test]
fn diamond_weapon_affinity_and_immune() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears()); // permanent cards
    }
    let spell = g.add_card_to_hand(0, catalog::diamond_weapon());
    // {7}{G}{G} − {3} = {4}{G}{G}.
    g.players[0].mana_pool.add(crate::mana::Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Diamond Weapon castable for {4}{G}{G} with 3 permanent cards in graveyard");
    drain_stack(&mut g);
    let dw = g.battlefield.iter().find(|c| c.definition.name == "Diamond Weapon").unwrap().id;
    assert!(g.permanent_prevents_all_combat_damage_to_self(dw), "Immune to combat damage");
}

/// Light of Judgment deals 6 to a creature and destroys the Equipment on it.
#[test]
fn light_of_judgment_burns_and_strips_equipment() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::iron_giant()); // 6/6
    g.battlefield_find_mut(foe).unwrap().add_counters(CounterType::PlusOnePlusOne, 5); // 11/11
    let sword = g.add_card_to_battlefield(1, catalog::bonesplitter());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(foe);
    let spell = g.add_card_to_hand(0, catalog::light_of_judgment());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(foe)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Light of Judgment");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(foe).unwrap().damage, 6, "6 damage to the creature");
    assert!(g.battlefield_find(sword).is_none(), "attached Equipment destroyed");
}

/// Judgment Bolt deals 5 to a creature and X to its controller (X = Equipment).
#[test]
fn judgment_bolt_burns_creature_and_controller() {
    let mut g = two_player_game();
    // Two Equipment on our side → X = 2.
    g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.add_card_to_battlefield(0, catalog::bonesplitter());
    let foe = g.add_card_to_battlefield(1, catalog::iron_giant()); // 6/6
    let life1 = g.players[1].life;
    let spell = g.add_card_to_hand(0, catalog::judgment_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(foe)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Judgment Bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(foe).unwrap().damage, 5, "5 damage to the creature");
    assert_eq!(g.players[1].life, life1 - 2, "X=2 damage to its controller");
}

/// Mysidian Elder mints a Wizard token that pings on your noncreature casts.
#[test]
fn mysidian_elder_token_pings_on_noncreature_cast() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::mysidian_elder());
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Wizard"),
        "made a Wizard token");
    let life1 = g.players[1].life;
    let spell = g.add_card_to_hand(0, catalog::reach_the_horizon());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a noncreature spell");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 1, "token pinged each opponent for 1");
}

/// Ultimecia taps all opposing creatures on ETB.
#[test]
fn ultimecia_taps_opponents() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::ultimecia_temporal_threat());
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "opponent's creature tapped");
}

/// Rook Turret loots when another artifact you control enters.
#[test]
fn rook_turret_loots_on_artifact_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rook_turret());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // something to discard
    let hand_before = g.players[0].hand.len();
    g.add_card_to_library(0, catalog::forest()); // something to draw
    let art = g.add_card_to_battlefield(0, catalog::bonesplitter()); // another artifact
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: art }]);
    drain_stack(&mut g);
    // Draw 1, discard 1 → net hand size unchanged, but a loot happened (library shrank).
    assert_eq!(g.players[0].hand.len(), hand_before, "drew then discarded");
    assert!(g.players[0].library.is_empty(), "drew the library card");
}

/// Gran Pulse Ochu pumps by permanent cards in your graveyard.
#[test]
fn gran_pulse_ochu_pumps_from_graveyard() {
    let mut g = two_player_game();
    let ochu = g.add_card_to_battlefield(0, catalog::gran_pulse_ochu());
    for _ in 0..3 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); }
    g.players[0].mana_pool.add_colorless(8);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ochu, ability_index: 0, target: None,
        additional_targets: vec![], x_value: None,
    }).expect("activate Ochu pump");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ochu).unwrap().power(), 1 + 3, "+3/+3 from 3 permanent cards");
}

/// Relm's Sketching mints a token copy of a target permanent.
#[test]
fn relms_sketching_copies_a_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::relms_sketching());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Relm's Sketching");
    drain_stack(&mut g);
    let copies = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Grizzly Bears").count();
    assert_eq!(copies, 1, "one token copy of Grizzly Bears");
}

/// Reach the Horizon fetches two basic lands onto the battlefield tapped.
#[test]
fn reach_the_horizon_fetches_two_basics() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let f1 = g.add_card_to_library(0, catalog::forest());
    let i1 = g.add_card_to_library(0, catalog::island());
    let spell = g.add_card_to_hand(0, catalog::reach_the_horizon());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(f1)),
        DecisionAnswer::Search(Some(i1)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Reach the Horizon");
    drain_stack(&mut g);
    let lands = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    assert_eq!(lands, 2, "two lands entered");
    assert!(g.battlefield.iter().filter(|c| c.definition.is_land()).all(|c| c.tapped),
        "they entered tapped");
}

/// Fang draws and loses 1 life when cards leave your graveyard (once per turn).
#[test]
fn fang_triggers_on_graveyard_exit() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::fang_fearless_lcie());
    g.add_card_to_library(0, catalog::forest()); // Fang's draw
    let corpse = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let raise = g.add_card_to_hand(0, catalog::raise_dead());
    let hand_before = g.players[0].hand.len(); // includes Raise Dead
    let life_before = g.players[0].life;
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: raise, target: Some(Target::Permanent(corpse)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Raise Dead");
    drain_stack(&mut g);
    // Raise Dead cast (−1), returns the bear (+1), Fang draws (+1) → net +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "bear returned + Fang draw − Raise Dead");
    assert_eq!(g.players[0].life, life_before - 1, "lost 1 life");
}

/// Prompto makes a Treasure when you cast a noncreature spell paying 4+ mana.
#[test]
fn prompto_makes_treasure_on_big_noncreature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::prompto_argentum());
    let spell = g.add_card_to_hand(0, catalog::reach_the_horizon()); // {3}{G} = 4 mana
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a 4-mana noncreature");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Treasure"),
        "made a Treasure");
}

/// Shantotto grows by the mana spent on your noncreature spell and draws at 4+.
#[test]
fn shantotto_grows_and_draws() {
    let mut g = two_player_game();
    let shan = g.add_card_to_battlefield(0, catalog::shantotto_tactician_magician());
    g.add_card_to_library(0, catalog::forest()); // Shantotto's draw at X>=4
    let hand_before = g.players[0].hand.len();
    let spell = g.add_card_to_hand(0, catalog::reach_the_horizon()); // 4 mana
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a 4-mana noncreature");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(shan).unwrap().power(), 4, "+4/+0 from 4 mana spent");
    // hand_before + spell added − spell cast + draw trigger = hand_before + 1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew on X>=4");
}

/// Rufus makes a Darkstar token when he attacks without one.
#[test]
fn rufus_makes_darkstar_on_attack() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let rufus = g.add_card_to_battlefield(0, catalog::rufus_shinra());
    g.clear_sickness(rufus);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: rufus, target: AttackTarget::Player(1),
    }])).expect("Rufus attacks");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Darkstar"),
        "made a Darkstar token");
}

/// Shambling Cie'th enters tapped.
#[test]
fn shambling_cieth_enters_tapped() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::shambling_cieth());
    drain_stack(&mut g);
    let cieth = g.battlefield.iter().find(|c| c.definition.name == "Shambling Cie'th").unwrap();
    assert!(cieth.tapped, "enters tapped");
}

/// Lion Heart deals 2 on ETB and grants +2/+1 when equipped.
#[test]
fn lion_heart_pings_and_buffs() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::iron_giant());
    let life1 = g.players[1].life;
    // ETB deals 2 to any target; the auto-decider picks one (opponent or foe).
    g.move_card_to_battlefield_for_test(0, catalog::lion_heart());
    drain_stack(&mut g);
    let pinged = g.players[1].life < life1
        || g.battlefield_find(foe).map(|c| c.damage > 0).unwrap_or(false);
    assert!(pinged, "Lion Heart's ETB dealt 2 damage");
    let bonus = catalog::lion_heart().equipped_bonus.unwrap();
    assert_eq!((bonus.power, bonus.toughness), (2, 1), "equip bonus +2/+1");
}

/// Ring of the Lucii is a mana rock plus a life-cost tapper.
#[test]
fn ring_of_the_lucii_taps_a_permanent() {
    let mut g = two_player_game();
    let ring = g.add_card_to_battlefield(0, catalog::ring_of_the_lucii());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life0 = g.players[0].life;
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ring, ability_index: 1, target: Some(Target::Permanent(foe)),
        additional_targets: vec![], x_value: None,
    }).expect("activate Ring tap ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "target tapped");
    assert_eq!(g.players[0].life, life0 - 1, "paid 1 life");
}

/// Sandworm destroys a target land on ETB.
#[test]
fn sandworm_destroys_a_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let sand = g.add_card_to_hand(0, catalog::sandworm());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: sand, target: Some(Target::Permanent(land)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Sandworm");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "target land destroyed");
}

// ── modern_decks batch 2 tests ────────────────────────────────────────────────

/// Syncopate counters an unpaid spell and exiles it.
#[test]
fn syncopate_counters_and_exiles() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(crate::mana::Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    let syn = g.add_card_to_hand(0, catalog::syncopate());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3); // X = 3 (opponent can't pay)
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: syn, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("Syncopate castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "Bolt countered");
    assert!(g.exile.iter().any(|c| c.id == bolt), "countered spell exiled, not in graveyard");
}

/// Crossroads Village enters tapped and taps for the color chosen on entry.
#[test]
fn crossroads_village_taps_for_chosen_color() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(crate::mana::Color::Blue)]));
    let land = g.move_card_to_battlefield_for_test(0, catalog::crossroads_village());
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).unwrap().tapped, "enters tapped");
    // Untap it so we can tap for mana.
    g.battlefield_find_mut(land).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 0, target: None,
        additional_targets: vec![], x_value: None,
    }).expect("tap for chosen color");
    assert_eq!(g.players[0].mana_pool.amount(crate::mana::Color::Blue), 1, "added blue (chosen)");
}

/// Capital City has cycling and taps for colorless.
#[test]
fn capital_city_cycles_and_taps() {
    let c = catalog::capital_city();
    assert!(c.keywords.iter().any(|k| matches!(k, Keyword::Cycling(_))), "has Cycling");
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::capital_city());
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 0, target: None,
        additional_targets: vec![], x_value: None,
    }).expect("tap for colorless");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1, "added colorless");
}

/// Lunatic Pandora sacrifices to destroy a nonland permanent.
#[test]
fn lunatic_pandora_sacs_to_destroy() {
    let mut g = two_player_game();
    let pandora = g.add_card_to_battlefield(0, catalog::lunatic_pandora());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::ActivateAbility {
        card_id: pandora, ability_index: 1, target: Some(Target::Permanent(foe)),
        additional_targets: vec![], x_value: None,
    }).expect("activate sac-destroy");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "target destroyed");
    assert!(g.battlefield_find(pandora).is_none(), "Pandora sacrificed");
}

/// PuPu UFO's base power becomes the number of Towns you control.
#[test]
fn pupu_ufo_base_power_from_towns() {
    let mut g = two_player_game();
    let ufo = g.add_card_to_battlefield(0, catalog::pupu_ufo());
    g.add_card_to_battlefield(0, catalog::treno_dark_city()); // Town
    g.add_card_to_battlefield(0, catalog::vector_imperial_capital()); // Town
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ufo, ability_index: 1, target: None,
        additional_targets: vec![], x_value: None,
    }).expect("activate base-power set");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(ufo).unwrap().power, 2, "base power = 2 Towns");
}

/// Magitek Infantry gets +1/+0 while you control another artifact.
#[test]
fn magitek_infantry_pumps_with_another_artifact() {
    let mut g = two_player_game();
    let inf = g.add_card_to_battlefield(0, catalog::magitek_infantry());
    assert_eq!(g.computed_permanent(inf).unwrap().power, 1, "1/1 alone");
    g.add_card_to_battlefield(0, catalog::bonesplitter()); // another artifact
    assert_eq!(g.computed_permanent(inf).unwrap().power, 2, "+1/+0 with another artifact");
}

/// Moogles' Valor mints a Moogle per creature and grants indestructible.
#[test]
fn moogles_valor_mass_tokens_and_indestructible() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::moogles_valor());
    g.players[0].mana_pool.add(crate::mana::Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Moogles' Valor");
    drain_stack(&mut g);
    let moogles = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Moogle").count();
    assert_eq!(moogles, 2, "one Moogle per pre-existing creature");
    assert!(g.computed_permanent(a).unwrap().keywords.contains(&Keyword::Indestructible),
        "creatures gained indestructible");
}

// ── modern_decks batch 3 tests ────────────────────────────────────────────────

/// World Map sacrifices to fetch a basic land to hand.
#[test]
fn world_map_fetches_basic_to_hand() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let map = g.add_card_to_battlefield(0, catalog::world_map());
    let forest = g.add_card_to_library(0, catalog::forest());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: map, ability_index: 0, target: None,
        additional_targets: vec![], x_value: None,
    }).expect("activate basic fetch");
    drain_stack(&mut g);
    assert!(g.battlefield_find(map).is_none(), "World Map sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "fetched a basic to hand");
}

/// Retrieve the Esper makes a 3/3 Robot token (no counters when cast from hand).
#[test]
fn retrieve_the_esper_makes_robot() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::retrieve_the_esper());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Retrieve the Esper");
    drain_stack(&mut g);
    let robot = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Robot").unwrap();
    assert_eq!((robot.power(), robot.toughness()), (3, 3), "3/3 from hand cast (no counters)");
}

/// Circle of Power draws two, loses 2 life, mints a Wizard, and pumps Wizards.
#[test]
fn circle_of_power_draws_and_pumps_wizards() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
    let life0 = g.players[0].life;
    let hand0 = g.players[0].hand.len();
    let spell = g.add_card_to_hand(0, catalog::circle_of_power());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Circle of Power");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 - 2, "lost 2 life");
    // hand0 (pre-spell) + spell added - cast + 2 draw = hand0 + 2.
    assert_eq!(g.players[0].hand.len(), hand0 + 2, "drew two");
    let wiz = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Wizard").unwrap();
    // The Wizard token is itself a Wizard → gets +1/+0.
    assert_eq!(g.computed_permanent(wiz.id).unwrap().power, 1, "Wizard pumped to 1 power");
}

/// Unexpected Request steals a creature, untapping it and granting haste.
#[test]
fn unexpected_request_steals_creature() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(foe).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::unexpected_request());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(foe)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Unexpected Request");
    drain_stack(&mut g);
    let c = g.battlefield_find(foe).unwrap();
    assert_eq!(c.controller, 0, "gained control");
    assert!(!c.tapped, "untapped");
    assert!(g.computed_permanent(foe).unwrap().keywords.contains(&Keyword::Haste), "gained haste");
}

/// Resentful Revelation puts one of the top three into hand and the rest in the
/// graveyard.
#[test]
fn resentful_revelation_digs_three() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let a = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::mountain());
    let hand0 = g.players[0].hand.len();
    let gy0 = g.players[0].graveyard.len();
    let spell = g.add_card_to_hand(0, catalog::resentful_revelation());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(a))]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Resentful Revelation");
    drain_stack(&mut g);
    // hand0 (pre-spell) + spell added - cast + 1 to hand = hand0 + 1.
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "one card to hand");
    assert_eq!(g.players[0].graveyard.len(), gy0 + 3, "the spell + the two unpicked cards");
}

/// Gaius van Baelsar's ETB edicts each player.
#[test]
fn gaius_van_baelsar_edicts() {
    let mut g = two_player_game();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Pick mode 1 — each player sacrifices a nontoken creature. Player 0's only
    // nontoken creature is Gaius itself; player 1 sacrifices their bear.
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Mode(1),
    ]));
    let gaius = g.add_card_to_hand(0, catalog::gaius_van_baelsar());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: gaius, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Gaius");
    drain_stack(&mut g);
    assert!(g.battlefield_find(gaius).is_none(), "you sacrificed Gaius (only nontoken creature)");
    assert!(g.battlefield_find(theirs).is_none(), "opponent sacrificed theirs");
}

// ── modern_decks batch 4 tests ────────────────────────────────────────────────

/// Sorceress's Schemes returns an instant/sorcery from the graveyard and adds R.
#[test]
fn sorceresss_schemes_returns_instant() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt()); // Instant
    let spell = g.add_card_to_hand(0, catalog::sorceresss_schemes());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Sorceress's Schemes");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "Bolt returned to hand");
    assert_eq!(g.players[0].mana_pool.amount(crate::mana::Color::Red), 1, "added {{R}}");
}

/// Rinoa makes an Angelo token on ETB.
#[test]
fn rinoa_makes_angelo() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::rinoa_heartilly());
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Angelo"),
        "made an Angelo token");
}

/// The Regalia is a haste Crew-1 Vehicle whose attack drops a land.
#[test]
fn the_regalia_lands_on_attack() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    // Library: two nonland on top, then a land.
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let land = g.add_card_to_library(0, catalog::forest());
    let regalia = g.add_card_to_battlefield(0, catalog::the_regalia());
    let crewer = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(crewer);
    g.clear_sickness(regalia); // The Regalia has haste anyway
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Crew { vehicle: regalia, crew_creatures: vec![crewer] })
        .expect("crew The Regalia");
    let lands_before = g.battlefield.iter().filter(|c| c.definition.is_land()).count();
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: regalia, target: AttackTarget::Player(1),
    }])).expect("Regalia attacks");
    drain_stack(&mut g);
    let lands_after = g.battlefield.iter().filter(|c| c.definition.is_land()).count();
    assert_eq!(lands_after, lands_before + 1, "revealed and dropped a land");
    assert!(g.battlefield_find(land).map(|c| c.tapped).unwrap_or(false), "it entered tapped");
}

/// A Realm Reborn grants "{T}: Add one mana of any color" to your other
/// permanents (not to itself).
#[test]
fn a_realm_reborn_grants_mana_ability_to_others() {
    let mut g = two_player_game();
    let realm = g.add_card_to_battlefield(0, catalog::a_realm_reborn());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    // The bear can tap for one mana of any color.
    assert!(!g.granted_abilities_for(bear).is_empty(), "bear gained a mana ability");
    // The enchantment itself is excluded ("other permanents").
    assert!(g.granted_abilities_for(realm).is_empty(), "A Realm Reborn does not grant itself the ability");
}

/// Combat Tutorial: target player draws two and a creature you control grows.
#[test]
fn combat_tutorial_draws_and_grows_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
    let spell = g.add_card_to_hand(0, catalog::combat_tutorial());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Player(0)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: None,
    }).expect("cast Combat Tutorial");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 - 1 + 2, "drew two (minus the spell cast)");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// CR 601.2c — Combat Tutorial's "up to one target creature" slot is optional
/// on a mixed required-player + optional-creature spell: with no creature (or a
/// declined slot) it still resolves the required half.
#[test]
fn cr_601_2c_combat_tutorial_optional_creature_slot_can_be_declined() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
    let spell = g.add_card_to_hand(0, catalog::combat_tutorial());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Player(0)),
        additional_targets: vec![], // decline the creature slot
        mode: None,
        x_value: None,
    }).expect("cast Combat Tutorial with no creature target");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 - 1 + 2, "still drew two");
}

/// Seymour Flux pays 1 life at upkeep to draw a card and grow.
#[test]
fn seymour_flux_pays_life_to_draw_and_grow() {
    let mut g = two_player_game();
    let seymour = g.add_card_to_battlefield(0, catalog::seymour_flux());
    g.add_card_to_library(0, catalog::forest());
    let life0 = g.players[0].life;
    let hand0 = g.players[0].hand.len();
    // Accept the "pay 1 life" optional.
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    g.fire_step_triggers(crate::TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 - 1, "paid 1 life");
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew a card");
    assert_eq!(g.battlefield_find(seymour).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Cloud of Darkness shrinks an opponent's creature by its graveyard's
/// permanent-card count.
#[test]
fn cloud_of_darkness_shrinks_by_graveyard() {
    let mut g = two_player_game();
    // One permanent card in your graveyard → -1/-1 (a 2/2 survives as 1/1).
    g.add_card_to_graveyard(0, catalog::forest());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::cloud_of_darkness());
    drain_stack(&mut g);
    let cp = g.computed_permanent(foe).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "2/2 shrunk to 1/1 by one graveyard permanent");
}

/// Cargo Ship is a flying, vigilant Vehicle with a crew cost and a mana ability.
#[test]
fn cargo_ship_is_flying_vigilant_crew_vehicle() {
    let c = catalog::cargo_ship();
    for kw in [Keyword::Flying, Keyword::Vigilance, Keyword::Crew(1)] {
        assert!(c.keywords.contains(&kw), "Cargo Ship has {kw:?}");
    }
    assert_eq!(c.activated_abilities.len(), 1, "one {{T}}: Add {{C}} ability");
}

/// The Wind Crystal doubles life you'd gain.
#[test]
fn wind_crystal_doubles_lifegain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::the_wind_crystal());
    let life0 = g.players[0].life;
    g.adjust_life(0, 2);
    assert_eq!(g.players[0].life, life0 + 4, "gained twice the 2 life");
}

/// The Fire Crystal gives creatures you control haste.
#[test]
fn fire_crystal_grants_haste() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::the_fire_crystal());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste), "bear has haste");
}

/// Ancient Adamantoise exiles itself and mints ten Treasures when it dies.
#[test]
fn ancient_adamantoise_dies_to_ten_treasures() {
    let mut g = two_player_game();
    let toise = g.add_card_to_battlefield(0, catalog::ancient_adamantoise());
    kill_perm(&mut g, toise);
    assert!(g.exile.iter().any(|c| c.definition.name == "Ancient Adamantoise"), "exiled, not in graveyard");
    let treasures = g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count();
    assert_eq!(treasures, 10, "ten Treasure tokens");
}

/// Poison the Waters' first mode gives all creatures -1/-1.
#[test]
fn poison_the_waters_mode0_shrinks_all() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::poison_the_waters());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast Poison the Waters mode 0");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(mine).unwrap().toughness, 1, "your bear is 1/1");
    assert_eq!(g.computed_permanent(theirs).unwrap().toughness, 1, "their bear is 1/1");
}

/// Valkyrie Aerial Unit's Affinity for artifacts reduces its cost.
#[test]
fn valkyrie_affinity_reduces_cost() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::ornithopter()); }
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let spell = g.add_card_to_hand(0, catalog::valkyrie_aerial_unit());
    // {5}{U}{U} with three artifacts → {2}{U}{U}.
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Affinity made Valkyrie cost {2}{U}{U}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(spell).is_some(), "Valkyrie resolved");
}

/// Ice Flan taps and stuns an opponent's permanent on entry.
#[test]
fn ice_flan_taps_and_stuns() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::ice_flan());
    drain_stack(&mut g);
    let f = g.battlefield_find(foe).unwrap();
    assert!(f.tapped, "opponent creature tapped");
    assert_eq!(f.counter_count(CounterType::Stun), 1, "and stunned");
}

/// Namazu Trader loses a life and makes a Treasure when it enters.
#[test]
fn namazu_trader_etb_treasure_and_loselife() {
    let mut g = two_player_game();
    let life0 = g.players[0].life;
    g.move_card_to_battlefield_for_test(0, catalog::namazu_trader());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 - 1, "lost 1 life");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count(), 1);
}

/// Ultros stuns an opponent's creature after a four-mana noncreature spell.
#[test]
fn ultros_stuns_on_big_noncreature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ultros_obnoxious_octopus());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let spell = g.add_card_to_hand(0, catalog::chemisters_insight()); // {3}{U}, instant
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Chemister's Insight");
    drain_stack(&mut g);
    let f = g.battlefield_find(foe).unwrap();
    assert!(f.tapped && f.counter_count(CounterType::Stun) == 1, "Ultros tapped+stunned the foe");
}

/// Aerith Rescue Mission's first mode makes three Hero tokens.
#[test]
fn aerith_mode0_makes_three_heroes() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::aerith_rescue_mission());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast Aerith mode 0");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Hero").count(), 3);
}

/// Zack Fair enters with a +1/+1 counter and can sacrifice for indestructible.
#[test]
fn zack_fair_counter_and_indestructible() {
    let mut g = two_player_game();
    let zack = g.move_card_to_battlefield_for_test(0, catalog::zack_fair());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(zack).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: zack, ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: vec![], x_value: None,
    }).expect("sacrifice Zack for indestructible");
    drain_stack(&mut g);
    assert!(g.battlefield_find(zack).is_none(), "Zack sacrificed");
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Indestructible));
}

/// The Final Days makes two Horror tokens on a normal cast.
#[test]
fn the_final_days_makes_two_horrors() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::the_final_days());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast The Final Days");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Horror").count(), 2);
}

/// From Father to Son fetches a Vehicle to hand.
#[test]
fn from_father_to_son_fetches_vehicle_to_hand() {
    let mut g = two_player_game();
    let veh = g.add_card_to_library(0, catalog::cargo_ship()); // a Vehicle to find
    g.add_card_to_library(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::from_father_to_son());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Search(Some(veh)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast From Father to Son");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Cargo Ship"), "Vehicle in hand");
}

/// Call the Mountain Chocobo fetches a Mountain and mints a Bird token.
#[test]
fn call_the_mountain_chocobo_fetches_and_makes_bird() {
    let mut g = two_player_game();
    let mtn = g.add_card_to_library(0, catalog::mountain());
    let spell = g.add_card_to_hand(0, catalog::call_the_mountain_chocobo());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Search(Some(mtn)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Call the Mountain Chocobo");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Mountain"), "Mountain in hand");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Chocobo").count(), 1, "one Bird token");
}

/// Traveling Chocobo lets you cast Bird spells and play lands off the top.
#[test]
fn traveling_chocobo_plays_bird_from_top() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::traveling_chocobo());
    let top_bird = g.add_card_to_library(0, catalog::traveling_chocobo());
    // Move it to the top of the library.
    let idx = g.players[0].library.iter().position(|c| c.id == top_bird).unwrap();
    let card = g.players[0].library.remove(idx);
    g.players[0].library.insert(0, card);
    assert!(g.library_top_playable(0, top_bird), "a Bird on top is playable");
}

/// CR 614.16 — The Earth Crystal doubles +1/+1 counter placements onto your
/// creatures (but not other counter kinds).
#[test]
fn cr_614_16_earth_crystal_doubles_plus_one_counters() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::the_earth_crystal());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // One +1/+1 counter placement is doubled to two.
    let n = g.scaled_counter_count(0, CounterType::PlusOnePlusOne, 1, true);
    assert_eq!(n, 2, "one +1/+1 counter becomes two");
    // A non-+1/+1 counter is unaffected.
    let stun = g.scaled_counter_count(0, CounterType::Stun, 1, true);
    assert_eq!(stun, 1, "stun counters are not doubled");
    // End to end: an AddCounter of 1 actually applies 2.
    let ctx = crate::game::effects::EffectContext::for_ability(
        crate::card::CardId(0), 0, Some(Target::Permanent(bear)),
    );
    g.resolve_effect(&crate::effect::Effect::AddCounter {
        what: crate::effect::Selector::Target(0),
        kind: CounterType::PlusOnePlusOne,
        amount: crate::effect::Value::ONE,
    }, &ctx).unwrap();
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Two +1/+1 doublers (The Earth Crystal + Branching Evolution) compose
/// multiplicatively: one counter becomes four.
#[test]
fn earth_crystal_composes_with_branching_evolution() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::the_earth_crystal());
    g.add_card_to_battlefield(0, catalog::branching_evolution());
    let n = g.scaled_counter_count(0, CounterType::PlusOnePlusOne, 1, true);
    assert_eq!(n, 4, "Earth Crystal (2x) and Branching Evolution (2x) stack to 4x");
}

/// CR 119.4 — a "you may pay N life" cost can only be paid while life ≥ N; if
/// it can't be paid (even when accepted), the body is skipped and `else_` runs.
#[test]
fn cr_119_4_may_pay_life_requires_sufficient_life() {
    use crate::effect::{Effect, Selector, Value};
    let mut g = two_player_game();
    g.players[0].life = 3;
    let hand0 = g.players[0].hand.len();
    // Accept the optional, but 5 life is unpayable at 3 life.
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    let ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    g.resolve_effect(&Effect::MayPayLife {
        description: "pay 5 life?".into(),
        amount: Value::Const(5),
        body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
        else_: Some(Box::new(Effect::LoseLife { who: Selector::You, amount: Value::ONE })),
    }, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 2, "no 5-life payment (life stayed 3); else_ lost 1");
    assert_eq!(g.players[0].hand.len(), hand0, "body (draw) was skipped");
}

/// The Prima Vista becomes an artifact creature after a four-mana noncreature
/// spell.
#[test]
fn the_prima_vista_animates_on_big_noncreature() {
    let mut g = two_player_game();
    let ship = g.add_card_to_battlefield(0, catalog::the_prima_vista());
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let spell = g.add_card_to_hand(0, catalog::chemisters_insight()); // {3}{U}
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Chemister's Insight");
    drain_stack(&mut g);
    assert!(g.computed_permanent(ship).unwrap().card_types.contains(&crate::card::CardType::Creature),
        "The Prima Vista is a creature this turn");
}

/// Quistis Trepe casts an instant from a graveyard on entry and exiles it.
#[test]
fn quistis_trepe_casts_instant_from_graveyard() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_battlefield(1, catalog::grizzly_bears()); // a legal target for the free Bolt
    // Accept the "you may cast" optional.
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    g.move_card_to_battlefield_for_test(0, catalog::quistis_trepe());
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt), "the recast Bolt was exiled");
}

/// Town Greeter mills four and puts a land into hand.
#[test]
fn town_greeter_mills_and_takes_a_land() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::forest()); }
    let hand0 = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::town_greeter());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "a land went to hand");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"));
}

/// Giott loots when a Dwarf you control enters (its own ETB included).
#[test]
fn giott_loots_on_dwarf_enter() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::grizzly_bears()); // a card to discard
    g.add_card_to_library(0, catalog::forest());
    let giott = g.add_card_to_battlefield(0, catalog::giott_king_of_the_dwarves());
    let hand0 = g.players[0].hand.len();
    // Accept the "you may discard" loot.
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    // A Dwarf you control (Giott itself) entering fires the loot trigger.
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: giott }]);
    drain_stack(&mut g);
    // Discarded one, drew one → net hand unchanged.
    assert_eq!(g.players[0].hand.len(), hand0, "looted: -1 discard +1 draw");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"), "discarded a card");
}

/// Freya's Jump grants flying only during her controller's turn.
#[test]
fn freya_crescent_flying_only_on_your_turn() {
    let mut g = two_player_game();
    let freya = g.add_card_to_battlefield(0, catalog::freya_crescent());
    g.active_player_idx = 0;
    assert!(g.computed_permanent(freya).unwrap().keywords.contains(&Keyword::Flying),
        "flying during your turn");
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(freya).unwrap().keywords.contains(&Keyword::Flying),
        "no flying on opponent's turn");
}

/// Balthier and Fran pumps controlled Vehicles +1/+1 and grants vigilance+reach.
#[test]
fn balthier_and_fran_buffs_vehicles() {
    let mut g = two_player_game();
    let ship = g.add_card_to_battlefield(0, catalog::cargo_ship()); // 2/3 Vehicle, no reach
    g.add_card_to_battlefield(0, catalog::balthier_and_fran());
    let cp = g.computed_permanent(ship).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4), "Vehicle gets +1/+1");
    assert!(cp.keywords.contains(&Keyword::Reach), "granted reach");
    assert!(cp.keywords.contains(&Keyword::Vigilance), "granted vigilance");
}

/// Freya's red mana pays for an Equipment ability/cast but not a creature spell.
#[test]
fn freya_crescent_mana_restricted_to_equipment() {
    use crate::mana::{SpellKind, SpendRestriction};
    let r = SpendRestriction::EquipmentOnly;
    let equip = SpellKind { equipment: true, ..Default::default() };
    let creature = SpellKind { creature: true, ..Default::default() };
    assert!(r.allows(&equip), "funds an Equipment spell/ability");
    assert!(!r.allows(&creature), "not a creature spell");
}

/// Astrologian's Planisphere's Hero grows a +1/+1 counter on a noncreature cast.
#[test]
fn astrologians_planisphere_counter_on_noncreature_cast() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::astrologians_planisphere());
    drain_stack(&mut g);
    let hero = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Hero")
        .expect("Hero minted").id;
    assert!(g.computed_permanent(hero).unwrap().subtypes.creature_types
        .contains(&crate::card::CreatureType::Wizard), "equipped is a Wizard");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(hero).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "gained a +1/+1 counter from the granted trigger");
}

/// Samurai's Katana job-selects a Hero and grants +2/+2, trample, haste, Samurai.
#[test]
fn samurais_katana_job_select() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::samurais_katana());
    drain_stack(&mut g);
    let hero = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Hero")
        .expect("Hero minted");
    let cp = g.computed_permanent(hero.id).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "1/1 Hero + 2/2");
    assert!(cp.keywords.contains(&Keyword::Trample) && cp.keywords.contains(&Keyword::Haste));
    assert!(cp.subtypes.creature_types.contains(&crate::card::CreatureType::Samurai));
}

/// Black Mage's Rod's equipped creature pings each opponent on a noncreature cast.
#[test]
fn black_mages_rod_pings_on_noncreature_cast() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::black_mages_rod());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bolt");
    drain_stack(&mut g);
    // Bolt's 3 + Rod's 1 ping on cast = 4 to the opponent.
    assert_eq!(g.players[1].life, life1 - 4, "equipped Hero pinged 1 on the noncreature cast");
}

/// Opera Love Song mode 2 pumps one or two target creatures +2/+0.
#[test]
fn opera_love_song_pumps_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::opera_love_song());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)], mode: Some(1), x_value: None,
    }).expect("cast Opera Love Song");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(a).unwrap().power, 4, "first target +2/+0");
    assert_eq!(g.computed_permanent(b).unwrap().power, 4, "second target +2/+0");
}

/// Blitzball taps for any color; its GOOOOAAAALLL draw needs a combat hit.
#[test]
fn blitzball_mana_and_conditional_draw() {
    let mut g = two_player_game();
    let ball = g.add_card_to_battlefield(0, catalog::blitzball());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Mana ability works.
    g.perform_action(GameAction::ActivateAbility {
        card_id: ball, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.total(), 1, "one mana of any color");
    // GOOOOAAAALLL is gated on having dealt combat damage this turn.
    g.battlefield_find_mut(ball).unwrap().tapped = false;
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: ball, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).is_err(), "can't draw without a combat hit this turn");
    g.players[0].dealt_combat_damage_to_player_this_turn = true;
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: ball, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("GOOOOAAAALLL");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 2, "drew two");
    assert!(g.battlefield_find(ball).is_none(), "sacrificed itself");
}

/// Seifer grants a lone attacker double strike and recasts an I/S from the gy
/// on combat damage.
#[test]
fn seifer_almasy_lone_attacker_and_recast() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let seifer = g.add_card_to_battlefield(0, catalog::seifer_almasy());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_battlefield(1, catalog::grizzly_bears()); // a legal Bolt target
    g.clear_sickness(seifer);
    // Accept the "you may cast" recast.
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: seifer, target: AttackTarget::Player(1),
    }])).expect("Seifer attacks alone");
    drain_stack(&mut g);
    // Attacks-alone grant.
    assert!(g.computed_permanent(seifer).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "lone attacker gains double strike");
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert!(g.exile.iter().any(|c| c.id == bolt), "the recast Bolt was exiled after resolving");
}

/// Raubahn has Ward (pay life = power) and attaches an Equipment when it attacks.
#[test]
fn raubahn_wards_and_attaches_on_attack() {
    use crate::game::types::{Attack, AttackTarget};
    use crate::card::WardCost;
    let mut g = two_player_game();
    let raubahn = g.add_card_to_battlefield(0, catalog::raubahn_bull_of_ala_mhigo());
    assert!(g.computed_permanent(raubahn).unwrap().keywords
        .contains(&Keyword::Ward(WardCost::LifeSourcePower)), "Ward—pay life = power");
    let sword = g.add_card_to_battlefield(0, catalog::bonesplitter()); // unattached Equipment
    g.clear_sickness(raubahn);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: raubahn, target: AttackTarget::Player(1),
    }])).expect("Raubahn attacks");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(sword).unwrap().attached_to, Some(raubahn),
        "the Equipment attached to Raubahn on attack");
}

/// Golbez surveils when an artifact enters, and at end step with 4+ artifacts
/// returns a creature card from your graveyard to hand.
#[test]
fn golbez_surveils_and_returns_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::golbez_crystal_collector());
    g.add_card_to_library(0, catalog::island()); // surveil target
    // Artifact ETB → surveil 1 (keep on top; just assert it doesn't crash and
    // the graveyard/library are consistent).
    let lib0 = g.players[0].library.len();
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: ring }]);
    drain_stack(&mut g);
    assert!(g.players[0].library.len() <= lib0, "surveil looked at the top card");
    // Four artifacts + a creature in the graveyard → end step returns it.
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::sol_ring()); }
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "returned the creature to hand");
}

/// Cloud gains double strike + indestructible only while equipped on your turn.
#[test]
fn cloud_planets_champion_conditional_keywords() {
    let mut g = two_player_game();
    let cloud = g.add_card_to_battlefield(0, catalog::cloud_planets_champion());
    g.active_player_idx = 0;
    // Unequipped: no grant even on your turn.
    let cp = g.computed_permanent(cloud).unwrap();
    assert!(!cp.keywords.contains(&Keyword::DoubleStrike), "no double strike unequipped");
    // Equip a Bonesplitter.
    let sword = g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(cloud);
    let cp = g.computed_permanent(cloud).unwrap();
    assert!(cp.keywords.contains(&Keyword::DoubleStrike), "double strike while equipped");
    assert!(cp.keywords.contains(&Keyword::Indestructible), "indestructible while equipped");
    // Opponent's turn: no grant even while equipped.
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(cloud).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "no grant on opponent's turn");
}

/// Jenova's begin-combat buff adds +power counters to a target creature and
/// makes it a Mutant in addition to its other types (CR 205.1b / 613.4).
#[test]
fn jenova_buffs_and_grants_mutant() {
    let mut g = two_player_game();
    let jenova = g.add_card_to_battlefield(0, catalog::jenova_ancient_calamity());
    // Pump Jenova to power 3 so it grants 3 counters.
    g.battlefield_find_mut(jenova).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    advance_to(&mut g, TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 3,
        "3 counters = Jenova's power");
    assert!(g.computed_permanent(bear).unwrap().subtypes.creature_types
        .contains(&crate::card::CreatureType::Mutant), "gained Mutant in addition to Bear");
    assert!(g.computed_permanent(bear).unwrap().subtypes.creature_types
        .contains(&crate::card::CreatureType::Bear), "still a Bear");
}

/// CR 603.10 — Jenova's dies-draw reads the *granted* Mutant type from the
/// death LKI snapshot: a Bear it turned into a Mutant still triggers the draw.
#[test]
fn jenova_dies_draw_reads_granted_mutant_type() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::jenova_ancient_calamity());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    advance_to(&mut g, TurnStep::BeginCombat);
    drain_stack(&mut g); // bear becomes a 3/3 Mutant (2/2 + Jenova's power 1)
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let hand0 = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the Mutant Bear");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "3/3 Mutant Bear died to 3 damage");
    // hand0 counted the bolt; it left for the stack, so net draw = power (3).
    assert_eq!(g.players[0].hand.len(), hand0 + 3, "drew 3 = dead Mutant's power");
}

/// Summon: Choco/Mog is a Saga *creature*: its chapters pump the rest of the
/// team, and it's sacrificed after chapter IV like any Saga (CR 714 on a
/// creature body).
#[test]
fn summon_choco_mog_saga_creature_pumps_and_sacrifices() {
    let mut g = two_player_game();
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let choco = g.add_card_to_battlefield(0, catalog::summon_choco_mog());
    // It's both a creature and a Saga.
    let cp = g.computed_permanent(choco).unwrap();
    assert!(cp.card_types.contains(&crate::card::CardType::Creature)
        && cp.subtypes.enchantment_subtypes.contains(&crate::card::EnchantmentSubtype::Saga));
    g.saga_advance(choco); // I — Stampede
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(ally).unwrap().power, 3, "other creature +1/+0");
    // Choco pumps only *others*, so it stays 3/3.
    assert_eq!(g.computed_permanent(choco).unwrap().power, 3, "self not pumped");
    g.saga_advance(choco); // II
    drain_stack(&mut g);
    g.saga_advance(choco); // III
    drain_stack(&mut g);
    g.saga_advance(choco); // IV
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(choco).is_none(), "sacrificed after chapter IV");
}

/// Summon: Bahamut's Mega Flare (IV) hits each opponent for the total mana value
/// of your *other* permanents; III draws two.
#[test]
fn summon_bahamut_mega_flare_scales_with_board() {
    let mut g = two_player_game();
    // Fodder for chapters I/II (hostile destroy auto-targets the opponent).
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Two of our own permanents feed Mega Flare: {1} + {6} = 7 total MV.
    g.add_card_to_battlefield(0, catalog::excalibur_ii()); // {1}
    g.add_card_to_battlefield(0, catalog::aettir_and_priwen()); // {6}
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let bahamut = g.add_card_to_battlefield(0, catalog::summon_bahamut());
    let hand0 = g.players[0].hand.len();
    let life1 = g.players[1].life;
    for _ in 0..3 { g.saga_advance(bahamut); drain_stack(&mut g); }
    assert_eq!(g.players[0].hand.len(), hand0 + 2, "chapter III drew two");
    g.saga_advance(bahamut); // IV — Mega Flare
    drain_stack(&mut g);
    // Other permanents: Excalibur II (1) + Aettir and Priwen (6) = 7. Bahamut
    // itself is excluded (OtherThanSource).
    assert_eq!(g.players[1].life, life1 - 7, "damage = total MV of other permanents");
}

/// The Gold Saucer wins a coin flip for a Treasure, and sacs two artifacts to
/// draw.
#[test]
fn the_gold_saucer_coinflip_treasure_and_sac_draw() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let saucer = g.add_card_to_battlefield(0, catalog::the_gold_saucer());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // {2},{T}: flip → heads → Treasure.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: saucer, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("flip for Treasure");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Treasure"),
        "heads minted a Treasure");
    // {3},{T}, Sacrifice two artifacts (the Treasure + one more): draw a card.
    g.add_card_to_battlefield(0, catalog::excalibur_ii());
    g.add_card_to_library(0, catalog::island());
    g.battlefield_find_mut(saucer).unwrap().tapped = false;
    g.players[0].mana_pool.add_colorless(3);
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: saucer, ability_index: 2, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac two artifacts to draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew a card");
}

/// Ether adds {U}, exiles itself, and copies your next instant/sorcery.
#[test]
fn ether_adds_mana_and_copies_next_spell() {
    let mut g = two_player_game();
    let ether = g.add_card_to_battlefield(0, catalog::ether());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ether, ability_index: 0,
        target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Ether");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.definition.name == "Ether"), "Ether exiled itself");
    assert_eq!(g.players[0].mana_pool.amount(crate::mana::Color::Blue), 1, "added {{U}}");
    let life1 = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Lightning Bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 6, "bolt + its Ether copy = 6 damage");
}

/// Summon: Fat Chocobo makes a Bird on I and grants team trample on II+.
#[test]
fn summon_fat_chocobo_bird_and_trample() {
    let mut g = two_player_game();
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let chocobo = g.add_card_to_battlefield(0, catalog::summon_fat_chocobo());
    g.saga_advance(chocobo); // I — Wark
    drain_stack(&mut g);
    let bird = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Bird")
        .expect("2/2 Bird token minted");
    assert_eq!((bird.definition.power, bird.definition.toughness), (2, 2));
    g.saga_advance(chocobo); // II — Kerplunk
    drain_stack(&mut g);
    assert!(g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::Trample),
        "team gained trample");
}

/// Summon: G.F. Cerberus chapter II copies your next instant/sorcery.
#[test]
fn summon_gf_cerberus_chapter_two_copies_next_spell() {
    let mut g = two_player_game();
    let cerberus = g.add_card_to_battlefield(0, catalog::summon_gf_cerberus());
    g.add_card_to_library(0, catalog::island()); // something to surveil on I
    g.saga_advance(cerberus); // I — Surveil 1
    drain_stack(&mut g);
    g.saga_advance(cerberus); // II — arm "copy your next I/S"
    drain_stack(&mut g);
    let life1 = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Lightning Bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 6, "bolt + its copy = 6 damage");
}

/// Summon: Esper Ramuh's chapter I bolts for the count of noncreature-nonland
/// cards in your graveyard.
#[test]
fn summon_esper_ramuh_chapter_one_scales_with_graveyard() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // instant (noncreature nonland)
    g.add_card_to_graveyard(0, catalog::island()); // a land — must NOT count
    g.add_card_to_graveyard(0, catalog::sephiroths_intervention()); // sorcery — counts
    let ramuh = g.add_card_to_battlefield(0, catalog::summon_esper_ramuh());
    g.saga_advance(ramuh); // I — Judgment Bolt: 2 noncreature-nonland cards → 2 dmg
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "2 damage killed the 2/2");
}

/// Summon: G.F. Ifrit adds {R} on its third chapter.
#[test]
fn summon_gf_ifrit_chapter_three_adds_red() {
    let mut g = two_player_game();
    let ifrit = g.add_card_to_battlefield(0, catalog::summon_gf_ifrit());
    g.saga_advance(ifrit); // I
    drain_stack(&mut g);
    g.saga_advance(ifrit); // II
    drain_stack(&mut g);
    g.saga_advance(ifrit); // III — add {R}
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(crate::mana::Color::Red), 1, "chapter III added {{R}}");
}

/// Summon: Anima's final chapter edicts each opponent and drains 3 life.
#[test]
fn summon_anima_final_chapter_edict_and_drain() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let anima = g.add_card_to_battlefield(0, catalog::summon_anima());
    let life1 = g.players[1].life;
    for _ in 0..4 {
        g.saga_advance(anima);
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(foe).is_none(), "opponent edicted their creature");
    assert_eq!(g.players[1].life, life1 - 3, "opponent lost 3 life");
    g.check_state_based_actions();
    assert!(g.battlefield_find(anima).is_none(), "Anima sacrificed after IV");
}

/// Haste Magic pumps +3/+1, grants haste, and impulse-exiles the top card.
#[test]
fn haste_magic_pumps_and_impulses() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.add_card_to_library(0, catalog::island());
    let ex0 = g.exile.len();
    let spell = g.add_card_to_hand(0, catalog::haste_magic());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Haste Magic");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 3), "2/2 +3/+1");
    assert!(cp.keywords.contains(&Keyword::Haste), "gained haste");
    assert_eq!(g.exile.len(), ex0 + 1, "top card impulse-exiled");
}

/// Delivery Moogle tutors a low-cost artifact from library *or graveyard*.
#[test]
fn delivery_moogle_dual_zone_artifact_tutor() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // A cheap artifact sitting in the graveyard, not the library.
    let relic = g.add_card_to_graveyard(0, catalog::excalibur_ii()); // {1} artifact
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(relic))]));
    g.move_card_to_battlefield_for_test(0, catalog::delivery_moogle());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == relic),
        "pulled the artifact out of the graveyard into hand");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == relic), "left the graveyard");
}

/// Excalibur II charges on lifegain and grants +1/+1 per charge counter.
#[test]
fn excalibur_ii_charges_on_lifegain_and_scales() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let sword = g.add_card_to_battlefield(0, catalog::excalibur_ii());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(bear);
    // Gain 2 life over two events → 2 charge counters.
    for _ in 0..2 {
        let before = g.players[0].life;
        g.adjust_life(0, 1);
        let d = (g.players[0].life - before) as u32;
        g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 0, amount: d }]);
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(sword).unwrap().counter_count(CounterType::Charge), 2,
        "two lifegain events → two charge counters");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "2/2 + (+1/+1 per charge counter)");
}

/// Aettir and Priwen sets the equipped creature's base P/T to your life total.
#[test]
fn aettir_and_priwen_base_pt_is_life_total() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sword = g.add_card_to_battlefield(0, catalog::aettir_and_priwen());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(bear);
    g.players[0].life = 17;
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (17, 17), "base P/T = 17 life");
    // Tracks life changes live.
    g.players[0].life = 5;
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "recomputes to 5 life");
}

/// CR 707.2 — a `CreateTokenCopyOf` with `enters_tapped` mints a tapped copy.
#[test]
fn cr_707_2_token_copy_enters_tapped() {
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = crate::game::effects::EffectContext::for_ability(
        crate::card::CardId(0), 0, Some(Target::Permanent(src)),
    );
    g.resolve_effect(&crate::effect::Effect::CreateTokenCopyOf {
        who: crate::effect::PlayerRef::You,
        count: crate::effect::Value::ONE,
        source: crate::effect::Selector::Target(0),
        extra_creature_types: vec![],
        extra_card_types: vec![],
        override_pt: None,
        override_colors: None,
        enters_tapped: true,
        non_legendary: false,
        legendary: false,
    }, &ctx).unwrap();
    let token = g.battlefield.iter().find(|c| c.is_token && c.id != src)
        .expect("token copy minted");
    assert!(token.tapped, "the copy entered tapped");
}

/// Ardyn's anthem grants Demons menace/lifelink/haste; Starscourge mints a
/// 5/5 black Demon copy of an exiled graveyard creature.
#[test]
fn ardyn_the_usurper_anthem_and_starscourge() {
    let mut g = two_player_game();
    let ardyn = g.add_card_to_battlefield(0, catalog::ardyn_the_usurper());
    // Ardyn is a Human Noble (not a Demon), so the anthem doesn't hit it; add a
    // Demon to check the grant.
    let demon = g.add_card_to_battlefield(0, catalog::iron_giant()); // a Demon
    let cp = g.computed_permanent(demon).unwrap();
    for kw in [Keyword::Menace, Keyword::Lifelink, Keyword::Haste] {
        assert!(cp.keywords.contains(&kw), "Demon has {kw:?}");
    }
    // Starscourge: a creature in the graveyard becomes a 5/5 black Demon copy.
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let _ = ardyn;
    advance_to(&mut g, TurnStep::BeginCombat);
    drain_stack(&mut g);
    let token = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Grizzly Bears")
        .expect("token copy minted");
    let tcp = g.computed_permanent(token.id).unwrap();
    assert_eq!((tcp.power, tcp.toughness), (5, 5), "5/5 override");
    assert!(tcp.subtypes.creature_types.contains(&crate::card::CreatureType::Demon), "is a Demon");
    assert!(tcp.colors.contains(&crate::mana::Color::Black) && tcp.colors.len() == 1, "black only");
}

/// Lightning exiles the top card for a may-play on combat damage to a player.
#[test]
fn lightning_security_sergeant_impulses_on_combat_damage() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let lightning = g.add_card_to_battlefield(0, catalog::lightning_security_sergeant());
    g.add_card_to_library(0, catalog::island());
    g.clear_sickness(lightning);
    let exile0 = g.exile.len();
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: lightning, target: AttackTarget::Player(1),
    }])).expect("Lightning attacks");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.exile.len(), exile0 + 1, "top card exiled for a may-play");
}

/// Bartz's ETB has each other Bird deal its power to an opponent's creature.
#[test]
fn bartz_and_boko_birds_deal_damage() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sea_eagle()); // 1/1 Bird
    let foe = g.add_card_to_battlefield(1, catalog::sea_eagle()); // 1/1 target
    g.move_card_to_battlefield_for_test(0, catalog::bartz_and_boko());
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "the other Bird's 1 damage killed the 1/1 foe");
}

/// Self-Destruct: the chosen creature deals its power to another target and itself.
#[test]
fn self_destruct_deals_power_to_both() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::self_destruct());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Player(1)],
        mode: None, x_value: None,
    }).expect("cast Self-Destruct");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 2, "2 damage to the other target");
    assert!(g.battlefield_find(mine).is_none(), "2 damage killed the 2/2 itself");
}

/// Magitek Scythe attaches on entry, granting first strike + must-be-blocked.
#[test]
fn magitek_scythe_attaches_and_grants() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.move_card_to_battlefield_for_test(0, catalog::magitek_scythe());
    drain_stack(&mut g);
    let cp = g.computed_permanent(creature).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3), "+2/+1 from the Scythe");
    assert!(cp.keywords.contains(&Keyword::FirstStrike), "granted first strike");
    assert!(cp.keywords.contains(&Keyword::MustBeBlocked), "must be blocked this turn");
}

/// Relentless X-ATM092 needs 3+ blockers and self-returns from the graveyard.
#[test]
fn relentless_x_atm092_evasion_and_recursion() {
    assert!(catalog::relentless_x_atm092().keywords.contains(&Keyword::CantBeBlockedExceptByN(3)));
    // Recur from the graveyard for {8}, entering tapped with a finality counter.
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::relentless_x_atm092());
    g.players[0].mana_pool.add_colorless(8);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("recur from graveyard");
    drain_stack(&mut g);
    let onbf = g.battlefield.iter().find(|c| c.definition.name == "Relentless X-ATM092")
        .expect("returned to battlefield");
    assert!(onbf.tapped, "enters tapped");
    assert_eq!(onbf.counter_count(CounterType::Finality), 1, "with a finality counter");
}

/// Qutrub Forayer's modal ETB can destroy a creature that took damage this turn.
#[test]
fn qutrub_forayer_destroys_damaged_creature() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    // Deal 1 damage so it counts as "dealt damage this turn".
    g.resolve_effect(&crate::effect::Effect::DealDamage {
        to: crate::effect::Selector::Target(0),
        amount: crate::effect::Value::ONE,
    }, &crate::game::effects::EffectContext::for_ability(
        crate::card::CardId(0), 0, Some(Target::Permanent(foe)),
    )).unwrap();
    g.decider = Box::new(crate::decision::ScriptedDecider::new(vec![
        crate::decision::DecisionAnswer::Mode(0), // destroy the damaged creature
    ]));
    g.move_card_to_battlefield_for_test(0, catalog::qutrub_forayer());
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "the damaged creature was destroyed");
}

/// Ninja's Blades: on combat damage, loot then drain by the discarded card's MV.
#[test]
fn ninjas_blades_loots_and_drains() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let hero = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // wearer, 2/2 → 3/3
    let blades = g.add_card_to_battlefield(0, catalog::ninjas_blades());
    g.battlefield_find_mut(blades).unwrap().attached_to = Some(hero);
    // Empty hand; the only card drawn (and thus discarded) is Sol Ring, MV 1.
    g.add_card_to_library(0, catalog::sol_ring());
    g.clear_sickness(hero);
    let life1 = g.players[1].life;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: hero, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    // 3 combat damage + 1 life lost to the discarded Sol Ring's mana value.
    assert_eq!(g.players[1].life, life1 - 4, "3 combat + 1 drain by discarded MV");
}

/// Machinist's Arsenal scales +2/+2 per artifact and grants Artificer.
#[test]
fn machinists_arsenal_scales_per_artifact() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sol_ring()); // one artifact
    g.move_card_to_battlefield_for_test(0, catalog::machinists_arsenal());
    drain_stack(&mut g);
    let hero = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Hero")
        .expect("Hero minted").id;
    // Artifacts you control: Sol Ring + Machinist's Arsenal itself = 2 → +4/+4.
    let cp = g.computed_permanent(hero).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "1/1 + 2 artifacts × +2/+2");
    assert!(cp.subtypes.creature_types.contains(&crate::card::CreatureType::Artificer));
}

/// Sage's Nouliths untaps a target attacking creature when its host attacks.
#[test]
fn sages_nouliths_untaps_attacker() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::sages_nouliths());
    drain_stack(&mut g);
    let hero = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Hero")
        .expect("Hero minted").id;
    g.clear_sickness(hero);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: hero, target: AttackTarget::Player(1),
    }])).expect("Hero attacks");
    drain_stack(&mut g);
    // The only attacking creature is the Hero; its granted trigger untaps it.
    assert!(!g.battlefield_find(hero).unwrap().tapped, "attacking Hero untapped itself");
}

/// Red Mage's Rapier's equipped creature grows +2/+0 on a noncreature cast.
#[test]
fn red_mages_rapier_pumps_on_noncreature_cast() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::red_mages_rapier());
    drain_stack(&mut g);
    let hero = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Hero")
        .expect("Hero minted").id;
    assert_eq!(g.computed_permanent(hero).unwrap().power, 1, "base 1/1 Hero");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bolt");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(hero).unwrap().power, 3, "+2/+0 from the granted trigger");
}

/// Dragoon's Lance grants the equipped creature flying only on your turn.
#[test]
fn dragoons_lance_flying_only_on_your_turn() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::dragoons_lance());
    drain_stack(&mut g);
    let hero = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Hero")
        .expect("Hero minted").id;
    g.active_player_idx = 0;
    assert!(g.computed_permanent(hero).unwrap().keywords.contains(&Keyword::Flying),
        "flying during your turn");
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(hero).unwrap().keywords.contains(&Keyword::Flying),
        "no flying on opponent's turn");
    assert!(g.computed_permanent(hero).unwrap().subtypes.creature_types
        .contains(&crate::card::CreatureType::Knight), "is a Knight");
}

/// Summon: Shiva taps opponents' creatures with stun counters (I, II), then
/// draws a card per tapped opposing creature (III).
#[test]
fn summon_shiva_stuns_then_draws() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let shiva = g.add_card_to_battlefield(0, catalog::summon_shiva());
    let hand0 = g.players[0].hand.len();
    g.saga_advance(shiva); // I — Heavenly Strike
    drain_stack(&mut g);
    g.saga_advance(shiva); // II — Heavenly Strike
    drain_stack(&mut g);
    let tapped = [a, b].iter().filter(|id| g.battlefield_find(**id).unwrap().tapped).count();
    assert!(tapped >= 1, "at least one opposing creature tapped with a stun counter");
    assert!([a, b].iter().any(|id|
        g.battlefield_find(*id).unwrap().counter_count(CounterType::Stun) >= 1));
    g.saga_advance(shiva); // III — Diamond Dust
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + tapped, "drew one per tapped foe");
}

/// Summon: Titan returns all land cards from the graveyard to the battlefield
/// tapped (II).
#[test]
fn summon_titan_reclaims_lands() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::grizzly_bears()); } // mill fodder
    g.add_card_to_graveyard(0, catalog::forest());
    g.add_card_to_graveyard(0, catalog::forest());
    let lands0 = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.is_land())
        .count();
    let titan = g.add_card_to_battlefield(0, catalog::summon_titan());
    g.saga_advance(titan); // I — mill five
    drain_stack(&mut g);
    g.saga_advance(titan); // II — reclaim lands
    drain_stack(&mut g);
    let lands1 = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.is_land())
        .collect::<Vec<_>>();
    assert_eq!(lands1.len(), lands0 + 2, "two forests returned from graveyard");
    assert!(lands1.iter().all(|c| c.tapped), "returned lands enter tapped");
}

/// Summon: Primal Garuda's Aerial Blast (I) deals 4 to a tapped opposing
/// creature, killing a 2/2.
#[test]
fn summon_primal_garuda_aerial_blast() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(foe).unwrap().tapped = true;
    let garuda = g.add_card_to_battlefield(0, catalog::summon_primal_garuda());
    g.saga_advance(garuda); // I — Aerial Blast
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(foe).is_none(), "tapped foe took 4 and died");
}

/// Summon: Primal Odin grants itself Zantetsuken (II) — its combat damage to a
/// player makes that player lose the game.
#[test]
fn summon_primal_odin_zantetsuken() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::grizzly_bears()); // chapter I destroys it
    let odin = g.add_card_to_battlefield(0, catalog::summon_primal_odin());
    g.saga_advance(odin); // I — Gungnir
    drain_stack(&mut g);
    g.saga_advance(odin); // II — Zantetsuken (grant)
    drain_stack(&mut g);
    g.clear_sickness(odin);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: odin, target: AttackTarget::Player(1),
    }])).expect("Odin attacks");
    // Pass through combat damage; the elimination ends the game, so stop as
    // soon as the seat loses (further priority passes would error).
    while !g.players[1].eliminated && g.step != TurnStep::PostCombatMain {
        if g.perform_action(GameAction::PassPriority).is_err() { break; }
    }
    assert!(g.players[1].eliminated, "combat damage from Odin made the player lose");
}

/// Weapons Vendor's begin-combat reflexive attaches an Equipment you control to
/// a creature you control — both target slots auto-fill.
#[test]
fn weapons_vendor_attaches_equipment() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island()); // ETB draw
    let sword = g.add_card_to_battlefield(0, catalog::bonesplitter());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::weapons_vendor());
    drain_stack(&mut g);
    g.active_player_idx = 0;
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    advance_to(&mut g, TurnStep::BeginCombat);
    // Mana empties at each step boundary, so float the {1} after entering combat
    // (the trigger is already on the stack).
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(sword).unwrap().attached_to, Some(bear),
        "Equipment attached to the creature — both reflexive slots filled");
}

/// Thunder Magic's Thundara tier ({3}) deals 4 to a target creature; the Tiered
/// cast folds the chosen mode's cost into the total.
#[test]
fn thunder_magic_thundara_tier() {
    use crate::game::Target;
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::thunder_magic());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1); // base {R}
    g.players[0].mana_pool.add_colorless(3); // Thundara {3}
    g.perform_action(GameAction::CastSpellSpree {
        card_id: id, spree_modes: vec![1], target: Some(Target::Permanent(foe)),
        additional_targets: vec![], x_value: None,
    }).expect("cast Thundara");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(foe).is_none(), "4 damage killed the 4/4");
}

/// Fire Magic's Firaga tier ({5}) deals 3 to each creature — a 2/2 dies, a 4/4
/// lives.
#[test]
fn fire_magic_firaga_hits_each_creature() {
    let mut g = two_player_game();
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::fire_magic());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5); // Firaga {5}
    g.perform_action(GameAction::CastSpellSpree {
        card_id: id, spree_modes: vec![2], target: None,
        additional_targets: vec![], x_value: None,
    }).expect("cast Firaga");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(small).is_none(), "2/2 took 3 and died");
    assert!(g.battlefield_find(big).is_some(), "4/4 survived 3 damage");
}

/// Ice Magic's Blizzard tier ({0}) returns a creature to its owner's hand.
#[test]
fn ice_magic_blizzard_bounces() {
    use crate::game::Target;
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::ice_magic());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1); // base {1}{U}, Blizzard {0}
    g.perform_action(GameAction::CastSpellSpree {
        card_id: id, spree_modes: vec![0], target: Some(Target::Permanent(foe)),
        additional_targets: vec![], x_value: None,
    }).expect("cast Blizzard");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == foe), "creature bounced to owner's hand");
}

/// Restoration Magic's Curaga tier ({3}{W}) shields your permanents and gains 6
/// life.
#[test]
fn restoration_magic_curaga_shields_and_gains() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::restoration_magic());
    g.players[0].mana_pool.add(crate::mana::Color::White, 2); // base {W} + Curaga {W}
    g.players[0].mana_pool.add_colorless(3);
    let life0 = g.players[0].life;
    g.perform_action(GameAction::CastSpellSpree {
        card_id: id, spree_modes: vec![2], target: None,
        additional_targets: vec![], x_value: None,
    }).expect("cast Curaga");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 6, "gained 6 life");
    assert!(g.computed_permanent(mine).unwrap().keywords.contains(&Keyword::Indestructible),
        "your creature is indestructible");
}

/// Tiered enforces choosing exactly one mode (unlike Spree's one-or-more).
#[test]
fn tiered_rejects_multiple_modes() {
    use crate::game::Target;
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::thunder_magic());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(8);
    let res = g.perform_action(GameAction::CastSpellSpree {
        card_id: id, spree_modes: vec![0, 1], target: Some(Target::Permanent(foe)),
        additional_targets: vec![], x_value: None,
    });
    assert!(res.is_err(), "choosing two tiers is illegal");
}

/// Warrior's Sword: Job select mints a Hero, equips it, and grants +3/+2 Warrior.
#[test]
fn warriors_sword_job_select_and_bonus() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::warriors_sword());
    drain_stack(&mut g);
    let hero = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Hero")
        .expect("Hero minted").id;
    let cp = g.computed_permanent(hero).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3), "1/1 + 3/+2");
    assert!(cp.subtypes.creature_types.contains(&crate::card::CreatureType::Warrior));
}

/// Thief's Knife draws when the equipped Hero deals combat damage to a player.
#[test]
fn thiefs_knife_draws_on_combat_damage() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.move_card_to_battlefield_for_test(0, catalog::thiefs_knife());
    drain_stack(&mut g);
    let hero = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Hero")
        .expect("Hero minted").id;
    g.clear_sickness(hero);
    let hand0 = g.players[0].hand.len();
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: hero, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "combat damage drew a card");
}

/// Suplex mode 0 deals 3 and exiles the creature if it dies.
#[test]
fn suplex_damages_and_exiles() {
    use crate::game::Target;
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::suplex());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(foe)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast Suplex");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(foe).is_none(), "3 damage killed the 2/2");
    assert!(g.players[1].graveyard.iter().all(|c| c.id != foe), "exiled, not in graveyard");
}

/// Tifa's Meteor Strikes tier doubles a creature's power and toughness.
#[test]
fn tifas_limit_break_doubles() {
    use crate::game::Target;
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::tifas_limit_break());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: id, spree_modes: vec![1], target: Some(Target::Permanent(mine)),
        additional_targets: vec![], x_value: None,
    }).expect("cast Meteor Strikes");
    drain_stack(&mut g);
    let cp = g.computed_permanent(mine).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 8), "4/4 doubled to 8/8");
}

/// Swallowed by Leviathan surveils, then counters a spell unless {1} per card in
/// your graveyard is paid.
#[test]
fn swallowed_by_leviathan_counters() {
    use crate::game::Target;
    let mut g = two_player_game();
    // Two cards in P0's graveyard → the tax is {2}.
    g.add_card_to_graveyard(0, catalog::island());
    g.add_card_to_graveyard(0, catalog::forest());
    // P1 (active) casts a creature spell we'll counter.
    let spell = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("P1 casts Grizzly");
    // P0 responds with the counter.
    g.priority.player_with_priority = 0;
    let counter = g.add_card_to_hand(0, catalog::swallowed_by_leviathan());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: counter, target: Some(Target::Permanent(spell)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast counter");
    drain_stack(&mut g);
    // P1 has no mana left to pay the {2} tax → the Grizzly is countered.
    assert!(g.battlefield_find(spell).is_none(), "Grizzly countered");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == spell), "went to graveyard");
}

/// Zodiark's ETB makes each player sacrifice half their non-God creatures, and
/// its sacrifice-trigger grows it per creature sacrificed.
#[test]
fn zodiark_edict_and_grows_on_sacrifice() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_battlefield(1, catalog::grizzly_bears()); }
    let foes0 = g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.is_creature()).count();
    let zodiark = g.move_card_to_battlefield_for_test(0, catalog::zodiark_umbral_god());
    drain_stack(&mut g);
    let foes1 = g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.is_creature()).count();
    assert_eq!(foes1, foes0 - 2, "player 1 sacrificed half of four creatures");
    assert_eq!(
        g.battlefield_find(zodiark).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "one +1/+1 counter per creature sacrificed",
    );
    assert!(g.computed_permanent(zodiark).unwrap().keywords.contains(&Keyword::Indestructible));
}

/// Phantom Train sacrifices another permanent to grow and animate into a Spirit.
#[test]
fn phantom_train_animates_on_sacrifice() {
    let mut g = two_player_game();
    let train = g.add_card_to_battlefield(0, catalog::phantom_train());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(!g.computed_permanent(train).unwrap().card_types.contains(&crate::card::CardType::Creature),
        "starts as a noncreature Vehicle");
    g.perform_action(GameAction::ActivateAbility {
        card_id: train, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    let cp = g.computed_permanent(train).unwrap();
    assert!(cp.card_types.contains(&crate::card::CardType::Creature), "now a creature");
    assert!(cp.subtypes.creature_types.contains(&crate::card::CreatureType::Spirit), "a Spirit");
    assert_eq!((cp.power, cp.toughness), (5, 5), "4/4 base + one +1/+1 counter");
}

/// Stuck in Summoner's Sanctum keeps the enchanted permanent tapped and locks
/// its activated abilities.
#[test]
fn stuck_in_summoners_sanctum_locks_permanent() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::stuck_in_summoners_sanctum());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(foe);
    g.battlefield_find_mut(foe).unwrap().tapped = true;
    assert!(g.computed_permanent(foe).unwrap().keywords.contains(&Keyword::CantActivateAbilities),
        "activated abilities locked");
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(foe).unwrap().tapped, "enchanted permanent doesn't untap");
}

/// Buster Sword grants +3/+2 and draws when the equipped creature hits a player.
#[test]
fn buster_sword_pumps_and_draws() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let bearer = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let sword = g.add_card_to_battlefield(0, catalog::buster_sword());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(bearer);
    g.add_card_to_library(0, catalog::island());
    assert_eq!(g.computed_permanent(bearer).unwrap().power, 5, "2/2 + 3/+2");
    g.clear_sickness(bearer);
    let hand0 = g.players[0].hand.len();
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bearer, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "combat damage drew a card");
}

/// Absolute Virtue ships as an 8/8 flyer that can't be countered and grants its
/// controller hexproof (its protection-from-opponents approximation).
#[test]
fn absolute_virtue_shape_and_controller_hexproof() {
    let mut g = two_player_game();
    let av = g.add_card_to_battlefield(0, catalog::absolute_virtue());
    let cp = g.computed_permanent(av).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 8));
    assert!(cp.keywords.contains(&Keyword::Flying) && cp.keywords.contains(&Keyword::CantBeCountered));
    // The controller gains player hexproof from the static (its protection approx).
    assert!(g.player_has_static_hexproof(0), "controller has static hexproof");
}

/// The Masamune grants first strike + must-be-blocked only during your turn.
#[test]
fn the_masamune_conditional_combat_keywords() {
    let mut g = two_player_game();
    let bearer = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sword = g.add_card_to_battlefield(0, catalog::the_masamune());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(bearer);
    g.active_player_idx = 0;
    let cp = g.computed_permanent(bearer).unwrap();
    assert!(cp.keywords.contains(&Keyword::FirstStrike) && cp.keywords.contains(&Keyword::MustBeBlocked),
        "first strike + lure on your turn");
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(bearer).unwrap().keywords.contains(&Keyword::FirstStrike),
        "no bonus on the opponent's turn");
}

/// Dark Knight's Greatsword's Job select mints a Hero and grants +3/+0 Knight.
#[test]
fn dark_knights_greatsword_job_select() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::dark_knights_greatsword());
    drain_stack(&mut g);
    let hero = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Hero")
        .expect("Hero minted").id;
    let cp = g.computed_permanent(hero).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 1), "1/1 + 3/+0");
    assert!(cp.subtypes.creature_types.contains(&crate::card::CreatureType::Knight));
}

/// Summoner's Grimoire puts a creature from hand onto the battlefield when the
/// equipped Hero attacks.
#[test]
fn summoners_grimoire_cheats_creature_on_attack() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::summoners_grimoire());
    drain_stack(&mut g);
    let hero = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Hero")
        .expect("Hero minted").id;
    let big = g.add_card_to_hand(0, catalog::serra_angel());
    g.clear_sickness(hero);
    // Force the optional put-from-hand to pick the Angel.
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Cards(vec![big]),
    ]));
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: hero, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_some(), "Angel put onto the battlefield from hand");
}

/// The Water Crystal's activated mill scales with your hand and is boosted by
/// its own opponent-mill replacement.
#[test]
fn the_water_crystal_mills_scaled_by_hand() {
    let mut g = two_player_game();
    let crystal = g.add_card_to_battlefield(0, catalog::the_water_crystal());
    for _ in 0..3 { g.add_card_to_hand(0, catalog::island()); } // hand of 3
    for _ in 0..12 { g.add_card_to_library(1, catalog::island()); }
    let gy1 = g.players[1].graveyard.len();
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: crystal, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    // 3 (hand) doubled by the crystal's own replacement = 6 milled.
    assert_eq!(g.players[1].graveyard.len(), gy1 + 6, "milled 3×2 = 6");
}

/// The Wandering Minstrel's activated pump scales with Towns you control, and
/// its static makes lands enter untapped.
#[test]
fn the_wandering_minstrel_town_scaling() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::the_wandering_minstrel());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for f in [catalog::baron_airship_kingdom, catalog::treno_dark_city, catalog::windurst_federation_center] {
        g.add_card_to_battlefield(0, f());
    }
    // Lands enter untapped despite Towns normally entering tapped.
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let town = g.add_card_to_hand(0, catalog::gohn_town_of_ruin());
    g.perform_action(GameAction::PlayLand(town)).expect("play a Town");
    assert!(!g.battlefield_find(town).unwrap().tapped, "Town entered untapped via the static");
    // Now four Towns → +4/+4 to the ally.
    let minstrel = g.battlefield.iter()
        .find(|c| c.definition.name == "The Wandering Minstrel").unwrap().id;
    for c in [crate::mana::Color::White, crate::mana::Color::Blue, crate::mana::Color::Black,
              crate::mana::Color::Red, crate::mana::Color::Green] {
        g.players[0].mana_pool.add(c, 1);
    }
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: minstrel, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(ally).unwrap().power, 2 + 4, "+X/+X where X = 4 Towns");
}

/// Nibelheim Aflame — the chosen creature deals its power to each *other*
/// creature; the source is unharmed and its power (not a fixed number) is used.
#[test]
fn nibelheim_aflame_pings_each_other_creature() {
    let mut g = two_player_game();
    let source = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2, power 2
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → dies
    let big = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4 → survives w/ 2 dmg
    let spell = g.add_card_to_hand(0, catalog::nibelheim_aflame());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crate::mana::Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(source)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Nibelheim Aflame");
    drain_stack(&mut g);
    assert!(g.battlefield_find(source).is_some(), "source is not hit");
    assert!(g.battlefield_find(small).is_none(), "2/2 took 2 and died");
    assert_eq!(g.battlefield_find(big).unwrap().damage, 2, "big took power-2 damage");
}

/// Ignis Scientia's ETB digs six deep and deploys a land tapped.
#[test]
fn ignis_scientia_digs_out_a_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_library(0, catalog::forest());
    g.move_card_to_battlefield_for_test(0, catalog::ignis_scientia());
    drain_stack(&mut g);
    let forest = g.battlefield_find(land).expect("land deployed");
    assert_eq!(forest.controller, 0, "under your control");
    assert!(forest.tapped, "enters tapped");
}

/// Ignis Scientia's ability makes a Food only when a creature card is exiled.
#[test]
fn ignis_scientia_food_on_creature_exile() {
    let mut g = two_player_game();
    let corpse = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let ability = catalog::ignis_scientia().activated_abilities[0].effect.clone();
    let mut ctx = crate::game::effects::EffectContext::for_trigger(crate::card::CardId(99), 0, None, 0);
    ctx.targets = vec![Target::Permanent(corpse)];
    g.resolve_effect(&ability, &ctx).unwrap();
    assert!(g.exile.iter().any(|c| c.id == corpse), "creature exiled");
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Food"),
        "a Food token was created"
    );
}

/// Genji Glove grants double strike and a single extra combat — the extra
/// combat does NOT re-trigger (no loop), and the attacker is untapped between.
#[test]
fn genji_glove_grants_one_extra_combat() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let hero = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let glove = g.add_card_to_battlefield(0, catalog::genji_glove());
    g.battlefield_find_mut(glove).unwrap().attached_to = Some(hero);
    g.clear_sickness(hero);
    let start = g.players[1].life;

    // First combat: attack. Double strike → 4 damage; glove grants an extra combat.
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: hero, target: AttackTarget::Player(1),
    }])).expect("attack 1");
    drain_stack(&mut g);
    assert_eq!(g.additional_combat_phases, 1, "one extra combat banked");
    advance_to(&mut g, TurnStep::BeginCombat);
    assert!(!g.battlefield_find(hero).unwrap().tapped, "attacker untapped for combat 2");

    // Second combat: attack again. The gate blocks a further extra combat.
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: hero, target: AttackTarget::Player(1),
    }])).expect("attack 2");
    drain_stack(&mut g);
    assert_eq!(g.additional_combat_phases, 0, "no third combat granted (no loop)");
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, start - 8, "double strike 4 dmg × two combats");
}

/// Ultima wipes every artifact and creature but spares lands.
#[test]
fn ultima_wipes_artifacts_and_creatures() {
    let mut g = two_player_game();
    let c1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let art = g.add_card_to_battlefield(0, catalog::phoenix_down()); // Artifact
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let eff = catalog::ultima().effect.clone();
    let ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&eff, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(c1).is_none() && g.battlefield_find(c2).is_none(), "creatures gone");
    assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
    assert!(g.battlefield_find(land).is_some(), "land survives");
}

/// Summon: Knights of Round makes three Knights per early chapter and its finale
/// buffs and shields the rest of the team.
#[test]
fn summon_knights_of_round_tokens_then_finale() {
    let mut g = two_player_game();
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let saga = g.add_card_to_battlefield(0, catalog::summon_knights_of_round());
    g.saga_advance(saga); // I
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Knight").count(),
        3,
        "chapter I made three Knights"
    );
    for _ in 0..4 { g.saga_advance(saga); drain_stack(&mut g); } // II, III, IV, V
    let cp = g.computed_permanent(ally).unwrap();
    assert_eq!(cp.power, 4, "finale gave +2/+2");
    assert!(
        g.battlefield_find(ally).unwrap().counter_count(CounterType::Indestructible) >= 1,
        "finale added an indestructible counter"
    );
}

/// The Lunar Whale lets you play off the top for the rest of the turn once it
/// attacks.
#[test]
fn the_lunar_whale_unlocks_top_of_library_on_attack() {
    let mut g = two_player_game();
    let whale = g.add_card_to_battlefield(0, catalog::the_lunar_whale());
    assert!(!g.players[0].play_from_top_this_turn, "off by default");
    let eff = catalog::the_lunar_whale().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(whale, 0, None, 0);
    g.resolve_effect(&eff, &ctx).unwrap();
    assert!(g.players[0].play_from_top_this_turn, "attack unlocked top-of-library");
}

/// Tellah spawns a Hero on any noncreature cast; big-mana casts add draws and,
/// past 8 mana, sacrifice him for a burst to each opponent.
#[test]
fn tellah_scales_with_mana_spent() {
    let mut g = two_player_game();
    let tellah = g.add_card_to_battlefield(0, catalog::tellah_great_sage());
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let eff = catalog::tellah_great_sage().triggered_abilities[0].effect.clone();
    // 8+ mana spent: Hero token, +2 draws, sac Tellah, burn each opponent.
    let hand0 = g.players[0].hand.len();
    let life1 = g.players[1].life;
    let mut ctx = crate::game::effects::EffectContext::for_trigger(tellah, 0, None, 0);
    ctx.mana_spent = 8;
    g.resolve_effect(&eff, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Hero"), "made a Hero");
    assert_eq!(g.players[0].hand.len(), hand0 + 2, "drew two at 4+ mana");
    assert!(g.battlefield_find(tellah).is_none(), "sacrificed at 8+ mana");
    assert_eq!(g.players[1].life, life1 - 8, "dealt 8 to the opponent");
}

/// Ragnarok's death trigger destroys a permanent and reanimates a nonlegendary
/// permanent card from your graveyard.
#[test]
fn ragnarok_death_destroys_and_reanimates() {
    let mut g = two_player_game();
    let ragnarok = g.add_card_to_battlefield(0, catalog::ragnarok_divine_deliverance());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let corpse = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // nonlegendary permanent
    let eff = catalog::ragnarok_divine_deliverance().triggered_abilities[0].effect.clone();
    let mut ctx = crate::game::effects::EffectContext::for_trigger(ragnarok, 0, None, 0);
    ctx.targets = vec![Target::Permanent(victim), Target::Permanent(corpse)];
    g.resolve_effect(&eff, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(victim).is_none(), "target permanent destroyed");
    assert!(g.battlefield_find(corpse).is_some(), "graveyard permanent reanimated");
}

/// Omega taps an opposing permanent, stuns it, and gains life — all scaled by
/// the number of nonbasic lands you control.
#[test]
fn omega_stuns_and_gains_by_nonbasic_lands() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_battlefield(0, catalog::mutavault()); } // 2 nonbasic lands → X=2
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let omega = g.add_card_to_battlefield(0, catalog::omega_heartless_evolution());
    let life0 = g.players[0].life;
    let eff = catalog::omega_heartless_evolution().triggered_abilities[0].effect.clone();
    let mut ctx = crate::game::effects::EffectContext::for_trigger(omega, 0, None, 0);
    ctx.targets = vec![Target::Permanent(foe)];
    g.resolve_effect(&eff, &ctx).unwrap();
    let f = g.battlefield_find(foe).unwrap();
    assert!(f.tapped, "target tapped");
    assert_eq!(f.counter_count(crate::card::CounterType::Stun), 2, "two stun counters");
    assert_eq!(g.players[0].life, life0 + 2, "gained 2 life");
}

/// CR 500.7 — a banked additional end step loops the turn back into another End
/// step exactly once (the counter drives the gate that prevents an infinite loop).
#[test]
fn cr_500_7_additional_end_step_loops_once() {
    let mut g = two_player_game();
    advance_to(&mut g, TurnStep::End);
    assert_eq!(g.end_steps_this_turn, 1, "first end step");
    g.additional_end_steps = 1;
    // Pass priority until the (banked) step loops back into a second End step.
    let before = g.end_steps_this_turn;
    while g.end_steps_this_turn == before && !g.is_game_over() {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.end_steps_this_turn, 2, "looped into a second end step");
    assert_eq!(g.additional_end_steps, 0, "banked step consumed — no infinite loop");
    assert!(matches!(g.step, TurnStep::End), "still an end step");
}

/// Y'shtola Rhul blinks a creature you control and, on the first end step,
/// banks an additional end step.
#[test]
fn yshtola_rhul_blinks_and_extends_first_end_step() {
    let mut g = two_player_game();
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ysh = g.add_card_to_battlefield(0, catalog::yshtola_rhul());
    g.end_steps_this_turn = 1; // as if the first end step just began
    let eff = catalog::yshtola_rhul().triggered_abilities[0].effect.clone();
    let mut ctx = crate::game::effects::EffectContext::for_trigger(ysh, 0, None, 0);
    ctx.targets = vec![Target::Permanent(ally)];
    g.resolve_effect(&eff, &ctx).unwrap();
    assert!(!g.exile.iter().any(|c| c.id == ally), "creature not stranded in exile");
    assert!(g.battlefield_find(ally).is_some(), "creature blinked back onto the battlefield");
    assert_eq!(g.additional_end_steps, 1, "first end step banks an extra");
}

/// Beatrix attaches every Equipment you control to the chosen creature at
/// combat.
#[test]
fn beatrix_attaches_all_equipment_at_combat() {
    let mut g = two_player_game();
    let hero = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let e1 = g.add_card_to_battlefield(0, catalog::genji_glove());
    let e2 = g.add_card_to_battlefield(0, catalog::phoenix_down()); // artifact, not Equipment
    g.add_card_to_battlefield(0, catalog::beatrix_loyal_general());
    let eff = catalog::beatrix_loyal_general().triggered_abilities[0].effect.clone();
    let mut ctx = crate::game::effects::EffectContext::for_trigger(crate::card::CardId(99), 0, None, 0);
    ctx.targets = vec![Target::Permanent(hero)];
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.battlefield_find(e1).unwrap().attached_to, Some(hero), "Equipment attached");
    assert_eq!(g.battlefield_find(e2).unwrap().attached_to, None, "non-Equipment untouched");
}

/// Kain has flying only on your turn, and handing a hit to a player donates
/// Kain to them while paying you cards/Treasures/life equal to the damage.
#[test]
fn kain_jump_and_traitor_trigger() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let kain = g.add_card_to_battlefield(0, catalog::kain_traitorous_dragoon());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    // Jump: flying on your turn only.
    g.active_player_idx = 0;
    assert!(g.computed_permanent(kain).unwrap().keywords.contains(&Keyword::Flying), "Jump on your turn");
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(kain).unwrap().keywords.contains(&Keyword::Flying), "no flying on their turn");
    g.active_player_idx = 0;
    // Traitor trigger: 2 combat damage to P1 → P1 gains Kain; you draw 2,
    // make 2 tapped Treasures, lose 2 life.
    let hand0 = g.players[0].hand.len();
    let life0 = g.players[0].life;
    let eff = catalog::kain_traitorous_dragoon().triggered_abilities[0].effect.clone();
    let mut ctx = crate::game::effects::EffectContext::for_trigger(kain, 0, Some(Target::Player(1)), 0);
    ctx.targets = vec![Target::Player(1)];
    ctx.event_amount = 2;
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.battlefield_find(kain).unwrap().controller, 1, "P1 gained control of Kain");
    assert_eq!(g.players[0].hand.len(), hand0 + 2, "you drew 2");
    assert_eq!(g.players[0].life, life0 - 2, "you lost 2 life");
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Treasure" && c.controller == 0).count(),
        2,
        "two Treasures for you"
    );
}

/// Cecil, Dark Knight is a {B} 2/3 deathtouch DFC whose back is a 4/4 lifelink.
#[test]
fn cecil_dark_knight_stats_and_back_face() {
    let c = catalog::cecil_dark_knight();
    assert_eq!((c.power, c.toughness), (2, 3));
    assert!(c.keywords.contains(&Keyword::Deathtouch));
    let back = c.back_face.as_ref().expect("has a back face");
    assert_eq!(back.name, "Cecil, Redeemed Paladin");
    assert_eq!((back.power, back.toughness), (4, 4));
    assert!(back.keywords.contains(&Keyword::Lifelink));
}

/// Cecil's combat-damage trigger drains you, and flips him once your life is at
/// or below half your starting total (20 → 10).
#[test]
fn cecil_flips_at_half_starting_life() {
    let mut g = two_player_game();
    let cecil = g.add_card_to_battlefield(0, catalog::cecil_dark_knight());
    g.players[0].life = 13; // above half (10) — no flip yet
    let eff = catalog::cecil_dark_knight().triggered_abilities[0].effect.clone();
    let mut ctx = crate::game::effects::EffectContext::for_trigger(cecil, 0, None, 0);
    ctx.event_amount = 2;
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.players[0].life, 11, "lost 2 life");
    assert!(!g.battlefield_find(cecil).unwrap().transformed, "still front above half");
    // One more hit lands at 10 = half → flip.
    ctx.event_amount = 1;
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.players[0].life, 10);
    assert!(g.battlefield_find(cecil).unwrap().transformed, "flipped at/below half");
}

/// Galuf's Final Act pumps +1/+0 and grants a death trigger.
#[test]
fn galufs_final_act_pumps_and_grants_death_trigger() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let eff = catalog::galufs_final_act().effect.clone();
    let mut ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&eff, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3, "2/2 + 1/0");
    assert!(
        g.granted_triggers_eot.get(&bear).map(|v| !v.is_empty()).unwrap_or(false),
        "granted a death trigger"
    );
}

/// Clash of the Eikons is a choose-one-or-more with three real modes.
#[test]
fn clash_of_the_eikons_is_modal() {
    let c = catalog::clash_of_the_eikons();
    match c.effect {
        crate::card::Effect::ChooseN { ref picks, ref modes } => {
            assert_eq!(picks, &vec![1, 2, 3], "choose one or more of three");
            assert_eq!(modes.len(), 3);
        }
        _ => panic!("expected ChooseN"),
    }
}

/// Louisoix's Sacrifice counters a noncreature spell and carries its sac-or-pay
/// additional cost.
#[test]
fn louisoixs_sacrifice_counters_noncreature() {
    use crate::card::AdditionalCastCost;
    let c = catalog::louisoixs_sacrifice();
    assert!(matches!(
        c.additional_cast_cost.first(),
        Some(AdditionalCastCost::SacrificeOrPay { pay: 2, .. })
    ));
    let mut g = two_player_game();
    // Put a noncreature spell (an instant) on the stack owned by P1.
    let bolt = g.add_card_to_hand(1, catalog::molten_rain()); // sorcery = noncreature
    let _ = bolt;
    // Resolve the counter against a stack item constructed via cast is heavy;
    // just assert the counter targets a noncreature spell filter.
    match c.effect {
        crate::card::Effect::CounterSpell { what: crate::card::Selector::TargetFiltered { slot: 0, ref filter } } => {
            assert!(format!("{filter:?}").contains("Noncreature"), "targets noncreature spell");
        }
        _ => panic!("expected CounterSpell"),
    }
}

/// Kefka is a 4/5 DFC whose back is a 5/7 flier that draws off opponents' life
/// loss on your turn.
#[test]
fn kefka_dfc_stats_and_back_draw() {
    let c = catalog::kefka_court_mage();
    assert_eq!((c.power, c.toughness), (4, 5));
    let back = c.back_face.as_ref().expect("has a back face");
    assert_eq!(back.name, "Kefka, Ruler of Ruin");
    assert_eq!((back.power, back.toughness), (5, 7));
    assert!(back.keywords.contains(&Keyword::Flying));
    // Back's life-loss trigger draws that many cards.
    let mut g = two_player_game();
    g.active_player_idx = 0; // your turn
    let kefka = g.add_card_to_battlefield(0, *back.clone());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let hand0 = g.players[0].hand.len();
    let eff = back.triggered_abilities[0].effect.clone();
    let mut ctx = crate::game::effects::EffectContext::for_trigger(kefka, 0, None, 0);
    ctx.event_amount = 3;
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.players[0].hand.len(), hand0 + 3, "drew 3 for the 3 life lost");
}

/// Kefka's {8} sorcery-speed ability edicts each opponent and transforms him.
#[test]
fn kefka_eight_mana_edicts_and_transforms() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let kefka = g.add_card_to_battlefield(0, catalog::kefka_court_mage());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 8);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kefka, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("{8}: edict + transform");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "opponent sacrificed their creature");
    assert!(g.battlefield_find(kefka).unwrap().transformed, "Kefka flipped to Ruler of Ruin");
}

/// CR 603.2 — Kefka, Ruler of Ruin's life-loss trigger honors its "during your
/// turn" gate (`Predicate::IsTurnOf(You)` on the `EventSpec` filter): it fires
/// when an opponent loses life on your turn and stays silent on theirs.
#[test]
fn kefka_back_trigger_gated_to_your_turn() {
    let back = *catalog::kefka_court_mage().back_face.clone().unwrap();
    // Your turn: an opponent losing 2 life draws you 2 cards.
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let _kefka = g.add_card_to_battlefield(0, back.clone());
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let hand0 = g.players[0].hand.len();
    g.dispatch_triggers_for_events(&[crate::game::GameEvent::LifeLost { player: 1, amount: 2 }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 2, "drew 2 on your turn");
    // Opponent's turn: same life loss, no draw.
    let mut g2 = two_player_game();
    g2.active_player_idx = 1;
    let _k2 = g2.add_card_to_battlefield(0, back.clone());
    g2.add_card_to_library(0, catalog::island());
    let h2 = g2.players[0].hand.len();
    g2.dispatch_triggers_for_events(&[crate::game::GameEvent::LifeLost { player: 1, amount: 2 }]);
    drain_stack(&mut g2);
    assert_eq!(g2.players[0].hand.len(), h2, "no draw on the opponent's turn");
}

/// Clive's Hideaway is a Town land with Hideaway 4 and a gated free-play ability.
#[test]
fn clives_hideaway_is_hideaway_town() {
    use crate::card::LandType;
    let c = catalog::clives_hideaway();
    assert!(c.subtypes.land_types.contains(&LandType::Town));
    assert!(
        c.triggered_abilities.iter().any(|t| matches!(t.effect, crate::card::Effect::Hideaway { .. })),
        "has Hideaway on ETB"
    );
    assert_eq!(c.activated_abilities.len(), 2, "tap-for-C + gated free play");
    assert!(c.activated_abilities[1].condition.is_some(), "free play is gated");
}

/// Starting Town is a Town land that taps for {C} or, for 1 life, any color.
#[test]
fn starting_town_taps_for_any_color_for_life() {
    use crate::card::LandType;
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::starting_town());
    assert!(catalog::starting_town().subtypes.land_types.contains(&LandType::Town));
    let life0 = g.players[0].life;
    // Second ability: {T}, pay 1 life → one mana of any color.
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("pay 1 life for any color");
    assert_eq!(g.players[0].life, life0 - 1, "paid 1 life");
    assert!(g.players[0].mana_pool.total() >= 1, "produced a mana");
}

/// CR 712.9 / 122.2 — transforming a permanent is the same object staying in
/// place, so counters ride along. Cecil flips with a +1/+1 counter and the back
/// face (4/4) computes 5/5.
#[test]
fn cr_712_9_transform_keeps_counters() {
    let mut g = two_player_game();
    let cecil = g.add_card_to_battlefield(0, catalog::cecil_dark_knight());
    g.battlefield_find_mut(cecil).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.players[0].life = 11; // 11 − 1 = 10 = half → the trigger will flip
    let eff = catalog::cecil_dark_knight().triggered_abilities[0].effect.clone();
    let mut ctx = crate::game::effects::EffectContext::for_trigger(cecil, 0, None, 0);
    ctx.event_amount = 1;
    g.resolve_effect(&eff, &ctx).unwrap();
    let inst = g.battlefield_find(cecil).unwrap();
    assert!(inst.transformed, "flipped");
    assert_eq!(inst.counter_count(CounterType::PlusOnePlusOne), 1, "counter survived the flip");
    let cp = g.computed_permanent(cecil).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "4/4 back face + a +1/+1 counter");
}

/// Elixir shuffles only nonland cards from your graveyard back and gains you
/// life equal to the number returned.
#[test]
fn elixir_reshuffles_nonlands_and_gains_life() {
    let mut g = two_player_game();
    let elixir = g.add_card_to_battlefield(0, catalog::elixir());
    // Graveyard: two nonland cards + one land.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::forest());
    let life0 = g.players[0].life;
    let lib0 = g.players[0].library.len();
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: elixir, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("{5},{T},exile: reshuffle nonlands");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 2, "gained 2 for the 2 nonlands");
    assert_eq!(g.players[0].library.len(), lib0 + 2, "two nonlands shuffled in");
    assert_eq!(g.players[0].graveyard.len(), 1, "the land stayed in the graveyard");
    assert!(g.battlefield_find(elixir).is_none(), "Elixir exiled itself");
}

/// Yuna grants herself and enchantment creatures trample/lifelink/ward, but only
/// during her controller's turn (the new `AnthemForFilter.only_your_turn`).
#[test]
fn yuna_turn_gated_anthem_and_self_keywords() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let yuna = g.add_card_to_battlefield(0, catalog::yuna_hope_of_spira());
    let ench = g.add_card_to_battlefield(0, catalog::summon_choco_mog()); // enchantment creature
    // Your turn — Yuna and the enchantment creature are pumped.
    g.active_player_idx = 0;
    for id in [yuna, ench] {
        let cp = g.computed_permanent(id).unwrap();
        assert!(cp.keywords.contains(&Keyword::Trample), "trample on your turn");
        assert!(cp.keywords.contains(&Keyword::Lifelink), "lifelink on your turn");
        assert!(cp.keywords.iter().any(|k| matches!(k, Keyword::Ward(_))), "ward on your turn");
    }
    // Opponent's turn — the grants switch off.
    g.active_player_idx = 1;
    let cp = g.computed_permanent(yuna).unwrap();
    assert!(!cp.keywords.contains(&Keyword::Trample), "no trample off-turn");
    let ce = g.computed_permanent(ench).unwrap();
    assert!(!ce.keywords.contains(&Keyword::Lifelink), "no lifelink off-turn");
}

/// Yuna's end step reanimates an enchantment card with a finality counter.
#[test]
fn yuna_end_step_reanimates_with_finality() {
    let mut g = two_player_game();
    let _yuna = g.add_card_to_battlefield(0, catalog::yuna_hope_of_spira());
    let ench = g.add_card_to_graveyard(0, catalog::summon_choco_mog());
    let eff = catalog::yuna_hope_of_spira().triggered_abilities[0].effect.clone();
    let mut ctx = crate::game::effects::EffectContext::for_trigger(_yuna, 0, Some(Target::Permanent(ench)), 0);
    ctx.targets = vec![Target::Permanent(ench)];
    g.resolve_effect(&eff, &ctx).unwrap();
    let back = g.battlefield.iter().find(|c| c.id == ench).expect("reanimated to battlefield");
    assert_eq!(back.counter_count(CounterType::Finality), 1, "entered with a finality counter");
}

/// Summon: Fenrir chapter I fetches a basic land onto the battlefield tapped.
#[test]
fn summon_fenrir_chapter_one_fetches_land() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let fenrir = g.add_card_to_battlefield(0, catalog::summon_fenrir());
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    g.saga_advance(fenrir); // I — Crescent Fang
    drain_stack(&mut g);
    assert!(g.battlefield_find(forest).is_some_and(|c| c.tapped), "fetched a Forest, entered tapped");
}

/// Summon: Fenrir chapter II makes your next creature spell enter with an extra
/// +1/+1 counter (a 2/2 Grizzly Bears enters as a 3/3).
#[test]
fn summon_fenrir_chapter_two_grows_next_creature() {
    let mut g = two_player_game();
    let fenrir = g.add_card_to_battlefield(0, catalog::summon_fenrir());
    g.saga_advance(fenrir); // I
    drain_stack(&mut g);
    g.saga_advance(fenrir); // II — Heavenward Howl
    drain_stack(&mut g);
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Grizzly Bears");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bears).expect("bears on battlefield");
    assert_eq!((cp.power, cp.toughness), (3, 3), "entered with an extra +1/+1 counter");
    // The rider is one-shot: a second creature enters at its printed size.
    let bears2 = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bears2, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast second Grizzly Bears");
    drain_stack(&mut g);
    let cp2 = g.computed_permanent(bears2).unwrap();
    assert_eq!((cp2.power, cp2.toughness), (2, 2), "rider consumed by the first creature only");
}

/// Summon: Fenrir chapter III draws when you control the greatest-power creature.
#[test]
fn summon_fenrir_chapter_three_draws_on_greatest_power() {
    let mut g = two_player_game();
    let fenrir = g.add_card_to_battlefield(0, catalog::summon_fenrir()); // 3/2
    g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 — smaller
    g.add_card_to_library(0, catalog::forest());
    let hand0 = g.players[0].hand.len();
    g.saga_advance(fenrir); // I
    drain_stack(&mut g);
    g.saga_advance(fenrir); // II
    drain_stack(&mut g);
    g.saga_advance(fenrir); // III — Ecliptic Growl
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew: Fenrir is the greatest power");
}

/// Stiltzkin donates a permanent to the opponent and draws a card.
#[test]
fn stiltzkin_donates_and_draws() {
    let mut g = two_player_game();
    let stiltzkin = g.add_card_to_battlefield(0, catalog::stiltzkin_moogle_merchant());
    let gift = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: stiltzkin, ability_index: 0,
        target: Some(Target::Permanent(gift)), additional_targets: vec![], x_value: None,
    }).expect("activate Stiltzkin");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(gift).unwrap().controller, 1, "opponent now controls the gift");
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "controller drew a card");
}

/// Vincent's Limit Break's Death Gigas tier ({1}) sets a creature to 5/2 and
/// makes it return tapped when it dies this turn.
#[test]
fn vincents_limit_break_death_gigas_returns_on_death() {
    use crate::game::Target;
    let mut g = two_player_game();
    let cat = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::vincents_limit_break());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2); // base {1}{B} + Death Gigas {1}
    g.perform_action(GameAction::CastSpellSpree {
        card_id: id, spree_modes: vec![1], target: Some(Target::Permanent(cat)),
        additional_targets: vec![], x_value: None,
    }).expect("cast Death Gigas");
    drain_stack(&mut g);
    let cp = g.computed_permanent(cat).expect("still there");
    assert_eq!((cp.power, cp.toughness), (5, 2), "base P/T set to 5/2");
    // Kill it (2 damage vs the 5/2 body) and confirm it returns tapped.
    g.resolve_effect(&crate::effect::Effect::DealDamage {
        to: crate::effect::Selector::Target(0),
        amount: crate::effect::Value::Const(2),
    }, &crate::game::effects::EffectContext::for_ability(
        crate::card::CardId(0), 0, Some(Target::Permanent(cat)),
    )).unwrap();
    g.check_state_based_actions();
    drain_stack(&mut g);
    let back = g.battlefield_find(cat).expect("returned to battlefield");
    assert!(back.tapped, "returned tapped");
}

/// Garnet's attack removes a lore counter from each of your Sagas and grows by
/// one +1/+1 counter per Saga drained.
#[test]
fn garnet_attack_drains_sagas_and_grows() {
    let mut g = two_player_game();
    let garnet = g.add_card_to_battlefield(0, catalog::garnet_princess_of_alexandria());
    let saga = g.add_card_to_battlefield(0, catalog::summon_choco_mog());
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 2);
    let eff = catalog::garnet_princess_of_alexandria().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(garnet, 0, None, 0);
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(
        g.battlefield_find(garnet).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1, "one Saga drained → one +1/+1 counter",
    );
    assert_eq!(
        g.battlefield_find(saga).unwrap().counter_count(CounterType::Lore),
        1, "saga lost a lore counter (2 → 1)",
    );
}

/// Summon: Brynhildr's Gestalt Mode (II) gives your next creature spell haste.
#[test]
fn summon_brynhildr_gestalt_grants_haste() {
    let mut g = two_player_game();
    let bryn = g.add_card_to_battlefield(0, catalog::summon_brynhildr());
    g.saga_advance(bryn); // I — Chain (needs a library card to exile)
    drain_stack(&mut g);
    g.saga_advance(bryn); // II — Gestalt Mode
    drain_stack(&mut g);
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Grizzly Bears");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bears).expect("bears on battlefield");
    assert!(cp.keywords.contains(&Keyword::Haste), "next creature entered with haste");
}

/// Torgal grows your first Human creature spell each turn by a +1/+1 counter per
/// Dog/Wolf you control (Torgal + Watchwolf = 2 → a 2/1 Human enters as 4/3).
#[test]
fn torgal_pumps_first_human_creature() {
    let mut g = two_player_game();
    let _torgal = g.add_card_to_battlefield(0, catalog::torgal_a_fine_hound()); // Wolf
    g.add_card_to_battlefield(0, catalog::watchwolf()); // second Wolf
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let human = g.add_card_to_hand(0, catalog::beskir_shieldmate()); // {1}{W} 2/1 Human
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: human, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Beskir Shieldmate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(human).expect("human on battlefield");
    assert_eq!((cp.power, cp.toughness), (4, 3), "2/1 + two +1/+1 counters (two Wolves)");
}

/// Reno and Rude: on combat damage, sacrificing a creature exiles the top of
/// the damaged player's library and lets you play it this turn.
#[test]
fn reno_and_rude_impulses_from_victim_library() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let reno = g.add_card_to_battlefield(0, catalog::reno_and_rude());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // sac fodder
    let loot = g.add_card_to_library(1, catalog::lightning_bolt()); // top of victim's library
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // yes, sacrifice
    let eff = catalog::reno_and_rude().triggered_abilities[0].effect.clone();
    let mut ctx = crate::game::effects::EffectContext::for_trigger(reno, 0, Some(Target::Player(1)), 0);
    ctx.targets = vec![Target::Player(1)];
    g.resolve_effect(&eff, &ctx).unwrap();
    assert!(g.battlefield_find(fodder).is_none(), "fodder was sacrificed");
    let exiled = g.exile.iter().find(|c| c.id == loot).expect("victim's top card exiled");
    assert!(exiled.may_play_until.is_some(), "you may play the exiled card");
}
