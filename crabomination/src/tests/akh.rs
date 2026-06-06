//! Functionality tests for Amonkhet Embalm (CR 702.88) / Eternalize
//! (CR 702.91) creatures (`catalog::sets::akh`).

use crate::catalog;
use crate::game::*;
use crate::mana::Color;
use crate::TurnStep;
use crate::game::{drain_stack, two_player_game};

/// Embalm exiles the card from the graveyard and mints a Zombie token copy
/// with the original's P/T.
#[test]
fn embalm_sacred_cat_mints_zombie_token_copy() {
    use crate::card::CreatureType;
    let mut g = two_player_game();
    let cat = g.add_card_to_graveyard(0, catalog::sacred_cat());
    g.players[0].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cat, ability_index: 0, target: None, x_value: None })
        .expect("Embalm {W} from graveyard");
    drain_stack(&mut g);
    // Original card is exiled (gone from graveyard).
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == cat), "card exiled by Embalm");
    assert!(g.exile.iter().any(|c| c.id == cat));
    // A 1/1 Zombie Cat token copy is on the battlefield.
    let tok = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Sacred Cat")
        .expect("token copy minted");
    assert_eq!((tok.power(), tok.toughness()), (1, 1));
    assert!(tok.definition.subtypes.creature_types.contains(&CreatureType::Zombie),
        "token gains Zombie type");
    assert!(tok.definition.subtypes.creature_types.contains(&CreatureType::Cat),
        "token keeps original Cat type");
}

/// Eternalize mints a 4/4 token copy.
#[test]
fn eternalize_adorned_pouncer_mints_four_four() {
    let mut g = two_player_game();
    let p = g.add_card_to_graveyard(0, catalog::adorned_pouncer());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: p, ability_index: 0, target: None, x_value: None })
        .expect("Eternalize {3}{W}{W}");
    drain_stack(&mut g);
    let tok = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Adorned Pouncer")
        .expect("token copy minted");
    assert_eq!((tok.power(), tok.toughness()), (4, 4), "Eternalize token is 4/4");
    assert!(tok.definition.keywords.contains(&crate::card::Keyword::DoubleStrike),
        "token keeps Double Strike");
}

/// Embalm is sorcery-speed only: rejected on the opponent's turn.
#[test]
fn embalm_rejected_at_instant_speed() {
    let mut g = two_player_game();
    let cat = g.add_card_to_graveyard(0, catalog::sacred_cat());
    g.players[0].mana_pool.add(Color::White, 1);
    // Opponent's turn / stack-nonempty equivalents: hand priority to p0 during
    // an opponent main isn't sorcery speed for p0.
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: cat, ability_index: 0, target: None, x_value: None });
    assert!(res.is_err(), "Embalm only as a sorcery — rejected on opponent's turn");
}

/// Anointer Priest gains 1 life when a creature token you control enters.
#[test]
fn anointer_priest_gains_life_on_token_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::anointer_priest());
    let life = g.players[0].life;
    // Mint a creature token under p0's control.
    let servo = crate::card::TokenDefinition {
        name: "Servo".into(), power: 1, toughness: 1,
        card_types: vec![crate::card::CardType::Creature],
        ..Default::default()
    };
    let tok = g.add_token_to_battlefield(0, &servo);
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: tok }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "Anointer Priest gains 1 life per token");
}

/// Angel of Sanctions exiles an opponent's permanent on ETB and returns it
/// when the Angel leaves.
#[test]
fn angel_of_sanctions_exiles_until_it_leaves() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let angel = g.add_card_to_hand(0, catalog::angel_of_sanctions());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: angel, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None })
        .expect("cast Angel of Sanctions");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear exiled by Angel ETB");
    // Angel leaves → bear returns.
    g.remove_from_battlefield_to_graveyard(angel);
    g.check_state_based_actions();
    assert!(g.battlefield.iter().any(|c| c.id == bear), "bear returns when Angel leaves");
}

/// Timeless Witness returns a card from your graveyard to hand on ETB.
#[test]
fn timeless_witness_returns_card_from_graveyard() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let wit = g.add_card_to_battlefield(0, catalog::timeless_witness());
    g.fire_self_etb_triggers(wit, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "Bolt returned to hand");
}

/// Sunscourge Champion gains life equal to its power on ETB.
#[test]
fn sunscourge_champion_gains_life_equal_to_power() {
    let mut g = two_player_game();
    let life = g.players[0].life;
    let champ = g.add_card_to_battlefield(0, catalog::sunscourge_champion());
    g.fire_self_etb_triggers(champ, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "gains 2 life (its power)");
}

/// Sinuous Striker's {U}: +1/-1 pump resolves.
#[test]
fn sinuous_striker_pumps_plus_one_minus_one() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::sinuous_striker());
    g.clear_sickness(s);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: s, ability_index: 0, target: None, x_value: None })
        .expect("activate {U} pump");
    drain_stack(&mut g);
    let cp = g.computed_permanent(s).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 1), "2/2 +1/-1 = 3/1");
}

/// Earthshaker Khenra's ETB makes a low-power creature unable to block.
#[test]
fn earthshaker_khenra_stops_a_blocker() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2, power 2
    let khenra = g.add_card_to_battlefield(0, catalog::earthshaker_khenra());
    g.fire_self_etb_triggers(khenra, 0);
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::CantBlock), "bear can't block this turn");
}

/// Every vanilla Embalm/Eternalize body has correct stats and a graveyard
/// token-copy activated ability.
#[test]
fn akh_embalm_bodies_have_graveyard_ability() {
    type Case = (fn() -> crate::card::CardDefinition, i32, i32);
    let cases: &[Case] = &[
        (catalog::unwavering_initiate, 3, 2),
        (catalog::steadfast_sentinel, 2, 3),
        (catalog::aven_initiate, 3, 2),
        (catalog::proven_combatant, 1, 1),
        (catalog::tah_crop_skirmisher, 2, 1),
        (catalog::honored_hydra, 6, 6),
        (catalog::dreamstealer, 1, 2),
        (catalog::oketras_attendant, 3, 3),
    ];
    for (f, p, t) in cases {
        let d = f();
        assert_eq!((d.power, d.toughness), (*p, *t), "{} stats", d.name);
        assert!(d.activated_abilities.iter().any(|a| a.from_graveyard && a.exile_self_cost),
            "{} has an Embalm/Eternalize graveyard ability", d.name);
    }
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Tah-Crop Elite's exert attack pumps your team +1/+1 and it won't untap.
#[test]
fn tah_crop_elite_exert_pumps_team() {
    let mut g = two_player_game();
    let elite = g.add_card_to_battlefield(0, catalog::tah_crop_elite());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(elite);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: elite, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let b = g.computed_permanent(bear).unwrap();
    assert_eq!((b.power, b.toughness), (3, 3), "team +1/+1 from exert");
    // Exerted: won't untap next untap step.
    assert!(g.battlefield_find(elite).unwrap().skip_next_untap, "exerted creature skips untap");
}

/// Glory-Bound Initiate's exert attack gives it +1/+3 and lifelink.
#[test]
fn glory_bound_initiate_exert_buffs_self() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let gbi = g.add_card_to_battlefield(0, catalog::glory_bound_initiate());
    g.clear_sickness(gbi);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: gbi, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let cp = g.computed_permanent(gbi).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "3/1 +1/+3 = 4/4");
    assert!(cp.keywords.contains(&Keyword::Lifelink), "gains lifelink");
}

/// Bloodrage Brawler discards a card on ETB.
#[test]
fn bloodrage_brawler_discards_on_etb() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::island());
    let hand = g.players[0].hand.len();
    let bb = g.add_card_to_battlefield(0, catalog::bloodrage_brawler());
    g.fire_self_etb_triggers(bb, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1, "discards a card on ETB");
}

/// Open Fire deals 3 to any target.
#[test]
fn open_fire_deals_three() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let of = g.add_card_to_hand(0, catalog::open_fire());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: of, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None })
        .expect("cast Open Fire");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "2/2 bear dies to 3 damage");
}

/// Gideon's Reproach only hits an attacking or blocking creature.
#[test]
fn gideons_reproach_requires_attacker_or_blocker() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let gr = g.add_card_to_hand(0, catalog::gideons_reproach());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Idle creature is not a legal target.
    let res = g.perform_action(GameAction::CastSpell {
        card_id: gr, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None });
    assert!(res.is_err(), "idle creature isn't attacking/blocking");
}

/// Cast Out flashes in and exiles a permanent until it leaves.
#[test]
fn cast_out_exiles_until_it_leaves() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let co = g.add_card_to_hand(0, catalog::cast_out());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: co, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None })
        .expect("cast Cast Out");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear exiled by Cast Out");
}

/// Greater Sandwurm can't be blocked by a power-2 creature; Naga Vitalist taps
/// for a color your lands make; Hooded Brawler exerts for +2/+2.
#[test]
fn akh_green_batch_functionality() {
    use crate::card::Keyword;
    // Greater Sandwurm evasion keyword present.
    let sw = catalog::greater_sandwurm();
    assert!(sw.keywords.iter().any(|k| matches!(k, Keyword::CantBeBlockedBy(_))));
    assert!(sw.keywords.iter().any(|k| matches!(k, Keyword::Cycling(_))));

    // Naga Vitalist taps for a color one of your basic lands could make.
    let mut g = two_player_game();
    let naga = g.add_card_to_battlefield(0, catalog::naga_vitalist());
    g.clear_sickness(naga);
    g.add_card_to_battlefield(0, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: naga, ability_index: 0, target: None, x_value: None })
        .expect("Naga taps for mana");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);

    // Hooded Brawler exerts for +2/+2 on attack.
    let hb = g.add_card_to_battlefield(0, catalog::hooded_brawler());
    g.clear_sickness(hb);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: hb, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let cp = g.computed_permanent(hb).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 4), "3/2 +2/+2 = 5/4 exerted");
}
