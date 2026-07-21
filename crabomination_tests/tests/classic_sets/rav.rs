//! Functionality tests for Ravnica: City of Guilds (RAV) gap cards.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::*;
use crabomination::mana::Color;

/// Static stat/keyword lines for the vanilla and french-vanilla RAV creatures.
#[test]
fn rav_stat_and_keyword_lines() {
    let sword = catalog::boros_swiftblade();
    assert_eq!((sword.power, sword.toughness), (1, 2));
    assert!(sword.keywords.contains(&Keyword::DoubleStrike));

    let hawk = catalog::courier_hawk();
    assert_eq!((hawk.power, hawk.toughness), (1, 2));
    assert!(hawk.keywords.contains(&Keyword::Flying) && hawk.keywords.contains(&Keyword::Vigilance));

    let eq = catalog::conclave_equenaut();
    assert!(eq.keywords.contains(&Keyword::Convoke) && eq.keywords.contains(&Keyword::Flying));
}

/// Barbarian Riftcutter sacrifices itself to destroy a land.
#[test]
fn barbarian_riftcutter_destroys_a_land() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let rift = g.add_card_to_battlefield(0, catalog::barbarian_riftcutter());
    g.clear_sickness(rift);
    let land = g.add_card_to_battlefield(1, catalog::forest());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rift, ability_index: 0,
        target: Some(Target::Permanent(land)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate Riftcutter");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "target land destroyed");
    assert!(g.battlefield_find(rift).is_none(), "Riftcutter sacrificed as a cost");
}

/// Dromad Purebred gains its controller 1 life whenever it's dealt damage.
#[test]
fn dromad_purebred_gains_life_on_damage() {
    let mut g = two_player_game();
    let dromad = g.add_card_to_battlefield(0, catalog::dromad_purebred());
    let life0 = g.players[0].life;
    let mut evs = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(dromad), 3, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 1, "gained 1 life (once) from being dealt damage");
}

/// Gate Hound grants your creatures vigilance only while it's enchanted.
#[test]
fn gate_hound_team_vigilance_while_enchanted() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let hound = g.add_card_to_battlefield(0, catalog::gate_hound());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Vigilance),
        "no vigilance while unenchanted");
    // Enchant the hound with an aura (Pacifism), turning on the static.
    let pacifism = g.add_card_to_hand(0, catalog::pacifism());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: pacifism, target: Some(Target::Permanent(hound)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("attach aura to Gate Hound");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Vigilance),
        "your creatures gain vigilance while Gate Hound is enchanted");
}

// ─── Ravnica-block gap cards (gaps) ─────────────────────────────────────────

/// The guild bounce-lands enter tapped, bounce a land, and tap for two colors.
#[test]
fn bounce_land_enters_tapped_and_bounces() {
    let mut g = two_player_game();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.move_card_to_battlefield_for_test(0, catalog::dimir_aqueduct());
    drain_stack(&mut g);
    let aqueduct = g.battlefield.iter().find(|c| c.definition.name == "Dimir Aqueduct").unwrap();
    assert!(aqueduct.tapped, "enters tapped");
    assert!(g.players[0].hand.iter().any(|c| c.id == forest), "returned a land to hand");
}

/// Benevolent Ancestor's tap prevents the next 1 damage to a chosen target.
#[test]
fn benevolent_ancestor_prevents_one_damage() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let anc = g.add_card_to_battlefield(0, catalog::benevolent_ancestor());
    g.clear_sickness(anc);
    g.perform_action(GameAction::ActivateAbility {
        card_id: anc, ability_index: 0,
        target: Some(Target::Player(0)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate prevention");
    drain_stack(&mut g);
    let life0 = g.players[0].life;
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Player(0), 3, None, &mut evs);
    assert_eq!(g.players[0].life, life0 - 2, "1 of the 3 damage was prevented");
}

/// Carrion Howler pumps +2/-1 for each life paid.
#[test]
fn carrion_howler_pump() {
    let mut g = two_player_game();
    let howler = g.add_card_to_battlefield(0, catalog::carrion_howler());
    g.clear_sickness(howler);
    g.perform_action(GameAction::ActivateAbility {
        card_id: howler, ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("pay 1 life to pump");
    drain_stack(&mut g);
    let c = g.computed_permanent(howler).unwrap();
    assert_eq!((c.power, c.toughness), (4, 1), "+2/-1 applied");
    assert_eq!(g.players[0].life, 19, "paid 1 life");
}

/// Conclave Phalanx gains life for each creature you control on entry.
#[test]
fn conclave_phalanx_gains_life_per_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let life0 = g.players[0].life;
    g.move_card_to_battlefield_for_test(0, catalog::conclave_phalanx());
    drain_stack(&mut g);
    // 1 bear + Phalanx itself = 2 creatures → gain 2.
    assert_eq!(g.players[0].life, life0 + 2, "gained life per creature controlled");
}

/// Dogpile deals damage equal to your attacking creatures.
#[test]
fn dogpile_scales_with_attackers() {
    use crabomination::game::types::{Attack, AttackTarget, Target};
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(a);
    g.clear_sickness(b);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ]).expect("declare two attackers");
    let dogpile = g.add_card_to_hand(0, catalog::dogpile());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Red, 1);
    let foe_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: dogpile, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Dogpile");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe_life - 2, "2 damage for two attackers");
}

/// Dimir Cutpurse's combat damage makes the player discard and draws for you.
#[test]
fn dimir_cutpurse_discard_and_draw() {
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let cut = g.add_card_to_battlefield(0, catalog::dimir_cutpurse());
    g.clear_sickness(cut);
    g.add_card_to_hand(1, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    let h1 = g.players[1].hand.len();
    let h0 = g.players[0].hand.len();
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: cut, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), h1 - 1, "opponent discarded");
    assert_eq!(g.players[0].hand.len(), h0 + 1, "you drew");
}

/// Clinging Darkness saps the enchanted creature -4/-1.
#[test]
fn clinging_darkness_weakens() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4
    let aura = g.add_card_to_hand(0, catalog::clinging_darkness());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("attach Clinging Darkness");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (2, 3), "-4/-1 applied");
}

/// Consult the Necrosages' draw mode gives the target player two cards.
#[test]
fn consult_necrosages_draw_mode() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let consult = g.add_card_to_hand(0, catalog::consult_the_necrosages());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let h0 = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: consult, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast Consult in draw mode");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h0 - 1 + 2, "drew two (net +1 after casting)");
}

/// Caregiver sacrifices a creature to set up a 1-damage prevention shield.
#[test]
fn caregiver_prevents_one_damage() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let cg = g.add_card_to_battlefield(0, catalog::caregiver());
    g.clear_sickness(cg);
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // sac fodder
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cg, ability_index: 0,
        target: Some(Target::Player(0)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate Caregiver");
    drain_stack(&mut g);
    let life0 = g.players[0].life;
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Player(0), 2, None, &mut evs);
    assert_eq!(g.players[0].life, life0 - 1, "1 of the 2 damage prevented");
}

/// Cerulean Sphinx shuffles itself back into its owner's library.
#[test]
fn cerulean_sphinx_shuffles_self_away() {
    let mut g = two_player_game();
    let sphinx = g.add_card_to_battlefield(0, catalog::cerulean_sphinx());
    g.clear_sickness(sphinx);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sphinx, ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate shuffle");
    drain_stack(&mut g);
    assert!(g.battlefield_find(sphinx).is_none(), "Sphinx left the battlefield");
    assert!(g.players[0].library.iter().any(|c| c.id == sphinx), "back in the library");
}

/// Drooling Groodion buffs one creature and shrinks another.
#[test]
fn drooling_groodion_split_pump() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let groo = g.add_card_to_battlefield(0, catalog::drooling_groodion());
    g.clear_sickness(groo);
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // sac fodder
    let mine = g.add_card_to_battlefield(0, catalog::craw_wurm()); // 6/4
    let theirs = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: groo, ability_index: 0,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], x_value: None,
    }).expect("activate Groodion");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(mine).map(|c| (c.power, c.toughness)), Some((8, 6)), "+2/+2");
    assert_eq!(g.computed_permanent(theirs).map(|c| (c.power, c.toughness)), Some((4, 2)), "-2/-2");
}

/// Dryad's Caress gains life per creature and untaps your team when {W} was spent.
#[test]
fn dryads_caress_lifegain_and_white_untap() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(mine).unwrap().tapped = true;
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let caress = g.add_card_to_hand(0, catalog::dryads_caress());
    // Pay the {4} generic with white mana so {W} counts as spent.
    g.players[0].mana_pool.add(Color::White, 4);
    g.players[0].mana_pool.add(Color::Green, 2);
    let life0 = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: caress, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Dryad's Caress paying white");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 2, "gained 1 per creature on the battlefield");
    assert!(!g.battlefield_find(mine).unwrap().tapped, "white spent -> untapped your creatures");
}

/// Empty the Catacombs returns every graveyard creature to its owner's hand.
#[test]
fn empty_the_catacombs_returns_all() {
    let mut g = two_player_game();
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(1, catalog::craw_wurm());
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // a noncreature stays
    let spell = g.add_card_to_hand(0, catalog::empty_the_catacombs());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Empty the Catacombs");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == a), "your creature returned");
    assert!(g.players[1].hand.iter().any(|c| c.id == b), "their creature returned");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Lightning Bolt"),
        "the noncreature card stayed in the graveyard");
}

/// Conclave's Blessing grants +0/+2 for each creature you control.
#[test]
fn conclaves_blessing_scales_toughness() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::conclaves_blessing());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(host)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("attach Conclave's Blessing");
    drain_stack(&mut g);
    // Three creatures you control → +0/+6.
    assert_eq!(g.computed_permanent(host).map(|c| c.toughness), Some(8), "+2 toughness per creature");
}

/// Autochthon Wurm is a 9/14 with Convoke and Trample.
#[test]
fn autochthon_wurm_stats_and_keywords() {
    let w = catalog::autochthon_wurm();
    assert_eq!((w.power, w.toughness), (9, 14));
    assert!(w.keywords.contains(&Keyword::Convoke));
    assert!(w.keywords.contains(&Keyword::Trample));
}

/// Cackling Imp's tap ability drains a target player 1 life.
#[test]
fn cackling_imp_drains_a_life() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let imp = g.add_card_to_battlefield(0, catalog::cackling_imp());
    g.clear_sickness(imp);
    let foe_life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: imp, ability_index: 0,
        target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate Cackling Imp");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe_life - 1, "target player lost 1 life");
    assert!(g.battlefield_find(imp).unwrap().tapped, "tapped as the cost");
}
