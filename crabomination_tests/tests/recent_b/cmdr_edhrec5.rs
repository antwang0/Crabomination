//! Commander: EDHREC average-deck gaps, fifth batch (`decks::cmdr_edhrec5`):
//! Lord of the Nazgûl's Wraiths and Ring spells.

use crabomination::card::{CardId, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn attack(g: &mut GameState, seat: usize, attackers: &[CardId], defender: usize) {
    g.active_player_idx = seat;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(defender) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn combat_damage(g: &mut GameState) {
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

fn end_step(g: &mut GameState) {
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(g);
}

fn on_bf(g: &GameState, id: CardId) -> bool {
    g.battlefield_find(id).is_some()
}

/// Witch-king of Angmar: one trigger for a whole swing (CR 603.2c "one or
/// more"), and the sacrifice only offers a creature that dealt seat 0 combat
/// damage this turn; the Ring tempts its controller once.
#[test]
fn witch_king_of_angmar_makes_the_hitters_pay_once() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::witch_king_of_angmar());
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::hill_giant());
    let home = ready(&mut g, 1, catalog::runeclaw_bear());
    let other = ready(&mut g, 2, catalog::grizzly_bears());
    attack(&mut g, 1, &[a, b], 0);
    combat_damage(&mut g);
    let gone = [a, b].iter().filter(|&&c| !on_bf(&g, c)).count();
    assert_eq!(gone, 1, "one sacrifice for the batch");
    assert!(on_bf(&g, home), "it didn't deal combat damage");
    assert!(on_bf(&g, other), "seat 2 controls no hitter");
    assert_eq!(g.players[0].ring_temptations, 1);
}

/// Witch-king of Angmar's discard ability: indestructible, and it taps.
#[test]
fn witch_king_of_angmar_discards_to_survive() {
    let mut g = pod(2);
    let wk = ready(&mut g, 0, catalog::witch_king_of_angmar());
    g.add_card_to_hand(0, catalog::island());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: wk,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("discard");
    drain_stack(&mut g);
    assert!(g.players[0].hand.is_empty());
    let c = g.battlefield_find(wk).unwrap();
    assert!(c.tapped);
    assert!(g.computed_permanent(wk).unwrap().keywords().contains(&Keyword::Indestructible));
}

/// Witch-king, Bringer of Ruin: the defending player gives up the creature
/// with the least power they control, not another opponent's.
#[test]
fn witch_king_bringer_of_ruin_takes_the_smallest() {
    let mut g = pod(3);
    let wk = ready(&mut g, 0, catalog::witch_king_bringer_of_ruin());
    let small = ready(&mut g, 1, catalog::llanowar_elves());
    let big = ready(&mut g, 1, catalog::hill_giant());
    let elsewhere = ready(&mut g, 2, catalog::llanowar_elves());
    attack(&mut g, 0, &[wk], 1);
    assert!(!on_bf(&g, small));
    assert!(on_bf(&g, big));
    assert!(on_bf(&g, elsewhere));
}

/// Ringwraiths: -3/-3 on entry, 3 life only off a legendary target; the Ring
/// tempting you returns it from the graveyard.
#[test]
fn ringwraiths_shrink_legends_and_come_back() {
    let mut g = pod(2);
    let legend = ready(&mut g, 1, catalog::witch_king_bringer_of_ruin());
    let rw = g.add_card_to_hand(0, catalog::ringwraiths());
    cast(&mut g, 0, rw, None).expect("Ringwraiths");
    let c = g.computed_permanent(legend);
    assert!(c.is_none_or(|c| c.toughness == 0), "5/3 took -3/-3");
    assert_eq!(g.players[1].life, 17, "a legendary target");

    let mut g = pod(2);
    let maw = ready(&mut g, 1, catalog::colossal_dreadmaw());
    let rw = g.add_card_to_hand(0, catalog::ringwraiths());
    cast(&mut g, 0, rw, None).expect("Ringwraiths");
    assert_eq!(g.computed_permanent(maw).unwrap().power, 3);
    assert_eq!(g.players[1].life, 20, "not legendary");

    let rw = g.add_card_to_graveyard(0, catalog::ringwraiths());
    let rescue = g.add_card_to_hand(0, catalog::sams_desperate_rescue());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bears = g.players[0].graveyard.iter().find(|c| c.definition.name == "Grizzly Bears").unwrap().id;
    cast(&mut g, 0, rescue, Some(Target::Permanent(bears))).expect("rescue");
    assert!(g.players[0].hand.iter().any(|c| c.id == bears));
    assert!(g.players[0].hand.iter().any(|c| c.id == rw), "the Ring tempted: Ringwraiths returns");
}

/// Sauron, the Necromancer: the attack trigger exiles a creature card for a
/// tapped, attacking 3/3 black Wraith copy with menace (CR 707.9b), exiled at
/// the end step unless Sauron is the Ring-bearer.
#[test]
fn sauron_the_necromancer_raises_wraiths() {
    for bearer in [false, true] {
        let mut g = pod(2);
        let sauron = ready(&mut g, 0, catalog::sauron_the_necromancer());
        let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
        if bearer {
            g.players[0].ring_bearer = Some(sauron);
        }
        attack(&mut g, 0, &[sauron], 1);
        assert!(g.exile.iter().any(|c| c.id == giant), "the card is exiled");
        let token = g
            .battlefield
            .iter()
            .find(|c| c.is_token && c.definition.name == "Hill Giant")
            .map(|c| c.id)
            .expect("the copy");
        let t = g.battlefield_find(token).unwrap();
        assert!(t.tapped);
        let cp = g.computed_permanent(token).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3));
        assert!(cp.keywords().contains(&Keyword::Menace));
        assert_eq!(cp.colors.to_vec(), vec![Color::Black]);
        assert_eq!(cp.subtypes().creature_types, vec![CreatureType::Wraith], "instead of Giant");
        assert!(g.attacking().iter().any(|a| a.attacker == token), "attacking");
        end_step(&mut g);
        assert_eq!(on_bf(&g, token), bearer, "kept only while Sauron bears the Ring");
    }
}

/// Glorious Gale counters a creature spell; only a legendary one tempts you.
#[test]
fn glorious_gale_tempts_off_a_legend() {
    for (def, tempted) in [(catalog::witch_king_of_angmar(), 1), (catalog::grizzly_bears(), 0)] {
        let mut g = pod(2);
        let creature = g.add_card_to_hand(1, def);
        flood(&mut g, 1);
        g.priority.player_with_priority = 1;
        g.active_player_idx = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: creature,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast the creature");
        let gale = g.add_card_to_hand(0, catalog::glorious_gale());
        flood(&mut g, 0);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: gale,
            target: Some(Target::Permanent(creature)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("Gale");
        drain_stack(&mut g);
        assert!(!on_bf(&g, creature), "countered");
        assert_eq!(g.players[0].ring_temptations, tempted);
    }
}

/// Ringsight finds only a card sharing a color with a legendary creature you
/// control — never a colorless one (its ruling).
#[test]
fn ringsight_searches_by_a_legends_colors() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::witch_king_of_angmar());
    ready(&mut g, 0, catalog::llanowar_elves());
    g.add_card_to_library(0, catalog::sol_ring());
    g.add_card_to_library(0, catalog::llanowar_elves());
    let black = g.add_card_to_library(0, catalog::grizzly_bears());
    let _ = black;
    let swamp_card = g.add_card_to_library(0, catalog::sams_desperate_rescue());
    let rs = g.add_card_to_hand(0, catalog::ringsight());
    cast(&mut g, 0, rs, None).expect("Ringsight");
    assert_eq!(g.players[0].ring_temptations, 1);
    let hand: Vec<_> = g.players[0].hand.iter().map(|c| c.id).collect();
    assert_eq!(hand, vec![swamp_card], "the only black card");
}

/// Gandalf, Friend of the Shire: sorceries at instant speed; the Ring tempting
/// you draws only when another creature becomes the bearer.
#[test]
fn gandalf_draws_when_another_bears_the_ring() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let gandalf = ready(&mut g, 0, catalog::gandalf_friend_of_the_shire());
    let rescue = g.add_card_to_hand(0, catalog::sams_desperate_rescue());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.step = TurnStep::Upkeep;
    let hand = g.players[0].hand.len();
    cast(&mut g, 0, rescue, Some(Target::Permanent(bear))).expect("a sorcery with flash");
    assert_eq!(g.players[0].ring_bearer, Some(gandalf), "the only creature");
    assert_eq!(g.players[0].hand.len(), hand, "rescue spent, bear returned, no draw");
    ready(&mut g, 0, catalog::hill_giant());
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(&crabomination::effect::Effect::RingTempts { who: crabomination::effect::PlayerRef::You }, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "the Giant bears it: draw");
}

/// Sauron's Ransom: four cards split between hand and graveyard; the Ring
/// tempts you.
#[test]
fn saurons_ransom_splits_four() {
    let mut g = pod(2);
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
    }
    let ransom = g.add_card_to_hand(0, catalog::saurons_ransom());
    cast(&mut g, 0, ransom, None).expect("Ransom");
    assert_eq!(g.players[0].hand.len() + g.players[0].graveyard.len(), 5, "four split, plus the Ransom");
    assert_eq!(g.players[0].library.len(), 2);
    assert_eq!(g.players[0].ring_temptations, 1);
}

fn sneak(g: &mut GameState, attacker: CardId, defender: usize, card: CardId) {
    g.attacking = vec![Attack { attacker, target: AttackTarget::Player(defender) }];
    g.step = TurnStep::DeclareBlockers;
    g.active_player_idx = 0;
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: card,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("sneak");
    drain_stack(g);
}

fn counters(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(crabomination::card::CounterType::PlusOnePlusOne))
}

fn mutagens(g: &GameState) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == "Mutagen").count()
}

/// CR 702.190b — a permanent spell cast for its sneak cost enters tapped and
/// attacking the player the returned creature was attacking (seat 2 here, not
/// the first opponent); Raphael's attackers have double strike.
#[test]
fn cr_702_190b_a_sneaked_raphael_enters_attacking_the_returned_creatures_defender() {
    let mut g = pod(3);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let raph = g.add_card_to_hand(0, catalog::raphael_the_nightwatcher());
    sneak(&mut g, bear, 2, raph);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "the bear paid the cost");
    let r = g.battlefield_find(raph).expect("Raphael entered");
    assert!(r.tapped);
    assert!(
        g.attacking.iter().any(|a| a.attacker == raph && a.target == AttackTarget::Player(2)),
        "attacking seat 2"
    );
    assert!(g.computed_permanent(raph).unwrap().keywords().contains(&Keyword::DoubleStrike));
}

/// A sneaked Dark Leo & Shredder hits: a Ninja token; at five Ninjas the
/// player hit loses half their life, rounded up.
#[test]
fn dark_leo_and_shredder_halve_a_player_at_five_ninjas() {
    let mut g = pod(2);
    let leo = ready(&mut g, 0, catalog::dark_leo_and_shredder());
    attack(&mut g, 0, &[leo], 1);
    combat_damage(&mut g);
    assert_eq!(g.players[1].life, 19, "fewer than five Ninjas");
    for _ in 0..3 {
        ready(&mut g, 0, catalog::ninja_of_the_deep_hours());
    }
    g.battlefield_find_mut(leo).unwrap().tapped = false;
    g.attacking.clear();
    attack(&mut g, 0, &[leo], 1);
    combat_damage(&mut g);
    // 1 damage (19 -> 18); the token makes five Ninjas; half of 18 is 9.
    assert_eq!(g.players[1].life, 9);
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Ninja").count(), 2);
}

/// The Ooze: a creature of yours leaving with +1/+1 counters leaves that many
/// Mutagens (CR 603.10a — read as it last existed); {T} exiles a graveyard
/// card for one more. Mutagen Man makes X and cuts their activation to {0}.
#[test]
fn the_ooze_and_mutagen_man_brew_mutagen() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::the_ooze());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(crabomination::card::CounterType::PlusOnePlusOne, 2);
    let plain = ready(&mut g, 0, catalog::grizzly_bears());
    let mut evs = Vec::new();
    g.destroy_permanent(bear, false, &mut evs);
    g.destroy_permanent(plain, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(mutagens(&g), 2, "two counters; the plain bear none");

    let mut g = pod(2);
    let man = g.add_card_to_hand(0, catalog::mutagen_man_living_ooze());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: man, target: None, additional_targets: vec![], mode: None, x_value: Some(3) })
        .expect("X = 3");
    drain_stack(&mut g);
    assert_eq!(mutagens(&g), 3);
    g.players[0].mana_pool = Default::default();
    let m = g.battlefield.iter().find(|c| c.definition.name == "Mutagen").unwrap().id;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: m,
        ability_index: 0,
        target: Some(Target::Permanent(man)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("{1} less: free");
    drain_stack(&mut g);
    assert_eq!(counters(&g, man), 1);
}

/// Michelangelo, Weirdness to 11 adds one to each +1/+1 counter placement;
/// Mikey & Leo draws on the first counter you put each turn only.
#[test]
fn weirdness_and_mikey_and_leo_on_counters() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    ready(&mut g, 0, catalog::michelangelo_weirdness_to_11());
    ready(&mut g, 0, catalog::mikey_and_leo_chaos_and_order());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let put = crabomination::effect::Effect::AddCounter {
        what: crabomination::effect::Selector::ExactObjects(vec![bear]),
        kind: crabomination::card::CounterType::PlusOnePlusOne,
        amount: crabomination::card::Value::ONE,
    };
    for _ in 0..2 {
        let evs = g.resolve_effect(&put, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert_eq!(counters(&g, bear), 4, "one plus one, twice");
    assert_eq!(g.players[0].hand.len(), hand + 1, "once each turn");
}

/// Mikey & Don: a Turtle off the top enters with an extra +1/+1 counter; a
/// non-Turtle spell on top stays put.
#[test]
fn mikey_and_don_cast_turtles_from_the_top() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::mikey_and_don_party_planners());
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    assert!(g
        .perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None
        })
        .is_err());
    g.players[0].library.clear();
    let turtle = g.add_card_to_library(0, catalog::leonardo_the_balance());
    cast(&mut g, 0, turtle, None).expect("a Turtle off the top");
    assert_eq!(counters(&g, turtle), 1);
    let hand_bff = g.add_card_to_hand(0, catalog::michelangelo_mutant_bff());
    cast(&mut g, 0, hand_bff, None).expect("from hand");
    assert_eq!(counters(&g, hand_bff), 0, "not cast this way");
}

/// Splinter doubles a Ninja's triggers (Michelangelo, Mutant BFF's entry
/// makes two Mutagens), and its {1}{U} makes a Ninja unblockable.
#[test]
fn splinter_doubles_ninja_triggers() {
    let mut g = pod(2);
    let splinter = ready(&mut g, 0, catalog::splinter_radical_rat());
    let bff = g.add_card_to_hand(0, catalog::michelangelo_mutant_bff());
    cast(&mut g, 0, bff, None).expect("BFF");
    assert_eq!(mutagens(&g), 2);
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: splinter,
        ability_index: 0,
        target: Some(Target::Permanent(bff)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("unblockable");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bff).unwrap().keywords().contains(&Keyword::Unblockable));
}
