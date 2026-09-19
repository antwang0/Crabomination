//! CR 701.27f — an ability can't transform a permanent that has already
//! transformed since the ability went on the stack.
//!
//! "If an activated or triggered ability of a permanent that isn't a delayed
//! triggered ability of that permanent tries to transform it, the permanent
//! does so only if it hasn't transformed or converted since the ability was
//! put onto the stack. … if the permanent has already transformed or
//! converted, an instruction to do either is ignored."
//!
//! Without it, two copies of one ability in a batch flip the face back and
//! forth and the card ends where it started. `EventKind::PermanentSacrificed`
//! has fanned out since long before this file, so Daring Sleuth — "whenever
//! you sacrifice a Clue, transform this" — read *untransformed* off a
//! two-Clue sacrifice and transformed off a one- or three-Clue one.

use crabomination::catalog;
use crabomination::game::effects::clue_token;
use crabomination::game::*;

/// Two Clues sacrificed in one batch are two triggers, and the second one's
/// transform is ignored.
#[test]
fn cr_701_27f_a_second_transform_of_the_same_source_is_ignored() {
    let mut g = two_player_game();
    let sleuth = g.add_card_to_battlefield(0, catalog::daring_sleuth());
    let evs: Vec<GameEvent> = (0..2)
        .map(|_| {
            let id = g.add_token_to_battlefield(0, &clue_token());
            GameEvent::PermanentSacrificed { card_id: id, who: 0 }
        })
        .collect();
    g.dispatch_triggers_for_events(&evs);
    assert_eq!(g.stack.len(), 2, "CR 603.6 — one trigger per sacrificed Clue");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(sleuth).expect("still there").definition.name,
        "Bearer of Overwhelming Truths",
        "the second trigger's transform is ignored, not a flip back",
    );
}

/// The rule is about the ability's OWN permanent. A single trigger still
/// transforms it, so the guard cannot be reading "any transform at all".
#[test]
fn cr_701_27f_one_trigger_still_transforms() {
    let mut g = two_player_game();
    let sleuth = g.add_card_to_battlefield(0, catalog::daring_sleuth());
    let clue = g.add_token_to_battlefield(0, &clue_token());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentSacrificed { card_id: clue, who: 0 }]);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(sleuth).expect("still there").definition.name,
        "Bearer of Overwhelming Truths",
    );
}

/// Three is the count a toggle would get right by accident. Every copy after
/// the first is ignored, so the face is the same as it is for two.
#[test]
fn cr_701_27f_a_third_copy_is_ignored_too() {
    let mut g = two_player_game();
    let sleuth = g.add_card_to_battlefield(0, catalog::daring_sleuth());
    let evs: Vec<GameEvent> = (0..3)
        .map(|_| {
            let id = g.add_token_to_battlefield(0, &clue_token());
            GameEvent::PermanentSacrificed { card_id: id, who: 0 }
        })
        .collect();
    g.dispatch_triggers_for_events(&evs);
    assert_eq!(g.stack.len(), 3);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(sleuth).expect("still there").definition.name,
        "Bearer of Overwhelming Truths",
    );
}

/// The third shipped card on the shape, and the one whose existing test passed
/// by accident. Legion's Landing reads "whenever you attack with three or more
/// creatures, transform this"; `EventKind::Attacks` fans out, so three
/// attackers were three transforms and landed on the back face by parity.
/// Four attackers put it back on the front until CR 701.27f.
#[test]
fn cr_701_27f_legions_landing_survives_an_even_attack() {
    use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let ll = g.add_card_to_battlefield(0, catalog::legions_landing());
    let attackers: Vec<CardId> = (0..4)
        .map(|_| {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.clear_sickness(c);
            c
        })
        .collect();
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass to declare attackers");
    }
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&c| Attack { attacker: c, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack with four");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(ll).expect("still there").definition.name,
        "Adanto, the First Fort",
        "four triggers, one transform",
    );
}

// ── Probing Telepathy (Aboleth Spawn): copying an entering creature's own
//    ETB-caused trigger ──────────────────────────────────────────────────────

/// A 1/1 with "Whenever a creature you control enters, draw a card" — a
/// trigger the creature's own entry causes, dispatched through the event
/// pipeline rather than the self-ETB path.
fn creature_enters_draw_watcher() -> crabomination::card::CardDefinition {
    use crabomination::card::{CardDefinition, CardType, SelectionRequirement, TriggeredAbility};
    use crabomination::effect::{Effect, EventKind, EventScope, EventSpec, Predicate, Selector, Value};
    CardDefinition {
        name: "Test Watcher",
        card_types: vec![CardType::Creature],
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature,
                },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// An opponent's Probing Telepathy copies the entrant's *own* ETB-caused
/// trigger (the copy is theirs: they draw), and never another permanent's
/// trigger off the same entry (Soul Warden's lifegain stays single).
#[test]
fn probing_telepathy_copies_only_the_entrants_own_etb_caused_trigger() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{GameAction, TurnStep};
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    stock_libraries(&mut g, 10);
    g.add_card_to_battlefield(1, catalog::aboleth_spawn());
    g.add_card_to_battlefield(0, catalog::soul_warden());
    let watcher = g.add_card_to_hand(0, creature_enters_draw_watcher());
    let (h0, h1) = (g.players[0].hand.len(), g.players[1].hand.len());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastSpell {
        card_id: watcher,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the free Watcher");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h0, "the Watcher's own draw replaces it in hand");
    assert_eq!(g.players[1].hand.len(), h1 + 1, "the copy draws for the Aboleth's controller");
    assert_eq!(g.players[0].life, 21, "Soul Warden triggers once");
    assert_eq!(g.players[1].life, 20, "another permanent's trigger isn't copied");
}
