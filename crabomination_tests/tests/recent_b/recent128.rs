//! Functionality tests for `catalog::sets::decks::recent128` (WOE).

use crabomination::card::{CounterType, EnchantmentSubtype, Keyword};
use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// Armory Mice is 3/1 normally, 3/3 once Celebration is active.
#[test]
fn armory_mice_celebration_toughness() {
    let mut g = two_player_game();
    let mice = g.add_card_to_battlefield(0, catalog::armory_mice());
    assert_eq!(g.computed_permanent(mice).unwrap().toughness, 1, "no celebration → 3/1");
    // Two nonland permanents entering this turn switches Celebration on.
    g.players[0].nonland_permanents_entered_this_turn = 2;
    assert_eq!(g.computed_permanent(mice).unwrap().toughness, 3, "celebration → +0/+2");
}

/// Belligerent of the Ball pumps a creature at combat only under Celebration.
#[test]
fn belligerent_celebration_combat_trigger() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let bell = g.add_card_to_battlefield(0, catalog::belligerent_of_the_ball());
    g.players[0].nonland_permanents_entered_this_turn = 2;
    while g.step != TurnStep::BeginCombat {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    let cp = g.computed_permanent(bell).unwrap();
    assert_eq!(cp.power, 4, "+1/+0 under celebration");
    assert!(cp.keywords.contains(&Keyword::Menace), "and menace");
}

/// Archive Dragon scrys 2 on entry.
#[test]
fn archive_dragon_scry() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let dragon = g.add_card_to_battlefield(0, catalog::archive_dragon());
    g.fire_self_etb_triggers(dragon, 0);
    drain_stack(&mut g);
    let cp = g.computed_permanent(dragon).unwrap();
    assert!(cp.keywords.contains(&Keyword::Flying) && cp.keywords.iter().any(|k| matches!(k, Keyword::Ward(_))));
}

/// Agatha's Champion fights only when bargained.
#[test]
fn agathas_champion_bargained_fight() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let fodder = g.add_card_to_battlefield(0, catalog::ornithopter()); // artifact to bargain
    let champ = g.add_card_to_hand(0, catalog::agathas_champion());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpellBargain {
        card_id: champ,
        sacrifice: Some(fodder),
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Agatha's Champion bargained");
    drain_stack(&mut g);
    // 4/4 fights the 2/2 → the bear dies.
    assert!(g.battlefield_find(victim).is_none(), "bargained fight kills the bear");
}

/// Cut In burns a creature and makes a Young Hero Role on your own.
#[test]
fn cut_in_damage_and_role() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::cut_in());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(enemy)),
        additional_targets: vec![Target::Permanent(mine)],
        mode: None,
        x_value: None,
    })
    .expect("cast Cut In");
    drain_stack(&mut g);
    assert!(g.battlefield_find(enemy).is_none(), "4 damage kills the 2/2");
    assert!(
        g.battlefield.iter().any(|c| c.attached_to == Some(mine)
            && c.definition.subtypes.enchantment_subtypes.contains(&EnchantmentSubtype::Role)),
        "Young Hero Role attached to my creature",
    );
}

/// Become Brutes gives haste and a Monster Role (+1/+1 trample).
#[test]
fn become_brutes_monster_role() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::become_brutes());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Become Brutes");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "Monster Role gives +1/+1");
    assert!(cp.keywords.contains(&Keyword::Trample) && cp.keywords.contains(&Keyword::Haste));
}

/// Diminisher Witch bargained shrinks an opponent's creature to 1/1.
#[test]
fn diminisher_witch_cursed_role() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let big = g.add_card_to_battlefield(1, catalog::gigantosaurus()); // 10/10
    let fodder = g.add_card_to_battlefield(0, catalog::ornithopter());
    let witch = g.add_card_to_hand(0, catalog::diminisher_witch());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpellBargain {
        card_id: witch,
        sacrifice: Some(fodder),
        target: Some(Target::Permanent(big)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Diminisher Witch bargained");
    drain_stack(&mut g);
    let cp = g.computed_permanent(big).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "Cursed Role sets it to 1/1");
}

/// Charging Hooligan grows by the number of attackers.
#[test]
fn charging_hooligan_scales_with_attackers() {
    let mut g = two_player_game();
    let hooligan = g.add_card_to_battlefield(0, catalog::charging_hooligan());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(hooligan);
    g.clear_sickness(ally);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: hooligan, target: AttackTarget::Player(1) },
        Attack { attacker: ally, target: AttackTarget::Player(1) },
    ]))
    .expect("attack with two");
    drain_stack(&mut g);
    // Base 3 + 2 attackers = 5.
    assert_eq!(g.computed_permanent(hooligan).unwrap().power, 5, "+1/+0 per attacker");
}

/// Barrow Naughty has lifelink only while you control another Faerie.
#[test]
fn barrow_naughty_conditional_lifelink() {
    let mut g = two_player_game();
    let naughty = g.add_card_to_battlefield(0, catalog::barrow_naughty());
    assert!(
        !g.computed_permanent(naughty).unwrap().keywords.contains(&Keyword::Lifelink),
        "no other Faerie → no lifelink",
    );
    g.add_card_to_battlefield(0, catalog::barrow_naughty()); // another Faerie
    assert!(
        g.computed_permanent(naughty).unwrap().keywords.contains(&Keyword::Lifelink),
        "another Faerie → lifelink",
    );
}

/// Ego Drain strips a nonland card from an opponent's hand.
#[test]
fn ego_drain_discards_nonland() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let land = g.add_card_to_hand(1, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::ego_drain());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Ego Drain");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"), "discarded");
    assert!(g.players[1].hand.iter().any(|c| c.id == land), "the land stays");
}

/// The Young Hero Role's granted trigger counters up its host on attack.
#[test]
fn young_hero_role_counter_on_attack() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::cut_in());
    // Give a throwaway enemy so Cut In's damage target is legal.
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(enemy)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: None,
    })
    .expect("cast Cut In for the Role");
    drain_stack(&mut g);
    g.clear_sickness(bear);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        1,
        "Young Hero Role puts a +1/+1 counter on attack",
    );
}
