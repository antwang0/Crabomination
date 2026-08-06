//! Exodus (EXO) — `catalog::sets::exo`.

use crabomination::card::{CardId, CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

fn activate(
    g: &mut GameState,
    id: CardId,
    index: usize,
    target: Option<Target>,
) -> Result<(), crabomination::game::GameError> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map(|_| ())
}

/// The three ETB recursion creatures each pull their own card type back.
#[test]
fn the_etb_recursion_cycle_returns_its_own_card_type() {
    for (def, buried) in [
        (catalog::anarchist(), catalog::stone_rain() as _),
        (catalog::cartographer(), catalog::forest()),
        (catalog::scrivener(), catalog::lightning_bolt()),
    ] {
        let mut g = two_player_game();
        let card = g.add_card_to_graveyard(0, buried);
        let decoy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let etb = def.triggered_abilities[0].effect.clone();
        let body = g.add_card_to_battlefield(0, def);
        let ctx = crabomination::game::effects::EffectContext::for_ability(
            body,
            0,
            Some(Target::Permanent(card)),
        );
        g.resolve_effect(&etb, &ctx).expect("etb");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == card));
        assert!(g.players[0].graveyard.iter().any(|c| c.id == decoy), "only the named type");
    }
}

/// Dauthi Warlord's power counts every shadow creature on the battlefield,
/// including its opponents'.
#[test]
fn dauthi_warlord_counts_the_whole_shadow_board() {
    let mut g = two_player_game();
    let lord = g.add_card_to_battlefield(0, catalog::dauthi_warlord());
    assert_eq!(g.computed_permanent(lord).unwrap().power, 1, "it counts itself");
    g.add_card_to_battlefield(1, catalog::dauthi_cutthroat());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cp = g.computed_permanent(lord).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 1), "the Bears aren't shadow");
}

/// Dauthi Cutthroat's activated kill only sees other shadow creatures.
#[test]
fn dauthi_cutthroat_only_targets_shadow() {
    let mut g = two_player_game();
    let cutthroat = g.add_card_to_battlefield(0, catalog::dauthi_cutthroat());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let jackal = g.add_card_to_battlefield(1, catalog::dauthi_jackal());
    g.clear_sickness(cutthroat);
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(activate(&mut g, cutthroat, 0, Some(Target::Permanent(bear))).is_err());
    activate(&mut g, cutthroat, 0, Some(Target::Permanent(jackal))).expect("kill the Jackal");
    drain_stack(&mut g);
    assert!(g.battlefield_find(jackal).is_none());
}

/// Bequeathal cashes in when the creature it enchants dies.
#[test]
fn bequeathal_draws_two_on_the_hosts_death() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::bequeathal());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::forest());
    }
    let hand = g.players[0].hand.len();
    let mut evs = vec![];
    g.destroy_permanent(bear, false, &mut evs);
    evs.extend(g.check_state_based_actions());
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

/// The three static Auras land their printed riders on the host.
#[test]
fn the_static_auras_ride_their_hosts() {
    for (def, want_pt, want_kw) in [
        (catalog::cursed_flesh(), (1, 1), Keyword::Fear),
        (catalog::maniacal_rage(), (4, 4), Keyword::CantBlock),
        (catalog::robe_of_mirrors(), (2, 2), Keyword::Shroud),
    ] {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_battlefield(0, def);
        g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), want_pt);
        assert!(cp.keywords.contains(&want_kw));
    }
}

/// Memory Crystal knocks {2} off a buyback surcharge.
#[test]
fn memory_crystal_discounts_buyback() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::memory_crystal());
    let allay = g.add_card_to_hand(0, catalog::allay());
    let target = g.add_card_to_battlefield(1, catalog::glorious_anthem());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // {1}{W} plus a buyback {3} discounted to {1}.
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpellBuyback {
        card_id: allay,
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("buyback cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none());
    assert!(g.players[0].hand.iter().any(|c| c.id == allay), "bought back");
}

/// Pegasus Stampede's buyback eats a land and leaves a flier behind.
#[test]
fn pegasus_stampede_buys_back_for_a_land() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::pegasus_stampede());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::plains());
    }
    let lands = g.battlefield.iter().filter(|c| c.definition.is_land()).count();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpellBuyback {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("buyback cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == spell));
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_land()).count(), lands - 1);
    assert!(
        g.battlefield
            .iter()
            .any(|c| c.definition.name == "Pegasus" && c.has_keyword(&Keyword::Flying))
    );
}

/// Hatred's X is paid in life and lands as raw power.
#[test]
fn hatred_pays_life_for_power() {
    let mut g = two_player_game();
    let hatred = g.add_card_to_hand(0, catalog::hatred());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: hatred,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(7),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 13);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 9);
}

/// Carnophage taps itself when its controller declines the life payment.
#[test]
fn carnophage_taps_when_you_wont_bleed() {
    let mut g = two_player_game();
    let zombie = g.add_card_to_battlefield(0, catalog::carnophage());
    let def = catalog::carnophage();
    let ctx = crabomination::game::effects::EffectContext::for_ability(zombie, 0, None);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    g.resolve_effect(&def.triggered_abilities[0].effect, &ctx).expect("upkeep");
    assert!(g.battlefield_find(zombie).unwrap().tapped);
    assert_eq!(g.players[0].life, 20);

    g.battlefield_find_mut(zombie).unwrap().tapped = false;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.resolve_effect(&def.triggered_abilities[0].effect, &ctx).expect("upkeep");
    assert!(!g.battlefield_find(zombie).unwrap().tapped);
    assert_eq!(g.players[0].life, 19);
}

/// Mindless Automaton eats a card to grow and two counters to draw.
#[test]
fn mindless_automaton_trades_cards_for_counters_and_back() {
    let mut g = two_player_game();
    let bot = g.add_card_to_battlefield_with_counters(0, catalog::mindless_automaton());
    g.add_card_to_hand(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    assert_eq!(g.computed_permanent(bot).unwrap().power, 2);

    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, bot, 0, None).expect("grow");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bot).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);

    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    activate(&mut g, bot, 1, None).expect("draw");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bot).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Furnace Brood blanks a regeneration shield for the turn (CR 701.19c).
#[test]
fn furnace_brood_turns_off_regeneration() {
    let mut g = two_player_game();
    let brood = g.add_card_to_battlefield(0, catalog::furnace_brood());
    let troll = g.add_card_to_battlefield(1, catalog::pygmy_troll());
    g.battlefield_find_mut(troll).unwrap().regeneration_shields = 1;
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Red, 1);
    activate(&mut g, brood, 0, Some(Target::Permanent(troll))).expect("blank the shield");
    drain_stack(&mut g);
    g.destroy_permanent(troll, false, &mut vec![]);
    g.check_state_based_actions();
    assert!(g.battlefield_find(troll).is_none(), "the shield didn't apply");
}

/// Rootwater Alligator eats a Forest instead of paying mana.
#[test]
fn rootwater_alligator_sacrifices_a_forest_to_regenerate() {
    let mut g = two_player_game();
    let gator = g.add_card_to_battlefield(0, catalog::rootwater_alligator());
    g.step = TurnStep::PreCombatMain;
    assert!(activate(&mut g, gator, 0, None).is_err(), "no Forest to eat");
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    activate(&mut g, gator, 0, None).expect("regenerate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(forest).is_none());
    assert_eq!(g.battlefield_find(gator).unwrap().regeneration_shields, 1);
}

/// Rabid Wolverines grows once per creature that blocks it.
#[test]
fn rabid_wolverines_grows_when_blocked() {
    let mut g = two_player_game();
    let wolverines = g.add_card_to_battlefield(0, catalog::rabid_wolverines());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(wolverines);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: wolverines,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, wolverines)])).expect("block");
    drain_stack(&mut g);
    let cp = g.computed_permanent(wolverines).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
}

/// Exalted Dragon pays a land as attackers are declared.
#[test]
fn exalted_dragon_sacrifices_a_land_to_attack() {
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(0, catalog::exalted_dragon());
    g.clear_sickness(dragon);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    let attack = vec![Attack { attacker: dragon, target: AttackTarget::Player(1) }];
    assert!(g.perform_action(GameAction::DeclareAttackers(attack.clone())).is_err());
    let plains = g.add_card_to_battlefield(0, catalog::plains());
    g.perform_action(GameAction::DeclareAttackers(attack)).expect("attack");
    assert!(g.battlefield_find(plains).is_none(), "a land paid the attack cost");
}

/// Plated Rootwalla's pump is once each turn.
#[test]
fn plated_rootwalla_pumps_only_once_a_turn() {
    let mut g = two_player_game();
    let walla = g.add_card_to_battlefield(0, catalog::plated_rootwalla());
    g.step = TurnStep::PreCombatMain;
    for _ in 0..2 {
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
    }
    activate(&mut g, walla, 0, None).expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(walla).unwrap().power, 6);
    assert!(activate(&mut g, walla, 0, None).is_err(), "once each turn");
}

/// Grollub's wounds are life for the table.
#[test]
fn grollub_pays_its_damage_out_as_life() {
    let mut g = two_player_game();
    let grollub = g.add_card_to_battlefield(0, catalog::grollub());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(grollub)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 23);
}

/// Convalescence only ticks while you're at 10 life or less.
#[test]
fn convalescence_waits_until_youre_low() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::convalescence());
    g.active_player_idx = 0;
    g.step = TurnStep::Untap;
    advance_to(&mut g, TurnStep::Draw);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "healthy players get nothing");

    g.players[0].life = 8;
    g.active_player_idx = 0;
    g.step = TurnStep::Untap;
    advance_to(&mut g, TurnStep::Draw);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 9);
}

/// Mana Breach bounces a land from whoever cast the spell.
#[test]
fn mana_breach_bounces_the_casters_land() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mana_breach());
    let forest = g.add_card_to_battlefield(1, catalog::forest());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == forest), "the caster paid the land");
}

/// Death's Duet pulls two creature cards back at once.
#[test]
fn deaths_duet_returns_two_creature_cards() {
    let mut g = two_player_game();
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::serra_angel());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let duet = g.add_card_to_hand(0, catalog::deaths_duet());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: duet,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == a));
    assert!(g.players[0].hand.iter().any(|c| c.id == b));
}

/// The plain bodies come out with the right stats and keywords.
#[test]
fn the_exodus_vanilla_bodies_match_their_printings() {
    for (def, cost, pt, kws) in [
        (catalog::sabertooth_wyvern(), 5, (3, 2), vec![Keyword::Flying, Keyword::FirstStrike]),
        (
            catalog::mirri_cat_warrior(),
            3,
            (2, 3),
            vec![Keyword::FirstStrike, Keyword::Vigilance],
        ),
        (
            catalog::paladin_en_vec(),
            3,
            (2, 2),
            vec![Keyword::FirstStrike, Keyword::Protection(Color::Black)],
        ),
    ] {
        assert_eq!(def.cost.cmc(), cost, "{}", def.name);
        assert_eq!((def.power, def.toughness), pt, "{}", def.name);
        assert!(def.card_types.contains(&CardType::Creature));
        for kw in kws {
            assert!(def.keywords.contains(&kw), "{} is missing {kw:?}", def.name);
        }
    }
}

/// The Keeper cycle only fires while the chosen opponent is actually ahead.
#[test]
fn the_keeper_cycle_needs_an_opponent_who_is_ahead() {
    let mut g = two_player_game();
    let keeper = g.add_card_to_battlefield(0, catalog::keeper_of_the_flame());
    g.clear_sickness(keeper);
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Red, 1);
    assert!(
        activate(&mut g, keeper, 0, Some(Target::Player(1))).is_err(),
        "life totals are level"
    );

    g.players[1].life = 25;
    activate(&mut g, keeper, 0, Some(Target::Player(1))).expect("burn the leader");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 23);
}

/// Keeper of the Mind wants a two-card hand gap, not just one.
#[test]
fn keeper_of_the_mind_wants_a_two_card_gap() {
    let mut g = two_player_game();
    let keeper = g.add_card_to_battlefield(0, catalog::keeper_of_the_mind());
    g.clear_sickness(keeper);
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_hand(1, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Blue, 1);
    assert!(activate(&mut g, keeper, 0, Some(Target::Player(1))).is_err(), "only one card up");

    g.add_card_to_hand(1, catalog::forest());
    activate(&mut g, keeper, 0, Some(Target::Player(1))).expect("draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Keeper of the Dead reads the comparison the other way round.
#[test]
fn keeper_of_the_dead_wants_their_graveyard_to_trail_yours() {
    let mut g = two_player_game();
    let keeper = g.add_card_to_battlefield(0, catalog::keeper_of_the_dead());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(keeper);
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Black, 1);
    let pick = |g: &mut GameState| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: keeper,
            ability_index: 0,
            target: Some(Target::Player(1)),
            additional_targets: vec![Target::Permanent(victim)],
            mode: None,
            x_value: None,
        })
    };
    assert!(pick(&mut g).is_err(), "graveyards are level");

    for _ in 0..2 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    pick(&mut g).expect("kill it");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none());
}
