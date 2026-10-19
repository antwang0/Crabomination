//! Commander: the 2026-10-19 (`01DJHkWk`) one-deck closers
//! (`decks::cmdr_01dj_a`).

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

/// Ghastlord of Fugue hits the player it attacked (seat 2 of three): that
/// player's hand is revealed and the chosen card is exiled; the bystander's
/// hand is untouched.
#[test]
fn ghastlord_of_fugue_exiles_from_the_damaged_players_hand() {
    let mut g = pod(3);
    let ghast = ready(&mut g, 0, catalog::ghastlord_of_fugue());
    g.players[1].hand.clear();
    g.players[2].hand.clear();
    let kept = g.add_card_to_hand(1, catalog::lightning_bolt());
    let gone = g.add_card_to_hand(2, catalog::lightning_bolt());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ghast,
        target: AttackTarget::Player(2),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::CombatDamage;
    let evs = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[2].life, 16);
    assert!(
        g.exile.iter().any(|c| c.id == gone),
        "the damaged player's card is exiled"
    );
    assert!(
        g.players[1].hand.iter().any(|c| c.id == kept),
        "the other opponent keeps theirs"
    );
}

fn pass_until(g: &mut GameState, seat: usize) {
    for _ in 0..600 {
        if g.active_player_idx == seat && g.step == TurnStep::PreCombatMain {
            return;
        }
        let _ = g.perform_action(GameAction::PassPriority);
    }
    panic!("never reached seat {seat}'s main phase");
}

/// Power of Persuasion's d20 table (CR 706.3a): 1-9 bounces, 10-19 puts the
/// creature into its owner's library, 20 steals it until the end of your next
/// turn (CR 611.2b) — then it goes home.
#[test]
fn power_of_persuasion_rolls_bounce_library_or_steal() {
    for (roll, want) in [(5u8, "hand"), (15, "library"), (20, "control")] {
        let mut g = pod(3);
        let bear = g.add_card_to_battlefield(2, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Blue, 3);
        let spell = g.add_card_to_hand(0, catalog::power_of_persuasion());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(roll)]));
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        match want {
            "hand" => assert!(g.players[2].hand.iter().any(|c| c.id == bear)),
            "library" => assert!(g.players[2].library.iter().any(|c| c.id == bear)),
            _ => {
                assert_eq!(g.battlefield_find(bear).map(|c| c.controller), Some(0));
                pass_until(&mut g, 1);
                pass_until(&mut g, 0);
                assert_eq!(
                    g.battlefield_find(bear).map(|c| c.controller),
                    Some(0),
                    "still ours on our next turn"
                );
                pass_until(&mut g, 1);
                assert_eq!(
                    g.battlefield_find(bear).map(|c| c.controller),
                    Some(2),
                    "home once that turn ends"
                );
            }
        }
    }
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.name == name)
        .map(|c| c.id)
        .collect()
}

fn cast_mode(g: &mut GameState, card_id: CardId, target: Option<Target>, mode: Option<usize>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id,
        target,
        additional_targets: vec![],
        mode,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

/// Stingerquill Annex enters tapped unless you control a planeswalker.
#[test]
fn stingerquill_annex_enters_untapped_with_a_planeswalker() {
    for walker in [false, true] {
        let mut g = pod(2);
        if walker {
            g.add_card_to_battlefield(0, catalog::tibalt_rakish_instigator());
        }
        let land = g.add_card_to_hand(0, catalog::stingerquill_annex());
        g.perform_action(GameAction::PlayLand(land)).expect("land");
        assert_eq!(g.battlefield_find(land).unwrap().tapped, !walker);
    }
}

/// Devoted Grafkeeper: casting Overwhelmed Archivist from the graveyard by
/// disturb (CR 702.146) is a spell cast from your graveyard, so it taps an
/// opponent's creature; the Archive Haunt it becomes loots on attack and is
/// exiled when it would die.
#[test]
fn devoted_grafkeeper_taps_when_a_disturb_spell_is_cast() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::devoted_grafkeeper());
    let bear = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let arch = g.add_card_to_graveyard(0, catalog::overwhelmed_archivist());
    g.players[0].mana_pool.add(Color::Blue, 4);
    g.perform_action(GameAction::CastDisturb {
        card_id: arch,
        target: None,
        additional_targets: vec![],
    })
    .expect("disturb");
    drain_stack(&mut g);
    assert!(
        g.battlefield_find(bear).unwrap().tapped,
        "the graveyard cast tapped the opponent's creature"
    );
    let haunt = g.battlefield_find(arch).expect("Archive Haunt in play");
    assert_eq!(haunt.definition.name, "Archive Haunt");
    g.remove_to_graveyard_with_triggers(arch);
    drain_stack(&mut g);
    assert!(
        g.exile.iter().any(|c| c.id == arch),
        "a disturbed card is exiled instead"
    );
}

/// CR 508.1d — Alluring Siren makes an opponent's creature attack *you*: in a
/// three-player game its controller can't send it at the third seat.
#[test]
fn alluring_siren_forces_an_attack_on_its_controller() {
    let mut g = pod(3);
    let siren = ready(&mut g, 0, catalog::alluring_siren());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: siren,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("siren");
    drain_stack(&mut g);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    let at = |seat| {
        GameAction::DeclareAttackers(vec![Attack {
            attacker: bear,
            target: AttackTarget::Player(seat),
        }])
    };
    assert!(
        g.clone()
            .perform_action(GameAction::DeclareAttackers(vec![]))
            .is_err(),
        "it must attack"
    );
    assert!(
        g.clone().perform_action(at(2)).is_err(),
        "and only the Siren's controller"
    );
    g.perform_action(at(0))
        .expect("attacks the Siren's controller");
}

/// Courtly Provocateur's first ability makes a creature attack this turn if
/// able (CR 508.1d); its second makes one block if able (CR 509.1c).
#[test]
fn courtly_provocateur_forces_attack_or_block() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let prov = ready(&mut g, 0, catalog::courtly_provocateur());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    for (index, keyword) in [(0, Keyword::MustAttack), (1, Keyword::MustBlock)] {
        let mut g = g.clone();
        g.perform_action(GameAction::ActivateAbility {
            card_id: prov,
            ability_index: index,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("provoke");
        drain_stack(&mut g);
        assert!(
            g.computed_permanent(bear)
                .unwrap()
                .keywords()
                .contains(&keyword)
        );
    }
}

/// Destroy `id` through a resolving spell, so every "dies" trigger sees it.
fn kill(g: &mut GameState, id: CardId) {
    g.players[0].mana_pool.add(Color::Black, 3);
    let spell = g.add_card_to_hand(0, catalog::murder());
    cast_mode(g, spell, Some(Target::Permanent(id)), None);
}

/// Minotaur, Roxxon CEO makes a Villain for itself or any nontoken creature
/// dying, an opponent's included — and nothing for a token.
#[test]
fn minotaur_roxxon_ceo_makes_villains_on_nontoken_deaths() {
    let mut g = pod(3);
    let mino = g.add_card_to_battlefield(0, catalog::minotaur_roxxon_ceo());
    let bear = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    kill(&mut g, bear);
    assert_eq!(named(&g, 0, "Villain").len(), 1);
    let villain = named(&g, 0, "Villain")[0];
    kill(&mut g, villain);
    assert_eq!(
        named(&g, 0, "Villain").len(),
        0,
        "a token dying makes nothing"
    );
    kill(&mut g, mino);
    assert_eq!(named(&g, 0, "Villain").len(), 1, "its own death counts");
}

/// Vraska's Final Mercy: either mode costs 2 life; the first destroys a
/// creature or planeswalker, the second empowers Jace 6 (CR 306.5b).
#[test]
fn vraskas_final_mercy_destroys_or_empowers() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 4);
    let a = g.add_card_to_hand(0, catalog::vraskas_final_mercy());
    cast_mode(&mut g, a, Some(Target::Permanent(bear)), Some(0));
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].life, 18);
    let b = g.add_card_to_hand(0, catalog::vraskas_final_mercy());
    cast_mode(&mut g, b, None, Some(1));
    let jace = named(&g, 0, "Jace");
    assert_eq!(jace.len(), 1);
    assert_eq!(
        g.battlefield_find(jace[0])
            .unwrap()
            .counter_count(crabomination::card::CounterType::Loyalty),
        6
    );
    assert_eq!(g.players[0].life, 16);
}
