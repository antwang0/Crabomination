//! Prophecy (PCY), second wave.

use crabomination::card::{CardDefinition, CardId, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn script(g: &mut GameState, answers: Vec<DecisionAnswer>) {
    g.decider = Box::new(ScriptedDecider::new(answers));
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
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

fn activate_x(
    g: &mut GameState,
    seat: usize,
    card_id: CardId,
    index: usize,
    target: Option<Target>,
    x: Option<u32>,
) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: x,
    })
    .expect("activate");
    drain_stack(g);
}

/// The printed-keyword bodies.
#[test]
fn pcy2_keyword_bodies_carry_their_printed_keywords() {
    let cases: &[(fn() -> CardDefinition, &[Keyword])] = &[
        (catalog::pygmy_razorback, &[Keyword::Trample]),
        (catalog::marsh_boa, &[Keyword::Landwalk(crabomination::card::LandType::Swamp)]),
        (catalog::pit_raptor, &[Keyword::Flying, Keyword::FirstStrike]),
        (catalog::quicksilver_wall, &[Keyword::Defender]),
        (catalog::jeweled_spirit, &[Keyword::Flying]),
        (catalog::magetas_boon, &[Keyword::Flash]),
        (catalog::jolraels_favor, &[Keyword::Flash]),
        (catalog::latullas_orders, &[Keyword::Flash]),
    ];
    for (factory, expected) in cases {
        let def = factory();
        for kw in *expected {
            assert!(def.keywords.contains(kw), "{} is missing {kw:?}", def.name);
        }
    }
}

/// Greel turns two cards into X random discards.
#[test]
fn greel_mind_raker_discards_x_at_random() {
    let mut g = main_phase();
    let greel = g.add_card_to_battlefield(0, catalog::greel_mind_raker());
    g.clear_sickness(greel);
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::grizzly_bears());
    }
    for _ in 0..4 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    activate_x(&mut g, 0, greel, 0, Some(Target::Player(1)), Some(2));
    assert_eq!(g.players[1].hand.len(), 2, "X = 2");
    assert!(g.players[0].hand.is_empty(), "and it cost two cards");
}

/// Latulla turns two cards into X damage.
#[test]
fn latulla_deals_x_damage() {
    let mut g = main_phase();
    let latulla = g.add_card_to_battlefield(0, catalog::latulla_keldon_overseer());
    g.clear_sickness(latulla);
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::grizzly_bears());
    }
    activate_x(&mut g, 0, latulla, 0, Some(Target::Player(1)), Some(4));
    assert_eq!(g.players[1].life, 16);
}

/// Jolrael stands their lands up so they can be swept.
#[test]
fn jolrael_animates_their_lands() {
    let mut g = main_phase();
    let jolrael = g.add_card_to_battlefield(0, catalog::jolrael_empress_of_beasts());
    g.clear_sickness(jolrael);
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::grizzly_bears());
    }
    let land = g.add_card_to_battlefield(1, catalog::island());
    activate_x(&mut g, 0, jolrael, 0, Some(Target::Player(1)), None);
    let cp = g.computed_permanent(land).expect("computed");
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.card_types.contains(&crabomination::card::CardType::Land), "still a land");
}

/// Mageta wraths everything but itself.
#[test]
fn mageta_wraths_around_itself() {
    let mut g = main_phase();
    let mageta = g.add_card_to_battlefield(0, catalog::mageta_the_lion());
    g.clear_sickness(mageta);
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::grizzly_bears());
    }
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate_x(&mut g, 0, mageta, 0, None, None);
    assert!(g.battlefield_find(mageta).is_some(), "Mageta survives");
    assert!(g.battlefield_find(mine).is_none());
    assert!(g.battlefield_find(theirs).is_none());
}

/// Hazy Homunculus slips past anyone holding mana up.
#[test]
fn hazy_homunculus_is_unblockable_while_they_hold_mana() {
    let mut g = two_player_game();
    let homunculus = g.add_card_to_battlefield(0, catalog::hazy_homunculus());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(1, catalog::island());
    assert!(!g.blocker_can_block_attacker(blocker, homunculus), "untapped land → unblockable");
    g.battlefield_find_mut(land).unwrap().tapped = true;
    assert!(g.blocker_can_block_attacker(blocker, homunculus), "tapped out → blockable");
}

/// Heightened Awareness costs your hand and pays an extra card each turn.
#[test]
fn heightened_awareness_discards_your_hand_then_draws_extra() {
    let mut g = main_phase();
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let awareness = g.add_card_to_hand(0, catalog::heightened_awareness());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    cast(&mut g, 0, awareness, None);
    assert!(g.players[0].hand.is_empty(), "the hand went as it entered");

    g.step = TurnStep::Draw;
    g.fire_step_triggers(TurnStep::Draw);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "one extra card at the draw step");
}

/// Keldon Firebombers cuts everyone back to three lands.
#[test]
fn keldon_firebombers_leaves_everyone_three_lands() {
    let mut g = main_phase();
    for _ in 0..5 {
        g.add_card_to_battlefield(0, catalog::mountain());
        g.add_card_to_battlefield(1, catalog::island());
    }
    let bombers = g.add_card_to_hand(0, catalog::keldon_firebombers());
    cast(&mut g, 0, bombers, None);
    for seat in 0..2 {
        let lands =
            g.battlefield.iter().filter(|c| c.controller == seat && c.definition.is_land()).count();
        assert_eq!(lands, 3, "seat {seat}");
    }
}

/// Keldon Berserker only grows while you're tapped out.
#[test]
fn keldon_berserker_grows_when_you_are_tapped_out() {
    let mut g = two_player_game();
    let berserker = g.add_card_to_battlefield(0, catalog::keldon_berserker());
    let land = g.add_card_to_battlefield(0, catalog::mountain());
    g.clear_sickness(berserker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: berserker, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(berserker).unwrap().power, 2, "you had mana up");

    g.battlefield_find_mut(land).unwrap().tapped = true;
    g.battlefield_find_mut(berserker).unwrap().tapped = false;
    g.attacking.clear();
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: berserker, target: AttackTarget::Player(1) }])
        .expect("attack again");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(berserker).unwrap().power, 5, "tapped out → +3/+0");
}

/// Lesser Gargadon pays a land every time it fights.
#[test]
fn lesser_gargadon_eats_a_land_when_it_attacks() {
    let mut g = two_player_game();
    let gargadon = g.add_card_to_battlefield(0, catalog::lesser_gargadon());
    let land = g.add_card_to_battlefield(0, catalog::mountain());
    g.clear_sickness(gargadon);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: gargadon, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "a land paid for the swing");
}

/// Living Terrain stands a land up as a 5/6 Treefolk.
#[test]
fn living_terrain_animates_the_enchanted_land() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let aura = g.add_card_to_hand(0, catalog::living_terrain());
    cast(&mut g, 0, aura, Some(Target::Permanent(land)));
    let cp = g.computed_permanent(land).expect("computed");
    assert_eq!((cp.power, cp.toughness), (5, 6));
    assert!(cp.card_types.contains(&crabomination::card::CardType::Creature));
    assert!(cp.card_types.contains(&crabomination::card::CardType::Land), "still a land");
}

/// Mana Vapors costs them their untap step.
#[test]
fn mana_vapors_keeps_their_lands_tapped() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(1, catalog::island());
    g.battlefield_find_mut(land).unwrap().tapped = true;
    let vapors = g.add_card_to_hand(0, catalog::mana_vapors());
    cast(&mut g, 0, vapors, Some(Target::Player(1)));
    g.active_player_idx = 1;
    g.step = TurnStep::Untap;
    g.do_untap();
    assert!(g.battlefield_find(land).unwrap().tapped, "it skipped the untap");
    g.do_untap();
    assert!(!g.battlefield_find(land).unwrap().tapped, "only one step");
}

/// Mungha Wurm strands your mana base.
#[test]
fn mungha_wurm_caps_your_untaps_at_one() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mungha_wurm());
    let a = g.add_card_to_battlefield(0, catalog::forest());
    let b = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(a).unwrap().tapped = true;
    g.battlefield_find_mut(b).unwrap().tapped = true;
    g.active_player_idx = 0;
    g.step = TurnStep::Untap;
    g.do_untap();
    let untapped = [a, b].iter().filter(|id| !g.battlefield_find(**id).unwrap().tapped).count();
    assert_eq!(untapped, 1);
}

/// Overburden bounces a land every time a real creature lands.
#[test]
fn overburden_bounces_a_land_per_nontoken_creature() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::overburden());
    let land = g.add_card_to_battlefield(1, catalog::island());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    cast(&mut g, 1, bear, None);
    assert!(g.battlefield_find(land).is_none(), "their land went back");
    assert!(g.players[1].hand.iter().any(|c| c.id == land));
}

/// Panic Attack clears up to three blockers.
#[test]
fn panic_attack_stops_three_blockers() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let attack = g.add_card_to_hand(0, catalog::panic_attack());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: attack,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    for id in [a, b] {
        assert!(
            g.computed_permanent(id).unwrap().keywords.contains(&Keyword::CantBlock),
            "both are locked out"
        );
    }
}

/// Plague Fiend's bite kills unless they pay.
#[test]
fn plague_fiend_kills_what_it_bites() {
    let mut g = two_player_game();
    let fiend = g.add_card_to_battlefield(0, catalog::plague_fiend());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(fiend);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: fiend, target: AttackTarget::Player(1) }])
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, fiend)])).expect("block");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.battlefield_find(blocker).is_none(), "they declined the tax");
}

/// Quicksilver Wall can be bought off by the player it's holding back.
#[test]
fn quicksilver_wall_can_be_bounced_by_any_player() {
    let mut g = main_phase();
    let wall = g.add_card_to_battlefield(0, catalog::quicksilver_wall());
    activate_x(&mut g, 1, wall, 0, None, None);
    assert!(g.battlefield_find(wall).is_none());
    assert!(g.players[0].hand.iter().any(|c| c.id == wall), "back to its owner");
}

/// Nakaya Shade's pump is a rhystic tax.
#[test]
fn nakaya_shade_pumps_when_nobody_pays() {
    let mut g = main_phase();
    let shade = g.add_card_to_battlefield(0, catalog::nakaya_shade());
    activate_x(&mut g, 0, shade, 0, None, None);
    let cp = g.computed_permanent(shade).expect("computed");
    assert_eq!((cp.power, cp.toughness), (2, 2));
}

/// Inflame finishes off everything already damaged this turn.
#[test]
fn inflame_only_hits_already_damaged_creatures() {
    let mut g = main_phase();
    let hurt = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw()); // 6/6
    let fresh = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Permanent(hurt)));
    let inflame = g.add_card_to_hand(0, catalog::inflame());
    cast(&mut g, 0, inflame, None);
    assert_eq!(g.battlefield_find(hurt).unwrap().damage, 5, "3 + 2");
    assert_eq!(g.battlefield_find(fresh).unwrap().damage, 0, "untouched");
}

/// Mine Bearer trades itself for an attacker.
#[test]
fn mine_bearer_trades_for_an_attacker() {
    let mut g = two_player_game();
    let bearer = g.add_card_to_battlefield(0, catalog::mine_bearer());
    g.clear_sickness(bearer);
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(0) }]).expect("attack");
    activate_x(&mut g, 0, bearer, 0, Some(Target::Permanent(attacker)), None);
    assert!(g.battlefield_find(attacker).is_none());
    assert!(g.battlefield_find(bearer).is_none(), "it sacrificed itself");
}

/// Outbreak shrinks a whole creature type; a Swamp can pay for it.
#[test]
fn outbreak_shrinks_the_named_type() {
    let mut g = main_phase();
    let swamp = g.add_card_to_hand(0, catalog::swamp());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let outbreak = g.add_card_to_hand(0, catalog::outbreak());
    script(&mut g, vec![DecisionAnswer::CreatureType(crabomination::card::CreatureType::Bear)]);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: outbreak,
        target: None,
        additional_targets: vec![],
        pitch_card: None,
        mode: None,
        x_value: None,
    })
    .expect("free cast");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == swamp), "the Swamp paid");
    let cp = g.computed_permanent(bear).expect("computed");
    assert_eq!((cp.power, cp.toughness), (1, 1), "Bears got -1/-1");
}

/// Keldon Arsonist trades two of your lands for one of theirs.
#[test]
fn keldon_arsonist_trades_two_lands_for_one() {
    let mut g = main_phase();
    let arsonist = g.add_card_to_battlefield(0, catalog::keldon_arsonist());
    g.add_card_to_battlefield(0, catalog::mountain());
    g.add_card_to_battlefield(0, catalog::mountain());
    let theirs = g.add_card_to_battlefield(1, catalog::island());
    activate_x(&mut g, 0, arsonist, 0, Some(Target::Permanent(theirs)), None);
    assert!(g.battlefield_find(theirs).is_none());
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count(),
        0,
        "both of yours went"
    );
}

/// Noxious Field turns a land into a symmetric sweeper.
#[test]
fn noxious_field_pings_everything() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::swamp());
    let x1 = g.add_card_to_battlefield(1, catalog::mons_goblin_raiders());
    let field = g.add_card_to_hand(0, catalog::noxious_field());
    cast(&mut g, 0, field, Some(Target::Permanent(land)));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("ping");
    drain_stack(&mut g);
    assert!(g.battlefield_find(x1).is_none(), "1/1s die");
    assert_eq!(g.players[0].life, 19);
    assert_eq!(g.players[1].life, 19);
}
