//! Saviors of Kamigawa (SOK) gap closure.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn always_yes(g: &mut GameState) {
    g.decider = Box::new(ScriptedDecider::new(
        std::iter::repeat_with(|| DecisionAnswer::Bool(true)).take(8),
    ));
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

/// Every SOK factory is registered under its printed name.
#[test]
fn sok_cards_are_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for f in [
        catalog::bounteous_kirin as fn() -> crabomination::card::CardDefinition,
        catalog::celestial_kirin,
        catalog::cloudhoof_kirin,
        catalog::arashi_the_sky_asunder,
        catalog::ghost_lit_nourisher,
        catalog::briarknit_kami,
        catalog::dreamcatcher,
        catalog::elder_pine_of_jukai,
        catalog::fiddlehead_kami,
        catalog::deathmask_nezumi,
        catalog::gnat_miser,
        catalog::ebony_owl_netsuke,
        catalog::gaze_of_adamaro,
        catalog::descendant_of_soramaro,
        catalog::death_of_a_thousand_stings,
        catalog::aether_shockwave,
        catalog::araba_mothrider,
        catalog::ayumi_the_last_visitor,
        catalog::burning_eye_zubera,
        catalog::captive_flame,
        catalog::cut_the_earthly_bond,
        catalog::death_denied,
        catalog::deathknell_kami,
        catalog::dense_canopy,
        catalog::dosans_oldest_chant,
        catalog::eiganjo_free_riders,
        catalog::feral_lightning,
        catalog::freed_from_the_real,
        catalog::glitterfang,
        catalog::godos_irregulars,
        catalog::blood_clock,
        catalog::evermind,
        catalog::descendant_of_masumaro,
    ] {
        let name = f().name;
        assert!(names.contains(&name), "{name} is not registered");
    }
}

/// Bounteous Kirin pays out that spell's mana value in life.
#[test]
fn bounteous_kirin_gains_the_spells_mana_value() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bounteous_kirin());
    let ray = g.add_card_to_hand(0, catalog::glacial_ray()); // {1}{R} Arcane
    g.players[0].mana_pool.add(Color::Red, 2);
    always_yes(&mut g);
    let life = g.players[0].life;
    cast(&mut g, ray, Some(Target::Player(1)));
    assert_eq!(g.players[0].life, life + 2);
}

/// Celestial Kirin wipes every permanent sharing the spell's mana value.
#[test]
fn celestial_kirin_wipes_the_matching_mana_value() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::celestial_kirin());
    let two_drop = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let one_drop = g.add_card_to_battlefield(1, catalog::memnite()); // MV 0
    let ray = g.add_card_to_hand(0, catalog::glacial_ray()); // MV 2
    g.players[0].mana_pool.add(Color::Red, 2);
    cast(&mut g, ray, Some(Target::Player(1)));
    assert!(g.battlefield_find(two_drop).is_none(), "the MV-2 Bear died");
    assert!(g.battlefield_find(one_drop).is_some(), "the MV-0 artifact survived");
}

/// Arashi channels out of hand to sweep the skies.
#[test]
fn arashi_channels_from_hand() {
    let mut g = two_player_game();
    let arashi = g.add_card_to_hand(0, catalog::arashi_the_sky_asunder());
    let flier = g.add_card_to_battlefield(1, catalog::serra_angel());
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 6);
    g.perform_action(GameAction::ActivateAbility {
        card_id: arashi,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(4),
    })
    .expect("channel");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == arashi), "discarded as a cost");
    assert!(g.battlefield_find(flier).is_none(), "the 4/4 flier ate 4");
    assert!(g.battlefield_find(ground).is_some(), "the ground creature is untouched");
}

/// Deathmask Nezumi grows and gains fear on a full hand.
#[test]
fn deathmask_nezumi_scales_with_your_hand() {
    let mut g = two_player_game();
    let rat = g.add_card_to_battlefield(0, catalog::deathmask_nezumi());
    assert_eq!(g.computed_permanent(rat).unwrap().power, 2);
    for _ in 0..7 {
        g.add_card_to_hand(0, catalog::forest());
    }
    let cp = g.computed_permanent(rat).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3));
    assert!(cp.keywords.contains(&Keyword::Fear));
}

/// Gnat Miser shaves a card off each opponent's hand size.
#[test]
fn gnat_miser_shrinks_opponent_hand_size() {
    let mut g = two_player_game();
    let base = g.effective_max_hand_size(1).unwrap();
    g.add_card_to_battlefield(0, catalog::gnat_miser());
    assert_eq!(g.effective_max_hand_size(1).unwrap(), base - 1);
    assert_eq!(g.effective_max_hand_size(0).unwrap(), base, "your own hand size is untouched");
}

/// Ebony Owl Netsuke punishes an opponent hoarding seven cards.
#[test]
fn ebony_owl_netsuke_burns_a_full_hand() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ebony_owl_netsuke());
    for _ in 0..7 {
        g.add_card_to_hand(1, catalog::forest());
    }
    g.active_player_idx = 1;
    let life = g.players[1].life;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 4);
}

/// Gaze of Adamaro scales with the target's own hand.
#[test]
fn gaze_of_adamaro_burns_for_their_hand_size() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_hand(1, catalog::forest());
    }
    let gaze = g.add_card_to_hand(0, catalog::gaze_of_adamaro());
    g.players[0].mana_pool.add(Color::Red, 4);
    let life = g.players[1].life;
    cast(&mut g, gaze, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 5);
}

/// Ayumi can't be blocked while the defender has a legendary land.
#[test]
fn ayumi_walks_past_legendary_lands() {
    let ayumi = catalog::ayumi_the_last_visitor();
    assert!(ayumi.keywords.iter().any(|k| matches!(k, Keyword::LandwalkFiltered(_))));
}

/// Burning-Eye Zubera only goes off if it soaked four damage.
#[test]
fn burning_eye_zubera_needs_four_damage() {
    let shoot = |damage: u32| {
        let mut g = two_player_game();
        let zubera = g.add_card_to_battlefield(0, catalog::burning_eye_zubera());
        let mut evs = vec![];
        g.deal_damage_to_from(
            crabomination::game::effects::EntityRef::Permanent(zubera),
            damage,
            None,
            &mut evs,
        );
        let mut sba = g.check_state_based_actions();
        evs.append(&mut sba);
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        20 - g.players[1].life
    };
    assert_eq!(shoot(3), 0, "a 3-damage bolt kills it without the payoff");
    assert_eq!(shoot(4), 3, "4 damage turns it on");
}

/// Cut the Earthly Bond bounces whatever is wearing an Aura.
#[test]
fn cut_the_earthly_bond_bounces_the_enchanted_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(1, catalog::pacifism());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    let cut = g.add_card_to_hand(0, catalog::cut_the_earthly_bond());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, cut, Some(Target::Permanent(bear)));
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
}

/// Death Denied rakes X creatures out of the graveyard.
#[test]
fn death_denied_returns_x_creatures() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let denied = g.add_card_to_hand(0, catalog::death_denied());
    g.players[0].mana_pool.add(Color::Black, 4);
    g.perform_action(GameAction::CastSpell {
        card_id: denied,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.iter().filter(|c| c.definition.is_creature()).count(), 2);
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Lightning Bolt"));
}

/// Dense Canopy pins fliers to blocking only fliers.
#[test]
fn dense_canopy_grounds_flying_blockers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dense_canopy());
    let flier = g.add_card_to_battlefield(1, catalog::serra_angel());
    assert!(
        g.computed_permanent(flier).unwrap().keywords.contains(&Keyword::CanBlockOnlyFlying)
    );
}

/// Feral Lightning's tokens burn out at the end step.
#[test]
fn feral_lightning_tokens_exile_at_end_of_turn() {
    let mut g = two_player_game();
    let feral = g.add_card_to_hand(0, catalog::feral_lightning());
    g.players[0].mana_pool.add(Color::Red, 6);
    cast(&mut g, feral, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 3);
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 0);
}

/// Glitterfang goes home at every end step.
#[test]
fn glitterfang_returns_to_hand_at_end_of_turn() {
    let mut g = two_player_game();
    let fang = g.add_card_to_battlefield(0, catalog::glitterfang());
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == fang));
}

/// Blood Clock taxes each upkeep two life or a permanent.
#[test]
fn blood_clock_takes_two_life_or_a_permanent() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::blood_clock());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // The AutoDecider declines to pay, so the permanent bounces.
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
}

/// Evermind has no mana cost — it's only reachable as a splice.
#[test]
fn evermind_is_uncastable_but_splices() {
    let mut g = two_player_game();
    let evermind = g.add_card_to_hand(0, catalog::evermind());
    let ray = g.add_card_to_hand(0, catalog::glacial_ray());
    g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add(Color::Blue, 2);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: evermind,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "a nonexistent mana cost can't be paid"
    );
    g.perform_action(GameAction::CastSpellSpliced {
        card_id: ray,
        splice_cards: vec![evermind],
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("spliced");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.iter().filter(|c| c.id == evermind).count(), 1);
    assert_eq!(g.players[0].cards_drawn_this_turn, 1);
}

/// Descendant of Masumaro swings on the hand-size gap.
#[test]
fn descendant_of_masumaro_tracks_the_hand_gap() {
    let mut g = two_player_game();
    let monk = g.add_card_to_battlefield(0, catalog::descendant_of_masumaro());
    for _ in 0..4 {
        g.add_card_to_hand(0, catalog::forest());
    }
    g.add_card_to_hand(1, catalog::forest());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(monk).unwrap().counter_count(CounterType::PlusOnePlusOne),
        3,
        "4 in your hand minus 1 in theirs"
    );
}

/// Death of a Thousand Stings crawls back while you're ahead on cards.
#[test]
fn death_of_a_thousand_stings_recurs_with_hand_advantage() {
    let mut g = two_player_game();
    let sting = g.add_card_to_graveyard(0, catalog::death_of_a_thousand_stings());
    g.add_card_to_hand(0, catalog::forest());
    always_yes(&mut g);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == sting));
}
