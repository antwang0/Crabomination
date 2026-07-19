//! Functionality tests for `catalog::sets::decks::recent268`.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::{drain_stack, two_player_game, CardId, GameState};
use crabomination::mana::Color;

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

/// Aether Channeler's draw mode draws a card.
#[test]
fn aether_channeler_modal_draw() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let ac = g.add_card_to_hand(0, catalog::aether_channeler());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    // Choose mode 2 (draw a card).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(2)]));
    cast(&mut g, ac, None);
    assert!(
        g.players[0].hand.iter().any(|c| c.definition.name == "Forest"),
        "drew the library card via modal ETB"
    );
}

/// Aggressive Sabotage discards two, and burns for 3 when kicked.
#[test]
fn aggressive_sabotage_kicked_burns() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::forest());
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_hand(1, catalog::mountain());
    let effect = catalog::aggressive_sabotage().effect;
    let mut ctx = EffectContext::for_spell(0, Some(Target::Player(1)), 0, 0);
    ctx.kicked = true;
    let life = g.players[1].life;
    let hand = g.players[1].hand.len();
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.players[1].hand.len(), hand - 2, "discarded two");
    assert_eq!(g.players[1].life, life - 3, "kicked burn for 3");
}

/// Argivian Phalanx's affinity for creatures cuts its cost.
#[test]
fn argivian_phalanx_affinity() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ph = g.add_card_to_hand(0, catalog::argivian_phalanx());
    // {5}{W} minus {2} for two creatures = {3}{W}.
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, ph, None);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Argivian Phalanx"),
        "cast for the affinity-reduced cost"
    );
}

/// Artillery Blast scales with domain and only hits tapped creatures.
#[test]
fn artillery_blast_domain_damage() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::island());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.battlefield_find_mut(victim).unwrap().tapped = true;
    let effect = catalog::artillery_blast().effect;
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(victim)), 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    // 1 + 2 basic land types = 3 damage → the 2/2 dies.
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "took 3, died");
}

/// Automatic Librarian scries on ETB.
#[test]
fn automatic_librarian_scries() {
    let mut g = two_player_game();
    let lib = g.add_card_to_hand(0, catalog::automatic_librarian());
    g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
        kept_top: vec![],
        bottom: vec![],
    }]));
    cast(&mut g, lib, None);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Automatic Librarian"));
}

/// Antagonize pumps +4/+3.
#[test]
fn antagonize_pumps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let effect = catalog::antagonize().effect;
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 5));
}

/// Attended Socialite grows when another creature enters.
#[test]
fn attended_socialite_alliance() {
    let mut g = two_player_game();
    let soc = g.add_card_to_battlefield(0, catalog::attended_socialite());
    let other = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, other, None);
    let cp = g.computed_permanent(soc).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 2), "socialite grew +1/+1");
}

/// Backup Agent puts a +1/+1 counter on a creature on ETB.
#[test]
fn backup_agent_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let agent = g.add_card_to_hand(0, catalog::backup_agent());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, agent, Some(Target::Permanent(bear)));
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

/// Armor of Shadows grants indestructible and +1/+0.
#[test]
fn armor_of_shadows_grants_indestructible() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let effect = catalog::armor_of_shadows().effect;
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3, "+1/+0");
    assert!(cp.keywords.contains(&Keyword::Indestructible));
}

/// Arms of Hadar shrinks all of a player's creatures.
#[test]
fn arms_of_hadar_mass_shrink() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let effect = catalog::arms_of_hadar().effect;
    let ctx = EffectContext::for_spell(0, Some(Target::Player(1)), 0, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none(), "shrank to 0/0 and died");
    assert!(g.battlefield_find(b).is_none(), "shrank to 0/0 and died");
}

/// A Little Chat digs two deep for a card.
#[test]
fn a_little_chat_digs() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    let effect = catalog::a_little_chat().effect;
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let hand = g.players[0].hand.len();
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.players[0].hand.len(), hand + 1, "one card to hand");
}
