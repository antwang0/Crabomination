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
