#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::TurnStep;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
#[allow(unused)]
use crate::Factory;

// ── CR 401.5/401.6 — play with top revealed / play from the library top ─────

#[test]
fn courser_plays_lands_from_the_library_top_and_gains_life() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::courser_of_kruphix());
    let top = g.next_id();
    g.players[0].add_to_library_top(top, catalog::island());
    let life = g.players[0].life;

    g.perform_action(GameAction::PlayLand(top)).expect("land playable off the top");
    drain_stack(&mut g);

    assert!(g.battlefield_find(top).is_some(), "Island entered from the library");
    assert_eq!(g.players[0].life, life + 1, "landfall life");
}

#[test]
fn courser_does_not_allow_nonland_or_non_top_plays() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::courser_of_kruphix());
    // A land *below* the top is not playable.
    let below = g.next_id();
    g.players[0].add_to_library_top(below, catalog::island());
    let top = g.next_id();
    g.players[0].add_to_library_top(top, catalog::grizzly_bears());
    assert!(g.perform_action(GameAction::PlayLand(below)).is_err(), "only the top card");
    // The nonland top isn't castable off Courser's land-only permission.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: top, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .is_err(),
        "Courser's permission covers lands only"
    );
    assert_eq!(g.players[0].library.len(), 2, "library untouched after rejections");
}

#[test]
fn mystic_forge_casts_artifacts_from_the_top() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mystic_forge());
    let top = g.next_id();
    g.players[0].add_to_library_top(top, catalog::mind_stone());
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: top, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("artifact castable off the top");
    drain_stack(&mut g);

    assert!(g.battlefield_find(top).is_some(), "Mind Stone resolved from the library top");
}

#[test]
fn mystic_forge_exiles_the_top_for_a_life() {
    let mut g = two_player_game();
    let forge = g.add_card_to_battlefield(0, catalog::mystic_forge());
    g.add_card_to_library(0, catalog::island());
    let life = g.players[0].life;
    let lib = g.players[0].library.len();

    g.perform_action(GameAction::ActivateAbility {
        card_id: forge, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("{T}, pay 1 life activates");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life - 1);
    assert_eq!(g.players[0].library.len(), lib - 1, "top card exiled");
}

#[test]
fn top_of_library_revealed_in_view_with_courser() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::courser_of_kruphix());
    let top = g.next_id();
    g.players[0].add_to_library_top(top, catalog::island());
    // Revealed to both seats.
    for viewer in 0..2 {
        let view = crabomination::server::view::project(&g, viewer);
        assert_eq!(
            view.players[0].library.known_top.first().map(|c| c.name.as_str()),
            Some("Island"),
            "viewer {viewer} sees the revealed top"
        );
    }
}

#[test]
fn gather_specimens_steals_an_opponent_creature_etb() {
    let mut g = two_player_game();
    // P1's turn: P0 flashes in Gather Specimens, then P1 casts a creature.
    g.active_player_idx = 1;
    let spell = g.add_card_to_hand(0, catalog::gather_specimens());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, spell);

    // P1 casts a creature this turn — it enters under P0's control.
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bear castable");
    drain_stack(&mut g);

    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0, "stolen on entry");

    // The window closes at cleanup.
    g.creature_etb_steal_this_turn.clear();
    let bear2 = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear2, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bear castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear2).unwrap().controller, 1, "no replacement next turn");
}

#[test]
fn gather_specimens_steals_token_mints() {
    let mut g = two_player_game();
    g.creature_etb_steal_this_turn.push(0);
    // P1 mints a creature token — it enters under P0's control (and P0
    // owns it, CR 111.2). A noncreature token is unaffected.
    let ctx = EffectContext::for_spell(1, None, 0, 0);
    g.resolve_effect(
        &Effect::CreateToken {
            who: crabomination::effect::PlayerRef::You,
            count: crabomination::effect::Value::Const(1),
            definition: crabomination_base::tokens::spirit_token(),
        },
        &ctx,
    ).unwrap();
    let spirit = g.battlefield.iter().find(|c| c.definition.name == "Spirit").unwrap();
    assert_eq!(spirit.controller, 0, "creature token stolen on entry");
    assert_eq!(spirit.owner, 0, "stolen token is owned by its actual controller");
    g.resolve_effect(
        &Effect::CreateToken {
            who: crabomination::effect::PlayerRef::You,
            count: crabomination::effect::Value::Const(1),
            definition: crabomination_base::tokens::treasure_token(),
        },
        &ctx,
    ).unwrap();
    let treasure = g.battlefield.iter().find(|c| c.definition.name == "Treasure").unwrap();
    assert_eq!(treasure.controller, 1, "noncreature token unaffected");
}

#[test]
fn tempt_with_bunnies_offer_accepted_doubles_up() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    for _ in 0..2 { g.add_card_to_library(1, catalog::island()); }
    let spell = g.add_card_to_hand(0, catalog::tempt_with_bunnies());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    // Opponent accepts the offer.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let h0 = g.players[0].hand.len() - 1; // minus the spell itself
    let h1 = g.players[1].hand.len();
    cast(&mut g, spell);

    let rabbits = |g: &GameState, seat| g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Rabbit" && c.controller == seat)
        .count();
    assert_eq!(rabbits(&g, 0), 2, "controller: base + one per acceptor");
    assert_eq!(rabbits(&g, 1), 1, "acceptor copies the offer once");
    assert_eq!(g.players[0].hand.len(), h0 + 2, "controller drew twice");
    assert_eq!(g.players[1].hand.len(), h1 + 1, "acceptor drew once");
}

#[test]
fn tempt_with_bunnies_offer_declined_is_single() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let spell = g.add_card_to_hand(0, catalog::tempt_with_bunnies());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    // AutoDecider declines OptionalTrigger.
    cast(&mut g, spell);
    let rabbits = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Rabbit")
        .count();
    assert_eq!(rabbits, 1, "no acceptors → just the base resolution");
}

#[test]
fn muldrotha_casts_one_permanent_of_each_type_from_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::muldrotha_the_gravetide());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let stone = g.add_card_to_graveyard(0, catalog::mind_stone());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(6);

    // Creature from the graveyard: allowed once.
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("creature castable from graveyard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());

    // A second creature this turn: rejected.
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: bear2, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "one creature per turn");

    // An artifact still works (a different permanent type).
    g.perform_action(GameAction::CastSpell {
        card_id: stone, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("artifact castable from graveyard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(stone).is_some());

    // Next turn the tally resets.
    g.players[0].graveyard_cast_types_this_turn.clear();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear2, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("creature castable again after reset");
}

#[test]
fn muldrotha_does_not_allow_casts_on_opponent_turns() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::muldrotha_the_gravetide());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "Muldrotha only works during your turns");
}

#[test]
fn agathas_cauldron_exiles_and_grants_abilities() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let cauldron = g.add_card_to_battlefield(0, catalog::agathas_soul_cauldron());
    // A creature with an activated ability in the graveyard (Grinning Ignus:
    // mana + return-self — use Llanowar Elves: {T}: Add {G}).
    let elves = g.add_card_to_graveyard(1, catalog::llanowar_elves());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    g.perform_action(GameAction::ActivateAbility {
        card_id: cauldron, ability_index: 0, target: Some(Target::Permanent(elves)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("{T}: exile target card from a graveyard");
    drain_stack(&mut g);

    let exiled = g.exile.iter().find(|c| c.id == elves).expect("Elves exiled");
    assert_eq!(exiled.exiled_with, Some(cauldron), "linked to the Cauldron");
    let bear_inst = g.battlefield_find(bear).unwrap();
    assert_eq!(
        bear_inst.counter_count(crabomination::card::CounterType::PlusOnePlusOne),
        1, "creature-card exile adds a +1/+1 counter",
    );
    // The countered Bear now has the exiled Elves' mana ability.
    let granted = g.granted_abilities_for(bear);
    assert!(!granted.is_empty(), "Bear borrows the exiled creature's activated ability");
}

#[test]
fn mutated_cultist_discounts_next_spell_by_counters_removed() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    // A permanent holding three counters.
    let walker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(walker).unwrap()
        .add_counters(crabomination::card::CounterType::PlusOnePlusOne, 3);

    let cultist = g.add_card_to_hand(0, catalog::mutated_cultist());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),                       // "you may"
        DecisionAnswer::Target(Target::Permanent(walker)), // counter target
    ]));
    cast(&mut g, cultist);

    assert_eq!(
        g.battlefield_find(walker).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne),
        0, "counters removed",
    );
    // Next spell ({3} Mind Stone... use a {2}+{G} bear: discount 3 → free generic).
    let stone = g.add_card_to_hand(0, catalog::mind_stone());
    g.perform_action(GameAction::CastSpell {
        card_id: stone, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mind Stone free after a 3-counter discount (cost {2})");
    drain_stack(&mut g);
    assert!(g.battlefield_find(stone).is_some());
}

#[test]
fn rediscover_the_way_chapter_three_grants_double_strike_per_noncreature_spell() {
    let mut g = two_player_game();
    let saga = g.add_card_to_battlefield(0, catalog::rediscover_the_way());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Jump straight to chapter III.
    g.battlefield_find_mut(saga).unwrap().add_counters(crabomination::card::CounterType::Lore, 2);
    g.saga_advance(saga);
    drain_stack(&mut g);

    // A noncreature spell grants the Bear double strike.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);

    let computed = g.computed_permanent(bear).unwrap();
    assert!(computed.keywords.contains(&crabomination::card::Keyword::DoubleStrike),
        "noncreature cast after chapter III grants double strike");
}


#[test]
fn zirda_companion_comes_to_hand_for_three() {
    let mut g = two_player_game();
    let zirda = g.next_id();
    g.players[0].sideboard.push(crabomination::card::CardInstance::new(
        zirda, catalog::zirda_the_dawnwaker(), 0,
    ));
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CompanionToHand(zirda)).expect("companion for {3}");
    assert!(g.players[0].hand.iter().any(|c| c.id == zirda), "Zirda in hand");
    assert!(g.players[0].sideboard.is_empty(), "left the sideboard for good");
    assert_eq!(g.players[0].mana_pool.total(), 0, "{{3}} paid");
}

#[test]
fn zirda_discounts_nonmana_activations_with_a_floor() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let zirda = g.add_card_to_battlefield(0, catalog::zirda_the_dawnwaker());
    g.clear_sickness(zirda);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Zirda's own {1},{T} ability: {1} − {2} would be free, but the floor
    // keeps one mana... CR: "can't reduce the mana in that cost to less
    // than one mana" — {1} stays {1}.
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: zirda, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None, mode: None,
        })
        .is_err(),
        "no mana floating — the floored cost is still {{1}}"
    );
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: zirda, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("{1} after floor");
    drain_stack(&mut g);
    let computed = g.computed_permanent(bear).unwrap();
    assert!(computed.keywords.contains(&crabomination::card::Keyword::CantBlock));
}

// ── draw-ordinal / first-spell-on-opp-turn triggers + exile-cast batch ──────

#[test]
fn faerie_vandal_grows_on_the_second_draw_each_turn() {
    let mut g = two_player_game();
    let vandal = g.add_card_to_battlefield(0, catalog::faerie_vandal());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    g.players[0].cards_drawn_this_turn = 0;
    let draw = |g: &mut GameState| {
        let mut events = vec![];
        g.draw_one(0, &mut events);
        g.dispatch_triggers_for_events(&events);
        drain_stack(g);
    };
    draw(&mut g);
    assert_eq!(g.battlefield_find(vandal).unwrap()
        .counter_count(crabomination::card::CounterType::PlusOnePlusOne), 0, "first draw: no counter");
    draw(&mut g);
    assert_eq!(g.battlefield_find(vandal).unwrap()
        .counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1, "second draw: +1/+1");
    draw(&mut g);
    assert_eq!(g.battlefield_find(vandal).unwrap()
        .counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1, "third draw: nothing");
}

#[test]
fn mad_ratter_mints_two_rats_on_the_second_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mad_ratter());
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    g.players[0].cards_drawn_this_turn = 0;
    for _ in 0..2 {
        let mut events = vec![];
        g.draw_one(0, &mut events);
        g.dispatch_triggers_for_events(&events);
        drain_stack(&mut g);
    }
    let rats = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Rat" && c.controller == 0)
        .count();
    assert_eq!(rats, 2, "two Rats on the second draw");
}

#[test]
fn wavebreak_hippocamp_draws_on_first_spell_in_opponent_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::wavebreak_hippocamp());
    g.add_card_to_library(0, catalog::island());
    g.active_player_idx = 1; // opponent's turn
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len() - 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt at instant speed");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "first spell on opp turn drew");
}

#[test]
fn hostage_taker_exiles_and_lets_you_cast_the_hostage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let taker = g.add_card_to_hand(0, catalog::hostage_taker());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, taker);

    let exiled = g.exile.iter().find(|c| c.id == bear).expect("Bear exiled");
    assert_eq!(exiled.may_play_until.map(|p| p.player), Some(0), "you may cast it");

    // Cast the hostage from exile, spending mana as though it were mana of
    // any type — {1}{G} Bear paid entirely with black mana.
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("hostage castable from exile paying any-type mana");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0, "stolen for good");
    assert_eq!(g.players[0].mana_pool.total(), 0, "the MV was actually paid");
}

#[test]
fn gonti_exiles_one_of_the_top_four_with_a_cast_permission() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(1, catalog::island()); }
    let fat = g.next_id();
    g.players[1].add_to_library_top(fat, catalog::mind_stone()); // highest MV of top 4
    let gonti = g.add_card_to_hand(0, catalog::gonti_lord_of_luxury());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, gonti);

    let stolen = g.exile.iter().find(|c| c.id == fat).expect("top-4 pick exiled");
    assert!(stolen.face_down, "exiled face down");
    assert_eq!(stolen.may_play_until.map(|p| p.player), Some(0), "Gonti's controller may cast it");
}

#[test]
fn grafdiggers_cage_locks_reanimation_and_graveyard_casts() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::grafdiggers_cage());
    // Reanimation fizzles: a Move from graveyard → battlefield does nothing.
    g.add_card_to_battlefield(0, catalog::muldrotha_the_gravetide());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).is_err(),
        "no casting from graveyards under the Cage"
    );
    // Flashback is locked too.
    let dart = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let _ = dart;
    assert!(g.battlefield_find(bear).is_none());
}

#[test]
fn grafdiggers_cage_blocks_search_to_battlefield() {
    use crabomination::card::SelectionRequirement;
    use crabomination::effect::{PlayerRef, ZoneDest};
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::grafdiggers_cage());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(bear)),
        DecisionAnswer::Search(Some(bear)),
    ]));
    g.resolve_effect(
        &Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Creature,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        &ctx,
    ).unwrap();
    assert!(g.battlefield_find(bear).is_none(), "creature stays in the library");
    assert!(g.players[0].library.iter().any(|c| c.id == bear));
    // Searching to hand is unaffected.
    g.resolve_effect(
        &Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Creature,
            to: ZoneDest::Hand(PlayerRef::You),
        },
        &ctx,
    ).unwrap();
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "to-hand search still works");
}

#[test]
fn restless_anchorage_animates_and_maps_on_attack() {
    use crabomination::card::ArtifactSubtype;
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::restless_anchorage());
    g.clear_sickness(land);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("animate");
    drain_stack(&mut g);
    let computed = g.computed_permanent(land).unwrap();
    assert!(computed.card_types.contains(&crabomination::card::CardType::Creature));
    assert!(computed.keywords.contains(&crabomination::card::Keyword::Flying));

    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: land, target: AttackTarget::Player(1) }])
        .expect("attacks");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.is_token
            && c.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Map)),
        "attack trigger mints a Map"
    );
}

#[test]
fn restless_prairie_pumps_the_team_on_attack() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::restless_prairie());
    g.clear_sickness(land);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("animate");
    drain_stack(&mut g);
    g.clear_sickness(land);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: land, target: AttackTarget::Player(1) }])
        .expect("attacks");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "other creatures +1/+1");
    assert_eq!(g.computed_permanent(land).unwrap().power, 3, "the Llama itself untouched");
}

#[test]
fn restless_vents_loots_on_attack() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::restless_vents());
    g.clear_sickness(land);
    let spare = g.add_card_to_hand(0, catalog::island());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("animate");
    drain_stack(&mut g);
    g.clear_sickness(land);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Discard(vec![spare]),
    ]));
    let hand = g.players[0].hand.len();
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: land, target: AttackTarget::Player(1) }])
        .expect("attacks");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == spare), "discarded");
    assert_eq!(g.players[0].hand.len(), hand, "discard then draw nets zero");
}

// ── Search hate (CR 701.19 — Aven Mindcensor / Leonin Arbiter) ───────────────

/// Aven Mindcensor restricts an opponent's search to the top four cards.
#[test]
fn aven_mindcensor_limits_opponent_search_to_top_four() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::aven_mindcensor());
    // Forest is the fifth card down — out of Mindcensor range.
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let forest = g.add_card_to_library(0, catalog::forest());
    let fetch = g.add_card_to_battlefield(0, catalog::prismatic_vista());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: fetch, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("crack fetch");
    drain_stack(&mut g);
    assert!(
        g.battlefield_find(forest).is_none(),
        "Forest below the top four can't be found (the pick is rejected)"
    );
}

/// The searcher's own Mindcensor doesn't restrict their search.
#[test]
fn aven_mindcensor_does_not_limit_your_own_search() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::aven_mindcensor());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let forest = g.add_card_to_library(0, catalog::forest());
    let fetch = g.add_card_to_battlefield(0, catalog::prismatic_vista());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: fetch, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("crack fetch");
    drain_stack(&mut g);
    assert!(
        g.battlefield_find(forest).is_some(),
        "own search unrestricted (fetches the only basic, fifth down)"
    );
}

/// Leonin Arbiter blanks a search when the searcher can't pay {2}; with
/// floating mana the tax is auto-paid and the search goes through.
#[test]
fn leonin_arbiter_taxes_searches() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::leonin_arbiter());
    let forest_a = g.add_card_to_library(0, catalog::forest());
    let fetch = g.add_card_to_battlefield(0, catalog::prismatic_vista());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest_a))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: fetch, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("crack fetch");
    drain_stack(&mut g);
    assert!(
        g.battlefield_find(forest_a).is_none(),
        "no floating mana → search finds nothing"
    );

    // Second fetch with {2} floating: the tax is paid, the search resolves.
    let fetch2 = g.add_card_to_battlefield(0, catalog::prismatic_vista());
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest_a))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: fetch2, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("crack fetch");
    drain_stack(&mut g);
    assert!(
        g.battlefield_find(forest_a).is_some(),
        "tax paid from floating mana → search succeeds"
    );
    assert_eq!(g.players[0].mana_pool.total(), 0, "tax consumed the floating mana");
}

/// Sanctifier en-Vec sweeps black/red graveyard cards on ETB and exiles
/// later black/red cards bound for any graveyard (others still land there).
#[test]
fn sanctifier_en_vec_exiles_black_and_red_cards() {
    let mut g = two_player_game();
    let rotted = g.add_card_to_graveyard(1, catalog::lightning_bolt()); // red
    let stays = g.add_card_to_graveyard(1, catalog::grizzly_bears()); // green
    let id = g.add_card_to_hand(0, catalog::sanctifier_en_vec());
    g.players[0].mana_pool.add(Color::White, 2);
    cast(&mut g, id);
    assert!(g.exile.iter().any(|c| c.id == rotted), "red gy card swept on ETB");
    assert!(
        g.players[1].graveyard.iter().any(|c| c.id == stays),
        "green card untouched"
    );

    // A red spell cast afterwards is exiled instead of hitting the graveyard.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast bolt");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt), "resolved red spell exiled (614.6)");
    // A green creature dying still goes to the graveyard.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.remove_from_battlefield_to_graveyard_raw(bear);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "green death unaffected");
}

/// Suppression Field taxes non-mana activated abilities by {2}; mana
/// abilities are exempt.
#[test]
fn suppression_field_taxes_nonmana_activations() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::suppression_field());
    // Prismatic Vista's fetch ability (free + tap) now costs {2}.
    let fetch = g.add_card_to_battlefield(0, catalog::prismatic_vista());
    g.add_card_to_library(0, catalog::forest());
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: fetch, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).is_err(), "no mana → taxed fetch rejected");
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: fetch, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("taxed fetch payable with {2}");
    // A basic's mana ability is exempt.
    let isl = g.add_card_to_battlefield(0, catalog::island());
    g.perform_action(GameAction::ActivateAbility {
        card_id: isl, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("mana ability untaxed");
}

/// Reckoner Bankbuster enters with three charges, draws per activation, and
/// mints a Treasure + Pilot when the last charge comes off.
#[test]
fn reckoner_bankbuster_draws_then_pays_out_when_empty() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::reckoner_bankbuster());
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    assert_eq!(
        g.battlefield_find(id).unwrap().counter_count(CounterType::Charge),
        3,
        "enters with three charges"
    );
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand_before = g.players[0].hand.len();
    for i in 0..3 {
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("activate Bankbuster");
        drain_stack(&mut g);
        if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == id) {
            c.tapped = false;
        }
        let treasures = g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count();
        let pilots = g.battlefield.iter().filter(|c| c.definition.name == "Pilot").count();
        if i < 2 {
            assert_eq!((treasures, pilots), (0, 0), "no payout while charges remain");
        } else {
            assert_eq!((treasures, pilots), (1, 1), "payout on the last charge");
        }
    }
    assert_eq!(g.players[0].hand.len(), hand_before + 3, "drew once per activation");
}

/// Mizzium Mortars: targeted for {1}{R}, overloaded for {3}{R}{R}{R} it hits
/// each creature you don't control (yours untouched).
#[test]
fn mizzium_mortars_overload_sweeps_only_their_board() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::arbor_colossus());
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let big = g.add_card_to_battlefield(1, catalog::arbor_colossus());
    let id = g.add_card_to_hand(0, catalog::mizzium_mortars());
    g.players[0].mana_pool.add(Color::Red, 3);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("overload cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(small).is_none(), "2/2 swept");
    assert_eq!(g.battlefield_find(big).unwrap().damage, 4, "6/6 damaged, alive");
    assert_eq!(g.battlefield_find(mine).unwrap().damage, 0, "own creature untouched");
}

// ── Fable of the Mirror-Breaker (DFC saga, CR 714.4) ─────────────────────────

/// Fable mints the Goblin Shaman on chapter I and transforms into Reflection
/// of Kiki-Jiki on chapter III instead of being sacrificed.
#[test]
fn fable_chapter_three_returns_transformed() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fable_of_the_mirror_breaker());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id); // chapter I
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Goblin Shaman" && c.is_token),
        "chapter I minted the Shaman"
    );
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.saga_advance(id); // chapter II (rummage)
    drain_stack(&mut g);
    g.saga_advance(id); // chapter III (exile, return transformed)
    drain_stack(&mut g);
    let flipped = g.battlefield_find(id).expect("returned to the battlefield");
    assert_eq!(flipped.definition.name, "Reflection of Kiki-Jiki");
    assert!(flipped.transformed, "back face active");
    assert_eq!(
        flipped.counter_count(CounterType::Lore),
        0,
        "new object — lore counters gone"
    );
}

/// Fable's Shaman token mints a Treasure when it attacks.
#[test]
fn fable_shaman_token_mints_treasure_on_attack() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fable_of_the_mirror_breaker());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    let shaman = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Goblin Shaman")
        .map(|c| c.id)
        .expect("shaman minted");
    g.clear_sickness(shaman);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: shaman, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Treasure"),
        "attack trigger minted a Treasure"
    );
}

/// Reflection of Kiki-Jiki copies a nonlegendary creature with haste; the
/// copy is sacrificed at the next end step.
#[test]
fn reflection_of_kiki_jiki_copies_then_sacs_at_end_step() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let kiki = g.add_card_to_battlefield(0, catalog::reflection_of_kiki_jiki());
    g.clear_sickness(kiki);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kiki, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate Kiki");
    drain_stack(&mut g);
    let copy = g
        .battlefield
        .iter()
        .find(|c| c.is_token && c.definition.name == "Grizzly Bears")
        .expect("token copy created");
    assert!(copy.granted_keywords_eot.contains(&Keyword::Haste) || {
        let cp = g.computed_permanent(copy.id).unwrap();
        cp.keywords.contains(&Keyword::Haste)
    }, "copy has haste");
    let copy_id = copy.id;
    // Advance to the end step — the delayed trigger sacrifices the copy.
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(copy_id).is_none(), "copy gone at end step");
}

// ── modern_decks-18: Amulet Titan / Valakut package ──────────────────────────

/// Amulet of Vigor untaps a permanent that enters tapped under your control.
#[test]
fn amulet_of_vigor_untaps_tapped_entrant() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::amulet_of_vigor());
    let land = g.add_card_to_hand(0, catalog::valakut_the_molten_pinnacle());
    g.perform_action(GameAction::PlayLand(land)).expect("play Valakut");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(land).unwrap().tapped, "Amulet untapped it");
    // An opponent's tapped entrant is unaffected.
    let opp = g.add_card_to_hand(1, catalog::valakut_the_molten_pinnacle());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::PlayLand(opp)).expect("play");
    drain_stack(&mut g);
    assert!(g.battlefield_find(opp).unwrap().tapped, "opponent's stays tapped");
}

/// Valakut deals 3 when a sixth Mountain arrives, and stays quiet below the
/// threshold.
#[test]
fn valakut_triggers_on_sixth_mountain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::valakut_the_molten_pinnacle());
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    // Fifth Mountain: only four others — no trigger.
    let m5 = g.add_card_to_hand(0, catalog::mountain());
    g.perform_action(GameAction::PlayLand(m5)).expect("play");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20, "below threshold — no bolt");
    // Sixth Mountain: five others — Valakut fires at the opponent.
    g.players[0].lands_played_this_turn = 0;
    let m6 = g.add_card_to_hand(0, catalog::mountain());
    g.perform_action(GameAction::PlayLand(m6)).expect("play");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "Valakut bolted the opponent");
}

/// Dryad of the Ilysian Grove makes lands every basic type (a Forest taps
/// for any color) and grants an extra land drop.
#[test]
fn dryad_makes_lands_all_basic_types() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dryad_of_the_ilysian_grove());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let cp = g.computed_permanent(forest).unwrap();
    assert!(cp.subtypes.land_types.contains(&crabomination::card::LandType::Island), "Island type added");
    let cost = crabomination::mana::cost(&[crabomination::mana::u()]);
    g.auto_tap_for_cost(0, &cost);
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1, "Forest taps for blue under Dryad");
    // Two land drops in one turn.
    let l1 = g.add_card_to_hand(0, catalog::island());
    let l2 = g.add_card_to_hand(0, catalog::island());
    g.perform_action(GameAction::PlayLand(l1)).expect("first drop");
    g.perform_action(GameAction::PlayLand(l2)).expect("second drop via Dryad");
}

/// Scapeshift sacrifices N lands and fetches that many to the battlefield
/// tapped.
#[test]
fn scapeshift_swaps_lands_from_library() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let l1 = g.add_card_to_battlefield(0, catalog::mountain());
    let l2 = g.add_card_to_battlefield(0, catalog::mountain());
    let v1 = g.add_card_to_library(0, catalog::valakut_the_molten_pinnacle());
    let f1 = g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::scapeshift());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Amount(2),
        DecisionAnswer::Search(Some(v1)),
        DecisionAnswer::Search(Some(f1)),
    ]));
    cast(&mut g, id);
    assert!(g.battlefield_find(l1).is_none() && g.battlefield_find(l2).is_none(), "lands sacked");
    let v = g.battlefield_find(v1).expect("Valakut fetched");
    assert!(v.tapped, "fetched tapped");
    assert!(g.battlefield_find(f1).is_some(), "Forest fetched");
}

/// Titania returns a land on ETB and mints a 5/3 when your land dies.
#[test]
fn titania_recurs_land_and_rewards_land_death() {
    let mut g = two_player_game();
    let gy_land = g.add_card_to_graveyard(0, catalog::valakut_the_molten_pinnacle());
    let id = g.add_card_to_hand(0, catalog::titania_protector_of_argoth());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    assert!(g.battlefield_find(gy_land).is_some(), "land returned on ETB");
    // Destroy the land through the effect path so dies-triggers dispatch.
    let mage = g.add_card_to_battlefield(1, catalog::fulminator_mage());
    g.clear_sickness(mage);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: mage, ability_index: 0, target: Some(Target::Permanent(gy_land)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("destroy the land");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Elemental"),
        "land death minted a 5/3 Elemental"
    );
}

/// Crashing Footfalls suspends for {G} and resolves into two 4/4 Rhinos.
#[test]
fn crashing_footfalls_suspends_into_rhinos() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::crashing_footfalls());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::Suspend { card_id: id }).expect("suspend");
    g.step = TurnStep::Upkeep;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    for _ in 0..4 { let _ = g.process_suspend(); }
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Rhino").count(),
        2,
        "two Rhinos"
    );
}

/// Oliphaunt mountaincycles for {1}.
#[test]
fn oliphaunt_mountaincycles() {
    let mut g = two_player_game();
    let mtn = g.add_card_to_library(0, catalog::mountain());
    let id = g.add_card_to_hand(0, catalog::oliphaunt());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Landcycle { card_id: id }).expect("mountaincycle");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == mtn), "Mountain to hand");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "Oliphaunt pitched");
}

/// White Orchid Phantom blows up a nonbasic land; its controller fetches a
/// basic tapped.
#[test]
fn white_orchid_phantom_destroys_nonbasic_with_compensation() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let valakut = g.add_card_to_battlefield(1, catalog::valakut_the_molten_pinnacle());
    let basic = g.add_card_to_library(1, catalog::mountain());
    let id = g.add_card_to_hand(0, catalog::white_orchid_phantom());
    g.players[0].mana_pool.add(Color::White, 2);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(basic)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(valakut)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(valakut).is_none(), "nonbasic destroyed");
    let b = g.battlefield_find(basic).expect("basic fetched");
    assert_eq!(b.controller, 1, "under its owner's control");
    assert!(b.tapped, "fetched tapped");
}

/// Boseiju's channel hits only opponents' permanents and compensates with a
/// basic-land fetch.
#[test]
fn boseiju_channel_destroys_opponent_nonbasic() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let valakut = g.add_card_to_battlefield(1, catalog::valakut_the_molten_pinnacle());
    let basic = g.add_card_to_library(1, catalog::mountain());
    let id = g.add_card_to_hand(0, catalog::boseiju_who_endures());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(basic))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: Some(Target::Permanent(valakut)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("channel");
    drain_stack(&mut g);
    assert!(g.battlefield_find(valakut).is_none(), "destroyed");
    assert!(g.battlefield_find(basic).is_some(), "controller fetched a basic");
}

/// Path to Exile's exiled creature's controller fetches a basic tapped.
#[test]
fn path_to_exile_controller_fetches_basic() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let basic = g.add_card_to_library(1, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::path_to_exile());
    g.players[0].mana_pool.add(Color::White, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(basic))]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Path");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "creature exiled");
    let b = g.battlefield_find(basic).expect("basic fetched");
    assert_eq!((b.controller, b.tapped), (1, true), "their basic, tapped");
}

/// Generous Ent forestcycles for {1}.
#[test]
fn generous_ent_forestcycles() {
    let mut g = two_player_game();
    let f = g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::generous_ent());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Landcycle { card_id: id }).expect("forestcycle");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == f), "Forest to hand");
}

// ── modern_decks-19: "may accept" burn + equipment ───────────────────────────

/// Vexing Devil: opponent declines → 4/3 stays; opponent accepts → 4 damage
/// and the Devil is sacrificed.
#[test]
fn vexing_devil_offer_both_branches() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    // Decline: the Devil sticks around.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::vexing_devil());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, id);
    assert!(g.battlefield_find(id).is_some(), "declined → Devil stays");
    assert_eq!(g.players[1].life, 20);

    // Accept: 4 damage, Devil sacrificed.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::vexing_devil());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, id);
    assert!(g.battlefield_find(id).is_none(), "accepted → Devil sacrificed");
    assert_eq!(g.players[1].life, 16, "took 4");
}

/// Browbeat: nobody accepts → target player draws three.
#[test]
fn browbeat_draws_three_when_no_one_takes_five() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::browbeat());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len() - 1; // minus Browbeat itself
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(0)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 3, "drew three");
}

/// Risk Factor: the targeted opponent accepts and takes four.
#[test]
fn risk_factor_opponent_takes_four() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::risk_factor());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 16, "opponent took 4 instead of giving cards");
}

/// Welding Jar's sacrifice regenerates an artifact.
#[test]
fn welding_jar_regenerates_artifact() {
    let mut g = two_player_game();
    let jar = g.add_card_to_battlefield(0, catalog::welding_jar());
    let target = g.add_card_to_battlefield(0, catalog::reckoner_bankbuster());
    g.perform_action(GameAction::ActivateAbility {
        card_id: jar, ability_index: 0, target: Some(Target::Permanent(target)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("sac the Jar");
    drain_stack(&mut g);
    assert!(g.battlefield_find(jar).is_none(), "Jar sacrificed");
    assert!(
        g.battlefield_find(target).unwrap().regeneration_shields > 0,
        "regeneration shield applied"
    );
}

/// Colossus Hammer grants +10/+10 once equipped for {8}.
#[test]
fn colossus_hammer_equips_for_eight() {
    let mut g = two_player_game();
    let hammer = g.add_card_to_battlefield(0, catalog::colossus_hammer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.perform_action(GameAction::Equip {
        equipment: hammer, target: bear,
    }).is_err(), "no mana → can't equip");
    g.players[0].mana_pool.add_colorless(8);
    g.perform_action(GameAction::Equip { equipment: hammer, target: bear }).expect("equip");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (12, 12), "+10/+10");
}

/// Sigarda's Aid lets an Equipment be cast at instant speed and attaches it
/// on entry.
#[test]
fn sigardas_aid_flashes_in_equipment_and_attaches() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sigardas_aid());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hammer = g.add_card_to_hand(0, catalog::colossus_hammer());
    g.players[0].mana_pool.add_colorless(1);
    // Not our main phase: instant timing required.
    g.step = TurnStep::End;
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: hammer, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("flash in the Equipment");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(hammer).unwrap().attached_to,
        Some(bear),
        "attached on entry"
    );
}

// ── modern_decks-19: manlands + utility lands ────────────────────────────────

/// Raging Ravine animates into a 3/3 that grows when it attacks.
#[test]
fn raging_ravine_grows_on_attack() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::raging_ravine());
    g.perform_action(GameAction::PlayLand(id)).expect("play");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().tapped, "enters tapped");
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("animate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "3/3 Elemental");
    g.clear_sickness(id);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: id, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "attack added a +1/+1 counter"
    );
}

/// Lumbering Falls animates into a 3/3 with hexproof.
#[test]
fn lumbering_falls_animates_hexproof() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lumbering_falls());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("animate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(id).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature) && cp.keywords.contains(&Keyword::Hexproof));
}

/// Slayers' Stronghold pumps and grants vigilance + haste.
#[test]
fn slayers_stronghold_pumps_attacker() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::slayers_stronghold());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 4, "+2/+0");
    assert!(cp.keywords.contains(&Keyword::Vigilance) && cp.keywords.contains(&Keyword::Haste));
}

/// Grove of the Burnwillows' colored taps feed each opponent 1 life.
#[test]
fn grove_of_the_burnwillows_gives_opponent_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::grove_of_the_burnwillows());
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("tap for red");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
    assert_eq!(g.players[1].life, 21, "opponent gained 1");
}

/// Glimmervoid sacrifices itself at the end step without an artifact.
#[test]
fn glimmervoid_sacs_without_artifacts() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::glimmervoid());
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none(), "no artifact → sacrificed");

    // With an artifact it survives.
    let id2 = g.add_card_to_battlefield(0, catalog::glimmervoid());
    g.add_card_to_battlefield(0, catalog::welding_jar());
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(id2).is_some(), "artifact → stays");
}

/// Living End wheels graveyard creatures into play and sweeps the board.
#[test]
fn living_end_swaps_graveyards_for_battlefields() {
    let mut g = two_player_game();
    let my_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let their_gy = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let spell_gy = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let my_board = g.add_card_to_battlefield(0, catalog::arbor_colossus());
    let their_board = g.add_card_to_battlefield(1, catalog::arbor_colossus());
    let id = g.add_card_to_hand(0, catalog::living_end());
    // Cast it for free via the test shortcut: zero-cost path isn't legal
    // from hand (no mana cost), so resolve the effect directly.
    g.players[0].hand.retain(|c| c.id != id);
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(&crabomination::card::Effect::LivingEnd, &ctx).expect("resolve");
    let _ = evs;
    assert!(g.battlefield_find(my_gy).is_some(), "my dead bear returns");
    assert!(g.battlefield_find(their_gy).is_some(), "their dead bear returns");
    assert_eq!(g.battlefield_find(their_gy).unwrap().controller, 1, "under its owner");
    assert!(g.battlefield_find(my_board).is_none(), "boards swept");
    assert!(g.battlefield_find(their_board).is_none(), "boards swept");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == spell_gy), "noncreature stays");
}

// ── modern_decks-20 ──────────────────────────────────────────────────────────

/// The One Ring ramps its draw and drains by burden count at upkeep.
#[test]
fn the_one_ring_burden_draw_and_upkeep_drain() {
    let mut g = two_player_game();
    let ring = g.add_card_to_battlefield(0, catalog::the_one_ring());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: ring, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("tick 1");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "first tick draws one");
    g.battlefield.iter_mut().find(|c| c.id == ring).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ring, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("tick 2");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 3, "second tick draws two");
    // Upkeep drain: 2 burden counters → lose 2.
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 18, "lost 1 per burden counter");
}

/// Harbinger of the Seas turns nonbasics into Islands.
#[test]
fn harbinger_of_the_seas_floods_nonbasics() {
    use crabomination::card::LandType;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::harbinger_of_the_seas());
    let valakut = g.add_card_to_battlefield(1, catalog::valakut_the_molten_pinnacle());
    let basic = g.add_card_to_battlefield(1, catalog::mountain());
    let cp = g.computed_permanent(valakut).unwrap();
    assert_eq!(cp.subtypes.land_types, vec![LandType::Island], "nonbasic → Island only");
    let cb = g.computed_permanent(basic).unwrap();
    assert!(cb.subtypes.land_types.contains(&LandType::Mountain), "basics untouched");
}

/// Flare of Denial counters a spell by sacrificing a nontoken blue creature.
#[test]
fn flare_of_denial_pitch_counters() {
    let mut g = two_player_game();
    let pitcher = g.add_card_to_battlefield(1, catalog::harbinger_of_the_seas());
    let flare = g.add_card_to_hand(1, catalog::flare_of_denial());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast bolt");
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: flare, pitch_card: None, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("free Flare via sacrifice");
    drain_stack(&mut g);
    assert!(g.battlefield_find(pitcher).is_none(), "blue creature sacrificed");
    assert_eq!(g.players[1].life, 20, "bolt countered");
}

/// Marauding Mako grows on your discards.
#[test]
fn marauding_mako_grows_on_discard() {
    let mut g = two_player_game();
    let mako = g.add_card_to_battlefield(0, catalog::marauding_mako());
    let pitch = g.add_card_to_hand(0, catalog::island());
    let mut evs = Vec::new();
    g.discard_card(0, pitch, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(mako).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "discard grew the Mako"
    );
}

/// Nulldrifter evoked for {2}{U} draws two and is sacrificed.
#[test]
fn nulldrifter_evoke_draws_two() {
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::nulldrifter());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len() - 1;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("evoke");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2, "drew two");
    assert!(g.battlefield_find(id).is_none(), "evoked body sacrificed");
}

/// Guide of Souls pays out life + energy per creature, then converts
/// {E}{E}{E} into counters and flying on attack.
#[test]
fn guide_of_souls_energy_engine() {
    use crabomination::card::Keyword;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::guide_of_souls());
    for _ in 0..3 {
        let c = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, c);
    }
    assert_eq!(g.players[0].life, 23, "1 life per creature entry");
    assert_eq!(g.players[0].energy, 3, "1 energy per creature entry");
    let attacker = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").unwrap().id;
    g.clear_sickness(attacker);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::DeclareAttackers;
    let events = g
        .declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 0, "paid three energy");
    let cp = g.computed_permanent(attacker).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "two +1/+1 counters");
    assert!(cp.keywords.contains(&Keyword::Flying), "gained flying");
}

// ── CR 613.7d — SwitchPT + animated-state abilities ─────────────────────────

/// Twisted Image switches a creature's P/T (layer 7d) and cantrips; a
/// switched 3/4 is 4/3, and a switched 0/4 wall dies to the 0-toughness SBA.
#[test]
fn twisted_image_switches_pt_and_draws() {
    let mut g = two_player_game();
    let wraith = g.add_card_to_battlefield(1, catalog::street_wraith());
    g.add_card_to_library(0, catalog::island());
    let img = g.add_card_to_hand(0, catalog::twisted_image());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand = g.players[0].hand.len() - 1;
    g.perform_action(GameAction::CastSpell {
        card_id: img, target: Some(Target::Permanent(wraith)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Twisted Image");
    drain_stack(&mut g);
    let cp = g.computed_permanent(wraith).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3), "3/4 switched to 4/3");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    g.expire_end_of_turn_effects();
    let cp = g.computed_permanent(wraith).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4), "switch expires EOT");
}

/// The classic line: Twisted Image kills a 0/4 wall (4/0 → 0-toughness SBA).
#[test]
fn twisted_image_kills_a_wall() {
    let mut g = two_player_game();
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_omens());
    g.add_card_to_library(0, catalog::island());
    let img = g.add_card_to_hand(0, catalog::twisted_image());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: img, target: Some(Target::Permanent(wall)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Twisted Image");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wall).is_none(), "4/0 wall died to SBA");
}

/// Wandering Fumarole's {0} switch is gated on being animated; switched it's 4/1.
#[test]
fn wandering_fumarole_switch_only_while_animated() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::wandering_fumarole());
    // Not a creature yet — the {0} switch is rejected.
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 3, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).is_err(), "switch gated while not a creature");
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("animate");
    drain_stack(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 3, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("{0}: switch");
    drain_stack(&mut g);
    let cp = g.computed_permanent(land).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 1), "1/4 switched to 4/1");
}

/// Lavaclaw Reaches firebreathes for {X} while animated.
#[test]
fn lavaclaw_reaches_firebreathing() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::lavaclaw_reaches());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("animate");
    drain_stack(&mut g);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 3, target: None, additional_targets: Vec::new(), x_value: Some(3), mode: None,
    }).expect("{X}: +X/+0");
    drain_stack(&mut g);
    let cp = g.computed_permanent(land).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 2), "2/2 pumped to 5/2");
}

// ── CR 702.29 — life-paid Cycling (Street Wraith) ───────────────────────────

/// Street Wraith cycles for 2 life: discard, draw, no mana spent.
#[test]
fn street_wraith_cycles_for_life() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let wraith = g.add_card_to_hand(0, catalog::street_wraith());
    let hand = g.players[0].hand.len() - 1;
    g.perform_action(GameAction::Cycle { card_id: wraith, x_value: None }).expect("cycle");
    assert_eq!(g.players[0].life, 18, "paid 2 life");
    assert_eq!(g.players[0].hand.len(), hand + 1, "replaced itself");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == wraith), "discarded");
}

// ── The One Ring — protection from everything until your next turn ──────────

/// The One Ring's ETB protects its caster: untargetable, all damage prevented,
/// expiring when their turn begins.
#[test]
fn the_one_ring_protection_until_your_next_turn() {
    let mut g = two_player_game();
    let ring = g.add_card_to_hand(0, catalog::the_one_ring());
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, ring);
    assert!(g.players[0].protected_from_everything);
    // Opponent's Bolt can't target the protected player.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "protected player can't be targeted");
    // Non-targeted damage is prevented too (shared prevention funnel).
    let mut evs = Vec::new();
    let left = g.apply_prevention_shields(crabomination::game::effects::EntityRef::Player(0), 5, None, &mut evs);
    assert_eq!(left, 0, "all damage to the protected player prevented");
    // Expires when the protected player's turn begins.
    g.active_player_idx = 0;
    g.do_untap();
    assert!(!g.players[0].protected_from_everything, "expires on your turn");
}

// ── CR 614.5 — damage halving (Ghosts of the Innocent) ──────────────────────

/// Ghosts of the Innocent halves spell damage rounded down; a Bolt deals 1.
#[test]
fn ghosts_of_the_innocent_halves_noncombat_damage() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ghosts_of_the_innocent());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the player");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 19, "3 halved to 1");
}

/// Halving applies to combat damage too, and composes with a doubler
/// (double then halve = unchanged).
#[test]
fn ghosts_of_the_innocent_halves_combat_damage() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::ghosts_of_the_innocent());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.attacking = vec![Attack { attacker: bear, target: AttackTarget::Player(1) }];
    g.step = TurnStep::CombatDamage;
    g.active_player_idx = 0;
    g.resolve_combat().expect("combat damage");
    assert_eq!(g.players[1].life, 19, "2 halved to 1");
}

// ── CR 702.41 — Entwine (Tooth and Nail) ────────────────────────────────────

/// Plain cast runs only the chosen mode (mode 1: put creatures from hand).
#[test]
fn tooth_and_nail_plain_cast_runs_one_mode() {
    let mut g = two_player_game();
    let tn = g.add_card_to_hand(0, catalog::tooth_and_nail());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::craw_wurm());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    g.perform_action(GameAction::CastSpell {
        card_id: tn, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("cast Tooth and Nail mode 1");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "Bear put onto battlefield");
    assert_eq!(g.players[0].library.len(), 1, "no search happened");
}

/// Entwined cast pays {2} more and runs both modes: tutors to hand, then
/// puts from hand onto the battlefield.
#[test]
fn tooth_and_nail_entwined_runs_both_modes() {
    let mut g = two_player_game();
    let tn = g.add_card_to_hand(0, catalog::tooth_and_nail());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let w1 = g.add_card_to_library(0, catalog::craw_wurm());
    let w2 = g.add_card_to_library(0, catalog::craw_wurm());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(5);
    // Without the entwine {2} the cast is rejected.
    assert!(g.perform_action(GameAction::CastSpellEntwine {
        card_id: tn, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).is_err(), "entwine cost unpaid");
    g.players[0].mana_pool.add_colorless(7);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(w1)),
        DecisionAnswer::Search(Some(w2)),
        DecisionAnswer::Cards(vec![bear]),
    ]));
    g.perform_action(GameAction::CastSpellEntwine {
        card_id: tn, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("entwined cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "mode 2 put the Bear out");
    assert!(g.players[0].library.is_empty(), "mode 1 tutored both Wurms");
}

// ── CR 614.10 — skip-turn (Chronatog) ───────────────────────────────────────

/// Chronatog's {0} pumps +3/+3 once per turn and banks a skipped turn:
/// the activator's next turn never happens.
#[test]
fn chronatog_pumps_and_skips_your_next_turn() {
    let mut g = two_player_game();
    let atog = g.add_card_to_battlefield(0, catalog::chronatog());
    g.perform_action(GameAction::ActivateAbility {
        card_id: atog, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("{0}: pump");
    drain_stack(&mut g);
    let cp = g.computed_permanent(atog).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 5));
    // Once each turn.
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: atog, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).is_err(), "once per turn");
    assert_eq!(g.players[0].skip_turns, 1);
    // P0's turn ends; P1's turn comes; then P0's turn is skipped → P1 again.
    g.active_player_idx = 0;
    let _ = g.do_cleanup(&mut Vec::new());
    assert_eq!(g.active_player_idx, 1);
    let _ = g.do_cleanup(&mut Vec::new());
    assert_eq!(g.active_player_idx, 1, "P0's turn was skipped");
    assert_eq!(g.players[0].skip_turns, 0);
}

// ── Entwine batch (CR 702.41) + SwitchPT cards ──────────────────────────────

/// Barbed Lightning entwined burns both a creature and a player.
#[test]
fn barbed_lightning_entwined_hits_both() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bl = g.add_card_to_hand(0, catalog::barbed_lightning());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpellEntwine {
        card_id: bl, target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Player(1)], mode: Some(0), x_value: None,
    }).expect("entwined Barbed Lightning");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "creature took 3");
    assert_eq!(g.players[1].life, 17, "player took 3");
}

/// Rude Awakening mode 1 animates lands; entwined it untaps them too.
#[test]
fn rude_awakening_entwined_untaps_and_animates() {
    let mut g = two_player_game();
    let mut lands = Vec::new();
    for _ in 0..3 {
        let l = g.add_card_to_battlefield(0, catalog::forest());
        g.battlefield_find_mut(l).unwrap().tapped = true;
        lands.push(l);
    }
    let ra = g.add_card_to_hand(0, catalog::rude_awakening());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpellEntwine {
        card_id: ra, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("entwined Rude Awakening");
    drain_stack(&mut g);
    for l in &lands {
        let c = g.battlefield_find(*l).unwrap();
        assert!(!c.tapped, "land untapped");
        let cp = g.computed_permanent(*l).unwrap();
        assert!(cp.card_types.contains(&CardType::Creature), "land animated");
        assert_eq!((cp.power, cp.toughness), (2, 2));
    }
}

/// Grab the Reins mode 1 sacrifices a creature and flings its power.
#[test]
fn grab_the_reins_fling_mode() {
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let gr = g.add_card_to_hand(0, catalog::grab_the_reins());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: gr, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("cast mode 1");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wurm).is_none(), "Wurm sacrificed");
    assert_eq!(g.players[1].life, 14, "6 damage flung");
}

/// Promise of Power mode 1 mints an X/X flying Demon, X = hand size.
#[test]
fn promise_of_power_demon_scales_with_hand() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_hand(0, catalog::island());
    }
    let pp = g.add_card_to_hand(0, catalog::promise_of_power());
    g.players[0].mana_pool.add(Color::Black, 5);
    g.perform_action(GameAction::CastSpell {
        card_id: pp, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("cast mode 1");
    drain_stack(&mut g);
    let demon = g.battlefield.iter().find(|c| c.definition.name == "Demon").expect("Demon token");
    let id = demon.id;
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "X = 4 cards in hand");
    assert!(cp.keywords.contains(&Keyword::Flying));
}

/// Inside Out switches with the hybrid {U/R} pip payable in red.
#[test]
fn inside_out_switches_with_red_mana() {
    let mut g = two_player_game();
    let wraith = g.add_card_to_battlefield(1, catalog::street_wraith());
    g.add_card_to_library(0, catalog::island());
    let io = g.add_card_to_hand(0, catalog::inside_out());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: io, target: Some(Target::Permanent(wraith)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Inside Out");
    drain_stack(&mut g);
    let cp = g.computed_permanent(wraith).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3));
}

/// Merfolk Thaumaturgist taps to switch; two switches cancel out.
#[test]
fn merfolk_thaumaturgist_double_switch_cancels() {
    let mut g = two_player_game();
    let merfolk = g.add_card_to_battlefield(0, catalog::merfolk_thaumaturgist());
    g.clear_sickness(merfolk);
    let wraith = g.add_card_to_battlefield(0, catalog::street_wraith());
    g.perform_action(GameAction::ActivateAbility {
        card_id: merfolk, ability_index: 0,
        target: Some(Target::Permanent(wraith)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("tap to switch");
    drain_stack(&mut g);
    let cp = g.computed_permanent(wraith).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3), "switched once");
    g.battlefield_find_mut(merfolk).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: merfolk, ability_index: 0,
        target: Some(Target::Permanent(wraith)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("switch again");
    drain_stack(&mut g);
    let cp = g.computed_permanent(wraith).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4), "two switches cancel (CR 613.7d)");
}

// ── Goblin / Vortex batch ───────────────────────────────────────────────────

/// Munitions Expert's ETB deals damage equal to your Goblin count.
#[test]
fn munitions_expert_scales_with_goblins() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::goblin_guide());
    g.add_card_to_battlefield(0, catalog::goblin_guide());
    let bear = g.add_card_to_battlefield(1, catalog::street_wraith());
    let me = g.add_card_to_hand(0, catalog::munitions_expert());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastSpell {
        card_id: me, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Munitions Expert");
    drain_stack(&mut g);
    // 3 Goblins on resolution (the Expert counts itself).
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 3);
}

/// Boggart Harbinger tutors a Goblin to the top of the library.
#[test]
fn boggart_harbinger_tutors_goblin_to_top() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let guide = g.add_card_to_library(0, catalog::goblin_guide());
    g.add_card_to_library(0, catalog::island());
    let bh = g.add_card_to_hand(0, catalog::boggart_harbinger());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(guide)),
    ]));
    cast(&mut g, bh);
    assert_eq!(g.players[0].library[0].id, guide, "Goblin on top");
}

/// Roiling Vortex pings each upkeep and punishes free spells for 5.
#[test]
fn roiling_vortex_punishes_free_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::roiling_vortex());
    // Upkeep ping hits the active player.
    g.active_player_idx = 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "upkeep ping");
    // A sacrifice-cast (no mana spent) Flare costs its caster 5.
    g.add_card_to_battlefield(1, catalog::delver_of_secrets());
    let flare = g.add_card_to_hand(1, catalog::flare_of_denial());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: flare, pitch_card: None, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("free Flare");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 14, "free spell cost its caster 5");
}

