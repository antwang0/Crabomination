//! Commander: the 2026-10-18 (`019Y6J4R`) twentieth closer batch
//! (`decks::cmdr_019y_t`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
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

fn activate(g: &mut GameState, card_id: CardId, ability_index: usize, target: Option<Target>) -> bool {
    let ok = g
        .perform_action(GameAction::ActivateAbility {
            card_id,
            ability_index,
            target,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_ok();
    drain_stack(g);
    ok
}

fn minus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::MinusOneMinusOne))
}

/// CR 701.68 — Blight N as an activation cost: the -1/-1 counters land on a
/// creature you control that survives them (the 3/3 giant, not the 1/2
/// Dissident, for blight 2). The exiled card is linked to the Dissident, and
/// on your turn it's castable for its cost plus three counters removed from
/// among your creatures (-1/-1 counters first).
#[test]
fn dawnhand_dissident_blights_and_recasts_its_exiles() {
    let mut g = pod(2);
    let dd = ready(&mut g, 0, catalog::dawnhand_dissident());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    assert!(activate(&mut g, dd, 1, Some(Target::Permanent(bear))), "{{T}}, Blight 2");
    assert!(g.exile.iter().any(|c| c.id == bear && c.exiled_with == Some(dd)), "linked exile");
    assert_eq!((minus(&g, giant), minus(&g, dd)), (2, 0));

    g.players[0].mana_pool.add(Color::Green, 2);
    let cast = |g: &mut GameState| {
        g.perform_action(GameAction::CastFromZoneWithoutPaying {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
    };
    assert!(cast(&mut g).is_err(), "only two counters to remove");
    let elf = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(elf).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.active_player_idx = 1;
    assert!(!g.may_play_castable(0).contains(&bear), "not on an opponent's turn");
    g.active_player_idx = 0;
    assert!(g.may_play_castable(0).contains(&bear), "offered to the caster");
    cast(&mut g).expect("cast from linked exile");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear), "the creature resolved");
    assert_eq!(minus(&g, giant), 0, "the -1/-1 counters went first");
    assert_eq!(g.battlefield_find(elf).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!(g.players[0].mana_pool.total(), 0, "its mana cost was paid");
}

/// CR 701.68 — blight 1 alone: the Dissident is the only creature and
/// survives as a 0/1 after surveilling.
#[test]
fn dawnhand_dissident_blights_itself_to_surveil() {
    let mut g = pod(2);
    let dd = ready(&mut g, 0, catalog::dawnhand_dissident());
    assert!(activate(&mut g, dd, 0, None), "{{T}}, Blight 1");
    assert_eq!(minus(&g, dd), 1);
}

/// CR 903.4 / 106.1 — Cryptic Spires makes only its circled colors, which
/// for a two-color Commander deck are its commander's identity (Siani and
/// Yoshimaru: white and blue); it enters tapped, and it is colorless for
/// identity (no colored symbol in its text).
#[test]
fn cryptic_spires_taps_for_the_circled_colors() {
    use crabomination::mana::Color;
    let mut g = pod(2);
    let cmdr = g.add_card_to_battlefield(0, catalog::hanna_ships_navigator());
    g.players[0].commanders.push(cmdr);
    g.players[0].commander_identity = [Color::White, Color::Blue].into_iter().collect();
    let spires = g.add_card_to_battlefield(0, catalog::cryptic_spires());
    let identity = crabomination::color_identity::color_identity(&catalog::cryptic_spires());
    assert!(identity.is_empty(), "colorless identity");
    g.battlefield_find_mut(spires).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: spires, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("tap for mana");
    let pool = &g.players[0].mana_pool;
    assert_eq!(pool.total(), 1);
    assert_eq!(pool.amount(Color::White) + pool.amount(Color::Blue), 1, "white or blue only");
}

/// CR 707.9b — Absorbing Man's first-main-phase copy keeps his exceptions:
/// copying Sol Ring he's named Absorbing Man, a legendary 4/4 Human Villain
/// artifact creature with vigilance and Sol Ring's mana ability.
#[test]
fn absorbing_man_copies_an_artifact_with_his_exceptions() {
    use crabomination::card::{CardType, CreatureType, Keyword, Supertype};
    let mut g = pod(2);
    let man = ready(&mut g, 0, catalog::absorbing_man());
    ready(&mut g, 0, catalog::sol_ring());
    g.step = TurnStep::Draw;
    while g.step != TurnStep::PreCombatMain && g.game_over.is_none() {
        g.perform_action(GameAction::PassPriority).ok();
    }
    drain_stack(&mut g);
    let c = g.battlefield_find(man).expect("still here");
    assert_eq!(c.definition.name, "Absorbing Man");
    assert!(c.definition.supertypes.contains(&Supertype::Legendary));
    assert!(c.definition.card_types.contains(&CardType::Artifact), "a copy of Sol Ring");
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Villain));
    assert!(c.definition.keywords.contains(&Keyword::Vigilance));
    assert!(!c.definition.activated_abilities.is_empty(), "Sol Ring's mana ability");
    let cp = g.computed_permanent(man).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.card_types().contains(&CardType::Creature));
}

/// Hurkyl — one card per type among noncreature spells cast this turn: an
/// instant and an artifact cast take one Instant and one Artifact card from
/// the top five (not a second instant, not the creature); the rest go to the
/// bottom.
#[test]
fn hurkyl_takes_one_card_per_cast_type() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::hurkyl_master_wizard());
    g.players[0].library.clear();
    for def in [catalog::grizzly_bears(), catalog::lightning_bolt(), catalog::sol_ring(), catalog::shock(), catalog::forest()] {
        g.add_card_to_library(0, def);
    }
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: ring, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("ring");
    drain_stack(&mut g);
    let hand_before = g.players[0].hand.len();
    g.step = TurnStep::PostCombatMain;
    while g.step != TurnStep::End && g.game_over.is_none() {
        g.perform_action(GameAction::PassPriority).ok();
    }
    drain_stack(&mut g);
    let names: Vec<&str> = g.players[0].hand.iter().skip(hand_before).map(|c| c.definition.name).collect();
    assert_eq!(names.len(), 2, "one instant and one artifact: {names:?}");
    assert!(names.contains(&"Sol Ring"));
    assert!(names.contains(&"Lightning Bolt") || names.contains(&"Shock"));
    assert_eq!(g.players[0].library.len(), 8);
    assert_eq!(g.players[0].library[0].definition.name, "Island", "the other three went to the bottom");
}

/// CR 610.3 — Cloak and Dagger's ETB targets an opponent (slot 0) and up to
/// one creature that player controls (slot 1, SameControllerAsTargetSlot);
/// you exile a nonland card from their hand or, declining that, the chosen
/// creature, and it comes back when Cloak and Dagger leave.
#[test]
fn cloak_and_dagger_exile_until_they_leave() {
    for take_hand in [true, false] {
        let mut g = pod(2);
        let opp_bear = ready(&mut g, 1, catalog::grizzly_bears());
        let mine = ready(&mut g, 0, catalog::grizzly_bears());
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.add_card_to_hand(1, catalog::forest());
        let cd = g.add_card_to_hand(0, catalog::cloak_and_dagger_entwined());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::CastSpell { card_id: cd, target: None, additional_targets: vec![], mode: None, x_value: None })
            .expect("cast");
        let script = if take_hand { vec![DecisionAnswer::Bool(true)] } else { vec![DecisionAnswer::Bool(false), DecisionAnswer::Bool(true)] };
        g.decider = Box::new(ScriptedDecider::new(script));
        drain_stack(&mut g);
        assert!(g.battlefield_find(mine).is_some(), "never its own side");
        assert_eq!(g.exile.iter().any(|c| c.id == bolt), take_hand, "the hand card");
        assert_eq!(g.battlefield_find(opp_bear).is_none(), !take_hand, "or the chosen creature");
        let cd_id = g.battlefield.iter().find(|c| c.definition.name == "Cloak and Dagger, Entwined").map(|c| c.id).unwrap();
        let murder = g.add_card_to_hand(0, catalog::murder());
        g.players[0].mana_pool.add(Color::Black, 3);
        g.perform_action(GameAction::CastSpell {
            card_id: murder, target: Some(Target::Permanent(cd_id)), additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("murder");
        drain_stack(&mut g);
        assert!(g.players[1].hand.iter().any(|c| c.definition.name == "Lightning Bolt"), "the card is in hand");
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 1), "the bear is back");
    }
}

/// Sculpted Sunburst (CR 101.4 order): I keep my 3/3; each opponent keeps one
/// creature with power 3 or less — the 5/5 can't be kept, so the one
/// opponent keeps the bear and the other, with only a 5/5, keeps nothing.
#[test]
fn sculpted_sunburst_keeps_one_creature_each() {
    let mut g = pod(3);
    let giant = ready(&mut g, 0, catalog::hill_giant());
    let my_bear = ready(&mut g, 0, catalog::grizzly_bears());
    let opp_bear = ready(&mut g, 1, catalog::grizzly_bears());
    let opp_elf = ready(&mut g, 1, catalog::llanowar_elves());
    let big = ready(&mut g, 2, catalog::craw_wurm());
    let sun = g.add_card_to_hand(0, catalog::sculpted_sunburst());
    g.players[0].mana_pool.add(Color::White, 5);
    g.perform_action(GameAction::CastSpell { card_id: sun, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_some(), "my chosen creature");
    assert!(g.battlefield_find(my_bear).is_none());
    assert!(g.battlefield_find(opp_bear).is_some(), "their strongest within power 3");
    assert!(g.battlefield_find(opp_elf).is_none());
    assert!(g.battlefield_find(big).is_none(), "power 6 can't be chosen");
}

/// Fumble — the bounced creature's Equipment and Aura change control (CR
/// 108.4) and attach to another creature: the opponent's bear goes home, its
/// Bonesplitter and Rancor end up mine on my bear.
#[test]
fn fumble_steals_the_attachments() {
    let mut g = pod(2);
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let split = ready(&mut g, 1, catalog::bonesplitter());
    let rancor = ready(&mut g, 1, catalog::rancor());
    for id in [split, rancor] {
        g.battlefield_find_mut(id).unwrap().attached_to = Some(theirs);
    }
    let f = g.add_card_to_hand(0, catalog::fumble());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: f, target: Some(Target::Permanent(theirs)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("fumble");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == theirs), "back to its owner's hand");
    for id in [split, rancor] {
        let c = g.battlefield_find(id).expect("still on the battlefield");
        assert_eq!(c.controller, 0, "{} is mine", c.definition.name);
        assert_eq!(c.attached_to, Some(mine));
    }
}

/// Threats Undetected — up to four creature cards with different powers are
/// found (two 2-power cards can't both be); the opponent sends back two, the
/// rest reach your hand and the library is shuffled.
#[test]
fn threats_undetected_finds_distinct_powers() {
    let mut g = pod(2);
    g.players[0].library.clear();
    for def in [
        catalog::grizzly_bears(),
        catalog::grizzly_bears(),
        catalog::hill_giant(),
        catalog::llanowar_elves(),
        catalog::craw_wurm(),
        catalog::forest(),
    ] {
        g.add_card_to_library(0, def);
    }
    let t = g.add_card_to_hand(0, catalog::threats_undetected());
    g.players[0].mana_pool.add(Color::Green, 3);
    let hand_before = g.players[0].hand.len() - 1;
    g.perform_action(GameAction::CastSpell { card_id: t, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    let got: Vec<&str> = g.players[0].hand.iter().skip(hand_before).map(|c| c.definition.name).collect();
    assert_eq!(got.len(), 2, "four found, two sent back: {got:?}");
    assert_eq!(g.players[0].library.len(), 4);
    assert!(g.players[0].searched_library_this_turn);
}

/// Trickster's Talisman — +1/+1; on combat damage to a player, sacrificing
/// it makes a token copy of the equipped creature (CR 707.2).
#[test]
fn tricksters_talisman_copies_on_combat_damage() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let tal = ready(&mut g, 0, catalog::tricksters_talisman());
    g.battlefield_find_mut(tal).unwrap().attached_to = Some(bear);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: bear,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain && g.game_over.is_none() {
        g.perform_action(GameAction::PassPriority).ok();
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(tal).is_none(), "sacrificed");
    let bears = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears" && c.controller == 0).count();
    assert_eq!(bears, 2, "a token copy");
}

/// CR 714.3b / 606.3 — Urza Assembles the Titans' chapter III (its third lore
/// counter, added after the draw step) lets Jace Beleren's loyalty abilities
/// be activated twice this turn, and no more.
#[test]
fn urza_assembles_the_titans_doubles_loyalty_activations() {
    let mut g = pod(2);
    let saga = ready(&mut g, 0, catalog::urza_assembles_the_titans());
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 2);
    let jace = ready(&mut g, 0, catalog::jace_beleren());
    g.battlefield_find_mut(jace).unwrap().add_counters(CounterType::Loyalty, 5);
    g.step = TurnStep::Draw;
    while g.step != TurnStep::PreCombatMain && g.game_over.is_none() {
        g.perform_action(GameAction::PassPriority).ok();
    }
    drain_stack(&mut g);
    let loyal = |g: &mut GameState| {
        let ok = g
            .perform_action(GameAction::ActivateLoyaltyAbility { card_id: jace, ability_index: 0, target: None, x_value: None })
            .is_ok();
        drain_stack(g);
        ok
    };
    assert!(loyal(&mut g), "first activation");
    assert!(loyal(&mut g), "second activation this turn (chapter III)");
    assert!(!loyal(&mut g), "no third");
}

/// Musician — two activations put two music counters on the bear but grant
/// the upkeep toll once; at its controller's upkeep, unable to pay {2}, it's
/// destroyed (CR 701.7).
#[test]
fn musician_grants_one_toll_that_grows() {
    let mut g = pod(2);
    let mus = ready(&mut g, 0, catalog::musician());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    for _ in 0..2 {
        g.battlefield_find_mut(mus).unwrap().tapped = false;
        g.perform_action(GameAction::ActivateAbility {
            card_id: mus, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: vec![], x_value: None, mode: None,
        })
        .expect("tap: music counter");
        drain_stack(&mut g);
    }
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!(b.counter_count(CounterType::Music), 2);
    assert_eq!(b.definition.triggered_abilities.len(), 1, "the toll is granted once");
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Draw && g.game_over.is_none() {
        g.perform_action(GameAction::PassPriority).ok();
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "destroyed: no {{2}} to pay");
}

/// CR 506.4 / 509.1h — Balduvian Warlord pulls my bear off the attacking
/// bear (which, blocked by nothing else, becomes unblocked) and makes it
/// block the Craw Wurm instead.
#[test]
fn balduvian_warlord_moves_a_blocker() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = pod(2);
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let wurm = ready(&mut g, 0, catalog::craw_wurm());
    let warlord = ready(&mut g, 1, catalog::balduvian_warlord());
    let mine = ready(&mut g, 1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: wurm, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(mine, a)])).expect("block");
    drain_stack(&mut g);
    assert!(g.attacker_is_blocked(a));
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: warlord, ability_index: 0, target: Some(Target::Permanent(mine)), additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("warlord");
    drain_stack(&mut g);
    assert!(!g.attacker_is_blocked(a), "unblocked");
    assert!(g.attacker_is_blocked(wurm), "now blocked");
    assert_eq!(g.blockers_of(wurm).as_slice(), &[mine]);
}

/// CR 707.10 — Giant Growth on Beamsplitter Mage alone is copied once, onto
/// the creature chosen (the stronger of my other two): the Mage and the
/// giant get +3/+3, the bear doesn't.
#[test]
fn beamsplitter_mage_copies_once_onto_a_chosen_creature() {
    let mut g = pod(2);
    let mage = ready(&mut g, 0, catalog::beamsplitter_mage());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    let gg = g.add_card_to_hand(0, catalog::giant_growth());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: gg, target: Some(Target::Permanent(mage)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let p = |g: &GameState, id| g.computed_permanent(id).unwrap().power;
    assert_eq!((p(&g, mage), p(&g, giant), p(&g, bear)), (5, 6, 2));
}

/// Cosmium Confluence, modes 1 + 2 + 2: fetch a Cave tapped, then twice
/// animate a Cave I control with three +1/+1 counters — a 0/0 Elemental
/// creature with haste that's still a land (CR 205.1b, 702.10).
#[test]
fn cosmium_confluence_animates_a_cave() {
    let mut g = pod(2);
    let cave = ready(&mut g, 0, catalog::captivating_cave());
    g.add_card_to_library(0, catalog::captivating_cave());
    let cc = g.add_card_to_hand(0, catalog::cosmium_confluence());
    g.players[0].mana_pool.add(Color::Green, 5);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: cc, spree_modes: vec![0, 1, 1], target: None, additional_targets: vec![], x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let caves: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Captivating Cave" && c.controller == 0).map(|c| c.id).collect();
    assert_eq!(caves.len(), 2, "one fetched");
    let animated: Vec<_> = caves.iter().filter(|&&id| g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne) > 0).collect();
    assert!(!animated.is_empty());
    let total: u32 = caves.iter().map(|&id| g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne)).sum();
    assert_eq!(total, 6, "three counters, twice");
    let a = *animated[0];
    let cp = g.computed_permanent(a).unwrap();
    assert!(cp.card_types().contains(&crabomination::card::CardType::Creature));
    assert!(cp.card_types().contains(&crabomination::card::CardType::Land), "still a land");
    let _ = cave;
}

/// Brine Comber — a Spirit on entering and another when an Aura spell
/// targets it; disturbed (CR 702.146) as Brinebound Gift onto the bear, a
/// Spirit as it enters and another when an Aura spell targets the bear.
#[test]
fn brine_comber_makes_spirits_from_aura_spells() {
    let mut g = pod(2);
    let spirits = |g: &GameState| g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Spirit").count();
    let bc = g.add_card_to_hand(0, catalog::brine_comber());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell { card_id: bc, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(spirits(&g), 1, "enters");
    let rancor = g.add_card_to_hand(0, catalog::rancor());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: rancor, target: Some(Target::Permanent(bc)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("rancor");
    drain_stack(&mut g);
    assert_eq!(spirits(&g), 2, "targeted by an Aura spell");

    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let bc = g.add_card_to_graveyard(0, catalog::brine_comber());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastDisturb { card_id: bc, target: Some(Target::Permanent(bear)), additional_targets: vec![] })
    .expect("disturb");
    drain_stack(&mut g);
    assert_eq!(spirits(&g), 1, "the Gift enters");
    let rancor = g.add_card_to_hand(0, catalog::rancor());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: rancor, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("rancor");
    drain_stack(&mut g);
    assert_eq!(spirits(&g), 2, "enchanted creature targeted by an Aura spell");
}

/// CR 707.10 — Scientist Supreme copies Ichor Wellspring's enters trigger
/// (an artifact source): two cards drawn instead of one, for 2 life.
#[test]
fn scientist_supreme_copies_an_artifact_trigger() {
    let mut g = pod(2);
    let sci = ready(&mut g, 0, catalog::scientist_supreme_of_a_i_m());
    let well = g.add_card_to_hand(0, catalog::ichor_wellspring());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastSpell { card_id: well, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    // Resolve the artifact spell; its enters trigger waits on the stack.
    g.perform_action(GameAction::PassPriority).ok();
    g.perform_action(GameAction::PassPriority).ok();
    let well_id = g.battlefield.iter().find(|c| c.definition.name == "Ichor Wellspring").map(|c| c.id).expect("entered");
    let hand = g.players[0].hand.len();
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sci, ability_index: 0, target: Some(Target::Permanent(g.top_ability_of(well_id).expect("its trigger"))),
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("copy the trigger");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2);
    assert_eq!(g.players[0].hand.len(), hand + 2, "the trigger and its copy");

    // An enchantment's trigger (Elemental Bond) isn't from an artifact source.
    let mut g = pod(2);
    let sci = ready(&mut g, 0, catalog::scientist_supreme_of_a_i_m());
    let bond = ready(&mut g, 0, catalog::elemental_bond());
    let giant = g.add_card_to_hand(0, catalog::hill_giant());
    g.players[0].mana_pool.add(Color::Red, 4);
    g.perform_action(GameAction::CastSpell { card_id: giant, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    g.perform_action(GameAction::PassPriority).ok();
    g.perform_action(GameAction::PassPriority).ok();
    let ability = g.top_ability_of(bond).expect("Bond's trigger");
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: sci, ability_index: 0, target: Some(Target::Permanent(ability)), additional_targets: vec![], x_value: None, mode: None,
        })
        .is_err(),
        "not an artifact source"
    );
}

/// Synthesis Pod — exiling my Lightning Bolt (mana value 1) off the stack as
/// the cost digs the opponent's library to its first mana-value-2 card, which
/// I cast for free; the bolt never resolves (CR 602.5b, 118.9).
#[test]
fn synthesis_pod_turns_a_spell_into_the_next_mana_value() {
    let mut g = pod(2);
    let podc = ready(&mut g, 0, catalog::synthesis_pod());
    g.players[1].library.clear();
    g.add_card_to_library(1, catalog::forest());
    let bear = g.add_card_to_library(1, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::hill_giant());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Blue, 2);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("bolt");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: podc, ability_index: 0, target: Some(Target::Player(1)), additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("pod the bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life, "the bolt was exiled, not resolved");
    assert!(g.exile.iter().any(|c| c.id == bolt));
    let b = g.battlefield_find(bear).expect("cast for free");
    assert_eq!(b.controller, 0);
    assert_eq!(g.players[1].library.len(), 2, "the forest shuffled back with the giant");
}

/// CR 706 / 707.12 — Wizard's Spellbook exiles the Lightning Bolt in my
/// graveyard and rolls 15: the copy is cast for {1}, dealing 3; the card
/// stays exiled with the Book.
#[test]
fn wizards_spellbook_casts_a_copy_for_one() {
    let mut g = pod(2);
    let book = ready(&mut g, 0, catalog::wizards_spellbook());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(15), DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: book, ability_index: 0, target: Some(Target::Permanent(bolt)), additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("tap the Book");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt && c.exiled_with == Some(book)), "exiled with the Book");
    assert_eq!(g.players[1].life, life - 3, "the copy resolved");
    assert_eq!(g.players[0].mana_pool.total(), 0, "{{1}} paid");
}
