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

// ── Modern staples batch: scam riders, baubles, Flares, flips ───────────────

/// Feign Death: the creature dies this turn → returns tapped with a +1/+1.
#[test]
fn feign_death_returns_dead_creature_tapped_with_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let fd = g.add_card_to_hand(0, catalog::feign_death());
    g.players[0].mana_pool.add(Color::Black, 1);
    cast_at(&mut g, fd, Target::Permanent(bear));
    // Kill the bear.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the bear");
    drain_stack(&mut g);
    let back = g.battlefield_find(bear).expect("bear returned to the battlefield");
    assert!(back.tapped, "returns tapped");
    assert_eq!(back.counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1);
}

/// Urza's Bauble: sacrifice → draw at your next upkeep.
#[test]
fn urzas_bauble_draws_at_next_upkeep() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let bauble = g.add_card_to_battlefield(0, catalog::urzas_bauble());
    g.perform_action(GameAction::ActivateAbility {
        card_id: bauble, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate bauble");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bauble).is_none(), "bauble sacrificed");
    let hand_before = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "delayed draw fired");
}

/// Codex Shredder: {T} mills target player; the sac ability returns a card
/// from your graveyard to hand.
#[test]
fn codex_shredder_mills_and_regrows() {
    let mut g = two_player_game();
    g.add_card_to_library(1, catalog::forest());
    let shredder = g.add_card_to_battlefield(0, catalog::codex_shredder());
    g.perform_action(GameAction::ActivateAbility {
        card_id: shredder, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("mill ability");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 1, "opponent milled one");

    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.battlefield_find_mut(shredder).unwrap().tapped = false;
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: shredder, ability_index: 1, target: Some(Target::Permanent(bolt)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("regrow ability");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "card returned to hand");
    assert!(g.battlefield_find(shredder).is_none(), "shredder sacrificed");
}

/// Flare of Malice: alt-cast by sacrificing a nontoken black creature; each
/// opponent sacrifices their greatest-MV creature or planeswalker.
#[test]
fn flare_of_malice_free_cast_via_black_creature_sacrifice() {
    let mut g = two_player_game();
    let pitch = g.add_card_to_battlefield(0, catalog::indulgent_aristocrat()); // black creature
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears());        // MV 2
    let big = g.add_card_to_battlefield(1, catalog::serra_angel());            // MV 5
    let flare = g.add_card_to_hand(0, catalog::flare_of_malice());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: flare, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("free Flare via sacrifice");
    drain_stack(&mut g);
    assert!(g.battlefield_find(pitch).is_none(), "black creature sacrificed as cost");
    assert!(g.battlefield_find(big).is_none(), "opponent lost the greatest-MV creature");
    assert!(g.battlefield_find(small).is_some(), "lesser creature survives");
}

/// Brutal Cathar exiles an opposing creature on ETB and gives it back when
/// it leaves the battlefield.
#[test]
fn brutal_cathar_exiles_until_it_leaves() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let cathar = g.add_card_to_hand(0, catalog::brutal_cathar());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_at(&mut g, cathar, Target::Permanent(bear));
    assert!(g.exile.iter().any(|c| c.id == bear), "bear exiled by the cathar");
    // Cathar dies → bear comes home.
    let mut evs = Vec::new();
    g.sacrifice_one(cathar, 0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "bear returns when the cathar leaves");
}

/// Wedding Announcement ticks invitation counters each of your end steps,
/// makes a token without two attackers, and transforms at three counters.
#[test]
fn wedding_announcement_counts_to_festivity() {
    let mut g = two_player_game();
    let wa = g.add_card_to_battlefield(0, catalog::wedding_announcement());
    for _ in 0..3 {
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
    }
    let c = g.battlefield_find(wa).unwrap();
    assert!(c.transformed, "three invitations → Wedding Festivity");
    assert_eq!(c.definition.name, "Wedding Festivity");
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Human").count(),
        3,
        "no attacks → a 1/1 Human each end step"
    );
}

/// Valley Floodcaller pumps and untaps your Otters on each noncreature cast.
#[test]
fn valley_floodcaller_pumps_and_untaps_otters_on_noncreature_cast() {
    let mut g = two_player_game();
    let vfc = g.add_card_to_battlefield(0, catalog::valley_floodcaller());
    g.battlefield_find_mut(vfc).unwrap().tapped = true;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(vfc).unwrap();
    assert!(!c.tapped, "otter untapped by its own trigger");
    assert_eq!(g.computed_permanent(vfc).unwrap().power, 3, "2/2 + 1 = 3");
}

/// Raffine: attacking with two creatures connives the target for 2.
#[test]
fn raffine_connives_target_attacker_for_attacker_count() {
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.add_card_to_hand(0, catalog::island()); // discard fodder (nonland in hand? island is land)
    let raffine = g.add_card_to_battlefield(0, catalog::raffine_scheming_seer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(raffine);
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: raffine, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ])).expect("attack with both");
    drain_stack(&mut g);
    // Connive 2: drew 2, discarded 2; the target got a counter per nonland
    // discarded. AutoDecider pitches from the front — counters depend on
    // what got pitched, so just assert the draw/discard happened and the
    // trigger resolved.
    assert_eq!(g.players[0].graveyard.len(), 2, "discarded two to connive");
}

/// Necrodominance: hand size capped at five, your graveyard-bound cards are
/// exiled instead, and the end step converts life into cards.
#[test]
fn necrodominance_statics_and_end_step_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::necrodominance());
    assert_eq!(g.effective_max_hand_size(0), Some(5), "max hand size five");
    // Graveyard redirect: a discarded/destroyed card goes to exile.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let mut evs = Vec::new();
    g.discard_card(0, bolt, &mut evs);
    assert!(g.exile.iter().any(|c| c.id == bolt), "your graveyard-bound card exiled");
    // End step: pay 3 → draw 3.
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(3)]));
    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before - 3);
    assert_eq!(g.players[0].hand.len(), hand_before + 3);
}

/// Sorin of House Markov transforms at your postcombat main once you've
/// gained three life this turn.
#[test]
fn sorin_transforms_after_three_life_gained() {
    let mut g = two_player_game();
    let sorin = g.add_card_to_battlefield(0, catalog::sorin_of_house_markov());
    g.fire_step_triggers(TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(sorin).unwrap().transformed, "no lifegain → no flip");
    g.adjust_life(0, 3);
    g.fire_step_triggers(TurnStep::PostCombatMain);
    drain_stack(&mut g);
    let flipped = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Sorin, Ravenous Neonate")
        .expect("transformed into the planeswalker");
    assert_eq!(flipped.counter_count(crabomination::card::CounterType::Loyalty), 3);
}

/// Cori-Steel Cutter: the second spell each turn mints a Monk and attaches.
#[test]
fn cori_steel_cutter_flurry_mints_and_attaches() {
    let mut g = two_player_game();
    let cutter = g.add_card_to_battlefield(0, catalog::cori_steel_cutter());
    for _ in 0..2 {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt castable");
        drain_stack(&mut g);
    }
    let monk = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Monk")
        .expect("flurry minted a Monk on the second spell");
    assert_eq!(
        g.battlefield_find(cutter).unwrap().attached_to,
        Some(monk.id),
        "cutter attached to the fresh Monk"
    );
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Monk").count(),
        1,
        "first spell minted nothing"
    );
}

/// Tamiyo transforms when you draw your third card in a turn.
#[test]
fn tamiyo_transforms_on_third_draw() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let tamiyo = g.add_card_to_battlefield(0, catalog::tamiyo_inquisitive_student());
    let mut evs = Vec::new();
    for _ in 0..2 {
        g.draw_one(0, &mut evs);
    }
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(tamiyo).is_some(), "two draws — still a creature");
    let mut evs = Vec::new();
    g.draw_one(0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let flipped = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Tamiyo, Seasoned Scholar")
        .expect("third draw flips Tamiyo");
    assert_eq!(flipped.counter_count(crabomination::card::CounterType::Loyalty), 2);
}

/// Flare of Duplication: alt-cast by sacrificing a red creature, copying a
/// spell on the stack.
#[test]
fn flare_of_duplication_copies_target_spell() {
    let mut g = two_player_game();
    let pitch = g.add_card_to_battlefield(0, catalog::dragons_rage_channeler());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    let flare = g.add_card_to_hand(0, catalog::flare_of_duplication());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: flare, pitch_card: None, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("free Flare via red creature sacrifice");
    drain_stack(&mut g);
    assert!(g.battlefield_find(pitch).is_none(), "red creature sacrificed");
    assert_eq!(g.players[1].life, 14, "bolt + copy = 6 damage");
}

/// Goblin Charbelcher counts nonland reveals, doubles on a Mountain, and
/// bottoms the reveals.
#[test]
fn goblin_charbelcher_damage_doubles_on_mountain() {
    let mut g = two_player_game();
    // add_card_to_library appends bottom-up; top-down result: two
    // nonlands, then a Mountain.
    g.add_card_to_library(0, catalog::counterspell());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::mountain());
    let lib_before = g.players[0].library.len();
    let belcher = g.add_card_to_battlefield(0, catalog::goblin_charbelcher());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: belcher, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("belch");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 16, "2 nonland × 2 (Mountain) = 4");
    assert_eq!(g.players[0].library.len(), lib_before, "reveals bottomed, not lost");
}

/// Transmogrify exiles the target and flips its controller's next creature
/// onto the battlefield.
#[test]
fn transmogrify_polymorphs_into_next_creature() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::serra_angel());
    g.add_card_to_library(0, catalog::forest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let tm = g.add_card_to_hand(0, catalog::transmogrify());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast_at(&mut g, tm, Target::Permanent(bear));
    assert!(g.exile.iter().any(|c| c.id == bear), "target exiled");
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Serra Angel" && c.controller == 0),
        "revealed creature hits the battlefield"
    );
}

/// Disruptor Flute taxes the named spell {3} and needles its abilities.
#[test]
fn disruptor_flute_taxes_named_spell() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::NamedCard(
        "Lightning Bolt".into(),
    )]));
    let flute = g.add_card_to_hand(0, catalog::disruptor_flute());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: flute, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("flute castable");
    drain_stack(&mut g);
    // Opponent's Bolt now costs {R} + {3}.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "taxed Bolt unaffordable on one red");
    g.players[1].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("affordable with the {3} tax");
}

/// Ral flips a coin on your-turn instant casts: a win may transform him.
#[test]
fn ral_transforms_on_won_flip() {
    let mut g = two_player_game();
    let ral = g.add_card_to_battlefield(0, catalog::ral_monsoon_mage());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true), // win the flip
        DecisionAnswer::Bool(true), // choose to transform
    ]));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    // {R} less {1} from Ral's discount → free? No: Bolt is {R}; colored pips
    // aren't reduced. Pay it.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ral).is_none() || g.battlefield_find(ral).unwrap().transformed
        || g.battlefield.iter().any(|c| c.definition.name == "Ral, Leyline Prodigy"),
        "won flip → transformed planeswalker");
    let flipped = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Ral, Leyline Prodigy")
        .expect("Ral flipped");
    assert_eq!(flipped.counter_count(crabomination::card::CounterType::Loyalty), 2);
}

/// Emrakul, the World Anew steals target player's creatures on cast and
/// can't be targeted by spells.
#[test]
fn emrakul_world_anew_steals_and_has_spell_protection() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let emrakul = g.add_card_to_hand(0, catalog::emrakul_the_world_anew());
    g.players[0].mana_pool.add_colorless(12);
    g.perform_action(GameAction::CastSpell {
        card_id: emrakul, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("emrakul castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0, "stole the board");
    // Protection from spells: opponent can't bolt it.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    let em_id = g.battlefield.iter().find(|c| c.definition.name == "Emrakul, the World Anew").unwrap().id;
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(em_id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "protection from spells blocks targeting");
}

/// Ugin's -X exiles each colored permanent with MV ≤ X.
#[test]
fn ugin_minus_x_exiles_colored_cheap_permanents() {
    let mut g = two_player_game();
    let ugin = g.add_card_to_battlefield(0, catalog::ugin_the_spirit_dragon());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());   // MV 2, green
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());    // MV 5, white
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());     // colorless
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: ugin, ability_index: 1, target: None, x_value: Some(3),
    }).expect("Ugin -3");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "MV 2 colored permanent exiled");
    assert!(g.battlefield_find(angel).is_some(), "MV 5 survives X=3");
    assert!(g.battlefield_find(stone).is_some(), "colorless ignored");
    assert_eq!(
        g.battlefield_find(ugin).unwrap().counter_count(crabomination::card::CounterType::Loyalty),
        4,
        "7 - 3 loyalty"
    );
}

/// Scion of Draco: domain discount and per-color keyword grants.
#[test]
fn scion_of_draco_domain_discount_and_color_grants() {
    let mut g = two_player_game();
    for f in [catalog::plains, catalog::island, catalog::swamp, catalog::mountain, catalog::forest] {
        let l = g.add_card_to_battlefield(0, f());
        let _ = l;
    }
    let scion = g.add_card_to_hand(0, catalog::scion_of_draco());
    // {12} - 5 × {2} = {2}.
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: scion, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("full domain → {2}");
    drain_stack(&mut g);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green
    assert!(
        g.computed_permanent(bear).unwrap().keywords.contains(&crabomination::card::Keyword::Trample),
        "green creature granted trample"
    );
}


/// CR 701.30 — Recross the Paths fetches the first land to the battlefield
/// and a won clash returns the spell to hand.
#[test]
fn cr_701_30_recross_the_paths_clash_win_returns_to_hand() {
    let mut g = two_player_game();
    // P0 top-down: nonland, then a Forest (the find), then a high-MV card
    // for the clash reveal.
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::serra_angel()); // MV 5 — clash reveal
    g.add_card_to_library(1, catalog::grizzly_bears()); // MV 2 — opponent reveal
    let rp = g.add_card_to_hand(0, catalog::recross_the_paths());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: rp, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Forest" && c.controller == 0),
        "first land revealed lands on the battlefield"
    );
    assert!(
        g.players[0].hand.iter().any(|c| c.id == rp),
        "won clash (5 vs 2) returns the spell to hand"
    );
}

// ── Modern staples batch 4 ───────────────────────────────────────────────────

/// Shoot the Sheriff can't hit outlaws.
#[test]
fn shoot_the_sheriff_rejects_outlaws() {
    let mut g = two_player_game();
    let rogue = g.add_card_to_battlefield(1, catalog::brazen_borrower()); // Rogue
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let sts = g.add_card_to_hand(0, catalog::shoot_the_sheriff());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: sts, target: Some(Target::Permanent(rogue)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "outlaw is protected");
    cast_at(&mut g, sts, Target::Permanent(bear));
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "non-outlaw destroyed");
}

/// Meltdown X=1 clears cheap artifacts only.
#[test]
fn meltdown_destroys_artifacts_up_to_x() {
    let mut g = two_player_game();
    let cheap = g.add_card_to_battlefield(1, catalog::ornithopter());   // MV 0
    let pricey = g.add_card_to_battlefield(1, catalog::mind_stone());   // MV 2
    let melt = g.add_card_to_hand(0, catalog::meltdown());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: melt, target: None, additional_targets: vec![], mode: None, x_value: Some(1),
    }).expect("Meltdown X=1");
    drain_stack(&mut g);
    assert!(g.battlefield_find(cheap).is_none(), "MV 0 destroyed");
    assert!(g.battlefield_find(pricey).is_some(), "MV 2 survives X=1");
}

/// Sterling Grove gives OTHER enchantments shroud (not itself) and tutors
/// an enchantment to the library top.
#[test]
fn sterling_grove_shroud_and_tutor() {
    let mut g = two_player_game();
    let grove = g.add_card_to_battlefield(0, catalog::sterling_grove());
    let other = g.add_card_to_battlefield(0, catalog::necrodominance());
    assert!(
        g.computed_permanent(other).unwrap().keywords.contains(&crabomination::card::Keyword::Shroud),
        "other enchantment shrouded"
    );
    assert!(
        !g.computed_permanent(grove).unwrap().keywords.contains(&crabomination::card::Keyword::Shroud),
        "grove itself is not shrouded"
    );
    g.add_card_to_library(0, catalog::leyline_of_the_void());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: grove, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("tutor");
    drain_stack(&mut g);
    assert!(g.battlefield_find(grove).is_none(), "grove sacrificed");
    assert_eq!(
        g.players[0].library.first().map(|c| c.definition.name),
        Some("Leyline of the Void"),
        "enchantment on top"
    );
}

/// Vein Ripper drains on any creature death.
#[test]
fn vein_ripper_drains_on_creature_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::vein_ripper());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut evs = Vec::new();
    g.sacrifice_one(bear, 1, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "opponent drained 2");
    assert_eq!(g.players[0].life, 22, "controller gained 2");
}

/// Sorin, Imperious Bloodlord -3 cheats a Vampire into play.
#[test]
fn sorin_bloodlord_minus_three_deploys_vampire() {
    let mut g = two_player_game();
    let sorin = g.add_card_to_battlefield(0, catalog::sorin_imperious_bloodlord());
    let vamp = g.add_card_to_hand(0, catalog::vein_ripper());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![vamp])]));
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: sorin, ability_index: 2, target: None, x_value: None,
    }).expect("Sorin -3");
    drain_stack(&mut g);
    assert!(g.battlefield_find(vamp).is_some(), "Vampire deployed from hand");
    assert_eq!(
        g.battlefield_find(sorin).unwrap().counter_count(crabomination::card::CounterType::Loyalty),
        1,
        "4 - 3 loyalty"
    );
}

/// Floodpits Drowner stuns on entry and shuffles itself + the stunned
/// creature away.
#[test]
fn floodpits_drowner_stuns_then_shuffles_both_away() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let fish = g.add_card_to_hand(0, catalog::floodpits_drowner());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_at(&mut g, fish, Target::Permanent(bear));
    let b = g.battlefield_find(bear).unwrap();
    assert!(b.tapped && b.counter_count(crabomination::card::CounterType::Stun) == 1, "tapped + stunned");

    let fish_id = g.battlefield.iter().find(|c| c.definition.name == "Floodpits Drowner").unwrap().id;
    g.battlefield_find_mut(fish_id).unwrap().summoning_sick = false;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: fish_id, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("shuffle ability");
    drain_stack(&mut g);
    assert!(g.players[1].library.iter().any(|c| c.id == bear), "bear shuffled in");
    assert!(g.players[0].library.iter().any(|c| c.id == fish_id), "drowner shuffled in");
}

/// Tune the Narrative draws and banks two energy.
#[test]
fn tune_the_narrative_draws_and_adds_energy() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let tn = g.add_card_to_hand(0, catalog::tune_the_narrative());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: tn, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
    assert_eq!(g.players[0].energy, 2, "two energy banked");
}

/// Leyline Axe may start on the battlefield from the opening hand.
#[test]
fn leyline_axe_starts_in_play_from_opening_hand() {
    let mut g = two_player_game();
    let axe = g.add_card_to_hand(0, catalog::leyline_axe());
    g.fire_start_of_game_effects();
    assert!(g.battlefield_find(axe).is_some(), "leyline drop");
}

/// Questing Druid grows on a non-green-colored spell; its Adventure
/// impulses two cards.
#[test]
fn questing_druid_grows_and_adventure_impulses() {
    let mut g = two_player_game();
    let druid = g.add_card_to_hand(0, catalog::questing_druid());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: druid, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("creature half");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("red spell");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(druid).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne),
        1,
        "red spell grew the druid"
    );

    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    let druid = g.add_card_to_hand(0, catalog::questing_druid());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastAdventure {
        card_id: druid, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Seek the Beast");
    drain_stack(&mut g);
    // The adventuring card itself also sits in exile (CR 715.3d).
    assert_eq!(
        g.exile.iter().filter(|c| c.owner == 0 && c.id != druid).count(),
        2,
        "impulsed two"
    );
}

// ── ZNR MDFC spell//land cycle ───────────────────────────────────────────────

/// The MDFC back face is playable as a tapped land.
#[test]
fn znr_mdfc_back_face_plays_as_tapped_land() {
    let mut g = two_player_game();
    let card = g.add_card_to_hand(0, catalog::malakir_rebirth());
    g.perform_action(GameAction::PlayLandBack(card)).expect("land half playable");
    let c = g.battlefield_find(card).expect("on battlefield");
    assert_eq!(c.definition.name, "Malakir Mire");
    assert!(c.tapped, "enters tapped");
}

/// Spikefield Hazard's kill is an exile instead.
#[test]
fn spikefield_hazard_exiles_what_it_kills() {
    let mut g = two_player_game();
    let thopter = g.add_card_to_battlefield(1, catalog::ornithopter()); // 0/2
    let spike = g.add_card_to_hand(0, catalog::spikefield_hazard());
    g.players[0].mana_pool.add(Color::Red, 1);
    // 1 damage doesn't kill a 0/2 — use a damaged target instead.
    g.battlefield_find_mut(thopter).unwrap().damage = 1;
    cast_at(&mut g, spike, Target::Permanent(thopter));
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == thopter), "died this turn → exiled instead");
    assert!(g.players[1].graveyard.iter().all(|c| c.id != thopter));
}

/// Malakir Rebirth costs 2 life and returns the dying creature tapped.
#[test]
fn malakir_rebirth_returns_creature_for_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mr = g.add_card_to_hand(0, catalog::malakir_rebirth());
    g.players[0].mana_pool.add(Color::Black, 1);
    cast_at(&mut g, mr, Target::Permanent(bear));
    assert_eq!(g.players[0].life, 18, "paid 2 life");
    let mut evs = Vec::new();
    g.sacrifice_one(bear, 0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let back = g.battlefield_find(bear).expect("returned");
    assert!(back.tapped);
}

/// Kazandu Mammoth grows on land drops.
#[test]
fn kazandu_mammoth_landfall_pump() {
    let mut g = two_player_game();
    let mammoth = g.add_card_to_battlefield(0, catalog::kazandu_mammoth());
    let land = g.add_card_to_hand(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::PlayLand(land)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(mammoth).unwrap().power, 5, "3 + 2 landfall");
}

/// Silundi Vision digs six for an instant or sorcery.
#[test]
fn silundi_vision_digs_for_instant() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let sv = g.add_card_to_hand(0, catalog::silundi_vision());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: sv, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(
        g.players[0].hand.iter().any(|c| c.definition.name == "Lightning Bolt"),
        "found the instant"
    );
}

// ── ZNR spell//land MDFC batch (2026) ────────────────────────────────────────

/// The rare-cycle MDFC land back pays 3 life to enter untapped (default
/// ChooseMode pick), or enters tapped otherwise.
#[test]
fn znr_painland_back_pays_three_to_untap() {
    let mut g = two_player_game();
    let card = g.add_card_to_hand(0, catalog::sea_gate_restoration());
    g.perform_action(GameAction::PlayLandBack(card)).expect("land half playable");
    drain_stack(&mut g);
    let land = g.battlefield_find(card).expect("on battlefield");
    assert_eq!(land.definition.name, "Sea Gate, Reborn");
    assert!(!land.tapped, "default picks pay-3-life → untapped");
    assert_eq!(g.players[0].life, 17, "paid 3 life");
}

/// Kazuul's Fury flings a sacrificed creature's power at any target.
#[test]
fn kazuuls_fury_deals_sacrificed_power() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 to sacrifice
    let kf = g.add_card_to_hand(0, catalog::kazuuls_fury());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_at(&mut g, kf, Target::Player(1));
    assert_eq!(g.players[1].life, 18, "2 damage = bear's power");
}

/// Kabira Takedown deals damage equal to creatures you control.
#[test]
fn kabira_takedown_scales_with_board() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let target = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let kt = g.add_card_to_hand(0, catalog::kabira_takedown());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_at(&mut g, kt, Target::Permanent(target));
    assert_eq!(g.battlefield_find(target).unwrap().damage, 2, "two creatures → 2 damage");
}

/// Makindi Stampede pumps the whole team +2/+2.
#[test]
fn makindi_stampede_pumps_team() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ms = g.add_card_to_hand(0, catalog::makindi_stampede());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: ms, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(a).unwrap().power, 4);
    assert_eq!(g.computed_permanent(b).unwrap().toughness, 4);
}

/// Khalni Ambush makes your creature fight an opponent's.
#[test]
fn khalni_ambush_fights() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let ka = g.add_card_to_hand(0, catalog::khalni_ambush());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: ka, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "2/2 dies to 4 damage");
}

/// Sejiri Shelter grants protection from the chosen color.
#[test]
fn sejiri_shelter_grants_protection() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ss = g.add_card_to_hand(0, catalog::sejiri_shelter());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    cast_at(&mut g, ss, Target::Permanent(bear));
    assert!(g.battlefield_find(bear).unwrap().has_keyword(&Keyword::Protection(Color::Red)));
}

/// Song-Mad Treachery steals a creature until end of turn.
#[test]
fn song_mad_treachery_steals() {
    let mut g = two_player_game();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let st = g.add_card_to_hand(0, catalog::song_mad_treachery());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    cast_at(&mut g, st, Target::Permanent(theirs));
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0, "now yours");
}

/// Zof Consumption drains 4 and gains 4.
#[test]
fn zof_consumption_drains_four() {
    let mut g = two_player_game();
    let zc = g.add_card_to_hand(0, catalog::zof_consumption());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: zc, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 16);
    assert_eq!(g.players[0].life, 24);
}

/// Vastwood Fortification puts a +1/+1 counter on a creature.
#[test]
fn vastwood_fortification_adds_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let vf = g.add_card_to_hand(0, catalog::vastwood_fortification());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast_at(&mut g, vf, Target::Permanent(bear));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "2 + counter");
}

/// Pelakka Predation makes an opponent discard a high-MV card.
#[test]
fn pelakka_predation_discards_expensive() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears()); // MV 2, kept
    g.add_card_to_hand(1, catalog::serra_angel()); // MV 5, discarded
    let pp = g.add_card_to_hand(0, catalog::pelakka_predation());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: pp, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Serra Angel"));
    assert!(g.players[1].hand.iter().any(|c| c.definition.name == "Grizzly Bears"));
}

/// Jwari Disruption counters a spell unless its controller pays {1}.
#[test]
fn jwari_disruption_counters_unpaid() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1); // just enough for the bolt
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp casts bolt");
    let jd = g.add_card_to_hand(0, catalog::jwari_disruption());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    cast_at(&mut g, jd, Target::Permanent(spell));
    // Opponent has no spare mana to pay {1} → bolt is countered.
    assert!(g.players[1].graveyard.iter().any(|c| c.id == spell), "countered");
    assert_eq!(g.players[0].life, 20, "bolt never resolved");
}

/// Shatterskull Smashing deals X damage divided (twice X at 6+).
#[test]
fn shatterskull_smashing_doubles_at_six() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw()); // 6/6
    let ss = g.add_card_to_hand(0, catalog::shatterskull_smashing());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(6); // X = 6
    g.perform_action(GameAction::CastSpell {
        card_id: ss, target: Some(Target::Permanent(big)),
        additional_targets: vec![], mode: None, x_value: Some(6),
    }).expect("cast X=6");
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_none(), "twice-6 = 12 damage kills the 6/6");
}

/// Sea Gate Restoration refills your hand and removes the hand-size cap.
#[test]
fn sea_gate_restoration_draws_and_lifts_cap() {
    let mut g = two_player_game();
    for _ in 0..10 { g.add_card_to_library(0, catalog::forest()); }
    g.add_card_to_hand(0, catalog::forest());
    g.add_card_to_hand(0, catalog::forest()); // 2 other cards in hand
    let sg = g.add_card_to_hand(0, catalog::sea_gate_restoration());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.players[0].mana_pool.add_colorless(4);
    let before = g.players[0].hand.len(); // 3 (incl. Sea Gate)
    g.perform_action(GameAction::CastSpell {
        card_id: sg, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // Hand after cast = 2; draw 2+1 = 3 → 5.
    assert_eq!(g.players[0].hand.len(), before - 1 + 3);
    assert!(g.players[0].max_hand_size.is_none(), "no maximum hand size");
}

/// Emeria's Call makes two 4/4 flyers and shields non-Angels.
#[test]
fn emerias_call_makes_angels() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ec = g.add_card_to_hand(0, catalog::emerias_call());
    g.players[0].mana_pool.add(Color::White, 3);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: ec, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let angels = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Angel Warrior").count();
    assert_eq!(angels, 2);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Indestructible),
        "non-Angel gains indestructible");
}

/// Turntimber Symbiosis puts a creature from the top seven onto the battlefield.
#[test]
fn turntimber_symbiosis_deploys_creature() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    for _ in 0..6 { g.add_card_to_library(0, catalog::forest()); }
    let ts = g.add_card_to_hand(0, catalog::turntimber_symbiosis());
    g.players[0].mana_pool.add(Color::Green, 3);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: ts, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Grizzly Bears"));
}

/// Ondu Inversion destroys all nonland permanents.
#[test]
fn ondu_inversion_wraths_nonlands() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let oi = g.add_card_to_hand(0, catalog::ondu_inversion());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpell {
        card_id: oi, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "creature destroyed");
    assert!(g.battlefield_find(land).is_some(), "land survives");
}

/// Akoum Hellhound grows on landfall.
#[test]
fn akoum_hellhound_landfall_pump() {
    let mut g = two_player_game();
    let dog = g.add_card_to_battlefield(0, catalog::akoum_hellhound());
    let land = g.add_card_to_hand(0, catalog::mountain());
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::PlayLand(land)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(dog).unwrap().power, 2, "0 + 2 landfall");
}

/// Nimana Skydancer mills two on enter.
#[test]
fn nimana_skydancer_mills_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(1, catalog::forest()); }
    let ns = g.add_card_to_hand(0, catalog::nimana_skydancer());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let gy_before = g.players[1].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: ns, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), gy_before + 2);
}

/// Cragplate Baloth enters with four +1/+1 counters when kicked.
#[test]
fn cragplate_baloth_kicked_enters_bigger() {
    let mut g = two_player_game();
    let cb = g.add_card_to_hand(0, catalog::cragplate_baloth());
    g.players[0].mana_pool.add(Color::Green, 3);
    g.players[0].mana_pool.add_colorless(7); // {5}{G}{G} + {2}{G} kicker
    g.perform_action(GameAction::CastSpellKicked {
        card_id: cb, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast kicked");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(cb).unwrap().power, 10, "6 + 4 counters");
}

/// Kazandu Nectarpot gains life on landfall.
#[test]
fn kazandu_nectarpot_landfall_lifegain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kazandu_nectarpot());
    let land = g.add_card_to_hand(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::PlayLand(land)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 21, "gained 1 on landfall");
}

/// Fearless Fledgling grows and flies on landfall.
#[test]
fn fearless_fledgling_landfall_counter_and_flying() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let f = g.add_card_to_battlefield(0, catalog::fearless_fledgling());
    let land = g.add_card_to_hand(0, catalog::plains());
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::PlayLand(land)).unwrap();
    drain_stack(&mut g);
    let cp = g.computed_permanent(f).unwrap();
    assert_eq!(cp.power, 2, "+1/+1 counter");
    assert!(cp.keywords.contains(&Keyword::Flying), "gains flying");
}

/// Sporeweb Weaver triggers on taking damage.
#[test]
fn sporeweb_weaver_enrage_makes_saproling() {
    let mut g = two_player_game();
    let spider = g.add_card_to_battlefield(0, catalog::sporeweb_weaver()); // 1/4
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, bolt, Target::Permanent(spider)); // 3 dmg — survives
    assert!(g.battlefield_find(spider).is_some(), "1/4 survives 3 damage");
    assert_eq!(g.players[0].life, 21, "gained 1 life from enrage");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Saproling"));
}

/// Subtle Strike both modes: -1/-1 on one creature, +1/+1 counter on another.
#[test]
fn subtle_strike_chooses_both() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ss = g.add_card_to_hand(0, catalog::subtle_strike());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Modes(vec![0, 1])]));
    g.perform_action(GameAction::CastSpell {
        card_id: ss, target: Some(Target::Permanent(foe)),
        additional_targets: vec![Target::Permanent(mine)], mode: None, x_value: None,
    }).expect("cast both modes");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(foe).unwrap().toughness, 1, "-1/-1");
    assert_eq!(g.computed_permanent(mine).unwrap().power, 3, "+1/+1 counter");
}

/// Canyon Jerboa pumps the team on landfall.
#[test]
fn canyon_jerboa_landfall_team_pump() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::canyon_jerboa());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let land = g.add_card_to_hand(0, catalog::plains());
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::PlayLand(land)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "+1/+1 to the team");
}

/// Territorial Scythecat permanently grows on landfall.
#[test]
fn territorial_scythecat_landfall_counter() {
    let mut g = two_player_game();
    let cat = g.add_card_to_battlefield(0, catalog::territorial_scythecat());
    let land = g.add_card_to_hand(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::PlayLand(land)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(cat).unwrap().power, 3, "2 + counter");
}

/// Makindi Ox taps a creature on landfall.
#[test]
fn makindi_ox_landfall_taps() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::makindi_ox());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let land = g.add_card_to_hand(0, catalog::plains());
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::PlayLand(land)).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "opponent's creature tapped");
}

/// Ondu Greathorn grows on landfall and has first strike.
#[test]
fn ondu_greathorn_landfall_pump() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let ox = g.add_card_to_battlefield(0, catalog::ondu_greathorn());
    assert!(g.battlefield_find(ox).unwrap().definition.keywords.contains(&Keyword::FirstStrike));
    let land = g.add_card_to_hand(0, catalog::plains());
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::PlayLand(land)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(ox).unwrap().power, 4, "2 + 2 landfall");
}

/// Joraga Visionary draws on enter.
#[test]
fn joraga_visionary_etb_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let jv = g.add_card_to_hand(0, catalog::joraga_visionary());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: jv, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // -1 cast + 1 ETB draw = net 0.
    assert_eq!(g.players[0].hand.len(), before, "drew a card on ETB");
}

/// Stonework Packbeast is all four party types and taps for any color.
#[test]
fn stonework_packbeast_is_full_party() {
    use crabomination::card::CreatureType;
    let mut g = two_player_game();
    let pb = g.add_card_to_battlefield(0, catalog::stonework_packbeast());
    let types = &g.battlefield_find(pb).unwrap().definition.subtypes.creature_types;
    for t in [CreatureType::Cleric, CreatureType::Rogue, CreatureType::Warrior, CreatureType::Wizard] {
        assert!(types.contains(&t), "is a {t:?}");
    }
}

/// Cleric of Chill Depths stuns what it blocks.
#[test]
fn cleric_of_chill_depths_stuns_blocked() {
    use crabomination::card::CounterType;
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let cleric = g.add_card_to_battlefield(0, catalog::cleric_of_chill_depths());
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(0),
    }])).expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(cleric, attacker)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(attacker).unwrap().counter_count(CounterType::Stun), 1,
        "blocked attacker is stunned");
}

/// Anticognition counters a creature spell unless its controller pays {2}.
#[test]
fn anticognition_counters_creature_spell() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp casts bear");
    let ac = g.add_card_to_hand(0, catalog::anticognition());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    cast_at(&mut g, ac, Target::Permanent(spell));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == spell), "countered (couldn't pay {{2}})");
}

/// Pride of the Clouds grows for each other flyer on the battlefield.
#[test]
fn pride_of_the_clouds_scales_with_flyers() {
    let mut g = two_player_game();
    let pride = g.add_card_to_battlefield(0, catalog::pride_of_the_clouds());
    assert_eq!(g.computed_permanent(pride).unwrap().power, 1, "alone: just base 1");
    g.add_card_to_battlefield(0, catalog::serra_angel()); // a flyer you control
    g.add_card_to_battlefield(0, catalog::serra_angel()); // another flyer you control
    assert_eq!(g.computed_permanent(pride).unwrap().power, 3, "1 + 2 other flyers you control");
}

/// Skyclave Squid can attack after a landfall despite Defender.
#[test]
fn skyclave_squid_landfall_lets_it_attack() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let squid = g.add_card_to_battlefield(0, catalog::skyclave_squid());
    g.clear_sickness(squid);
    let land = g.add_card_to_hand(0, catalog::island());
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::PlayLand(land)).unwrap();
    drain_stack(&mut g);
    assert!(!g.computed_permanent(squid).unwrap().keywords.contains(&Keyword::Defender),
        "defender suppressed this turn");
}

/// Brushfire Elemental can't be blocked by small creatures and grows on landfall.
#[test]
fn brushfire_elemental_evasion_and_landfall() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let be = g.add_card_to_battlefield(0, catalog::brushfire_elemental());
    assert!(g.battlefield_find(be).unwrap().definition.keywords.iter().any(|k|
        matches!(k, Keyword::CantBeBlockedBy(_))));
    let land = g.add_card_to_hand(0, catalog::mountain());
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::PlayLand(land)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(be).unwrap().power, 3, "1 + 2 landfall");
}

/// Resolute Strike pumps a creature +2/+2.
#[test]
fn resolute_strike_pumps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let rs = g.add_card_to_hand(0, catalog::resolute_strike());
    g.players[0].mana_pool.add(Color::White, 1);
    cast_at(&mut g, rs, Target::Permanent(bear));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "2 + 2");
}

/// Adventure Awaits digs five for a creature.
#[test]
fn adventure_awaits_finds_creature() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let aa = g.add_card_to_hand(0, catalog::adventure_awaits());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aa, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"));
}

/// Sneaking Guide makes a small creature unblockable.
#[test]
fn sneaking_guide_grants_unblockable() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let guide = g.add_card_to_battlefield(0, catalog::sneaking_guide());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
    g.clear_sickness(guide);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: guide, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Unblockable));
}

/// Knight of Malice gets +1/+0 only while a white permanent is in play.
#[test]
fn knight_of_malice_pumps_against_white() {
    let mut g = two_player_game();
    let knight = g.add_card_to_battlefield(0, catalog::knight_of_malice());
    assert_eq!(g.computed_permanent(knight).unwrap().power, 2, "no white permanent yet");
    g.add_card_to_battlefield(1, catalog::serra_angel()); // a white permanent
    assert_eq!(g.computed_permanent(knight).unwrap().power, 3, "+1/+0 while white is out");
}

/// Glasspool Mimic enters as a copy of a creature you control.
#[test]
fn glasspool_mimic_copies_your_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4 to copy
    let gm = g.add_card_to_hand(0, catalog::glasspool_mimic());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: gm, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let mimic = g.battlefield_find(gm).expect("entered");
    assert_eq!((mimic.power(), mimic.toughness()), (4, 4), "copied the 4/4 Angel");
}

// ── Follow-ups batch A: lockdowns, land swaps, processors, mill engines ──────

/// Sunken Citadel's second ability adds two restricted mana that fund land
/// abilities but not spells.
#[test]
fn sunken_citadel_restricted_mana_is_land_abilities_only() {
    use crabomination::mana::{ManaCost, SpellKind};
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::sunken_citadel());
    g.battlefield_find_mut(land).unwrap().chosen_color = Some(crabomination::mana::Color::Red);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).unwrap();
    let pool = &mut g.players[0].mana_pool;
    assert_eq!(pool.restricted_total(), 2, "two restricted red");
    let cost = ManaCost::new(vec![crabomination::mana::generic(2)]);
    assert!(pool.pay_for_spell(&cost, &SpellKind::default()).is_err(), "can't fund a spell");
    let land_kind = SpellKind { land_ability: true, ..Default::default() };
    assert!(pool.pay_for_spell(&cost, &land_kind).is_ok(), "funds a land ability");
}

/// Soulless Jailer blocks permanent reanimation from graveyards and
/// noncreature casts from graveyards/exile, but not creature flashback-style
/// casts of creatures or hand casts.
#[test]
fn soulless_jailer_locks_graveyard_entries_and_noncreature_recasts() {
    use crabomination::effect::{Effect, ZoneDest};
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::soulless_jailer());
    // A permanent card can't enter from the graveyard…
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    g.resolve_effect(
        &Effect::Move {
            what: crabomination::effect::Selector::Target(0),
            to: ZoneDest::Battlefield { controller: crabomination::effect::PlayerRef::You, tapped: false },
        },
        &ctx,
    )
    .unwrap();
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear), "reanimation blocked");
    // …and a noncreature flashback cast is refused.
    let dart = g.add_card_to_graveyard(0, catalog::lava_dart());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 2);
    g.add_card_to_battlefield(0, catalog::mountain());
    assert!(
        g.perform_action(GameAction::CastFlashback {
            card_id: dart, target: Some(Target::Player(1)), additional_targets: vec![],
            mode: None, x_value: None,
        })
        .is_err(),
        "noncreature graveyard cast blocked"
    );
}

/// Vedalken Plotter's ETB swaps control of your land and an opponent's.
#[test]
fn vedalken_plotter_swaps_land_control() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::mountain());
    let theirs = g.add_card_to_battlefield(1, catalog::island());
    let plotter = g.add_card_to_hand(0, catalog::vedalken_plotter());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    crabomination::game::cast_at(&mut g, plotter, Target::Permanent(theirs));
    assert_eq!(g.battlefield_find(mine).unwrap().controller, 1, "my land handed over");
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0, "their land taken");
}

/// Oblivion Sower's cast trigger exiles four and takes the opponent's lands
/// from exile.
#[test]
fn oblivion_sower_takes_exiled_lands() {
    let mut g = two_player_game();
    // Top four of opponent's library: two lands, two spells.
    g.add_card_to_library(1, catalog::island());
    g.add_card_to_library(1, catalog::lightning_bolt());
    g.add_card_to_library(1, catalog::island());
    g.add_card_to_library(1, catalog::lightning_bolt());
    let sower = g.add_card_to_hand(0, catalog::oblivion_sower());
    g.players[0].mana_pool.add_colorless(6);
    g.step = TurnStep::PreCombatMain;
    crabomination::game::cast(&mut g, sower);
    let my_lands = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.is_land() && c.owner == 1)
        .count();
    assert_eq!(my_lands, 2, "both exiled lands stolen");
    assert!(g.battlefield_find(sower).is_some(), "Sower resolved to the battlefield");
}

/// Mind Grind mills each opponent down to X lands revealed.
#[test]
fn mind_grind_mills_until_x_lands() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::lightning_bolt());
        g.add_card_to_library(1, catalog::island());
    }
    let grind = g.add_card_to_hand(0, catalog::mind_grind());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: grind, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).unwrap();
    drain_stack(&mut g);
    // bolt, island, bolt, island — stops at the second land.
    assert_eq!(g.players[1].graveyard.len(), 4, "milled through the second land");
}

/// Sphinx's Tutelage repeats while the two milled nonlands share a color.
#[test]
fn sphinxs_tutelage_repeats_on_shared_color() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sphinxs_tutelage());
    // Opponent's top: two red spells (shared color → repeat), then a land
    // pair (no nonland pair → stop).
    g.add_card_to_library(1, catalog::lightning_bolt());
    g.add_card_to_library(1, catalog::lightning_bolt());
    g.add_card_to_library(1, catalog::island());
    g.add_card_to_library(1, catalog::island());
    g.add_card_to_library(1, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    let mut events = Vec::new();
    g.draw_one(0, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 4, "two bolts, repeat, two islands, stop");
}

/// Processor Assault wants an opponent-owned exile card as fuel.
#[test]
fn processor_assault_processes_exile_card() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::serra_angel());
    let assault = g.add_card_to_hand(0, catalog::processor_assault());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    // No opponent card in exile → cast is rejected.
    assert!(g
        .perform_action(GameAction::CastSpell {
            card_id: assault, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        })
        .is_err());
    // Exile an opponent-owned card; the cast processes it and burns for 5.
    let mut fuel = crabomination::card::CardInstance::new(g.next_id(), catalog::island(), 1);
    fuel.owner = 1;
    let fuel_id = fuel.id;
    g.exile.push(fuel);
    crabomination::game::cast_at(&mut g, assault, Target::Permanent(bear));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == fuel_id), "processed to gy");
    assert!(g.battlefield_find(bear).is_none(), "Serra Angel burned for 5");
}

/// Karplusan Minotaur's cumulative upkeep flips coins; each win/loss pings.
#[test]
fn karplusan_minotaur_upkeep_flips_ping() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let minotaur = g.add_card_to_battlefield(0, catalog::karplusan_minotaur());
    // One age counter → one flip; script a win, then aim the ping.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
    ]));
    let events = g.process_cumulative_upkeep();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(minotaur).is_some(), "flip cost always payable");
    assert_eq!(g.players[1].life, 19, "won flip pings for 1");
}

/// Consuming Aberration's cast trigger mills each opponent through their
/// first land (faithful reveal-until-land, not the old flat mill 3).
#[test]
fn consuming_aberration_mills_until_land() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::consuming_aberration());
    g.add_card_to_library(1, catalog::lightning_bolt());
    g.add_card_to_library(1, catalog::lightning_bolt());
    g.add_card_to_library(1, catalog::island());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    crabomination::game::cast_at(&mut g, bolt, Target::Player(1));
    assert_eq!(g.players[1].graveyard.len(), 3, "two bolts revealed, stops at the island");
}

/// Underworld Breach grants graveyard nonlands escape for their mana cost
/// plus exiling three other cards.
#[test]
fn underworld_breach_grants_escape() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::underworld_breach());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let f1 = g.add_card_to_graveyard(0, catalog::forest());
    let f2 = g.add_card_to_graveyard(0, catalog::forest());
    let f3 = g.add_card_to_graveyard(0, catalog::forest());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastEscape {
        card_id: bolt, exile_cards: vec![f1, f2, f3],
        target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "bolt escaped");
    // The three forests are exiled; the resolved bolt returns to the yard.
    assert_eq!(g.exile.len(), 3, "fuel exiled");
    assert_eq!(g.players[0].graveyard.len(), 1, "just the resolved bolt");
}

/// Six grants retrace to graveyard nonland permanents during your turn only.
#[test]
fn six_grants_retrace_during_your_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::six());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    // A land stays in hand to fund the retrace discard cost.
    g.add_card_to_hand(0, catalog::forest());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    // Not your turn → no retrace.
    let bear_card = g.players[0].graveyard.iter().find(|c| c.id == bear).unwrap().clone();
    assert!(!g.effective_retrace(&bear_card, 0));
    g.active_player_idx = 0;
    assert!(g.effective_retrace(&bear_card, 0));
    g.perform_action(GameAction::CastRetrace {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "bear retraced onto the battlefield");
}

/// Student of Warfare's level bands set base P/T and grant keywords
/// (CR 702.87); level up is sorcery-speed.
#[test]
fn student_of_warfare_levels_up() {
    use crabomination::card::{CounterType, Keyword};
    let mut g = two_player_game();
    let student = g.add_card_to_battlefield(0, catalog::student_of_warfare());
    g.step = TurnStep::PreCombatMain;
    let c = g.computed_permanent(student).unwrap();
    assert_eq!((c.power, c.toughness), (1, 1));
    for _ in 0..2 {
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: student, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).unwrap();
        drain_stack(&mut g);
    }
    let c = g.computed_permanent(student).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "level 2 band");
    assert!(c.keywords.contains(&Keyword::FirstStrike));
    // Jump to level 7.
    g.battlefield_find_mut(student).unwrap().add_counters(CounterType::Level, 5);
    let c = g.computed_permanent(student).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4), "level 7+ band");
    assert!(c.keywords.contains(&Keyword::DoubleStrike));
    // Level up is sorcery-speed only.
    g.step = TurnStep::DeclareBlockers;
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: student, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).is_err());
}

/// The Ozolith catches a leaver's counters and unloads at begin combat.
#[test]
fn the_ozolith_collects_and_unloads_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let ozo = g.add_card_to_battlefield(0, catalog::the_ozolith());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    let mut evs = Vec::new();
    g.sacrifice_one(bear, 0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(ozo).unwrap().counter_count(CounterType::PlusOnePlusOne),
        3,
        "counters caught"
    );
    // Begin combat on our turn: move them onto a creature.
    let target = g.add_card_to_battlefield(0, catalog::serra_angel());
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne),
        3,
        "counters delivered"
    );
    assert_eq!(g.battlefield_find(ozo).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Nadu fires when your creatures are targeted — land to battlefield, else
/// to hand — capped at twice per creature per turn.
#[test]
fn nadu_reveals_on_target_capped_per_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::nadu_winged_wisdom());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::forest());
    let lands_before = g.battlefield.iter().filter(|c| c.definition.is_land()).count();
    // Target the bear three times: only the first two fire.
    for _ in 0..3 {
        g.dispatch_triggers_for_events(&[GameEvent::BecameTarget { target: bear, caster: 0 }]);
        drain_stack(&mut g);
    }
    let lands_after = g.battlefield.iter().filter(|c| c.definition.is_land()).count();
    assert_eq!(lands_after - lands_before, 1, "first reveal: forest to battlefield");
    assert_eq!(g.players[0].hand.len(), 1, "second reveal: bolt to hand");
    assert_eq!(g.players[0].library.len(), 1, "third target capped — library untouched");
}

/// Springheart Nantuko's landfall: attached → token copy of the host;
/// unattached → 1/1 Insect (always for {1}{G}, may decline).
#[test]
fn springheart_nantuko_landfall_copies_host_or_makes_insect() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let nantuko = g.add_card_to_battlefield(0, catalog::springheart_nantuko());
    // Unattached: pay → Insect.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let land = g.add_card_to_hand(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::PlayLand(land)).unwrap();
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Insect" && c.is_token),
        "insect minted while unattached"
    );
    // Attached to a Serra Angel: pay → token copy of the angel.
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    g.battlefield_find_mut(nantuko).unwrap().attached_to = Some(angel);
    g.battlefield_find_mut(nantuko).unwrap().bestowed = true;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].lands_played_this_turn = 0;
    let land2 = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land2)).unwrap();
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield
            .iter()
            .filter(|c| c.definition.name == "Serra Angel" && c.is_token)
            .count(),
        1,
        "token copy of the host"
    );
}

/// Kozilek's cast trigger manifests two from the opponent's hand and draws.
#[test]
fn kozilek_broken_reality_manifests_opponent_hand() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::island());
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let koz = g.add_card_to_hand(0, catalog::kozilek_the_broken_reality());
    g.players[0].mana_pool.add_colorless(9);
    let hand_before = g.players[0].hand.len() - 1; // minus Kozilek itself
    g.step = TurnStep::PreCombatMain;
    crabomination::game::cast(&mut g, koz);
    let manifested = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 1 && c.face_down)
        .count();
    assert_eq!(manifested, 2, "two cards manifested face down");
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew one per manifest");
    // Anthem: other colorless creatures get +3/+2 (manifests are colorless 2/2s).
    let face_down_pt = g
        .battlefield
        .iter()
        .find(|c| c.controller == 1 && c.face_down)
        .map(|c| c.id)
        .and_then(|id| g.computed_permanent(id))
        .map(|c| (c.power, c.toughness));
    assert_eq!(face_down_pt, Some((2, 2)), "opponent's manifest unaffected by my anthem");
}

/// Ulamog, the Defiler: exiles half the library, enters with counters equal
/// to the greatest exiled MV, computes annihilator from them, and wards with
/// a two-permanent sacrifice.
#[test]
fn ulamog_defiler_counters_and_annihilator() {
    use crabomination::card::{CounterType, Keyword};
    let mut g = two_player_game();
    g.players[1].library.clear();
    for _ in 0..3 { g.add_card_to_library(1, catalog::island()); }
    g.add_card_to_library(1, catalog::ulamog_the_ceaseless_hunger()); // MV 10 in library
    let ula = g.add_card_to_hand(0, catalog::ulamog_the_defiler());
    g.players[0].mana_pool.add_colorless(10);
    g.step = TurnStep::PreCombatMain;
    crabomination::game::cast(&mut g, ula);
    // Top half rounded up of 4 = 2 exiled (island, island).
    assert_eq!(g.players[1].library.len(), 2);
    let c = g.battlefield_find(ula).unwrap();
    // Greatest MV in exile is 0 (two Islands) — unless something else sits
    // in exile; counters equal that greatest MV.
    assert_eq!(
        c.counter_count(CounterType::PlusOnePlusOne) > 0,
        g.exile.iter().any(|x| x.definition.cost.cmc() > 0),
    );
    // Force five counters and read annihilator off the computed view.
    g.battlefield_find_mut(ula).unwrap().counters.insert(CounterType::PlusOnePlusOne, 5);
    let kws = g.computed_permanent(ula).unwrap().keywords;
    assert!(kws.contains(&Keyword::Annihilator(5)), "annihilator scales with counters");
}

/// Karn, the Great Creator: opponents can't activate artifact abilities;
/// +1 animates a noncreature artifact; -2 wishes an artifact from the
/// sideboard.
#[test]
fn karn_great_creator_locks_animates_and_wishes() {
    let mut g = two_player_game();
    let karn = g.add_card_to_battlefield(0, catalog::karn_the_great_creator());
    // Opponent's mana rock is locked out.
    let rock = g.add_card_to_battlefield(1, catalog::mind_stone());
    g.priority.player_with_priority = 1;
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: rock, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).is_err(), "opponent's artifact ability locked");
    // Our own artifacts are fine.
    let mine = g.add_card_to_battlefield(0, catalog::mind_stone());
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: mine, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("own rock still works");
    // +1: animate the opponent's rock (MV 2) into a 2/2 until next turn.
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: karn, ability_index: 0, target: Some(Target::Permanent(rock)), x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    let c = g.computed_permanent(rock).unwrap();
    assert!(c.card_types.contains(&crabomination::card::CardType::Creature));
    assert_eq!((c.power, c.toughness), (2, 2), "MV/MV body");
    // -2: wish an artifact from the sideboard.
    let wish_target = crabomination::card::CardInstance::new(g.next_id(), catalog::mind_stone(), 0);
    let wid = wish_target.id;
    g.players[0].sideboard.push(wish_target);
    g.turn_number += 1; // fresh loyalty window
    g.battlefield_find_mut(karn).unwrap().loyalty_uses_this_turn = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: karn, ability_index: 1, target: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == wid), "wished to hand");
}

/// Not Dead After All: the granted die-trigger reanimates the creature
/// tapped with a Wicked Role attached; the Role pumps +1/+1.
#[test]
fn not_dead_after_all_returns_with_wicked_role() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::not_dead_after_all());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.step = TurnStep::PreCombatMain;
    crabomination::game::cast_at(&mut g, spell, Target::Permanent(bear));
    let mut evs = Vec::new();
    g.sacrifice_one(bear, 0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let back = g.battlefield_find(bear).expect("returned to the battlefield");
    assert!(back.tapped, "returns tapped");
    let role = g
        .battlefield
        .iter()
        .find(|c| c.is_token && c.definition.name == "Wicked")
        .expect("Wicked Role minted");
    assert_eq!(role.attached_to, Some(bear), "attached to the returned creature");
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "Role grants +1/+1");
    // The Role hitting the graveyard drains each opponent for 1.
    let role_id = role.id;
    g.remove_to_graveyard_with_triggers(role_id);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "each opponent loses 1");
}

/// Ajani, Nacatl Pariah flips when another Cat dies; the Avenger's -4 makes
/// each opponent keep one permanent of each type.
#[test]
fn ajani_nacatl_pariah_flips_and_cataclysms() {
    use crabomination::card::CardType;
    let mut g = two_player_game();
    let ajani = g.add_card_to_hand(0, catalog::ajani_nacatl_pariah());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    crabomination::game::cast(&mut g, ajani);
    let cat = g
        .battlefield
        .iter()
        .find(|c| c.is_token && c.definition.name == "Cat Warrior")
        .map(|c| c.id)
        .expect("ETB Cat token");
    // The token Cat dies → Ajani returns transformed as a planeswalker.
    let mut evs = Vec::new();
    g.sacrifice_one(cat, 0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let walker = g.battlefield_find(ajani).expect("Ajani back");
    assert!(walker.transformed, "returned transformed");
    assert!(walker.definition.is_planeswalker());
    assert_eq!(walker.counter_count(crabomination::card::CounterType::Loyalty), 3);
    // -4: opponent keeps one creature (their best), sacrifices the rest.
    g.battlefield_find_mut(ajani).unwrap().add_counters(crabomination::card::CounterType::Loyalty, 2);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::serra_angel());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: ajani, ability_index: 2, target: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    let opp_creatures: Vec<&str> = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 1 && c.definition.card_types.contains(&CardType::Creature))
        .map(|c| c.definition.name)
        .collect();
    assert_eq!(opp_creatures, vec!["Serra Angel"], "kept the highest-MV creature");
}

/// Indomitable Creativity at X=2 destroys both targets; each controller
/// polymorphs into their next artifact/creature.
#[test]
fn indomitable_creativity_polymorphs_destroyed_targets() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].library.clear();
    g.players[1].library.clear();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::serra_angel());
    g.add_card_to_library(1, catalog::mind_stone());
    let spell = g.add_card_to_hand(0, catalog::indomitable_creativity());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 3);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)],
        mode: None,
        x_value: Some(2),
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none());
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Serra Angel"),
        "my bear became the Angel (island shuffled back)"
    );
    assert!(
        g.battlefield.iter().any(|c| c.controller == 1 && c.definition.name == "Mind Stone"),
        "their bear became the rock"
    );
}

/// The completed Landscape cycle fetches one of its three basic types
/// tapped; the new Verges gate their second color on the named land types.
#[test]
fn landscape_and_verge_cycle_completion() {
    let mut g = two_player_game();
    // Landscape: sac-fetch a basic Swamp/Forest/Island tapped.
    let scape = g.add_card_to_battlefield(0, catalog::foreboding_landscape());
    g.clear_sickness(scape);
    g.players[0].library.clear();
    let swamp = g.add_card_to_library(0, catalog::swamp());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new(vec![
        DecisionAnswer::Search(Some(swamp)),
    ]));
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: scape, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).unwrap();
    drain_stack(&mut g);
    let fetched = g.battlefield_find(swamp).expect("Swamp fetched");
    assert!(fetched.tapped);
    assert!(g.battlefield_find(scape).is_none(), "Landscape sacrificed");
    // Verge: the gated color needs a Forest or Island in play.
    let verge = g.add_card_to_battlefield(0, catalog::willowrush_verge());
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: verge, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).is_err(), "no Forest/Island yet");
    g.add_card_to_battlefield(0, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: verge, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("gated {G} with a Forest in play");
    assert_eq!(g.players[0].mana_pool.amount(crabomination::mana::Color::Green), 1);
}

// ── Epic (CR 702.50) ─────────────────────────────────────────────────────────

/// Enduring Ideal resolves: fetches an enchantment to the battlefield, locks
/// the caster out of casting spells, and copies itself each upkeep.
#[test]
fn enduring_ideal_epic_locks_casting_and_copies_each_upkeep() {
    let mut g = two_player_game();
    // Library: two trigger-free enchantments to fetch.
    let e1 = g.add_card_to_library(0, catalog::rest_in_peace());
    let e2 = g.add_card_to_library(0, catalog::rest_in_peace());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(e1)),
        DecisionAnswer::Search(Some(e2)),
    ]));
    let ideal = g.add_card_to_hand(0, catalog::enduring_ideal());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: ideal, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Enduring Ideal");
    drain_stack(&mut g);
    let enchantments_on_bf = |g: &crabomination::game::GameState| {
        g.battlefield
            .iter()
            .filter(|c| {
                c.controller == 0
                    && c.definition.card_types.contains(&crabomination::card::CardType::Enchantment)
            })
            .count()
    };
    assert_eq!(enchantments_on_bf(&g), 1, "fetched an enchantment");
    assert_eq!(g.players[0].epic_spells.len(), 1, "epic snapshot stored");

    // CR 702.50b — the caster can't cast spells anymore.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    let err = g
        .perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        })
        .expect_err("epic lock");
    assert!(matches!(err, GameError::EpicLocked), "got {err:?}");

    // CR 702.50a — at the controller's upkeep the spell is copied.
    g.step = TurnStep::Upkeep;
    g.active_player_idx = 0;
    let _ = g.process_epic();
    assert!(!g.stack.is_empty(), "epic copy on the stack");
    drain_stack(&mut g);
    assert_eq!(enchantments_on_bf(&g), 2, "copy fetched another enchantment");
}

/// The epic copy is itself uncounterable and doesn't re-arm Epic (one
/// snapshot, not one per upkeep).
#[test]
fn epic_copy_does_not_stack_additional_snapshots() {
    let mut g = two_player_game();
    let e1 = g.add_card_to_library(0, catalog::rest_in_peace());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(e1)),
        DecisionAnswer::Search(None),
    ]));
    let ideal = g.add_card_to_hand(0, catalog::enduring_ideal());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: ideal, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    g.step = TurnStep::Upkeep;
    g.active_player_idx = 0;
    let _ = g.process_epic();
    drain_stack(&mut g);
    assert_eq!(g.players[0].epic_spells.len(), 1, "copy resolution doesn't re-arm Epic");
}

