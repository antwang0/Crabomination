//! Functionality tests for `catalog::sets::decks::recent113` — the
//! even-mana-value lock (Void Winnower), first-creature-spell cost reduction
//! (Conduit of Ruin), and Modern Horizons staples.

use crate::catalog;
use crate::game::types::{Attack, AttackTarget, Target, TurnStep};
use crate::game::*;
use crate::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    let mut guard = 0;
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
        guard += 1;
        assert!(guard < 60, "advance_to overran");
    }
}

/// CR 601.3e — Void Winnower stops an opponent's even-mana-value spell but not
/// an odd one (zero is even is covered by the block test).
#[test]
fn void_winnower_locks_even_mv_casts() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::void_winnower());
    // Grizzly Bears ({1}{G}) is MV 2 (even) — locked for the opponent.
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(), "even-mv spell locked");
    // {R} Lightning Bolt is MV 1 (odd) — castable.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("odd-mv spell allowed");
}

/// CR 509.1 — an opponent can't block with an even-mana-value creature under
/// Void Winnower, but an odd one blocks fine.
#[test]
fn void_winnower_locks_even_mv_blocks() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::void_winnower());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    // Grizzly Bears (MV 2, even) can't block; Savannah Lions (MV 1, odd) can.
    let even_blk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let odd_blk = g.add_card_to_battlefield(1, catalog::savannah_lions());
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(even_blk, attacker)])).is_err(),
        "even-mv creature can't block"
    );
    g.perform_action(GameAction::DeclareBlockers(vec![(odd_blk, attacker)]))
        .expect("odd-mv creature blocks");
}

/// Conduit of Ruin shaves {2} off the first creature spell each turn, not the
/// second.
#[test]
fn conduit_of_ruin_first_creature_spell_discount() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::conduit_of_ruin());
    // Grizzly Bears is {1}{G}; the discount makes it free (generic-only clamp).
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("first creature spell discounted to {G}");
    drain_stack(&mut g);
    // A second creature spell gets no discount.
    let bears2 = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bears2, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .is_err(),
        "second creature spell is full price"
    );
}

/// Price of Progress hits each player for twice their nonbasic-land count.
#[test]
fn price_of_progress_scales_per_player() {
    let mut g = two_player_game();
    // P0 controls two nonbasics, P1 one; basics don't count.
    g.add_card_to_battlefield(0, catalog::steam_vents());
    g.add_card_to_battlefield(0, catalog::steam_vents());
    g.add_card_to_battlefield(0, catalog::island());
    g.add_card_to_battlefield(1, catalog::steam_vents());
    let pop = g.add_card_to_hand(0, catalog::price_of_progress());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let l0 = g.players[0].life;
    let l1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: pop, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0 - 4, "2 nonbasics x2");
    assert_eq!(g.players[1].life, l1 - 2, "1 nonbasic x2");
}

/// Undead Augur draws and drains a life when a Zombie you control dies.
#[test]
fn undead_augur_zombie_death_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::undead_augur());
    let goblin = g.add_card_to_battlefield(0, catalog::putrid_goblin()); // a Zombie
    g.add_card_to_library(0, catalog::island());
    let life = g.players[0].life;
    g.battlefield_find_mut(goblin).unwrap().damage = 2; // lethal → CreatureDied
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "drew on the Zombie death");
    assert_eq!(g.players[0].life, life - 1, "lost 1 life");
}

/// King of the Pride buffs other Cats but not itself.
#[test]
fn king_of_the_pride_cat_anthem() {
    let mut g = two_player_game();
    let king = g.add_card_to_battlefield(0, catalog::king_of_the_pride());
    let lion = g.add_card_to_battlefield(0, catalog::savannah_lions()); // a Cat
    let kcp = g.computed_permanent(king).unwrap();
    assert_eq!((kcp.power, kcp.toughness), (2, 1), "king unbuffed");
    let lcp = g.computed_permanent(lion).unwrap();
    assert_eq!((lcp.power, lcp.toughness), (4, 2), "other Cat gets +2/+1");
}

/// Vesperlark returns a small creature when it leaves the battlefield.
#[test]
fn vesperlark_ltb_reanimates_small() {
    let mut g = two_player_game();
    let lark = g.add_card_to_battlefield(0, catalog::vesperlark());
    let lion = g.add_card_to_graveyard(0, catalog::savannah_lions()); // power 2 — illegal
    let goblin = g.add_card_to_graveyard(0, catalog::putrid_goblin()); // power 2 — illegal
    let elf = g.add_card_to_graveyard(0, catalog::llanowar_elves()); // power 1 — legal
    let _ = (lion, goblin);
    let evs = g.remove_to_graveyard_with_triggers(lark);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(elf).is_some(), "power-1 creature returned");
}

/// Igneous Elemental costs {2} less with a land in the graveyard.
#[test]
fn igneous_elemental_land_discount() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::island());
    let elem = g.add_card_to_hand(0, catalog::igneous_elemental());
    // {4}{R}{R} minus {2} = {2}{R}{R}.
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: elem, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("discounted by the graveyard land");
}

/// Mother Bear mints two Bears from the graveyard, exiling itself.
#[test]
fn mother_bear_graveyard_bears() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::mother_bear());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("activate from graveyard");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Bear").count(),
        2,
        "two Bear tokens"
    );
    assert!(g.exile.iter().any(|c| c.id == bear), "Mother Bear exiled as the cost");
}

/// Savage Swipe pumps a power-2 creature before it fights.
#[test]
fn savage_swipe_conditional_pump_fight() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
    let theirs = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    let swipe = g.add_card_to_hand(0, catalog::savage_swipe());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: swipe, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    // Bears become 4/4, deal 4 to the 3/3 (dies); take 3 back and survive at 4/1.
    assert!(g.battlefield_find(theirs).is_none(), "3/3 dies to the pumped fighter");
    assert!(g.battlefield_find(mine).is_some(), "pumped fighter survives");
}

/// Fists of Flame draws, then pumps by cards drawn this turn.
#[test]
fn fists_of_flame_draw_scaled_pump() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let fists = g.add_card_to_hand(0, catalog::fists_of_flame());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: fists, target: Some(Target::Permanent(bears)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bears).unwrap();
    assert_eq!(cp.power, 3, "2 base + one card drawn this turn");
    assert!(cp.keywords.contains(&crate::card::Keyword::Trample));
}

/// Changeling Outcast can't block and can't be blocked.
#[test]
fn changeling_outcast_evasion() {
    let mut g = two_player_game();
    let outcast = g.add_card_to_battlefield(0, catalog::changeling_outcast());
    let cp = g.computed_permanent(outcast).unwrap();
    assert!(cp.keywords.contains(&crate::card::Keyword::CantBlock));
    assert!(cp.keywords.contains(&crate::card::Keyword::Unblockable));
    assert!(cp.keywords.contains(&crate::card::Keyword::Changeling));
}

/// Irregular Cohort brings a changeling friend.
#[test]
fn irregular_cohort_makes_token() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::irregular_cohort());
    drain_stack(&mut g);
    let tok = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Shapeshifter")
        .expect("token minted");
    assert!(tok.definition.keywords.contains(&crate::card::Keyword::Changeling));
}

/// Rain of Revelation nets two cards (draw three, discard one).
#[test]
fn rain_of_revelation_draw_three_discard_one() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let rain = g.add_card_to_hand(0, catalog::rain_of_revelation());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: rain, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 2, "drew 3, discarded 1");
}

/// Martyr's Soul enters with two counters when you control no tapped lands.
#[test]
fn martyrs_soul_counters_when_untapped() {
    let mut g = two_player_game();
    let soul = g.move_card_to_battlefield_for_test(0, catalog::martyrs_soul());
    drain_stack(&mut g);
    let cp = g.computed_permanent(soul).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 4), "3/2 + two +1/+1 counters");
}

/// Orcish Hellraiser burns a player when it dies (echo aside).
#[test]
fn orcish_hellraiser_death_burn() {
    let mut g = two_player_game();
    let devil = g.add_card_to_battlefield(0, catalog::orcish_hellraiser());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Player(1)),
    ]));
    let life = g.players[1].life;
    g.battlefield_find_mut(devil).unwrap().damage = 2;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "2 damage on death");
}

/// Vengeful Devil's ping is morbid-gated.
#[test]
fn vengeful_devil_morbid_gate() {
    let mut g = two_player_game();
    let devil = g.add_card_to_battlefield(0, catalog::vengeful_devil());
    g.clear_sickness(devil);
    g.priority.player_with_priority = 0;
    // No creature has died — activation is illegal.
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: devil, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: vec![], x_value: None,
        })
        .is_err(),
        "morbid not active yet"
    );
    // Kill something, then the ping is legal.
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(fodder).unwrap().damage = 2;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: devil, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    })
    .expect("morbid active");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1);
}

/// Pondering Mage draws on arrival.
#[test]
fn pondering_mage_etb_draw() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.move_card_to_battlefield_for_test(0, catalog::pondering_mage());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "drew a card");
}

/// Graveshifter rescues a creature from the graveyard on arrival.
#[test]
fn graveshifter_returns_creature() {
    let mut g = two_player_game();
    let lion = g.add_card_to_graveyard(0, catalog::savannah_lions());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
        crate::decision::DecisionAnswer::Target(Target::Permanent(lion)),
    ]));
    g.move_card_to_battlefield_for_test(0, catalog::graveshifter());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == lion), "creature back to hand");
}

/// Excavating Anurid grows and gains vigilance once threshold is on.
#[test]
fn excavating_anurid_threshold() {
    let mut g = two_player_game();
    let frog = g.add_card_to_battlefield(0, catalog::excavating_anurid());
    let cp = g.computed_permanent(frog).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "no threshold yet");
    for _ in 0..7 {
        g.add_card_to_graveyard(0, catalog::island());
    }
    let cp = g.computed_permanent(frog).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "+1/+1 with threshold");
    assert!(cp.keywords.contains(&crate::card::Keyword::Vigilance));
}

/// Goblin War Party's entwine takes both modes: three Goblins and a team pump.
#[test]
fn goblin_war_party_entwine_both_modes() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let gwp = g.add_card_to_hand(0, catalog::goblin_war_party());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellEntwine {
        card_id: gwp, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("entwine both");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Goblin").count(),
        3,
        "three Goblins"
    );
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "team +1/+1");
    assert!(cp.keywords.contains(&crate::card::Keyword::Haste));
}

/// Viashino Sandsprinter returns to hand at the end step.
#[test]
fn viashino_sandsprinter_end_step_bounce() {
    let mut g = two_player_game();
    let viashino = g.add_card_to_battlefield(0, catalog::viashino_sandsprinter());
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(viashino).is_none(), "left the battlefield");
    assert!(g.players[0].hand.iter().any(|c| c.id == viashino), "back in hand");
}

/// Treetop Ambusher pumps a friend when it attacks.
#[test]
fn treetop_ambusher_attack_pump() {
    let mut g = two_player_game();
    let ambusher = g.add_card_to_battlefield(0, catalog::treetop_ambusher());
    let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(ambusher);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(friend)),
    ]));
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ambusher, target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let cp = g.computed_permanent(friend).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 to the chosen friend");
}

/// Zhalfirin Decoy's tap is gated on a creature having entered this turn
/// (CR 603 — its own arrival satisfies the gate).
#[test]
fn zhalfirin_decoy_entered_gate() {
    let mut g = two_player_game();
    let decoy = g.add_card_to_battlefield(0, catalog::zhalfirin_decoy());
    g.clear_sickness(decoy);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    // Nothing entered this turn (direct-add bypasses the counter) → illegal.
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: decoy, ability_index: 0, target: Some(Target::Permanent(victim)),
            additional_targets: vec![], x_value: None,
        })
        .is_err(),
        "no creature entered this turn"
    );
    // Mark a creature as having entered, then the tap is legal.
    g.players[0].creatures_entered_this_turn.push(decoy);
    g.perform_action(GameAction::ActivateAbility {
        card_id: decoy, ability_index: 0, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], x_value: None,
    })
    .expect("gate satisfied");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).unwrap().tapped, "victim tapped");
}

/// Murasa Behemoth swells while a land sits in the graveyard.
#[test]
fn murasa_behemoth_land_in_graveyard() {
    let mut g = two_player_game();
    let beh = g.add_card_to_battlefield(0, catalog::murasa_behemoth());
    let cp = g.computed_permanent(beh).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "no land in graveyard");
    g.add_card_to_graveyard(0, catalog::island());
    let cp = g.computed_permanent(beh).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 8), "+3/+3 with a graveyard land");
}

/// Knight of Old Benalia rallies the team when it enters.
#[test]
fn knight_of_old_benalia_etb_pump() {
    let mut g = two_player_game();
    let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::knight_of_old_benalia());
    drain_stack(&mut g);
    let cp = g.computed_permanent(friend).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "other creature gets +1/+1");
}

/// Rank Officer drains each opponent by exiling a creature from the graveyard.
#[test]
fn rank_officer_graveyard_drain() {
    let mut g = two_player_game();
    let officer = g.add_card_to_battlefield(0, catalog::rank_officer());
    g.clear_sickness(officer);
    let fodder = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: officer, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("activate drain");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "opponent lost 2");
    assert!(g.exile.iter().any(|c| c.id == fodder), "creature exiled as the cost");
}

/// Silumgar Scavenger grows when another creature you control dies.
#[test]
fn silumgar_scavenger_grows_on_death() {
    let mut g = two_player_game();
    let scav = g.add_card_to_battlefield(0, catalog::silumgar_scavenger());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(fodder).unwrap().damage = 2;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let cp = g.computed_permanent(scav).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4), "+1/+1 counter from the death");
}
