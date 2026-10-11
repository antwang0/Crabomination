//! Commander: EDHREC average-deck gaps, fourth batch (`decks::cmdr_edhrec4`):
//! Shroofus Sproutsire's Saprolings; Miirym's and Tiamat's Dragons.

use crabomination::card::{CardId, CounterType, Keyword, Value};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector};
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

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast(g: &mut GameState, id: CardId) {
    cast_at(g, id, None);
}

fn cast_at(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, ability_index: usize) -> Result<(), crabomination::game::GameError> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == name).count()
}

fn kill(g: &mut GameState, id: CardId) {
    let ctx = EffectContext::for_spell(1, None, 0, 0);
    let evs = g.resolve_effect(&Effect::Destroy { what: Selector::ExactObjects(vec![id]) }, &ctx).expect("destroy");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

/// Sporecrown Thallid pumps other Fungi and Saprolings only; the Thallid
/// cycle banks a spore counter each upkeep and cashes three for a
/// Saproling; Utopia Mycon's Saproling is a mana ability.
#[test]
fn thallids_bank_spores_into_saprolings() {
    let mut g = pod(2);
    let crown = ready(&mut g, 0, catalog::sporecrown_thallid());
    let mycon = ready(&mut g, 0, catalog::utopia_mycon());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(mycon).unwrap().power, 1, "a Fungus");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "not a Fungus");
    assert_eq!(g.computed_permanent(crown).unwrap().power, 2, "not itself");
    for turn in 0..3 {
        assert!(activate(&mut g, mycon, 0).is_err() || turn == 3, "not yet three spores");
        g.step = TurnStep::Upkeep;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(mycon).unwrap().counter_count(CounterType::Spore), 3);
    activate(&mut g, mycon, 0).expect("remove three");
    assert_eq!(named(&g, "Saproling"), 1);
    let sap = g.battlefield.iter().find(|c| c.definition.name == "Saproling").unwrap().id;
    assert_eq!(g.computed_permanent(sap).unwrap().power, 2, "Sporecrown");
    g.players[0].mana_pool = Default::default();
    activate(&mut g, mycon, 1).expect("sacrifice for mana");
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

/// Fungal Sprouting counts the greatest power; Might of the Masses counts
/// creatures you control; Undercellar Myconid makes one entering and one dying.
#[test]
fn saproling_spells_scale_with_the_board() {
    let mut g = pod(2);
    let myconid = g.add_card_to_hand(0, catalog::undercellar_myconid());
    flood(&mut g);
    cast(&mut g, myconid);
    assert_eq!(named(&g, "Saproling"), 1);
    ready(&mut g, 0, catalog::colossal_dreadmaw());
    let sprouting = g.add_card_to_hand(0, catalog::fungal_sprouting());
    cast(&mut g, sprouting);
    assert_eq!(named(&g, "Saproling"), 7, "the Dreadmaw's 6");
    kill(&mut g, myconid);
    assert_eq!(named(&g, "Saproling"), 8, "and one on death");
}

/// Fungal Plots: the exile is a creature card from your graveyard, and the
/// two-Saproling sacrifice gains 2 and draws.
#[test]
fn fungal_plots_turns_corpses_into_cards() {
    let mut g = pod(2);
    let plots = ready(&mut g, 0, catalog::fungal_plots());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    flood(&mut g);
    assert!(activate(&mut g, plots, 0).is_err(), "no creature card to exile");
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    activate(&mut g, plots, 0).expect("exile one");
    activate(&mut g, plots, 0).expect("exile the other");
    let hand = g.players[0].hand.len();
    activate(&mut g, plots, 1).expect("two Saprolings");
    assert_eq!((g.players[0].life, g.players[0].hand.len()), (22, hand + 1));
}

fn advance_to_turn_of(g: &mut GameState, seat: usize) {
    loop {
        let ev = g.advance_step(Vec::new()).expect("step");
        g.dispatch_triggers_for_events(&ev);
        drain_stack(g);
        if g.active_player_idx == seat && g.step == TurnStep::PreCombatMain {
            break;
        }
    }
}

fn has_kw(g: &GameState, id: CardId, kw: Keyword) -> bool {
    g.computed_permanent(id).is_some_and(|c| c.keywords().contains(&kw))
}

/// CR 106.6 — Jade Orb's rider: the Dragon creature spell its {G} paid for
/// enters with a +1/+1 counter and hexproof that lasts until its caster's
/// next turn; Scaled Nurturer's gains 2 life. Mana from elsewhere does neither.
#[test]
fn dragon_mana_riders_name_the_dragon_they_paid_for() {
    let mut g = pod(2);
    let orb = ready(&mut g, 0, catalog::jade_orb_of_dragonkind());
    activate(&mut g, orb, 0).expect("Orb's {G}");
    g.players[0].mana_pool.add_colorless(1);
    let whelp = g.add_card_to_hand(0, catalog::scaled_nurturer());
    cast(&mut g, whelp);
    assert_eq!(g.battlefield_find(whelp).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(has_kw(&g, whelp, Keyword::Hexproof));
    let nurturer = ready(&mut g, 0, catalog::scaled_nurturer());
    activate(&mut g, nurturer, 0).expect("Nurturer's {G}");
    g.players[0].mana_pool.add_colorless(1);
    let second = g.add_card_to_hand(0, catalog::scaled_nurturer());
    cast(&mut g, second);
    assert_eq!(g.players[0].life, 22, "a Dragon paid for by the Nurturer");
    assert!(!has_kw(&g, second, Keyword::Hexproof), "not the Orb's mana");
    // The hexproof ends as its caster's next turn begins.
    stock_libraries(&mut g, 5);
    advance_to_turn_of(&mut g, 1);
    assert!(has_kw(&g, whelp, Keyword::Hexproof), "through the opponent's turn");
    advance_to_turn_of(&mut g, 0);
    assert!(!has_kw(&g, whelp, Keyword::Hexproof));
}

/// Earthquake Dragon costs {X} less for the total mana value of your
/// Dragons; Korlessa casts Dragon spells off the top of the library.
#[test]
fn dragons_make_dragons_cheaper_and_castable_from_the_top() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::thrakkus_the_butcher());
    ready(&mut g, 0, catalog::korlessa_scale_singer());
    let quake = g.add_card_to_library(0, catalog::earthquake_dragon());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(7);
    cast(&mut g, quake);
    assert!(g.battlefield_find(quake).is_some(), "{{14}}{{G}} less 5 + 2, from the top");
    assert_eq!(g.players[0].mana_pool.total(), 0);
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
            .is_err(),
        "not a Dragon"
    );
}

/// Rith: an opponent's creature dealt excess damage (CR 120.10) makes the
/// end-step Dragon; lethal-exact damage does not.
#[test]
fn rith_rewards_excess_damage() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::rith_liberated_primeval());
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 2, catalog::grizzly_bears());
    let burn = |g: &mut GameState, id: CardId, n: i32| {
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        let evs = g
            .resolve_effect(&Effect::DealDamage { to: Selector::ExactObjects(vec![id]), amount: Value::Const(n) }, &ctx)
            .expect("burn");
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    };
    let end_step = |g: &mut GameState| {
        g.step = TurnStep::End;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(g);
    };
    burn(&mut g, a, 2);
    end_step(&mut g);
    assert_eq!(named(&g, "Dragon"), 0, "exactly lethal");
    burn(&mut g, b, 5);
    end_step(&mut g);
    assert_eq!(named(&g, "Dragon"), 1);
}

/// Rivaz: one Dragon creature spell a turn from the graveyard, and it comes
/// back with "When this creature dies, exile it."
#[test]
fn rivaz_recasts_a_dragon_that_dies_into_exile() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::rivaz_of_the_claw());
    let first = g.add_card_to_graveyard(0, catalog::scaled_nurturer());
    let second = g.add_card_to_graveyard(0, catalog::scaled_nurturer());
    flood(&mut g);
    cast(&mut g, first);
    assert!(g.battlefield_find(first).is_some());
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell { card_id: second, target: None, additional_targets: vec![], mode: None, x_value: None })
            .is_err(),
        "once a turn"
    );
    kill(&mut g, first);
    assert!(g.exile.iter().any(|c| c.id == first), "its granted trigger exiled it");
}

/// Dragon's Fire: 3 damage, or a beheld Dragon's power when that is more
/// (CR 701.4a — a Dragon card revealed from hand counts).
#[test]
fn dragons_fire_reads_the_beheld_dragons_power() {
    let mut g = pod(2);
    let maw = ready(&mut g, 1, catalog::colossal_dreadmaw());
    flood(&mut g);
    let fire = g.add_card_to_hand(0, catalog::dragons_fire());
    cast_at(&mut g, fire, Some(Target::Permanent(maw)));
    assert_eq!(g.battlefield_find(maw).unwrap().damage, 3);
    g.add_card_to_hand(0, catalog::wrathful_red_dragon());
    let maw2 = ready(&mut g, 1, catalog::colossal_dreadmaw());
    let fire2 = g.add_card_to_hand(0, catalog::dragons_fire());
    cast_at(&mut g, fire2, Some(Target::Permanent(maw2)));
    assert_eq!(g.battlefield_find(maw2).unwrap().damage, 5, "the revealed 5/5");
}

/// Thrakkus doubles each Dragon's power as it attacks; Sarkhan Unbroken's
/// −8 puts every Dragon creature card in the library onto the battlefield.
#[test]
fn thrakkus_and_sarkhan_scale_the_dragons() {
    let mut g = pod(2);
    let thrakkus = ready(&mut g, 0, catalog::thrakkus_the_butcher());
    let wrath = ready(&mut g, 0, catalog::wrathful_red_dragon());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: thrakkus, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(wrath).unwrap().power, 10);
    assert_eq!(g.computed_permanent(thrakkus).unwrap().power, 6);

    let mut g = pod(2);
    let sarkhan = ready(&mut g, 0, catalog::sarkhan_unbroken());
    g.battlefield_find_mut(sarkhan).unwrap().add_counters(CounterType::Loyalty, 4);
    for def in [catalog::wrathful_red_dragon(), catalog::grizzly_bears(), catalog::scaled_nurturer()] {
        g.add_card_to_library(0, def);
    }
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: sarkhan, ability_index: 2, target: None, x_value: None })
        .expect("-8");
    drain_stack(&mut g);
    assert_eq!(named(&g, "Wrathful Red Dragon") + named(&g, "Scaled Nurturer"), 2);
    assert_eq!(named(&g, "Grizzly Bears"), 0);
}

/// Insidious Fungus' draw mode also puts a land from hand onto the
/// battlefield tapped (it shipped without that half).
#[test]
fn insidious_fungus_draws_then_drops_a_land() {
    let mut g = pod(2);
    let fungus = ready(&mut g, 0, catalog::insidious_fungus());
    g.add_card_to_library(0, catalog::island());
    let forest = g.add_card_to_hand(0, catalog::forest());
    flood(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: fungus,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: Some(2),
    })
    .expect("draw mode");
    drain_stack(&mut g);
    let land = g.battlefield_find(forest).expect("the Forest entered");
    assert!(land.tapped);
    assert_eq!(g.players[0].hand.len(), 1, "the Island drawn");
}
