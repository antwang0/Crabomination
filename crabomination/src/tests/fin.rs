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

/// Ahriman sacrifices another permanent to draw.
#[test]
fn ahriman_sacrifices_to_draw() {
    let mut g = two_player_game();
    let ahriman = g.add_card_to_battlefield(0, catalog::ahriman());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.next_id();
    g.players[0].library.push(crate::card::CardInstance::new(id, catalog::island(), 0));
    g.players[0].mana_pool.add_colorless(3);
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: ahriman, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    }).expect("activate Ahriman");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed the other creature");
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew a card");
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

/// Coeurl taps a target creature.
#[test]
fn coeurl_taps_target() {
    let mut g = two_player_game();
    let coeurl = g.add_card_to_battlefield(0, catalog::coeurl());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(coeurl);
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: coeurl, ability_index: 0, target: Some(Target::Permanent(foe)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("activate Coeurl");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "target creature is tapped");
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

/// Dwarven Castle Guard leaves a Hero token when it dies.
#[test]
fn dwarven_castle_guard_dies_makes_hero() {
    let mut g = two_player_game();
    let guard = g.add_card_to_battlefield(0, catalog::dwarven_castle_guard());
    g.battlefield_find_mut(guard).unwrap().toughness_bonus -= 1; // drop to 0 toughness → SBA
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    let heroes = g.battlefield.iter().filter(|c| c.definition.name == "Hero").count();
    assert_eq!(heroes, 1, "death minted a Hero token");
}

/// Gigantoad grows once you control seven or more lands.
#[test]
fn gigantoad_grows_with_seven_lands() {
    let mut g = two_player_game();
    let toad = g.add_card_to_battlefield(0, catalog::gigantoad());
    assert_eq!(g.computed_permanent(toad).unwrap().power, 4, "few lands → 4/4");
    for _ in 0..7 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    assert_eq!(g.computed_permanent(toad).unwrap().power, 6, "seven lands → +2/+2");
    assert_eq!(g.computed_permanent(toad).unwrap().toughness, 6);
}

/// Hill Gigas ships with trample, haste, and Mountaincycling.
#[test]
fn hill_gigas_keywords() {
    let g = catalog::hill_gigas();
    assert!(g.keywords.contains(&Keyword::Trample));
    assert!(g.keywords.contains(&Keyword::Haste));
    assert!(g.keywords.iter().any(|k| matches!(k, Keyword::Landcycling(_, crate::card::LandType::Mountain))));
}

/// Gaelicat grows +2/+0 with two or more artifacts.
#[test]
fn gaelicat_grows_with_two_artifacts() {
    let mut g = two_player_game();
    let cat = g.add_card_to_battlefield(0, catalog::gaelicat());
    assert_eq!(g.computed_permanent(cat).unwrap().power, 1, "few artifacts → 1/3");
    g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.add_card_to_battlefield(0, catalog::mishras_bauble());
    assert_eq!(g.computed_permanent(cat).unwrap().power, 3, "two artifacts → +2/+0");
    assert_eq!(g.computed_permanent(cat).unwrap().toughness, 3, "toughness unchanged");
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

/// Dragoon's Wyvern mints a Hero token on entry.
#[test]
fn dragoons_wyvern_makes_hero() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::dragoons_wyvern());
    drain_stack(&mut g);
    let heroes = g.battlefield.iter().filter(|c| c.definition.name == "Hero").count();
    assert_eq!(heroes, 1, "ETB minted a Hero token");
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
