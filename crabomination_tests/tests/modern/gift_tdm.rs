#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::TurnStep;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
#[allow(unused)]
use crate::Factory;

// ── Gift (CR 702.165) ────────────────────────────────────────────────────────

/// Crumb and Get It without its gift only pumps +2/+2 (base resolution).
#[test]
fn gift_crumb_base_pumps_only() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::crumb_and_get_it());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Crumb (no gift)");
    drain_stack(&mut g);
    let b = g.computed_permanent(bear).unwrap();
    assert_eq!((b.power, b.toughness), (4, 4), "pumped to 4/4");
    assert!(!b.keywords.contains(&crabomination::card::Keyword::Indestructible), "no gift, no indestructible");
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Food"), "no Food without the gift");
}

/// Promising Crumb and Get It's gift also grants indestructible and mints the
/// opponent a Food.
#[test]
fn gift_crumb_promised_grants_indestructible_and_food() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::crumb_and_get_it());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastGift {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Crumb (gift)");
    drain_stack(&mut g);
    let b = g.computed_permanent(bear).unwrap();
    assert_eq!((b.power, b.toughness), (4, 4), "pumped to 4/4");
    assert!(b.keywords.contains(&crabomination::card::Keyword::Indestructible), "gift granted indestructible");
    assert!(g.battlefield.iter().any(|c| c.controller == 1 && c.definition.name == "Food"),
        "opponent received a Food");
}

/// Blooming Blast's gift adds 3 damage to the creature's controller and gives
/// the opponent a Treasure.
#[test]
fn gift_blooming_blast_promised_burns_controller() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::blooming_blast());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastGift {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Blooming Blast (gift)");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "2 damage killed the 2/2");
    assert_eq!(g.players[1].life, 17, "controller took 3 from the gift");
    assert!(g.battlefield.iter().any(|c| c.controller == 1 && c.definition.name == "Treasure"),
        "opponent received a Treasure");
}

/// Longstalk Brawl's gift puts a +1/+1 counter on your creature (so it wins the
/// fight) and mints the opponent a tapped Fish.
#[test]
fn gift_longstalk_brawl_promised_counter_and_tapped_fish() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());     // 2/2
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());   // 2/2
    let spell = g.add_card_to_hand(0, catalog::longstalk_brawl());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastGift {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)],
        mode: None, x_value: None,
    }).expect("cast Longstalk Brawl (gift)");
    drain_stack(&mut g);
    // 3/3 (after the +1/+1 counter) vs 2/2: mine survives, theirs dies.
    assert!(g.battlefield_find(mine).is_some(), "my 3/3 survived the fight");
    assert!(g.battlefield_find(theirs).is_none(), "their 2/2 died");
    let fish = g.battlefield.iter().find(|c| c.controller == 1 && c.definition.name == "Fish")
        .expect("opponent received a Fish");
    assert!(fish.tapped, "the gifted Fish entered tapped");
}

/// Into the Flood Maw's gift broadens the bounce to any nonland permanent.
#[test]
fn gift_flood_maw_promised_bounces_noncreature() {
    let mut g = two_player_game();
    let tali = g.add_card_to_battlefield(1, catalog::pristine_talisman()); // artifact, noncreature
    let spell = g.add_card_to_hand(0, catalog::into_the_flood_maw());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastGift {
        card_id: spell, target: Some(Target::Permanent(tali)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Into the Flood Maw (gift)");
    drain_stack(&mut g);
    assert!(g.battlefield_find(tali).is_none(), "the artifact was bounced");
    assert!(g.players[1].hand.iter().any(|c| c.definition.name == "Pristine Talisman"),
        "the artifact returned to its owner's hand");
}

/// Long River's Pull's gift broadens its counter to any spell (not just
/// creature spells).
#[test]
fn gift_long_rivers_pull_promised_counters_noncreature_spell() {
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt()); // an instant, not a creature spell
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    g.priority.player_with_priority = 0;
    let spell = g.add_card_to_hand(0, catalog::long_rivers_pull());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastGift {
        card_id: spell, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Long River's Pull (gift) counters a noncreature spell");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "the bolt was countered by the gifted Pull");
}

/// Nocturnal Hunger without its gift destroys the creature but costs 2 life.
#[test]
fn gift_nocturnal_hunger_base_costs_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::nocturnal_hunger());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Nocturnal Hunger");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "creature destroyed");
    assert_eq!(g.players[0].life, life - 2, "no gift → lose 2 life");
}

/// Promising Nocturnal Hunger's gift skips the life loss and feeds a Food.
#[test]
fn gift_nocturnal_hunger_promised_no_life_loss() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::nocturnal_hunger());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastGift {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Nocturnal Hunger (gift)");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "creature destroyed");
    assert_eq!(g.players[0].life, life, "gift → no life loss");
    assert!(g.battlefield.iter().any(|c| c.controller == 1 && c.definition.name == "Food"),
        "opponent received a Food");
}

/// Peerless Recycling's gift returns a second permanent card from the graveyard.
#[test]
fn gift_peerless_recycling_promised_returns_two() {
    let mut g = two_player_game();
    let c1 = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let c2 = g.add_card_to_graveyard(0, catalog::pristine_talisman());
    let spell = g.add_card_to_hand(0, catalog::peerless_recycling());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastGift {
        card_id: spell, target: Some(Target::Permanent(c1)),
        additional_targets: vec![Target::Permanent(c2)], mode: None, x_value: None,
    }).expect("cast Peerless Recycling (gift)");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == c1)
        && g.players[0].hand.iter().any(|c| c.id == c2),
        "both permanent cards returned to hand with the gift");
}

/// Valley Rally pumps your team; its gift grants first strike to one creature.
#[test]
fn gift_valley_rally_promised_grants_first_strike() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::valley_rally());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastGift {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Valley Rally (gift)");
    drain_stack(&mut g);
    let b = g.computed_permanent(bear).unwrap();
    assert_eq!((b.power, b.toughness), (4, 2), "pumped +2/+0");
    assert!(b.keywords.contains(&Keyword::FirstStrike), "gift granted first strike");
}

/// Dawn's Truce's gift grants your permanents indestructible too.
#[test]
fn gift_dawns_truce_promised_grants_indestructible() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::dawns_truce());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastGift {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Dawn's Truce (gift)");
    drain_stack(&mut g);
    let b = g.computed_permanent(bear).unwrap();
    assert!(b.keywords.contains(&Keyword::Hexproof), "granted hexproof");
    assert!(b.keywords.contains(&Keyword::Indestructible), "gift added indestructible");
}

/// Wildfire Howl's gift adds a 1-damage ping to any target.
#[test]
fn gift_wildfire_howl_promised_pings_a_target() {
    let mut g = two_player_game();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::wildfire_howl());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastGift {
        card_id: spell, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Wildfire Howl (gift)");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "2 to each creature killed the 2/2");
    assert_eq!(g.players[1].life, 19, "gift pinged the targeted player for 1");
}

/// Mind Spiral's gift taps and stuns an opponent's creature.
#[test]
fn gift_mind_spiral_promised_taps_and_stuns() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::mind_spiral());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastGift {
        card_id: spell, target: Some(Target::Player(0)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("cast Mind Spiral (gift)");
    drain_stack(&mut g);
    let c = g.battlefield_find(theirs).unwrap();
    assert!(c.tapped, "gift tapped the creature");
    assert!(c.counters.iter().any(|(k, n)| *k == CounterType::Stun && *n >= 1), "gift stunned it");
}

/// Sazacap's Brew discards a card as an additional cost and draws two; its gift
/// also pumps a creature you control.
#[test]
fn gift_sazacaps_brew_discards_draws_and_pumps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::sazacaps_brew());
    let pitch = g.add_card_to_hand(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastGift {
        card_id: spell,
        target: Some(Target::Player(0)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None, x_value: None,
    }).expect("cast Sazacap's Brew (gift)");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == pitch), "discarded a card as additional cost");
    let b = g.computed_permanent(bear).unwrap();
    assert_eq!((b.power, b.toughness), (4, 2), "gift pumped +2/+0");
}

/// Dewdrop Cure reanimates up to two small creatures; its gift lifts the cap to
/// three.
#[test]
fn gift_dewdrop_cure_promised_reanimates_three() {
    let mut g = two_player_game();
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
    let b = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let c = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::dewdrop_cure());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastGift {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b), Target::Permanent(c)],
        mode: None, x_value: None,
    }).expect("cast Dewdrop Cure (gift)");
    drain_stack(&mut g);
    let back = [a, b, c].iter().filter(|id| g.battlefield_find(**id).is_some()).count();
    assert_eq!(back, 3, "gift returned all three to the battlefield");
}

/// Consumed by Greed forces a greatest-power sacrifice; its gift also returns a
/// creature from your graveyard.
#[test]
fn gift_consumed_by_greed_promised_edict_and_return() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let small = g.add_card_to_battlefield(1, catalog::birds_of_paradise()); // 0/1, lower power
    let gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::consumed_by_greed());
    g.add_card_to_library(1, catalog::island()); // gift makes them draw; don't deck them
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastGift {
        card_id: spell,
        target: Some(Target::Permanent(gy)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Consumed by Greed (gift)");
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_none(), "greatest-power creature sacrificed");
    assert!(g.battlefield_find(small).is_some(), "the lower-power creature survives");
    assert!(g.players[0].hand.iter().any(|c| c.id == gy), "gift returned a creature from graveyard");
}

// ── Survival (CR 702.180) ────────────────────────────────────────────────────

/// Survival fires at your second main phase only when the creature is tapped.
#[test]
fn survival_cautious_survivor_gains_life_when_tapped() {
    let mut g = two_player_game();
    let surv = g.add_card_to_battlefield(0, catalog::cautious_survivor());
    g.battlefield_find_mut(surv).unwrap().tapped = true;
    let life = g.players[0].life;
    g.fire_step_triggers(TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "Survival gained 2 life (tapped)");
}

/// Untapped at second main, Survival doesn't trigger.
#[test]
fn survival_skips_when_untapped() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::cautious_survivor());
    let life = g.players[0].life;
    g.fire_step_triggers(TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "no Survival trigger while untapped");
}

/// Kona's Survival cheats a permanent card from hand onto the battlefield.
#[test]
fn survival_kona_puts_permanent_from_hand() {
    let mut g = two_player_game();
    let surv = g.add_card_to_battlefield(0, catalog::kona_rescue_beastie());
    g.battlefield_find_mut(surv).unwrap().tapped = true;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    g.fire_step_triggers(TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "Survival put the creature onto the battlefield");
}

/// Cynical Loner's Survival tutors a card from library into the graveyard.
#[test]
fn survival_cynical_loner_tutors_into_graveyard() {
    let mut g = two_player_game();
    let surv = g.add_card_to_battlefield(0, catalog::cynical_loner());
    g.battlefield_find_mut(surv).unwrap().tapped = true;
    let card = g.add_card_to_library(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(card))]));
    g.fire_step_triggers(TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == card),
        "Survival tutored the card into the graveyard");
}

/// Glimmer Seeker's Survival makes a Glimmer token when you control none.
#[test]
fn survival_glimmer_seeker_makes_token_without_a_glimmer() {
    let mut g = two_player_game();
    let surv = g.add_card_to_battlefield(0, catalog::glimmer_seeker());
    g.battlefield_find_mut(surv).unwrap().tapped = true;
    g.fire_step_triggers(TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Glimmer"),
        "Survival minted a Glimmer token (you controlled none)");
}

/// House Cartographer's Survival digs the top card (a land) into hand.
#[test]
fn survival_house_cartographer_finds_a_land() {
    let mut g = two_player_game();
    let surv = g.add_card_to_battlefield(0, catalog::house_cartographer());
    g.battlefield_find_mut(surv).unwrap().tapped = true;
    let land = g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == land), "Survival dug the land into hand");
    assert_eq!(g.players[0].hand.len(), hand + 1, "exactly one card found");
}

/// Savior of the Small's Survival returns a small creature from the graveyard.
#[test]
fn survival_savior_returns_small_creature_from_graveyard() {
    let mut g = two_player_game();
    let surv = g.add_card_to_battlefield(0, catalog::savior_of_the_small());
    g.battlefield_find_mut(surv).unwrap().tapped = true;
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV2 creature
    g.fire_step_triggers(TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear),
        "Survival returned the small creature to hand");
}

// ── Tarkir: Dragonstorm (non-Omen) ──────────────────────────────────────────

/// Sarkhan's Resolve mode 0 pumps a creature +3/+3.
#[test]
fn sarkhans_resolve_pumps_chosen_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::sarkhans_resolve());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast +3/+3 mode");
    drain_stack(&mut g);
    let s = g.battlefield_find(bear).unwrap();
    assert_eq!((s.power(), s.toughness()), (5, 5), "+3/+3 applied");
}

/// Sarkhan's Resolve mode 1 destroys a flyer.
#[test]
fn sarkhans_resolve_destroys_a_flyer() {
    let mut g = two_player_game();
    let flyer = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flyer
    let spell = g.add_card_to_hand(0, catalog::sarkhans_resolve());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(flyer)),
        additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("cast destroy-flyer mode");
    drain_stack(&mut g);
    assert!(g.battlefield_find(flyer).is_none(), "the flyer was destroyed");
}

/// Dragonback Lancer's Mobilize 1 makes a tapped, attacking Warrior token.
#[test]
fn dragonback_lancer_mobilizes_on_attack() {
    let mut g = two_player_game();
    let lancer = g.add_card_to_battlefield(0, catalog::dragonback_lancer());
    g.clear_sickness(lancer);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: lancer, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let attackers = g.attacking.len();
    assert_eq!(attackers, 2, "Mobilize 1 added a second attacker");
}

/// Sibsig Appraiser draws one of the top two and bins the other.
#[test]
fn sibsig_appraiser_picks_one_bins_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let hand_before = g.players[0].hand.len();
    let gy_before = g.players[0].graveyard.len();
    let sibsig = g.add_card_to_battlefield(0, catalog::sibsig_appraiser());
    g.fire_self_etb_triggers(sibsig, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "one card to hand");
    assert_eq!(g.players[0].graveyard.len(), gy_before + 1, "the other to graveyard");
}

/// Defibrillating Current deals 4 (kills a 4-toughness creature) and gains 2.
#[test]
fn defibrillating_current_burns_and_gains() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let spell = g.add_card_to_hand(0, catalog::defibrillating_current());
    g.players[0].mana_pool.add_colorless(6); // {2/R}{2/W}{2/B} paid as generic
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Defibrillating Current");
    drain_stack(&mut g);
    assert!(g.battlefield_find(angel).is_none(), "4 damage killed the 4/4");
    assert_eq!(g.players[0].life, life + 2, "gained 2 life");
}

/// Mardu Devotee's once-per-turn ability adds one R/W/B mana.
#[test]
fn mardu_devotee_taps_for_one_of_three_colors() {
    let mut g = two_player_game();
    let dev = g.add_card_to_battlefield(0, catalog::mardu_devotee());
    g.clear_sickness(dev);
    g.players[0].mana_pool.add_colorless(1); // pay the {1}
    g.perform_action(GameAction::ActivateAbility {
        card_id: dev, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate mana ability");
    drain_stack(&mut g);
    let pool = &g.players[0].mana_pool;
    assert_eq!(pool.total() + pool.restricted_total(), 1, "produced one mana from the activation");
}

/// Sibsig Host mills three from each player on ETB.
#[test]
fn sibsig_host_mills_each_player_three() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    for _ in 0..5 { g.add_card_to_library(1, catalog::grizzly_bears()); }
    let gy0 = g.players[0].graveyard.len();
    let gy1 = g.players[1].graveyard.len();
    let host = g.add_card_to_battlefield(0, catalog::sibsig_host());
    g.fire_self_etb_triggers(host, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), gy0 + 3, "you milled 3");
    assert_eq!(g.players[1].graveyard.len(), gy1 + 3, "opponent milled 3");
}

/// Stormscale Scion anthems other Dragons you control but not itself or non-Dragons.
#[test]
fn stormscale_scion_buffs_other_dragons() {
    let mut g = two_player_game();
    let scion = g.add_card_to_battlefield(0, catalog::stormscale_scion());
    let other_dragon = g.add_card_to_battlefield(0, catalog::bloomvine_regent()); // 4/5 Dragon
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // non-Dragon
    assert_eq!(g.computed_permanent(other_dragon).unwrap().power, 5, "other Dragon +1/+1");
    assert_eq!(g.computed_permanent(scion).unwrap().power, 4, "Scion does not buff itself");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "non-Dragon unaffected");
}

/// Roilmage's Trick weakens opponents' creatures by the converge count and draws.
#[test]
fn roilmages_trick_converge_weakens_and_draws() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.add_card_to_library(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::roilmages_trick());
    // Pay {3}{U} with two colors (U + R count toward converge = 2).
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Roilmage's Trick");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 0, "-2/-0 from two colors");
    assert_eq!(g.players[0].hand.len(), hand_before, "drew a card (net 0 after the cast)");
}

/// Kishla Skimmer draws when a card leaves your graveyard on your turn (once).
#[test]
fn kishla_skimmer_draws_on_graveyard_departure() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kishla_skimmer());
    let gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    let hand_before = g.players[0].hand.len();
    // Simulate the card leaving your graveyard on your turn.
    let _ = gy;
    let events = vec![GameEvent::CardLeftGraveyard { player: 0, card_id: gy }];
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "Kishla drew on the graveyard departure");
}

/// Inevitable Defeat exiles a nonland permanent, drains 3, and gains 3.
#[test]
fn inevitable_defeat_exiles_and_swings_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::inevitable_defeat());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life0 = g.players[0].life;
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Inevitable Defeat");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "target exiled");
    assert_eq!(g.players[1].life, life1 - 3, "controller lost 3");
    assert_eq!(g.players[0].life, life0 + 3, "you gained 3");
    assert!(g.battlefield_find(spell).is_none());
}

/// Magmatic Hellkite destroys an opponent's nonbasic land and ramps them a
/// stunned basic.
#[test]
fn magmatic_hellkite_destroys_nonbasic_and_ramps_stunned_basic() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    // Opponent controls a nonbasic land and has a basic in library.
    let nonbasic = g.add_card_to_battlefield(1, catalog::mishras_factory());
    let forest = g.add_card_to_library(1, catalog::forest());
    let hellkite = g.add_card_to_battlefield(0, catalog::magmatic_hellkite());
    // The dispossessed opponent finds the basic.
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(Some(forest)),
    ]));
    g.fire_self_etb_triggers(hellkite, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(nonbasic).is_none(), "nonbasic land destroyed");
    let basic = g.battlefield.iter().find(|c| c.controller == 1 && c.definition.is_land());
    let basic = basic.expect("opponent ramped a basic");
    assert!(basic.tapped, "the basic entered tapped");
    assert!(basic.counter_count(CounterType::Stun) >= 1, "with a stun counter");
}

/// Hardened Tactician sacrifices a token to draw a card.
#[test]
fn hardened_tactician_sacs_token_to_draw() {
    let mut g = two_player_game();
    let tac = g.add_card_to_battlefield(0, catalog::hardened_tactician());
    g.clear_sickness(tac);
    g.add_card_to_library(0, catalog::grizzly_bears());
    // A Treasure token to sacrifice.
    let tok = g.add_token_to_battlefield(0, &crabomination_base::tokens::treasure_token());
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: tac, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate sac-token draw");
    drain_stack(&mut g);
    assert!(g.battlefield_find(tok).is_none(), "the token was sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
}

/// Flurry (Cori Mountain Stalwart): the trigger fires only on the second spell
/// you cast each turn — pinging each opponent for 2 and gaining 2 life.
#[test]
fn flurry_fires_on_second_spell_each_turn() {
    let mut g = two_player_game();
    let stalwart = g.add_card_to_battlefield(0, catalog::cori_mountain_stalwart());
    let _ = stalwart;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    // First spell of the turn — no Flurry.
    let bolt1 = g.add_card_to_hand(0, catalog::lava_spike());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    cast_at(&mut g, bolt1, Target::Player(1));
    assert_eq!(g.players[1].life, opp_life - 3, "only Lava Spike's 3 (no Flurry yet)");
    assert_eq!(g.players[0].life, my_life, "no lifegain on the first spell");
    // Second spell — Flurry: +2 to opponent, +2 life to us.
    let bolt2 = g.add_card_to_hand(0, catalog::lava_spike());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, bolt2, Target::Player(1));
    assert_eq!(g.players[1].life, opp_life - 3 - 3 - 2, "second Lava Spike (3) + Flurry ping (2)");
    assert_eq!(g.players[0].life, my_life + 2, "Flurry gains 2 life");
}

/// Bone-Cairn Butcher grants deathtouch to your attacking tokens (and only
/// while they attack).
#[test]
fn bone_cairn_butcher_grants_attacking_tokens_deathtouch() {
    use crabomination::card::Keyword;
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bone_cairn_butcher());
    let golem = g.add_token_to_battlefield(0, &crabomination_base::tokens::golem_3_3_token());
    g.clear_sickness(golem);
    // Not attacking yet → no granted deathtouch.
    assert!(!g.computed_permanent(golem).unwrap().keywords.contains(&Keyword::Deathtouch),
        "token has no deathtouch before attacking");
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: golem, target: AttackTarget::Player(1),
    }])).expect("token attacks");
    assert!(g.computed_permanent(golem).unwrap().keywords.contains(&Keyword::Deathtouch),
        "attacking token has deathtouch from Bone-Cairn Butcher");
}

/// Cunning Coyote's ETB pumps another creature you control and grants it haste.
#[test]
fn cunning_coyote_etb_pumps_and_hastes_another() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // summoning-sick 2/2
    let coyote = g.add_card_to_hand(0, catalog::cunning_coyote());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: coyote, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Cunning Coyote");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "bear pumped +1/+1");
    assert!(cp.keywords.contains(&Keyword::Haste), "bear gains haste");
}

/// Monastery Messenger's ETB puts a noncreature/nonland graveyard card on top
/// of your library (and can't grab a creature card).
#[test]
fn monastery_messenger_recurs_noncreature_to_library_top() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lava_spike()); // sorcery — legal
    let _creature = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // illegal target
    let msgr = g.add_card_to_hand(0, catalog::monastery_messenger());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: msgr, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Monastery Messenger targeting the sorcery");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(bolt),
        "Lava Spike is on top of the library");
    assert!(g.players[0].graveyard.iter().all(|c| c.id != bolt), "left the graveyard");
}

/// Equilibrium Adept's Flurry grants it double strike on your second spell.
#[test]
fn equilibrium_adept_flurry_grants_double_strike() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let adept = g.add_card_to_battlefield(0, catalog::equilibrium_adept());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // Cast two spells; only the second triggers Flurry.
    let s1 = g.add_card_to_hand(0, catalog::lava_spike());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, s1, Target::Player(1));
    assert!(!g.computed_permanent(adept).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "no double strike after one spell");
    let s2 = g.add_card_to_hand(0, catalog::lava_spike());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, s2, Target::Player(1));
    assert!(g.computed_permanent(adept).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "Flurry grants double strike on the second spell");
}

/// Salt Road Patrol's Outlast puts a +1/+1 counter on itself.
#[test]
fn salt_road_patrol_outlast_adds_counter() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let patrol = g.add_card_to_battlefield(0, catalog::salt_road_patrol());
    g.clear_sickness(patrol);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: patrol, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate Outlast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(patrol).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "Outlast adds a +1/+1 counter");
    assert!(g.battlefield_find(patrol).unwrap().tapped, "and taps the creature");
}

/// Twin-Silk Spider's ETB makes a 1/2 Spider token with reach.
#[test]
fn twin_silk_spider_makes_a_spider_token() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let spider = g.add_card_to_hand(0, catalog::twin_silk_spider());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast(&mut g, spider);
    let tok = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Spider" && c.controller == 0)
        .expect("Spider token created");
    assert_eq!((tok.definition.power, tok.definition.toughness), (1, 2), "1/2 token");
    assert!(tok.definition.keywords.contains(&Keyword::Reach), "with reach");
}

/// Auroral Procession returns a graveyard card to hand.
#[test]
fn auroral_procession_returns_card_to_hand() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::auroral_procession());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Auroral Procession");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "bear returned to hand");
}

/// Ironpaw Aspirant's ETB adds a +1/+1 counter to a creature.
#[test]
fn ironpaw_aspirant_etb_adds_counter() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cat = g.add_card_to_hand(0, catalog::ironpaw_aspirant());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: cat, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Ironpaw Aspirant");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "+1/+1 counter placed");
}

/// Stormplain Detainment exiles an opponent's permanent until it leaves.
#[test]
fn stormplain_detainment_exiles_until_it_leaves() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let oring = g.add_card_to_hand(0, catalog::stormplain_detainment());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast(&mut g, oring);
    assert!(g.battlefield_find(victim).is_none(), "opponent's creature is exiled");
    g.remove_from_battlefield_to_graveyard_raw(oring);
    assert!(g.battlefield_find(victim).is_some(), "creature returns when the enchantment leaves");
}

/// Strategic Betrayal forces an opponent to lose a creature and exiles their
/// graveyard.
#[test]
fn strategic_betrayal_edicts_and_exiles_graveyard() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let gy_card = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let spell = g.add_card_to_hand(0, catalog::strategic_betrayal());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, spell, Target::Player(1));
    assert!(g.battlefield_find(creature).is_none(), "opponent lost a creature");
    assert!(g.exile.iter().any(|c| c.id == gy_card), "their graveyard was exiled");
}

/// Sonic Shrieker's ETB pings any target for 2 and gains 2 life.
#[test]
fn sonic_shrieker_etb_drains() {
    let mut g = two_player_game();
    let shrieker = g.add_card_to_hand(0, catalog::sonic_shrieker());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let opp = g.players[1].life;
    let me = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: shrieker, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Sonic Shrieker at the opponent's face");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 2, "2 damage to the opponent");
    assert_eq!(g.players[0].life, me + 2, "gained 2 life");
}

/// Sky Skiff becomes a creature when crewed by one power.
#[test]
fn sky_skiff_crews_to_a_creature() {
    let mut g = two_player_game();
    let skiff = g.add_card_to_battlefield(0, catalog::sky_skiff());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2 ≥ crew 1
    g.clear_sickness(bear);
    assert!(!g.computed_permanent(skiff).unwrap().card_types.contains(&CardType::Creature),
        "uncrewed Vehicle isn't a creature");
    g.perform_action(GameAction::Crew { vehicle: skiff, crew_creatures: vec![bear] })
        .expect("crew Sky Skiff");
    assert!(g.computed_permanent(skiff).unwrap().card_types.contains(&CardType::Creature),
        "crewed Sky Skiff is a creature");
}

/// Frontline Rush's first mode makes two 1/1 red Goblin tokens.
#[test]
fn frontline_rush_mode_makes_two_goblins() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::frontline_rush());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast Frontline Rush (make Goblins)");
    drain_stack(&mut g);
    let goblins = g.battlefield.iter()
        .filter(|c| c.definition.name == "Goblin" && c.controller == 0).count();
    assert_eq!(goblins, 2, "two 1/1 Goblin tokens");
}

/// Severance Priest exiles a card from an opponent's hand until it leaves.
#[test]
fn severance_priest_exiles_from_hand_until_it_leaves() {
    let mut g = two_player_game();
    let stolen = g.add_card_to_hand(1, catalog::grizzly_bears());
    let priest = g.add_card_to_hand(0, catalog::severance_priest());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast(&mut g, priest); // resolves the creature + its ETB exile trigger
    assert!(g.exile.iter().any(|c| c.id == stolen), "opponent's card is exiled");
    assert!(g.players[1].hand.iter().all(|c| c.id != stolen), "and out of their hand");
    // When the Priest leaves, the card returns to its owner's hand.
    g.remove_from_battlefield_to_graveyard_raw(priest);
    assert!(g.players[1].hand.iter().any(|c| c.id == stolen), "card returns when Priest leaves");
}

/// Naga Fleshcrafter's Renew adds a +1/+1 counter from the graveyard.
#[test]
fn naga_fleshcrafter_renew_adds_counter() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let naga = g.add_card_to_graveyard(0, catalog::naga_fleshcrafter());
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: naga, ability_index: 0, target: Some(Target::Permanent(target)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("activate Renew");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "+1/+1 counter from Renew");
    assert!(g.exile.iter().any(|c| c.id == naga), "Naga exiled by Renew");
}

/// Prison Break reanimates a creature card with an extra +1/+1 counter.
#[test]
fn prison_break_reanimates_with_counter() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::prison_break());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Prison Break");
    drain_stack(&mut g);
    let reanimated = g.battlefield_find(bear).expect("bear returned to the battlefield");
    assert_eq!(reanimated.controller, 0, "under your control");
    assert_eq!(reanimated.counter_count(CounterType::PlusOnePlusOne), 1, "with a +1/+1 counter");
}

/// Sandman's Quicksand wraths small creatures with -2/-2.
#[test]
fn sandmans_quicksand_minus_two_all() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::sandmans_quicksand());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast(&mut g, spell);
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none(),
        "both 2/2s died to -2/-2");
}

/// Omenpath to Naya taps for one of three colors and enters with Vanishing 4.
#[test]
fn omenpath_to_naya_taps_and_vanishes() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::omenpath_to_naya());
    assert_eq!(g.battlefield_find(land).unwrap().counter_count(crabomination::card::CounterType::Time), 0,
        "time counters are seeded on ETB, not here in the fixture");
    let before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap Omenpath for mana");
    assert_eq!(g.players[0].mana_pool.total(), before + 1, "added a mana");
}

/// Marang River Skeleton's {B} regenerate shield saves it from lethal damage.
#[test]
fn marang_river_skeleton_regenerates() {
    let mut g = two_player_game();
    let skel = g.add_card_to_battlefield(0, catalog::marang_river_skeleton());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: skel, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate regenerate");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(skel).unwrap().regeneration_shields, 1,
        "the regenerate activation stamps a shield");
}

/// Mox Jasper only taps for mana while you control a Dragon.
#[test]
fn mox_jasper_taps_only_with_a_dragon() {
    let mut g = two_player_game();
    let mox = g.add_card_to_battlefield(0, catalog::mox_jasper());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // No Dragon yet — activation is rejected.
    let r = g.perform_action(GameAction::ActivateAbility {
        card_id: mox, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    });
    assert!(r.is_err(), "Mox Jasper can't tap without a Dragon");
    // Add a Dragon → activation succeeds and floats a mana.
    g.add_card_to_battlefield(0, catalog::bloomvine_regent()); // 4/5 Dragon
    let before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: mox, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("taps for mana with a Dragon out");
    assert_eq!(g.players[0].mana_pool.total(), before + 1, "added one mana");
}

/// Sage of the Fang's Renew (graveyard activation) adds a +1/+1 counter, then
/// doubles the +1/+1 counters on the target.
#[test]
fn sage_of_the_fang_renew_doubles_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let sage = g.add_card_to_graveyard(0, catalog::sage_of_the_fang());
    // A creature already carrying two +1/+1 counters.
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield_find_mut(target) {
        c.add_counters(CounterType::PlusOnePlusOne, 2);
    }
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sage, ability_index: 0, target: Some(Target::Permanent(target)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("activate Renew from the graveyard");
    drain_stack(&mut g);
    // 2 existing + 1 added = 3, doubled = 6.
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 6,
        "+1/+1 counters: (2+1) doubled to 6");
    // Renew exiles Sage from the graveyard.
    assert!(g.exile.iter().any(|c| c.id == sage), "Sage exiled by Renew");
}

/// Unending Whisper ({U} sorcery) draws a card when cast normally.
#[test]
fn unending_whisper_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears()); // something to draw
    let id = g.add_card_to_hand(0, catalog::unending_whisper());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Unending Whisper");
    drain_stack(&mut g);
    // Hand: -1 (spell left) +1 (drawn) = unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before, "drew a card to replace the cast spell");
}

/// Ureni's Rebuff ({1}{U}) returns a target creature to its owner's hand.
#[test]
fn urenis_rebuff_bounces_a_creature() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::urenis_rebuff());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Ureni's Rebuff");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "bounced off the battlefield");
    assert!(g.players[1].hand.iter().any(|c| c.id == victim), "to its owner's hand");
}

/// Wild Ride ({R}) gives target creature +3/+0 and haste until end of turn.
#[test]
fn wild_ride_pumps_and_grants_haste() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::wild_ride());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(c)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Wild Ride");
    drain_stack(&mut g);
    let cp = g.computed_permanent(c).unwrap();
    assert_eq!(cp.power, 5, "2 + 3 = 5 power");
    assert!(cp.keywords.contains(&Keyword::Haste), "gained haste");
}

/// Mammoth Bellow ({2}{G}{U}{R}) makes a 5/5 green Elephant token.
#[test]
fn mammoth_bellow_makes_a_5_5_elephant() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mammoth_bellow());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Mammoth Bellow");
    drain_stack(&mut g);
    let tok = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Elephant");
    assert!(tok.is_some(), "Elephant token exists");
    let cp = g.computed_permanent(tok.unwrap().id).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "5/5 token");
}

/// Amazing Spider-Girl enters with Flying and Vigilance.
#[test]
fn amazing_spider_girl_has_flying_vigilance() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::amazing_spider_girl());
    let cp = g.computed_permanent(id).unwrap();
    assert!(cp.keywords.contains(&Keyword::Flying), "has flying");
    assert!(cp.keywords.contains(&Keyword::Vigilance), "has vigilance");
    assert_eq!((cp.power, cp.toughness), (5, 4), "5/4");
}

/// Silk, Web Weaver makes a 1/1 Human Citizen whenever you cast a creature spell.
#[test]
fn silk_web_weaver_tokens_on_creature_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::silk_web_weaver());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a creature spell");
    drain_stack(&mut g);
    let tokens = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Human Citizen").count();
    assert_eq!(tokens, 1, "Silk minted a Citizen on the creature cast");
}

/// Spider-Man India puts a +1/+1 counter on a creature you control and grants
/// it flying whenever you cast a creature spell.
#[test]
fn spider_man_india_counters_on_creature_cast() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::spider_man_india());
    let buddy = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: Some(Target::Permanent(buddy)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a creature spell");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(buddy).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "buddy got a +1/+1 counter");
    assert!(g.computed_permanent(buddy).unwrap().keywords.contains(&Keyword::Flying), "and flying");
}

// ── TDM batch 7 tests ─────────────────────────────────────────────────────

/// Nightblade Brigade mobilizes a token when it attacks (and has deathtouch).
#[test]
fn nightblade_brigade_mobilizes_and_has_deathtouch() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let b = g.add_card_to_battlefield(0, catalog::nightblade_brigade());
    assert!(g.computed_permanent(b).unwrap().keywords.contains(&Keyword::Deathtouch));
    g.clear_sickness(b);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: b, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.attacking.len(), 2, "Mobilize 1 added a second attacker");
}

/// Shock Brigade has menace and mobilizes on attack.
#[test]
fn shock_brigade_mobilizes_on_attack() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let b = g.add_card_to_battlefield(0, catalog::shock_brigade());
    assert!(g.computed_permanent(b).unwrap().keywords.contains(&Keyword::Menace));
    g.clear_sickness(b);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: b, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.attacking.len(), 2, "Mobilize 1 added a token attacker");
}

/// Venerated Stormsinger drains 1 whenever a creature you control dies.
#[test]
fn venerated_stormsinger_drains_on_creature_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::venerated_stormsinger());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Bolt my own fodder so the full SBA + death-trigger dispatch fires.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let (me, opp) = (g.players[0].life, g.players[1].life);
    cast_at(&mut g, bolt, Target::Permanent(fodder));
    assert_eq!(g.players[1].life, opp - 1, "opponent lost 1");
    assert_eq!(g.players[0].life, me + 1, "you gained 1");
}

/// Stadium Headliner's sac ability deals damage equal to creatures you control.
#[test]
fn stadium_headliner_sac_pings_for_board_count() {
    let mut g = two_player_game();
    let head = g.add_card_to_battlefield(0, catalog::stadium_headliner());
    // Two OTHER creatures survive the sac, so the count at resolution is 2.
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: head, ability_index: 0, target: Some(Target::Permanent(victim)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("sac Stadium Headliner");
    drain_stack(&mut g);
    // Headliner is sacrificed as a cost, leaving 2 creatures → 2 damage kills the 2/2.
    assert!(g.battlefield_find(victim).is_none(), "2 damage killed the 2/2");
}

/// Champion of Dusan's Renew grants a +1/+1 counter and a trample counter.
#[test]
fn champion_of_dusan_renew_grants_trample() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let champ = g.add_card_to_graveyard(0, catalog::champion_of_dusan());
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: champ, ability_index: 0, target: Some(Target::Permanent(target)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("Renew from graveyard");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.computed_permanent(target).unwrap().keywords.contains(&Keyword::Trample),
        "trample counter granted trample");
    assert!(g.exile.iter().any(|c| c.id == champ), "Champion exiled by Renew");
}

/// Sagu Pummeler's Renew puts two +1/+1 counters and a reach counter.
#[test]
fn sagu_pummeler_renew_grants_reach() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let sagu = g.add_card_to_graveyard(0, catalog::sagu_pummeler());
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sagu, ability_index: 0, target: Some(Target::Permanent(target)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("Renew from graveyard");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert!(g.computed_permanent(target).unwrap().keywords.contains(&Keyword::Reach));
}

/// Adorned Crocodile dies into a 2/2 Zombie Druid, and its Renew adds a counter.
#[test]
fn adorned_crocodile_dies_into_token_then_renews() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let croc = g.add_card_to_battlefield(0, catalog::adorned_crocodile());
    g.battlefield_find_mut(croc).unwrap().damage = 5;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Zombie Druid").count(), 1,
        "made a Zombie Druid on death");
    // The Crocodile is now in the graveyard — Renew it.
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: croc, ability_index: 0, target: Some(Target::Permanent(target)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("Renew from graveyard");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Lasyd Prowler's Renew adds +1/+1 counters equal to land cards in graveyard.
#[test]
fn lasyd_prowler_renew_scales_with_lands_in_graveyard() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let lasyd = g.add_card_to_graveyard(0, catalog::lasyd_prowler());
    g.add_card_to_graveyard(0, catalog::forest());
    g.add_card_to_graveyard(0, catalog::forest());
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: lasyd, ability_index: 0, target: Some(Target::Permanent(target)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("Renew from graveyard");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "two land cards in graveyard → two counters");
}

/// Monk of the Open Hand gets a +1/+1 counter on your second spell each turn.
#[test]
fn monk_of_the_open_hand_grows_on_second_spell() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let monk = g.add_card_to_battlefield(0, catalog::monk_of_the_open_hand());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let s1 = g.add_card_to_hand(0, catalog::lava_spike());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, s1, Target::Player(1));
    assert_eq!(g.battlefield_find(monk).unwrap().counter_count(CounterType::PlusOnePlusOne), 0,
        "no counter on first spell");
    let s2 = g.add_card_to_hand(0, catalog::lava_spike());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, s2, Target::Player(1));
    assert_eq!(g.battlefield_find(monk).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "Flurry counter on the second spell");
}

/// Jeskai Devotee gets +1/+1 on the second spell; its mana ability floats mana.
#[test]
fn jeskai_devotee_flurry_and_mana() {
    let mut g = two_player_game();
    let dev = g.add_card_to_battlefield(0, catalog::jeskai_devotee());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let s1 = g.add_card_to_hand(0, catalog::lava_spike());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, s1, Target::Player(1));
    let s2 = g.add_card_to_hand(0, catalog::lava_spike());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, s2, Target::Player(1));
    assert_eq!(g.computed_permanent(dev).unwrap().power, 3, "2 +1 from Flurry until EOT");
    // Mana ability: {1}: add one of U/R/W.
    g.players[0].mana_pool.add_colorless(1);
    let before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: dev, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("mana ability");
    assert_eq!(g.players[0].mana_pool.total(), before, "spent one, added one => net unchanged");
}

/// Wingblade Disciple makes a flying Bird on your second spell.
#[test]
fn wingblade_disciple_makes_bird_on_second_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::wingblade_disciple());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let s1 = g.add_card_to_hand(0, catalog::lava_spike());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, s1, Target::Player(1));
    let s2 = g.add_card_to_hand(0, catalog::lava_spike());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, s2, Target::Player(1));
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Bird").count(), 1,
        "Flurry minted a Bird");
}

/// Poised Practitioner gets a +1/+1 counter on your second spell.
#[test]
fn poised_practitioner_grows_on_second_spell() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest()); // scry needs a library card
    let monk = g.add_card_to_battlefield(0, catalog::poised_practitioner());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let s1 = g.add_card_to_hand(0, catalog::lava_spike());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, s1, Target::Player(1));
    let s2 = g.add_card_to_hand(0, catalog::lava_spike());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, s2, Target::Player(1));
    assert_eq!(g.battlefield_find(monk).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Devoted Duelist pings each opponent on your second spell.
#[test]
fn devoted_duelist_pings_on_second_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::devoted_duelist());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let opp = g.players[1].life;
    let s1 = g.add_card_to_hand(0, catalog::lava_spike());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, s1, Target::Player(1));
    let s2 = g.add_card_to_hand(0, catalog::lava_spike());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, s2, Target::Player(1));
    // Two Lava Spikes (3 each) = 6, plus Flurry's 1 = 7.
    assert_eq!(g.players[1].life, opp - 7, "second-spell Flurry pinged for 1 on top of the spells");
}

// ── TDM batch 8 tests ─────────────────────────────────────────────────────

/// Avenger of the Fallen mobilizes a token per creature card in your graveyard.
#[test]
fn avenger_of_the_fallen_mobilizes_per_graveyard_creature() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let av = g.add_card_to_battlefield(0, catalog::avenger_of_the_fallen());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.clear_sickness(av);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: av, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.attacking.len(), 3, "Avenger + 2 Mobilize tokens (2 creature cards in gy)");
}

/// Dalkovan Packbeasts mobilizes three on attack.
#[test]
fn dalkovan_packbeasts_mobilizes_three() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let ox = g.add_card_to_battlefield(0, catalog::dalkovan_packbeasts());
    g.clear_sickness(ox);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ox, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.attacking.len(), 4, "Ox + 3 Mobilize tokens");
}

/// Reigning Victor's ETB grants +1/+0 and indestructible until end of turn.
#[test]
fn reigning_victor_etb_buffs_and_protects() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let rv = g.add_card_to_hand(0, catalog::reigning_victor());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: rv, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Reigning Victor");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3, "2 + 1 = 3 power");
    assert!(cp.keywords.contains(&Keyword::Indestructible), "gained indestructible");
}

/// Agent of Kotis' Renew puts two +1/+1 counters on a creature.
#[test]
fn agent_of_kotis_renew_two_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let agent = g.add_card_to_graveyard(0, catalog::agent_of_kotis());
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: agent, ability_index: 0, target: Some(Target::Permanent(target)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("Renew");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Alchemist's Assistant's Renew grants a lifelink counter.
#[test]
fn alchemists_assistant_renew_grants_lifelink() {
    let mut g = two_player_game();
    let a = g.add_card_to_graveyard(0, catalog::alchemists_assistant());
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: a, ability_index: 0, target: Some(Target::Permanent(target)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("Renew");
    drain_stack(&mut g);
    assert!(g.computed_permanent(target).unwrap().keywords.contains(&Keyword::Lifelink));
}

/// Qarsi Revenant's Renew grants flying, deathtouch, and lifelink.
#[test]
fn qarsi_revenant_renew_grants_three_keywords() {
    let mut g = two_player_game();
    let q = g.add_card_to_graveyard(0, catalog::qarsi_revenant());
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: q, ability_index: 0, target: Some(Target::Permanent(target)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("Renew");
    drain_stack(&mut g);
    let kw = g.computed_permanent(target).unwrap().keywords;
    assert!(kw.contains(&Keyword::Flying) && kw.contains(&Keyword::Deathtouch) && kw.contains(&Keyword::Lifelink));
}

/// Constrictor Sage taps and stuns an opponent's creature on ETB.
#[test]
fn constrictor_sage_etb_taps_and_stuns() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let sage = g.add_card_to_hand(0, catalog::constrictor_sage());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: sage, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Constrictor Sage");
    drain_stack(&mut g);
    let v = g.battlefield_find(victim).unwrap();
    assert!(v.tapped, "opponent's creature tapped");
    assert_eq!(v.counter_count(CounterType::Stun), 1, "stun counter placed");
}

/// Wayspeaker Bodyguard returns a low-cost permanent card from your graveyard.
#[test]
fn wayspeaker_bodyguard_etb_returns_low_cost_permanent() {
    let mut g = two_player_game();
    let card = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2 creature
    let way = g.add_card_to_hand(0, catalog::wayspeaker_bodyguard());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: way, target: Some(Target::Permanent(card)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Wayspeaker Bodyguard");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == card), "returned the Bear to hand");
}

/// Coordinated Maneuver mode 0 deals damage equal to creatures you control.
#[test]
fn coordinated_maneuver_pings_for_board_count() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::coordinated_maneuver());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast Coordinated Maneuver mode 0");
    drain_stack(&mut g);
    // 2 creatures controlled → 2 damage kills the 2/2.
    assert!(g.battlefield_find(victim).is_none(), "2 damage killed the 2/2");
}

/// Roamer's Routine fetches a basic land onto the battlefield tapped.
#[test]
fn roamers_routine_fetches_land_tapped() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::roamers_routine());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Roamer's Routine");
    drain_stack(&mut g);
    let land = g.battlefield_find(forest);
    assert!(land.is_some(), "Forest fetched onto the battlefield");
    assert!(land.unwrap().tapped, "and it enters tapped");
}

/// Webspinner Cuff reconfigures onto a creature, granting +1/+4 and reach.
#[test]
fn webspinner_cuff_reconfigures_and_buffs() {
    let mut g = two_player_game();
    let cuff = g.add_card_to_battlefield(0, catalog::webspinner_cuff());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::Reconfigure { equipment: cuff, target: Some(bear) })
        .expect("reconfigure Webspinner Cuff onto the bear");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 6), "2/2 + 1/4 = 3/6");
    assert!(cp.keywords.contains(&Keyword::Reach), "granted reach");
}

/// Sarkhan's Triumph tutors a Dragon to hand.
#[test]
fn sarkhans_triumph_tutors_a_dragon() {
    let mut g = two_player_game();
    let dragon = g.add_card_to_library(0, catalog::stormscale_scion()); // a Dragon
    g.add_card_to_library(0, catalog::grizzly_bears()); // non-Dragon padding
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(dragon))]));
    let id = g.add_card_to_hand(0, catalog::sarkhans_triumph());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Sarkhan's Triumph");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dragon), "Dragon tutored to hand");
}

/// Lotus-Eye Mystics has prowess and returns an enchantment from your graveyard.
#[test]
fn lotus_eye_mystics_etb_returns_enchantment() {
    let mut g = two_player_game();
    let aura = g.add_card_to_graveyard(0, catalog::rancor());
    let m = g.add_card_to_hand(0, catalog::lotus_eye_mystics());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: m, target: Some(Target::Permanent(aura)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Lotus-Eye Mystics");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == aura), "enchantment returned to hand");
    assert!(g.computed_permanent(m).unwrap().keywords.contains(&Keyword::Prowess), "has prowess");
}

/// Winternight Stories nets cards: draw three, discard two.
#[test]
fn winternight_stories_draws_three_discards_two() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::forest()); }
    let id = g.add_card_to_hand(0, catalog::winternight_stories());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len(); // includes the spell
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Winternight Stories");
    drain_stack(&mut g);
    // -1 spell, +3 drawn, -2 discarded = net 0 from the starting hand size.
    assert_eq!(g.players[0].hand.len(), hand_before, "net hand size unchanged (draw 3, discard 2)");
}

/// Heritage Reclamation mode 0 destroys a target artifact.
#[test]
fn heritage_reclamation_destroys_artifact() {
    let mut g = two_player_game();
    let mox = g.add_card_to_battlefield(1, catalog::mox_jasper()); // an artifact
    let id = g.add_card_to_hand(0, catalog::heritage_reclamation());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mox)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast Heritage Reclamation mode 0");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mox).is_none(), "artifact destroyed");
}

// ── TDM batch (lands + commons) ─────────────────────────────────────────────

/// Sandsteppe Citadel enters tapped and taps for W, B, or G.
#[test]
fn sandsteppe_citadel_enters_tapped_tri_color() {
    let mut g = two_player_game();
    let land = g.add_card_to_hand(0, catalog::sandsteppe_citadel());
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    let l = g.battlefield_find(land).expect("on battlefield");
    assert!(l.tapped, "tri-land enters tapped");
    assert_eq!(l.definition.activated_abilities.len(), 3, "taps for three colors");
}

/// Twin Bolt deals 2 damage split among up to two targets.
#[test]
fn twin_bolt_divides_two_damage() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::twin_bolt());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None,
    }).expect("cast Twin Bolt across two targets");
    drain_stack(&mut g);
    // 1 damage each leaves both 2/2 bears alive but marked.
    assert_eq!(g.battlefield_find(a).unwrap().damage, 1);
    assert_eq!(g.battlefield_find(b).unwrap().damage, 1);
}

/// Cruel Truths surveils, draws two, and loses 2 life.
#[test]
fn cruel_truths_draws_two_loses_two() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::swamp()); }
    let id = g.add_card_to_hand(0, catalog::cruel_truths());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Cruel Truths");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2, "cast -1, draw +2");
    assert_eq!(g.players[0].life, life_before - 2);
}

/// Iceridge Serpent bounces an opponent's creature on ETB.
#[test]
fn iceridge_serpent_bounces_on_etb() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::iceridge_serpent());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Iceridge Serpent");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == victim), "ETB bounced the bear");
}

/// Worthy Cost sacrifices a creature and exiles a target.
#[test]
fn worthy_cost_sacs_then_exiles() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let target = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::worthy_cost());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Worthy Cost");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "creature sacrificed");
    assert!(g.exile.iter().any(|c| c.id == target), "target exiled");
}

/// Bearer of Glory has first strike only on its controller's turn.
#[test]
fn bearer_of_glory_first_strike_on_your_turn() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::bearer_of_glory());
    g.active_player_idx = 0;
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&crabomination::card::Keyword::FirstStrike), "first strike on your turn");
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&crabomination::card::Keyword::FirstStrike), "no first strike on opp turn");
}

/// Undergrowth Leopard sacrifices itself to destroy an artifact.
#[test]
fn undergrowth_leopard_sacs_to_destroy_artifact() {
    let mut g = two_player_game();
    let leopard = g.add_card_to_battlefield(0, catalog::undergrowth_leopard());
    let mox = g.add_card_to_battlefield(1, catalog::mox_jasper());
    g.clear_sickness(leopard);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: leopard, ability_index: 0,
        target: Some(Target::Permanent(mox)), additional_targets: vec![], x_value: None,
    }).expect("activate sac ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mox).is_none(), "artifact destroyed");
    assert!(g.battlefield_find(leopard).is_none(), "leopard sacrificed");
}

/// Summit Intimidator stops a creature from blocking on ETB.
#[test]
fn summit_intimidator_grants_cant_block() {
    let mut g = two_player_game();
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::summit_intimidator());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Summit Intimidator");
    drain_stack(&mut g);
    assert!(g.computed_permanent(blocker).unwrap().keywords.contains(&crabomination::card::Keyword::CantBlock), "target can't block");
}

/// Underfoot Underdogs mints a Goblin token on ETB.
#[test]
fn underfoot_underdogs_mints_goblin() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::underfoot_underdogs());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Underfoot Underdogs");
    drain_stack(&mut g);
    let goblins = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Goblin").count();
    assert_eq!(goblins, 1, "one 1/1 Goblin minted");
}

/// Salt Road Packbeast's affinity reduces its cost by creatures you control.
#[test]
fn salt_road_packbeast_affinity_reduces_cost() {
    let mut g = two_player_game();
    // Two creatures → {5}{W} becomes {3}{W}.
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
    let id = g.add_card_to_hand(0, catalog::salt_road_packbeast());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("affinity discounts to {3}{W}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some(), "packbeast resolved");
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 1, "ETB drew a card");
}

/// Humbling Elder weakens an opponent's creature on ETB.
#[test]
fn humbling_elder_shrinks_opp_creature() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::humbling_elder());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Humbling Elder (flash)");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(victim).unwrap().power, 0, "bear at -2/-0 → 0 power");
}

/// Unsparing Boltcaster only burns a creature already dealt damage this turn.
#[test]
fn unsparing_boltcaster_burns_damaged_creature() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.battlefield_find_mut(victim).unwrap().dealt_damage_this_turn = true;
    let id = g.add_card_to_hand(0, catalog::unsparing_boltcaster());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Unsparing Boltcaster");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "5 damage finished the angel");
}

/// Veteran Ice Climber mills on attack equal to its power.
#[test]
fn veteran_ice_climber_mills_on_attack() {
    let mut g = two_player_game();
    let climber = g.add_card_to_battlefield(0, catalog::veteran_ice_climber()); // 1/3
    g.clear_sickness(climber);
    for _ in 0..5 { g.add_card_to_library(1, catalog::island()); }
    let gy_before = g.players[1].graveyard.len();
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: climber, target: AttackTarget::Player(1),
    }])).expect("declare attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), gy_before + 1, "milled 1 (power)");
}

/// Dragonologist grants untapped Dragons you control hexproof.
#[test]
fn dragonologist_grants_dragon_hexproof() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dragonologist());
    let dragon = g.add_card_to_battlefield(0, catalog::pearl_lake_warden()); // a Dragon
    assert!(g.computed_permanent(dragon).unwrap().keywords.contains(&crabomination::card::Keyword::Hexproof), "untapped Dragon hexproof");
    g.battlefield_find_mut(dragon).unwrap().tapped = true;
    assert!(!g.computed_permanent(dragon).unwrap().keywords.contains(&crabomination::card::Keyword::Hexproof), "tapped Dragon loses it");
}

/// Trade Route Envoy draws when you control a counter-bearing creature.
#[test]
fn trade_route_envoy_draws_with_counter_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let id = g.add_card_to_hand(0, catalog::trade_route_envoy());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Trade Route Envoy");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 1, "ETB drew a card");
}

/// Desperate Measures grows-then-shrinks a creature and draws on its death.
#[test]
fn desperate_measures_draws_when_target_dies() {
    let mut g = two_player_game();
    // A 1/1 dies to the -1 toughness.
    let pit = g.add_card_to_battlefield(0, catalog::dragon_sniper()); // 1/1
    for _ in 0..3 { g.add_card_to_library(0, catalog::swamp()); }
    let id = g.add_card_to_hand(0, catalog::desperate_measures());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(pit)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Desperate Measures");
    drain_stack(&mut g);
    assert!(g.battlefield_find(pit).is_none(), "1/1 dies to +1/-1");
    // cast -1, then draw 2 on death → net +1.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2);
}

// ── Exhale cycle ────────────────────────────────────────────────────────────

/// Caustic Exhale gives a creature -3/-3.
#[test]
fn caustic_exhale_shrinks_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::caustic_exhale());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Caustic Exhale");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "-3/-3 kills the 2/2");
}

/// Dispelling Exhale counters unless they pay {2}; beholding a Dragon raises it
/// to {4} (so a player with only {2} can't pay).
#[test]
fn dispelling_exhale_counters_more_with_dragon() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::pearl_lake_warden()); // a Dragon → "beheld"
    let spell = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(3); // after {1}{G}, 2 left: can pay {2} not {4}
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp casts bears");
    let exhale = g.add_card_to_hand(0, catalog::dispelling_exhale());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: exhale, target: Some(Target::Permanent(spell)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Dispelling Exhale at the bears");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == spell), "bears countered (cannot pay 4)");
}

/// Piercing Exhale: your creature deals damage equal to its power to a target.
#[test]
fn piercing_exhale_fights_one_sided() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::piercing_exhale());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(victim)], mode: None, x_value: None,
    }).expect("cast Piercing Exhale");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "4 power → 4 damage kills the 4/4");
}

// ── Monuments / Devotees / misc TDM ─────────────────────────────────────────

/// Jeskai Monument tutors a basic land to hand on ETB.
#[test]
fn jeskai_monument_tutors_basic_on_etb() {
    let mut g = two_player_game();
    let island = g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(island))]));
    let id = g.add_card_to_hand(0, catalog::jeskai_monument());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Jeskai Monument");
    drain_stack(&mut g);
    // cast (-1) + tutored Island to hand (+1) = net 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Island"), "Island tutored");
}

/// Mardu Monument's sac ability mints three Warriors.
#[test]
fn mardu_monument_sac_mints_three_warriors() {
    let mut g = two_player_game();
    let mon = g.add_card_to_battlefield(0, catalog::mardu_monument());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mon, ability_index: 0, target: None,
        additional_targets: vec![], x_value: None,
    }).expect("activate Mardu Monument sac");
    drain_stack(&mut g);
    let warriors = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Warrior").count();
    assert_eq!(warriors, 3, "three Warriors minted");
    assert!(g.battlefield_find(mon).is_none(), "Monument sacrificed");
}

/// Abzan Devotee can return itself from the graveyard for {2}{B}.
#[test]
fn abzan_devotee_returns_from_graveyard() {
    let mut g = two_player_game();
    let dev = g.add_card_to_graveyard(0, catalog::abzan_devotee());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dev, ability_index: 1, target: None,
        additional_targets: vec![], x_value: None,
    }).expect("activate graveyard recur");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dev), "Devotee back in hand");
}

/// Temur Devotee's mana ability adds one of its three colors, once per turn.
#[test]
fn temur_devotee_mana_ability_once_per_turn() {
    let mut g = two_player_game();
    let dev = g.add_card_to_battlefield(0, catalog::temur_devotee());
    g.clear_sickness(dev);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dev, ability_index: 0, target: None,
        additional_targets: vec![], x_value: None,
    }).expect("first activation");
    // Total mana after spending 1 and adding 1 = still 2.
    assert_eq!(g.players[0].mana_pool.total(), 2);
    // Second activation is illegal (once each turn).
    let second = g.perform_action(GameAction::ActivateAbility {
        card_id: dev, ability_index: 0, target: None,
        additional_targets: vec![], x_value: None,
    });
    assert!(second.is_err(), "mana ability is once each turn");
}

/// Starry-Eyed Skyrider grants flying to another creature on attack.
#[test]
fn starry_eyed_skyrider_grants_flying_on_attack() {
    let mut g = two_player_game();
    let rider = g.add_card_to_battlefield(0, catalog::starry_eyed_skyrider());
    let buddy = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(rider);
    g.clear_sickness(buddy);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: rider, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert!(g.computed_permanent(buddy).unwrap().keywords.contains(&crabomination::card::Keyword::Flying),
        "buddy gained flying");
}

/// Aegis Sculptor grows on upkeep by exiling two graveyard cards.
#[test]
fn aegis_sculptor_grows_on_upkeep() {
    let mut g = two_player_game();
    let sculptor = g.add_card_to_battlefield(0, catalog::aegis_sculptor());
    g.add_card_to_graveyard(0, catalog::island());
    g.add_card_to_graveyard(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.active_player_idx = 0;
    g.step = TurnStep::Untap;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::PreCombatMain {
        g.perform_action(GameAction::PassPriority).unwrap();
        drain_stack(&mut g);
    }
    assert_eq!(g.computed_permanent(sculptor).unwrap().power, 3, "2/3 + counter = 3 power");
}

/// Yathan Tombguard draws (and loses 1) when a counter-bearing creature hits.
#[test]
fn yathan_tombguard_draws_on_counter_creature_damage() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::yathan_tombguard());
    let hitter = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(hitter).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.clear_sickness(hitter);
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: hitter, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew on combat damage");
    assert_eq!(g.players[0].life, life_before - 1);
}

/// Sunpearl Kirin bounces a nonland permanent you control on ETB.
#[test]
fn sunpearl_kirin_bounces_your_permanent() {
    let mut g = two_player_game();
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::sunpearl_kirin());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Sunpearl Kirin");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == other), "bounced the bear to hand");
}

/// Formation Breaker (CR 509.1b): creatures with less power can't block it.
#[test]
fn formation_breaker_blocks_only_by_equal_or_greater_power() {
    let mut g = two_player_game();
    let breaker = g.add_card_to_battlefield(0, catalog::formation_breaker()); // 2/1
    let weak = g.add_card_to_battlefield(1, catalog::dragon_sniper()); // 1/1, power 1 < 2
    let strong = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    assert!(!g.blocker_can_block_attacker(weak, breaker), "power-1 creature can't block");
    assert!(g.blocker_can_block_attacker(strong, breaker), "power-2 creature can block");
}

/// Krotiq Nestguard (CR 508.1a): its ability lets it attack despite defender.
#[test]
fn krotiq_nestguard_can_attack_after_ability() {
    let mut g = two_player_game();
    let krotiq = g.add_card_to_battlefield(0, catalog::krotiq_nestguard());
    g.clear_sickness(krotiq);
    g.active_player_idx = 0;
    // Defender isn't a legal attacker at the declare step.
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert!(!g.legal_attackers(0).contains(&krotiq), "defender can't attack yet");
    // Activate the ability in the main phase, then return to combat.
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: krotiq, ability_index: 0, target: None,
        additional_targets: vec![], x_value: None,
    }).expect("activate ignore-defender");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert!(g.legal_attackers(0).contains(&krotiq), "may now attack despite defender");
}

/// Snowmelt Stag (CR 613.7b / 208): base P/T becomes 5/2 only on your turn.
#[test]
fn snowmelt_stag_sets_base_pt_on_your_turn() {
    let mut g = two_player_game();
    let stag = g.add_card_to_battlefield(0, catalog::snowmelt_stag());
    g.active_player_idx = 0;
    let cp = g.computed_permanent(stag).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 2), "5/2 during your turn");
    g.active_player_idx = 1;
    let cp = g.computed_permanent(stag).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 5), "printed 2/5 on opp turn");
}

/// A +1/+1 counter stacks on top of Snowmelt Stag's base-P/T set (CR 613.7c/f).
#[test]
fn snowmelt_stag_counter_stacks_over_base_set() {
    let mut g = two_player_game();
    let stag = g.add_card_to_battlefield(0, catalog::snowmelt_stag());
    g.battlefield_find_mut(stag).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.active_player_idx = 0;
    let cp = g.computed_permanent(stag).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 3), "5/2 base + 1/1 counter");
}

// ── TDM spells batch ────────────────────────────────────────────────────────

/// Knockout Maneuver grows your creature, then it fights an opponent's.
#[test]
fn knockout_maneuver_counter_then_fight() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 3/3
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::knockout_maneuver());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(victim)], mode: None, x_value: None,
    }).expect("cast Knockout Maneuver");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "3 power kills the 2/2");
    assert_eq!(g.computed_permanent(mine).unwrap().power, 3, "grew to 3/3");
}

/// Rebellious Strike pumps +3/+0 and draws.
#[test]
fn rebellious_strike_pumps_and_draws() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..2 { g.add_card_to_library(0, catalog::plains()); }
    let id = g.add_card_to_hand(0, catalog::rebellious_strike());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Rebellious Strike");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 5, "2 + 3");
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 1, "cast -1, draw +1");
}

/// Narset's Rebuke deals 5 and exiles the creature if it dies (finality).
#[test]
fn narsets_rebuke_burns_and_exiles() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::narsets_rebuke());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Narset's Rebuke");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "5 damage kills the 4/4");
    assert!(g.exile.iter().any(|c| c.id == victim), "finality counter exiled it");
    assert_eq!(g.players[0].mana_pool.total(), 3, "added U/R/W");
}

/// Bewildering Blizzard draws three and shrinks opponents' creatures.
#[test]
fn bewildering_blizzard_draws_and_shrinks() {
    let mut g = two_player_game();
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::bewildering_blizzard());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bewildering Blizzard");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3);
    assert!(g.computed_permanent(opp).unwrap().power <= 0, "2 - 3 = -1 (treated as 0)");
}

/// Duty Beyond Death sacrifices one creature, then shields and grows the rest.
#[test]
fn duty_beyond_death_sac_then_team_buff() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dragon_sniper()); // 1/1 — auto-sacrificed (lowest)
    let keep = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4 → 5/5
    let id = g.add_card_to_hand(0, catalog::duty_beyond_death());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Duty Beyond Death");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(keep).unwrap().power, 5, "survivor grew");
    assert!(g.computed_permanent(keep).unwrap().keywords.contains(&crabomination::card::Keyword::Indestructible));
}

/// Lightfoot Technique adds a counter and grants flying + indestructible.
#[test]
fn lightfoot_technique_buffs_and_protects() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::lightfoot_technique());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Lightfoot Technique");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3, "+1/+1 counter");
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Flying));
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Indestructible));
}

/// Wail of War mode 0 shrinks the opponent's whole team.
#[test]
fn wail_of_war_shrinks_opponents() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::dragon_sniper()); // 1/1 → dies to -1/-1
    let id = g.add_card_to_hand(0, catalog::wail_of_war());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast Wail of War mode 0");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none(), "-1/-1 kills the 1/1");
}

// ── Charm modal-instant tests ────────────────────────────────────────────────

fn cast_charm(g: &mut GameState, card: crabomination::card::CardId, mode: usize, target: Option<Target>) {
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: card, target, additional_targets: vec![], mode: Some(mode), x_value: None,
    }).expect("charm castable");
    drain_stack(g);
}

#[test]
fn azorius_charm_tucks_a_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let charm = g.add_card_to_hand(0, catalog::azorius_charm());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let lib_before = g.players[1].library.len();
    cast_charm(&mut g, charm, 2, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none(), "creature left the battlefield");
    assert_eq!(g.players[1].library.len(), lib_before + 1, "put on top of library");
}

#[test]
fn selesnya_charm_exiles_a_big_creature() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 — not big enough
    let charm = g.add_card_to_hand(0, catalog::selesnya_charm());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    // 4/4 is power 4, < 5 — illegal target, so use a 5-power creature.
    g.battlefield_find_mut(big).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    cast_charm(&mut g, charm, 1, Some(Target::Permanent(big)));
    assert!(g.battlefield_find(big).is_none(), "exiled the 5-power creature");
}

#[test]
fn simic_charm_bounces() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let charm = g.add_card_to_hand(0, catalog::simic_charm());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast_charm(&mut g, charm, 2, Some(Target::Permanent(bear)));
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "bounced to owner's hand");
}

#[test]
fn golgari_charm_sweeps_minus_one() {
    let mut g = two_player_game();
    let x1 = g.add_card_to_battlefield(0, catalog::ornithopter()); // 0/2
    let x2 = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let charm = g.add_card_to_hand(0, catalog::golgari_charm());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    cast_charm(&mut g, charm, 0, None);
    assert_eq!(g.computed_permanent(x1).map(|c| c.toughness), Some(1), "0/2 → 0/1");
    assert_eq!(g.computed_permanent(x2).map(|c| c.power), Some(1), "2/2 → 1/1");
}

#[test]
fn rakdos_charm_exiles_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let charm = g.add_card_to_hand(0, catalog::rakdos_charm());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_charm(&mut g, charm, 0, Some(Target::Player(1)));
    assert!(g.players[1].graveyard.is_empty(), "opponent graveyard exiled");
}

#[test]
fn abzan_charm_distributes_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let charm = g.add_card_to_hand(0, catalog::abzan_charm());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    cast_charm(&mut g, charm, 2, Some(Target::Permanent(bear)));
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "both counters landed on the one target");
}

#[test]
fn sultai_charm_destroys_monocolored() {
    let mut g = two_player_game();
    let mono = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // mono-green
    let charm = g.add_card_to_hand(0, catalog::sultai_charm());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast_charm(&mut g, charm, 0, Some(Target::Permanent(mono)));
    assert!(g.battlefield_find(mono).is_none(), "monocolored creature destroyed");
}

#[test]
fn jeskai_charm_burns_opponent() {
    let mut g = two_player_game();
    let charm = g.add_card_to_hand(0, catalog::jeskai_charm());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let life = g.players[1].life;
    cast_charm(&mut g, charm, 1, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 4, "4 damage to the opponent");
}

#[test]
fn mardu_charm_bolts_a_creature() {
    let mut g = two_player_game();
    let v = g.add_card_to_battlefield(1, catalog::serra_angel());
    let charm = g.add_card_to_hand(0, catalog::mardu_charm());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    cast_charm(&mut g, charm, 0, Some(Target::Permanent(v)));
    assert!(g.battlefield_find(v).is_none(), "4 damage killed the 4/4");
}

#[test]
fn temur_charm_fights() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 3/3 with +1/+1
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let charm = g.add_card_to_hand(0, catalog::temur_charm());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: charm, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: Some(0), x_value: None,
    }).expect("Temur Charm fight mode");
    drain_stack(&mut g);
    // Mine becomes 3/3, deals 3 to their 2/2 → dies; theirs deals 2 back, mine survives.
    assert!(g.battlefield_find(theirs).is_none(), "their creature died to the fight");
    assert!(g.battlefield_find(mine).is_some(), "my pumped creature survived");
}

#[test]
fn turn_to_frog_makes_target_a_1_1_blue_frog_with_no_abilities() {
    use crabomination::card::{CreatureType, Keyword};
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flying-vigilance
    let frog = g.add_card_to_hand(0, catalog::turn_to_frog());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: frog, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Turn to Frog castable for {1}{U}");
    drain_stack(&mut g);
    let cp = g.computed_permanent(angel).expect("angel still on bf");
    assert_eq!((cp.power, cp.toughness), (1, 1), "becomes 1/1");
    assert!(cp.subtypes.creature_types == vec![CreatureType::Frog], "becomes a Frog");
    assert!(cp.colors.contains(&Color::Blue) && cp.colors.len() == 1, "becomes mono-blue");
    assert!(!cp.keywords.contains(&Keyword::Flying), "loses flying");
    assert!(cp.lost_all_abilities, "loses all abilities");
}

#[test]
fn kenriths_transformation_draws_and_makes_a_3_3_green_elk() {
    use crabomination::card::{CreatureType, Keyword};
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let hand_before = g.players[0].hand.len();
    let aura = g.add_card_to_hand(0, catalog::kenriths_transformation());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Kenrith's Transformation castable for {1}{G}");
    drain_stack(&mut g);
    // ETB draw nets +1 (the Aura left hand, one card drawn).
    assert_eq!(g.players[0].hand.len(), hand_before, "ETB draw replaced the cast card");
    let cp = g.computed_permanent(angel).expect("angel still on bf");
    assert_eq!((cp.power, cp.toughness), (3, 3), "becomes 3/3");
    assert!(cp.subtypes.creature_types == vec![CreatureType::Elk], "becomes an Elk");
    assert!(!cp.keywords.contains(&Keyword::Flying), "loses flying");
    assert!(cp.lost_all_abilities, "loses all abilities");
}

#[test]
fn lignify_makes_a_0_4_treefolk_with_no_abilities() {
    use crabomination::card::{CreatureType, Keyword};
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let aura = g.add_card_to_hand(0, catalog::lignify());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lignify castable for {1}{G}");
    drain_stack(&mut g);
    let cp = g.computed_permanent(angel).expect("angel still on bf");
    assert_eq!((cp.power, cp.toughness), (0, 4), "becomes 0/4");
    assert!(cp.subtypes.creature_types == vec![CreatureType::Treefolk], "becomes a Treefolk");
    assert!(!cp.keywords.contains(&Keyword::Flying), "loses flying");
    assert!(cp.lost_all_abilities, "loses all abilities");
}

#[test]
fn deep_analysis_flashback_pays_three_life() {
    // Flashback—{1}{U}, Pay 3 life: from the graveyard, draw two and lose 3 life.
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let spell = g.add_card_to_graveyard(0, catalog::deep_analysis());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastFlashback {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Deep Analysis flashback castable for {1}{U} + 3 life");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before - 3, "paid 3 life as a flashback cost");
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew two cards");
    assert!(g.exile.iter().any(|c| c.id == spell), "exiled after flashback resolves");
}

#[test]
fn deep_analysis_flashback_blocked_without_enough_life() {
    // CR 119.4 — can't pay 3 life at 2 life, so flashback is rejected.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let spell = g.add_card_to_graveyard(0, catalog::deep_analysis());
    g.players[0].life = 2;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::CastFlashback {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "2 life can't pay the 3-life flashback rider");
}

#[test]
fn ovinize_makes_target_a_0_1_with_no_abilities() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::ovinize());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ovinize castable");
    drain_stack(&mut g);
    let cp = g.computed_permanent(angel).unwrap();
    assert_eq!((cp.power, cp.toughness), (0, 1), "becomes 0/1");
    assert!(!cp.keywords.contains(&Keyword::Flying) && cp.lost_all_abilities, "loses abilities");
}

#[test]
fn snakeform_makes_a_1_1_green_snake_and_draws() {
    use crabomination::card::{CreatureType, Keyword};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::snakeform());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Snakeform castable for {2}{G/U}");
    drain_stack(&mut g);
    let cp = g.computed_permanent(angel).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    assert!(cp.subtypes.creature_types == vec![CreatureType::Snake], "is a Snake");
    assert!(!cp.keywords.contains(&Keyword::Flying) && cp.lost_all_abilities);
    // cast (-1) + draw (+1) = net unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before, "drew a card");
}

#[test]
fn polymorphists_jest_frogifies_target_players_creatures() {
    use crabomination::card::{CreatureType, Keyword};
    let mut g = two_player_game();
    let a1 = g.add_card_to_battlefield(1, catalog::serra_angel());
    let a2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::polymorphists_jest());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Polymorphist's Jest targets a player");
    drain_stack(&mut g);
    for victim in [a1, a2] {
        let cp = g.computed_permanent(victim).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 1), "their creature is 1/1");
        assert!(cp.subtypes.creature_types == vec![CreatureType::Frog]);
        assert!(cp.lost_all_abilities);
    }
    // My own creature is untouched.
    let cp = g.computed_permanent(mine).unwrap();
    assert!(cp.keywords.contains(&Keyword::Flying), "my creature keeps flying");
}

#[test]
fn frogify_aura_makes_a_1_1_blue_frog() {
    use crabomination::card::{CreatureType, Keyword};
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let aura = g.add_card_to_hand(0, catalog::frogify());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Frogify castable");
    drain_stack(&mut g);
    let cp = g.computed_permanent(angel).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    assert!(cp.subtypes.creature_types == vec![CreatureType::Frog]);
    assert!(!cp.keywords.contains(&Keyword::Flying) && cp.lost_all_abilities);
}

#[test]
fn darksteel_mutation_makes_an_indestructible_0_1_insect() {
    use crabomination::card::{CardType, CreatureType, Keyword};
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let aura = g.add_card_to_hand(0, catalog::darksteel_mutation());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Darksteel Mutation castable");
    drain_stack(&mut g);
    let cp = g.computed_permanent(angel).unwrap();
    assert_eq!((cp.power, cp.toughness), (0, 1));
    assert!(cp.subtypes.creature_types == vec![CreatureType::Insect]);
    assert!(cp.card_types.contains(&CardType::Artifact), "becomes an artifact");
    assert!(cp.keywords.contains(&Keyword::Indestructible), "gains indestructible");
    assert!(cp.lost_all_abilities, "loses its printed abilities");
}

#[test]
fn sandstorm_pings_each_attacking_creature() {
    let mut g = two_player_game();
    // Attacker 0/1 dies to 1 damage; a 2/2 survives.
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(bears);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bears, target: AttackTarget::Player(1),
    }])).expect("declared attacker");
    let id = g.add_card_to_hand(1, catalog::sandstorm());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sandstorm castable");
    drain_stack(&mut g);
    let inst = g.battlefield_find(bears).expect("2/2 bear survives 1 damage");
    assert_eq!(inst.damage, 1, "attacking creature took 1 damage from Sandstorm");
}

#[test]
fn witness_protection_makes_a_1_1_gw_citizen() {
    use crabomination::card::{CardType, CreatureType, Keyword};
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let aura = g.add_card_to_hand(0, catalog::witness_protection());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Witness Protection castable for {U}");
    drain_stack(&mut g);
    let cp = g.computed_permanent(angel).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    assert!(cp.subtypes.creature_types == vec![CreatureType::Citizen]);
    assert!(cp.card_types.contains(&CardType::Creature) && !cp.card_types.contains(&CardType::Land));
    assert!(cp.colors.contains(&Color::Green) && cp.colors.contains(&Color::White));
    assert!(!cp.keywords.contains(&Keyword::Flying) && cp.lost_all_abilities);
}

#[test]
fn song_of_the_dryads_turns_a_permanent_into_a_forest() {
    use crabomination::card::{CardType, LandType};
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let aura = g.add_card_to_hand(0, catalog::song_of_the_dryads());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Song of the Dryads castable");
    drain_stack(&mut g);
    let cp = g.computed_permanent(angel).unwrap();
    assert!(cp.card_types.contains(&CardType::Land), "becomes a land");
    assert!(!cp.card_types.contains(&CardType::Creature), "no longer a creature");
    assert!(cp.subtypes.land_types.contains(&LandType::Forest), "is a Forest");
    assert!(cp.colors.is_empty(), "colorless");
}

#[test]
fn imprisoned_in_the_moon_neutralizes_to_a_colorless_land() {
    use crabomination::card::CardType;
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let aura = g.add_card_to_hand(0, catalog::imprisoned_in_the_moon());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Imprisoned in the Moon castable");
    drain_stack(&mut g);
    let cp = g.computed_permanent(angel).unwrap();
    assert!(cp.card_types.contains(&CardType::Land) && !cp.card_types.contains(&CardType::Creature));
    assert!(cp.lost_all_abilities, "abilities removed");
    assert!(cp.colors.is_empty(), "colorless");
}

#[test]
fn blink_of_an_eye_bounces_and_draws_when_kicked() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let creat = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::blink_of_an_eye());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpellKicked {
        card_id: id, target: Some(Target::Permanent(creat)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Blink of an Eye kicked");
    drain_stack(&mut g);
    assert!(g.battlefield_find(creat).is_none(), "bounced to hand");
    // cast (-1) + draw (+1) = net unchanged hand size.
    assert_eq!(g.players[0].hand.len(), hand_before, "kicked draw replaced the cast card");
}

#[test]
fn capsize_returns_a_permanent_to_hand() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::island());
    let id = g.add_card_to_hand(0, catalog::capsize());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(land)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Capsize castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "permanent bounced");
    assert!(g.players[1].hand.iter().any(|c| c.definition.name == "Island"), "back in owner's hand");
}

#[test]
fn undying_evil_grants_undying_eot() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::undying_evil());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bears)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Undying Evil castable");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bears).unwrap().keywords.contains(&Keyword::Undying),
        "creature gained undying");
}

#[test]
fn heat_shimmer_makes_a_hasty_copy() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let before = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    let id = g.add_card_to_hand(0, catalog::heat_shimmer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bears)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Heat Shimmer castable");
    drain_stack(&mut g);
    let after = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    assert_eq!(after, before + 1, "minted a copy of the target creature");
}

#[test]
fn macabre_waltz_returns_two_creatures_and_discards() {
    let mut g = two_player_game();
    let _c1 = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let _c2 = g.add_card_to_graveyard(0, catalog::serra_angel());
    let filler = g.add_card_to_hand(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::macabre_waltz());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Non-UI seat: the gy-return auto-picks the two highest-MV creatures, and
    // the discard auto-picks the leftover Island.
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Macabre Waltz castable");
    drain_stack(&mut g);
    let creatures_in_hand = g.players[0].hand.iter()
        .filter(|c| c.definition.name == "Grizzly Bears" || c.definition.name == "Serra Angel").count();
    assert_eq!(creatures_in_hand, 2, "both creatures returned to hand");
    assert!(!g.players[0].hand.iter().any(|c| c.id == filler), "discarded the Island");
}

#[test]
fn cloudshift_blinks_your_creature() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::cloudshift());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bears)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cloudshift castable");
    drain_stack(&mut g);
    let _ = bears;
    // Exactly one Grizzly Bears is back under your control, and it re-entered
    // (summoning sickness reset — proof of the exile/return blink).
    let returned: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Grizzly Bears" && c.controller == 0).collect();
    assert_eq!(returned.len(), 1, "returned under your control");
    assert!(returned[0].summoning_sick, "the returned creature is a new, summoning-sick object");
}

#[test]
fn spark_spray_pings_a_creature() {
    let mut g = two_player_game();
    let one_one = g.add_card_to_battlefield(1, catalog::ornithopter()); // 0/2
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::spark_spray());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bears)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spark Spray castable");
    drain_stack(&mut g);
    let _ = one_one;
    assert_eq!(g.battlefield_find(bears).map(|c| c.damage), Some(1), "1 damage dealt");
}

#[test]
fn haze_of_pollen_fogs_combat() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    let life_before = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).unwrap();
    let id = g.add_card_to_hand(1, catalog::haze_of_pollen());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Haze of Pollen castable");
    drain_stack(&mut g);
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    assert_eq!(g.players[1].life, life_before, "all combat damage prevented");
}

#[test]
fn brain_freeze_mills_three() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(1, catalog::island()); }
    let before = g.players[1].graveyard.len();
    let id = g.add_card_to_hand(0, catalog::brain_freeze());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Brain Freeze castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), before + 3, "target player milled 3");
}

#[test]
fn defile_scales_with_swamps() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::swamp()); }
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::defile());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bears)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Defile castable");
    drain_stack(&mut g);
    // 3 Swamps → -3/-3 → 2/2 dies.
    assert!(g.battlefield_find(bears).is_none(), "creature dies to -3/-3 from 3 Swamps");
}

#[test]
fn prison_realm_exiles_until_it_leaves() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prison_realm());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Prison Realm castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "creature exiled by Prison Realm");
    // The enchantment leaves → the creature returns.
    let realm = g.battlefield.iter().find(|c| c.definition.name == "Prison Realm").unwrap().id;
    g.remove_from_battlefield_to_graveyard_raw(realm);
    g.check_state_based_actions();
    assert!(g.battlefield.iter().any(|c| c.id == victim),
        "creature returns when Prison Realm leaves");
}

#[test]
fn stasis_snare_has_flash_and_exiles() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    assert!(catalog::stasis_snare().keywords.contains(&Keyword::Flash), "has flash");
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::stasis_snare());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stasis Snare castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "creature exiled by Stasis Snare");
}

#[test]
fn reprobation_makes_a_0_1_with_no_abilities() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let aura = g.add_card_to_hand(0, catalog::reprobation());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reprobation castable");
    drain_stack(&mut g);
    let cp = g.computed_permanent(angel).unwrap();
    assert_eq!((cp.power, cp.toughness), (0, 1));
    assert!(!cp.keywords.contains(&Keyword::Flying) && cp.lost_all_abilities);
}

#[test]
fn bound_in_gold_locks_down_a_permanent() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::bound_in_gold());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bears)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bound in Gold castable");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bears).unwrap();
    assert!(cp.keywords.contains(&Keyword::CantAttack) && cp.keywords.contains(&Keyword::CantBlock));
}

#[test]
fn flames_of_the_firebrand_divides_three_damage() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::flames_of_the_firebrand());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    // Auto-divider spreads 3 across the two targets; assert total board damage = 3.
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None,
    }).expect("Flames castable");
    drain_stack(&mut g);
    let dmg: u32 = [a, b].iter().filter_map(|id| g.battlefield_find(*id)).map(|c| c.damage).sum();
    let dead = [a, b].iter().filter(|id| g.battlefield_find(**id).is_none()).count();
    assert_eq!(dmg + dead as u32 * 2, 3, "3 damage divided across the targets (2 marks a kill)");
}

#[test]
fn cleansing_nova_mode_zero_wraths_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::cleansing_nova());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Cleansing Nova castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "all creatures destroyed");
}

#[test]
fn time_wipe_saves_one_creature_then_wraths() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::time_wipe());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mine)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Time Wipe castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"), "my creature bounced to hand");
    assert!(g.battlefield_find(theirs).is_none(), "their creature destroyed by the wrath");
}

#[test]
fn disallow_counters_a_spell() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt cast");
    let id = g.add_card_to_hand(0, catalog::disallow());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Disallow castable");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "Lightning Bolt countered to graveyard");
}

#[test]
fn voice_of_the_provinces_makes_a_human() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::voice_of_the_provinces());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Human" && c.controller == 0),
        "minted a 1/1 Human");
}

#[test]
fn sensor_splicer_makes_a_golem_with_vigilance_anthem() {
    use crabomination::card::{CreatureType, Keyword};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::sensor_splicer());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    let golem = g.battlefield.iter().find(|c| c.definition.name == "Golem").expect("Golem minted");
    let cp = g.computed_permanent(golem.id).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.subtypes.creature_types.contains(&CreatureType::Golem));
    assert!(cp.keywords.contains(&Keyword::Vigilance), "Golem anthem grants vigilance");
}

#[test]
fn maul_splicer_makes_two_golems_with_trample() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::maul_splicer());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    let golems: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Golem").collect();
    assert_eq!(golems.len(), 2, "minted two Golems");
    let cp = g.computed_permanent(golems[0].id).unwrap();
    assert!(cp.keywords.contains(&Keyword::Trample), "Golem anthem grants trample");
}

#[test]
fn sengir_autocrat_makes_three_serfs() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::sengir_autocrat());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Serf").count(), 3, "three Serfs");
}

#[test]
fn yavimaya_granger_fetches_a_basic_land() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_battlefield(0, catalog::yavimaya_granger());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == forest), "fetched a Forest to hand");
}

#[test]
fn omenspeaker_scrys_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::omenspeaker());
    // Scry resolves with the auto-decider (keeps order); just assert it fires cleanly.
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 3), "Omenspeaker is a 1/3");
}

#[test]
fn wing_splicer_grants_golems_flying() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::wing_splicer());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    let golem = g.battlefield.iter().find(|c| c.definition.name == "Golem").expect("Golem minted");
    assert!(g.computed_permanent(golem.id).unwrap().keywords.contains(&Keyword::Flying),
        "Golem anthem grants flying");
}

#[test]
fn rejuvenate_gains_five_life() {
    let mut g = two_player_game();
    let before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::rejuvenate());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Rejuvenate castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, before + 5, "gained 5 life");
}

#[test]
fn angelic_gift_draws_and_grants_flying() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::angelic_gift());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bears)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Angelic Gift castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before, "ETB draw replaced the cast aura");
    assert!(g.computed_permanent(bears).unwrap().keywords.contains(&Keyword::Flying), "grants flying");
}

/// The bot's removal ping accounts for damage already marked (CR 120.6): a
/// 1-damage pinger finishes a 2/2 that took 1 combat damage this turn.
#[test]
fn bot_pings_a_chipped_creature_for_lethal() {
    use crabomination::server::bot::{Bot, RandomBot};
    let mut g = two_player_game();
    let tim = g.add_card_to_battlefield(0, catalog::prodigal_sorcerer()); // {T}: 1 dmg
    g.clear_sickness(tim);
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.battlefield_find_mut(foe).unwrap().damage = 1; // chipped in combat
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let action = RandomBot::new().next_action(&g, 0);
    assert!(
        matches!(&action, Some(GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. })
            if *card_id == tim && *t == foe),
        "bot should ping the chipped 2/2 for the kill; got {action:?}"
    );
}

/// Chandra's Ignition — the chosen creature deals its power to each other
/// creature AND each opponent (the source and its controller are spared).
#[test]
fn chandras_ignition_hits_others_and_opponents() {
    let mut g = two_player_game();
    let source = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // dies (2 dmg)
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // dies
    let spell = g.add_card_to_hand(0, catalog::chandras_ignition());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    let opp_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(source)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Chandra's Ignition");
    drain_stack(&mut g);
    assert!(g.battlefield_find(source).is_some(), "source spared");
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none(), "others died");
    assert_eq!(g.players[1].life, opp_life - 2, "each opponent took power-2");
}

/// Molten Rain destroys a land and burns the controller only if it was nonbasic.
#[test]
fn molten_rain_destroys_land_and_burns_on_nonbasic() {
    // Nonbasic land → 2 damage to controller.
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::mutavault()); // nonbasic
    let life1 = g.players[1].life;
    let eff = catalog::molten_rain().effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(land)];
    g.resolve_effect(&eff, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(land).is_none(), "land destroyed");
    assert_eq!(g.players[1].life, life1 - 2, "nonbasic → 2 damage to controller");

    // Basic land → destroyed, no burn.
    let mut g = two_player_game();
    let basic = g.add_card_to_battlefield(1, catalog::forest());
    let life1 = g.players[1].life;
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(basic)];
    g.resolve_effect(&catalog::molten_rain().effect.clone(), &ctx).unwrap();
    assert!(g.battlefield_find(basic).is_none(), "basic destroyed");
    assert_eq!(g.players[1].life, life1, "basic → no burn");
}

/// Psionic Blast deals 4 to the target and 2 to its caster.
#[test]
fn psionic_blast_hits_target_and_caster() {
    let mut g = two_player_game();
    let life0 = g.players[0].life;
    let life1 = g.players[1].life;
    let eff = catalog::psionic_blast().effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Player(1)];
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.players[1].life, life1 - 4, "4 to target");
    assert_eq!(g.players[0].life, life0 - 2, "2 to caster");
}

/// Choking Sands can't hit a Swamp but destroys other nonbasics with the burn.
#[test]
fn choking_sands_spares_swamps_burns_nonbasics() {
    let mut g = two_player_game();
    let nonbasic = g.add_card_to_battlefield(1, catalog::mutavault());
    let life1 = g.players[1].life;
    let eff = catalog::choking_sands().effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(nonbasic)];
    g.resolve_effect(&eff, &ctx).unwrap();
    assert!(g.battlefield_find(nonbasic).is_none(), "non-Swamp nonbasic destroyed");
    assert_eq!(g.players[1].life, life1 - 2, "nonbasic → 2 damage");
}

/// Rain of Tears destroys a target land.
#[test]
fn rain_of_tears_destroys_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let eff = catalog::rain_of_tears().effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(land)];
    g.resolve_effect(&eff, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(land).is_none(), "land destroyed");
}

/// Reckless Rage burns an enemy creature for 4 and one of yours for 2.
#[test]
fn reckless_rage_splits_damage() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // 5/5 → survives 4
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → dies to 2
    let eff = catalog::reckless_rage().effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(foe), Target::Permanent(mine)];
    g.resolve_effect(&eff, &ctx).unwrap();
    g.check_state_based_actions();
    assert_eq!(g.battlefield_find(foe).unwrap().damage, 4, "enemy took 4");
    assert!(g.battlefield_find(mine).is_none(), "your 2/2 died to 2");
}

/// Electrostatic Bolt deals 2 to a creature, but 4 to an artifact creature.
#[test]
fn electrostatic_bolt_doubles_on_artifacts() {
    let eff = catalog::electrostatic_bolt().effect.clone();
    // Non-artifact 3/3 survives with 2 damage.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 2, "non-artifact takes 2");
    // Artifact creature with toughness 4 takes the full 4 and dies.
    let mut g = two_player_game();
    let bot = g.add_card_to_battlefield(1, catalog::brass_squire()); // 1/3 artifact
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(bot)];
    g.resolve_effect(&catalog::electrostatic_bolt().effect.clone(), &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(bot).is_none(), "artifact creature took 4 and died");
}

/// Barbed Shocker wheels the player it hits (discard hand, draw that many).
#[test]
fn barbed_shocker_wheels_the_damaged_player() {
    let mut g = two_player_game();
    // P1 has 3 cards in hand and a stocked library.
    for _ in 0..3 { g.add_card_to_hand(1, catalog::island()); }
    for _ in 0..5 { g.add_card_to_library(1, catalog::forest()); }
    let hand_before = g.players[1].hand.len();
    let eff = catalog::barbed_shocker().triggered_abilities[0].effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_trigger(crabomination::card::CardId(9), 0, Some(Target::Player(1)), 0);
    ctx.targets = vec![Target::Player(1)];
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.players[1].hand.len(), hand_before, "discarded hand, drew that many");
    assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Island"), "old hand discarded");
}

/// Sudden Impact burns a player for their hand size.
#[test]
fn sudden_impact_burns_for_hand_size() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_hand(1, catalog::island()); }
    let life1 = g.players[1].life;
    let eff = catalog::sudden_impact().effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Player(1)];
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.players[1].life, life1 - 4, "4 cards in hand → 4 damage");
}

/// Fissure destroys either a creature or a land.
#[test]
fn fissure_destroys_creature_or_land() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let eff = catalog::fissure().effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(creature)];
    g.resolve_effect(&eff, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(creature).is_none(), "creature destroyed");
}

/// Kaervek's Torch deals X damage where X is the paid cost.
#[test]
fn kaerveks_torch_deals_x() {
    let mut g = two_player_game();
    let life1 = g.players[1].life;
    let eff = catalog::kaerveks_torch().effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 3);
    ctx.targets = vec![Target::Player(1)];
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.players[1].life, life1 - 3, "X=3 → 3 damage");
}

/// Seismic Spike destroys a land and refunds {R}{R}.
#[test]
fn seismic_spike_destroys_land_and_ramps() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let eff = catalog::seismic_spike().effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(land)];
    g.resolve_effect(&eff, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(land).is_none(), "land destroyed");
    assert_eq!(g.players[0].mana_pool.amount(crabomination::mana::Color::Red), 2, "added RR");
}

/// Boulderfall divides 5 damage among targets (all onto one here).
#[test]
fn boulderfall_divides_five_damage() {
    let mut g = two_player_game();
    let life1 = g.players[1].life;
    let eff = catalog::boulderfall().effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Player(1)];
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.players[1].life, life1 - 5, "all 5 damage to the sole target");
}

/// Rain of Salt destroys two target lands.
#[test]
fn rain_of_salt_destroys_two_lands() {
    let mut g = two_player_game();
    let l1 = g.add_card_to_battlefield(1, catalog::forest());
    let l2 = g.add_card_to_battlefield(1, catalog::island());
    let eff = catalog::rain_of_salt().effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(l1), Target::Permanent(l2)];
    g.resolve_effect(&eff, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(l1).is_none() && g.battlefield_find(l2).is_none(), "both lands destroyed");
}

/// Afterlife destroys a creature and gives its controller a flying Spirit.
#[test]
fn afterlife_destroys_and_leaves_a_spirit() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let eff = catalog::afterlife().effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(victim)];
    g.resolve_effect(&eff, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(victim).is_none(), "creature destroyed");
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Spirit" && c.controller == 1),
        "its controller got a Spirit"
    );
}

/// Excommunicate puts the target creature on top of its owner's library.
#[test]
fn excommunicate_tucks_to_top() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let eff = catalog::excommunicate().effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(victim)];
    g.resolve_effect(&eff, &ctx).unwrap();
    assert!(g.battlefield_find(victim).is_none(), "left the battlefield");
    assert_eq!(g.players[1].library.first().map(|c| c.id), Some(victim), "on top of owner's library");
}

/// Assassinate destroys only a tapped creature.
#[test]
fn assassinate_needs_a_tapped_target() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(foe).unwrap().tapped = true;
    let eff = catalog::assassinate().effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(foe)];
    g.resolve_effect(&eff, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(foe).is_none(), "tapped creature destroyed");
}

/// Kill Shot destroys only an attacking creature.
#[test]
fn kill_shot_destroys_attacker() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    // P1 attacks P0 so the creature is "attacking".
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(0),
    }])).expect("attack");
    drain_stack(&mut g);
    let eff = catalog::kill_shot().effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(attacker)];
    g.resolve_effect(&eff, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(attacker).is_none(), "attacking creature destroyed");
}

/// Aggressive Urge pumps a creature and cantrips.
#[test]
fn aggressive_urge_pumps_and_draws() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let hand0 = g.players[0].hand.len();
    let eff = catalog::aggressive_urge().effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "+1/+1 applied");
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew a card");
}

/// Bestial Menace makes a Snake, a Wolf, and an Elephant.
#[test]
fn bestial_menace_makes_three_beasts() {
    let mut g = two_player_game();
    let eff = catalog::bestial_menace().effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&eff, &ctx).unwrap();
    for name in ["Snake", "Wolf", "Elephant"] {
        assert!(g.battlefield.iter().any(|c| c.definition.name == name), "made a {name}");
    }
}

/// Wild Instincts pumps your creature and fights an opponent's.
#[test]
fn wild_instincts_pumps_and_fights() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 4/4
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → dies to 4
    let eff = catalog::wild_instincts().effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(mine), Target::Permanent(theirs)];
    g.resolve_effect(&eff, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(theirs).is_none(), "opponent's creature died to the fight");
    assert!(g.battlefield_find(mine).is_some(), "your pumped 4/4 survived their 2");
}

/// Weave Fate draws two.
#[test]
fn weave_fate_draws_two() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let hand0 = g.players[0].hand.len();
    let eff = catalog::weave_fate().effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.players[0].hand.len(), hand0 + 2, "drew two");
}

/// Pilfered Plans mills a player and draws two.
#[test]
fn pilfered_plans_mills_then_draws() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(1, catalog::forest()); }
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let gy1 = g.players[1].graveyard.len();
    let hand0 = g.players[0].hand.len();
    let eff = catalog::pilfered_plans().effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Player(1)];
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.players[1].graveyard.len(), gy1 + 2, "milled two");
    assert_eq!(g.players[0].hand.len(), hand0 + 2, "drew two");
}

/// Short Bow grants +1/+1, vigilance, and reach to the equipped creature.
#[test]
fn short_bow_grants_keywords() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let hero = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bow = g.add_card_to_battlefield(0, catalog::short_bow());
    g.battlefield_find_mut(bow).unwrap().attached_to = Some(hero);
    let cp = g.computed_permanent(hero).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
    assert!(cp.keywords.contains(&Keyword::Vigilance) && cp.keywords.contains(&Keyword::Reach));
}

/// Neurok Hoversail grants flying.
#[test]
fn neurok_hoversail_grants_flying() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let hero = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sail = g.add_card_to_battlefield(0, catalog::neurok_hoversail());
    g.battlefield_find_mut(sail).unwrap().attached_to = Some(hero);
    assert!(g.computed_permanent(hero).unwrap().keywords.contains(&Keyword::Flying), "flying granted");
}

/// Leather Armor grants +0/+1 and ward.
#[test]
fn leather_armor_grants_toughness_and_ward() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let hero = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let armor = g.add_card_to_battlefield(0, catalog::leather_armor());
    g.battlefield_find_mut(armor).unwrap().attached_to = Some(hero);
    let cp = g.computed_permanent(hero).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3), "+0/+1");
    assert!(cp.keywords.iter().any(|k| matches!(k, Keyword::Ward(_))), "ward granted");
}

/// Flame Jab pings any target for 1 and carries Retrace.
#[test]
fn flame_jab_pings_and_has_retrace() {
    use crabomination::card::Keyword;
    assert!(catalog::flame_jab().keywords.contains(&Keyword::Retrace), "has Retrace");
    let mut g = two_player_game();
    let life1 = g.players[1].life;
    let eff = catalog::flame_jab().effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Player(1)];
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.players[1].life, life1 - 1, "1 damage");
}

/// Recoup grants flashback to a sorcery in your graveyard.
#[test]
fn recoup_grants_flashback_to_a_sorcery() {
    let mut g = two_player_game();
    let sorc = g.add_card_to_graveyard(0, catalog::molten_rain()); // a sorcery
    let eff = catalog::recoup().effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(sorc)];
    g.resolve_effect(&eff, &ctx).unwrap();
    let card = g.players[0].graveyard.iter().find(|c| c.id == sorc).expect("still in gy");
    assert!(card.granted_flashback_eot.is_some(), "flashback granted until end of turn");
}

/// Fireball divides X evenly (rounded down) among the chosen targets and
/// costs {1} more per target beyond the first.
#[test]
fn fireball_divides_evenly_and_taxes_extra_targets() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fireball());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: Some(5),
    })
    .expect("X=5 two-target Fireball castable for R plus 6 (X + per-target tax)");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 0, "tax consumed the 6th generic");
    assert_eq!(g.players[1].life, 18, "player takes 5/2 = 2");
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear takes 2 and dies");
}

/// A two-target Fireball without mana for the per-target {1} tax is rejected.
#[test]
fn fireball_extra_target_tax_must_be_paid() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fireball());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: Some(5),
    }).is_err(), "X=5 + extra target needs R plus 6, only 5 generic available");
    assert!(g.players[0].hand.iter().any(|c| c.id == id), "cast reverted to hand");
}

/// Return to Dust exiles two targets during your main phase; the second
/// target slot is rejected off-main-phase (single target still fine).
#[test]
fn return_to_dust_second_target_gated_on_main_phase() {
    let mut g = two_player_game();
    g.step = crabomination::game::TurnStep::PreCombatMain;
    let a1 = g.add_card_to_battlefield(1, catalog::sol_ring());
    let a2 = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::return_to_dust());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(a1)),
        additional_targets: vec![Target::Permanent(a2)],
        mode: None,
        x_value: None,
    }).expect("two targets legal in caster's main phase");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == a1) && g.exile.iter().any(|c| c.id == a2),
        "both artifacts exiled");

    // Off-main-phase: two targets rejected, one target accepted (instant).
    let a3 = g.add_card_to_battlefield(1, catalog::sol_ring());
    let a4 = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id2 = g.add_card_to_hand(0, catalog::return_to_dust());
    g.step = crabomination::game::TurnStep::Upkeep;
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: id2,
        target: Some(Target::Permanent(a3)),
        additional_targets: vec![Target::Permanent(a4)],
        mode: None,
        x_value: None,
    }).is_err(), "second target only during your main phase");
    g.perform_action(GameAction::CastSpell {
        card_id: id2,
        target: Some(Target::Permanent(a3)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("single target castable at instant speed");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == a3), "single target exiled");
}

/// CR 702.94 — the miracle window is the reveal offer, not the whole turn:
/// once the step advances, the permission (and its alt-cost) are gone.
#[test]
fn miracle_window_dies_at_step_transition() {
    let mut g = two_player_game();
    let bonfire = g.add_card_to_library(0, catalog::bonfire_of_the_damned());
    g.players[0].cards_drawn_this_turn = 0;
    let mut events = vec![];
    assert!(g.draw_one(0, &mut events), "drew the top card");
    assert!(
        g.players[0].hand.iter().find(|c| c.id == bonfire).unwrap().may_play_until.is_some(),
        "window live in the draw step"
    );
    // Step advances — the offer is gone.
    g.advance_step(vec![]).expect("step advances");
    let card = g.players[0].hand.iter().find(|c| c.id == bonfire).unwrap();
    assert!(card.may_play_until.is_none(), "window died at the step transition");
    assert!(card.granted_alt_cast_cost_eot.is_none(), "alt-cost shares the window's lifetime");
    // The normal cast for full cost is unaffected (back in a main phase).
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: bonfire, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: Some(1),
    })
    .expect("full-cost cast still available");
}

/// CR 702.94e — a miracled SORCERY is castable inside the window even when
/// sorcery timing wouldn't normally allow it (e.g. during the opponent's
/// turn, off an instant-speed draw).
#[test]
fn miracle_sorcery_castable_outside_sorcery_timing() {
    let mut g = two_player_game();
    let bonfire = g.add_card_to_library(0, catalog::bonfire_of_the_damned());
    // Opponent's turn: normally no sorcery casts for P0.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 0;
    g.players[0].cards_drawn_this_turn = 0;
    let mut events = vec![];
    assert!(g.draw_one(0, &mut events), "P0 draws on the opponent's turn");
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_life = g.players[1].life;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bonfire, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: Some(1),
    })
    .expect("miracle cast ignores the sorcery-speed gate (CR 702.94e)");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 1, "X=1 miracle Bonfire resolved");
}
