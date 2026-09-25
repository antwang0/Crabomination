//! Commander: the Etali, Primal Conqueror batch (`decks::cmdr_etali`).

use crabomination::card::{
    CardDefinition, CardId, CardInstance, CardType, CounterType, CreatureType, Keyword,
    MayPlayDuration, MayPlayPermission, WardCost,
};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn a_main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn a_flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 12);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn a_advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Ids of the token copies named `name` on the battlefield.
fn a_tokens_named(g: &GameState, name: &str) -> Vec<CardId> {
    g.battlefield
        .iter()
        .filter(|c| c.is_token && c.definition.name == name)
        .map(|c| c.id)
        .collect()
}

fn a_has_haste(g: &GameState, id: CardId) -> bool {
    g.computed_permanent(id).is_some_and(|c| c.keywords().contains(&Keyword::Haste))
}

/// From a main phase, walk to the end step and resolve its triggers.
fn a_to_end_step(g: &mut GameState) {
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    a_advance_to(g, TurnStep::End);
    drain_stack(g);
}

/// Pass priority until combat is over (end-of-combat triggers resolved).
fn a_finish_combat(g: &mut GameState) {
    let mut iters = 0;
    while g.step != TurnStep::PostCombatMain && iters < 50 {
        let _ = g.pass_priority();
        drain_stack(g);
        iters += 1;
    }
}

fn a_activate(g: &mut GameState, card_id: CardId, ability_index: usize, target: Option<Target>) {
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index,
        target,
        additional_targets: Vec::new(),
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Jaxis: {R}, {T}, discard → a hasty copy that is sacrificed at the end step
/// and draws a card as it dies.
#[test]
fn jaxis_the_troublemaker_copy_is_hasty_sacrificed_and_draws() {
    let mut g = a_main_phase();
    stock_libraries(&mut g, 5);
    let jaxis = g.add_card_to_battlefield(0, catalog::jaxis_the_troublemaker());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(jaxis);
    g.add_card_to_hand(0, catalog::island());
    g.players[0].mana_pool.add(Color::Red, 1);
    a_activate(&mut g, jaxis, 0, Some(Target::Permanent(bear)));
    assert!(g.players[0].hand.is_empty(), "the discard was paid");
    let tokens = a_tokens_named(&g, "Grizzly Bears");
    assert_eq!(tokens.len(), 1, "one token copy");
    assert!(a_has_haste(&g, tokens[0]), "the copy has haste");

    a_to_end_step(&mut g);
    assert!(a_tokens_named(&g, "Grizzly Bears").is_empty(), "sacrificed at the end step");
    assert_eq!(g.players[0].hand.len(), 1, "its granted dies-trigger drew a card");
}

/// Rionya: X = 1 + spells cast this turn — one instant before combat makes
/// two hasty copies.
#[test]
fn rionya_fire_dancer_scales_copies_with_spells_cast() {
    let mut g = a_main_phase();
    g.add_card_to_battlefield(0, catalog::rionya_fire_dancer());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, bolt, Target::Player(1));
    // An artifact spell is noncreature but not an instant or sorcery.
    let stone = g.add_card_to_hand(0, catalog::mind_stone());
    a_flood(&mut g, 0);
    cast(&mut g, stone);

    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let tokens = a_tokens_named(&g, "Grizzly Bears");
    assert_eq!(tokens.len(), 2, "X = 1 + one instant");
    assert!(tokens.iter().all(|t| a_has_haste(&g, *t)), "they gain haste");

    a_to_end_step(&mut g);
    assert!(a_tokens_named(&g, "Grizzly Bears").is_empty(), "exiled at the end step");
}

/// Orthion: the small ability makes one hasty copy, the big one five.
#[test]
fn orthion_hero_of_lavabrink_makes_one_or_five_copies() {
    for (ability, expected) in [(0usize, 1usize), (1, 5)] {
        let mut g = a_main_phase();
        let orthion = g.add_card_to_battlefield(0, catalog::orthion_hero_of_lavabrink());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(orthion);
        a_flood(&mut g, 0);
        a_activate(&mut g, orthion, ability, Some(Target::Permanent(bear)));
        let tokens = a_tokens_named(&g, "Grizzly Bears");
        assert_eq!(tokens.len(), expected, "ability {ability}");
        assert!(tokens.iter().all(|t| a_has_haste(&g, *t)), "they gain haste");
        a_to_end_step(&mut g);
        assert!(a_tokens_named(&g, "Grizzly Bears").is_empty(), "sacrificed at the end step");
    }
}

/// Delina: a 15–20 roll makes a copy and rolls again; the second (low) roll
/// makes one more. Both attack, and both are exiled at end of combat.
#[test]
fn delina_wild_mage_high_roll_rolls_again() {
    let mut g = a_main_phase();
    let delina = g.add_card_to_battlefield(0, catalog::delina_wild_mage());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(delina);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::DieRoll(17),
        DecisionAnswer::DieRoll(4),
    ]));
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: delina, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    let tokens: Vec<CardId> =
        g.battlefield.iter().filter(|c| c.is_token).map(|c| c.id).collect();
    assert_eq!(tokens.len(), 2, "17 → copy + reroll, 4 → one more copy");
    for t in &tokens {
        assert!(g.battlefield_find(*t).unwrap().tapped, "tapped");
        assert!(g.attacking().iter().any(|a| a.attacker == *t), "and attacking");
    }
    a_finish_combat(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.is_token), "exiled at end of combat");
}

/// Flamerush Rider copies the other attacker, tapped and attacking, until end
/// of combat.
#[test]
fn flamerush_rider_copies_another_attacker_for_the_combat() {
    let mut g = a_main_phase();
    let rider = g.add_card_to_battlefield(0, catalog::flamerush_rider());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(rider);
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![
        Attack { attacker: rider, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ])
    .expect("attack");
    drain_stack(&mut g);
    let tokens = a_tokens_named(&g, "Grizzly Bears");
    assert_eq!(tokens.len(), 1, "a copy of the other attacker");
    assert!(g.attacking().iter().any(|a| a.attacker == tokens[0]), "attacking");
    a_finish_combat(&mut g);
    assert!(a_tokens_named(&g, "Grizzly Bears").is_empty(), "exiled at end of combat");
}

/// Mirage Phalanx pairs on entry (Soulbond); at the beginning of combat each
/// of the pair makes a hasty copy of itself, exiled at end of combat.
#[test]
fn mirage_phalanx_pairs_and_copies_both_at_combat() {
    let mut g = a_main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let phalanx = g.add_card_to_hand(0, catalog::mirage_phalanx());
    a_flood(&mut g, 0);
    cast(&mut g, phalanx);
    assert_eq!(g.battlefield_find(phalanx).unwrap().soulbond_partner, Some(bear));
    assert_eq!(g.battlefield_find(bear).unwrap().soulbond_partner, Some(phalanx));

    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let copies: Vec<_> = a_tokens_named(&g, "Mirage Phalanx")
        .into_iter()
        .chain(a_tokens_named(&g, "Grizzly Bears"))
        .collect();
    assert_eq!(copies.len(), 2, "one copy of each paired creature");
    assert!(copies.iter().all(|t| a_has_haste(&g, *t)), "the copies have haste");

    a_finish_combat(&mut g);
    assert!(a_tokens_named(&g, "Mirage Phalanx").is_empty(), "exiled at end of combat");
    assert!(a_tokens_named(&g, "Grizzly Bears").is_empty(), "exiled at end of combat");
}

/// Unpaired, Mirage Phalanx makes no copies.
#[test]
fn mirage_phalanx_unpaired_does_nothing_at_combat() {
    let mut g = a_main_phase();
    g.add_card_to_battlefield(0, catalog::mirage_phalanx());
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert!(a_tokens_named(&g, "Mirage Phalanx").is_empty());
}

/// Flameshadow Conjuring: paying {R} as a nontoken creature enters makes a
/// hasty copy that is exiled at the end step.
#[test]
fn flameshadow_conjuring_pays_r_for_a_hasty_copy() {
    let mut g = a_main_phase();
    g.add_card_to_battlefield(0, catalog::flameshadow_conjuring());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast bear");
    // The {R} for the "you may pay" floats until the trigger resolves.
    g.players[0].mana_pool.add(Color::Red, 1);
    drain_stack(&mut g);
    let tokens = a_tokens_named(&g, "Grizzly Bears");
    assert_eq!(tokens.len(), 1, "paid {{R}} → one copy");
    assert!(a_has_haste(&g, tokens[0]), "the copy gains haste");
    a_to_end_step(&mut g);
    assert!(a_tokens_named(&g, "Grizzly Bears").is_empty(), "exiled at the end step");
}

/// Molten Echoes copies a nontoken creature of the chosen type, and only that
/// type.
#[test]
fn molten_echoes_copies_only_the_chosen_type() {
    let mut g = a_main_phase();
    let echoes = g.add_card_to_battlefield(0, catalog::molten_echoes());
    g.battlefield_find_mut(echoes).unwrap().chosen_creature_type = Some(CreatureType::Bear);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, bear);
    let tokens = a_tokens_named(&g, "Grizzly Bears");
    assert_eq!(tokens.len(), 1, "a Bear entered → one copy");
    assert!(a_has_haste(&g, tokens[0]), "the copy gains haste");

    let thopter = g.add_card_to_hand(0, catalog::ornithopter());
    cast(&mut g, thopter);
    assert!(a_tokens_named(&g, "Ornithopter").is_empty(), "a non-Bear doesn't trigger");

    a_to_end_step(&mut g);
    assert!(a_tokens_named(&g, "Grizzly Bears").is_empty(), "exiled at the end step");
}

/// Kindle the Inner Flame: a hasty copy that is sacrificed at the end step.
#[test]
fn kindle_the_inner_flame_makes_a_hasty_temporary_copy() {
    let mut g = a_main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let kindle = g.add_card_to_hand(0, catalog::kindle_the_inner_flame());
    a_flood(&mut g, 0);
    cast_at(&mut g, kindle, Target::Permanent(bear));
    let tokens = a_tokens_named(&g, "Grizzly Bears");
    assert_eq!(tokens.len(), 1);
    assert!(a_has_haste(&g, tokens[0]), "except it has haste");
    a_to_end_step(&mut g);
    assert!(a_tokens_named(&g, "Grizzly Bears").is_empty(), "sacrificed at the end step");
}

/// Chandra, Flameshaper's +1 copies a creature with haste until the end step.
#[test]
fn chandra_flameshaper_plus_one_makes_a_hasty_copy() {
    let mut g = a_main_phase();
    let chandra = g.add_card_to_battlefield(0, catalog::chandra_flameshaper());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: chandra,
        ability_index: 1,
        target: Some(Target::Permanent(bear)),
        x_value: None,
    })
    .expect("+1");
    drain_stack(&mut g);
    let tokens = a_tokens_named(&g, "Grizzly Bears");
    assert_eq!(tokens.len(), 1);
    assert!(a_has_haste(&g, tokens[0]), "except it has haste");
    a_to_end_step(&mut g);
    assert!(a_tokens_named(&g, "Grizzly Bears").is_empty(), "sacrificed at the end step");
}

/// Blade of Selves grants myriad: attacking one of two opponents makes a copy
/// attacking the other.
#[test]
fn blade_of_selves_grants_myriad() {
    let mut g = multi_player_game(3);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blade = g.add_card_to_battlefield(0, catalog::blade_of_selves());
    g.battlefield_find_mut(blade).unwrap().attached_to = Some(bear);
    g.clear_sickness(bear);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack seat 1");
    drain_stack(&mut g);
    let copies = a_tokens_named(&g, "Grizzly Bears");
    assert_eq!(copies.len(), 1, "one copy per non-defending opponent");
    assert!(
        g.attacking()
            .iter()
            .any(|a| a.attacker == copies[0] && a.target == AttackTarget::Player(2)),
        "the copy attacks the other opponent"
    );
}

/// Mirror Box: the legend rule is off for *your* permanents only; each
/// legendary creature you control gets +1/+1; each nontoken creature you
/// control gets +1/+1 per other creature you control with its name (tokens
/// count, but aren't pumped themselves).
#[test]
fn mirror_box_exempts_your_legends_and_scales_by_same_name() {
    let mut g = a_main_phase();
    g.add_card_to_battlefield(0, catalog::mirror_box());
    let a = g.add_card_to_battlefield(0, catalog::krenko_mob_boss());
    let b = g.add_card_to_battlefield(0, catalog::krenko_mob_boss());
    let opp_a = g.add_card_to_battlefield(1, catalog::krenko_mob_boss());
    let opp_b = g.add_card_to_battlefield(1, catalog::krenko_mob_boss());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear_token = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear_token).unwrap().is_token = true;
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let opp_bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.check_state_based_actions();
    assert!(g.battlefield_find(a).is_some() && g.battlefield_find(b).is_some(), "yours survive");
    assert_eq!(
        [opp_a, opp_b].iter().filter(|id| g.battlefield_find(**id).is_some()).count(),
        1,
        "the legend rule still applies to the opponent's pair"
    );
    let pt = |id| g.computed_permanent(id).map(|c| (c.power, c.toughness));
    assert_eq!(pt(a), Some((5, 5)), "Krenko 3/3 +1/+1 legendary +1/+1 for the other Krenko");
    assert_eq!(pt(bear), Some((4, 4)), "two other Grizzly Bears (one a token)");
    assert_eq!(pt(bear2), Some((4, 4)));
    assert_eq!(pt(bear_token), Some((2, 2)), "a token isn't pumped");
    assert_eq!(pt(opp_bear), Some((2, 2)), "the opponent's creatures aren't pumped");
    assert_eq!(pt(opp_bear2), Some((2, 2)));
}

/// Cursed Mirror becomes a hasty copy of a creature until end of turn.
#[test]
fn cursed_mirror_becomes_a_hasty_copy() {
    let mut g = a_main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mirror = g.add_card_to_battlefield(0, catalog::cursed_mirror());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Cards(vec![bear]),
    ]));
    g.fire_self_etb_triggers(mirror, 0);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(mirror).unwrap().definition.name, "Grizzly Bears");
    assert_eq!(g.computed_permanent(mirror).map(|c| (c.power, c.toughness)), Some((2, 2)));
    assert!(a_has_haste(&g, mirror), "except it has haste");
}

/// Hellkite Courser: its ETB puts the chosen one of two commanders onto the
/// battlefield with haste — not a cast, so no tax — and the next end step
/// returns it to the command zone.
#[test]
fn hellkite_courser_borrows_a_commander_until_the_end_step() {
    let mut g = a_main_phase();
    let cmds = g.seat_commanders(0, vec![catalog::krenko_mob_boss(), catalog::grizzly_bears()]);
    let (krenko, bears) = (cmds[0], cmds[1]);
    let courser = g.add_card_to_battlefield(0, catalog::hellkite_courser());
    let cp = g.computed_permanent(courser).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 5));
    assert!(cp.keywords().contains(&Keyword::Flying));
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Cards(vec![bears]),
    ]));
    g.fire_self_etb_triggers(courser, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_some(), "the chosen commander entered");
    assert!(g.players[0].command.iter().any(|c| c.id == krenko), "the other stays home");
    assert!(a_has_haste(&g, bears), "it gains haste");
    assert_eq!(g.commander_cast_count.get(&bears).copied().unwrap_or(0), 0, "not cast: no tax");
    a_to_end_step(&mut g);
    assert!(g.battlefield_find(bears).is_none(), "returned at the end step");
    assert!(g.players[0].command.iter().any(|c| c.id == bears), "to the command zone");

    // CR 400.7 — a commander that dies and is recast before the end step is
    // a new object: the delayed "return it" has nothing to return.
    let mut g = a_main_phase();
    let bears = g.seat_commanders(0, vec![catalog::grizzly_bears()])[0];
    let courser = g.add_card_to_battlefield(0, catalog::hellkite_courser());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_self_etb_triggers(courser, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_some());
    g.remove_from_battlefield_to_graveyard_raw(bears);
    g.check_state_based_actions();
    assert!(g.players[0].command.iter().any(|c| c.id == bears), "back home via the SBA");
    a_flood(&mut g, 0);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("recast the commander");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_some());
    a_to_end_step(&mut g);
    assert!(g.battlefield_find(bears).is_some(), "the recast commander stays");
}

/// Sanctum of Eternity returns your commander to your hand on your turn
/// (declining the CR 903.9b command-zone redirect).
#[test]
fn sanctum_of_eternity_returns_commander_to_hand() {
    let mut g = a_main_phase();
    let cmd = g.seat_commanders(0, vec![catalog::krenko_mob_boss()])[0];
    a_flood(&mut g, 0);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("cast commander");
    drain_stack(&mut g);
    assert!(g.battlefield_find(cmd).is_some(), "commander on the battlefield");
    let sanctum = g.add_card_to_battlefield(0, catalog::sanctum_of_eternity());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    a_activate(&mut g, sanctum, 1, Some(Target::Permanent(cmd)));
    assert!(g.players[0].hand.iter().any(|c| c.id == cmd), "back in hand");
}

/// Passionate Archaeologist: with your commander out, casting a spell from
/// exile deals its mana value to an opponent.
#[test]
fn passionate_archaeologist_burns_for_exile_casts() {
    let mut g = a_main_phase();
    stock_libraries(&mut g, 5);
    let cmd = g.seat_commanders(0, vec![catalog::krenko_mob_boss()])[0];
    a_flood(&mut g, 0);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("cast commander");
    drain_stack(&mut g);
    g.add_card_to_battlefield(0, catalog::passionate_archaeologist());

    let div = g.next_id();
    let mut card = CardInstance::new(div, catalog::divination(), 0);
    card.controller = 0;
    card.may_play_until = Some(MayPlayPermission { cast_only: false,
        player: 0,
        granted_turn: g.turn_number,
        duration: MayPlayDuration::EndOfThisTurn,
        exile_after: false,
        miracle: false,
        pay_life: false,
    });
    g.exile.push(card);
    let life = g.players[1].life;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: div,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from exile");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "Divination's mana value (3) to the opponent");
}

fn b_main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn b_flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 12);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

/// `seat` casts `id` (no drain) — lets a second spell go on top of it.
fn b_cast_by(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
}

/// Stamp a free "you may cast it" permission on an exiled card for seat 0.
fn b_grant_free(g: &mut GameState, id: CardId) {
    let turn = g.turn_number;
    let c = g.exile.iter_mut().find(|c| c.id == id).expect("card is in exile");
    c.may_play_until = Some(MayPlayPermission { cast_only: false,
        player: 0,
        granted_turn: turn,
        duration: MayPlayDuration::EndOfThisTurn,
        exile_after: false,
        miracle: false,
        pay_life: false,
    });
}

fn b_cast_from_exile(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from exile");
    drain_stack(g);
}

fn b_activate(g: &mut GameState, id: CardId, index: usize) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
}

fn b_in_exile_playable(g: &GameState, id: CardId) -> bool {
    g.exile.iter().any(|c| c.id == id && c.may_play_until.is_some())
}

/// Seat 0 swings `attackers` at seat 1, no blocks, and combat runs out.
fn b_swing(g: &mut GameState, attackers: &[CardId]) {
    for &a in attackers {
        g.clear_sickness(a);
    }
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(
        attackers
            .iter()
            .map(|&a| Attack { attacker: a, target: AttackTarget::Player(1) })
            .collect(),
    )
    .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

// ── Tests ─────────────────────────────────────────────────────────────────

/// Etali's ETB exiles down to a nonland card in EVERY library and casts both
/// for free; {9}{G/P} flips it to the 11/11 back face.
#[test]
fn etali_primal_conqueror_casts_each_players_first_nonland_then_transforms() {
    let mut g = b_main_phase();
    let my_land = g.add_card_to_library(0, catalog::island());
    let my_bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let their_land = g.add_card_to_library(1, catalog::forest());
    let their_lion = g.add_card_to_library(1, catalog::savannah_lions());
    for seat in 0..2 {
        g.add_card_to_library(seat, catalog::island());
    }
    let etali = g.add_card_to_hand(0, catalog::etali_primal_conqueror());
    b_flood(&mut g, 0);
    cast(&mut g, etali);

    for land in [my_land, their_land] {
        assert!(g.exile.iter().any(|c| c.id == land), "the land passed over stays exiled");
    }
    for spell in [my_bear, their_lion] {
        let c = g.battlefield_find(spell).expect("free-cast off the exile");
        assert_eq!(c.controller, 0, "Etali's controller casts every hit");
    }

    b_flood(&mut g, 0);
    b_activate(&mut g, etali, 0);
    drain_stack(&mut g);
    let cp = g.computed_permanent(etali).expect("still on the battlefield");
    assert_eq!((cp.power, cp.toughness), (11, 11), "transformed into Primal Sickness");
    assert!(cp.keywords().contains(&Keyword::Indestructible));
}

/// Nalfeshnee copies a spell cast from exile. A permanent spell's copy becomes
/// a token.
#[test]
fn nalfeshnee_copies_spells_cast_from_exile() {
    let mut g = b_main_phase();
    g.add_card_to_battlefield(0, catalog::nalfeshnee());
    let bolt = g.add_card_to_exile(0, catalog::lightning_bolt());
    b_grant_free(&mut g, bolt);
    b_cast_from_exile(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 14, "the Bolt and its copy both hit");

    let bear = g.add_card_to_exile(0, catalog::grizzly_bears());
    b_grant_free(&mut g, bear);
    b_cast_from_exile(&mut g, bear, None);
    let bears: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Grizzly Bears" && c.controller == 0)
        .collect();
    assert_eq!(bears.len(), 2, "the creature spell was copied");
    assert_eq!(bears.iter().filter(|c| c.is_token).count(), 1, "the copy is a token");
}

/// Keeper of Secrets pings only for spells cast from somewhere other than the
/// hand, for their mana value.
#[test]
fn keeper_of_secrets_punishes_non_hand_casts() {
    let mut g = b_main_phase();
    g.add_card_to_battlefield(0, catalog::keeper_of_secrets());
    b_flood(&mut g, 0);
    let from_hand = g.add_card_to_hand(0, catalog::lightning_bolt());
    b_cast_by(&mut g, 0, from_hand, Some(Target::Player(1)));
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "a hand cast doesn't trigger Keeper");

    let from_exile = g.add_card_to_exile(0, catalog::lightning_bolt());
    b_grant_free(&mut g, from_exile);
    b_cast_from_exile(&mut g, from_exile, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 13, "Bolt for 3 plus Keeper for its mana value 1");
}

/// Eternal Scourge exiles itself when an opponent targets it, then comes back
/// from exile for {3}.
#[test]
fn eternal_scourge_dodges_to_exile_and_returns() {
    let mut g = b_main_phase();
    let scourge = g.add_card_to_battlefield(0, catalog::eternal_scourge());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    b_cast_by(&mut g, 1, bolt, Some(Target::Permanent(scourge)));
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == scourge), "targeted by an opponent -> exiled");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == scourge), "the Bolt never landed");

    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    b_activate(&mut g, scourge, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(scourge).is_some(), "back from exile");
}

/// Two creatures connecting make ONE Treasure. Sacrificing it impulse-exiles
/// the top card.
#[test]
fn professional_face_breaker_one_treasure_per_combat_damage_batch() {
    let mut g = b_main_phase();
    let fb = g.add_card_to_battlefield(0, catalog::professional_face_breaker());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let top = g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::island());
    b_swing(&mut g, &[fb, bear]);
    assert_eq!(g.players[1].life, 16);
    let treasures = |g: &GameState| {
        g.battlefield.iter().filter(|c| c.definition.name == "Treasure" && c.controller == 0).count()
    };
    assert_eq!(treasures(&g), 1, "one batch of combat damage -> one Treasure");

    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    b_activate(&mut g, fb, 0);
    drain_stack(&mut g);
    assert_eq!(treasures(&g), 0, "the Treasure paid the cost");
    assert!(b_in_exile_playable(&g, top), "the top card is exiled and playable this turn");
}

/// Sacrificing a land to Evendo's own mana ability exiles the top card, and
/// on your turn that card is playable for its mana cost.
#[test]
fn evendo_brushrazer_land_sac_exiles_a_playable_card() {
    let mut g = b_main_phase();
    let ev = g.add_card_to_battlefield(0, catalog::evendo_brushrazer());
    g.clear_sickness(ev);
    let land = g.add_card_to_battlefield(0, catalog::mountain());
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    b_activate(&mut g, ev, 0);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 2, "{{T}}, sac a land: {{R}}{{R}}");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == land), "the Mountain was sacrificed");
    assert!(b_in_exile_playable(&g, top), "nontoken sacrifice -> exiled and playable");

    g.players[0].mana_pool.add(Color::Green, 1);
    b_cast_from_exile(&mut g, top, None);
    assert!(g.battlefield_find(top).is_some(), "cast from exile for its own {{1}}{{G}}");
}

/// Tibalt's Trickery counters, mills the random count (AutoDecider's d3 = 2),
/// then casts the next nonland card for free and bottoms the passed-over land.
#[test]
fn tibalts_trickery_counters_then_cascades_its_controller() {
    let mut g = b_main_phase();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    b_flood(&mut g, 0);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let trick = g.add_card_to_hand(0, catalog::tibalts_trickery());
    b_cast_by(&mut g, 0, bolt, Some(Target::Player(1)));
    b_cast_by(&mut g, 0, trick, Some(Target::Permanent(bolt)));
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, 20, "the Bolt was countered");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt));
    let milled = g.players[0].graveyard.iter().filter(|c| c.definition.name == "Island").count();
    assert_eq!(milled, 2, "the die's midpoint milled two");
    assert!(g.battlefield_find(bear).is_some(), "the next nonland card was cast for free");
    assert_eq!(g.players[0].library.len(), 1, "the exiled Island went to the bottom");
}

/// Delayed Blast Fireball deals 2 from hand and 5 when foretold (cast from
/// exile), to each opponent and their creatures only.
#[test]
fn delayed_blast_fireball_is_bigger_from_exile() {
    let setup = || {
        let mut g = b_main_phase();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
        b_flood(&mut g, 0);
        (g, mine, bear, wurm)
    };

    let (mut g, mine, bear, wurm) = setup();
    let fb = g.add_card_to_hand(0, catalog::delayed_blast_fireball());
    b_cast_by(&mut g, 0, fb, None);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
    assert_eq!(g.players[0].life, 20, "only opponents are hit");
    assert!(g.battlefield_find(bear).is_none(), "2 damage kills the 2/2");
    assert!(g.battlefield_find(wurm).is_some(), "the 6/4 survives 2");
    assert!(g.battlefield_find(mine).is_some(), "your creatures are spared");

    let (mut g, _, _, wurm) = setup();
    let fb = g.add_card_to_exile(0, catalog::delayed_blast_fireball());
    g.exile.iter_mut().find(|c| c.id == fb).unwrap().face_down = true;
    g.perform_action(GameAction::CastForetold {
        card_id: fb,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the foretold Fireball");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 15, "cast from exile: 5 damage");
    assert!(g.battlefield_find(wurm).is_none(), "5 damage kills the 6/4");
}

/// Call Forth the Tempest cascades twice, then burns the opponents' creatures
/// for the other spells cast this turn (Bears + Divination here).
#[test]
fn call_forth_the_tempest_double_cascades_then_sweeps() {
    let mut g = b_main_phase();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::divination());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    // Total mana value of the other spells is 2 + 3 = 5: a 5-toughness body
    // dies, a 6-toughness one survives.
    let courser = g.add_card_to_battlefield(1, catalog::hellkite_courser());
    let demon = g.add_card_to_battlefield(1, catalog::nalfeshnee());
    let tempest = g.add_card_to_hand(0, catalog::call_forth_the_tempest());
    b_flood(&mut g, 0);
    let hand_before = g.players[0].hand.len();
    cast(&mut g, tempest);

    assert!(g.battlefield_find(bear).is_some(), "first cascade cast the Bears");
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2, "second cascade cast Divination");
    assert!(g.battlefield_find(wurm).is_none(), "the sweep kills the opposing 6/4");
    assert!(g.battlefield_find(courser).is_none(), "5 damage kills a 6/5");
    assert!(g.battlefield_find(demon).is_some(), "5 damage doesn't kill a 4/6");
    assert!(g.battlefield_find(bear).is_some(), "your own creatures aren't hit");
}

/// Escape to the Wilds exiles five playable cards and grants a second land
/// drop, so two exiled lands can be played this turn.
#[test]
fn escape_to_the_wilds_exiles_five_and_adds_a_land_drop() {
    let mut g = b_main_phase();
    let island = g.add_card_to_library(0, catalog::island());
    let forest = g.add_card_to_library(0, catalog::forest());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::escape_to_the_wilds());
    b_flood(&mut g, 0);
    cast(&mut g, spell);
    let playable = g.exile.iter().filter(|c| c.may_play_until.is_some()).count();
    assert_eq!(playable, 5, "the top five are exiled and playable");
    assert_eq!(g.players[0].library.len(), 1);
    assert_eq!(g.players[0].extra_land_plays, 1);

    g.perform_action(GameAction::PlayLand(island)).expect("first land from exile");
    g.perform_action(GameAction::PlayLand(forest)).expect("the extra land drop");
    assert!(g.battlefield_find(island).is_some() && g.battlefield_find(forest).is_some());
}

/// Rishkar's Expertise draws off the biggest power, then free-casts a spell
/// with mana value 5 or less from hand (AutoDecider takes the biggest one).
#[test]
fn rishkars_expertise_draws_greatest_power_and_casts_free() {
    let mut g = b_main_phase();
    g.add_card_to_battlefield(0, catalog::craw_wurm()); // power 6
    for _ in 0..8 {
        g.add_card_to_library(0, catalog::island());
    }
    let angel = g.add_card_to_hand(0, catalog::serra_angel()); // MV 5
    let big = g.add_card_to_hand(0, catalog::craw_wurm()); // MV 6 — too big
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::rishkars_expertise());
    b_flood(&mut g, 0);
    cast(&mut g, spell);
    assert!(g.battlefield_find(angel).is_some(), "the MV-5 Angel was cast for free");
    assert!(g.players[0].hand.iter().any(|c| c.id == big), "MV 6 isn't eligible");
    // 3 cards in hand after the cast, +6 drawn, -1 Angel.
    assert_eq!(g.players[0].hand.len(), 8);
}

/// World War Hulk: I free-casts a green creature, II adds three +1/+1
/// counters, III doubles it and grants trample.
#[test]
fn world_war_hulk_chapters_cast_grow_and_double() {
    let mut g = b_main_phase();
    let wurm = g.add_card_to_hand(0, catalog::craw_wurm()); // green 6/4
    let angel = g.add_card_to_hand(0, catalog::serra_angel()); // white — not eligible
    let saga = g.add_card_to_hand(0, catalog::world_war_hulk());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, saga);
    assert!(g.battlefield_find(wurm).is_some(), "chapter I cast the green creature for free");
    assert!(g.players[0].hand.iter().any(|c| c.id == angel), "a white creature isn't eligible");

    g.saga_advance(saga);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(wurm).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);

    g.saga_advance(saga);
    drain_stack(&mut g);
    let cp = g.computed_permanent(wurm).unwrap();
    assert_eq!((cp.power, cp.toughness), (18, 14), "9/7 doubled");
    assert!(cp.keywords().contains(&Keyword::Trample));
}

fn c_main(seats: usize) -> GameState {
    let mut g = multi_player_game(seats);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// Play `def` as seat 0's land drop and resolve anything it put on the stack.
fn c_play_land(g: &mut GameState, def: CardDefinition) -> CardId {
    let id = g.add_card_to_hand(0, def);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].lands_played_this_turn = 0;
    g.perform_action(GameAction::PlayLand(id)).expect("play land");
    drain_stack(g);
    id
}

/// Activate seat 0's ability `index` on `id` (no target) and resolve.
fn c_activate(g: &mut GameState, id: CardId, index: usize) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn c_cast_mode(g: &mut GameState, id: CardId, mode: Option<usize>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

// ── Heartwood Crafter // Soul Tether ────────────────────────────────────────

/// Heartwood Crafter enters prepared; casting its Soul Tether copy mints a
/// red-green Heartwood artifact and unprepares the Crafter.
#[test]
fn heartwood_crafter_enters_prepared_and_soul_tether_makes_a_heartwood() {
    let mut g = c_main(2);
    let crafter = g.add_card_to_hand(0, catalog::heartwood_crafter());
    g.players[0].mana_pool.add(Color::Green, 1);
    c_cast_mode(&mut g, crafter, None);
    assert_eq!(
        g.battlefield_find(crafter).unwrap().counter_count(CounterType::Prepared),
        1,
        "enters prepared"
    );

    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: crafter,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Soul Tether castable for {2}{R/G}");
    drain_stack(&mut g);

    let token = g
        .battlefield
        .iter()
        .find(|c| c.is_token && c.definition.name == "Heartwood")
        .expect("Heartwood token");
    assert!(token.definition.card_types.contains(&CardType::Artifact));
    assert!(!token.definition.card_types.contains(&CardType::Creature));
    assert_eq!(token.definition.activated_abilities.len(), 2, "{{T}}: Add {{R}} or {{G}}");
    assert_eq!(
        g.battlefield_find(crafter).unwrap().counter_count(CounterType::Prepared),
        0,
        "casting the copy unprepares it"
    );
}

// ── Rituals / mana creatures ────────────────────────────────────────────────

/// Dragon's Desire counts every opponent's artifacts, not its caster's.
#[test]
fn dragons_desire_adds_red_per_opponent_artifact() {
    let mut g = c_main(3);
    g.add_card_to_battlefield(0, catalog::sol_ring()); // own — not counted
    g.add_card_to_battlefield(1, catalog::sol_ring());
    g.add_card_to_battlefield(1, catalog::ornithopter());
    g.add_card_to_battlefield(2, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::dragons_desire());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    c_cast_mode(&mut g, id, None);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3, "three opposing artifacts");
}

/// Somberwald Sage's three mana is creature-only.
#[test]
fn somberwald_sage_makes_three_creature_only_mana() {
    let mut g = c_main(2);
    let sage = g.add_card_to_battlefield(0, catalog::somberwald_sage());
    g.clear_sickness(sage);
    c_activate(&mut g, sage, 0);
    assert_eq!(g.players[0].mana_pool.restricted_total(), 3, "three restricted mana");
    assert_eq!(g.players[0].mana_pool.total(), 0, "none of it is free mana");

    let soldier = g.add_card_to_hand(0, catalog::yotian_soldier());
    c_cast_mode(&mut g, soldier, None);
    assert!(g.battlefield_find(soldier).is_some(), "paid a creature spell");
    assert_eq!(g.players[0].mana_pool.restricted_total(), 0);
}

/// Gwenna grows and untaps when you cast a power-5+ creature spell.
#[test]
fn gwenna_eyes_of_gaea_untaps_and_grows_on_a_big_creature() {
    let mut g = c_main(2);
    let gwenna = g.add_card_to_battlefield(0, catalog::gwenna_eyes_of_gaea());
    g.battlefield_find_mut(gwenna).unwrap().tapped = true;

    // A 2-power creature doesn't trigger.
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    c_cast_mode(&mut g, bears, None);
    assert!(g.battlefield_find(gwenna).unwrap().tapped, "power 2: no trigger");

    let wurm = g.add_card_to_hand(0, catalog::craw_wurm()); // 6/4
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    c_cast_mode(&mut g, wurm, None);
    let gw = g.battlefield_find(gwenna).unwrap();
    assert!(!gw.tapped, "untapped by the trigger");
    assert_eq!(gw.counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Open the Omenpaths: mode 0 floats four creature-only mana; mode 1 pumps
/// the team +1/+0.
#[test]
fn open_the_omenpaths_mana_mode_and_pump_mode() {
    let mut g = c_main(2);
    let first = g.add_card_to_hand(0, catalog::open_the_omenpaths());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    c_cast_mode(&mut g, first, Some(0));
    assert_eq!(g.players[0].mana_pool.restricted_total(), 4, "two + two restricted mana");

    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let second = g.add_card_to_hand(0, catalog::open_the_omenpaths());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    c_cast_mode(&mut g, second, Some(1));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "+1/+0");
    assert_eq!(g.computed_permanent(bear).unwrap().toughness, 2);
    assert_eq!(g.computed_permanent(theirs).unwrap().power, 2, "only yours");
}

/// Infernal Plunge sacrifices a creature as a cost and adds {R}{R}{R}.
#[test]
fn infernal_plunge_sacrifices_for_three_red() {
    let mut g = c_main(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::infernal_plunge());
    g.players[0].mana_pool.add(Color::Red, 1);
    c_cast_mode(&mut g, id, None);
    assert!(g.battlefield_find(bear).is_none(), "creature sacrificed");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3);
}

/// Reckless Barbarian and Tinder Wall: "Sacrifice this creature: Add {R}{R}."
#[test]
fn reckless_barbarian_and_tinder_wall_sacrifice_for_two_red() {
    let cases: [fn() -> CardDefinition; 2] =
        [catalog::reckless_barbarian, catalog::tinder_wall];
    for def in cases {
        let mut g = c_main(2);
        let name = def().name;
        let id = g.add_card_to_battlefield(0, def());
        c_activate(&mut g, id, 0);
        assert!(g.battlefield_find(id).is_none(), "{name} sacrificed itself");
        assert_eq!(g.players[0].mana_pool.amount(Color::Red), 2, "{name} added {{R}}{{R}}");
    }
    assert!(catalog::tinder_wall().keywords.contains(&Keyword::Defender));
}

// ── Cost reducers ───────────────────────────────────────────────────────────

/// Radagast: the first creature spell each turn costs {2} less; the second
/// pays full price.
#[test]
fn radagast_of_rhosgobel_discounts_the_first_creature_spell() {
    // On the opponent's turn: the first creature spell has flash and costs
    // {2} less; the second has neither.
    let mut g = c_main(2);
    g.add_card_to_battlefield(0, catalog::radagast_of_rhosgobel());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 0;
    let first = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    c_cast_mode(&mut g, first, None);
    assert!(g.battlefield_find(first).is_some(), "{{1}}{{G}} bears for {{G}} at instant speed");

    g.priority.player_with_priority = 0;
    let second = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 4);
    let res = g.perform_action(GameAction::CastSpell {
        card_id: second,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(res.is_err(), "the second creature spell has no flash");
}

/// Shadow in the Warp: discounts your first creature spell and burns an
/// opponent for their first noncreature spell each turn — only the first.
#[test]
fn shadow_in_the_warp_discounts_and_punishes_first_noncreature_spell() {
    let mut g = c_main(2);
    g.add_card_to_battlefield(0, catalog::shadow_in_the_warp());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    c_cast_mode(&mut g, bears, None);
    assert!(g.battlefield_find(bears).is_some(), "bears for {{G}}");

    let life = g.players[1].life;
    for _ in 0..2 {
        let ritual = g.add_card_to_hand(1, catalog::dark_ritual());
        g.players[1].mana_pool.add(Color::Black, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: ritual,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("opponent casts Dark Ritual");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, life - 2, "only the first noncreature spell burns");
}

// ── Hexing Squelcher ────────────────────────────────────────────────────────

/// Hexing Squelcher is uncounterable, has Ward—Pay 2 life, and shares the
/// ward with your other creatures only.
#[test]
fn hexing_squelcher_grants_life_ward_to_your_other_creatures() {
    let mut g = c_main(2);
    let sq = g.add_card_to_battlefield(0, catalog::hexing_squelcher());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ward = Keyword::Ward(WardCost::Life(2));
    let sqp = g.computed_permanent(sq).unwrap();
    assert!(sqp.keywords().contains(&ward));
    assert!(sqp.keywords().contains(&Keyword::CantBeCountered));
    assert!(g.computed_permanent(mine).unwrap().keywords().contains(&ward), "other creature");
    assert!(!g.computed_permanent(theirs).unwrap().keywords().contains(&ward), "not theirs");
}

// ── _____ Goblin ────────────────────────────────────────────────────────────

/// _____ Goblin: the sticker fills the blank and adds {R} per unique vowel;
/// the next Goblin can't reuse it (the ballot's best is then a 3-vowel
/// word); declining keeps the blank name and adds nothing.
#[test]
fn blank_goblin_name_sticker_names_it_and_adds_red_per_unique_vowel() {
    let mut g = c_main(2);
    // Stand-in sheets 0-2: Wobbly Audacious Grim / Eerie Fabulous Spry /
    // Quizzical Bold Tiny.
    g.set_sticker_sheets(0, [0, 1, 2]);
    let cast_goblin = |g: &mut GameState| {
        let id = g.add_card_to_hand(0, catalog::blank_goblin());
        g.players[0].mana_pool = Default::default();
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        c_cast_mode(g, id, None);
        let name = g.battlefield_find(id).expect("on the battlefield").definition.name;
        (id, name, g.players[0].mana_pool.amount(Color::Red))
    };

    // The auto decider takes the ballot's first (richest) sticker.
    let (first, name, red) = cast_goblin(&mut g);
    assert_eq!((name, red), ("Audacious Goblin", 4), "A U I O");
    let cp = g.computed_permanent(first).unwrap();
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Guest), "Goblin Guest");
    assert_eq!((cp.power, cp.toughness), (2, 2));

    // Audacious is on an object seat 0 owns, so it's gone from the ballot.
    let (_, name, red) = cast_goblin(&mut g);
    assert_eq!((name, red), ("Fabulous Goblin", 3), "the best sticker left");

    // "No sticker" is the ballot's last entry.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(99)]));
    let (_, name, red) = cast_goblin(&mut g);
    assert_eq!((name, red), ("_____ Goblin", 0), "declined: no name, no mana");
}

// ── Lands ───────────────────────────────────────────────────────────────────

/// The conditional-tapped R/G lands, each on both sides of its condition,
/// plus Wooded Ridgeline (always tapped).
#[test]
fn cmdr_etali_rg_lands_enter_tapped_on_their_conditions() {
    type Setup = fn(&mut GameState);
    let none: Setup = |_| {};
    let mountain_in_play: Setup = |g| {
        g.add_card_to_battlefield(0, catalog::mountain());
    };
    let two_lands: Setup = |g| {
        g.add_card_to_battlefield(0, catalog::mountain());
        g.add_card_to_battlefield(0, catalog::forest());
    };
    let forest_in_hand: Setup = |g| {
        g.add_card_to_hand(0, catalog::forest());
    };
    let island_in_hand: Setup = |g| {
        g.add_card_to_hand(0, catalog::island());
    };
    /// (land factory, seat count, board setup, does it enter tapped).
    type SeatTapCase = (fn() -> CardDefinition, usize, Setup, bool);
    let cases: [SeatTapCase; 11] = [
        (catalog::arena_of_glory, 2, none, true),
        (catalog::arena_of_glory, 2, mountain_in_play, false),
        (catalog::spire_garden, 2, none, true),
        (catalog::spire_garden, 3, none, false),
        (catalog::spire_garden, 4, none, false),
        (catalog::rockfall_vale, 2, mountain_in_play, true),
        (catalog::rockfall_vale, 2, two_lands, false),
        (catalog::game_trail, 2, island_in_hand, true),
        (catalog::game_trail, 2, forest_in_hand, false),
        (catalog::wooded_ridgeline, 2, two_lands, true),
        (catalog::wooded_ridgeline, 3, none, true),
    ];
    for (def, seats, setup, tapped) in cases {
        let mut g = c_main(seats);
        setup(&mut g);
        let name = def().name;
        let id = c_play_land(&mut g, def());
        assert_eq!(
            g.battlefield_find(id).unwrap().tapped,
            tapped,
            "{name} with {seats} seats"
        );
    }
    let ridge = catalog::wooded_ridgeline();
    assert!(ridge.subtypes.land_types.contains(&crabomination::card::LandType::Mountain));
    assert!(ridge.subtypes.land_types.contains(&crabomination::card::LandType::Forest));
}

/// Arena of Glory's exert ability: {R}, {T} → {R}{R} with the haste rider,
/// and the land won't untap next untap step.
#[test]
fn arena_of_glory_exerts_for_two_hasty_red() {
    let mut g = c_main(2);
    let arena = g.add_card_to_battlefield(0, catalog::arena_of_glory());
    g.players[0].mana_pool.add(Color::Red, 1);
    c_activate(&mut g, arena, 1);
    assert_eq!(g.players[0].mana_pool.rider_amount(Color::Red), 2, "{{R}}{{R}} with the haste rider");
    let a = g.battlefield_find(arena).unwrap();
    assert!(a.tapped);
    assert!(a.skip_next_untap, "exerted");
}

/// Fire-Lit Thicket filters {R/G} into two mana of R/G, or taps for {C}.
#[test]
fn fire_lit_thicket_filters_hybrid_into_two_rg() {
    let mut g = c_main(2);
    let land = g.add_card_to_battlefield(0, catalog::fire_lit_thicket());
    c_activate(&mut g, land, 0);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1, "{{T}}: Add {{C}}");

    let land2 = g.add_card_to_battlefield(0, catalog::fire_lit_thicket());
    g.players[0].mana_pool.add(Color::Green, 1);
    c_activate(&mut g, land2, 1);
    let pool = &g.players[0].mana_pool;
    assert_eq!(pool.amount(Color::Red) + pool.amount(Color::Green), 2, "two R/G pips");
    assert!(g.battlefield_find(land2).unwrap().tapped);
}

fn main_phase_d() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood_d(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 12);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

/// Pia and Formidable Speaker share "you may discard a card; if you do,
/// tutor a [type] card to hand": the discard is the gate, the find lands in
/// hand, the fodder in the graveyard.
#[test]
fn pia_and_formidable_speaker_discard_to_tutor() {
    /// (the discard-to-tutor card, the card it goes and finds).
    type TutorCase = (fn() -> CardDefinition, fn() -> CardDefinition);
    let cases: [TutorCase; 2] = [
        (catalog::pia_aether_ascetic, catalog::oblivion_ring),
        (catalog::formidable_speaker, catalog::grizzly_bears),
    ];
    for (tutor, find) in cases {
        let mut g = main_phase_d();
        let want = g.add_card_to_library(0, find());
        g.add_card_to_library(0, catalog::forest());
        let fodder = g.add_card_to_hand(0, catalog::mountain());
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Bool(true),
            DecisionAnswer::Search(Some(want)),
        ]));
        g.move_card_to_battlefield_for_test(0, tutor());
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == want), "{}: tutored to hand", tutor().name);
        assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder), "{}: discarded", tutor().name);
    }
}

/// Formidable Speaker — {1}, {T}: untap another target permanent.
#[test]
fn formidable_speaker_untaps_another_permanent() {
    let mut g = main_phase_d();
    let speaker = g.add_card_to_battlefield(0, catalog::formidable_speaker());
    g.clear_sickness(speaker);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: speaker,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate the untap");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bear).unwrap().tapped, "bear untapped");
    assert!(g.battlefield_find(speaker).unwrap().tapped, "speaker tapped for the cost");
}

/// Curse-Marred Demon tutors any card, then discards a card at random: the
/// hand size is unchanged and one card is in the graveyard.
#[test]
fn curse_marred_demon_tutors_then_discards_at_random() {
    let mut g = main_phase_d();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bolt))]));
    let demon = g.move_card_to_battlefield_for_test(0, catalog::curse_marred_demon());
    drain_stack(&mut g);
    assert!(g.battlefield_find(demon).is_some());
    assert_eq!(g.players[0].hand.len(), 1, "one found, one discarded");
    assert_eq!(g.players[0].graveyard.len(), 1, "a card was discarded");
    assert_eq!(g.players[0].library.len(), 2, "the bolt left the library");
}

/// Temur Sabertooth bounces another creature and gains indestructible.
#[test]
fn temur_sabertooth_bounces_and_gains_indestructible() {
    let mut g = main_phase_d();
    let cat = g.add_card_to_battlefield(0, catalog::temur_sabertooth());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: cat,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate Sabertooth");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "bear returned to hand");
    assert!(
        g.computed_permanent(cat).unwrap().keywords().contains(&Keyword::Indestructible),
        "Sabertooth is indestructible this turn"
    );
}

/// Kogla and Yidaro's fight mode kills an opposing creature on entry.
#[test]
fn kogla_and_yidaro_etb_fights() {
    let mut g = main_phase_d();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let kogla = g.add_card_to_hand(0, catalog::kogla_and_yidaro());
    flood_d(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    cast(&mut g, kogla);
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "the 2/2 lost the fight");
    assert!(g.battlefield_find(kogla).is_some(), "the 7/7 survives");
}

/// "{2}{R}{G}, Discard this card: destroy up to one artifact/enchantment,
/// shuffle this card into your library from your graveyard, draw a card."
#[test]
fn kogla_and_yidaro_discards_to_destroy_and_shuffles_back() {
    let mut g = main_phase_d();
    stock_libraries(&mut g, 3);
    let kogla = g.add_card_to_hand(0, catalog::kogla_and_yidaro());
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kogla,
        ability_index: 0,
        target: Some(Target::Permanent(stone)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate from hand");
    drain_stack(&mut g);
    assert!(g.battlefield_find(stone).is_none(), "artifact destroyed");
    // Shuffled in *before* the draw, so the draw may find Kogla itself.
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == kogla), "not left in the graveyard");
    assert!(
        g.players[0].library.iter().chain(g.players[0].hand.iter()).any(|c| c.id == kogla),
        "shuffled back into the library"
    );
    assert_eq!(g.players[0].hand.len(), 1, "drew a card");
    assert_eq!(g.players[0].library.len(), 3, "3 Forests + Kogla - 1 draw");
}

/// Valakut Awakening: cards chosen from hand go back, then draw that many
/// plus one. The land back enters tapped.
#[test]
fn valakut_awakening_cycles_hand_and_back_enters_tapped() {
    let mut g = main_phase_d();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let f1 = g.add_card_to_hand(0, catalog::forest());
    let f2 = g.add_card_to_hand(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::valakut_awakening());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![f1, f2])]));
    cast(&mut g, spell);
    assert_eq!(g.players[0].hand.len(), 3, "two put back, three drawn");
    assert_eq!(g.players[0].library.len(), 4, "5 + 2 - 3");

    let mut g = main_phase_d();
    let id = g.add_card_to_hand(0, catalog::valakut_awakening());
    g.perform_action(GameAction::PlayLandBack(id)).expect("play the land back");
    drain_stack(&mut g);
    let land = g.battlefield_find(id).expect("land on battlefield");
    assert_eq!(land.definition.name, "Valakut Stoneforge");
    assert!(land.tapped, "enters tapped");
}

/// Last March of the Ents draws by the greatest toughness (Serra Angel, 4)
/// and deploys every creature card from hand.
#[test]
fn last_march_of_the_ents_draws_by_toughness_and_deploys() {
    let mut g = main_phase_d();
    g.add_card_to_battlefield(0, catalog::serra_angel());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.add_card_to_library(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::last_march_of_the_ents());
    flood_d(&mut g, 0);
    cast(&mut g, spell);
    let bears = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Grizzly Bears")
        .count();
    assert_eq!(bears, 4, "drew four, put all four onto the battlefield");
    assert_eq!(g.players[0].library.len(), 1, "the Forest stays");
    assert!(catalog::last_march_of_the_ents().keywords.contains(&Keyword::CantBeCountered));
}

/// Cream of the Crop: a power-3 creature entering lets you look at the top
/// three and keep one on top, bottoming the rest.
#[test]
fn cream_of_the_crop_keeps_one_of_top_x_on_top() {
    let mut g = main_phase_d();
    g.add_card_to_battlefield(0, catalog::cream_of_the_crop());
    let a = g.add_card_to_library(0, catalog::island());
    let b = g.add_card_to_library(0, catalog::island());
    let c = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::mountain());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::ScryOrder { kept_top: vec![c], bottom: vec![a, b] },
    ]));
    flood_d(&mut g, 0);
    let giant = g.add_card_to_hand(0, catalog::hill_giant()); // power 3
    cast(&mut g, giant);
    drain_stack(&mut g);
    assert_eq!(g.players[0].library[0].id, c, "the chosen card is on top");
    let n = g.players[0].library.len();
    let bottom: Vec<_> = g.players[0].library[n - 2..].iter().map(|c| c.id).collect();
    assert!(bottom.contains(&a) && bottom.contains(&b), "the rest went to the bottom");
}

/// Elemental Bond draws for a power-3 creature, not for a 2-power one.
#[test]
fn elemental_bond_draws_on_power_three_or_more() {
    let mut g = main_phase_d();
    stock_libraries(&mut g, 3);
    g.add_card_to_battlefield(0, catalog::elemental_bond());
    let before = g.players[0].hand.len();
    flood_d(&mut g, 0);
    let giant = g.add_card_to_hand(0, catalog::hill_giant());
    cast(&mut g, giant);
    assert_eq!(g.players[0].hand.len(), before + 1, "power 3 draws");
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bears);
    assert_eq!(g.players[0].hand.len(), before + 1, "power 2 does not");
}

/// Minsc & Boo: the ETB makes Boo (legendary 1/1 Hamster, trample, haste);
/// −2 sacrificing a 3-power Boo deals 3 and, Boo being a Hamster, draws 3.
#[test]
fn minsc_boo_makes_boo_then_flings_him() {
    let mut g = main_phase_d();
    stock_libraries(&mut g, 5);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let minsc = g.move_card_to_battlefield_for_test(0, catalog::minsc_boo_timeless_heroes());
    drain_stack(&mut g);
    let boo = g
        .battlefield
        .iter()
        .find(|c| c.controller == 0 && c.definition.name == "Boo")
        .map(|c| c.id)
        .expect("Boo created");
    let kws = g.computed_permanent(boo).unwrap().keywords().to_vec();
    assert!(kws.contains(&Keyword::Trample) && kws.contains(&Keyword::Haste));
    assert!(catalog::minsc_boo_timeless_heroes().can_be_commander);

    g.battlefield_find_mut(boo).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    g.battlefield_find_mut(minsc).unwrap().counters.insert(CounterType::Loyalty, 3);
    let life = g.players[1].life;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: minsc,
        ability_index: 1,
        target: Some(Target::Player(1)),
        x_value: None,
    })
    .expect("−2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(boo).is_none(), "Boo was sacrificed");
    assert_eq!(g.players[1].life, life - 3, "X = Boo's power 3");
    assert_eq!(g.players[0].hand.len(), hand + 3, "a Hamster: draw X");
}

/// Invasion of Ikoria with X=2 puts a non-Human MV ≤ 2 creature onto the
/// battlefield; Zilortha's static lets non-Humans assign as though unblocked.
#[test]
fn invasion_of_ikoria_fetches_and_zilortha_grants_unblocked_damage() {
    let mut g = main_phase_d();
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::serra_angel());
    let inv = g.add_card_to_hand(0, catalog::invasion_of_ikoria());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bears))]));
    g.perform_action(GameAction::CastSpell {
        card_id: inv,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast Invasion of Ikoria with X=2");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bears).map(|c| c.controller), Some(0), "bears fetched");
    assert_eq!(
        g.battlefield_find(inv).unwrap().counter_count(CounterType::Defense),
        6,
        "enters with 6 defense"
    );

    let back = *catalog::invasion_of_ikoria().back_face.expect("Zilortha");
    assert_eq!(back.name, "Zilortha, Apex of Ikoria");
    let mut g = main_phase_d();
    g.add_card_to_battlefield(0, back);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let human = g.add_card_to_battlefield(0, catalog::elite_vanguard());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let has = |g: &GameState, id| {
        g.computed_permanent(id)
            .unwrap()
            .keywords()
            .contains(&Keyword::AssignsDamageAsThoughUnblocked)
    };
    assert!(has(&g, bear), "non-Human you control");
    assert!(!has(&g, human), "Humans excluded");
    assert!(!has(&g, theirs), "only yours");
}

/// Strionic Resonator copies Elemental Bond's draw trigger: two cards.
#[test]
fn strionic_resonator_copies_a_triggered_ability() {
    let mut g = main_phase_d();
    stock_libraries(&mut g, 5);
    let bond = g.add_card_to_battlefield(0, catalog::elemental_bond());
    let resonator = g.add_card_to_battlefield(0, catalog::strionic_resonator());
    let giant = g.add_card_to_hand(0, catalog::hill_giant());
    flood_d(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: giant,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Hill Giant");
    // Both pass: the giant resolves and Bond's trigger goes on the stack.
    g.perform_action(GameAction::PassPriority).unwrap();
    g.perform_action(GameAction::PassPriority).unwrap();
    assert!(!g.stack.is_empty(), "Bond's draw trigger is waiting");
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: resonator,
        ability_index: 0,
        target: Some(Target::Permanent(bond)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("copy the trigger");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 2, "original + copy");
}

/// Hunting Velociraptor: Dinosaur spells you cast have prowl {2}{R} — offered
/// once a Dinosaur dealt you combat damage this turn, and not to a non-Dinosaur.
#[test]
fn hunting_velociraptor_grants_dinosaurs_prowl() {
    let mut g = main_phase_d();
    let id = g.add_card_to_battlefield(0, catalog::hunting_velociraptor());
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 2));
    assert!(cp.keywords().contains(&Keyword::FirstStrike));
    let dreadmaw = g.add_card_to_hand(0, catalog::colossal_dreadmaw());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    let alt = |g: &mut GameState, card_id| {
        g.perform_action(GameAction::CastSpellAlternative {
            card_id,
            pitch_card: None,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    assert!(alt(&mut g, dreadmaw).is_err(), "no Dinosaur combat damage yet");
    g.players[0].prowl_types_this_turn.push(CreatureType::Dinosaur);
    assert!(alt(&mut g, bears).is_err(), "Grizzly Bears isn't a Dinosaur");
    alt(&mut g, dreadmaw).expect("prowl {2}{R}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dreadmaw).is_some(), "a 6-drop for three mana");
}
