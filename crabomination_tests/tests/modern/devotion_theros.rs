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

// ── Devotion (CR 700.5) — Value::DevotionTo / Gray Merchant of Asphodel ───────

#[test]
fn devotion_to_black_counts_black_pips_on_your_permanents() {
    let mut g = two_player_game();
    // Putrid Imp is {B} (one black pip); add two of them.
    g.add_card_to_battlefield(0, catalog::putrid_imp());
    g.add_card_to_battlefield(0, catalog::putrid_imp());
    // An opponent's black permanent doesn't count toward your devotion.
    g.add_card_to_battlefield(1, catalog::putrid_imp());
    assert_eq!(g.devotion_to(0, &[Color::Black]), 2);
    assert_eq!(g.devotion_to(1, &[Color::Black]), 1);
}

#[test]
fn gray_merchant_drains_for_devotion_to_black() {
    let mut g = two_player_game();
    // One pre-existing black pip on the battlefield; Gray Merchant adds two.
    g.add_card_to_battlefield(0, catalog::putrid_imp());
    let id = g.add_card_to_hand(0, catalog::gray_merchant_of_asphodel());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Black, 2);
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Gray Merchant castable for {3}{B}{B}");
    drain_stack(&mut g);
    // Devotion = 1 (Imp) + 2 (Gray Merchant itself, on the battlefield by
    // the time its ETB resolves) = 3.
    assert_eq!(g.players[1].life, opp_life - 3, "opponent loses devotion");
    assert_eq!(g.players[0].life, my_life + 3, "you gain devotion");
}

// ── Theros gods — devotion-gated creature toggle (CR 700.5) ───────────────────

#[test]
fn nylea_is_not_a_creature_below_five_devotion() {
    let mut g = two_player_game();
    // Nylea alone contributes 1 green pip ({3}{G}); devotion 1 < 5.
    let nylea = g.add_card_to_battlefield(0, catalog::nylea_god_of_the_hunt());
    let cp = g.computed_permanent(nylea).unwrap();
    assert!(!cp.card_types.contains(&CardType::Creature),
        "Nylea isn't a creature at devotion 1");
    assert!(cp.card_types.contains(&CardType::Enchantment));
}

#[test]
fn nylea_becomes_a_creature_at_five_devotion_and_anthems_others() {
    let mut g = two_player_game();
    let nylea = g.add_card_to_battlefield(0, catalog::nylea_god_of_the_hunt());
    // Four more green-pip permanents push devotion to 5.
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::elvish_mystic());
    }
    let cp = g.computed_permanent(nylea).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature),
        "Nylea is a creature once devotion ≥ 5");
    // Anthem: another creature you control gets +2/+0.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bcp = g.computed_permanent(bear).unwrap();
    assert_eq!(bcp.power, 4, "Nylea grants +2/+0 to other creatures");
    assert_eq!(bcp.toughness, 2);
}

#[test]
fn erebos_prevents_controller_life_gain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::erebos_god_of_the_dead());
    let before = g.players[0].life;
    g.adjust_life(0, 5);
    assert_eq!(g.players[0].life, before, "Erebos stops your life gain");
}

// ── CR 508.0 — "attacks only alone" (Master of Cruelties) ─────────────────────

#[test]
fn master_of_cruelties_cannot_attack_alongside_another_creature() {
    let mut g = two_player_game();
    let moc = g.add_card_to_battlefield(0, catalog::master_of_cruelties());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(moc);
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    // Declaring both is illegal — AttacksAlone rejects the batch.
    assert!(g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: moc, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ])).is_err(), "Master of Cruelties can't attack alongside another creature");
}

#[test]
fn master_of_cruelties_attacks_alone_and_sets_life_to_one() {
    let mut g = two_player_game();
    let moc = g.add_card_to_battlefield(0, catalog::master_of_cruelties());
    g.clear_sickness(moc);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: moc, target: AttackTarget::Player(1) },
    ])).expect("attacking alone is fine");
    drain_stack(&mut g);
    // The life-set rides AttacksAndIsntBlocked, so it fires once the
    // defender declares no blockers.
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no block");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 1, "unblocked attack sets the defender's life to 1");
}

#[test]
fn master_of_cruelties_deals_no_combat_damage_so_player_survives_at_one() {
    // CR 510.1 — the "deals no combat damage this turn" rider must stop
    // Master's 1-power first-strike deathtouch ping from killing the
    // defender after their life is set to 1.
    let mut g = two_player_game();
    let moc = g.add_card_to_battlefield(0, catalog::master_of_cruelties());
    g.clear_sickness(moc);
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: moc, target: AttackTarget::Player(1) },
    ])).expect("attack alone");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no block");
    drain_stack(&mut g);
    // Push through the combat-damage step(s).
    while g.step != TurnStep::PostCombatMain && g.players[1].life == 1 {
        g.perform_action(GameAction::PassPriority).expect("advance");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, 1,
        "defender stays at 1 — Master assigns no combat damage");
    assert!(!g.players[1].eliminated, "defender did not lose");
}

#[test]
fn nykthos_taps_for_devotion_of_chosen_color() {
    use crabomination::decision::{Decision, DecisionAnswer};
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    // Three black pips on the battlefield → devotion to black = 3.
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::putrid_imp()); }
    let nyk = g.add_card_to_battlefield(0, catalog::nykthos_shrine_to_nyx());
    g.clear_sickness(nyk);
    g.players[0].mana_pool.add_colorless(2); // pay the {2} activation cost
    g.perform_action(GameAction::ActivateAbility {
        card_id: nyk, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Nykthos devotion ability activates");
    let pd = g.pending_decision.as_ref().expect("ChooseColor suspends");
    assert!(matches!(pd.decision, Decision::ChooseColor { .. }));
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Color(Color::Black)))
        .unwrap();
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 3,
        "adds black mana equal to devotion to black");
}

/// Esika's Chariot's attack trigger copies a token you control (CR 707).
#[test]
fn esikas_chariot_attack_copies_a_token() {
    let mut g = two_player_game();
    // Cast the Chariot so its ETB mints the two Cat tokens to copy.
    let id = g.add_card_to_hand(0, catalog::esikas_chariot());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Esika's Chariot castable");
    drain_stack(&mut g);
    let cats_before = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Cat").count();
    assert_eq!(cats_before, 2, "ETB made two Cats");
    // Crew it and send it in.
    g.clear_sickness(id);
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::Crew { vehicle: id, crew_creatures: vec![b1, b2] })
        .expect("crew 4");
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: id, target: AttackTarget::Player(1) },
    ])).expect("crewed chariot attacks");
    drain_stack(&mut g);
    let cats_after = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Cat").count();
    assert_eq!(cats_after, 3, "attack trigger copied a Cat token");
}

// ── Theros god-weapon anthem enchantments ─────────────────────────────────────

#[test]
fn spear_of_heliod_pumps_your_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::spear_of_heliod());
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "your creature gets +1/+1");
    let ocp = g.computed_permanent(opp).unwrap();
    assert_eq!((ocp.power, ocp.toughness), (2, 2), "opponent's creature unaffected");
}

#[test]
fn whip_of_erebos_grants_lifelink_to_your_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::whip_of_erebos());
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Lifelink));
}

#[test]
fn hammer_of_purphoros_grants_haste() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::hammer_of_purphoros());
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Haste));
}

#[test]
fn thassa_grants_unblockable_to_your_creature() {
    let mut g = two_player_game();
    let thassa = g.add_card_to_battlefield(0, catalog::thassa_god_of_the_sea());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(thassa);
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: thassa, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Thassa's unblockable ability");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Unblockable));
}

// ── Theros filler commons (devotion-shell bodies) ─────────────────────────────

#[test]
fn sedge_scorpion_and_pharikas_chosen_have_deathtouch() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::sedge_scorpion());
    let p = g.add_card_to_battlefield(0, catalog::pharikas_chosen());
    assert!(g.battlefield_find(s).unwrap().has_keyword(&crabomination::card::Keyword::Deathtouch));
    assert!(g.battlefield_find(p).unwrap().has_keyword(&crabomination::card::Keyword::Deathtouch));
}

#[test]
fn two_headed_cerberus_has_double_strike() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::two_headed_cerberus());
    assert!(g.battlefield_find(c).unwrap().has_keyword(&crabomination::card::Keyword::DoubleStrike));
}

#[test]
fn voyaging_satyr_untaps_a_land() {
    let mut g = two_player_game();
    let satyr = g.add_card_to_battlefield(0, catalog::voyaging_satyr());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(land).unwrap().tapped = true;
    g.clear_sickness(satyr);
    g.perform_action(GameAction::ActivateAbility {
        card_id: satyr, ability_index: 0, target: Some(Target::Permanent(land)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("untap land");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(land).unwrap().tapped, "land untapped");
}

#[test]
fn leonin_snarecaster_etb_taps_a_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::leonin_snarecaster());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Leonin Snarecaster castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "ETB taps the target creature");
}

#[test]
fn voyages_end_bounces_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::voyages_end());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Voyage's End castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "creature bounced");
    assert!(g.players[1].hand.iter().any(|c| c.definition.name == "Grizzly Bears"), "to owner's hand");
}

#[test]
fn theros_vanilla_bodies_have_expected_stats() {
    let mut g = two_player_game();
    let ox = g.add_card_to_battlefield(0, catalog::yoked_ox());
    let courser = g.add_card_to_battlefield(0, catalog::nessian_courser());
    let goliath = g.add_card_to_battlefield(0, catalog::vulpine_goliath());
    let minotaur = g.add_card_to_battlefield(0, catalog::felhide_minotaur());
    assert_eq!((g.battlefield_find(ox).unwrap().power(), g.battlefield_find(ox).unwrap().toughness()), (0, 4));
    assert_eq!((g.battlefield_find(courser).unwrap().power(), g.battlefield_find(courser).unwrap().toughness()), (3, 3));
    assert!(g.battlefield_find(goliath).unwrap().has_keyword(&crabomination::card::Keyword::Trample));
    assert_eq!((g.battlefield_find(minotaur).unwrap().power(), g.battlefield_find(minotaur).unwrap().toughness()), (2, 3));
}

#[test]
fn magda_makes_a_treasure_when_a_dwarf_taps() {
    let mut g = two_player_game();
    let magda = g.add_card_to_battlefield(0, catalog::magda_brazen_outlaw());
    g.clear_sickness(magda);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    // Magda is a Dwarf; attacking taps her → "becomes tapped" → Treasure.
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: magda, target: AttackTarget::Player(1) },
    ])).expect("Magda attacks");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"),
        "tapping a Dwarf you control makes a Treasure");
}

// ── More Theros instants/removal ──────────────────────────────────────────────

#[test]
fn griptide_puts_creature_on_top_of_library() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::griptide());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Griptide castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "creature left the battlefield");
    assert_eq!(g.players[1].library.first().map(|c| c.definition.name), Some("Grizzly Bears"),
        "creature is on top of its owner's library");
}

#[test]
fn lash_of_the_whip_shrinks_and_kills_a_small_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::lash_of_the_whip());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lash castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "-4/-4 kills the 2/2");
}

#[test]
fn pharikas_cure_pings_and_gains_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::pharikas_cure());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pharika's Cure castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "2 damage kills the 2/2");
    assert_eq!(g.players[0].life, life + 2, "gain 2 life");
}

#[test]
fn fade_into_antiquity_exiles_an_enchantment() {
    let mut g = two_player_game();
    let ench = g.add_card_to_battlefield(1, catalog::spear_of_heliod());
    let id = g.add_card_to_hand(0, catalog::fade_into_antiquity());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(ench)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fade into Antiquity castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == ench), "enchantment exiled");
    assert!(g.exile.iter().any(|c| c.id == ench), "to exile zone");
}

#[test]
fn nyleas_disciple_gains_life_for_green_devotion() {
    let mut g = two_player_game();
    // Two green pips already out; Disciple's own {3}{G} adds 1 → devotion 3.
    g.add_card_to_battlefield(0, catalog::elvish_mystic());
    g.add_card_to_battlefield(0, catalog::elvish_mystic());
    let id = g.add_card_to_hand(0, catalog::nyleas_disciple());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Nylea's Disciple castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 4, "gain life = devotion to green (2 + self)");
}

#[test]
fn mnemonic_wall_returns_an_instant_from_graveyard() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // a creature, ineligible
    let id = g.add_card_to_hand(0, catalog::mnemonic_wall());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mnemonic Wall castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Lightning Bolt"),
        "instant returned to hand");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "creature stays in the graveyard");
}

#[test]
fn cavalry_pegasus_and_traveling_philosopher_bodies() {
    let mut g = two_player_game();
    let peg = g.add_card_to_battlefield(0, catalog::cavalry_pegasus());
    let phil = g.add_card_to_battlefield(0, catalog::traveling_philosopher());
    assert!(g.battlefield_find(peg).unwrap().has_keyword(&crabomination::card::Keyword::Flying));
    assert_eq!((g.battlefield_find(phil).unwrap().power(), g.battlefield_find(phil).unwrap().toughness()), (2, 2));
}

#[test]
fn horizon_scholar_etb_scrys_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::horizon_scholar());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Horizon Scholar castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&crabomination::card::Keyword::Flying));
    assert_eq!((c.power(), c.toughness()), (4, 4));
}

#[test]
fn theros_artifact_bodies() {
    let mut g = two_player_game();
    let raptor = g.add_card_to_battlefield(0, catalog::anvilwrought_raptor());
    let sable = g.add_card_to_battlefield(0, catalog::bronze_sable());
    let guard = g.add_card_to_battlefield(0, catalog::guardians_of_meletis());
    let uni = g.add_card_to_battlefield(0, catalog::opaline_unicorn());
    let rc = g.battlefield_find(raptor).unwrap();
    assert!(rc.has_keyword(&crabomination::card::Keyword::Flying) && rc.has_keyword(&crabomination::card::Keyword::FirstStrike));
    assert!(g.battlefield_find(guard).unwrap().has_keyword(&crabomination::card::Keyword::Defender));
    assert!(g.battlefield_find(sable).unwrap().definition.card_types.contains(&CardType::Artifact));
    // Opaline Unicorn taps for any color.
    g.clear_sickness(uni);
    g.perform_action(GameAction::ActivateAbility {
        card_id: uni, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("mana ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1, "Opaline Unicorn made one mana");
}

#[test]
fn returned_centaur_etb_mills_four() {
    let mut g = two_player_game();
    for _ in 0..8 { g.add_card_to_library(0, catalog::island()); }
    let yard = g.players[0].graveyard.len();
    let id = g.add_card_to_hand(0, catalog::returned_centaur());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Returned Centaur castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), yard + 4, "ETB self-mills four");
}

#[test]
fn baleful_eidolon_is_an_enchantment_creature_with_deathtouch() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::baleful_eidolon());
    let c = g.battlefield_find(id).unwrap();
    assert!(c.definition.card_types.contains(&CardType::Enchantment));
    assert!(c.has_keyword(&crabomination::card::Keyword::Deathtouch));
}

#[test]
fn theros_minotaur_bodies() {
    let mut g = two_player_game();
    let bm = g.add_card_to_battlefield(0, catalog::borderland_minotaur());
    let dr = g.add_card_to_battlefield(0, catalog::deathbellow_raider());
    let aw = g.add_card_to_battlefield(0, catalog::asphodel_wanderer());
    assert_eq!((g.battlefield_find(bm).unwrap().power(), g.battlefield_find(bm).unwrap().toughness()), (4, 3));
    assert_eq!((g.battlefield_find(dr).unwrap().power(), g.battlefield_find(dr).unwrap().toughness()), (2, 3));
    assert_eq!((g.battlefield_find(aw).unwrap().power(), g.battlefield_find(aw).unwrap().toughness()), (1, 1));
}

// ── Pithing Needle / "name a card" (CR 201.3) ────────────────────────────────

#[test]
fn pithing_needle_etb_stamps_the_named_card() {
    // Casting Pithing Needle fires its ETB NameCard trigger; the scripted
    // decider names Tormod's Crypt, which is stamped onto the artifact.
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::NamedCard("Tormod's Crypt".into()),
    ]));
    let needle = g.add_card_to_hand(0, catalog::pithing_needle());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: needle, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pithing Needle castable for {1}");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(needle).and_then(|c| c.named_card.as_deref()),
        Some("Tormod's Crypt"),
        "ETB stamps the chosen card name");
}

#[test]
fn pithing_needle_suppresses_named_sources_nonmana_ability() {
    // Needle names Tormod's Crypt → the Crypt's {T},Sac: exile-graveyard
    // (non-mana) ability is shut off. A Crypt naming nothing relevant works.
    let mut g = two_player_game();
    let crypt = g.add_card_to_battlefield(0, catalog::tormods_crypt());
    let needle = g.add_card_to_battlefield(0, catalog::pithing_needle());
    g.battlefield_find_mut(needle).unwrap().named_card = Some("Tormod's Crypt".into());
    g.priority.player_with_priority = 0;

    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: crypt, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).unwrap_err();
    assert!(matches!(err, GameError::AbilitySuppressedByNamedCard),
        "named source's non-mana ability is suppressed, got {err:?}");
}

#[test]
fn pithing_needle_naming_a_different_card_leaves_ability_usable() {
    let mut g = two_player_game();
    let crypt = g.add_card_to_battlefield(0, catalog::tormods_crypt());
    let needle = g.add_card_to_battlefield(0, catalog::pithing_needle());
    g.battlefield_find_mut(needle).unwrap().named_card = Some("Llanowar Elves".into());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: crypt, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
        .expect("an unrelated name doesn't suppress the Crypt");
}

#[test]
fn phyrexian_revoker_is_a_two_one_construct_that_names_and_suppresses() {
    // Body + ETB-name path: cast it, name Tormod's Crypt, then the Crypt's
    // non-mana ability is shut off.
    let mut g = two_player_game();
    let crypt = g.add_card_to_battlefield(1, catalog::tormods_crypt());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::NamedCard("Tormod's Crypt".into()),
    ]));
    let rev = g.add_card_to_hand(0, catalog::phyrexian_revoker());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: rev, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Phyrexian Revoker castable for {2}");
    drain_stack(&mut g);
    let body = g.battlefield_find(rev).unwrap();
    assert_eq!((body.power(), body.toughness()), (2, 1));
    assert!(body.definition.card_types.contains(&CardType::Artifact));

    g.priority.player_with_priority = 1;
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: crypt, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).unwrap_err();
    assert!(matches!(err, GameError::AbilitySuppressedByNamedCard));
}

// ── Juggernaut / "attacks each combat if able" (CR 508.1d) ───────────────────

#[test]
fn juggernaut_must_be_declared_as_attacker() {
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let jug = g.add_card_to_battlefield(0, catalog::juggernaut());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(jug);
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;

    // Declaring only the Bear while Juggernaut can attack is illegal.
    let err = g.declare_attackers(vec![
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ]).unwrap_err();
    assert!(matches!(err, GameError::CannotAttack(id) if id == jug),
        "Juggernaut must attack if able, got {err:?}");

    // Including the Juggernaut is accepted.
    g.declare_attackers(vec![
        Attack { attacker: jug, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ]).expect("declaration including Juggernaut is legal");
}

#[test]
fn juggernaut_tapped_is_exempt_from_must_attack() {
    let mut g = two_player_game();
    let jug = g.add_card_to_battlefield(0, catalog::juggernaut());
    g.battlefield_find_mut(jug).unwrap().tapped = true; // can't attack → exempt
    g.clear_sickness(jug);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![]).expect("a tapped Juggernaut isn't forced to attack");
}

#[test]
fn lovestruck_beast_cant_attack_without_a_one_one() {
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let beast = g.add_card_to_battlefield(0, catalog::lovestruck_beast());
    g.clear_sickness(beast);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;

    // No 1/1 in play → can't attack.
    let err = g
        .declare_attackers(vec![Attack { attacker: beast, target: AttackTarget::Player(1) }])
        .unwrap_err();
    assert!(matches!(err, GameError::CannotAttack(id) if id == beast));

    // A 1/1 (Goldhound) satisfies the gate.
    g.add_card_to_battlefield(0, catalog::goldhound());
    g.declare_attackers(vec![Attack { attacker: beast, target: AttackTarget::Player(1) }])
        .expect("Lovestruck Beast attacks with a 1/1 in play");
}

// ── Spellbomb cycle + utility (claude/modern_decks) ──────────────────────────

#[test]
fn nihil_spellbomb_exiles_opponent_graveyard() {
    let mut g = two_player_game();
    let bomb = g.add_card_to_battlefield(0, catalog::nihil_spellbomb());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bomb, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
        .expect("{T},Sac: exile opp graveyard");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.is_empty(), "opponent graveyard exiled");
    assert!(!g.battlefield.iter().any(|c| c.id == bomb), "spellbomb sacrificed");
}

#[test]
fn pyrite_spellbomb_deals_two_to_a_player() {
    let mut g = two_player_game();
    let bomb = g.add_card_to_battlefield(0, catalog::pyrite_spellbomb());
    g.priority.player_with_priority = 0;
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bomb, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None , mode: None})
        .expect("{T},Sac: 2 damage to any target");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2);
}

#[test]
fn seal_of_removal_bounces_a_creature() {
    let mut g = two_player_game();
    let seal = g.add_card_to_battlefield(0, catalog::seal_of_removal());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: seal, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None , mode: None})
        .expect("Sac: bounce target creature");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear left the battlefield");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "bear returned to owner's hand");
}

#[test]
fn vendetta_destroys_nonblack_creature_and_loses_life_equal_to_toughness() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 green
    let vend = g.add_card_to_hand(0, catalog::vendetta());
    g.players[0].mana_pool.add(Color::Black, 1);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: vend, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vendetta castable for {B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "nonblack creature destroyed");
    assert_eq!(g.players[0].life, life - 2, "lose life equal to its toughness (2)");
}

#[test]
fn sylvan_spellbomb_fetches_a_basic_land_to_hand() {
    let mut g = two_player_game();
    let bomb = g.add_card_to_battlefield(0, catalog::sylvan_spellbomb());
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bomb, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
        .expect("{T},Sac: fetch a basic land");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == forest), "Forest goes to hand");
}

#[test]
fn expedition_map_fetches_any_land_to_hand() {
    let mut g = two_player_game();
    let map = g.add_card_to_battlefield(0, catalog::expedition_map());
    let land = g.add_card_to_library(0, catalog::watery_grave()); // nonbasic
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(land))]));
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: map, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
        .expect("{2},{T},Sac: fetch any land");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == land), "nonbasic land goes to hand");
}

#[test]
fn executioners_capsule_destroys_nonblack_creature() {
    let mut g = two_player_game();
    let cap = g.add_card_to_battlefield(0, catalog::executioners_capsule());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cap, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None , mode: None})
        .expect("{1}{B},Sac: destroy nonblack creature");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "nonblack creature destroyed");
    assert!(!g.battlefield.iter().any(|c| c.id == cap), "Capsule sacrificed");
}

#[test]
fn horizon_spellbomb_draw_mode_works() {
    let mut g = two_player_game();
    let bomb = g.add_card_to_battlefield(0, catalog::horizon_spellbomb());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: bomb, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
        .expect("{W},Sac: draw a card");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}

#[test]
fn rapid_hybridization_destroys_and_makes_a_frog_lizard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let rh = g.add_card_to_hand(0, catalog::rapid_hybridization());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: rh, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Rapid Hybridization castable for {U}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "creature destroyed");
    let frog = g.battlefield.iter().find(|c| c.definition.name == "Frog Lizard");
    assert!(frog.is_some(), "controller gets a Frog Lizard token");
    let frog = frog.unwrap();
    assert_eq!(frog.controller, 1);
    assert_eq!((frog.power(), frog.toughness()), (3, 3));
}

#[test]
fn impulse_digs_four_and_puts_one_in_hand() {
    let mut g = two_player_game();
    for _ in 0..8 { g.add_card_to_library(0, catalog::island()); }
    let imp = g.add_card_to_hand(0, catalog::impulse());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: imp, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Impulse castable for {1}{U}");
    drain_stack(&mut g);
    // Cast (-1) + one card to hand (+1) = net unchanged.
    assert_eq!(g.players[0].hand.len(), hand, "one of four dug cards lands in hand");
}

#[test]
fn serum_visions_draws_then_scries() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let sv = g.add_card_to_hand(0, catalog::serum_visions());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: sv, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Serum Visions castable for {U}");
    drain_stack(&mut g);
    // Cast (-1) + draw (+1) = net unchanged; scry doesn't change hand size.
    assert_eq!(g.players[0].hand.len(), hand);
}

#[test]
fn flame_rift_deals_four_to_each_player() {
    let mut g = two_player_game();
    let fr = g.add_card_to_hand(0, catalog::flame_rift());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    g.perform_action(GameAction::CastSpell {
        card_id: fr, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Flame Rift castable for {1}{R}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0 - 4, "caster takes 4");
    assert_eq!(g.players[1].life, l1 - 4, "opponent takes 4");
}

#[test]
fn ultimate_price_destroys_monocolored_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // mono-green
    let up = g.add_card_to_hand(0, catalog::ultimate_price());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: up, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ultimate Price castable for {1}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "monocolored creature destroyed");
}

#[test]
fn walk_the_plank_destroys_non_merfolk() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wtp = g.add_card_to_hand(0, catalog::walk_the_plank());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: wtp, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Walk the Plank castable for {1}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "non-Merfolk creature destroyed");
}

#[test]
fn rabid_bite_deals_your_creatures_power_to_an_enemy() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let rb = g.add_card_to_hand(0, catalog::rabid_bite());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: rb, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("Rabid Bite castable for {1}{G}");
    drain_stack(&mut g);
    // 2 power → 2 damage → the 2/2 enemy dies.
    assert!(!g.battlefield.iter().any(|c| c.id == theirs), "enemy took lethal (2) damage");
    assert!(g.battlefield.iter().any(|c| c.id == mine), "your creature takes no damage back");
}

/// The auto-target picker fills *both* of Rabid Bite's slots — slot 0
/// (your creature, hidden inside `Value::PowerOf`) and slot 1 (the
/// opponent's creature) — so the bot can cast it unaided.
#[test]
fn rabid_bite_auto_targets_both_slots() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let eff = catalog::rabid_bite().effect;
    let (slot0, additional) = g.auto_targets_for_effect_all_slots(&eff, 0, None);
    assert_eq!(slot0, Some(Target::Permanent(mine)), "slot 0 = your creature");
    assert_eq!(additional, vec![Target::Permanent(theirs)], "slot 1 = opponent's creature");
}

/// Collector Ouphe locks non-mana artifact abilities (Millstone) but
/// leaves mana abilities (Sol Ring) alone.
#[test]
fn collector_ouphe_locks_artifact_abilities_but_not_mana() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::collector_ouphe());
    let mill = g.add_card_to_battlefield(0, catalog::millstone());
    let sol = g.add_card_to_battlefield(0, catalog::sol_ring());
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: mill, ability_index: 0,
        target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None, mode: None,
    });
    assert!(err.is_err(), "Millstone's non-mana ability is locked under Collector Ouphe: {err:?}");
    g.perform_action(GameAction::ActivateAbility {
        card_id: sol, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Sol Ring's mana ability still works under the lock");
    assert!(g.players[0].mana_pool.total() >= 2, "Sol Ring added mana");
}

#[test]
fn oust_tucks_creature_and_owner_gains_five() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let oust = g.add_card_to_hand(0, catalog::oust());
    g.players[0].mana_pool.add(Color::White, 1);
    let owner_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: oust, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Oust castable for {W}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "creature left battlefield");
    assert!(g.players[1].library.iter().any(|c| c.id == bear), "tucked into owner's library");
    assert_eq!(g.players[1].life, owner_life + 5, "its owner gains 5 life");
}

#[test]
fn soul_snare_exiles_an_attacker() {
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let snare = g.add_card_to_battlefield(0, catalog::soul_snare());
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    // Player 1 attacks; then player 0 sacrifices the Snare to exile it.
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(0) }])
        .expect("declare attacker");
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: snare, ability_index: 0, target: Some(Target::Permanent(attacker)), additional_targets: Vec::new(), x_value: None , mode: None})
        .expect("{1},Sac: exile attacking creature");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == attacker), "attacking creature exiled");
}

#[test]
fn dragon_fodder_makes_two_goblins() {
    let mut g = two_player_game();
    let df = g.add_card_to_hand(0, catalog::dragon_fodder());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: df, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Dragon Fodder castable for {1}{R}");
    drain_stack(&mut g);
    let gobs = g.battlefield.iter().filter(|c| c.definition.name == "Goblin" && c.controller == 0).count();
    assert_eq!(gobs, 2, "two 1/1 Goblins created");
}

#[test]
fn hordeling_outburst_makes_three_goblins() {
    let mut g = two_player_game();
    let ho = g.add_card_to_hand(0, catalog::hordeling_outburst());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: ho, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hordeling Outburst castable for {1}{R}{R}");
    drain_stack(&mut g);
    let gobs = g.battlefield.iter().filter(|c| c.definition.name == "Goblin" && c.controller == 0).count();
    assert_eq!(gobs, 3, "three 1/1 Goblins created");
}

#[test]
fn krenkos_command_makes_two_goblins() {
    let mut g = two_player_game();
    let kc = g.add_card_to_hand(0, catalog::krenkos_command());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: kc, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Krenko's Command castable for {1}{R}");
    drain_stack(&mut g);
    let gobs = g.battlefield.iter().filter(|c| c.definition.name == "Goblin" && c.controller == 0).count();
    assert_eq!(gobs, 2, "two 1/1 Goblins created");
}

