//! Functionality tests for the MH3 batch-3 cards in `catalog::sets::mh3c`
//! and the Battle cry keyword (CR 702.92).

use crate::catalog;
use crate::card::{CounterType, Keyword};
use crate::game::types::{Attack, AttackTarget};
use crate::game::*;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Fill P0's pool with plenty of every color + colorless, then cast `id`.
fn cast(g: &mut GameState, id: crate::card::CardId, target: Option<crate::game::types::Target>,
        extra: Vec<crate::game::types::Target>) {
    for c in [crate::mana::Color::White, crate::mana::Color::Blue, crate::mana::Color::Black,
              crate::mana::Color::Red, crate::mana::Color::Green] {
        g.players[0].mana_pool.add(c, 8);
    }
    g.players[0].mana_pool.add_colorless(8);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target, additional_targets: extra, mode: None, x_value: None,
    }).expect("cast");
    drain_stack(g);
}

/// Goblin Wardriver's battle cry pumps each *other* attacker +1/+0, not itself.
#[test]
fn goblin_wardriver_battle_cry_pumps_team() {
    let mut g = two_player_game();
    let driver = g.add_card_to_battlefield(0, catalog::goblin_wardriver());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(driver);
    g.clear_sickness(bear);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: driver, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "other attacker +1/+0");
    assert_eq!(g.computed_permanent(driver).unwrap().power, 2, "battle cry excludes itself");
}

/// Battle cry does nothing when the creature attacks alone (no other attackers).
#[test]
fn battle_cry_solo_attacker_no_pump() {
    let mut g = two_player_game();
    let driver = g.add_card_to_battlefield(0, catalog::goblin_wardriver());
    g.clear_sickness(driver);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: driver, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(driver).unwrap().power, 2, "no team, no pump");
}

/// Reckless Pyrosurfer gains battle cry from a landfall trigger and then pumps
/// the rest of the attacking team when it swings.
#[test]
fn reckless_pyrosurfer_landfall_grants_battle_cry() {
    let mut g = two_player_game();
    let surfer = g.add_card_to_battlefield(0, catalog::reckless_pyrosurfer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    let land = g.add_card_to_hand(0, catalog::mountain());
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: surfer, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "granted battle cry pumps team");
}

/// Wurmcoil Larva dies into a 1/2 deathtouch and a 2/1 lifelink Wurm token.
#[test]
fn wurmcoil_larva_dies_into_two_wurms() {
    let mut g = two_player_game();
    let larva = g.add_card_to_battlefield(0, catalog::wurmcoil_larva());
    g.remove_to_graveyard_with_triggers(larva);
    drain_stack(&mut g);
    g.check_state_based_actions();
    let wurms: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Phyrexian Wurm").collect();
    assert_eq!(wurms.len(), 2, "two Wurm tokens");
    assert!(wurms.iter().any(|c| c.definition.keywords.contains(&Keyword::Deathtouch)));
    assert!(wurms.iter().any(|c| c.definition.keywords.contains(&Keyword::Lifelink)));
}

/// Spawn-Gang Commander mints three Eldrazi Spawn on cast and can sacrifice one
/// to ping any target for 2.
#[test]
fn spawn_gang_commander_spawns_and_pings() {
    let mut g = two_player_game();
    g.players[0].mana_pool.add(crate::mana::Color::Red, 5);
    g.players[0].mana_pool.add_colorless(2);
    let id = g.add_card_to_hand(0, catalog::spawn_gang_commander());
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let spawns = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Eldrazi Spawn").count();
    assert_eq!(spawns, 3, "cast trigger made three Eldrazi Spawn");
}

/// Vaultborn Tyrant draws + gains life when another power-4+ creature you
/// control enters.
#[test]
fn vaultborn_tyrant_etb_payoff() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::vaultborn_tyrant());
    g.add_card_to_library(0, catalog::forest());
    let life = g.players[0].life;
    // Cast Serra Angel (4/4, power ≥ 4) so the ETB event flows through the
    // trigger dispatcher; net hand change = -1 cast + 1 draw = 0.
    let angel = g.add_card_to_hand(0, catalog::serra_angel());
    let hand = g.players[0].hand.len();
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(crate::mana::Color::White, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: angel, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast angel");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3, "gained 3 when a power-4+ creature entered");
    assert_eq!(g.players[0].hand.len(), hand, "cast -1 + draw +1 = net 0");
}

/// Vaultborn Tyrant dies into an artifact token copy of itself.
#[test]
fn vaultborn_tyrant_dies_into_artifact_copy() {
    let mut g = two_player_game();
    let tyrant = g.add_card_to_battlefield(0, catalog::vaultborn_tyrant());
    g.remove_to_graveyard_with_triggers(tyrant);
    drain_stack(&mut g);
    g.check_state_based_actions();
    let copy = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Vaultborn Tyrant");
    let copy = copy.expect("token copy exists");
    assert!(g.computed_permanent(copy.id).unwrap().card_types.contains(&crate::card::CardType::Artifact),
        "the copy is an artifact");
}

/// Hydra Trainer's exert attack pumps a target by the number of counters on
/// your permanents.
#[test]
fn hydra_trainer_exert_pumps_by_counter_count() {
    let mut g = two_player_game();
    let trainer = g.add_card_to_battlefield(0, catalog::hydra_trainer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Two +1/+1 counters across your permanents → X = 2.
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    g.clear_sickness(trainer);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: trainer, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    // Bear is 2/2 base + 2 counters + 2/2 from exert = 6/6.
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 6), "+X/+X where X = 2 counters");
    assert!(g.battlefield_find(trainer).unwrap().skip_next_untap, "exerted");
}

/// Signature Slam pumps your creature (making it modified) then swings its power
/// at an enemy creature.
#[test]
fn signature_slam_modified_creatures_deal_damage() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.players[0].mana_pool.add(crate::mana::Color::Green, 3);
    let id = g.add_card_to_hand(0, catalog::signature_slam());
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(mine)),
        additional_targets: vec![crate::game::types::Target::Permanent(enemy)],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    g.check_state_based_actions();
    // mine becomes 3/3 (counter → modified), deals 3 to the 2/2 enemy → dead.
    assert_eq!(g.computed_permanent(mine).unwrap().power, 3, "got a +1/+1 counter");
    assert!(g.battlefield_find(enemy).is_none(), "enemy took 3, died");
}

/// Ajani Fells the Godsire: I exiles a big enemy creature, II makes a Cat and a
/// vigilance counter, III grants double strike.
#[test]
fn ajani_fells_the_godsire_chapters() {
    let mut g = two_player_game();
    let saga = g.add_card_to_battlefield(0, catalog::ajani_fells_the_godsire());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4, power ≥ 3
    g.saga_advance(saga); // I — exile the big enemy
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_none(), "chapter I exiled the power-3+ creature");
    g.saga_advance(saga); // II — Cat + vigilance counter
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Cat Warrior"),
        "chapter II made a Cat Warrior");
    assert!(g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::Vigilance),
        "chapter II put a vigilance counter on a creature");
    g.saga_advance(saga); // III — double strike
    drain_stack(&mut g);
    assert!(g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "chapter III granted double strike");
}

// ── Batch 4: modal DFCs + Aura ────────────────────────────────────────────────

use crate::game::types::Target;

/// Boggart Trawler's ETB exiles an opponent's graveyard; its back is a land.
#[test]
fn boggart_trawler_etb_exiles_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::boggart_trawler());
    cast(&mut g, id, None, vec![]);
    assert!(g.players[1].graveyard.is_empty(), "opponent's graveyard exiled");
    assert_eq!(catalog::boggart_trawler().back_face.unwrap().name, "Boggart Bog");
}

/// Boggart Bog (the pain-land back) can be played tapped without paying life.
#[test]
fn boggart_bog_back_enters_via_play_land_back() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::boggart_trawler());
    g.perform_action(GameAction::PlayLandBack(id)).expect("play the back");
    drain_stack(&mut g);
    let land = g.battlefield_find(id).expect("land on battlefield");
    assert_eq!(land.definition.name, "Boggart Bog");
}

/// Fell the Profane destroys a creature.
#[test]
fn fell_the_profane_destroys() {
    let mut g = two_player_game();
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fell_the_profane());
    cast(&mut g, id, Some(Target::Permanent(enemy)), vec![]);
    assert!(g.battlefield_find(enemy).is_none(), "creature destroyed");
}

/// Razorgrass Ambush deals 3 to an attacking creature.
#[test]
fn razorgrass_ambush_hits_attacker() {
    let mut g = two_player_game();
    // P0 attacks with a 4/4; while it's declared as an attacker, P0 casts the
    // Ambush at it (any attacking creature is a legal target).
    let atk = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.declare_attackers(vec![Attack { attacker: atk, target: AttackTarget::Player(1) }]).expect("attack");
    let id = g.add_card_to_hand(0, catalog::razorgrass_ambush());
    cast(&mut g, id, Some(Target::Permanent(atk)), vec![]);
    g.check_state_based_actions();
    // 4/4 took 3 → survives but marked; verify it took damage.
    assert_eq!(g.battlefield_find(atk).unwrap().damage, 3, "3 damage to the attacker");
}

/// Legion Leadership doubles a creature's power and grants first strike.
#[test]
fn legion_leadership_doubles_power() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::legion_leadership());
    cast(&mut g, id, Some(Target::Permanent(c)), vec![]);
    let cp = g.computed_permanent(c).unwrap();
    assert_eq!(cp.power, 8, "power doubled 4 → 8");
    assert!(cp.keywords.contains(&Keyword::FirstStrike), "gains first strike");
}

/// Revitalizing Repast adds a counter and grants indestructible.
#[test]
fn revitalizing_repast_counter_and_indestructible() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::revitalizing_repast());
    cast(&mut g, id, Some(Target::Permanent(c)), vec![]);
    let cp = g.computed_permanent(c).unwrap();
    assert_eq!(cp.power, 3, "got a +1/+1 counter");
    assert!(cp.keywords.contains(&Keyword::Indestructible), "gains indestructible");
}

/// Stump Stomp makes your creature deal its power to an enemy creature.
#[test]
fn stump_stomp_one_sided_fight() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::stump_stomp());
    cast(&mut g, id, Some(Target::Permanent(mine)), vec![Target::Permanent(enemy)]);
    g.check_state_based_actions();
    assert!(g.battlefield_find(enemy).is_none(), "enemy took 4, died");
    assert_eq!(g.battlefield_find(mine).unwrap().damage, 0, "one-sided — no back-swing");
}

/// Waterlogged Teachings tutors an instant to hand.
#[test]
fn waterlogged_teachings_tutors_instant() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::waterlogged_teachings());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Search(Some(bolt)),
    ]));
    cast(&mut g, id, None, vec![]);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "instant tutored to hand");
}

/// Lion Umbra can only enchant a modified creature and grants +3/+3, vigilance,
/// reach.
#[test]
fn lion_umbra_buffs_modified_creature() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    // Make it "modified" with a +1/+1 counter so it's a legal enchant target.
    g.battlefield_find_mut(c).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let id = g.add_card_to_hand(0, catalog::lion_umbra());
    cast(&mut g, id, Some(Target::Permanent(c)), vec![]);
    let cp = g.computed_permanent(c).unwrap();
    // 2/2 base + 1/1 counter + 3/3 aura = 6/6.
    assert_eq!((cp.power, cp.toughness), (6, 6), "+3/+3 on top of the counter");
    assert!(cp.keywords.contains(&Keyword::Vigilance) && cp.keywords.contains(&Keyword::Reach));
}

// ── Batch 5 tests ─────────────────────────────────────────────────────────────

/// Witch Enchanter's ETB destroys an opponent's artifact or enchantment.
#[test]
fn witch_enchanter_etb_destroys_permanent() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::wurmcoil_larva()); // an artifact
    let id = g.add_card_to_hand(0, catalog::witch_enchanter());
    cast(&mut g, id, Some(Target::Permanent(art)), vec![]);
    g.check_state_based_actions();
    // Wurmcoil Larva dies → replaced by tokens, so the original is gone.
    assert!(g.battlefield_find(art).is_none(), "opponent's artifact destroyed");
}

/// Pinnacle Monk returns an instant/sorcery from your graveyard on entry.
#[test]
fn pinnacle_monk_returns_spell_from_graveyard() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::pinnacle_monk());
    // Only one instant/sorcery in the graveyard — AutoDecider picks it.
    cast(&mut g, id, None, vec![]);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "spell returned to hand");
}

/// Bridgeworks Battle pumps your creature +2/+2 and fights an enemy.
#[test]
fn bridgeworks_battle_pump_and_fight() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 4/4
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::bridgeworks_battle());
    cast(&mut g, id, Some(Target::Permanent(mine)), vec![Target::Permanent(enemy)]);
    g.check_state_based_actions();
    assert!(g.battlefield_find(enemy).is_none(), "4-power fighter killed the 2/2");
    // Ours took 2 back but is 4/4 → survives.
    assert!(g.battlefield_find(mine).is_some(), "our pumped creature survives the fight");
}

/// Disciple of Freyalise can sac a creature to gain and draw its power.
#[test]
fn disciple_of_freyalise_sac_draws_power() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::serra_angel()); // power 4
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let life = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::disciple_of_freyalise());
    let hand = g.players[0].hand.len(); // includes the Disciple
    // Yes to the may-sacrifice; only the angel is a legal sac, so it's auto-picked.
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    // hand loses the Disciple on cast (-1), then draws 4.
    cast(&mut g, id, None, vec![]);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed the angel");
    assert_eq!(g.players[0].life, life + 4, "gained X = 4 life");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 4, "drew X = 4 cards");
}

/// Glasswing Grace grants +2/+2, flying, and lifelink.
#[test]
fn glasswing_grace_buffs_creature() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::glasswing_grace());
    cast(&mut g, id, Some(Target::Permanent(c)), vec![]);
    let cp = g.computed_permanent(c).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
    assert!(cp.keywords.contains(&Keyword::Flying) && cp.keywords.contains(&Keyword::Lifelink));
}

/// Strength of the Harvest scales with your creatures and enchantments.
#[test]
fn strength_of_the_harvest_scales() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // another creature
    let id = g.add_card_to_hand(0, catalog::strength_of_the_harvest());
    cast(&mut g, id, Some(Target::Permanent(c)), vec![]);
    // After attaching, you control 2 creatures + the aura enchantment = 3.
    let cp = g.computed_permanent(c).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "2/2 + 3 (2 creatures + 1 enchantment)");
}

/// Kudo sets other creatures' base P/T to 2/2 and makes them Bears, but leaves
/// itself alone.
#[test]
fn kudo_sets_base_pt_and_bear_type() {
    let mut g = two_player_game();
    let kudo = g.add_card_to_battlefield(0, catalog::kudo_king_among_bears());
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4 base
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 base
    // Other creatures become base 2/2 (counters/auras still layer on top).
    let a = g.computed_permanent(angel).unwrap();
    assert_eq!((a.power, a.toughness), (2, 2), "Serra Angel's base is now 2/2");
    assert!(a.subtypes.creature_types.contains(&crate::card::CreatureType::Bear), "and it's a Bear");
    // Applies across the table, not just your creatures.
    assert!(g.computed_permanent(enemy).unwrap().subtypes.creature_types
        .contains(&crate::card::CreatureType::Bear), "opponent's creature is a Bear too");
    // Kudo itself keeps its printed 2/2 and is unaffected by "other".
    assert_eq!(g.computed_permanent(kudo).unwrap().power, 2);
}

/// Drowner of Truth makes two Eldrazi Spawn only when {C} was spent to cast it.
#[test]
fn drowner_of_truth_colorless_spent_makes_spawn() {
    // Paid with a colorless {C} for one generic pip → spawn trigger fires.
    let mut g = two_player_game();
    g.players[0].mana_pool.add(crate::mana::Color::Green, 6);
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1); // the {C} that matters
    let id = g.add_card_to_hand(0, catalog::drowner_of_truth());
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast with {C}");
    drain_stack(&mut g);
    let spawns = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Eldrazi Spawn").count();
    assert_eq!(spawns, 2, "{{C}} spent → two Eldrazi Spawn");
}

#[test]
fn drowner_of_truth_no_colorless_no_spawn() {
    // Paid entirely with colored mana → no {C}, no spawn.
    let mut g = two_player_game();
    g.players[0].mana_pool.add(crate::mana::Color::Green, 7);
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    let id = g.add_card_to_hand(0, catalog::drowner_of_truth());
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast without {C}");
    drain_stack(&mut g);
    let spawns = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Eldrazi Spawn").count();
    assert_eq!(spawns, 0, "no {{C}} spent → no spawn");
}

/// Bespoke Battlewagon taps for {E}{E}, spends {E}{E} to tap a creature, and
/// spends {E}{E}{E}{E} to become a creature until end of turn.
#[test]
fn bespoke_battlewagon_energy_and_animate() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let wagon = g.add_card_to_battlefield(0, catalog::bespoke_battlewagon());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(wagon);
    // {T}: get {E}{E}
    g.perform_action(GameAction::ActivateAbility {
        card_id: wagon, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for energy");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 2, "two energy");
    // Pay {E}{E}{E}{E} to self-animate (no tap needed) — top up first.
    g.players[0].energy = 4;
    g.perform_action(GameAction::ActivateAbility {
        card_id: wagon, ability_index: 3, target: None, additional_targets: vec![], x_value: None,
    }).expect("pay 4 energy to animate");
    drain_stack(&mut g);
    assert!(g.computed_permanent(wagon).unwrap().card_types.contains(&crate::card::CardType::Creature),
        "Vehicle is now an artifact creature");
    assert_eq!(g.players[0].energy, 0, "spent all four energy");
    // A second wagon can tap a creature for {E}{E}.
    let w2 = g.add_card_to_battlefield(0, catalog::bespoke_battlewagon());
    g.clear_sickness(w2);
    g.players[0].energy = 2;
    g.perform_action(GameAction::ActivateAbility {
        card_id: w2, ability_index: 1, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
    }).expect("tap target creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "the bear is tapped");
}

/// Imskir Iron-Eater's ETB draws and loses X = half your artifacts, rounded
/// down. With four artifacts, X = 2.
#[test]
fn imskir_etb_draws_and_loses_half_artifacts() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_battlefield(0, catalog::ornithopter()); }
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let life = g.players[0].life;
    let lib = g.players[0].library.len();
    let imskir = g.add_card_to_hand(0, catalog::imskir_iron_eater());
    cast(&mut g, imskir, None, vec![]);
    assert_eq!(g.players[0].life, life - 2, "lose X=2 life");
    assert_eq!(g.players[0].library.len(), lib - 2, "drew X=2 cards");
}

/// Imskir's activated ability sacrifices an artifact and deals its mana value
/// as damage to any target.
#[test]
fn imskir_sac_artifact_deals_mana_value_damage() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let imskir = g.add_card_to_battlefield(0, catalog::imskir_iron_eater());
    let stone = g.add_card_to_battlefield(0, catalog::mind_stone()); // MV 2
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let opp = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: imskir, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("{3}{R}, sac an artifact");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 2, "MV-2 artifact deals 2 damage");
    assert!(g.battlefield_find(stone).is_none(), "the artifact was sacrificed");
}

/// Deem Inferior tucks target permanent second from the top of its owner's
/// library (owner chose "second from top").
#[test]
fn deem_inferior_tucks_second_from_top() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    use crate::game::types::Target;
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::grizzly_bears()); // library top exists
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let deem = g.add_card_to_hand(0, catalog::deem_inferior());
    cast(&mut g, deem, Some(Target::Permanent(victim)), vec![]);
    assert_eq!(g.players[1].library.get(1).map(|c| c.id), Some(victim),
        "tucked second from the top of owner's library");
}

/// Deem Inferior costs {1} less per card drawn this turn — after two draws it
/// is castable for {1}{U}.
#[test]
fn deem_inferior_cost_reduction_per_card_drawn() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    g.players[0].cards_drawn_this_turn = 2; // {3}{U} → {1}{U}
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1); // pays the reduced {1}
    let deem = g.add_card_to_hand(0, catalog::deem_inferior());
    g.perform_action(GameAction::CastSpell {
        card_id: deem, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for {1}{U} after two draws");
}

/// Snow-Covered Wastes is a snow land that taps for one colorless {C}.
#[test]
fn snow_covered_wastes_taps_for_colorless() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::snow_covered_wastes());
    assert!(g.battlefield.iter().find(|c| c.id == land).unwrap().definition.is_snow(), "is a snow land");
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("{T}: Add {C}");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1, "one colorless mana");
}

/// Path of Annihilation makes two Eldrazi Spawn on entry and gains 4 life when
/// you cast a mana-value-7-or-greater creature spell.
#[test]
fn path_of_annihilation_spawns_and_gains_life() {
    let mut g = two_player_game();
    let path = g.add_card_to_hand(0, catalog::path_of_annihilation());
    cast(&mut g, path, None, vec![]);
    let spawns = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Eldrazi Spawn").count();
    assert_eq!(spawns, 2, "ETB makes two Eldrazi Spawn");
    let life = g.players[0].life;
    let big = g.add_card_to_hand(0, catalog::drowner_of_truth()); // {5}{G/U}{G/U} = MV 7
    cast(&mut g, big, None, vec![]);
    assert_eq!(g.players[0].life, life + 4, "MV-7 creature cast gains 4 life");
}

/// Propagator Drone grants evolve to your creature tokens: a bigger creature
/// entering pumps an Eldrazi Spawn token (0/1) with a +1/+1 counter.
#[test]
fn propagator_drone_tokens_have_evolve() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::propagator_drone());
    let spawn = g.add_token_to_battlefield(0, &crabomination_base::tokens::eldrazi_spawn_token());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None, vec![]); // 2/2 > 0/1 → evolve on the token
    assert_eq!(g.battlefield.iter().find(|c| c.id == spawn).unwrap()
        .counter_count(CounterType::PlusOnePlusOne), 1, "token evolved");
}

/// Wumpus Aberration: cast with no {C} → the opponent may drop a creature from
/// hand onto the battlefield, and it enters under *their* control.
#[test]
fn wumpus_aberration_no_colorless_opponent_free_drops() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.players[0].mana_pool.add(crate::mana::Color::Green, 4); // all colored, no {C}
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    let id = g.add_card_to_hand(0, catalog::wumpus_aberration());
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast without {C}");
    drain_stack(&mut g);
    let dropped = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears");
    assert!(dropped.is_some(), "opponent put their creature onto the battlefield");
    assert_eq!(dropped.unwrap().controller, 1, "it enters under the opponent's control");
}

/// With {C} spent, Wumpus Aberration's cast trigger does nothing.
#[test]
fn wumpus_aberration_colorless_spent_no_drop() {
    let mut g = two_player_game();
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3); // pays generic with {C}
    let _bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::wumpus_aberration());
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast with {C}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Grizzly Bears"),
        "{{C}} spent → opponent gets no free drop");
}
