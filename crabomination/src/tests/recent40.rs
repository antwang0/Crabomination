//! Functionality tests for `catalog::sets::decks::recent40` — utility lands and
//! the new `StaticEffect::PreventAllDamageToController` (Glacial Chasm, CR 615).

use crate::card::Keyword;
use crate::catalog;
use crate::game::effects::EntityRef;
use crate::game::two_player_game;
use crate::game::*;

fn activate(g: &mut GameState, id: CardId, idx: usize) {
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: idx, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("ability activates");
    drain_stack(g);
}

#[test]
fn ancient_tomb_pings_its_controller() {
    let mut g = two_player_game();
    let tomb = g.add_card_to_battlefield(0, catalog::ancient_tomb());
    let life = g.players[0].life;
    activate(&mut g, tomb, 0);
    assert_eq!(g.players[0].life, life - 2, "tapping Ancient Tomb deals 2 to you");
    assert_eq!(g.players[0].mana_pool.total(), 2, "added two colorless");
}

#[test]
fn tarnished_citadel_painful_rainbow() {
    let mut g = two_player_game();
    let cit = g.add_card_to_battlefield(0, catalog::tarnished_citadel());
    let life = g.players[0].life;
    // ability 0 = {C} (painless), ability 1 = any color + 3 damage.
    activate(&mut g, cit, 1);
    assert_eq!(g.players[0].life, life - 3, "the any-color mode deals 3 to you");
}

#[test]
fn castle_locthwain_draws_and_drains() {
    let mut g = two_player_game();
    let castle = g.add_card_to_battlefield(0, catalog::castle_locthwain());
    for _ in 0..3 { g.add_card_to_library(0, catalog::swamp()); }
    g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    activate(&mut g, castle, 1); // draw-then-lose-life
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    // After drawing, hand has `hand + 1` cards → lose that much life.
    assert_eq!(g.players[0].life, life - (hand as i32 + 1), "lose life = cards in hand");
}

#[test]
fn castle_ardenvale_makes_a_human() {
    let mut g = two_player_game();
    let castle = g.add_card_to_battlefield(0, catalog::castle_ardenvale());
    g.players[0].mana_pool.add(crate::mana::Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, castle, 1);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Human"),
        "minted a 1/1 Human token"
    );
}

#[test]
fn faceless_haven_animates_to_a_changeling() {
    let mut g = two_player_game();
    let haven = g.add_card_to_battlefield(0, catalog::faceless_haven());
    g.players[0].mana_pool.add_snow(crate::mana::Color::White, 3);
    activate(&mut g, haven, 1);
    let cp = g.computed_permanent(haven).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3), "becomes a 4/3");
    assert!(cp.keywords.contains(&Keyword::Vigilance), "with vigilance");
    assert!(cp.keywords.contains(&Keyword::Changeling), "and all creature types");
    assert!(cp.card_types.contains(&crate::card::CardType::Land), "still a land");
}

#[test]
fn crawling_barrens_grows_and_animates() {
    let mut g = two_player_game();
    let barrens = g.add_card_to_battlefield(0, catalog::crawling_barrens());
    g.players[0].mana_pool.add_colorless(4);
    activate(&mut g, barrens, 1);
    let cp = g.computed_permanent(barrens).unwrap();
    // 0/0 base + two +1/+1 counters = 2/2.
    assert_eq!((cp.power, cp.toughness), (2, 2), "two +1/+1 counters on the 0/0 land");
}

#[test]
fn field_of_the_dead_makes_a_zombie_at_seven_names() {
    let mut g = two_player_game();
    // Six distinct lands + Field of the Dead = seven differently-named lands.
    g.add_card_to_battlefield(0, catalog::plains());
    g.add_card_to_battlefield(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::swamp());
    g.add_card_to_battlefield(0, catalog::mountain());
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::ancient_tomb());
    g.add_card_to_battlefield(0, catalog::field_of_the_dead());
    // An eighth distinct land enters → trigger sees ≥7 names.
    let newcomer = g.add_card_to_battlefield(0, catalog::city_of_traitors());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: newcomer }]);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Zombie"),
        "made a 2/2 Zombie"
    );
}

#[test]
fn field_of_the_dead_silent_below_seven_names() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::plains());
    g.add_card_to_battlefield(0, catalog::island());
    let field = g.add_card_to_battlefield(0, catalog::field_of_the_dead());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: field }]);
    drain_stack(&mut g);
    assert!(
        !g.battlefield.iter().any(|c| c.definition.name == "Zombie"),
        "only three names → no Zombie"
    );
}

#[test]
fn glacial_chasm_prevents_all_damage_to_you() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::glacial_chasm());
    let life = g.players[0].life;
    let mut events = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(0), 5, None, &mut events);
    assert_eq!(g.players[0].life, life, "all damage to the controller is prevented");
    // The opponent (no Chasm) still takes damage.
    let foe_life = g.players[1].life;
    g.deal_damage_to_from(EntityRef::Player(1), 5, None, &mut events);
    assert_eq!(g.players[1].life, foe_life - 5, "the other player is unaffected");
}

#[test]
fn glacial_chasm_grounds_your_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::glacial_chasm());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(
        g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::CantAttack),
        "your creatures can't attack while the Chasm is out"
    );
}
