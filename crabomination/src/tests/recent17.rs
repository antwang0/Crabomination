//! Functionality tests for the `catalog::sets::decks::recent17` Foundations batch.

use crate::card::{ArtifactSubtype, Keyword};
use crate::catalog;
use crate::game::types::Target;
use crate::game::*;

/// Burglar Rat's ETB makes each opponent discard a card.
#[test]
fn burglar_rat_each_opponent_discards() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::burglar_rat());
    let before = g.players[1].hand.len();
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), before - 1, "opponent discarded one");
}

/// Corsair Captain mints a Treasure on ETB and pumps other Pirates.
#[test]
fn corsair_captain_treasure_and_pirate_anthem() {
    let mut g = two_player_game();
    let cap = g.add_card_to_battlefield(0, catalog::corsair_captain());
    g.fire_self_etb_triggers(cap, 0);
    drain_stack(&mut g);
    let treasures = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Treasure))
        .count();
    assert_eq!(treasures, 1, "one Treasure minted");
    // Another Pirate gets +1/+1; the Captain itself doesn't (other-only).
    let other = g.add_card_to_battlefield(0, catalog::corsair_captain());
    assert_eq!(g.computed_permanent(other).unwrap().power, 3, "other Pirate buffed");
    assert_eq!(g.computed_permanent(cap).unwrap().power, 3, "buffed by the other Captain");
}

/// Crow of Dark Tidings mills two when it enters.
#[test]
fn crow_mills_two_on_etb() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let id = g.add_card_to_battlefield(0, catalog::crow_of_dark_tidings());
    let gy = g.players[0].graveyard.len();
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), gy + 2, "milled two");
}

/// Crusader of Odric's P/T scales with the creatures you control.
#[test]
fn crusader_of_odric_scales_with_creatures() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::crusader_of_odric());
    assert_eq!(g.computed_permanent(c).unwrap().power, 1, "just itself");
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(c).unwrap().power, 2, "two creatures now");
}

/// Angel of Finality exiles the opponent's graveyard on ETB.
#[test]
fn angel_of_finality_exiles_opponent_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let id = g.add_card_to_battlefield(0, catalog::angel_of_finality());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.is_empty(), "opponent graveyard exiled");
}

/// Bishop's Soldier has lifelink.
#[test]
fn bishops_soldier_has_lifelink() {
    let d = catalog::bishops_soldier();
    assert!(d.keywords.contains(&Keyword::Lifelink));
}

/// Affectionate Indrik fights a creature you don't control on ETB.
#[test]
fn affectionate_indrik_fights() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_battlefield(0, catalog::affectionate_indrik()); // 4/4
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "2/2 died to the 4/4's fight");
}

/// Angelic Edict exiles a target creature.
#[test]
fn angelic_edict_exiles_creature() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::angelic_edict());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(creature)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Angelic Edict");
    drain_stack(&mut g);
    assert!(g.battlefield_find(creature).is_none(), "creature exiled");
    assert!(g.exile.iter().any(|c| c.id == creature));
}

/// Broken Wings destroys a creature with flying.
#[test]
fn broken_wings_destroys_flyer() {
    let mut g = two_player_game();
    let flyer = g.add_card_to_battlefield(1, catalog::crow_of_dark_tidings()); // 2/1 flyer
    let id = g.add_card_to_hand(0, catalog::broken_wings());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(flyer)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Broken Wings");
    drain_stack(&mut g);
    assert!(g.battlefield_find(flyer).is_none(), "flyer destroyed");
}

/// Ambush Wolf has flash and exiles a graveyard card on ETB.
#[test]
fn ambush_wolf_flash_and_exiles_gy_card() {
    let mut g = two_player_game();
    assert!(catalog::ambush_wolf().keywords.contains(&Keyword::Flash));
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::ambush_wolf());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.is_empty(), "the single gy card was exiled");
}

/// Crackling Cyclops grows +3/+0 when you cast a noncreature spell.
#[test]
fn crackling_cyclops_pumps_on_noncreature_cast() {
    let mut g = two_player_game();
    let cyc = g.add_card_to_battlefield(0, catalog::crackling_cyclops());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bolt");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(cyc).unwrap().power, 3, "+3/+0 from the noncreature cast");
}

/// Angel of Vitality grants +1 life and gets +2/+2 at 25+ life.
#[test]
fn angel_of_vitality_lifegain_and_threshold() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(0, catalog::angel_of_vitality());
    g.players[0].life = 20;
    assert_eq!(g.computed_permanent(angel).unwrap().power, 2, "no buff below 25");
    // Gain 4 → bonus makes it 5 → life 25 → threshold met.
    g.adjust_life_applied(0, 4);
    assert_eq!(g.players[0].life, 25, "gained 4 plus 1");
    assert_eq!(g.computed_permanent(angel).unwrap().power, 4, "+2/+2 at 25 life");
}

/// Basilisk Collar grants deathtouch and lifelink to the equipped creature.
#[test]
fn basilisk_collar_grants_deathtouch_lifelink() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let collar = g.add_card_to_battlefield(0, catalog::basilisk_collar());
    g.battlefield_find_mut(collar).unwrap().attached_to = Some(bear);
    let kws = &g.computed_permanent(bear).unwrap().keywords;
    assert!(kws.contains(&Keyword::Deathtouch) && kws.contains(&Keyword::Lifelink));
}

/// Archway Angel gains 2 life per Gate on ETB.
#[test]
fn archway_angel_gains_life_per_gate() {
    let mut g = two_player_game();
    // Azorius Guildgate-style Gate — give it the Gate land type directly.
    let mut gate = catalog::forest();
    gate.subtypes.land_types = vec![crate::card::LandType::Gate];
    g.add_card_to_battlefield(0, gate.clone());
    g.add_card_to_battlefield(0, gate);
    let angel = g.add_card_to_battlefield(0, catalog::archway_angel());
    let life = g.players[0].life;
    g.fire_self_etb_triggers(angel, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 4, "2 life per Gate, two Gates");
}

/// Cemetery Recruitment returns a creature; a Zombie also draws.
#[test]
fn cemetery_recruitment_returns_and_zombie_draws() {
    let mut g = two_player_game();
    let crow = g.add_card_to_graveyard(0, catalog::crow_of_dark_tidings()); // a Zombie
    g.add_card_to_library(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::cemetery_recruitment());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(crow)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Cemetery Recruitment");
    drain_stack(&mut g);
    // +1 returned creature, +1 drawn (Zombie), -1 the spell itself left hand.
    assert_eq!(g.players[0].hand.len(), hand + 1, "returned Zombie + drew a card");
    assert!(g.players[0].graveyard.iter().all(|c| c.definition.name != "Crow of Dark Tidings"));
}

/// Seasoned Hallowblade discards a card to gain indestructible.
#[test]
fn seasoned_hallowblade_discards_for_indestructible() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::grizzly_bears()); // discard fodder
    let blade = g.add_card_to_battlefield(0, catalog::seasoned_hallowblade());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: blade, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Hallowblade");
    drain_stack(&mut g);
    assert!(g.computed_permanent(blade).unwrap().keywords.contains(&Keyword::Indestructible));
    assert!(g.battlefield_find(blade).unwrap().tapped, "it taps itself");
}

/// Dryad Greenseeker reveals the top land into hand.
#[test]
fn dryad_greenseeker_reveals_top_land() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let dryad = g.add_card_to_battlefield(0, catalog::dryad_greenseeker());
    g.clear_sickness(dryad);
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: dryad, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Greenseeker");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "top land went to hand");
}

/// Aggressive Mammoth grants trample to your other creatures.
#[test]
fn aggressive_mammoth_grants_team_trample() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::aggressive_mammoth());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample));
}

/// Scrabbling Claws sacrifices to exile a graveyard card and draw.
#[test]
fn scrabbling_claws_sac_exiles_and_draws() {
    let mut g = two_player_game();
    let claws = g.add_card_to_battlefield(0, catalog::scrabbling_claws());
    let victim = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: claws, ability_index: 1, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], x_value: None,
    }).expect("activate sac ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(claws).is_none(), "sacrificed");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    assert!(g.players[1].graveyard.is_empty(), "gy card exiled");
}

/// A Pirate-typed Cyclops is unaffected — sanity that the Pirate anthem is typed.
#[test]
fn corsair_anthem_is_pirate_typed() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::corsair_captain());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // not a Pirate
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "non-Pirate unbuffed");
}

/// CR 702.38 — Amplify N: the creature enters with N +1/+1 counters for each
/// matching card revealed in hand. Feral Throwback (Amplify 2, Beast) with two
/// Beasts in hand enters as a 3/3 + 4 counters = 7/7.
#[test]
fn cr_702_38_amplify_counts_revealed_hand_cards() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::canopy_crawler()); // a Beast
    g.add_card_to_hand(0, catalog::feral_throwback()); // also a Beast
    g.add_card_to_hand(0, catalog::lightning_bolt()); // not a Beast — ignored
    let id = g.move_card_to_battlefield_for_test(0, catalog::feral_throwback());
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 7), "3/3 base + 2×2 Beast counters");
}

/// CR 702.38 — Amplify with no matching cards in hand leaves the base body.
#[test]
fn cr_702_38_amplify_no_reveals_stays_base() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::kilnmouth_dragon());
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "no Dragons in hand → base 5/5");
}

/// CR 301.5 — "equipped by N": Balan's double strike keys on the attached
/// Equipment count, dropping when an Equipment is removed.
#[test]
fn cr_301_5_equipped_by_count_gates_keyword() {
    let mut g = two_player_game();
    let balan = g.add_card_to_battlefield(0, catalog::balan_wandering_knight());
    let e1 = g.add_card_to_battlefield(0, catalog::bonesplitter());
    let e2 = g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.battlefield_find_mut(e1).unwrap().attached_to = Some(balan);
    g.battlefield_find_mut(e2).unwrap().attached_to = Some(balan);
    assert!(g.computed_permanent(balan).unwrap().keywords.contains(&Keyword::DoubleStrike));
    // Detach one → only one Equipment left → no double strike.
    g.battlefield_find_mut(e2).unwrap().attached_to = None;
    assert!(!g.computed_permanent(balan).unwrap().keywords.contains(&Keyword::DoubleStrike));
}

/// CR 119 — a life-total threshold static (Angel of Vitality's +2/+2 at 25+
/// life) turns on and off as life crosses the boundary.
#[test]
fn cr_119_life_threshold_static_toggles() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(0, catalog::angel_of_vitality());
    g.players[0].life = 24;
    assert_eq!(g.computed_permanent(angel).unwrap().power, 2, "below 25");
    g.players[0].life = 25;
    assert_eq!(g.computed_permanent(angel).unwrap().power, 4, "at 25");
    g.players[0].life = 24;
    assert_eq!(g.computed_permanent(angel).unwrap().power, 2, "back below 25");
}
