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

// ── Wave 2 (`catalog::sets::exo2`) ───────────────────────────────────────────

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast(
    g: &mut GameState,
    id: CardId,
    target: Option<Target>,
) -> Result<(), crabomination::game::GameError> {
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map(|_| ())
}

/// Cinder Crawler's pump is gated on being blocked.
#[test]
fn cinder_crawler_only_pumps_once_blocked() {
    let mut g = two_player_game();
    let crawler = ready(&mut g, 0, catalog::cinder_crawler());
    let wall = ready(&mut g, 1, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Red, 1);
    assert!(activate(&mut g, crawler, 0, None).is_err(), "unblocked");

    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: crawler,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, crawler)])).expect("block");
    g.players[0].mana_pool.add(Color::Red, 1);
    activate(&mut g, crawler, 0, None).expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(crawler).unwrap().power, 2);
}

/// Elvish Berserker grows once per creature in the blocking gang.
#[test]
fn elvish_berserker_scales_with_the_blocking_gang() {
    let mut g = two_player_game();
    let berserker = ready(&mut g, 0, catalog::elvish_berserker());
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: berserker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(a, berserker), (b, berserker)]))
        .expect("block");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(berserker).unwrap().power, 3);
}

/// Jackalope Herd bounces itself off any spell its controller casts.
#[test]
fn jackalope_herd_bounces_itself_on_your_next_spell() {
    let mut g = two_player_game();
    let herd = ready(&mut g, 0, catalog::jackalope_herd());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
    drain_stack(&mut g);
    assert!(g.battlefield_find(herd).is_none());
    assert!(g.players[0].hand.iter().any(|c| c.id == herd));
}

/// Mirozel flickers home the moment anything targets it.
#[test]
fn mirozel_returns_itself_when_targeted() {
    let mut g = two_player_game();
    let mirozel = ready(&mut g, 0, catalog::mirozel());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(mirozel)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt it");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == mirozel), "it dodged");
}

/// Soltari Visionary eats an enchantment off the player it connects with.
#[test]
fn soltari_visionary_destroys_an_enchantment_on_connect() {
    let mut g = two_player_game();
    let visionary = ready(&mut g, 0, catalog::soltari_visionary());
    let ench = g.add_card_to_battlefield(1, catalog::treasure_trove());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: visionary,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    advance_to(&mut g, TurnStep::End);
    println!("LIFE {} BF {:?}", g.players[1].life, g.battlefield.iter().map(|c| c.definition.name).collect::<Vec<_>>());
    assert!(g.battlefield_find(ench).is_none());
}

/// Welkin Hawk tutors up its own twin when it dies.
#[test]
fn welkin_hawk_fetches_another_welkin_hawk() {
    let mut g = two_player_game();
    let hawk = g.add_card_to_battlefield(0, catalog::welkin_hawk());
    let twin = g.add_card_to_library(0, catalog::welkin_hawk());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(twin)),
    ]));
    let mut events = Vec::new();
    g.destroy_permanent(hawk, false, &mut events);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == twin));
}

/// Zealots en-Dal only pays out while every nonland permanent you have is white.
#[test]
fn zealots_en_dal_wants_a_mono_white_board() {
    let mut g = two_player_game();
    for seat in 0..2 {
        for _ in 0..20 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    ready(&mut g, 0, catalog::zealots_en_dal());
    let intruder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    advance_to(&mut g, TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "the green bear blocks it");

    g.battlefield.retain(|c| c.id != intruder);
    advance_to(&mut g, TurnStep::End);
    advance_to(&mut g, TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 21);
}

/// Spike Rogue trades counters in both directions.
#[test]
fn spike_rogue_moves_counters_both_ways() {
    let mut g = two_player_game();
    let rogue = g.add_card_to_battlefield_with_counters(0, catalog::spike_rogue());
    g.clear_sickness(rogue);
    // The 0/3 Wall stays the lowest-power donor, so the "remove a counter from
    // a creature you control" half is unambiguous.
    let wall = ready(&mut g, 0, catalog::wall_of_wood());
    assert_eq!(g.battlefield_find(rogue).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Green, 2);
    activate(&mut g, rogue, 0, Some(Target::Permanent(wall))).expect("feed the wall");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(rogue).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(wall).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);

    g.players[0].mana_pool.add(Color::Green, 2);
    activate(&mut g, rogue, 1, None).expect("take it back");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(rogue).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(g.battlefield_find(wall).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Workhorse turns its counters into colorless mana.
#[test]
fn workhorse_burns_counters_for_mana() {
    let mut g = two_player_game();
    let horse = g.add_card_to_battlefield_with_counters(0, catalog::workhorse());
    g.clear_sickness(horse);
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, horse, 0, None).expect("tap for mana");
    assert_eq!(g.battlefield_find(horse).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

/// Thopter Squadron converts counters to Thopters and Thopters back to counters.
#[test]
fn thopter_squadron_cycles_counters_and_tokens() {
    let mut g = two_player_game();
    let squad = g.add_card_to_battlefield_with_counters(0, catalog::thopter_squadron());
    g.clear_sickness(squad);
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, squad, 0, None).expect("mint a Thopter");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(squad).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Thopter").count(), 1);

    activate(&mut g, squad, 1, None).expect("eat it again");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(squad).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}

/// Shackles locks its host down and can be picked back up for {W}.
#[test]
fn shackles_locks_then_returns_to_hand() {
    let mut g = two_player_game();
    let victim = ready(&mut g, 1, catalog::grizzly_bears());
    let shackles = g.add_card_to_hand(0, catalog::shackles());
    g.players[0].mana_pool.add(Color::White, 4);
    cast(&mut g, shackles, Some(Target::Permanent(victim))).expect("enchant");
    drain_stack(&mut g);
    g.battlefield_find_mut(victim).unwrap().tapped = true;
    g.do_untap();
    assert!(g.battlefield_find(victim).unwrap().tapped, "locked down");

    activate(&mut g, shackles, 0, None).expect("pick it up");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == shackles));
}

/// Song of Serenity benches every enchanted creature, whoever owns the Aura.
#[test]
fn song_of_serenity_benches_enchanted_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::song_of_serenity());
    let victim = ready(&mut g, 1, catalog::grizzly_bears());
    let free = ready(&mut g, 1, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(1, catalog::maniacal_rage());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(victim);
    assert!(g.computed_permanent(victim).unwrap().keywords.contains(&Keyword::CantAttack));
    assert!(!g.computed_permanent(free).unwrap().keywords.contains(&Keyword::CantAttack));
}

/// Spellshock taxes both players two life per spell.
#[test]
fn spellshock_bites_every_caster() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::spellshock());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 18, "its own controller pays too");
}

/// Null Brooch pitches your hand to stop a noncreature spell.
#[test]
fn null_brooch_counters_only_noncreature_spells() {
    let mut g = two_player_game();
    let brooch = ready(&mut g, 0, catalog::null_brooch());
    g.add_card_to_hand(0, catalog::forest());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, brooch, 0, Some(Target::Permanent(bolt))).expect("counter");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20);
    assert!(g.players[0].hand.is_empty(), "the hand was the cost");
}

/// Slaughter's buyback is four life, and the kill can't be regenerated.
#[test]
fn slaughter_buys_back_for_four_life() {
    let mut g = two_player_game();
    let victim = ready(&mut g, 1, catalog::grizzly_bears());
    let slaughter = g.add_card_to_hand(0, catalog::slaughter());
    g.players[0].mana_pool.add(Color::Black, 4);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellBuyback {
        card_id: slaughter,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("buyback");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none());
    assert_eq!(g.players[0].life, 16);
    assert!(g.players[0].hand.iter().any(|c| c.id == slaughter));
}

/// Necrologia only casts in your end step and pays its X in life.
#[test]
fn necrologia_is_an_end_step_life_for_cards_trade() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::necrologia());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.players[0].mana_pool.add(Color::Black, 5);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let go = |g: &mut GameState| {
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: Some(3),
        })
    };
    assert!(go(&mut g).is_err(), "main phase is too early");

    g.step = TurnStep::End;
    g.priority.player_with_priority = 0;
    go(&mut g).expect("cast in the end step");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 3);
    assert_eq!(g.players[0].life, 17);
}

/// Sonic Burst's random discard is a real additional cost.
#[test]
fn sonic_burst_costs_a_random_card() {
    let mut g = two_player_game();
    let burst = g.add_card_to_hand(0, catalog::sonic_burst());
    g.players[0].mana_pool.add(Color::Red, 2);
    assert!(cast(&mut g, burst, Some(Target::Player(1))).is_err(), "no card to pitch");

    g.add_card_to_hand(0, catalog::forest());
    cast(&mut g, burst, Some(Target::Player(1))).expect("burst");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 16);
    assert_eq!(g.players[0].graveyard.len(), 2, "the burst and the pitched card");
}

/// Resuscitate hands your whole team a regeneration ability for the turn.
#[test]
fn resuscitate_grants_the_team_regeneration() {
    let mut g = two_player_game();
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::resuscitate());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, spell, None).expect("resuscitate");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::Green, 1);
    activate(&mut g, bear, 0, None).expect("regenerate");
    drain_stack(&mut g);
    let mut events = Vec::new();
    g.destroy_permanent(bear, false, &mut events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "the shield held");
}

/// Mind Maggots converts pitched creature cards into two counters each.
#[test]
fn mind_maggots_eats_creature_cards_for_counters() {
    let mut g = two_player_game();
    let a = g.add_card_to_hand(0, catalog::grizzly_bears());
    let b = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![a, b])]));
    let etb = catalog::mind_maggots().triggered_abilities[0].effect.clone();
    let maggots = g.add_card_to_battlefield(0, catalog::mind_maggots());
    let ctx = crabomination::game::effects::EffectContext::for_ability(maggots, 0, None);
    g.resolve_effect(&etb, &ctx).expect("etb");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "only the creature cards were eligible");
    assert_eq!(g.battlefield_find(maggots).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
}

/// Thrull Surgeon sacrifices itself to pick a card out of a hand.
#[test]
fn thrull_surgeon_picks_the_discard() {
    let mut g = two_player_game();
    let surgeon = ready(&mut g, 0, catalog::thrull_surgeon());
    let prize = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![prize])]));
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Black, 2);
    activate(&mut g, surgeon, 0, Some(Target::Player(1))).expect("operate");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == prize));
    assert!(g.battlefield_find(surgeon).is_none(), "it was the cost");
}

/// Aether Tide's X is paid in discarded creature cards and bounces that many.
#[test]
fn aether_tide_trades_creature_cards_for_bounces() {
    let mut g = two_player_game();
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::grizzly_bears());
    let tide = g.add_card_to_hand(0, catalog::aether_tide());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: tide,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: Some(2),
    })
    .expect("tide");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}

/// Oath of Lieges only fires for the player who's behind on lands.
#[test]
fn oath_of_lieges_only_helps_the_player_behind() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::oath_of_lieges());
    g.add_card_to_battlefield(0, catalog::forest());
    let fetched = g.add_card_to_library(1, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(fetched)),
    ]));
    // `advance_to` lands on seat 1's upkeep; they trail seat 0 on lands.
    advance_to(&mut g, TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.active_player_idx, 1);
    assert!(g.battlefield_find(fetched).is_some());
}

/// Oath of Mages pings the player who's ahead on life.
#[test]
fn oath_of_mages_pings_the_life_leader() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::oath_of_mages());
    // `advance_to` lands on seat 1's upkeep, so seat 0 has to be the leader.
    g.players[0].life = 25;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    advance_to(&mut g, TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 24);
}

/// Cataclysm keeps one land as well as one of each nonland type.
#[test]
fn cataclysm_spares_one_of_each_type_including_a_land() {
    let mut g = two_player_game();
    for seat in 0..2 {
        for _ in 0..3 {
            g.add_card_to_battlefield(seat, catalog::forest());
        }
        g.add_card_to_battlefield(seat, catalog::grizzly_bears());
        g.add_card_to_battlefield(seat, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::cataclysm());
    g.players[0].mana_pool.add(Color::White, 4);
    cast(&mut g, spell, None).expect("cataclysm");
    drain_stack(&mut g);
    for seat in 0..2 {
        let mine: Vec<_> = g.battlefield.iter().filter(|c| c.controller == seat).collect();
        assert_eq!(mine.len(), 2, "one land + one creature");
    }
}

/// Entropic Specter sizes itself to the opponent it named as it entered.
#[test]
fn entropic_specter_reads_the_chosen_hand() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::forest());
    }
    let specter = g.add_card_to_hand(0, catalog::entropic_specter());
    g.players[0].mana_pool.add(Color::Black, 5);
    cast(&mut g, specter, None).expect("cast it");
    drain_stack(&mut g);
    let cp = g.computed_permanent(specter).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
}

/// Monstrous Hound needs a land lead to attack at all.
#[test]
fn monstrous_hound_needs_a_land_lead() {
    let mut g = two_player_game();
    let hound = ready(&mut g, 0, catalog::monstrous_hound());
    g.add_card_to_battlefield(1, catalog::forest());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let attack = |g: &mut GameState| {
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: hound,
            target: AttackTarget::Player(1),
        }]))
    };
    assert!(attack(&mut g).is_err(), "level on lands");

    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::forest());
    attack(&mut g).expect("now it's ahead");
}

/// Spike Cannibal hoovers up every +1/+1 counter on the board.
#[test]
fn spike_cannibal_steals_every_counter() {
    let mut g = two_player_game();
    let donor = g.add_card_to_battlefield_with_counters(1, catalog::spike_hatcher());
    let etb = catalog::spike_cannibal().triggered_abilities[0].effect.clone();
    let cannibal = g.add_card_to_battlefield_with_counters(0, catalog::spike_cannibal());
    let ctx = crabomination::game::effects::EffectContext::for_ability(cannibal, 0, None);
    g.resolve_effect(&etb, &ctx).expect("etb");
    assert_eq!(g.battlefield_find(donor).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!(g.battlefield_find(cannibal).unwrap().counter_count(CounterType::PlusOnePlusOne), 7);
}

/// Reconnaissance pulls an attacker out of combat and untaps it.
#[test]
fn reconnaissance_pulls_an_attacker_back() {
    let mut g = two_player_game();
    let recon = ready(&mut g, 0, catalog::reconnaissance());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    activate(&mut g, recon, 0, Some(Target::Permanent(bear))).expect("recall");
    drain_stack(&mut g);
    assert!(!g.attacking.iter().any(|a| a.attacker == bear));
    assert!(!g.battlefield_find(bear).unwrap().tapped);
    advance_to(&mut g, TurnStep::End);
    assert_eq!(g.players[1].life, 20, "no damage got through");
}

/// Plaguebearer's {X}{X}{B} only hits a nonblack creature of exactly mana value X.
#[test]
fn plaguebearer_kills_exactly_mana_value_x() {
    let mut g = two_player_game();
    let bearer = ready(&mut g, 0, catalog::plaguebearer());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(6);
    let go = |g: &mut GameState, x: u32| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: bearer,
            ability_index: 0,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: Some(x),
        })
    };
    assert!(go(&mut g, 1).is_err(), "the bear is mana value 2");
    go(&mut g, 2).expect("kill it");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
}

/// Wall of Nets exiles what it blocked once combat ends.
#[test]
fn wall_of_nets_exiles_what_it_blocked() {
    let mut g = two_player_game();
    let wall = ready(&mut g, 0, catalog::wall_of_nets());
    let attacker = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, attacker)])).expect("block");
    advance_to(&mut g, TurnStep::End);
    assert!(g.battlefield_find(attacker).is_none(), "exiled at end of combat");
}

/// Fade Away taxes each player {1} per creature on the board.
#[test]
fn fade_away_taxes_a_permanent_per_creature() {
    let mut g = two_player_game();
    ready(&mut g, 1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::forest());
    g.add_card_to_battlefield(1, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::fade_away());
    g.players[0].mana_pool.add(Color::Blue, 3);
    cast(&mut g, spell, None).expect("fade away");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 1).count(),
        2,
        "no mana up, so one permanent goes"
    );
}

/// Dominating Licid steals its host while it's attached and hands it back on
/// detach.
#[test]
fn dominating_licid_holds_its_host_while_attached() {
    let mut g = two_player_game();
    let licid = ready(&mut g, 0, catalog::dominating_licid());
    let victim = ready(&mut g, 1, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Blue, 3);
    activate(&mut g, licid, 0, Some(Target::Permanent(victim))).expect("attach");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(victim).unwrap().controller, 0);
}

/// Penance banks a hand card against the next black or red hit.
#[test]
fn penance_shields_against_a_red_source() {
    let mut g = two_player_game();
    let penance = ready(&mut g, 0, catalog::penance());
    g.add_card_to_hand(0, catalog::forest());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.step = TurnStep::PreCombatMain;
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    activate(&mut g, penance, 0, Some(Target::Permanent(bolt))).expect("bank a card");
    drain_stack(&mut g);
    assert!(g.players[0].hand.is_empty(), "the card went on top of the library");
    assert_eq!(g.players[0].life, 20, "prevented");
}

/// Crashing Boars' conscript is chosen by the defending player, not by the
/// engine's first match.
#[test]
fn crashing_boars_lets_the_defender_pick() {
    let mut g = two_player_game();
    let boars = g.add_card_to_battlefield(0, catalog::crashing_boars());
    g.clear_sickness(boars);
    let first = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let second = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![second])]));
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: boars,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(second).unwrap().must_block, Some(boars));
    assert_eq!(g.battlefield_find(first).unwrap().must_block, None, "the other stays free");
}
