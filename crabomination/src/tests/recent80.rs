//! Functionality tests for `catalog::sets::decks::recent80`.

use crate::catalog;
use crate::game::two_player_game;
use crate::game::types::{Attack, AttackTarget, Target, TurnStep};
use crate::game::*;

#[test]
fn bonehoard_scales_germ_by_creature_cards_in_all_graveyards() {
    let mut g = two_player_game();
    // Two creature cards in p0's graveyard, one in p1's = 3 creature cards
    // across all graveyards.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let bh = g.add_card_to_hand(0, catalog::bonehoard());
    g.players[0].mana_pool.add_colorless(4);
    // Cast Bonehoard → living-weapon ETB mints a Germ and attaches.
    cast(&mut g, bh);
    let germ = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Phyrexian Germ" && c.controller == 0)
        .expect("living weapon mints a Germ");
    assert_eq!(
        g.battlefield_find(bh).unwrap().attached_to,
        Some(germ.id),
        "Bonehoard attaches to its Germ"
    );
    let cp = g.computed_permanent(germ.id).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+X/+X, X = creature cards in all graveyards");
}

#[test]
fn necropolis_fiend_debuffs_by_exiled_count() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let fiend = g.add_card_to_battlefield(0, catalog::necropolis_fiend());
    g.clear_sickness(fiend);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    // {2}, {T}, Exile 2 cards from graveyard: victim gets -2/-2.
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: fiend,
        ability_index: 0,
        target: Some(Target::Permanent(victim)),
        additional_targets: Vec::new(),
        x_value: Some(2),
    })
    .expect("activate -X/-X for X=2");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 1, "two of three graveyard cards exiled as a cost");
    // 2/2 with -2/-2 → 0 toughness → dies as an SBA.
    assert!(
        !g.battlefield.iter().any(|c| c.id == victim),
        "the -2/-2'd 2/2 dies to the state-based check"
    );
}

#[test]
fn charmed_sleep_taps_and_locks_untap() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::charmed_sleep());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(foe)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Charmed Sleep castable for {1}{U}{U}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "ETB taps the enchanted creature");
    // The enchanted creature's controller's untap step must not free it.
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(foe).unwrap().tapped, "enchanted creature doesn't untap");
}

#[test]
fn ghost_ship_regenerates() {
    let mut g = two_player_game();
    let ship = g.add_card_to_battlefield(0, catalog::ghost_ship());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 3);
    // {U}{U}{U}: set up a regeneration shield.
    g.perform_action(GameAction::ActivateAbility {
        card_id: ship,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("regenerate");
    drain_stack(&mut g);
    // Lethal damage is replaced by the shield: the Ship survives.
    g.battlefield_find_mut(ship).unwrap().damage = 99;
    let sba = g.check_state_based_actions();
    let _ = sba;
    assert!(g.battlefield.iter().any(|c| c.id == ship), "regeneration shield saves the Ship");
}

#[test]
fn serpent_assassin_etb_destroys_nonblack() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let white = g.add_card_to_battlefield(1, catalog::serra_angel()); // nonblack
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let assassin = g.add_card_to_hand(0, catalog::serpent_assassin());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: assassin,
        target: Some(Target::Permanent(white)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Serpent Assassin castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == white), "ETB destroys the nonblack creature");
}

#[test]
fn sea_monster_needs_defender_island() {
    let mut g = two_player_game();
    let sm = g.add_card_to_battlefield(0, catalog::sea_monster());
    g.clear_sickness(sm);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: sm,
            target: AttackTarget::Player(1),
        }]))
        .is_err(),
        "can't attack a defender with no Island"
    );
    g.add_card_to_battlefield(1, catalog::island());
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: sm,
            target: AttackTarget::Player(1),
        }]))
        .is_ok(),
        "may attack once the defender controls an Island"
    );
}

#[test]
fn sea_serpent_needs_defender_island_to_attack() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::island()); // p0 keeps its Island so it survives upkeep
    let serp = g.add_card_to_battlefield(0, catalog::sea_serpent());
    g.clear_sickness(serp);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: serp,
            target: AttackTarget::Player(1),
        }]))
        .is_err(),
        "can't attack a defender with no Island"
    );
    g.add_card_to_battlefield(1, catalog::island());
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: serp,
            target: AttackTarget::Player(1),
        }]))
        .is_ok(),
        "may attack once the defender controls an Island"
    );
}

#[test]
fn caustic_bronco_unsaddled_hits_you_saddled_hits_opponent() {
    // Unsaddled: the controller loses life equal to the revealed card's MV.
    let mut g = two_player_game();
    let bronco = g.add_card_to_battlefield(0, catalog::caustic_bronco());
    g.clear_sickness(bronco);
    g.add_card_to_library(0, catalog::grizzly_bears()); // top card, MV 2
    let my_life = g.players[0].life;
    let opp_life = g.players[1].life;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bronco,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, my_life - 2, "unsaddled: you lose the revealed card's MV");
    assert_eq!(g.players[1].life, opp_life, "opponent untouched while unsaddled");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"), "revealed card to hand");
}

#[test]
fn caustic_bronco_saddled_hits_opponent() {
    let mut g = two_player_game();
    let bronco = g.add_card_to_battlefield(0, catalog::caustic_bronco());
    g.clear_sickness(bronco);
    g.battlefield_find_mut(bronco).unwrap().saddled = true;
    g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2
    let my_life = g.players[0].life;
    let opp_life = g.players[1].life;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bronco,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 2, "saddled: each opponent loses the MV");
    assert_eq!(g.players[0].life, my_life, "you untouched while saddled");
}

#[test]
fn goblin_recruiter_stacks_goblins_on_top() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // A non-Goblin on top, then a Goblin deeper in the library.
    g.add_card_to_library(0, catalog::grizzly_bears());
    let goblin = g.add_card_to_library(0, catalog::goblin_recruiter());
    let rec = g.add_card_to_hand(0, catalog::goblin_recruiter());
    // Search picks the Goblin, then stops.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(goblin)),
        DecisionAnswer::Search(None),
    ]));
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, rec);
    // ETB tutors the Goblin to the top of the library.
    let top = g.players[0].library.first().expect("library non-empty");
    assert_eq!(top.id, goblin, "the searched-up Goblin sits on top");
}

#[test]
fn sea_serpent_sacrificed_when_you_control_no_island() {
    let mut g = two_player_game();
    let serp = g.add_card_to_battlefield(0, catalog::sea_serpent());
    g.clear_sickness(serp);
    g.step = TurnStep::Cleanup;
    for _ in 0..30 {
        if !g.battlefield.iter().any(|c| c.id == serp) {
            break;
        }
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    assert!(
        !g.battlefield.iter().any(|c| c.id == serp),
        "Sea Serpent sacrifices itself at upkeep with no Island"
    );
}
