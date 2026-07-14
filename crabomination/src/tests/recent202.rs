//! Functionality tests for `catalog::sets::decks::recent202`.

use crate::catalog;
use crate::game::types::Target;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

/// Rite of the Dragoncaller mints a 5/5 flying Dragon on each instant/sorcery cast.
#[test]
fn rite_of_the_dragoncaller_mints_dragon_on_spell() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::rite_of_the_dragoncaller());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let before = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_creature()).count();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Lightning Bolt");
    drain_stack(&mut g);
    let dragons = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.definition.name == "Dragon"
    }).count();
    assert_eq!(dragons, 1, "one 5/5 Dragon token minted");
    let after = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_creature()).count();
    assert_eq!(after, before + 1);
}

/// Koma makes four Koma's Coil tokens on combat damage to a player.
#[test]
fn koma_makes_four_coils_on_combat_damage() {
    let mut g = two_player_game();
    let koma = g.add_card_to_battlefield(0, catalog::koma_world_eater());
    g.fire_combat_damage_to_player_triggers(koma, 1, 8);
    drain_stack(&mut g);
    let coils = g.battlefield.iter().filter(|c| c.definition.name == "Koma's Coil").count();
    assert_eq!(coils, 4, "four 3/3 Serpent coils");
}

/// Koma can't be countered and has ward {4}.
#[test]
fn koma_is_uncounterable_and_warded() {
    use crate::card::{Keyword, WardCost};
    let k = catalog::koma_world_eater();
    assert!(k.keywords.contains(&Keyword::CantBeCountered));
    assert!(k.keywords.iter().any(|kw| matches!(kw, Keyword::Ward(WardCost::Mana(_)))));
}

/// Niv-Mizzet draws for noncombat damage its controller's sources deal to an
/// opponent, and grants no maximum hand size.
#[test]
fn niv_mizzet_draws_on_noncombat_damage() {
    let mut g = two_player_game();
    let niv = g.add_card_to_battlefield(0, catalog::niv_mizzet_visionary());
    for _ in 0..5 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let hand0 = g.players[0].hand.len();
    let _ = niv;
    let evs = vec![GameEvent::DamageDealt {
        amount: 3,
        to_player: Some(1),
        to_card: None,
        from_controller: Some(0),
        combat: false,
    }];
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 3, "drew 3 for 3 noncombat damage");
    assert!(g.effective_max_hand_size(0).is_none(), "no maximum hand size");
}

/// Perforating Artist's Raid punisher only fires if you attacked; the opponent
/// loses 3 life when they can't/won't pay.
#[test]
fn perforating_artist_raid_punisher() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::perforating_artist());
    g.active_player_idx = 0;
    // Mark that player 0 attacked this turn (Raid). Empty the opponent's hand
    // and board so they can't shed via discard/sacrifice.
    g.players[0].attacked_this_turn = true;
    g.players[1].hand.clear();
    let l1 = g.players[1].life;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 3, "opponent lost 3 (no nonland/discard to shed)");
}

/// With a card in hand, the opponent sheds it to the Raid punisher instead of
/// losing life.
#[test]
fn perforating_artist_opponent_discards_to_dodge() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::perforating_artist());
    g.active_player_idx = 0;
    g.players[0].attacked_this_turn = true;
    g.players[1].hand.clear();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let l1 = g.players[1].life;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1, "opponent kept life by discarding");
    assert_eq!(g.players[1].graveyard.len(), 1, "opponent discarded a card");
}

/// Without an attack, the Raid trigger's intervening-if fails.
#[test]
fn perforating_artist_no_raid_no_trigger() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::perforating_artist());
    g.active_player_idx = 0;
    g.players[0].attacked_this_turn = false;
    let l1 = g.players[1].life;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1, "no attack → no punisher");
}

/// Kiora's ETB draws two and discards two.
#[test]
fn kiora_etb_loots_two() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let hand0 = g.players[0].hand.len();
    let kiora = g.add_card_to_battlefield(0, catalog::kiora_the_rising_tide());
    g.fire_self_etb_triggers(kiora, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0, "drew two, discarded two");
    assert_eq!(g.players[0].graveyard.len(), 2, "two cards discarded to graveyard");
}

/// Kiora's threshold attack trigger makes an 8/8 Scion once seven cards are in
/// the graveyard.
#[test]
fn kiora_threshold_makes_scion() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    for _ in 0..7 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); }
    let kiora = g.add_card_to_battlefield(0, catalog::kiora_the_rising_tide());
    g.clear_sickness(kiora);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let evs = g
        .declare_attackers(vec![Attack { attacker: kiora, target: AttackTarget::Player(1) }])
        .expect("Kiora attacks");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let scions = g.battlefield.iter().filter(|c| c.definition.name == "Scion of the Deep").count();
    assert_eq!(scions, 1, "8/8 Scion minted at threshold");
}

/// Lunar Insight draws once per distinct mana value among your nonland permanents.
#[test]
fn lunar_insight_draws_per_distinct_mv() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Distinct MVs among nonland permanents: Grizzly Bears (2) and Lightning
    // Bolt isn't a permanent — use two creatures of different MV.
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // MV 2
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // MV 2 (dup)
    g.add_card_to_battlefield(0, catalog::serra_angel()); // MV 5
    for _ in 0..5 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let li = g.add_card_to_hand(0, catalog::lunar_insight());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: li, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Lunar Insight");
    drain_stack(&mut g);
    // Two distinct MVs (2 and 5) → draw 2. Hand was hand0, minus the cast spell.
    assert_eq!(g.players[0].hand.len(), hand0 - 1 + 2, "drew 2 for 2 distinct MVs");
}

/// Soulstone Sanctuary animates into a 3/3 with all creature types (Changeling).
#[test]
fn soulstone_sanctuary_animates_all_types() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let land = g.add_card_to_battlefield(0, catalog::soulstone_sanctuary());
    g.players[0].mana_pool.add_colorless(4);
    // Activate the {4} animate ability (index 1).
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: vec![],
        x_value: None,
    })
    .expect("animate Soulstone Sanctuary");
    drain_stack(&mut g);
    let c = g.computed_permanent(land).expect("soulstone");
    assert!(c.card_types.contains(&crate::card::CardType::Creature), "now a creature");
    assert!(c.card_types.contains(&crate::card::CardType::Land), "still a land");
    assert_eq!((c.power, c.toughness), (3, 3));
    assert!(c.keywords.contains(&crate::card::Keyword::Changeling), "all creature types via Changeling");
}
