//! Functionality tests for `catalog::sets::decks::recent214`.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, Selector, Value};
use crabomination::game::types::{Attack, AttackTarget, Target};
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    let mut guard = 0;
    while g.step != step && guard < 40 {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
        guard += 1;
    }
}

/// Biogenic Upgrade distributes three counters onto a lone target, then doubles
/// them (3 → 6) via the new `Selector::AllTargets`.
#[test]
fn biogenic_upgrade_distributes_then_doubles() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::biogenic_upgrade());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Biogenic Upgrade");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 6,
        "3 distributed then doubled to 6");
}

/// Herald of Faith flies and gains 2 life when it attacks.
#[test]
fn herald_of_faith_flies_and_gains_life() {
    let mut g = two_player_game();
    let herald = g.add_card_to_battlefield(0, catalog::herald_of_faith());
    assert!(g.computed_permanent(herald).unwrap().keywords.contains(&Keyword::Flying));
    g.clear_sickness(herald);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: herald, target: AttackTarget::Player(1) }]).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 22, "gained 2 on attack");
}

/// Arcanis draws three, then bounces itself to hand.
#[test]
fn arcanis_draws_and_bounces() {
    let mut g = two_player_game();
    let arcanis = g.add_card_to_battlefield(0, catalog::arcanis_the_omnipotent());
    for _ in 0..5 { g.add_card_to_library(0, catalog::forest()); }
    g.clear_sickness(arcanis);
    g.step = TurnStep::PreCombatMain;
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: arcanis, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("draw three");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 3, "drew three cards");
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: arcanis, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("bounce self");
    drain_stack(&mut g);
    assert!(g.battlefield_find(arcanis).is_none(), "Arcanis left the battlefield");
    assert!(g.players[0].hand.iter().any(|c| c.id == arcanis), "returned to hand");
}

/// Confiscate steals control of an opponent's permanent while attached.
#[test]
fn confiscate_steals_control() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::confiscate());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Confiscate");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0, "control stolen");
}

/// Unflinching Courage grants +2/+2, trample, and lifelink.
#[test]
fn unflinching_courage_buffs() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::unflinching_courage());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Unflinching Courage");
    drain_stack(&mut g);
    let v = g.computed_permanent(bear).unwrap();
    assert_eq!((v.power, v.toughness), (4, 4), "+2/+2");
    assert!(v.keywords.contains(&Keyword::Trample) && v.keywords.contains(&Keyword::Lifelink));
}

/// Suspicious Shambler exiles itself from the graveyard to make two Zombies.
#[test]
fn suspicious_shambler_recurs_zombies() {
    let mut g = two_player_game();
    let shambler = g.add_card_to_graveyard(0, catalog::suspicious_shambler());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: shambler, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate from graveyard");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == shambler), "exiled as a cost");
    let zombies = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Zombie").count();
    assert_eq!(zombies, 2, "two 2/2 Zombie tokens");
}

/// Kalastria Highborn drains 2 when another Vampire you control dies and you
/// pay {B}. Killed via a real bolt so the full SBA+dispatch path fires.
#[test]
fn kalastria_highborn_drains_on_vampire_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kalastria_highborn());
    let fodder = g.add_card_to_battlefield(0, catalog::kalastria_highborn()); // another 2/2 Vampire
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Black, 1); // for the "may pay {B}"
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(fodder)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the fodder Vampire");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "each opponent lost 2");
    assert_eq!(g.players[0].life, 22, "you gained 2");
}

/// Kargan Dragonrider only flies while you control a Dragon.
#[test]
fn kargan_flies_with_a_dragon() {
    let mut g = two_player_game();
    let kargan = g.add_card_to_battlefield(0, catalog::kargan_dragonrider());
    assert!(!g.computed_permanent(kargan).unwrap().keywords.contains(&Keyword::Flying), "no Dragon → no flying");
    g.add_card_to_battlefield(0, catalog::sprite_dragon());
    assert!(g.computed_permanent(kargan).unwrap().keywords.contains(&Keyword::Flying), "Dragon → flying");
}

/// Kitesail Corsair gains flying only while attacking.
#[test]
fn kitesail_flies_while_attacking() {
    let mut g = two_player_game();
    let corsair = g.add_card_to_battlefield(0, catalog::kitesail_corsair());
    assert!(!g.computed_permanent(corsair).unwrap().keywords.contains(&Keyword::Flying), "idle → no flying");
    g.clear_sickness(corsair);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: corsair, target: AttackTarget::Player(1) }]).expect("attack");
    assert!(g.computed_permanent(corsair).unwrap().keywords.contains(&Keyword::Flying), "attacking → flying");
}

/// Sphinx of the Final Word makes its controller's instants/sorceries
/// uncounterable.
#[test]
fn sphinx_makes_your_spells_uncounterable() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sphinx_of_the_final_word());
    let mine = crabomination::card::CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
    let theirs = crabomination::card::CardInstance::new(g.next_id(), catalog::lightning_bolt(), 1);
    assert!(g.caster_grants_uncounterable(0, &mine), "your instant is uncounterable");
    assert!(!g.caster_grants_uncounterable(1, &theirs), "opponent's is not");
}

/// Drogskol Reaver draws a card whenever you gain life.
#[test]
fn drogskol_draws_on_lifegain() {
    let mut g = two_player_game();
    let reaver = g.add_card_to_battlefield(0, catalog::drogskol_reaver());
    g.add_card_to_library(0, catalog::forest());
    let before = g.players[0].hand.len();
    let evs = g.resolve_effect(&Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        &crabomination::game::effects::EffectContext::for_ability(reaver, 0, None)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1, "drew a card off lifegain");
}

/// Primeval Bounty mints a 3/3 Beast when you cast a creature spell.
#[test]
fn primeval_bounty_mints_beast_on_creature_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::primeval_bounty());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a creature");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Beast"),
        "cast trigger made a Beast");
}

/// Deadly Plot's second mode reanimates a Zombie tapped.
#[test]
fn deadly_plot_reanimates_zombie() {
    let mut g = two_player_game();
    let zombie = g.add_card_to_graveyard(0, catalog::suspicious_shambler()); // a Zombie creature
    let spell = g.add_card_to_hand(0, catalog::deadly_plot());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(zombie)),
        additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("cast Deadly Plot mode 1");
    drain_stack(&mut g);
    let z = g.battlefield_find(zombie).expect("reanimated");
    assert_eq!(z.controller, 0);
    assert!(z.tapped, "returns tapped");
}

/// Surrak grants haste at combat when creatures you control total 8+ power.
#[test]
fn surrak_grants_haste_when_formidable() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::surrak_the_hunt_caller()); // 5/4
    let vamp = g.add_card_to_battlefield(0, catalog::highborn_vampire()); // 4/3 → total 9
    g.active_player_idx = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(vamp))]));
    advance_to(&mut g, TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert!(g.computed_permanent(vamp).unwrap().keywords.contains(&Keyword::Haste), "gained haste");
}

/// Gateway Sneak becomes unblockable when a Gate you control enters.
#[test]
fn gateway_sneak_unblockable_on_gate() {
    let mut g = two_player_game();
    let sneak = g.add_card_to_battlefield(0, catalog::gateway_sneak());
    let gate = g.add_card_to_hand(0, catalog::azorius_guildgate());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(gate)).expect("play a Gate");
    drain_stack(&mut g);
    assert!(g.computed_permanent(sneak).unwrap().keywords.contains(&Keyword::Unblockable),
        "Gate entering made Gateway Sneak unblockable");
}

/// Shipwreck Dowser returns an instant/sorcery from your graveyard on ETB.
#[test]
fn shipwreck_dowser_returns_instant() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let dowser = g.add_card_to_hand(0, catalog::shipwreck_dowser());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bolt))]));
    g.perform_action(GameAction::CastSpell {
        card_id: dowser, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Shipwreck Dowser");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "returned the bolt to hand");
}

/// Prayer of Binding exiles an opponent's permanent and gains 2 life.
#[test]
fn prayer_of_binding_exiles_and_gains_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let prayer = g.add_card_to_hand(0, catalog::prayer_of_binding());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: prayer, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Prayer of Binding");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "opponent's creature exiled");
    assert_eq!(g.players[0].life, 22, "gained 2 life");
}

/// Wildborn Preserver grows by X when another non-Human enters and you pay {X}.
#[test]
fn wildborn_preserver_grows_on_nonhuman_etb() {
    let mut g = two_player_game();
    let elf = g.add_card_to_battlefield(0, catalog::wildborn_preserver());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears()); // a Bear (non-Human)
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3); // 1 for the bear + 2 for the pay
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast the Bear");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(elf).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "paid {{2}} → two +1/+1 counters");
}

/// Immersturm Predator's sac ability taps it (indestructible), and the
/// becomes-tapped triggers grow it and exile a graveyard card.
#[test]
fn immersturm_predator_sac_taps_and_grows() {
    let mut g = two_player_game();
    let pred = g.add_card_to_battlefield(0, catalog::immersturm_predator());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let gy = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    g.clear_sickness(pred);
    g.perform_action(GameAction::ActivateAbility {
        card_id: pred, ability_index: 0,
        target: Some(Target::Permanent(fodder)), additional_targets: Vec::new(), x_value: None,
    }).expect("sac another creature");
    drain_stack(&mut g);
    let cp = g.computed_permanent(pred).expect("predator alive");
    assert!(cp.keywords.contains(&Keyword::Indestructible), "gained indestructible");
    assert!(g.battlefield_find(pred).unwrap().tapped, "tapped by its own ability");
    assert_eq!(g.battlefield_find(pred).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "becomes-tapped grew it");
    assert!(g.exile.iter().any(|c| c.id == gy), "exiled a graveyard card");
}

/// Gratuitous Violence doubles a controlled creature's damage but not the
/// opponent's (source-restricted CR 614.2 doubler).
#[test]
fn gratuitous_violence_doubles_only_your_creatures() {
    use crabomination::game::effects::EntityRef;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::gratuitous_violence());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert_eq!(g.scale_damage_to(Some(mine), EntityRef::Player(1), 2), 4, "your creature doubles");
    assert_eq!(g.scale_damage_to(Some(theirs), EntityRef::Player(0), 2), 2, "opponent's is normal");
}
