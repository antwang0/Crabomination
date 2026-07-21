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

// ── RAV gap wave 2 ───────────────────────────────────────────────────────────

/// Stat/keyword lines for the vanilla and french-vanilla gap-wave-2 creatures.
#[test]
fn rav_gap2_stat_lines() {
    let golem = catalog::glass_golem();
    assert_eq!((golem.power, golem.toughness), (6, 2));
    assert!(golem.card_types.contains(&crabomination::card::CardType::Artifact));

    let spider = catalog::goliath_spider();
    assert_eq!((spider.power, spider.toughness), (7, 6));
    assert!(spider.keywords.contains(&Keyword::Reach));

    let gharial = catalog::grayscaled_gharial();
    assert!(gharial.keywords.iter().any(|k| matches!(k,
        Keyword::Landwalk(crabomination::card::LandType::Island))));

    let fiend = catalog::goblin_fire_fiend();
    assert!(fiend.keywords.contains(&Keyword::Haste) && fiend.keywords.contains(&Keyword::MustBeBlocked));
}

/// Greater Forgeling pumps +3/-3 until end of turn.
#[test]
fn greater_forgeling_pumps() {
    let mut g = two_player_game();
    let f = g.add_card_to_battlefield(0, catalog::greater_forgeling());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: f, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    let cp = g.computed_permanent(f).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 1), "3/4 → 6/1 after +3/-3");
}

/// Centaur Safeguard's death trigger gains 3 life when accepted.
#[test]
fn centaur_safeguard_gains_on_death() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::centaur_safeguard());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life0 = g.players[0].life;
    g.battlefield_find_mut(c).unwrap().damage = 1; // lethal vs 3/1
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 3, "gained 3 on death");
}

/// Blazing Archon — creatures can't attack the player who controls it.
#[test]
fn blazing_archon_bars_attacks() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::blazing_archon()); // P0 is protected
    g.active_player_idx = 1;
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    let res = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(0),
    }]));
    assert!(res.is_err(), "the attack is barred by Blazing Archon");
}

/// Sell-Sword Brute deals 2 to its controller when it dies.
#[test]
fn sell_sword_brute_burns_controller_on_death() {
    let mut g = two_player_game();
    let brute = g.add_card_to_battlefield(0, catalog::sell_sword_brute());
    let life0 = g.players[0].life;
    g.battlefield_find_mut(brute).unwrap().damage = 2;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 - 2, "controller took 2 on death");
}

/// Infectious Host drains a target player 2 when it dies.
#[test]
fn infectious_host_drains_on_death() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::infectious_host());
    let life1 = g.players[1].life;
    g.battlefield_find_mut(host).unwrap().damage = 1;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 2, "target player lost 2");
}

/// Roofstalker Wight grants itself flying until end of turn.
#[test]
fn roofstalker_wight_grants_flying() {
    let mut g = two_player_game();
    let w = g.add_card_to_battlefield(0, catalog::roofstalker_wight());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(!g.computed_permanent(w).unwrap().keywords.contains(&Keyword::Flying));
    g.perform_action(GameAction::ActivateAbility {
        card_id: w, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("grant flying");
    drain_stack(&mut g);
    assert!(g.computed_permanent(w).unwrap().keywords.contains(&Keyword::Flying), "has flying now");
}

/// Sewerdreg sacrifices itself to exile a card from a graveyard.
#[test]
fn sewerdreg_exiles_a_graveyard_card() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let dreg = g.add_card_to_battlefield(0, catalog::sewerdreg());
    let corpse = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: dreg, ability_index: 0,
        target: Some(Target::Permanent(corpse)), additional_targets: vec![], x_value: None,
    }).expect("sac to exile");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dreg).is_none(), "Sewerdreg sacrificed");
    assert!(g.exile.iter().any(|c| c.id == corpse), "graveyard card exiled");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == corpse));
}

/// Loxodon Gatekeeper taps opponents' permanents as they enter.
#[test]
fn loxodon_gatekeeper_taps_opponents() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::loxodon_gatekeeper());
    let foe = g.move_card_to_battlefield_for_test(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "opponent's creature entered tapped");
}

/// Oathsworn Giant gives other creatures you control +0/+2 and vigilance.
#[test]
fn oathsworn_giant_team_buff() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::oathsworn_giant());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 4), "+0/+2 anthem");
    assert!(cp.keywords.contains(&Keyword::Vigilance), "granted vigilance");
}

/// Moroii's upkeep trigger costs its controller 1 life.
#[test]
fn moroii_upkeep_life_loss() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::moroii());
    let life0 = g.players[0].life;
    g.active_player_idx = 0;
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 - 1, "lost 1 on upkeep");
}

/// Keening Banshee shrinks a creature -2/-2 on entry.
#[test]
fn keening_banshee_shrinks_on_etb() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let banshee = g.add_card_to_hand(0, catalog::keening_banshee());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: banshee, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast banshee");
    drain_stack(&mut g);
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    assert!(g.battlefield_find(bear).is_none(), "2/2 became 0/0 and died");
}

/// Primordial Sage lets you draw when you cast a creature spell.
#[test]
fn primordial_sage_draws_on_creature_cast() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::primordial_sage());
    g.add_card_to_library(0, catalog::forest());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bear");
    drain_stack(&mut g);
    // Cast the bear (-1 from hand) then drew 1 → net same hand size.
    assert_eq!(g.players[0].hand.len(), hand_before, "drew a card off the creature cast");
}

/// Junktroller tucks a graveyard card to the bottom of its owner's library.
#[test]
fn junktroller_tucks_graveyard_card() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let troll = g.add_card_to_battlefield(0, catalog::junktroller());
    g.clear_sickness(troll);
    let corpse = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: troll, ability_index: 0,
        target: Some(Target::Permanent(corpse)), additional_targets: vec![], x_value: None,
    }).expect("tuck");
    drain_stack(&mut g);
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == corpse), "left the graveyard");
    assert!(g.players[1].library.iter().any(|c| c.id == corpse), "on the bottom of library");
}

/// Lore Broker wheels one card for every player.
#[test]
fn lore_broker_each_player_loots() {
    let mut g = two_player_game();
    let broker = g.add_card_to_battlefield(0, catalog::lore_broker());
    g.clear_sickness(broker);
    for p in 0..2 {
        g.add_card_to_hand(p, catalog::grizzly_bears()); // a card to discard
        g.add_card_to_library(p, catalog::forest()); // a card to draw
    }
    let (h0, h1) = (g.players[0].hand.len(), g.players[1].hand.len());
    g.perform_action(GameAction::ActivateAbility {
        card_id: broker, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("wheel");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h0, "P0 drew one, discarded one");
    assert_eq!(g.players[1].hand.len(), h1, "P1 drew one, discarded one");
}

// ── Hunted cycle (gap wave 4) ────────────────────────────────────────────────

/// Stat/keyword lines for the Hunted cycle.
#[test]
fn hunted_cycle_stat_lines() {
    let h = catalog::hunted_horror();
    assert_eq!((h.power, h.toughness), (7, 7));
    assert!(h.keywords.contains(&Keyword::Trample));
    assert!(catalog::hunted_phantasm().keywords.contains(&Keyword::Unblockable));
    let d = catalog::hunted_dragon();
    assert!(d.keywords.contains(&Keyword::Flying) && d.keywords.contains(&Keyword::Haste));
    assert_eq!((catalog::hunted_troll().power, catalog::hunted_troll().toughness), (8, 4));
}

/// Hunted Lammasu's ETB gives a *target opponent* a 4/4 Horror token.
#[test]
fn hunted_lammasu_gifts_opponent_a_horror() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let etb = catalog::hunted_lammasu().triggered_abilities[0].effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Player(1)];
    let evs = g.resolve_effect(&etb, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let horrors: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.name == "Horror").collect();
    assert_eq!(horrors.len(), 1, "opponent got one 4/4 Horror");
    assert_eq!((horrors[0].definition.power, horrors[0].definition.toughness), (4, 4));
}

/// Hunted Horror's ETB gives the opponent two 3/3 Centaurs with protection from black.
#[test]
fn hunted_horror_centaurs_have_pro_black() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let etb = catalog::hunted_horror().triggered_abilities[0].effect.clone();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Player(1)];
    let evs = g.resolve_effect(&etb, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let centaurs: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.name == "Centaur").collect();
    assert_eq!(centaurs.len(), 2, "two Centaur tokens");
    assert!(centaurs[0].definition.keywords.contains(&Keyword::Protection(Color::Black)),
        "pro-black");
}

/// Hunted Troll can regenerate itself for {G}.
#[test]
fn hunted_troll_regenerates() {
    let mut g = two_player_game();
    let troll = g.add_card_to_battlefield(0, catalog::hunted_troll());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: troll, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("regen shield");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(troll).unwrap().regeneration_shields, 1,
        "{{G}} stamps a regeneration shield");
}

/// Stoneshaker Shaman makes the active player sacrifice an untapped land at
/// each end step.
#[test]
fn stoneshaker_shaman_eats_a_land_each_end_step() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::stoneshaker_shaman());
    let land = g.add_card_to_battlefield(0, catalog::forest()); // untapped
    g.active_player_idx = 0;
    g.step = TurnStep::PostCombatMain;
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "the untapped land was sacrificed");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == land), "it's in the graveyard");
}

/// Boros Fury-Shield burns the creature's controller for its power when {R} was
/// spent to cast it.
#[test]
fn boros_fury_shield_burns_when_red_spent() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    // Player 1 attacks with a 2/2.
    g.active_player_idx = 1;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(0),
    }])).expect("attack");
    // Player 0 responds with Fury-Shield, paying the two generic with red.
    g.priority.player_with_priority = 0;
    let shield = g.add_card_to_hand(0, catalog::boros_fury_shield());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Red, 2);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: shield, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast fury-shield");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2, "dealt power (2) to the creature's controller");
}

/// Siege of Towers animates a Mountain into a 3/1 that's still a land.
#[test]
fn siege_of_towers_animates_a_mountain() {
    use crabomination::card::CardType;
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let mtn = g.add_card_to_battlefield(0, catalog::mountain());
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(mtn)];
    let def = catalog::siege_of_towers();
    let evs = g.resolve_effect(&def.effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    let cp = g.computed_permanent(mtn).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 1), "becomes a 3/1");
    assert!(cp.card_types.contains(&CardType::Creature), "now a creature");
    assert!(cp.card_types.contains(&CardType::Land), "still a land");
}

/// Greater Mossdog is a 3/3 with dredge 3.
#[test]
fn greater_mossdog_stat_and_dredge() {
    let m = catalog::greater_mossdog();
    assert_eq!((m.power, m.toughness), (3, 3));
    assert!(m.keywords.iter().any(|k| matches!(k, Keyword::Dredge(3))));
}

/// Flow of Ideas draws one card per Island you control.
#[test]
fn flow_of_ideas_draws_per_island() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::island()); }
    for _ in 0..5 { g.add_card_to_library(0, catalog::forest()); }
    let h0 = g.players[0].hand.len();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&catalog::flow_of_ideas().effect, &ctx).unwrap();
    assert_eq!(g.players[0].hand.len(), h0 + 3, "drew a card per Island (3)");
}

/// Hour of Reckoning destroys nontoken creatures but spares tokens.
#[test]
fn hour_of_reckoning_spares_tokens() {
    use crabomination::card::{CardType, TokenDefinition};
    let mut g = two_player_game();
    let real = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let token_def = TokenDefinition {
        name: "Soldier".into(), power: 1, toughness: 1,
        card_types: vec![CardType::Creature], ..Default::default()
    };
    let token = g.add_token_to_battlefield(0, &token_def);
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(&catalog::hour_of_reckoning().effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    assert!(g.battlefield_find(real).is_none(), "nontoken creature destroyed");
    assert!(g.battlefield_find(token).is_some(), "token spared");
}

/// Guardian of Vitu-Ghazi is a 4/7 with convoke and vigilance.
#[test]
fn guardian_of_vitu_ghazi_stat_line() {
    let g = catalog::guardian_of_vitu_ghazi();
    assert_eq!((g.power, g.toughness), (4, 7));
    assert!(g.keywords.contains(&Keyword::Convoke) && g.keywords.contains(&Keyword::Vigilance));
}

// ── RAV gap wave 6 (gaps6.rs) ────────────────────────────────────────────────

/// Hammerfist Giant taps to deal 4 to each non-flyer and each player.
#[test]
fn hammerfist_giant_sweeps_grounded_and_players() {
    let mut g = two_player_game();
    let giant = g.add_card_to_battlefield(0, catalog::hammerfist_giant());
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let flyer = g.add_card_to_battlefield(1, catalog::serra_angel()); // flying
    g.clear_sickness(giant);
    let foe_life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: giant, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap sweep");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ground).is_none(), "grounded creature took 4, died");
    assert_eq!(g.battlefield_find(flyer).unwrap().damage, 0, "flyer untouched");
    assert_eq!(g.players[1].life, foe_life - 4, "each player took 4");
}

/// Blockbuster sacrifices for 3 damage to each tapped creature and each player.
#[test]
fn blockbuster_hits_tapped_creatures_and_players() {
    let mut g = two_player_game();
    let bb = g.add_card_to_battlefield(0, catalog::blockbuster());
    let tapped = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let untapped = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(tapped).unwrap().tapped = true;
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let foe_life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bb, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac blast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bb).is_none(), "enchantment sacrificed");
    assert!(g.battlefield_find(tapped).is_none(), "tapped creature took 3, died");
    assert_eq!(g.battlefield_find(untapped).unwrap().damage, 0, "untapped untouched");
    assert_eq!(g.players[1].life, foe_life - 3, "each player took 3");
}

/// Flight of Fancy draws two on entry and grants the host flying.
#[test]
fn flight_of_fancy_draws_and_grants_flying() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
    let hand = g.players[0].hand.len();
    // Aura attached to the host grants it flying (equip bonus).
    let aura = g.add_card_to_battlefield(0, catalog::flight_of_fancy());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(host);
    assert!(g.computed_permanent(host).unwrap().keywords.contains(&Keyword::Flying),
        "host has flying");
    // Its ETB trigger draws two cards.
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let etb = &catalog::flight_of_fancy().triggered_abilities[0].effect;
    g.resolve_effect(etb, &ctx).unwrap();
    assert_eq!(g.players[0].hand.len(), hand + 2, "ETB drew two");
}

/// Dimir House Guard has fear and regenerates by sacrificing a creature.
#[test]
fn dimir_house_guard_fear_and_regenerate() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let guard = g.add_card_to_battlefield(0, catalog::dimir_house_guard());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.battlefield_find(guard).unwrap().definition.keywords.contains(&Keyword::Fear));
    g.perform_action(GameAction::ActivateAbility {
        card_id: guard, ability_index: 0, target: None,
        additional_targets: vec![Target::Permanent(fodder)], x_value: None,
    }).expect("regenerate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "creature sacrificed");
    assert!(g.battlefield_find(guard).unwrap().regeneration_shields >= 1, "regen shield up");
}

/// Ethereal Usher makes a target creature unblockable until end of turn.
#[test]
fn ethereal_usher_grants_unblockable() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let usher = g.add_card_to_battlefield(0, catalog::ethereal_usher());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(usher);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: usher, ability_index: 0, target: Some(Target::Permanent(ally)),
        additional_targets: vec![], x_value: None,
    }).expect("grant unblockable");
    drain_stack(&mut g);
    assert!(g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::Unblockable));
}

/// Viashino Fangtail taps to ping any target for 1.
#[test]
fn viashino_fangtail_pings() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let fang = g.add_card_to_battlefield(0, catalog::viashino_fangtail());
    g.clear_sickness(fang);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: fang, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "pinged for 1");
}

/// Undercity Shade pumps itself +1/+1 for {B}.
#[test]
fn undercity_shade_pumps() {
    let mut g = two_player_game();
    let shade = g.add_card_to_battlefield(0, catalog::undercity_shade());
    assert!(shade_is_fearsome(&g, shade));
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: shade, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("shade pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(shade).unwrap().power, 2, "+1/+1");
}

fn shade_is_fearsome(g: &GameState, id: crabomination::game::CardId) -> bool {
    g.battlefield_find(id).unwrap().definition.keywords.contains(&Keyword::Fear)
}

/// Viashino Slasher trades toughness for power (+1/-1).
#[test]
fn viashino_slasher_pumps_lopsided() {
    let mut g = two_player_game();
    let slasher = g.add_card_to_battlefield(0, catalog::viashino_slasher());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: slasher, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("slasher pump");
    drain_stack(&mut g);
    let cp = g.computed_permanent(slasher).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 1), "+1/-1");
}

/// Tattered Drake regenerates for {B}.
#[test]
fn tattered_drake_regenerates() {
    let mut g = two_player_game();
    let drake = g.add_card_to_battlefield(0, catalog::tattered_drake());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: drake, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("regen");
    drain_stack(&mut g);
    assert!(g.battlefield_find(drake).unwrap().regeneration_shields >= 1);
}

/// Surveilling Sprite draws a card when it dies (opting in).
#[test]
fn surveilling_sprite_dies_draw() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let sprite = g.add_card_to_battlefield(0, catalog::surveilling_sprite());
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    g.battlefield_find_mut(sprite).unwrap().damage = 5;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew on death");
}

/// Zephyr Spirit bounces itself to hand when it blocks.
#[test]
fn zephyr_spirit_returns_on_block() {
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    // P1 attacks; P0's Zephyr Spirit blocks and returns to hand.
    let spirit = g.add_card_to_battlefield(0, catalog::zephyr_spirit());
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    // Hand P1 the turn.
    while g.active_player_idx == 0 {
        g.perform_action(GameAction::PassPriority).expect("pass to P1 turn");
    }
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("advance to attacks");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(0),
    }])).expect("P1 attacks");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("advance to blocks");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(spirit, attacker)])).expect("block");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == spirit), "returned to hand on block");
}

/// Cyclopean Snare taps a creature and bounces itself to hand.
#[test]
fn cyclopean_snare_taps_and_returns() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let snare = g.add_card_to_battlefield(0, catalog::cyclopean_snare());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: snare, ability_index: 0, target: Some(Target::Permanent(foe)),
        additional_targets: vec![], x_value: None,
    }).expect("snare");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "target tapped");
    assert!(g.players[0].hand.iter().any(|c| c.id == snare), "snare returned to hand");
}

/// Festival of the Guildpact prevents the next X damage to you and cantrips.
#[test]
fn festival_prevents_x_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    // Cast with X = 3.
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 3);
    g.resolve_effect(&catalog::festival_of_the_guildpact().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "cantrip drew");
    // Now deal 5 to player 0; only 2 should land (3 prevented).
    let life = g.players[0].life;
    let dctx = crabomination::game::effects::EffectContext::for_spell(1, None, 0, 0);
    let mut dctx2 = dctx.clone();
    dctx2.targets = vec![crabomination::game::types::Target::Player(0)];
    g.resolve_effect(&crabomination::effect::Effect::DealDamage {
        to: crabomination::effect::Selector::Target(0),
        amount: crabomination::effect::Value::Const(5),
    }, &dctx2).unwrap();
    assert_eq!(g.players[0].life, life - 2, "3 of the 5 prevented");
}

/// War-Torch Goblin sacrifices to deal 2 to a blocking creature.
#[test]
fn war_torch_goblin_burns_blocker() {
    use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let goblin = g.add_card_to_battlefield(0, catalog::war_torch_goblin());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(attacker);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("advance to attacks");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("advance to blocks");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).expect("block");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: goblin, ability_index: 0, target: Some(Target::Permanent(blocker)),
        additional_targets: vec![], x_value: None,
    }).expect("torch the blocker");
    drain_stack(&mut g);
    assert!(g.battlefield_find(blocker).is_none(), "2/2 blocker took 2 and died");
}
