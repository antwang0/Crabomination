//! Functionality tests for `catalog::sets::decks::recent283`.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::{two_player_game, Target};

/// Aven Heartstabber gains +2/+2 and deathtouch once 5 distinct mana values
/// sit in its controller's graveyard.
#[test]
fn aven_heartstabber_graveyard_scaler() {
    let mut g = two_player_game();
    let aven = g.add_card_to_battlefield(0, catalog::aven_heartstabber());
    assert_eq!(g.computed_permanent(aven).unwrap().power, 1, "1/1 with an empty graveyard");
    // Five cards of distinct mana values (0,1,2,3,4).
    g.add_card_to_graveyard(0, catalog::forest()); // MV 0
    g.add_card_to_graveyard(0, catalog::kindled_heroism()); // {R} → MV 1
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
    g.add_card_to_graveyard(0, catalog::repel_calamity()); // {1}{W} → MV 2 (dup)
    assert_eq!(g.computed_permanent(aven).unwrap().power, 1, "still four distinct values");
    g.add_card_to_graveyard(0, catalog::horses_of_the_bruinen()); // {3}{U}{U} → MV 5
    g.add_card_to_graveyard(0, catalog::eagle_of_deliverance()); // {4}{W}{W} → MV 6
    assert_eq!(g.computed_permanent(aven).unwrap().power, 3, "+2/+2 at 5+ distinct values");
    assert!(
        g.computed_permanent(aven).unwrap().keywords.contains(&crabomination::card::Keyword::Deathtouch),
        "gains deathtouch",
    );
}

/// Aven Heartstabber's death trigger mills two and draws one.
#[test]
fn aven_heartstabber_death_mills_draws() {
    let mut g = two_player_game();
    let aven = g.add_card_to_battlefield(0, catalog::aven_heartstabber());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let hand = g.players[0].hand.len();
    let gy = g.players[0].graveyard.len();
    let effect = catalog::aven_heartstabber().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_ability(aven, 0, None)).unwrap();
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew one");
    assert_eq!(g.players[0].graveyard.len(), gy + 2, "milled two");
}

/// Ambitious Dragonborn enters with counters equal to the greatest power
/// among creatures you control and creature cards in your graveyard.
#[test]
fn ambitious_dragonborn_counts_graveyard_power() {
    use crabomination::game::{drain_stack, GameAction, TurnStep};
    use crabomination::mana::Color;
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_graveyard(0, catalog::eagle_of_deliverance()); // 5/5 creature card in gy
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 on board (less than 5)
    let spell = g.add_card_to_hand(0, catalog::ambitious_dragonborn());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Ambitious Dragonborn");
    drain_stack(&mut g);
    let db = g.battlefield.iter().find(|c| c.definition.name == "Ambitious Dragonborn").unwrap();
    assert_eq!(
        db.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        5,
        "X = 5 from the graveyard Eagle (greater than the 2/2 on board)",
    );
}

/// Jolly Gerbils draws whenever its controller gives a gift.
#[test]
fn jolly_gerbils_draws_on_gift() {
    use crabomination::game::GameAction;
    use crabomination::mana::Color;
    use crabomination::game::drain_stack;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::jolly_gerbils());
    let own = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest()); // something to draw
    let crumb = g.add_card_to_hand(0, catalog::crumb_and_get_it()); // {W}, Gift a Food
    g.players[0].mana_pool.add(Color::White, 1);
    g.step = crabomination::game::TurnStep::PreCombatMain;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastGift {
        card_id: crumb,
        target: Some(Target::Permanent(own)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Crumb and Get It with gift");
    drain_stack(&mut g);
    // -1 for the spell leaving hand, +1 for Jolly Gerbils' draw = net hand.
    assert_eq!(g.players[0].hand.len(), hand, "Jolly Gerbils drew off the gift");
    assert!(
        g.battlefield.iter().any(|c| c.controller == 1 && c.definition.name == "Food"),
        "opponent received the promised Food",
    );
}

/// Argivian Cavalier mints a Soldier on entry, and its Enlist taps a helper
/// to add its power in combat.
#[test]
fn argivian_cavalier_etb_token_and_enlist() {
    use crabomination::game::{drain_stack, Attack, AttackTarget, GameAction, TurnStep};
    let mut g = two_player_game();
    let cav = g.add_card_to_battlefield(0, catalog::argivian_cavalier());
    // ETB token.
    let etb = catalog::argivian_cavalier().triggered_abilities[1].effect.clone();
    g.resolve_effect(&etb, &EffectContext::for_ability(cav, 0, None)).unwrap();
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Soldier"),
        "minted a 1/1 Soldier",
    );
    // Enlist through the real combat flow: tap the Soldier helper.
    let helper = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(cav);
    g.clear_sickness(helper);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: cav,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(cav).unwrap().power(), 4, "2 base + 2 from the helper");
}
