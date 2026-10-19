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

fn run(g: &mut GameState, effect: &crabomination::effect::Effect) {
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn lore(g: &mut GameState, saga: CardId) {
    use crabomination::card::CounterType;
    use crabomination::effect::{Effect, Selector, Value};
    run(
        g,
        &Effect::AddCounter {
            what: Selector::ExactObjects(vec![saga]),
            kind: CounterType::Lore,
            amount: Value::ONE,
        },
    );
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [
        Color::White,
        Color::Blue,
        Color::Black,
        Color::Red,
        Color::Green,
    ] {
        g.players[seat].mana_pool.add(c, 6);
    }
}

fn cast_targets(g: &mut GameState, card_id: CardId, targets: &[Target]) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn in_exile(g: &GameState, id: CardId) -> bool {
    g.exile.iter().any(|c| c.id == id)
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| {
        c.counter_count(crabomination::card::CounterType::PlusOnePlusOne)
    })
}

fn combat(g: &mut GameState, attacks: Vec<Attack>, blocks: Vec<(CardId, CardId)>) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(attacks))
        .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    let defender = g.attacking.first().map_or(1, |a| match a.target {
        AttackTarget::Player(p) => p,
        _ => 1,
    });
    g.priority.player_with_priority = defender;
    g.perform_action(GameAction::DeclareBlockers(blocks))
        .expect("block");
    drain_stack(g);
}

fn damage(g: &mut GameState) {
    g.step = TurnStep::CombatDamage;
    let evs = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

/// Fight to the Death destroys the blocker and the creature it blocked; the
/// unblocked attacker and the creature that stayed home survive.
#[test]
fn fight_to_the_death_destroys_only_the_blocks() {
    let mut g = pod(2);
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let free = ready(&mut g, 0, catalog::grizzly_bears());
    let wall = ready(&mut g, 1, catalog::grizzly_bears());
    let home = ready(&mut g, 1, catalog::grizzly_bears());
    let at = |attacker| Attack {
        attacker,
        target: AttackTarget::Player(1),
    };
    combat(&mut g, vec![at(a), at(free)], vec![(wall, a)]);
    let spell = g.add_card_to_hand(0, catalog::fight_to_the_death());
    cast_targets(&mut g, spell, &[]);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(wall).is_none());
    assert!(g.battlefield_find(free).is_some() && g.battlefield_find(home).is_some());
}

/// The Super Hero Civil War: chapter I takes up to two creatures with total
/// mana value 6 or less while the Saga stays; II pumps; after III's fight the
/// Saga is sacrificed and the creatures go home.
#[test]
fn super_hero_civil_war_borrows_while_the_saga_remains() {
    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::hill_giant());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::hill_giant());
    let saga = g.add_card_to_hand(0, catalog::the_super_hero_civil_war());
    cast_targets(&mut g, saga, &[Target::Permanent(a), Target::Permanent(b)]);
    assert_eq!(g.battlefield_find(a).unwrap().controller, 0);
    assert_eq!(g.battlefield_find(b).unwrap().controller, 0);
    lore(&mut g, saga);
    assert_eq!(
        g.computed_permanent(mine).unwrap().power,
        4,
        "+1/+1 from chapter II"
    );
    lore(&mut g, saga);
    g.check_state_based_actions();
    assert!(g.battlefield_find(saga).is_none());
    for id in [a, b] {
        assert!(
            g.battlefield_find(id).is_none_or(|c| c.controller == 1),
            "back home once the Saga left"
        );
    }
}

/// Helm of Chatzuk grants banding until end of turn.
#[test]
fn helm_of_chatzuk_grants_banding() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let helm = ready(&mut g, 0, catalog::helm_of_chatzuk());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: helm,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("helm");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(bear)
            .unwrap()
            .keywords()
            .contains(&Keyword::Banding)
    );
}

/// CR 509.1c — Revenge of the Hunted: +6/+6, trample, and every creature able
/// to block it must; declaring no blocks is illegal.
#[test]
fn revenge_of_the_hunted_lures_every_blocker() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let b1 = ready(&mut g, 1, catalog::grizzly_bears());
    let b2 = ready(&mut g, 1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::revenge_of_the_hunted());
    cast_targets(&mut g, spell, &[Target::Permanent(bear)]);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 8);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(
        g.clone()
            .perform_action(GameAction::DeclareBlockers(vec![(b1, bear)]))
            .is_err(),
        "both must block"
    );
    g.perform_action(GameAction::DeclareBlockers(vec![(b1, bear), (b2, bear)]))
        .expect("both block");
}

/// Crossover Collaboration exiles two to play until the end of your next
/// turn; teamwork (tapping power 2) also makes a Treasure.
#[test]
fn crossover_collaboration_treasure_needs_teamwork() {
    for team in [false, true] {
        let mut g = pod(2);
        let bear = ready(&mut g, 0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::crossover_collaboration());
        flood(&mut g, 0);
        g.priority.player_with_priority = 0;
        let (card_id, target, additional_targets, mode, x_value) =
            (spell, None, vec![], None, None);
        g.perform_action(if team {
            GameAction::CastSpellKicked {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            }
        } else {
            GameAction::CastSpell {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            }
        })
        .expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.exile.len(), 2, "two exiled to play");
        assert_eq!(named(&g, 0, "Treasure").len(), usize::from(team));
        assert_eq!(g.battlefield_find(bear).unwrap().tapped, team);
    }
}

/// Storm of Steel splits into 2 damage to each of two targets (CR 115.3).
#[test]
fn storm_of_steel_hits_two_targets() {
    let mut g = pod(3);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::storm_of_steel());
    cast_targets(&mut g, spell, &[Target::Permanent(bear), Target::Player(2)]);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[2].life, 18);
}

/// CR 702.190 — Karai sneaked in returns the creature card to the
/// battlefield; cast normally and attacking, it goes to hand.
#[test]
fn karai_reanimates_only_when_sneaked() {
    for sneaked in [true, false] {
        let mut g = pod(2);
        let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
        let karai = if sneaked {
            let bear = ready(&mut g, 0, catalog::grizzly_bears());
            let karai = g.add_card_to_hand(0, catalog::karai_future_of_the_foot());
            g.attacking = vec![Attack {
                attacker: bear,
                target: AttackTarget::Player(1),
            }];
            g.step = TurnStep::DeclareBlockers;
            flood(&mut g, 0);
            g.perform_action(GameAction::CastSpellAlternative {
                card_id: karai,
                pitch_card: None,
                target: None,
                additional_targets: vec![],
                mode: None,
                x_value: None,
            })
            .expect("sneak");
            drain_stack(&mut g);
            karai
        } else {
            let karai = ready(&mut g, 0, catalog::karai_future_of_the_foot());
            combat(
                &mut g,
                vec![Attack {
                    attacker: karai,
                    target: AttackTarget::Player(1),
                }],
                vec![],
            );
            karai
        };
        assert!(g.attacking.iter().any(|a| a.attacker == karai));
        damage(&mut g);
        if sneaked {
            assert!(
                g.battlefield_find(giant).is_some(),
                "sneaked: onto the battlefield"
            );
        } else {
            assert!(
                g.players[0].hand.iter().any(|c| c.id == giant),
                "cast: to hand"
            );
        }
    }
}

/// Turncoat Kunoichi cast normally holds its prisoner until it leaves; a
/// sneaked one exiles for good.
#[test]
fn turncoat_kunoichi_exile_is_permanent_when_sneaked() {
    for sneaked in [false, true] {
        let mut g = pod(2);
        let victim = g.add_card_to_battlefield(1, catalog::hill_giant());
        let k = g.add_card_to_hand(0, catalog::turncoat_kunoichi());
        if sneaked {
            let bear = ready(&mut g, 0, catalog::grizzly_bears());
            g.attacking = vec![Attack {
                attacker: bear,
                target: AttackTarget::Player(1),
            }];
            g.step = TurnStep::DeclareBlockers;
            flood(&mut g, 0);
            g.perform_action(GameAction::CastSpellAlternative {
                card_id: k,
                pitch_card: None,
                target: Some(Target::Permanent(victim)),
                additional_targets: vec![],
                mode: None,
                x_value: None,
            })
            .expect("sneak");
            drain_stack(&mut g);
        } else {
            cast_targets(&mut g, k, &[Target::Permanent(victim)]);
        }
        assert!(in_exile(&g, victim));
        kill(&mut g, k);
        assert_eq!(g.battlefield_find(victim).is_some(), !sneaked);
    }
}

// ── Marvel Villains (`decks::cmdr_01dj_c`) ──────────────────────────────────

fn power(g: &GameState, id: CardId) -> i32 {
    g.computed_permanent(id).unwrap().power
}

/// The Masters of Evil pumps other Villains +2/+1 and, from hand, discards
/// itself to tutor a Plan.
#[test]
fn masters_of_evil_pumps_villains_and_finds_a_plan() {
    let mut g = pod(2);
    let crossbones = ready(&mut g, 0, catalog::crossbones_malicious_mercenary());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let masters = ready(&mut g, 0, catalog::the_masters_of_evil());
    assert_eq!(power(&g, crossbones), 5);
    assert_eq!(power(&g, bear), 2);
    assert_eq!(power(&g, masters), 5);
    let plan = g.add_card_to_library(0, catalog::villainous_syndication());
    let second = g.add_card_to_hand(0, catalog::the_masters_of_evil());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: second,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("discard to tutor");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == plan));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == second));
}

/// Crossbones: two Villains entering in one turn grow him once and burn each
/// opponent once (CR 603.2h "only once each turn").
#[test]
fn crossbones_triggers_once_each_turn() {
    let mut g = pod(3);
    let cb = ready(&mut g, 0, catalog::crossbones_malicious_mercenary());
    for _ in 0..2 {
        let v = g.add_card_to_hand(0, catalog::red_room_recruit());
        cast_targets(&mut g, v, &[]);
    }
    assert_eq!(plus(&g, cb), 1);
    assert_eq!((g.players[1].life, g.players[2].life), (18, 18));
}

/// Citizen V's lifelink damage counts as a lifegain event: each Villain gets a
/// counter, a non-Villain doesn't.
#[test]
fn citizen_v_counters_villains_on_lifegain() {
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = pod(2);
    let zemo = ready(&mut g, 0, catalog::citizen_v_helmut_zemo());
    let rec = ready(&mut g, 0, catalog::red_room_recruit());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    run(
        &mut g,
        &Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(3),
        },
    );
    assert_eq!((plus(&g, zemo), plus(&g, rec), plus(&g, bear)), (1, 1, 0));
}

/// Crimson Cowl makes one Villain per player its nontoken Villains attack:
/// two defenders, two tokens; a lone token attacker makes none.
#[test]
fn crimson_cowl_makes_a_villain_per_attacked_player() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::crimson_cowl_master_of_evil());
    let v1 = ready(&mut g, 0, catalog::red_room_recruit());
    let v2 = ready(&mut g, 0, catalog::hydra_troopers());
    let v3 = ready(&mut g, 0, catalog::red_room_recruit());
    let before = named(&g, 0, "Villain").len();
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack {
            attacker: v1,
            target: AttackTarget::Player(1),
        },
        Attack {
            attacker: v2,
            target: AttackTarget::Player(1),
        },
        Attack {
            attacker: v3,
            target: AttackTarget::Player(2),
        },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Villain").len(), before + 2);
}

/// Thunderbolts Conspiracy brings a dead Villain back with a finality counter
/// as a Hero; the second death exiles it (CR 122.1h). A non-Villain stays dead.
#[test]
fn thunderbolts_conspiracy_returns_villains_once() {
    use crabomination::card::{CounterType, CreatureType};
    let mut g = pod(2);
    ready(&mut g, 0, catalog::thunderbolts_conspiracy());
    let v = ready(&mut g, 0, catalog::red_room_recruit());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    kill(&mut g, v);
    let back = g.battlefield_find(v).expect("returned");
    assert_eq!(back.counter_count(CounterType::Finality), 1);
    assert!(
        g.computed_permanent(v)
            .unwrap()
            .subtypes()
            .creature_types
            .contains(&CreatureType::Hero)
    );
    kill(&mut g, v);
    assert!(in_exile(&g, v), "finality exiles it");
    kill(&mut g, bear);
    assert!(g.battlefield_find(bear).is_none());
}

/// Scorpion connives a creature of yours at your end step only after a death
/// this turn.
#[test]
fn scorpion_connives_after_a_death() {
    for death in [false, true] {
        let mut g = pod(2);
        let sc = ready(&mut g, 0, catalog::scorpion_seething_striker());
        g.players[0].hand.clear();
        g.add_card_to_hand(0, catalog::lightning_bolt());
        if death {
            let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
            kill(&mut g, bear);
        }
        let hand = g.players[0].hand.len();
        g.step = TurnStep::PostCombatMain;
        while g.step != TurnStep::End && g.game_over.is_none() {
            g.perform_action(GameAction::PassPriority).ok();
        }
        drain_stack(&mut g);
        assert_eq!(
            g.players[0].graveyard.len() > usize::from(death),
            death,
            "connive discards a card"
        );
        assert_eq!(g.players[0].hand.len(), hand);
        let _ = sc;
    }
}

/// HYDRA Troopers: two creature cards in your graveyard make a tapped Villain;
/// fewer mill two.
#[test]
fn hydra_troopers_villain_or_mill() {
    for stocked in [false, true] {
        let mut g = pod(2);
        if stocked {
            g.add_card_to_graveyard(0, catalog::grizzly_bears());
            g.add_card_to_graveyard(0, catalog::hill_giant());
        }
        let t = g.add_card_to_hand(0, catalog::hydra_troopers());
        let lib = g.players[0].library.len();
        cast_targets(&mut g, t, &[]);
        let tokens = named(&g, 0, "Villain");
        assert_eq!(tokens.len(), usize::from(stocked));
        if stocked {
            assert!(g.battlefield_find(tokens[0]).unwrap().tapped);
        } else {
            assert_eq!(g.players[0].library.len(), lib - 2);
        }
    }
}

/// Venom grows on any other creature's death and draws only for a Villain.
#[test]
fn venom_grows_and_draws_on_villains() {
    let mut g = pod(3);
    let venom = ready(&mut g, 0, catalog::venom_eddie_brock());
    let bear = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let v = g.add_card_to_battlefield(1, catalog::red_room_recruit());
    let hand = g.players[0].hand.len();
    kill(&mut g, bear);
    assert_eq!((plus(&g, venom), g.players[0].hand.len()), (1, hand));
    kill(&mut g, v);
    assert_eq!((plus(&g, venom), g.players[0].hand.len()), (2, hand + 1));
}

/// Criminal Enterprise makes a Villain and drains each opponent when one dies.
#[test]
fn criminal_enterprise_drains_on_villain_deaths() {
    let mut g = pod(3);
    let ce = g.add_card_to_hand(0, catalog::criminal_enterprise());
    cast_targets(&mut g, ce, &[]);
    let token = named(&g, 0, "Villain")[0];
    kill(&mut g, token);
    assert_eq!(
        (g.players[0].life, g.players[1].life, g.players[2].life),
        (21, 19, 19)
    );
}

/// Mechanical Mobster exiles a graveyard card and connives a creature of
/// yours.
#[test]
fn mechanical_mobster_exiles_and_connives() {
    let mut g = pod(2);
    let theirs = g.add_card_to_graveyard(1, catalog::hill_giant());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].hand.clear();
    g.add_card_to_hand(0, catalog::lightning_bolt());
    let m = g.add_card_to_hand(0, catalog::mechanical_mobster());
    cast_targets(&mut g, m, &[]);
    let _ = bear;
    assert!(in_exile(&g, theirs), "the graveyard card is exiled");
    assert_eq!(g.players[0].hand.len(), 1, "drew one, discarded one");
}

/// Valentina makes other Villains Heroes and connives a Hero you control.
#[test]
fn valentina_turns_villains_into_heroes() {
    use crabomination::card::CreatureType;
    let mut g = pod(2);
    let val = ready(&mut g, 0, catalog::valentina_allegra_de_fontaine());
    let v = ready(&mut g, 0, catalog::red_room_recruit());
    assert!(
        g.computed_permanent(v)
            .unwrap()
            .subtypes()
            .creature_types
            .contains(&CreatureType::Hero)
    );
    assert!(
        !g.computed_permanent(val)
            .unwrap()
            .subtypes()
            .creature_types
            .contains(&CreatureType::Hero),
        "other Villains only"
    );
    g.players[0].hand.clear();
    g.add_card_to_hand(0, catalog::lightning_bolt());
    g.perform_action(GameAction::ActivateAbility {
        card_id: val,
        ability_index: 0,
        target: Some(Target::Permanent(v)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("connive");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// MACH-1 surveils as it enters and once a turn on lifegain — but not on a
/// lifegain the turn it entered (the once-a-turn trigger already fired).
#[test]
fn mach_1_surveils_once_each_turn() {
    use crabomination::effect::{Effect, Selector, Value};
    let gain = Effect::GainLife {
        who: Selector::You,
        amount: Value::ONE,
    };
    let surveils = |g: &GameState| g.stack.len();
    let mut g = pod(2);
    let mach = g.add_card_to_hand(0, catalog::mach_1_swooping_scoundrel());
    cast_targets(&mut g, mach, &[]);
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(&gain, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    assert_eq!(surveils(&g), 0, "spent on entering");
    g.battlefield_find_mut(mach).unwrap().entered_turn = None;
    for want in [1, 1] {
        let evs = g.resolve_effect(&gain, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        assert_eq!(surveils(&g), want, "once each turn");
    }
}

/// Villainous Syndication: four Villain taps mill four and fill the plan; the
/// fourth counter sacrifices it and returns a creature card.
#[test]
fn villainous_syndication_completes_its_plan() {
    let mut g = pod(2);
    let plan = ready(&mut g, 0, catalog::villainous_syndication());
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    for _ in 0..4 {
        ready(&mut g, 0, catalog::red_room_recruit());
    }
    for _ in 0..4 {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: plan,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("tap a Villain");
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(plan).is_none(), "sacrificed at four");
    assert!(
        g.battlefield_find(giant).is_some(),
        "the creature card came back"
    );
}

/// Decoy Ploy returns a Villain and a Hero card with both modes.
#[test]
fn decoy_ploy_returns_a_villain_and_a_hero() {
    let mut g = pod(2);
    let v = g.add_card_to_graveyard(0, catalog::red_room_recruit());
    let h = g.add_card_to_graveyard(0, catalog::black_widow_natasha_romanoff());
    let spell = g.add_card_to_hand(0, catalog::decoy_ploy());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: spell,
        spree_modes: vec![0, 1],
        target: Some(Target::Permanent(v)),
        additional_targets: vec![Target::Permanent(h)],
        x_value: None,
    })
    .expect("both modes");
    drain_stack(&mut g);
    for id in [v, h] {
        assert!(g.players[0].hand.iter().any(|c| c.id == id));
    }
}

/// Lethal Protection destroys a creature and may return one from your
/// graveyard.
#[test]
fn lethal_protection_kills_and_recovers() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::lethal_protection());
    cast_targets(
        &mut g,
        spell,
        &[Target::Permanent(bear), Target::Permanent(giant)],
    );
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.players[0].hand.iter().any(|c| c.id == giant));
}

/// HYDRA Infiltration: two discards on entry; a lone attacker drains 1.
#[test]
fn hydra_infiltration_discards_and_drains_alone() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::forest());
    }
    let hand = g.players[1].hand.len();
    let e = g.add_card_to_hand(0, catalog::hydra_infiltration());
    cast_targets(&mut g, e, &[Target::Player(1)]);
    assert_eq!(g.players[1].hand.len(), hand - 2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!((g.players[0].life, g.players[1].life), (21, 19));
}

/// Avengers: Under Siege — two Villains; 2 damage to non-Villains and each
/// opponent; a Treasure per Villain.
#[test]
fn avengers_under_siege_chapters() {
    let mut g = pod(3);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let saga = g.add_card_to_hand(0, catalog::avengers_under_siege());
    cast_targets(&mut g, saga, &[]);
    assert_eq!(named(&g, 0, "Villain").len(), 2);
    lore(&mut g, saga);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(named(&g, 0, "Villain").len(), 2, "Villains are spared");
    assert_eq!((g.players[1].life, g.players[2].life), (18, 18));
    lore(&mut g, saga);
    assert_eq!(named(&g, 0, "Treasure").len(), 2);
}

/// Mysterio makes an Illusion per nontoken Villain (himself included) and
/// exiles exactly those when he leaves.
#[test]
fn mysterio_illusions_leave_with_him() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::red_room_recruit());
    let m = g.add_card_to_hand(0, catalog::mysterio_master_of_illusion());
    cast_targets(&mut g, m, &[]);
    assert_eq!(named(&g, 0, "Illusion Villain").len(), 2);
    kill(&mut g, m);
    assert_eq!(named(&g, 0, "Illusion Villain").len(), 0);
}

/// Madame Masque connives as she enters (her first draw of the turn); the
/// next draw is the turn's second and makes a Villain.
#[test]
fn madame_masque_rewards_the_second_draw() {
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = pod(2);
    let mm = g.add_card_to_hand(0, catalog::madame_masque());
    cast_targets(&mut g, mm, &[]);
    assert_eq!(
        named(&g, 0, "Villain").len(),
        0,
        "the connive draw was the first"
    );
    run(
        &mut g,
        &Effect::Draw {
            who: Selector::You,
            amount: Value::ONE,
        },
    );
    assert_eq!(named(&g, 0, "Villain").len(), 1);
}

/// Doc Ock's Henchmen connive whenever they attack.
#[test]
fn doc_ocks_henchmen_connive_on_attack() {
    let mut g = pod(2);
    let h = ready(&mut g, 0, catalog::doc_ocks_henchmen());
    g.players[0].hand.clear();
    g.add_card_to_hand(0, catalog::hill_giant());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: h,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[0].graveyard.len(), 1);
}
