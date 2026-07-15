//! Functionality tests for the `catalog::sets::decks::recent97` Kamigawa: Neon
//! Dynasty batch 3.

use crabomination::catalog;
use crabomination::card::{CounterType, CreatureType, Keyword};
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, Target};
use crabomination::game::*;

/// Advance from the current step to PostCombatMain, resolving combat damage.
fn pass_through_combat(g: &mut GameState) {
    while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(g);
}

/// Kappa Tech-Wrecker enters with a deathtouch keyword counter.
#[test]
fn kappa_enters_with_deathtouch_counter() {
    let mut g = two_player_game();
    let kappa = g.add_card_to_battlefield(0, catalog::kappa_tech_wrecker());
    g.fire_self_etb_triggers(kappa, 0);
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(kappa).unwrap().keywords.contains(&Keyword::Deathtouch),
        "deathtouch counter grants the keyword"
    );
}

/// Kappa's combat damage may remove the counter to exile an opponent's artifact.
#[test]
fn kappa_combat_exiles_artifact() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::bonesplitter());
    let kappa = g.add_card_to_battlefield(0, catalog::kappa_tech_wrecker());
    g.fire_self_etb_triggers(kappa, 0);
    drain_stack(&mut g);
    g.clear_sickness(kappa);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(art)),
    ]));
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: kappa,
        target: AttackTarget::Player(1),
    }]))
    .expect("kappa attacks");
    drain_stack(&mut g);
    pass_through_combat(&mut g);
    assert!(g.battlefield_find(art).is_none(), "artifact exiled by Kappa");
}

/// Biting-Palm Ninja enters with a menace keyword counter.
#[test]
fn biting_palm_enters_with_menace_counter() {
    let mut g = two_player_game();
    let ninja = g.add_card_to_battlefield(0, catalog::biting_palm_ninja());
    g.fire_self_etb_triggers(ninja, 0);
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(ninja).unwrap().keywords.contains(&Keyword::Menace),
        "menace counter grants the keyword"
    );
}

/// Kami of Restless Shadows returns a Ninja from your graveyard (mode 0).
#[test]
fn kami_restless_shadows_returns_ninja() {
    let mut g = two_player_game();
    let ninja = g.add_card_to_graveyard(0, catalog::dokuchi_silencer());
    let kami = g.add_card_to_battlefield(0, catalog::kami_of_restless_shadows());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Mode(0),
        DecisionAnswer::Target(Target::Permanent(ninja)),
    ]));
    g.fire_self_etb_triggers(kami, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == ninja), "Ninja returned to hand");
}

/// Explosive Entry destroys an artifact and puts a +1/+1 counter on a creature.
#[test]
fn explosive_entry_destroys_and_counters() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::bonesplitter());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::explosive_entry());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(art)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: None,
    })
    .expect("cast Explosive Entry");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "bear got a +1/+1 counter"
    );
}

/// Blade of the Oni sets the equipped creature to a 5/5 menacing Demon.
#[test]
fn blade_of_the_oni_makes_a_demon() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blade = g.add_card_to_battlefield(0, catalog::blade_of_the_oni());
    g.battlefield.iter_mut().find(|c| c.id == blade).unwrap().attached_to = Some(bear);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "base P/T set to 5/5");
    assert!(cp.keywords.contains(&Keyword::Menace), "gains menace");
    assert!(cp.subtypes.creature_types.contains(&CreatureType::Demon), "is a Demon");
}

/// Towashi Guide-Bot's ETB puts a +1/+1 counter on a creature you control.
#[test]
fn towashi_guide_bot_etb_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bot = g.add_card_to_battlefield(0, catalog::towashi_guide_bot());
    g.fire_self_etb_triggers(bot, 0);
    drain_stack(&mut g);
    let placed: u32 = [bear, bot]
        .iter()
        .map(|id| g.battlefield_find(*id).unwrap().counter_count(CounterType::PlusOnePlusOne))
        .sum();
    assert_eq!(placed, 1, "one +1/+1 counter placed");
}

/// Towashi Guide-Bot's draw ability is wired to discount per modified creature.
#[test]
fn towashi_guide_bot_cost_reduction() {
    let def = catalog::towashi_guide_bot();
    let ability = &def.activated_abilities[0];
    let filter = ability.cost_reduction_per.as_ref().expect("draws discount per modified creature");
    // Reduction is gated on modified creatures you control.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert!(
        g.evaluate_requirement_static(filter, &Target::Permanent(bear), 0, None),
        "a modified creature satisfies the discount filter"
    );
}

/// Naomi's ETB makes a Samurai when you control an artifact and an enchantment.
#[test]
fn naomi_makes_samurai_with_artifact_and_enchantment() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bonesplitter()); // artifact
    g.add_card_to_battlefield(0, catalog::golden_tail_disciple()); // enchantment creature
    let naomi = g.add_card_to_battlefield(0, catalog::naomi_pillar_of_order());
    g.fire_self_etb_triggers(naomi, 0);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Samurai"),
        "Samurai token created"
    );
}

/// Jukai Trainee grows when it becomes blocked.
#[test]
fn jukai_trainee_pumps_when_blocked() {
    let mut g = two_player_game();
    let trainee = g.add_card_to_battlefield(0, catalog::jukai_trainee());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(trainee);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: trainee,
        target: AttackTarget::Player(1),
    }]))
    .expect("trainee attacks");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, trainee)]))
        .expect("bear blocks");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(trainee).unwrap().power, 3, "2/2 + 1/1 when blocked");
}

/// Gloomshrieker returns a permanent from your graveyard and has menace.
#[test]
fn gloomshrieker_returns_from_graveyard() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let gloom = g.add_card_to_battlefield(0, catalog::gloomshrieker());
    g.fire_self_etb_triggers(gloom, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "grizzly returned to hand");
    assert!(
        g.computed_permanent(gloom).unwrap().keywords.contains(&Keyword::Menace),
        "Gloomshrieker has menace"
    );
}

/// Gloomshrieker exiles itself instead of dying (CR 603-style self-replacement).
#[test]
fn gloomshrieker_exiles_itself_on_death() {
    let mut g = two_player_game();
    let gloom = g.add_card_to_battlefield(0, catalog::gloomshrieker());
    g.remove_to_graveyard_with_triggers(gloom);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().all(|c| c.id != gloom), "not in graveyard");
    assert!(g.exile.iter().any(|c| c.id == gloom), "exiled instead");
}

/// Ecologist's Terrarium tutors a basic land to hand on entry.
#[test]
fn ecologists_terrarium_fetches_basic() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    let terr = g.add_card_to_battlefield(0, catalog::ecologists_terrarium());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(forest)),
    ]));
    g.fire_self_etb_triggers(terr, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == forest), "basic land in hand");
}

/// Scrapwork Mutt loots on entry (discard a card to draw a card).
#[test]
fn scrapwork_mutt_loots_on_etb() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::grizzly_bears()); // fodder to discard
    g.add_card_to_library(0, catalog::island()); // to draw
    let mutt = g.add_card_to_battlefield(0, catalog::scrapwork_mutt());
    let before = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_self_etb_triggers(mutt, 0);
    drain_stack(&mut g);
    // Net hand size unchanged (discard 1, draw 1), and Scrapwork has Unearth.
    assert_eq!(g.players[0].hand.len(), before, "looted: discard one, draw one");
    assert!(
        !catalog::scrapwork_mutt().activated_abilities.is_empty(),
        "Scrapwork Mutt has an Unearth ability"
    );
}

/// Norika lets you cast an enchantment from your graveyard after a lone attack.
#[test]
fn norika_grants_graveyard_enchantment_cast() {
    let mut g = two_player_game();
    let ench = g.add_card_to_graveyard(0, catalog::golden_tail_disciple()); // enchantment creature
    let norika = g.add_card_to_battlefield(0, catalog::norika_yamazaki_the_poet());
    g.clear_sickness(norika);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(ench))]));
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: norika,
        target: AttackTarget::Player(1),
    }]))
    .expect("norika attacks alone");
    drain_stack(&mut g);
    let card = g.players[0].graveyard.iter().find(|c| c.id == ench).expect("still in gy");
    assert!(card.may_play_until.is_some(), "granted permission to cast from graveyard");
}

/// Kami of Celebration impulse-exiles the top card when a modified creature
/// you control attacks.
#[test]
fn kami_of_celebration_impulse_on_modified_attack() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::kami_of_celebration());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield
        .iter_mut()
        .find(|c| c.id == attacker)
        .unwrap()
        .add_counters(CounterType::PlusOnePlusOne, 1); // make it "modified"
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("modified bear attacks");
    drain_stack(&mut g);
    assert!(
        g.exile.iter().any(|c| c.owner == 0 && c.may_play_until.is_some()),
        "top card exiled with permission to play"
    );
}

/// Dokuchi Silencer may discard a creature to destroy an opponent's creature on
/// combat damage.
#[test]
fn dokuchi_silencer_destroys_on_combat_damage() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::grizzly_bears()); // creature card to discard
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == victim).unwrap().tapped = true; // can't block
    let ninja = g.add_card_to_battlefield(0, catalog::dokuchi_silencer());
    g.clear_sickness(ninja);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ninja,
        target: AttackTarget::Player(1),
    }]))
    .expect("dokuchi attacks");
    drain_stack(&mut g);
    pass_through_combat(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "victim destroyed");
}

/// Moonsnare Prototype taps another permanent for {C}.
#[test]
fn moonsnare_prototype_makes_mana() {
    let mut g = two_player_game();
    let helper = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let proto = g.add_card_to_battlefield(0, catalog::moonsnare_prototype());
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let _ = helper;
    g.perform_action(GameAction::ActivateAbility {
        card_id: proto,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate Moonsnare mana ability");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1, "produced colorless mana");
}
