//! Functionality tests for the Mystical Archive (STA) batch in
//! `catalog::sets::stx::sta`: Infuriate, Blue Sun's Zenith, Abundant Harvest,
//! Urza's Rage, Natural Order.

use crabomination::catalog;
use crabomination::game::two_player_game;
use crabomination::mana::Color;
use super::*;

/// Infuriate pumps the target +3/+2 until end of turn.
#[test]
fn infuriate_pumps_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::infuriate());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Infuriate");
    drain_stack(&mut g);

    let c = g.battlefield_find(bear).unwrap();
    assert_eq!((c.power(), c.toughness()), (5, 4), "+3/+2 EOT");
}

/// Blue Sun's Zenith draws X for the target player and shuffles itself back
/// into the library (not the graveyard).
#[test]
fn blue_suns_zenith_draws_x_and_shuffles_back() {
    let mut g = two_player_game();
    for _ in 0..5 {
        let id = g.next_id();
        g.players[0].add_to_library_top(id, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::blue_suns_zenith());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("cast Blue Sun's Zenith");
    drain_stack(&mut g);

    // Cast removes it from hand (-1), then draws 2 (+2) → net +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew 2, cast left hand");
    assert!(g.players[0].graveyard.iter().all(|c| c.id != spell), "not in graveyard");
    assert!(g.players[0].library.iter().any(|c| c.id == spell), "shuffled into library");
}

/// Abundant Harvest (land mode auto-picked) digs to the first land and puts it
/// into hand; the nonland miss is bottomed.
#[test]
fn abundant_harvest_finds_a_land() {
    let mut g = two_player_game();
    // Top → bottom: bear (nonland miss), Forest (the find).
    let forest = g.next_id();
    g.players[0].add_to_library_top(forest, catalog::forest());
    let bear = g.next_id();
    g.players[0].add_to_library_top(bear, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::abundant_harvest());
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Abundant Harvest");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == forest), "land put into hand");
    assert!(g.players[0].hand.iter().all(|c| c.id != bear), "nonland miss not in hand");
}

/// Urza's Rage deals 3 unkicked, 10 kicked, and can't be countered.
#[test]
fn urzas_rage_kicked_deals_ten() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::urzas_rage());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(10); // {2}{R} + kicker {8}{R} = {10}{R}{R}
    let life = g.players[1].life;

    g.perform_action(GameAction::CastSpellKicked {
        card_id: spell,
        target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast kicked Urza's Rage");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life - 10, "kicked → 10 damage");
}

/// Natural Order sacrifices a green creature and fetches a green creature onto
/// the battlefield.
#[test]
fn natural_order_sacrifices_and_fetches() {
    let mut g = two_player_game();
    let sac = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green creature to sac
    let big = g.next_id();
    g.players[0].add_to_library_top(big, catalog::craw_wurm()); // green creature in library
    let spell = g.add_card_to_hand(0, catalog::natural_order());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    // The library search is declined by the AutoDecider; script the pick.
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(Some(big)),
    ]));

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Natural Order");
    drain_stack(&mut g);

    assert!(g.battlefield_find(sac).is_none(), "green creature sacrificed as cost");
    assert!(g.battlefield_find(big).is_some(), "fetched creature on battlefield");
}

/// Tainted Pact (auto-decider) exiles the top card and takes it into hand.
#[test]
fn tainted_pact_takes_top_card() {
    let mut g = two_player_game();
    let bear = g.next_id();
    g.players[0].add_to_library_top(bear, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::tainted_pact());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Tainted Pact");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "top card put into hand");
    assert!(g.exile.iter().all(|c| c.id != bear), "taken card is not left in exile");
}

/// Declining each uniquely-named card digs until a duplicate name is exiled;
/// that card (and the misses) stay in exile and nothing reaches hand.
#[test]
fn tainted_pact_digs_until_duplicate_name() {
    let mut g = two_player_game();
    // Library top → bottom: Forest, Grizzly Bears, Grizzly Bears.
    let g1 = g.next_id();
    g.players[0].add_to_library_top(g1, catalog::grizzly_bears());
    let g2 = g.next_id();
    g.players[0].add_to_library_top(g2, catalog::grizzly_bears());
    let forest = g.next_id();
    g.players[0].add_to_library_top(forest, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::tainted_pact());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Decline the two uniquely-named cards (Forest, first Grizzly).
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));

    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Tainted Pact");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().all(|c| c.id != forest && c.id != g1 && c.id != g2),
        "nothing reached hand");
    assert!(g.players[0].library.is_empty(), "library emptied while digging");
    for id in [forest, g1, g2] {
        assert!(g.exile.iter().any(|c| c.id == id), "card {id:?} ended in exile");
    }
}

/// Mizzix's Mastery exiles the targeted instant from the graveyard and
/// free-casts it (the copy auto-targets the opponent).
#[test]
fn mizzixs_mastery_recasts_graveyard_instant() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let spell = g.add_card_to_hand(0, catalog::mizzixs_mastery());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    // Confirm the "cast without paying?" prompt.
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    let life = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Mizzix's Mastery");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life - 3, "recast Bolt dealt 3");
}

/// Overloaded Mizzix's Mastery free-casts every instant/sorcery in the
/// graveyard (Bolt for 3 + Shock for 2 = 5 to the opponent).
#[test]
fn mizzixs_mastery_overload_recasts_each() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::shock());
    let spell = g.add_card_to_hand(0, catalog::mizzixs_mastery());
    g.players[0].mana_pool.add(Color::Red, 3);
    g.players[0].mana_pool.add_colorless(5);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    let life = g.players[1].life;

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: spell, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("overload Mizzix's Mastery");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life - 5, "both spells recast for 3 + 2");
}

/// Lorehold Tomb Robber: copy a graveyard creature card (hasty token),
/// exile the original, and exile the token at the next end step.
#[test]
fn lorehold_tomb_robber_copies_gy_creature_with_haste_then_exiles_both() {
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // 2/2 in gy
    let id = g.add_card_to_hand(0, catalog::lorehold_tomb_robber());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(dead)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Tomb Robber castable");
    drain_stack(&mut g);

    // A copy token of the graveyard creature is on P0's battlefield, with haste.
    let token = g
        .battlefield
        .iter()
        .find(|c| c.is_token && c.controller == 0 && c.definition.name == "Grizzly Bears")
        .expect("a copy token of the graveyard creature");
    let token_id = token.id;
    assert_eq!(token.power(), 2);
    assert_eq!(token.toughness(), 2);
    assert!(
        g.permanent_has_keyword(token_id, &crabomination::card::Keyword::Haste),
        "the copy has haste",
    );
    // The original card is exiled out of the graveyard.
    assert!(
        !g.players[0].graveyard.iter().any(|c| c.id == dead),
        "the original creature card left the graveyard",
    );
    assert!(g.exile.iter().any(|c| c.id == dead), "the original is exiled");

    // At the next end step, the token is exiled — and, being a token, it
    // then ceases to exist (CR 111.7), so it's gone from every zone.
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(
        !g.battlefield.iter().any(|c| c.id == token_id),
        "the token is exiled at the next end step",
    );
    assert!(
        !g.exile.iter().any(|c| c.id == token_id),
        "the exiled token ceases to exist (not lingering in exile)",
    );
}

/// Library Larcenist mills the *defending* player two on combat damage
/// (regression: the mill targets the damaged player, not the controller).
#[test]
fn library_larcenist_mills_defending_player() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let lar = g.add_card_to_battlefield(0, catalog::library_larcenist());
    g.clear_sickness(lar);
    for _ in 0..4 { g.add_card_to_library(1, catalog::grizzly_bears()); }
    let before = g.players[1].graveyard.len();
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: lar, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let mut guard = 0;
    while g.step != TurnStep::PostCombatMain && guard < 40 {
        g.perform_action(GameAction::PassPriority).expect("pass");
        guard += 1;
    }
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), before + 2, "defending player milled two");
}
