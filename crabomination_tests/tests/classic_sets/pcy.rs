//! Prophecy (PCY), first wave.

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EntityRef;
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

fn activate(g: &mut GameState, seat: usize, card_id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// The vanilla-shaped bodies: printed keywords only.
#[test]
fn pcy_keyword_bodies_carry_their_printed_keywords() {
    let cases: &[(fn() -> crabomination::card::CardDefinition, &[Keyword])] = &[
        (catalog::diving_griffin, &[Keyword::Flying, Keyword::Vigilance]),
        (catalog::avatar_of_fury, &[Keyword::Flying]),
        (catalog::avatar_of_might, &[Keyword::Trample]),
        (catalog::avatar_of_will, &[Keyword::Flying]),
        (catalog::avatar_of_woe, &[Keyword::Fear]),
        (catalog::bog_elemental, &[Keyword::Protection(Color::White)]),
        (catalog::bog_glider, &[Keyword::Flying]),
        (catalog::copper_leaf_angel, &[Keyword::Flying]),
        (catalog::alexis_cloak, &[Keyword::Flash]),
        (catalog::greels_caress, &[Keyword::Flash]),
    ];
    for (factory, expected) in cases {
        let def = factory();
        for kw in *expected {
            assert!(def.keywords.contains(kw), "{} is missing {kw:?}", def.name);
        }
    }
}

/// Abolish can be cast for free by pitching a Plains.
#[test]
fn abolish_can_be_paid_by_discarding_a_plains() {
    let mut g = main_phase();
    let plains = g.add_card_to_hand(0, catalog::plains());
    let abolish = g.add_card_to_hand(0, catalog::abolish());
    let target = g.add_card_to_battlefield(1, catalog::sol_ring());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: abolish,
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        pitch_card: None,
        mode: None,
        x_value: None,
    })
    .expect("free cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "the artifact is destroyed");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == plains), "the Plains paid for it");
}

/// Foil's alt cost eats an Island plus one other card.
#[test]
fn foil_pitches_an_island_and_another_card() {
    let mut g = main_phase();
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let foil = g.add_card_to_hand(0, catalog::foil());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: foil,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        pitch_card: None,
        mode: None,
        x_value: None,
    })
    .expect("free Foil");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "the Bolt was countered");
    assert_eq!(g.players[0].graveyard.len(), 3, "Island + another card + Foil itself");
}

/// The Avatar cycle's board-state discounts knock {6} off.
#[test]
fn avatars_cost_six_less_on_their_condition() {
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    let fury = crabomination::card::CardInstance::new(g.next_id(), catalog::avatar_of_fury(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &fury, None), 0, "no discount yet");
    for _ in 0..7 {
        g.add_card_to_battlefield(1, catalog::mountain());
    }
    assert_eq!(cost_reduction_for_spell(&g, 0, &fury, None), 6, "seven opposing lands");

    let will = crabomination::card::CardInstance::new(g.next_id(), catalog::avatar_of_will(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &will, None), 6, "their hand is empty");
    g.add_card_to_hand(1, catalog::grizzly_bears());
    assert_eq!(cost_reduction_for_spell(&g, 0, &will, None), 0, "not any more");
}

/// Citadel of Pain bills whoever left mana up.
#[test]
fn citadel_of_pain_burns_for_untapped_lands() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::citadel_of_pain());
    g.add_card_to_battlefield(1, catalog::island());
    let tapped = g.add_card_to_battlefield(1, catalog::island());
    g.add_card_to_battlefield(1, catalog::island());
    g.battlefield_find_mut(tapped).unwrap().tapped = true;
    g.active_player_idx = 1;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "two untapped lands, two damage");
}

/// Glittering Lion shrugs off damage until someone pays to switch it off.
#[test]
fn glittering_lion_can_be_unlocked_by_any_player() {
    let mut g = main_phase();
    let lion = g.add_card_to_battlefield(0, catalog::glittering_lion());
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    let mut events = vec![];
    g.deal_damage_to_from(EntityRef::Permanent(lion), 3, Some(dragon), &mut events);
    assert_eq!(g.battlefield_find(lion).unwrap().damage, 0, "prevented");

    activate(&mut g, 1, lion, 0, None);
    let mut events = vec![];
    g.deal_damage_to_from(EntityRef::Permanent(lion), 3, Some(dragon), &mut events);
    g.check_state_based_actions();
    assert!(g.battlefield_find(lion).is_none(), "the opponent paid the {{3}} and killed it");
}

/// Branded Brawlers sits out while either side is holding up mana.
#[test]
fn branded_brawlers_only_fights_on_an_empty_board() {
    let mut g = two_player_game();
    let brawlers = g.add_card_to_battlefield(0, catalog::branded_brawlers());
    let their_land = g.add_card_to_battlefield(1, catalog::mountain());
    g.clear_sickness(brawlers);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert!(
        g.declare_attackers(vec![Attack { attacker: brawlers, target: AttackTarget::Player(1) }])
            .is_err(),
        "their untapped land locks the attack"
    );
    g.battlefield_find_mut(their_land).unwrap().tapped = true;
    g.declare_attackers(vec![Attack { attacker: brawlers, target: AttackTarget::Player(1) }])
        .expect("now it can swing");
}

/// Fen Stalker only sneaks through while you're tapped out.
#[test]
fn fen_stalker_has_fear_only_while_you_are_tapped_out() {
    let mut g = two_player_game();
    let stalker = g.add_card_to_battlefield(0, catalog::fen_stalker());
    let land = g.add_card_to_battlefield(0, catalog::swamp());
    assert!(!g.computed_permanent(stalker).unwrap().keywords.contains(&Keyword::Fear));
    g.battlefield_find_mut(land).unwrap().tapped = true;
    assert!(g.computed_permanent(stalker).unwrap().keywords.contains(&Keyword::Fear));
}

/// Chimeric Idol trades your whole mana base for a 3/3 body.
#[test]
fn chimeric_idol_taps_your_lands_to_animate() {
    let mut g = main_phase();
    let idol = g.add_card_to_battlefield(0, catalog::chimeric_idol());
    let land = g.add_card_to_battlefield(0, catalog::mountain());
    activate(&mut g, 0, idol, 0, None);
    assert!(g.battlefield_find(land).unwrap().tapped, "all your lands tap");
    let cp = g.computed_permanent(idol).expect("computed");
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.card_types.contains(&crabomination::card::CardType::Creature));
}

/// Excise exiles an attacker unless its controller matches the X.
#[test]
fn excise_exiles_an_attacker_that_isnt_paid_for() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(0) }]).expect("attack");
    let excise = g.add_card_to_hand(0, catalog::excise());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: excise,
        target: Some(Target::Permanent(attacker)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == attacker), "they declined the {{3}}");
}

/// Copper-Leaf Angel eats a land per counter.
#[test]
fn copper_leaf_angel_converts_a_land_into_a_counter() {
    let mut g = main_phase();
    let angel = g.add_card_to_battlefield(0, catalog::copper_leaf_angel());
    g.clear_sickness(angel);
    let land = g.add_card_to_battlefield(0, catalog::forest());
    activate(&mut g, 0, angel, 0, None);
    assert!(g.battlefield_find(land).is_none(), "the land paid for it");
    assert_eq!(
        g.battlefield_find(angel).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

/// Bog Elemental walks off unless it's fed a land each upkeep.
#[test]
fn bog_elemental_eats_a_land_each_upkeep() {
    let mut g = two_player_game();
    let elemental = g.add_card_to_battlefield(0, catalog::bog_elemental());
    let land = g.add_card_to_battlefield(0, catalog::swamp());
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "a land went instead");
    assert!(g.battlefield_find(elemental).is_some(), "so it stuck around");

    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(elemental).is_none(), "no land left to feed it");
}

/// Excavation lets either player trade a land for a card.
#[test]
fn excavation_can_be_activated_by_the_opponent() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::excavation());
    let excavation = g.battlefield.iter().find(|c| c.definition.name == "Excavation").unwrap().id;
    let their_land = g.add_card_to_battlefield(1, catalog::island());
    g.add_card_to_library(1, catalog::lightning_bolt());
    let before = g.players[1].hand.len();
    activate(&mut g, 1, excavation, 0, None);
    assert!(g.battlefield_find(their_land).is_none(), "they paid their own land");
    assert_eq!(g.players[1].hand.len(), before + 1, "and they drew the card");
}

/// Devastate blows up a land and singes everything.
#[test]
fn devastate_kills_a_land_and_pings_the_board() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(1, catalog::mountain());
    let x1 = g.add_card_to_battlefield(0, catalog::mons_goblin_raiders());
    let devastate = g.add_card_to_hand(0, catalog::devastate());
    cast(&mut g, 0, devastate, Some(Target::Permanent(land)));
    assert!(g.battlefield_find(land).is_none());
    assert!(g.battlefield_find(x1).is_none(), "1/1s die");
    assert_eq!(g.players[1].life, 19);
    assert_eq!(g.players[0].life, 19, "including you");
}

/// Elephant Resurgence hands everyone a graveyard-sized Elephant.
#[test]
fn elephant_resurgence_scales_each_token_to_its_owner() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_graveyard(1, catalog::grizzly_bears());
    }
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::elephant_resurgence());
    cast(&mut g, 0, spell, None);
    let mine = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Elephant" && c.controller == 0)
        .expect("your Elephant");
    let theirs = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Elephant" && c.controller == 1)
        .expect("their Elephant");
    assert_eq!(g.computed_permanent(mine.id).unwrap().power, 1, "one creature in your gy");
    assert_eq!(g.computed_permanent(theirs.id).unwrap().power, 3);
}

/// Gulf Squid's ETB taps out the player it points at.
#[test]
fn gulf_squid_taps_their_lands() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::island());
    let b = g.add_card_to_battlefield(1, catalog::island());
    let squid = g.add_card_to_hand(0, catalog::gulf_squid());
    cast(&mut g, 0, squid, Some(Target::Player(1)));
    assert!(g.battlefield_find(a).unwrap().tapped);
    assert!(g.battlefield_find(b).unwrap().tapped);
}

/// Death Charmer's bite bills the blocker's controller unless they pay.
#[test]
fn death_charmer_drains_the_creature_it_bites() {
    let mut g = two_player_game();
    let charmer = g.add_card_to_battlefield(0, catalog::death_charmer());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(charmer);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: charmer, target: AttackTarget::Player(1) }])
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, charmer)])).expect("block");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[1].life, 18, "they declined the {{2}}");
}

/// Flay takes one card at random, then another unless they pay {1}.
#[test]
fn flay_strips_two_cards_from_an_unpaying_player() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    let flay = g.add_card_to_hand(0, catalog::flay());
    cast(&mut g, 0, flay, Some(Target::Player(1)));
    assert_eq!(g.players[1].hand.len(), 1, "two random discards");
}

/// Barbed Field turns the enchanted land into a pinger.
#[test]
fn barbed_field_grants_the_land_a_ping() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::mountain());
    let field = g.add_card_to_hand(0, catalog::barbed_field());
    cast(&mut g, 0, field, Some(Target::Permanent(land)));
    let granted = g.granted_abilities_for(land);
    assert_eq!(granted.len(), 1, "the Aura's ability rides the land");
    script(&mut g, vec![]);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 1,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19);
}

/// Fault Riders' pump is once a turn and costs a land.
#[test]
fn fault_riders_pumps_once_per_turn_for_a_land() {
    let mut g = main_phase();
    let riders = g.add_card_to_battlefield(0, catalog::fault_riders());
    g.add_card_to_battlefield(0, catalog::mountain());
    g.add_card_to_battlefield(0, catalog::mountain());
    activate(&mut g, 0, riders, 0, None);
    let cp = g.computed_permanent(riders).expect("computed");
    assert_eq!(cp.power, 4);
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: riders,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "only once each turn"
    );
}

/// Blessed Wind resets a life total to 20 either way.
#[test]
fn blessed_wind_sets_life_to_twenty() {
    let mut g = main_phase();
    g.players[1].life = 3;
    let wind = g.add_card_to_hand(0, catalog::blessed_wind());
    cast(&mut g, 0, wind, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 20);
}
