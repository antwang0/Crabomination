//! Cards whose printed rider shipped dropped (pod decks first, then the
//! `audit_activated_costs.py` finds): each test asserts the rider, not the
//! card's already-covered body.

use crabomination::card::{CardId, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::game::*;
use crabomination::mana::Color;
use crabomination::TurnStep;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<Vec<GameEvent>, GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
}

/// Puresteel Paladin — metalcraft: Equipment you control have equip {0}.
/// Colossus Hammer's {8} is free with three artifacts, and not with two.
#[test]
fn puresteel_paladin_metalcraft_equips_for_zero() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::puresteel_paladin());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hammer = g.add_card_to_battlefield(0, catalog::colossus_hammer());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    assert!(g.perform_action(GameAction::Equip { equipment: hammer, target: bear }).is_err(), "two artifacts");
    g.add_card_to_battlefield(0, catalog::sol_ring());
    g.perform_action(GameAction::Equip { equipment: hammer, target: bear }).expect("equip {0}");
    assert_eq!(g.battlefield_find(hammer).unwrap().attached_to, Some(bear));
}

/// Colossus Hammer — "Equipped creature gets +10/+10 and loses flying."
#[test]
fn colossus_hammer_takes_flying_away() {
    let mut g = main_phase();
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    let hammer = g.add_card_to_battlefield(0, catalog::colossus_hammer());
    g.players[0].mana_pool.add_colorless(8);
    g.perform_action(GameAction::Equip { equipment: hammer, target: angel }).expect("equip");
    let cp = g.computed_permanent(angel).unwrap();
    assert_eq!((cp.power, cp.toughness), (14, 14));
    assert!(!cp.keywords().contains(&Keyword::Flying));
}

/// Terror of the Peaks — an opponent's spell targeting it costs an additional
/// 3 life, an additional cost CR 119.4 won't let 2 life pay.
#[test]
fn terror_of_the_peaks_taxes_targeting_spells_three_life() {
    let mut g = main_phase();
    let terror = g.add_card_to_battlefield(0, catalog::terror_of_the_peaks());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].life = 2;
    g.players[1].mana_pool.add(Color::Red, 1);
    assert!(cast(&mut g, bolt, Some(Target::Permanent(terror))).is_err(), "2 life can't pay 3");
    g.players[1].life = 20;
    cast(&mut g, bolt, Some(Target::Permanent(terror))).expect("bolt");
    assert_eq!(g.players[1].life, 17);
    let face = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    cast(&mut g, face, Some(Target::Player(0))).expect("bolt the player");
    assert_eq!(g.players[1].life, 17, "only spells that target it");
}

/// Hagra Mauling — {1} less if an opponent controls no basic lands.
#[test]
fn hagra_mauling_is_cheaper_against_no_basics() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hm = g.add_card_to_hand(0, catalog::hagra_mauling());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, hm, Some(Target::Permanent(bear))).expect("{1}{B}{B} with no basics opposite");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    let hm2 = g.add_card_to_hand(0, catalog::hagra_mauling());
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::forest());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    assert!(cast(&mut g, hm2, Some(Target::Permanent(bear2))).is_err(), "full price with a Forest there");
}

/// Plaguecrafter — "Each player who can't [sacrifice] discards a card."
#[test]
fn plaguecrafter_makes_the_empty_board_discard() {
    let mut g = main_phase();
    g.add_card_to_hand(1, catalog::island());
    let pc = g.add_card_to_hand(0, catalog::plaguecrafter());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, pc, None).expect("plaguecrafter");
    drain_stack(&mut g);
    assert!(g.players[1].hand.is_empty(), "no creature or planeswalker: discard");
    assert!(g.battlefield_find(pc).is_none(), "its controller sacrificed it");
}

/// Angelic Destiny — the enchanted creature "is an Angel in addition to its
/// other types".
#[test]
fn angelic_destiny_makes_an_angel() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ad = g.add_card_to_hand(0, catalog::angelic_destiny());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, ad, Some(Target::Permanent(bear))).expect("destiny");
    drain_stack(&mut g);
    let types = g.computed_permanent(bear).unwrap().subtypes().creature_types.clone();
    assert!(types.contains(&CreatureType::Angel) && types.contains(&CreatureType::Bear));
}

/// Legion Warboss — the Goblin "gains haste until end of turn and attacks
/// this combat if able".
#[test]
fn legion_warboss_goblin_must_attack() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::legion_warboss());
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let gob = g.battlefield.iter().find(|c| c.definition.name == "Goblin").unwrap().id;
    let kw = g.computed_permanent(gob).unwrap().keywords().to_vec();
    assert!(kw.contains(&Keyword::Haste) && kw.contains(&Keyword::MustAttack));
}

/// Shalai, Voice of Plenty — "You, planeswalkers you control, and other
/// creatures you control have hexproof."
#[test]
fn shalai_gives_your_planeswalkers_hexproof() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::shalai_voice_of_plenty());
    let pw = g.add_card_to_battlefield(0, catalog::chandra_torch_of_defiance());
    assert!(g.computed_permanent(pw).unwrap().keywords().contains(&Keyword::Hexproof));
}

/// Howling Mine — "if this artifact is untapped": a tapped Mine draws nothing.
#[test]
fn howling_mine_only_while_untapped() {
    let mut g = main_phase();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let mine = g.add_card_to_battlefield(0, catalog::howling_mine());
    g.battlefield_find_mut(mine).unwrap().tapped = true;
    g.step = TurnStep::Draw;
    g.fire_step_triggers(TurnStep::Draw);
    drain_stack(&mut g);
    assert!(g.players[0].hand.is_empty(), "tapped: no extra card");
    g.battlefield_find_mut(mine).unwrap().tapped = false;
    g.fire_step_triggers(TurnStep::Draw);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Claim Jumper — "Then if an opponent controls more lands than you, repeat
/// this process once": two lands behind, it fetches two Plains.
#[test]
fn claim_jumper_repeats_while_behind() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::plains());
        g.add_card_to_battlefield(1, catalog::island());
    }
    g.add_card_to_battlefield(0, catalog::plains());
    let cj = g.add_card_to_hand(0, catalog::claim_jumper());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, cj, None).expect("claim jumper");
    drain_stack(&mut g);
    let plains = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Plains").count();
    assert_eq!(plains, 3, "one, then once more while still behind");
}

/// Veyran, Voice of Duality — no prowess; its magecraft is one +1/+1 that its
/// own static makes trigger twice (CR 603.2 "triggers an additional time").
#[test]
fn veyran_doubles_its_own_magecraft() {
    let mut g = main_phase();
    let v = g.add_card_to_battlefield(0, catalog::veyran_voice_of_duality());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
    drain_stack(&mut g);
    let cp = g.computed_permanent(v).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
}

/// Magda, Brazen Outlaw — "*Other* Dwarves you control get +1/+0."
#[test]
fn magda_pumps_other_dwarves_only() {
    let mut g = main_phase();
    let magda = g.add_card_to_battlefield(0, catalog::magda_brazen_outlaw());
    assert_eq!(g.computed_permanent(magda).unwrap().power, 2);
}

/// Nahiri, the Harbinger −8 — the fetched creature gains haste and returns to
/// your hand at the beginning of the next end step.
#[test]
fn nahiri_ultimate_hastes_and_returns() {
    let mut g = main_phase();
    let nahiri = g.add_card_to_battlefield(0, catalog::nahiri_the_harbinger());
    g.battlefield_find_mut(nahiri).unwrap().add_counters(crabomination::card::CounterType::Loyalty, 4);
    let giant = g.add_card_to_library(0, catalog::hill_giant());
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: nahiri, ability_index: 2, target: None, x_value: None })
        .expect("-8");
    drain_stack(&mut g);
    assert!(g.computed_permanent(giant).unwrap().keywords().contains(&Keyword::Haste));
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == giant), "back to hand at the end step");
}

/// Kari Zev, Skyship Raider — Ragavan enters tapped and attacking (CR 508.3a)
/// and is exiled at end of combat.
#[test]
fn kari_zev_ragavan_attacks_and_leaves() {
    let mut g = main_phase();
    let kari = g.add_card_to_battlefield(0, catalog::kari_zev_skyship_raider());
    g.clear_sickness(kari);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: kari,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let rag = g.battlefield.iter().find(|c| c.definition.name == "Ragavan").expect("Ragavan").id;
    assert!(g.battlefield_find(rag).unwrap().tapped);
    assert!(g.attacking.iter().any(|a| a.attacker == rag), "tapped and attacking");
    g.process_attacking_token_cleanup();
    assert!(g.battlefield_find(rag).is_none(), "exiled at end of combat");
}

/// Bloodthirsty Adversary — kicked once, it exiles an instant with mana value
/// 3 or less from your graveyard, copies it and casts the copy free.
#[test]
fn bloodthirsty_adversary_recasts_a_graveyard_spell() {
    let mut g = main_phase();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let adv = g.add_card_to_battlefield(0, catalog::bloodthirsty_adversary());
    g.battlefield_find_mut(adv).unwrap().kick_count = 1;
    let etb = catalog::bloodthirsty_adversary().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(adv, 0, None, 0);
    g.resolve_effect(&etb, &ctx).expect("etb");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt), "the card is exiled");
    assert_eq!(g.players[1].life, 17, "its copy was cast free at the opponent");
}

/// Aura of Silence — only *opponents'* artifact and enchantment spells cost
/// {2} more; its controller's don't.
#[test]
fn aura_of_silence_taxes_opponents_only() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::aura_of_silence());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, ring, None).expect("its controller pays {1}");
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let ring2 = g.add_card_to_hand(1, catalog::sol_ring());
    g.players[1].mana_pool.add_colorless(1);
    assert!(cast(&mut g, ring2, None).is_err(), "an opponent owes {{3}}");
}

/// Compulsive Research — "discards two cards unless they discard a land card".
#[test]
fn compulsive_research_discards_a_land_instead() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    g.add_card_to_hand(0, catalog::island());
    let cr = g.add_card_to_hand(0, catalog::compulsive_research());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, cr, Some(Target::Player(0))).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 3, "one land discarded, three Bolts kept");
}

/// Compulsive Research — "*Target player* draws three cards. Then that player
/// discards …": aimed at an opponent, they draw and they discard.
#[test]
fn compulsive_research_can_target_an_opponent() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::lightning_bolt());
    }
    let cr = g.add_card_to_hand(0, catalog::compulsive_research());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let mine = g.players[0].hand.len();
    cast(&mut g, cr, Some(Target::Player(1))).expect("cast at P1");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), mine - 1, "the caster only spent the card");
    assert_eq!(g.players[1].hand.len(), 1, "P1 drew three Bolts and, with no land, discarded two");
    assert_eq!(g.players[1].graveyard.len(), 2);
}

/// Mask of Griselbrand — "you may pay X life … If you do, draw X cards": the
/// cards cost life.
#[test]
fn mask_of_griselbrand_pays_life_for_the_cards() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let mask = g.add_card_to_battlefield(0, catalog::mask_of_griselbrand());
    g.battlefield_find_mut(mask).unwrap().attached_to = Some(giant);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.remove_to_graveyard_with_triggers(giant);
    drain_stack(&mut g);
    assert_eq!((g.players[0].hand.len(), g.players[0].life), (3, 17));
}

/// Cankerbloom — "Choose one": the artifact mode destroys the artifact and
/// proliferates nothing (it did all three).
#[test]
fn cankerbloom_chooses_one_mode() {
    use crabomination::card::CounterType;
    let mut g = main_phase();
    let cb = g.add_card_to_battlefield(0, catalog::cankerbloom());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cb,
        ability_index: 0,
        target: Some(Target::Permanent(ring)),
        additional_targets: vec![],
        x_value: None,
        mode: Some(0),
    })
    .expect("sac for the artifact mode");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_none());
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "no proliferate");
}

/// Ursine Monstrosity — "choose an opponent at random. This creature attacks
/// that player this combat if able" (CR 508.1d) names a defender.
#[test]
fn ursine_monstrosity_must_attack_a_chosen_opponent() {
    let mut g = crabomination::game::multi_player_game(3);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::ursine_monstrosity());
    g.add_card_to_library(0, catalog::island());
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    let chosen = c.chosen_player.expect("an opponent is chosen");
    assert!(chosen == 1 || chosen == 2);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::MustAttackChosenPlayer));
}

/// Finale of Devastation — "a creature card with mana value X or less" from
/// "your library and/or graveyard": at X=1 a 4-drop is out of reach and a
/// 1-drop in the graveyard is not.
#[test]
fn finale_of_devastation_is_capped_by_x_and_reaches_the_graveyard() {
    let mut g = main_phase();
    let big = g.add_card_to_library(0, catalog::serra_angel());
    let elf = g.add_card_to_graveyard(0, catalog::llanowar_elves());
    let id = g.add_card_to_hand(0, catalog::finale_of_devastation());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(1) })
        .expect("X=1");
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_none(), "mana value 5 > X");
    assert!(g.battlefield_find(elf).is_some(), "the graveyard 1-drop");
}

/// Comeuppance — damage from sources you don't control is prevented and
/// reflected: an attacking creature takes its own damage back; a Bolt's 3
/// goes to the Bolt's controller.
#[test]
fn comeuppance_reflects_what_it_prevents() {
    let mut g = main_phase();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let c = g.add_card_to_hand(0, catalog::comeuppance());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, c, None).expect("comeuppance");
    drain_stack(&mut g);
    let (mine, theirs) = (g.players[0].life, g.players[1].life);
    g.priority.player_with_priority = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Player(0))).expect("bolt at P0");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, mine, "prevented");
    assert_eq!(g.players[1].life, theirs - 3, "a noncreature source: its controller takes it");
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: bear,
        target: crabomination::game::types::AttackTarget::Player(0),
    }]))
    .expect("attack");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("damage");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, mine, "combat damage prevented");
    assert!(g.battlefield_find(bear).is_none(), "the bear took its own 2 back");
}

/// Comeuppance — it protects you and your planeswalkers; it was a fog for the
/// whole table (your own attackers dealt nothing).
#[test]
fn comeuppance_is_not_a_fog() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let c = g.add_card_to_hand(0, catalog::comeuppance());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, c, None).expect("comeuppance");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: bear,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("damage");
    assert_eq!(g.players[1].life, 18);
}

/// Decree of Justice — "When you cycle this card, you may pay {X}. If you do,
/// create X 1/1 white Soldier creature tokens."
#[test]
fn decree_of_justice_cycles_into_soldiers() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::island());
    let d = g.add_card_to_hand(0, catalog::decree_of_justice());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Amount(2)]));
    g.perform_action(GameAction::Cycle { card_id: d, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    let soldiers = g.battlefield.iter().filter(|c| c.definition.name == "Soldier").count();
    assert_eq!(soldiers, 2);
}

/// Experiment Twelve — "this creature *or another creature you control* is
/// turned face up": another disguised creature flipping grows too.
#[test]
fn experiment_twelve_grows_your_other_flips() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::experiment_twelve());
    let other = g.add_card_to_battlefield(0, catalog::hill_giant());
    let ev = vec![GameEvent::TurnedFaceUp { card_id: other }];
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(other).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne), 3);
}

/// Guardian Project — no card when the entering creature shares a name with a
/// creature card in your graveyard; a card when it's the only one.
#[test]
fn guardian_project_skips_a_repeated_name() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_battlefield(0, catalog::guardian_project());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let enter = |g: &mut GameState, def| {
        let id = g.add_card_to_battlefield(0, def);
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: id }]);
        drain_stack(g);
    };
    enter(&mut g, catalog::grizzly_bears());
    assert!(g.players[0].hand.is_empty(), "a Bears is in the graveyard");
    enter(&mut g, catalog::hill_giant());
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Pteramander — "{1} less to activate for each instant and sorcery card in
/// your graveyard": seven of them make Adapt 4 cost {U}.
#[test]
fn pteramander_adapts_cheaper_per_graveyard_spell() {
    let mut g = main_phase();
    let pt = g.add_card_to_battlefield(0, catalog::pteramander());
    for _ in 0..7 {
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
    }
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: pt, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("{U} after seven off");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(pt).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne), 4);
}

/// Mobilized District — "{1} less for each legendary creature and
/// planeswalker you control": two legends make its {4} a {2}.
#[test]
fn mobilized_district_is_cheaper_per_legend() {
    let mut g = main_phase();
    let md = g.add_card_to_battlefield(0, catalog::mobilized_district());
    g.add_card_to_battlefield(0, catalog::magda_brazen_outlaw());
    g.add_card_to_battlefield(0, catalog::kari_zev_skyship_raider());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: md, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("{2} with two legends");
    drain_stack(&mut g);
    assert!(g.computed_permanent(md).unwrap().card_types().contains(&crabomination::card::CardType::Creature));
}

/// Voracious Fell Beast — "Create a Food token for each creature sacrificed
/// this way": three opponents with a creature each make three Foods.
#[test]
fn voracious_fell_beast_makes_a_food_per_sacrifice() {
    let mut g = crabomination::game::multi_player_game(4);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    for p in 1..4 {
        g.add_card_to_battlefield(p, catalog::grizzly_bears());
    }
    let vfb = g.add_card_to_hand(0, catalog::voracious_fell_beast());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, vfb, None).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Food").count(), 3);
}

/// Circuitous Route — "basic land cards and/or Gate cards".
#[test]
fn circuitous_route_fetches_gates() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    let gate = g.add_card_to_library(0, catalog::azorius_guildgate());
    let cr = g.add_card_to_hand(0, catalog::circuitous_route());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Search(Some(gate)), DecisionAnswer::Search(None)]));
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, cr, None).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(gate).is_some_and(|c| c.tapped));
}

/// Lethal Scheme — "Each creature that convoked this spell connives."
#[test]
fn lethal_scheme_convokers_connive() {
    let mut g = main_phase();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    let helper = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::hill_giant());
    let ls = g.add_card_to_hand(0, catalog::lethal_scheme());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpellConvoke {
        card_id: ls, target: Some(Target::Permanent(victim)), additional_targets: vec![], mode: None, x_value: None,
        convoke_creatures: vec![helper],
    })
    .expect("convoked");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none());
    assert_eq!(
        g.battlefield_find(helper).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne),
        1,
        "the convoker connived and discarded a nonland card",
    );
}

/// Augur of Autumn — coven (three creatures with different powers) lets you
/// cast creature spells from the top of your library; without it, no.
#[test]
fn augur_of_autumn_coven_casts_creatures_from_the_top() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::augur_of_autumn()); // power 2
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
    let top = g.add_card_to_library(0, catalog::llanowar_elves());
    g.players[0].mana_pool.add(Color::Green, 2);
    assert!(cast(&mut g, top, None).is_err(), "no coven yet");
    g.add_card_to_battlefield(0, catalog::hill_giant()); // power 3
    g.add_card_to_battlefield(0, catalog::llanowar_elves()); // power 1
    cast(&mut g, top, None).expect("coven: cast from the top");
    drain_stack(&mut g);
    assert!(g.battlefield_find(top).is_some());
}

/// Disallow — "Counter target spell, activated ability, or triggered
/// ability": it counters Legion Warboss's begin-combat trigger.
#[test]
fn disallow_counters_a_triggered_ability() {
    let mut g = main_phase();
    g.active_player_idx = 1;
    let boss = g.add_card_to_battlefield(1, catalog::legion_warboss());
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    assert!(!g.stack.is_empty(), "the Warboss trigger waits on the stack");
    let d = g.add_card_to_hand(0, catalog::disallow());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    cast(&mut g, d, Some(Target::Permanent(boss))).expect("Disallow the trigger");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Goblin"));
}

/// Mizzix of the Izmagnus — experience only for an instant or sorcery with
/// mana value *greater than* your experience: at 1, a Bolt gives none.
#[test]
fn mizzix_gains_experience_only_above_its_count() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::mizzix_of_the_izmagnus());
    g.players[0].experience = 1;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt, {R} after the discount too");
    drain_stack(&mut g);
    assert_eq!(g.players[0].experience, 1, "mana value 1 is not greater than 1");
}

/// River Song's Diary — only an instant or sorcery cast *from a hand* is
/// exiled with it; one cast from exile (Etali's free cast) is not.
#[test]
fn river_songs_diary_takes_hand_cast_spells_only() {
    use crabomination::effect::{Effect, PlayerRef, Selector, Value};
    let mut g = main_phase();
    let diary = g.add_card_to_battlefield(0, catalog::river_songs_diary());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt && c.exiled_with == Some(diary)), "hand-cast: exiled with the Diary");
    let top = g.add_card_to_library(0, catalog::lightning_bolt());
    g.players[0].library.rotate_right(1);
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &Effect::Seq(vec![
            Effect::ExileTopOfLibrary {
                who: Selector::Player(PlayerRef::You),
                amount: Value::ONE,
                link_to_source: false,
                face_down: false,
            },
            Effect::CastExiledFree {
                what: Selector::ExiledThisResolution { filter: crabomination::card::SelectionRequirement::Any },
            },
        ]),
        &ctx,
    )
    .expect("free cast from exile");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == top), "cast from exile: to the graveyard");
}

/// Imprisoned in the Moon — the enchanted permanent is a colorless land *with
/// "{T}: Add {C}"* (CR 613.1f: the same effect removes the old abilities and
/// grants this one). Its controller, not the Aura's, taps it for mana.
#[test]
fn imprisoned_in_the_moon_grants_tap_for_colorless() {
    let mut g = main_phase();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let aura = g.add_card_to_hand(0, catalog::imprisoned_in_the_moon());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, aura, Some(Target::Permanent(angel))).expect("Imprisoned in the Moon");
    drain_stack(&mut g);
    assert!(g.computed_permanent(angel).unwrap().lost_all_abilities);
    g.priority.player_with_priority = 1;
    let before = g.players[1].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: angel, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("the granted {T}: Add {C}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].mana_pool.total(), before + 1, "one colorless for the land's controller");
    assert!(g.battlefield_find(angel).unwrap().tapped);
}

/// Serra Avenger — "You can't cast this spell during your first, second, or
/// third turns of the game." Played through real turns (turn 1 is the
/// starting seat's, CR 103.5; each later turn is counted as it is handed on):
/// refused in player 0's first three main phases,
/// cast in the fourth.
#[test]
fn serra_avenger_waits_for_your_fourth_turn() {
    let mut g = two_player_game();
    for p in 0..2 {
        for _ in 0..12 {
            g.add_card_to_library(p, catalog::plains());
        }
    }
    let avenger = g.add_card_to_hand(0, catalog::serra_avenger());
    for turn in 1..=4 {
        while !(g.step == TurnStep::PreCombatMain && g.active_player_idx == 0) {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
        g.players[0].mana_pool.add(Color::White, 2);
        let r = cast(&mut g, avenger, None);
        if turn < 4 {
            assert!(
                matches!(r, Err(GameError::SelectionRequirementViolated)),
                "your turn {turn}: can't be cast, got {r:?}"
            );
            while g.step == TurnStep::PreCombatMain {
                g.perform_action(GameAction::PassPriority).expect("leave main");
            }
        } else {
            r.expect("your fourth turn: castable");
        }
    }
}

/// Master Biomancer — each other creature you control enters with +1/+1
/// counters equal to its power *and as a Mutant in addition to its other
/// types* (CR 614.1c). A cast bear is a Bear Mutant 4/4; the type stays after
/// the Biomancer leaves; an opponent's bear gets neither.
#[test]
fn master_biomancer_makes_entering_creatures_mutants() {
    use crabomination::card::CreatureType;
    let mut g = main_phase();
    let bio = g.add_card_to_battlefield(0, catalog::master_biomancer());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, bear, None).expect("bear");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Mutant), "a Mutant");
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Bear), "in addition to Bear");
    assert_eq!((cp.power, cp.toughness), (4, 4), "two counters from Biomancer's power");
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(!g.computed_permanent(theirs).unwrap().subtypes().creature_types.contains(&CreatureType::Mutant));
    g.remove_to_graveyard_with_triggers(bio);
    assert!(g.computed_permanent(bear).unwrap().subtypes().creature_types.contains(&CreatureType::Mutant), "the type stays");
}

/// Ajani's Chosen — "If that enchantment is an Aura, you may attach it to the
/// token." Holy Strength cast on our bear moves to the new Cat on a yes;
/// Pacifism on an opponent's creature is never lifted onto our token.
#[test]
fn ajanis_chosen_may_move_the_aura_onto_the_cat() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::ajanis_chosen());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::holy_strength());
    g.players[0].mana_pool.add(Color::White, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, aura, Some(Target::Permanent(bear))).expect("Holy Strength");
    drain_stack(&mut g);
    let host = g.battlefield_find(aura).and_then(|a| a.attached_to).expect("still attached");
    assert_ne!(host, bear, "moved off the bear");
    assert_eq!(g.battlefield_find(host).unwrap().definition.name, "Cat", "onto the token");

    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pac = g.add_card_to_hand(0, catalog::pacifism());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, pac, Some(Target::Permanent(theirs))).expect("Pacifism");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(pac).and_then(|a| a.attached_to), Some(theirs), "stays on their creature");
}

/// Mishra's Factory — "{T}: Target Assembly-Worker creature gets +1/+1 until
/// end of turn." One animated Factory (a 2/2 Assembly-Worker) pumped by a
/// second is a 3/3.
#[test]
fn mishras_factory_pumps_an_assembly_worker() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(0, catalog::mishras_factory());
    let b = g.add_card_to_battlefield(0, catalog::mishras_factory());
    g.players[0].mana_pool.add_colorless(1);
    let act = |g: &mut GameState, card_id, ability_index, target| {
        g.perform_action(GameAction::ActivateAbility {
            card_id, ability_index, target, additional_targets: Vec::new(), x_value: None, mode: None,
        })
    };
    act(&mut g, a, 1, None).expect("animate");
    drain_stack(&mut g);
    act(&mut g, b, 2, Some(Target::Permanent(a))).expect("pump the Assembly-Worker");
    drain_stack(&mut g);
    let cp = g.computed_permanent(a).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(g.battlefield_find(b).unwrap().tapped);
}

/// Astrologian's Planisphere — the equipped creature has "whenever you draw
/// your third card each turn, put a +1/+1 counter on this creature" (CR 121):
/// the second draw does nothing, the third adds one, the fourth nothing.
#[test]
fn astrologians_planisphere_counts_the_third_draw() {
    use crabomination::card::CounterType;
    let mut g = main_phase();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let eq = g.add_card_to_battlefield(0, catalog::astrologians_planisphere());
    g.battlefield_find_mut(eq).unwrap().attached_to = Some(bear);
    let counters = |g: &GameState| g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne);
    for (nth, want) in [(1, 0), (2, 0), (3, 1), (4, 1)] {
        let mut evs = Vec::new();
        assert!(g.draw_one(0, &mut evs));
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(counters(&g), want, "after draw {nth}");
    }
}

/// Conduit of Worlds — "{T}: Choose target nonland permanent card in your
/// graveyard. If you haven't cast a spell this turn, you may cast that card.
/// If you do, you can't cast additional spells this turn." The bear is cast
/// from the graveyard for its cost; after it, a Bolt from hand is refused.
/// With a spell already cast this turn, the activation grants nothing.
#[test]
fn conduit_of_worlds_casts_from_the_graveyard_then_locks() {
    let mut g = main_phase();
    let conduit = g.add_card_to_battlefield(0, catalog::conduit_of_worlds());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let act = |g: &mut GameState, target| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: conduit, ability_index: 0, target, additional_targets: Vec::new(), x_value: None, mode: None,
        })
    };
    act(&mut g, Some(Target::Permanent(bear))).expect("activate");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast the bear from the graveyard");
    assert_eq!(g.players[0].mana_pool.total(), 0, "its cost was paid");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
    g.players[0].mana_pool.add(Color::Red, 1);
    assert!(cast(&mut g, bolt, Some(Target::Player(1))).is_err(), "no additional spells this turn");

    // A fresh turn with a spell already cast: the ability does nothing.
    let mut g = main_phase();
    let conduit = g.add_card_to_battlefield(0, catalog::conduit_of_worlds());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt first");
    drain_stack(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: conduit, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().find(|c| c.id == bear).unwrap().may_play_until.is_none(), "no permission");

    // Bug fix: the window is this step's only — the grant arms the step
    // sweep (it used to skip it, and the permission outlived the step).
    let mut g = main_phase();
    let conduit = g.add_card_to_battlefield(0, catalog::conduit_of_worlds());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: conduit, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let granted = |g: &GameState| g.players[0].graveyard.iter().find(|c| c.id == bear).unwrap().may_play_until.is_some();
    assert!(granted(&g));
    let _ = g.advance_step(Vec::new());
    assert!(!granted(&g), "the next step closes the window");
}

/// Mishra's Factory, Blinkmoth Nexus, Mishra's Foundry — each "becomes a …
/// artifact creature" and carries its printed pump: (land, animate cost,
/// pump cost, pump size, pump needs an attacker).
#[test]
fn colorless_manlands_are_artifacts_and_pump() {
    use crabomination::card::CardType;
    type Row = (fn() -> CardDefinition, u32, u32, i32, bool);
    let table: [Row; 3] = [
        (catalog::mishras_factory, 1, 0, 1, false),
        (catalog::blinkmoth_nexus, 1, 1, 1, false),
        (catalog::mishras_foundry, 2, 1, 2, true),
    ];
    for (f, animate, pump, n, attacking) in table {
        let mut g = main_phase();
        let pumper = g.add_card_to_battlefield(0, f());
        let body = g.add_card_to_battlefield(0, f());
        g.clear_sickness(body);
        g.players[0].mana_pool.add_colorless(animate + pump);
        let act = |g: &mut GameState, id, idx, target| {
            g.perform_action(GameAction::ActivateAbility {
                card_id: id, ability_index: idx, target, additional_targets: vec![], x_value: None, mode: None,
            })
        };
        act(&mut g, body, 1, None).expect("animate");
        drain_stack(&mut g);
        let cp = g.computed_permanent(body).unwrap();
        assert!(cp.card_types().contains(&CardType::Artifact), "{} is an artifact creature", cp.def.name);
        let base = cp.power;
        if attacking {
            g.step = TurnStep::DeclareAttackers;
            g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
                attacker: body,
                target: crabomination::game::types::AttackTarget::Player(1),
            }]))
            .expect("attack");
        }
        act(&mut g, pumper, 2, Some(Target::Permanent(body))).expect("pump");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(body).unwrap().power, base + n, "{}", cp.def.name);
    }
}

/// Zack Fair — "{1}, Sacrifice Zack Fair: Target creature you control gains
/// indestructible until end of turn. Put Zack Fair's counters on that
/// creature" — read off the sacrificed Zack (CR 608.2h).
#[test]
fn zack_fair_hands_its_counters_over() {
    use crabomination::card::CounterType;
    let mut g = main_phase();
    let zack = g.add_card_to_battlefield(0, catalog::zack_fair());
    g.battlefield_find_mut(zack).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: zack, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("sac Zack");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 2);
    assert!(g.computed_permanent(bear).unwrap().power == 4);
}

/// Shorikai, Genesis Engine — its Pilot "crews Vehicles as though its power
/// were 2 greater" (CR 702.122e): one 1/1 Pilot crews The Belligerent (crew 3).
#[test]
fn shorikai_pilot_crews_as_power_three() {
    let mut g = main_phase();
    let shorikai = g.add_card_to_battlefield(0, catalog::shorikai_genesis_engine());
    let ship = g.add_card_to_battlefield(0, catalog::the_belligerent());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: shorikai, ability_index: 0, target: None,
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let pilot = g.battlefield.iter().find(|c| c.definition.name == "Pilot").expect("pilot").id;
    g.perform_action(GameAction::Crew { vehicle: ship, crew_creatures: vec![pilot] }).expect("crew 3");
}

/// Powder Ganger — "destroy *up to one* target artifact": the opponent's Sol
/// Ring goes; with only our own artifact out, nothing is destroyed.
#[test]
fn powder_ganger_destroys_up_to_one_artifact() {
    let mut g = main_phase();
    let theirs = g.add_card_to_battlefield(1, catalog::sol_ring());
    let pg = g.add_card_to_hand(0, catalog::powder_ganger());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, pg, None).expect("Powder Ganger");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "their artifact destroyed");

    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::sol_ring());
    let pg = g.add_card_to_hand(0, catalog::powder_ganger());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, pg, None).expect("Powder Ganger");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_some(), "zero targets is legal: ours survives");
}

/// Primeval Spawn — "You may cast any number of spells with total mana value
/// 10 or less from among them" (not one): five Bears fit, a sixth doesn't.
#[test]
fn primeval_spawn_casts_spells_totalling_ten() {
    let mut g = main_phase();
    let spawn = g.add_card_to_battlefield(0, catalog::primeval_spawn());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let mut events = Vec::new();
    g.destroy_permanent(spawn, false, &mut events);
    drain_stack(&mut g);
    let bears = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    assert_eq!(bears, 5, "five two-drops total ten");
}

/// Celeborn the Wise — "Whenever you scry, Celeborn gets +1/+1 until end of
/// turn for each card looked at while scrying this way": scry 2 is +2/+2, and
/// a surveil is not a scry.
#[test]
fn celeborn_grows_per_card_scried() {
    use crabomination::effect::{Effect, PlayerRef, Value};
    let mut g = main_phase();
    let celeborn = g.add_card_to_battlefield(0, catalog::celeborn_the_wise());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let ev = g.resolve_effect(&Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) }, &ctx).expect("surveil");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(celeborn).unwrap().power, 3, "surveil isn't scry");
    let ev = g.resolve_effect(&Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }, &ctx).expect("scry");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(celeborn).unwrap().power, 5);
}

/// Mirkwood Trapper — "Whenever a player attacks you, target attacking
/// creature gets -2/-0 until end of turn": one attacker, chosen, not the first
/// declared (the auto-targeter takes the bigger one).
#[test]
fn mirkwood_trapper_shrinks_a_target_attacker() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::mirkwood_trapper());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wurm = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw());
    for c in [bear, wurm] {
        g.clear_sickness(c);
    }
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: bear, target: AttackTarget::Player(0) },
        Attack { attacker: wurm, target: AttackTarget::Player(0) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    let p = |g: &GameState, c| g.computed_permanent(c).unwrap().power;
    assert_eq!((p(&g, bear), p(&g, wurm)), (2, 4));
}

/// Ondu Spiritdancer — "Whenever an enchantment you control enters, you may
/// create a token that's a copy of it. Do this only once each turn": the
/// second enchantment that turn isn't copied (and isn't asked about).
#[test]
fn ondu_spiritdancer_copies_the_entering_enchantment_once_a_turn() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    g.add_card_to_battlefield(0, catalog::ondu_spiritdancer());
    for _ in 0..2 {
        let e = g.add_card_to_hand(0, catalog::soaring_lightbringer());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(4);
        cast(&mut g, e, None).expect("cast");
        drain_stack(&mut g);
    }
    let count = |n: &str| g.battlefield.iter().filter(|c| c.definition.name == n).count();
    assert_eq!(count("Ondu Spiritdancer"), 1);
    assert_eq!(count("Soaring Lightbringer"), 3, "two cast, one copy");
}

/// Speed, Young Avenger — "target creature with haste can't be blocked this
/// turn except by creatures with haste": a haste blocker still can, so it is
/// not plain unblockability.
#[test]
fn speed_young_avenger_is_blockable_by_haste() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let speed = g.add_card_to_battlefield(0, catalog::speed_young_avenger());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
    drain_stack(&mut g);
    let cp = g.computed_permanent(speed).unwrap();
    let kws = cp.keywords();
    assert!(!kws.contains(&Keyword::Unblockable));
    assert!(kws.iter().any(|k| matches!(k, Keyword::CantBeBlockedExceptBy(_))));
}

/// Triumph of Saint Katherine — Praesidium Protectiva: dying, it is shuffled
/// with the top six into a pile that goes back on top (a miracle set-up).
#[test]
fn triumph_of_saint_katherine_hides_in_the_top_seven() {
    let mut g = main_phase();
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::plains());
    }
    let seventh_down = g.players[0].library[6].id;
    let t = g.add_card_to_battlefield(0, catalog::triumph_of_saint_katherine());
    let mut events = Vec::new();
    g.destroy_permanent(t, false, &mut events);
    drain_stack(&mut g);
    let lib = &g.players[0].library;
    assert_eq!(lib.len(), 11);
    let at = lib.iter().position(|c| c.id == t).expect("in the library");
    assert!(at < 7, "among the top seven, at {at}");
    assert_eq!(lib[7].id, seventh_down, "the rest of the library is untouched");
}

/// Callidus Assassin — "You may have this creature enter **tapped** as a copy
/// of any creature on the battlefield, except it has 'When this creature
/// enters, destroy up to one other target creature with the same name'."
#[test]
fn callidus_assassin_enters_tapped_and_kills_its_namesake() {
    let mut g = main_phase();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ca = g.add_card_to_hand(0, catalog::callidus_assassin());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, ca, None).expect("cast");
    drain_stack(&mut g);
    let me = g.battlefield_find(ca).expect("on the battlefield");
    assert_eq!(me.definition.name, "Grizzly Bears");
    assert!(me.tapped, "entered tapped");
    assert!(g.battlefield_find(theirs).is_none(), "the namesake is destroyed");
}

/// Bug fix: "you may play that card" / "mana of any type can be spent" is a
/// paid cast. Seven pod-deck grants (Yasmin Khan, Embrace the Unknown,
/// Heartless Conscription, Armory Paladin, The Flux, Advanced Reconstruction,
/// Fateful Tempest) stamped no cost, so the exiled card was cast for free.
#[test]
fn impulse_grants_bill_the_card_cost() {
    let from_exile = |g: &mut GameState, card_id: CardId| {
        g.perform_action(GameAction::CastFromZoneWithoutPaying {
            card_id, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
    };
    let exiled_bear = |g: &GameState| g.exile.iter().find(|c| c.definition.name == "Grizzly Bears").map(|c| c.id);

    // Yasmin Khan's tap impulse.
    let mut g = main_phase();
    let yasmin = g.add_card_to_battlefield(0, catalog::yasmin_khan());
    g.clear_sickness(yasmin);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: yasmin, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("tap Yasmin");
    drain_stack(&mut g);
    let bear = exiled_bear(&g).expect("exiled");
    assert!(from_exile(&mut g, bear).is_err(), "not free");
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    from_exile(&mut g, bear).expect("paid {1}{G}");

    // Embrace the Unknown's two cards.
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let embrace = g.add_card_to_hand(0, catalog::embrace_the_unknown());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, embrace, None).expect("cast");
    drain_stack(&mut g);
    let bear = exiled_bear(&g).expect("exiled");
    assert!(from_exile(&mut g, bear).is_err(), "not free");

    // Heartless Conscription: any type of mana, but still the mana value.
    let mut g = main_phase();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let conscript = g.add_card_to_hand(0, catalog::heartless_conscription());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(6);
    cast(&mut g, conscript, None).expect("cast");
    drain_stack(&mut g);
    assert!(from_exile(&mut g, theirs).is_err(), "not free");
    g.players[0].mana_pool.add(Color::Blue, 2);
    from_exile(&mut g, theirs).expect("two blue pay for {1}{G}");

    // The reverse: Chandra, Torch of Defiance's +1 "cast that card" is the
    // card's own cost, not its mana value in any color.
    let mut g = main_phase();
    let chandra = g.add_card_to_battlefield(0, catalog::chandra_torch_of_defiance());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: chandra, ability_index: 0, target: None, x_value: None })
        .expect("+1");
    drain_stack(&mut g);
    let bear = exiled_bear(&g).expect("exiled");
    g.players[0].mana_pool.add(Color::Red, 2);
    assert!(from_exile(&mut g, bear).is_err(), "{{R}}{{R}} doesn't pay {{1}}{{G}}");
    g.players[0].mana_pool.add(Color::Green, 1);
    from_exile(&mut g, bear).expect("with a green");
}

/// Ramos, Dragon Engine — a counter per color of the cast spell (Terminate
/// two, Sol Ring none); "remove five +1/+1 counters" is a cost, the ability
/// doesn't tap, and it works once each turn. It counted one per spell, tapped,
/// and made mana from any number of counters.
#[test]
fn ramos_counts_colors_and_pays_five_counters_once_a_turn() {
    use crabomination::card::CounterType;
    let mut g = main_phase();
    let ramos = g.add_card_to_battlefield(0, catalog::ramos_dragon_engine());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let counters = |g: &GameState| g.battlefield_find(ramos).unwrap().counter_count(CounterType::PlusOnePlusOne);
    let terminate = g.add_card_to_hand(0, catalog::terminate());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, terminate, Some(Target::Permanent(bear))).expect("Terminate");
    drain_stack(&mut g);
    assert_eq!(counters(&g), 2, "black and red");
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, ring, None).expect("Sol Ring");
    drain_stack(&mut g);
    assert_eq!(counters(&g), 2, "a colorless spell adds none");
    let act = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: ramos, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
        })
    };
    assert!(act(&mut g).is_err(), "two counters can't pay five");
    g.battlefield_find_mut(ramos).unwrap().add_counters(CounterType::PlusOnePlusOne, 8);
    act(&mut g).expect("remove five");
    assert_eq!(counters(&g), 5);
    assert_eq!(g.players[0].mana_pool.total(), 10);
    assert!(!g.battlefield_find(ramos).unwrap().tapped, "no {{T}} in the cost");
    assert!(act(&mut g).is_err(), "once each turn");
}

/// "Each upkeep" / "each end step" / "the next end step" are every player's
/// step (CR 503.1 / 513.1), not only the controller's — the scope these five
/// shipped with. Driven on the opponent's turn (seat 1 active).
#[test]
fn each_step_triggers_fire_on_an_opponents_turn() {
    use crabomination::card::CounterType;
    let opp_turn = |step| {
        let mut g = main_phase();
        g.active_player_idx = 1;
        g.step = step;
        g
    };
    // Tendershoot Dryad: a Saproling on every upkeep.
    let mut g = opp_turn(TurnStep::Upkeep);
    g.add_card_to_battlefield(0, catalog::tendershoot_dryad());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Saproling" && c.controller == 0));
    // Séance Board: morbid soul counter at every end step.
    let mut g = opp_turn(TurnStep::End);
    let board = g.add_card_to_battlefield(0, catalog::seance_board());
    g.players[1].creatures_died_this_turn = 1;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(board).unwrap().counter_count(CounterType::Soul), 1);
    // Joined Researchers: prepared at every end step while an opponent holds more.
    let mut g = opp_turn(TurnStep::End);
    let jr = g.add_card_to_battlefield(0, catalog::joined_researchers());
    g.add_card_to_hand(1, catalog::forest());
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(jr).unwrap().counter_count(CounterType::Prepared), 1);
    // Manaform Hellkite: an Illusion made on the opponent's turn is exiled at
    // that turn's end step, not left to attack on yours.
    let mut g = opp_turn(TurnStep::PreCombatMain);
    g.add_card_to_battlefield(0, catalog::manaform_hellkite());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt on their turn");
    drain_stack(&mut g);
    let illusion = |g: &GameState| g.battlefield.iter().any(|c| c.definition.name == "Dragon Illusion");
    assert!(illusion(&g));
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(!illusion(&g), "exiled at the next end step, whoever's turn");
}

/// Instill Furor — "at the beginning of YOUR end step": the enchanted
/// creature's controller's. An opponent's end step, when it cannot have
/// attacked, no longer sacrifices it.
#[test]
fn instill_furor_fires_only_on_its_hosts_controllers_end_step() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::instill_furor());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    g.active_player_idx = 1;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "opponent's end step");
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "its own end step, no attack");
}

/// "Another creature dies" / "another nontoken Elf enters" / "a Warrior
/// attacks" / "a creature with a counter attacks one of your opponents" name
/// anyone's creature; these four shipped scoped to their controller's.
#[test]
fn anyones_creature_triggers_see_an_opponents_creature() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Attack, AttackTarget};
    // Reaper of the Wilds: an opponent's creature dying scries.
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::reaper_of_the_wilds());
    g.add_card_to_library(0, catalog::forest());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(bear))).expect("bolt the bear");
    let ev = drain_stack(&mut g);
    assert!(ev.iter().any(|e| matches!(e, GameEvent::ScriedOrSurveiled { player: 0, .. })));
    // Wirewood Hivemaster: an opponent's nontoken Elf, and not itself.
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.add_card_to_battlefield(0, catalog::wirewood_hivemaster());
    let insects = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Insect").count();
    drain_stack(&mut g);
    assert_eq!(insects(&g), 0, "itself entering is not another Elf");
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Green, 1);
    let elf = g.add_card_to_hand(1, catalog::llanowar_elves());
    cast(&mut g, elf, None).expect("their Elf");
    drain_stack(&mut g);
    assert_eq!(insects(&g), 1);
    // Najeela: an opponent's Warrior attacking asks; the headless seat declines.
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::najeela_the_blade_blossom());
    let orc = g.add_card_to_battlefield(1, catalog::elvish_warrior());
    g.clear_sickness(orc);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: orc, target: AttackTarget::Player(0) }]))
        .expect("their Warrior attacks");
    drain_stack(&mut g);
    let tok = g.battlefield.iter().find(|c| c.definition.name == "Warrior").expect("the token, on a yes");
    assert_eq!(tok.controller, 1, "its controller creates it");
    // Skyboon Evangelist: at three seats, seat 1's countered creature
    // attacking seat 2 (Skyboon's other opponent) gains flying.
    let mut g = multi_player_game(3);
    g.add_card_to_battlefield(0, catalog::skyboon_evangelist());
    let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(atk).unwrap().add_counters(crabomination::card::CounterType::PlusOnePlusOne, 1);
    g.clear_sickness(atk);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: atk, target: AttackTarget::Player(2) }]))
        .expect("attack seat 2");
    drain_stack(&mut g);
    assert!(g.permanent_has_keyword(atk, &Keyword::Flying));
}

/// Elemental Uprising / the Embodiment cycle / Lifespark Spellbomb: "becomes a
/// 4/4 (3/3) … until end of turn" is a set P/T that ends, not the Awaken
/// helper's permanent +1/+1 counters on a 0/0 (which left a counter-laden land
/// behind and stacked with the next animation).
#[test]
fn until_end_of_turn_land_animations_leave_no_counters() {
    use crabomination::card::CounterType;
    let mut g = main_phase();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let eu = g.add_card_to_hand(0, catalog::elemental_uprising());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, eu, Some(Target::Permanent(forest))).expect("uprising");
    drain_stack(&mut g);
    let cp = g.computed_permanent(forest).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.keywords().contains(&Keyword::Haste));
    assert_eq!(g.battlefield_find(forest).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    g.do_cleanup(&mut vec![]);
    assert!(!g.computed_permanent(forest).unwrap().card_types().contains(&crabomination::card::CardType::Creature));

    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let island = g.add_card_to_battlefield(0, catalog::island());
    let bomb = g.add_card_to_battlefield(0, catalog::lifespark_spellbomb());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility { card_id: bomb, ability_index: 0, target: Some(Target::Permanent(island)), additional_targets: vec![], x_value: None, mode: None })
        .expect("spellbomb");
    drain_stack(&mut g);
    let cp = g.computed_permanent(island).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(!cp.keywords().contains(&Keyword::Haste), "no haste on the Spellbomb's 3/3");
}

/// "Create … tokens. They gain menace and haste until end of turn." — the
/// grant ends with the turn; it was baked on the token definition (Mardu
/// Monument, Windcrag Siege's lifelink Goblin, Sokenzan, Krenko, Baron of Tin
/// Street, Harried Dronesmith, Salt Road Skirmish).
#[test]
fn created_tokens_gain_keywords_only_until_end_of_turn() {
    let mut g = main_phase();
    let monument = g.add_card_to_battlefield(0, catalog::mardu_monument());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility { card_id: monument, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("monument");
    drain_stack(&mut g);
    let warriors: Vec<CardId> = g.battlefield.iter().filter(|c| c.definition.name == "Warrior").map(|c| c.id).collect();
    assert_eq!(warriors.len(), 3);
    for &w in &warriors {
        let cp = g.computed_permanent(w).unwrap();
        let kws = cp.keywords();
        assert!(kws.contains(&Keyword::Menace) && kws.contains(&Keyword::Haste));
    }
    g.do_cleanup(&mut vec![]);
    for &w in &warriors {
        let cp = g.computed_permanent(w).unwrap();
        let kws = cp.keywords();
        assert!(!kws.contains(&Keyword::Menace) && !kws.contains(&Keyword::Haste), "gone after the turn");
    }
}

/// "Target ... card from YOUR graveyard" — ~35 cards read any graveyard
/// (`InGraveyard`), so Buried Ruin could take an opponent's artifact to your
/// hand. And Grave Researcher's Reanimate is the reverse: "from A graveyard".
#[test]
fn your_graveyard_targets_are_yours_and_a_graveyard_is_any() {
    let mut g = main_phase();
    let ruin = g.add_card_to_battlefield(0, catalog::buried_ruin());
    let theirs = g.add_card_to_graveyard(1, catalog::sol_ring());
    let mine = g.add_card_to_graveyard(0, catalog::sol_ring());
    let activate = |g: &mut GameState, t| {
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: ruin, ability_index: 1, target: Some(Target::Permanent(t)),
            additional_targets: vec![], x_value: None, mode: None,
        })
    };
    assert!(activate(&mut g, theirs).is_err(), "an opponent's graveyard isn't yours");
    activate(&mut g, mine).expect("your own artifact card");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == mine));
    // Reanimate (Grave Researcher's spell half) reaches an opponent's graveyard.
    let def = catalog::grave_researcher();
    let reanimate = def.prepare_spell.as_ref().expect("its spell").effect.clone();
    let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_ability(
        crabomination::card::CardId(0), 0, Some(Target::Permanent(bear)));
    g.resolve_effect(&reanimate, &ctx).unwrap();
    assert_eq!(g.battlefield_find(bear).map(|c| c.controller), Some(0));
}

/// "Exile target creature you **own**, then return it under your control"
/// (Slip On the Ring, Charming Prince, Sword of Hearth and Home) read "you
/// control": a borrowed creature could be flickered to keep it for good, and
/// your own stolen one couldn't be taken back.
#[test]
fn slip_on_the_ring_targets_by_owner_not_controller() {
    let mut g = main_phase();
    let borrowed = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(borrowed).unwrap().controller = 0;
    let stolen = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(stolen).unwrap().controller = 1;
    let slip = g.add_card_to_hand(0, catalog::slip_on_the_ring());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(cast(&mut g, slip, Some(Target::Permanent(borrowed))).is_err(), "not yours to flicker");
    cast(&mut g, slip, Some(Target::Permanent(stolen))).expect("your own card, back home");
    drain_stack(&mut g);
    let back = g.battlefield.iter().find(|c| c.owner == 0 && c.definition.name == "Grizzly Bears").unwrap();
    assert_eq!(back.controller, 0);
}

/// "Create a tapped Treasure / Beast / Horror token" — the token enters
/// tapped (Blood Money's sibling cards shipped untapped Treasures that paid
/// for the rest of the turn): Goldvein Hydra, Ancient Adamantoise, Great
/// Train Heist, Baloth Prime, Drana's Chosen, The Final Days.
#[test]
fn printed_tapped_tokens_enter_tapped() {
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::forest());
    let days = g.add_card_to_hand(0, catalog::the_final_days());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, days, None).expect("The Final Days");
    drain_stack(&mut g);
    let horrors: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Horror").collect();
    assert_eq!(horrors.len(), 2);
    assert!(horrors.iter().all(|c| c.tapped), "tapped Horrors");
    let hydra = g.add_card_to_battlefield(0, catalog::goldvein_hydra());
    g.battlefield_find_mut(hydra).unwrap().add_counters(crabomination::card::CounterType::PlusOnePlusOne, 3);
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let kill = crabomination::effect::Effect::Destroy {
        what: crabomination::effect::Selector::EachPermanent(crabomination::card::SelectionRequirement::HasCreatureType(CreatureType::Hydra)),
    };
    let evs = g.resolve_effect(&kill, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let treasures: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Treasure").collect();
    assert_eq!(treasures.len(), 3, "one per point of power");
    assert!(treasures.iter().all(|c| c.tapped), "tapped Treasures");
}

/// Sphinx of Foresight — "You may reveal this card from your opening hand. If
/// you do, scry 3 at the beginning of your first upkeep." (it shipped with
/// only the recurring upkeep scry 1).
#[test]
fn sphinx_of_foresight_scries_three_from_the_opening_hand() {
    let mut g = main_phase();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(0, catalog::sphinx_of_foresight());
    g.fire_start_of_game_effects();
    g.step = TurnStep::Untap;
    let mut ev = g.advance_step(Vec::new()).unwrap();
    ev.extend(drain_stack(&mut g));
    assert!(ev.iter().any(|e| matches!(e, GameEvent::ScriedOrSurveiled { player: 0, surveil: false, looked_at: 3 })));
}

/// Conditional self-cost reductions that shipped dropped: Gust of Wind ("{2}
/// less if you control a creature with flying") and Swampsnare Trap ("{1}
/// less if it targets a creature with flying").
#[test]
fn flyer_conditional_cost_reductions() {
    let mut g = main_phase();
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let gust = g.add_card_to_hand(0, catalog::gust_of_wind());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(cast(&mut g, gust, Some(Target::Permanent(opp))).is_err(), "{{3}}{{U}} without a flyer");
    g.add_card_to_battlefield(0, catalog::serra_angel());
    cast(&mut g, gust, Some(Target::Permanent(opp))).expect("{1}{U} with one");
    drain_stack(&mut g);
    let flyer = g.add_card_to_battlefield(1, catalog::serra_angel());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let trap = g.add_card_to_hand(0, catalog::swampsnare_trap());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(cast(&mut g, trap, Some(Target::Permanent(bear))).is_err(), "full price on a ground creature");
    cast(&mut g, trap, Some(Target::Permanent(flyer))).expect("{1}{B} on a flyer");
}

/// CR 111.4 — a token's name is its subtypes: Carrier Thrall's "Eldrazi
/// Scion" was a plain Eldrazi, so no Scion payoff counted it (also Eldrazi
/// Confluence, Basking Broodscale's Spawn, Sporemound's Saprolings shipped as
/// Plants; `scripts/audit_token_types.py --gate`).
#[test]
fn carrier_thrall_leaves_an_eldrazi_scion() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::carrier_thrall());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let kill = crabomination::effect::Effect::Destroy {
        what: crabomination::effect::Selector::EachPermanent(crabomination::card::SelectionRequirement::Creature),
    };
    let evs = g.resolve_effect(&kill, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let scion = g.battlefield.iter().find(|c| c.is_token).expect("a token").id;
    let cp = g.computed_permanent(scion).unwrap();
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Scion));
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Eldrazi));
}

/// Serra Paragon — "Once during each of your turns, you may play a land from
/// your graveyard or cast a permanent spell with mana value 3 or less from
/// your graveyard. If you do, it gains 'When this permanent is put into a
/// graveyard from the battlefield, exile it and you gain 2 life.'" Lands used
/// to be unlimited and the rider was never granted.
#[test]
fn serra_paragon_shares_its_once_and_grants_the_exile_rider() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::serra_paragon());
    let forest = g.add_card_to_graveyard(0, catalog::forest());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.perform_action(GameAction::PlayLandFromGraveyard(forest)).expect("the land is the once");
    g.players[0].mana_pool.add(Color::Green, 2);
    assert!(cast(&mut g, bears, None).is_err(), "the land used this turn's grant");

    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::serra_paragon());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, bears, None).expect("Bears from the graveyard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_some());
    let life = g.players[0].life;
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let kill = crabomination::effect::Effect::Destroy {
        what: crabomination::effect::Selector::EachPermanent(
            crabomination::card::SelectionRequirement::HasCreatureType(CreatureType::Bear),
        ),
    };
    let evs = g.resolve_effect(&kill, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bears), "exiled by the rider");
    assert_eq!(g.players[0].life, life + 2);
    let card = g.exile.iter().find(|c| c.id == bears).unwrap();
    assert!(card.definition.triggered_abilities.is_empty(), "the rider ended with the permanent");
}

/// Cost riders the cost-less scan (2026-09-27) found dropped — each pays
/// exactly the discounted cost from an exact pool.
fn pay_exactly(g: &mut GameState, id: CardId, generic: u32, color: Color, n: u32, target: Option<Target>) -> Result<(), GameError> {
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add_colorless(generic);
    g.players[0].mana_pool.add(color, n);
    cast(g, id, target)?;
    drain_stack(g);
    Ok(())
}

/// Tentative Connection costs {3} less with a menace creature out.
#[test]
fn tentative_connection_menace_discount() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let tc = g.add_card_to_hand(0, catalog::tentative_connection());
    assert!(pay_exactly(&mut g, tc, 0, Color::Red, 1, Some(Target::Permanent(victim))).is_err(), "full {{3}}{{R}} without menace");
    g.add_card_to_battlefield(0, catalog::boggart_brute());
    pay_exactly(&mut g, tc, 0, Color::Red, 1, Some(Target::Permanent(victim))).expect("{R} with a menace creature");
}

/// Umori: as it enters choose a card type; spells of that type cost {1}
/// less (creature is the first choice).
#[test]
fn umori_discounts_the_chosen_type() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    g.add_card_to_battlefield_entering(0, catalog::umori_the_collector());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    pay_exactly(&mut g, bear, 0, Color::Green, 1, None).expect("Bears for {G}");
}

/// Momo: the first non-Lemur flyer you cast during your turn costs {1}
/// less — not the second.
#[test]
fn momo_discounts_the_first_flyer_on_your_turn() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::momo_friendly_flier());
    let d1 = g.add_card_to_hand(0, catalog::wind_drake());
    let d2 = g.add_card_to_hand(0, catalog::wind_drake());
    pay_exactly(&mut g, d1, 1, Color::Blue, 1, None).expect("first Wind Drake for {1}{U}");
    assert!(pay_exactly(&mut g, d2, 1, Color::Blue, 1, None).is_err(), "the second is {{2}}{{U}}");
}

/// Fervent Champion: equips that target it cost {3} less.
#[test]
fn fervent_champion_discounts_equips_onto_it() {
    let mut g = main_phase();
    let champ = g.add_card_to_battlefield(0, catalog::fervent_champion());
    let hammer = g.add_card_to_battlefield(0, catalog::loxodon_warhammer());
    g.perform_action(GameAction::Equip { equipment: hammer, target: champ }).expect("equip {3} for {0}");
    assert_eq!(g.battlefield_find(hammer).unwrap().attached_to, Some(champ));
}

/// Spellbane Centaur — "Creatures you control can't be the targets of blue
/// spells or abilities from blue sources." It granted protection from blue,
/// which also stopped blue blockers and blue damage.
#[test]
fn spellbane_centaur_is_targeting_only() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::spellbane_centaur());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords().contains(&Keyword::HexproofFromColor(Color::Blue)));
    assert!(!cp.keywords().contains(&Keyword::Protection(Color::Blue)), "blue creatures still block it");
}

/// Trade Caravan — "Activate only during an opponent's upkeep." It was gated
/// to its controller's own upkeep.
#[test]
fn trade_caravan_untaps_on_an_opponents_upkeep() {
    use crabomination::card::CounterType;
    let mut g = main_phase();
    let caravan = g.add_card_to_battlefield(0, catalog::trade_caravan());
    g.battlefield_find_mut(caravan).unwrap().add_counters(CounterType::Currency, 4);
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(forest).unwrap().tapped = true;
    let untap = |g: &mut GameState| g.perform_action(GameAction::ActivateAbility {
        card_id: caravan, ability_index: 0, target: Some(Target::Permanent(forest)),
        additional_targets: vec![], x_value: None, mode: None,
    });
    g.step = TurnStep::Upkeep;
    assert!(untap(&mut g).is_err(), "not during your own upkeep");
    g.active_player_idx = 1;
    untap(&mut g).expect("an opponent's upkeep");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(forest).unwrap().tapped);
}

/// Intervening "if" clauses that shipped dropped (CR 603.4): Resolute
/// Archangel only raises a life total below the starting one (it lowered a
/// Commander player's 45 to 40), and Unstoppable Slasher returns only if it
/// had no counters — its stun-countered return used to come back every time.
#[test]
fn resolute_archangel_and_unstoppable_slasher_check_their_ifs() {
    use crabomination::effect::{Effect, Selector};
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    for (life, after) in [(25, 25), (7, 20)] {
        let mut g = main_phase();
        g.players[0].life = life;
        let angel = g.add_card_to_hand(0, catalog::resolute_archangel());
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(5);
        cast(&mut g, angel, None).expect("Archangel");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, after, "from {life}");
    }
    let mut g = main_phase();
    let slasher = g.add_card_to_battlefield(0, catalog::unstoppable_slasher());
    let kill = Effect::Destroy { what: Selector::EachPermanent(crabomination::card::SelectionRequirement::Creature) };
    for round in 0..2 {
        let evs = g.resolve_effect(&kill, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(slasher).is_some(), round == 0, "returns once, then stays dead");
    }
}

/// Fearless Swashbuckler — "Whenever you attack, if a Pirate and a Vehicle
/// attacked this combat, draw three cards, then discard two." It looted on
/// every attack.
#[test]
fn fearless_swashbuckler_needs_a_pirate_and_a_vehicle() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = main_phase();
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
    }
    let swash = g.add_card_to_battlefield(0, catalog::fearless_swashbuckler());
    g.clear_sickness(swash);
    g.step = TurnStep::DeclareAttackers;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: swash, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "a Pirate alone draws nothing");
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<Vec<GameEvent>, GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: index, target, additional_targets: vec![], x_value: None, mode: None,
    })
}

/// Heirloom Mirror shipped as an invented mana rock. Printed: {1}, {T}, pay 1
/// life, discard a card — draw, mill, a ritual counter; at three, remove them
/// and transform into Inherited Fiend.
#[test]
fn heirloom_mirror_transforms_on_its_third_ritual() {
    let mut g = main_phase();
    let mirror = g.add_card_to_battlefield(0, catalog::heirloom_mirror());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
    }
    for turn in 0..3 {
        g.add_card_to_hand(0, catalog::island());
        g.players[0].mana_pool.add_colorless(1);
        g.battlefield_find_mut(mirror).unwrap().tapped = false;
        activate(&mut g, mirror, 0, None).expect("activate the Mirror");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(mirror).unwrap().transformed, turn == 2);
    }
    assert_eq!(g.players[0].life, 17, "1 life a ritual");
    assert_eq!(g.players[0].graveyard.len(), 6, "three discards, three mills");
    let fiend = g.computed_permanent(mirror).unwrap();
    assert!(fiend.keywords().contains(&Keyword::Flying) && fiend.power == 4, "a 4/4 flying Inherited Fiend");
}

/// Rootcoil Creeper's two dropped abilities: two mana that only a spell cast
/// from your graveyard may spend, and exiling itself to return a card with
/// flashback you own from exile.
#[test]
fn rootcoil_creeper_graveyard_mana_and_flashback_return() {
    let mut g = main_phase();
    let rc = g.add_card_to_battlefield(0, catalog::rootcoil_creeper());
    g.clear_sickness(rc);
    activate(&mut g, rc, 1, None).expect("the graveyard mana");
    assert_eq!(g.players[0].mana_pool.restricted_total(), 2);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(cast(&mut g, bear, None).is_err(), "a hand-cast Bears can't spend it");

    let mut g = main_phase();
    let rc = g.add_card_to_battlefield(0, catalog::rootcoil_creeper());
    g.clear_sickness(rc);
    let tt = g.add_card_to_exile(0, catalog::think_twice());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    activate(&mut g, rc, 2, Some(Target::Permanent(tt))).expect("exile the Creeper");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == tt), "Think Twice returned to hand");
    assert!(g.battlefield_find(rc).is_none());
}

/// Intrepid Stablemaster's dropped mode: two mana that only a Mount or
/// Vehicle spell may spend.
#[test]
fn intrepid_stablemaster_funds_vehicles_only() {
    let mut g = main_phase();
    let sm = g.add_card_to_battlefield(0, catalog::intrepid_stablemaster());
    g.clear_sickness(sm);
    activate(&mut g, sm, 1, None).expect("the restricted mana");
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(cast(&mut g, bear, None).is_err(), "Bears is neither");
    let copter = g.add_card_to_hand(0, catalog::smugglers_copter());
    cast(&mut g, copter, None).expect("a Vehicle spell");
    drain_stack(&mut g);
    assert!(g.battlefield_find(copter).is_some());
}

/// CR 701.19c — "It can't be regenerated" / "they can't be regenerated" /
/// "can't be regenerated this turn": these shipped as a plain destroy or a
/// bare damage spell, so a regeneration shield saved the creature (oracle
/// scan, 2026-09-28). Rout, Winds of Rath, Vendetta, Seal of Doom and
/// Afterlife are pod cards.
#[test]
fn cr_701_19c_cant_be_regenerated_riders_beat_a_shield() {
    // (spell, targeted, X)
    type Case = (crate::Factory, bool, Option<u32>);
    let spells: [Case; 9] = [
        (catalog::afterlife, true, None),
        (catalog::befoul, true, None),
        (catalog::fissure, true, None),
        (catalog::vendetta, true, None),
        (catalog::rout, false, None),
        (catalog::winds_of_rath, false, None),
        (catalog::incinerate, true, None),
        (catalog::carbonize, true, None),
        (catalog::disintegrate, true, Some(3)),
    ];
    for (f, targeted, x_value) in spells {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().regeneration_shields = 1;
        let spell = g.add_card_to_hand(0, f());
        for c in Color::ALL {
            g.players[0].mana_pool.add(c, 3);
        }
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let name = f().name;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: targeted.then_some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value,
        })
        .expect(name);
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "{name}: the shield doesn't save it");
    }
    // Seal of Doom's sacrifice ability.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().regeneration_shields = 1;
    let seal = g.add_card_to_battlefield(0, catalog::seal_of_doom());
    g.perform_action(GameAction::ActivateAbility {
        card_id: seal,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("Seal of Doom");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "Seal of Doom: the shield doesn't save it");
}

/// Order of Midnight — "Flying. This creature can't block." The adventure's
/// creature face shipped with flying only, so it blocked.
#[test]
fn order_of_midnight_cant_block() {
    use crabomination::game::types::{Attack, AttackTarget, GameAction};
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    let order = g.add_card_to_battlefield(1, catalog::order_of_midnight());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    let blockers = g.legal_blockers(1);
    assert!(blockers.contains(&bears), "the Bears may block");
    assert!(!blockers.contains(&order), "Order of Midnight can't");
}

/// Angelic Intervention — "Target creature or planeswalker you control gains
/// protection … If it's a creature, put a +1/+1 counter on it." The
/// planeswalker target was refused.
#[test]
fn angelic_intervention_protects_a_planeswalker_without_a_counter() {
    use crabomination::card::CounterType;
    let mut g = main_phase();
    let walker = g.add_card_to_battlefield(0, catalog::chandra_torch_of_defiance());
    let ai = g.add_card_to_hand(0, catalog::angelic_intervention());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, ai, Some(Target::Permanent(walker))).expect("a planeswalker you control");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(walker).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Ceaseless Conflict — "a 3/2 Spirit for each NONTOKEN creature you
/// CONTROLLED that was destroyed this way". It counted destroyed creatures
/// you OWN, tokens included: a stolen creature gave nothing, your creature
/// under an opponent's control and your tokens gave Spirits.
#[test]
fn ceaseless_conflict_counts_nontoken_creatures_you_controlled() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let token = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(token).unwrap().is_token = true;
    for _ in 0..2 {
        let stolen = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(stolen).unwrap().controller = 0;
    }
    let lent = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(lent).unwrap().controller = 1;
    let cc = g.add_card_to_hand(0, catalog::ceaseless_conflict());
    g.players[0].mana_pool.add(Color::White, 5);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    crabomination::game::cast(&mut g, cc);
    let spirits = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Spirit").count();
    assert_eq!(spirits, 3, "your Bears and the two stolen; not the token, not the lent one");
}

/// CR 614.1c — "This artifact enters tapped" is a replacement, not an ETB
/// trigger: Worn Powerstone and Star Compass tapped themselves with a trigger,
/// leaving a window to tap them for mana first.
#[test]
fn worn_powerstone_enters_tapped_as_a_replacement() {
    let mut g = main_phase();
    let stone = g.add_card_to_hand(0, catalog::worn_powerstone());
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, stone, None).expect("Worn Powerstone");
    g.perform_action(GameAction::PassPriority).ok();
    g.perform_action(GameAction::PassPriority).ok();
    let c = g.battlefield_find(stone).expect("resolved");
    assert!(c.tapped, "tapped as it entered");
    assert!(g.stack.is_empty(), "no enters-tapped trigger to respond to");
}

/// CR 707.9b — Volrath, the Shapestealer's copy is "7/5 and it has this
/// ability": the copied Bears keep Volrath's P/T and its {1} copy ability.
#[test]
fn cr_707_9a_volrath_copy_is_seven_five_and_keeps_its_ability() {
    let mut g = main_phase();
    let volrath = g.add_card_to_battlefield(0, catalog::volrath_the_shapestealer());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(crabomination::card::CounterType::PlusOnePlusOne, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: volrath,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("{1}: become a copy");
    drain_stack(&mut g);
    let cp = g.computed_permanent(volrath).unwrap();
    assert_eq!(g.battlefield_find(volrath).unwrap().definition.name, "Grizzly Bears");
    assert_eq!((cp.power, cp.toughness), (7, 5));
    assert_eq!(g.battlefield_find(volrath).unwrap().definition.activated_abilities.len(), 1, "keeps {{1}}");
}

/// CR 707.9b — Mizzium Transreliquat's {1}{U}{R} copy keeps "this ability"
/// (its second one) and not its {3} one.
#[test]
fn cr_707_9a_mizzium_transreliquat_permanent_copy_keeps_only_this_ability() {
    let mut g = main_phase();
    let mizz = g.add_card_to_battlefield(0, catalog::mizzium_transreliquat());
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mizz,
        ability_index: 1,
        target: Some(Target::Permanent(ring)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("{1}{U}{R}: become a copy");
    drain_stack(&mut g);
    let def = &g.battlefield_find(mizz).unwrap().definition;
    assert_eq!(def.name, "Sol Ring");
    assert_eq!(def.activated_abilities.len(), 2, "Sol Ring's mana ability + this ability");
    assert_eq!(def.activated_abilities[1].mana_cost.cmc(), 3);
}

/// CR 707.9b — Lazav, Dimir Mastermind's copy keeps his name, legendary,
/// hexproof and the trigger, so he can copy again.
#[test]
fn cr_707_9b_lazav_dimir_mastermind_copy_keeps_name_hexproof_and_trigger() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    let lazav = g.add_card_to_battlefield(0, catalog::lazav_dimir_mastermind());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(bear))).expect("bolt");
    drain_stack(&mut g);
    let cp = g.computed_permanent(lazav).unwrap();
    assert_eq!(g.battlefield_find(lazav).unwrap().definition.name, "Lazav, Dimir Mastermind");
    assert_eq!((cp.power, cp.toughness), (2, 2), "the Bears' P/T");
    assert!(cp.keywords().contains(&Keyword::Hexproof));
    assert!(cp.supertypes().contains(&crabomination::card::Supertype::Legendary));
    assert_eq!(g.battlefield_find(lazav).unwrap().definition.triggered_abilities.len(), 1);
}

/// CR 707.9b — Saheeli, Sublime Artificer's −2 copy is "an artifact in
/// addition to its other types".
#[test]
fn cr_707_9b_saheeli_sublime_artificer_copy_stays_an_artifact() {
    let mut g = main_phase();
    let saheeli = g.add_card_to_battlefield(0, catalog::saheeli_sublime_artificer());
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: saheeli,
        ability_index: 0,
        target: Some(Target::Permanent(ring)),
        x_value: None,
    })
    .expect("-2");
    drain_stack(&mut g);
    let cp = g.computed_permanent(ring).unwrap();
    assert_eq!(g.battlefield_find(ring).unwrap().definition.name, "Grizzly Bears");
    assert!(cp.card_types().contains(&crabomination::card::CardType::Creature));
    assert!(cp.card_types().contains(&crabomination::card::CardType::Artifact));
}

/// Vile Redeemer — a Scion "for each nontoken creature that died under your
/// control this turn": neither your token nor an opponent's creature counts.
#[test]
fn vile_redeemer_counts_only_your_nontoken_deaths() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let tok = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(tok).unwrap().is_token = true;
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wrath = g.add_card_to_hand(0, catalog::wrath_of_god());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, wrath, None).expect("wrath");
    drain_stack(&mut g);
    assert!(g.battlefield.is_empty());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let vr = g.add_card_to_hand(0, catalog::vile_redeemer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, vr, None).expect("vile redeemer");
    drain_stack(&mut g);
    let scions = g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Scion").count();
    assert_eq!(scions, 1);
}

/// "Nontoken" filters that shipped dropped (nontoken oracle scan): Homicide
/// Investigator ignores a dying token but sees itself die; Hamlet Vanguard
/// doesn't count a Human token; Paradoxical Outcome leaves a token.
#[test]
fn nontoken_filters_skip_tokens() {
    let clues = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Clue").count();
    let mut g = main_phase();
    let inv = g.add_card_to_battlefield(0, catalog::homicide_investigator());
    let tok = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(tok).unwrap().is_token = true;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(tok))).expect("bolt the token");
    drain_stack(&mut g);
    assert_eq!(clues(&g), 0, "a token died");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(inv))).expect("bolt the Investigator");
    drain_stack(&mut g);
    assert_eq!(clues(&g), 1, "it sees itself die");

    let mut g = main_phase();
    let human = g.add_card_to_battlefield(0, catalog::hamlet_vanguard());
    g.battlefield_find_mut(human).unwrap().is_token = true;
    g.add_card_to_battlefield(0, catalog::hamlet_vanguard());
    let hv = g.add_card_to_hand(0, catalog::hamlet_vanguard());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, hv, None).expect("vanguard");
    drain_stack(&mut g);
    let n = g.battlefield_find(hv).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne);
    assert_eq!(n, 2, "one nontoken Human");

    let mut g = main_phase();
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    g.battlefield_find_mut(ring).unwrap().is_token = true;
    let signet = g.add_card_to_battlefield(0, catalog::arcane_signet());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let po = g.add_card_to_hand(0, catalog::paradoxical_outcome());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, po, None).expect("paradoxical outcome");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_some(), "the token stays");
    assert!(g.battlefield_find(signet).is_none(), "the Signet returns");
    assert_eq!(g.players[0].hand.len(), 2, "the Signet and one draw");
}

/// Croaking Counterpart — "target non-Frog creature … except it's a 1/1
/// green Frog": the copy is green and keeps a legend's supertype.
#[test]
fn croaking_counterpart_copy_is_green_and_stays_legendary() {
    let mut g = main_phase();
    let legend = g.add_card_to_battlefield(1, catalog::thalia_guardian_of_thraben());
    let cc = g.add_card_to_hand(0, catalog::croaking_counterpart());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, cc, Some(Target::Permanent(legend))).expect("croak");
    drain_stack(&mut g);
    let frog = g.battlefield.iter().find(|c| c.controller == 0 && c.is_token).map(|c| c.id).expect("a copy");
    let cp = g.computed_permanent(frog).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    assert!(cp.colors.contains(Color::Green) && !cp.colors.contains(Color::White));
    assert!(cp.supertypes().contains(&crabomination::card::Supertype::Legendary));
}

/// "Other … you control" statics that shipped dropped ("other" anthem scan):
/// Bellowing Tanglewurm's intimidate, Spider-Ham's Animal May-Ham, The
/// Seriema's 7+ indestructible for other tapped legends.
#[test]
fn other_creature_statics_apply_to_your_side_only() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::bellowing_tanglewurm());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(g.computed_permanent(mine).unwrap().keywords().contains(&Keyword::Intimidate));
    assert!(!g.computed_permanent(theirs).unwrap().keywords().contains(&Keyword::Intimidate));

    let mut g = main_phase();
    let ham = g.add_card_to_battlefield(0, catalog::spider_ham_peter_porker());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pt = |g: &GameState, id| {
        let c = g.computed_permanent(id).unwrap();
        (c.power, c.toughness)
    };
    assert_eq!(pt(&g, mine), (3, 3), "a Bear you control");
    assert_eq!(pt(&g, theirs), (2, 2));
    assert_eq!(pt(&g, ham), (2, 2), "not itself");

    let mut g = main_phase();
    let ship = g.add_card_to_battlefield(0, catalog::the_seriema());
    let thalia = g.add_card_to_battlefield(0, catalog::thalia_guardian_of_thraben());
    g.battlefield_find_mut(thalia).unwrap().tapped = true;
    let ind = |g: &GameState| g.computed_permanent(thalia).unwrap().keywords().contains(&Keyword::Indestructible);
    assert!(!ind(&g), "below 7 charge counters");
    g.battlefield_find_mut(ship).unwrap().add_counters(crabomination::card::CounterType::Charge, 7);
    assert!(ind(&g), "tapped legend at 7+");
    g.battlefield_find_mut(thalia).unwrap().tapped = false;
    assert!(!ind(&g), "untapped");
}

/// Trigger halves that shipped dropped (trigger-kind scan): Orcish
/// Bowmasters pings and amasses as it enters; Glint-Sleeve Siphoner gets {E}
/// as it enters; Dreamstealer's combat damage makes that player discard that
/// many cards.
#[test]
fn dropped_trigger_halves_fire() {
    let mut g = main_phase();
    let bow = g.add_card_to_hand(0, catalog::orcish_bowmasters());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, bow, None).expect("bowmasters");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Army"), "amassed on ETB");

    let mut g = main_phase();
    let gs = g.add_card_to_hand(0, catalog::glint_sleeve_siphoner());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, gs, None).expect("siphoner");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 1, "{{E}} on ETB");

    let mut g = main_phase();
    let ds = g.add_card_to_battlefield(0, catalog::dreamstealer());
    g.clear_sickness(ds);
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::island());
    }
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: ds,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 2, "1 combat damage, 1 discard");
}

/// Time Elemental — "When this creature attacks or blocks, at end of combat,
/// sacrifice it and it deals 5 damage to you": the block half was missing.
#[test]
fn time_elemental_dooms_itself_on_a_block() {
    use crabomination::game::types::{Attack, AttackTarget, GameAction};
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    let te = g.add_card_to_battlefield(1, catalog::time_elemental());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]))
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(te, attacker)])).expect("block");
    let life = g.players[1].life;
    while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(te).is_none(), "sacrificed at end of combat");
    assert_eq!(g.players[1].life, life - 5);
}

/// "Whenever enchanted creature deals damage to an opponent" is ANY damage —
/// Curiosity on a pinger is the classic draw engine — and only to an
/// opponent. Curiosity, Keen Sense and Snake Umbra (pod cards) listened to
/// combat damage to any player.
#[test]
fn damage_to_an_opponent_auras_draw_off_noncombat_damage() {
    use crabomination::effect::{Effect, PlayerRef, Selector, Value};
    use crabomination::game::effects::EffectContext;
    for aura in [catalog::curiosity, catalog::keen_sense, catalog::snake_umbra] {
        for (victim, draws) in [(1usize, 1usize), (0, 0)] {
            let mut g = two_player_game();
            g.add_card_to_library(0, catalog::island());
            let pinger = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            let a = g.add_card_to_battlefield(0, aura());
            g.battlefield_find_mut(a).unwrap().attached_to = Some(pinger);
            g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
                crabomination::decision::DecisionAnswer::Bool(true),
            ]));
            let hand = g.players[0].hand.len();
            let ctx = EffectContext::for_ability(pinger, 0, None);
            let to = if victim == 1 { PlayerRef::EachOpponent } else { PlayerRef::You };
            let evs = g
                .resolve_effect(&Effect::DealDamage { to: Selector::Player(to), amount: Value::Const(1) }, &ctx)
                .unwrap();
            g.dispatch_triggers_for_events(&evs);
            drain_stack(&mut g);
            assert_eq!(g.players[0].hand.len() - hand, draws, "{} pinging seat {victim}", aura().name);
        }
    }
}

/// Cube riders that shipped dropped. The Sun's Zenith cycle shuffles itself
/// back into its owner's library; Green Sun's Zenith is capped at X; Red
/// Sun's Zenith exiles a creature it would kill.
#[test]
fn suns_zenith_cycle_riders() {
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::llanowar_elves());
    g.add_card_to_library(0, catalog::craterhoof_behemoth());
    let gsz = g.add_card_to_hand(0, catalog::green_suns_zenith());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: gsz, target: None, additional_targets: vec![], mode: None, x_value: Some(1) })
        .expect("GSZ for 1");
    drain_stack(&mut g);
    let names: Vec<_> = g.battlefield.iter().map(|c| c.definition.name).collect();
    assert!(!names.contains(&"Craterhoof Behemoth"), "mana value 8 > X");
    assert!(g.players[0].graveyard.iter().all(|c| c.id != gsz), "not in the graveyard");
    assert!(g.players[0].library.iter().any(|c| c.id == gsz), "shuffled into the library");

    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let rsz = g.add_card_to_hand(0, catalog::red_suns_zenith());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: rsz,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("RSZ for 2");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().all(|c| c.id != bear), "exiled, not dead");
    assert!(g.exile.iter().any(|c| c.id == bear));
    assert!(g.players[0].library.iter().any(|c| c.id == rsz));
}

/// Leyline of the Void and Leyline of the Guildpact begin the game on the
/// battlefield from an opening hand (their docs said the clause was dropped).
#[test]
fn opening_hand_leylines_start_in_play() {
    for f in [catalog::leyline_of_the_void as fn() -> crabomination::card::CardDefinition, catalog::leyline_of_the_guildpact] {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, f());
        g.fire_start_of_game_effects();
        assert!(g.battlefield_find(id).is_some(), "{} starts in play", f().name);
    }
}

/// Holy Light hits nonwhite creatures only; Temporal Mastery exiles itself;
/// Legion Loyalist's battalion makes your team unblockable by tokens.
#[test]
fn holy_light_mastery_and_loyalist_riders() {
    let mut g = main_phase();
    let knight = g.add_card_to_battlefield(0, catalog::white_knight());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hl = g.add_card_to_hand(0, catalog::holy_light());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, hl, None).expect("holy light");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(knight).unwrap().power, 2, "white: untouched");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 1, "nonwhite: -1/-1");

    let mut g = main_phase();
    let tm = g.add_card_to_hand(0, catalog::temporal_mastery());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(5);
    cast(&mut g, tm, None).expect("temporal mastery");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == tm), "exiled as it resolves");

    let mut g = main_phase();
    let ll = g.add_card_to_battlefield(0, catalog::legion_loyalist());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [ll, a, b] {
        g.clear_sickness(id);
    }
    let tok = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(tok).unwrap().is_token = true;
    g.step = TurnStep::DeclareAttackers;
    let attacks = [ll, a, b]
        .map(|attacker| crabomination::game::types::Attack { attacker, target: crabomination::game::types::AttackTarget::Player(1) });
    g.perform_action(GameAction::DeclareAttackers(attacks.to_vec())).expect("battalion");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(tok, a)])).is_err(), "a token can't block them");
}

/// CR 115.4 — "any target" is a creature, player, planeswalker or battle. ~70
/// damage effects were written with the match-everything filter, so a land
/// or an artifact was a legal target; Arc Trail's "another target" could
/// also name the first one twice.
#[test]
fn cr_115_4_any_target_is_not_any_permanent() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let fa = g.add_card_to_hand(0, catalog::fire_ambush());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    assert!(cast(&mut g, fa, Some(Target::Permanent(land))).is_err(), "a land isn't a target for damage");
    let at = g.add_card_to_hand(0, catalog::arc_trail());
    let twice = g.perform_action(GameAction::CastSpell {
        card_id: at,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: None,
    });
    assert!(twice.is_err(), "another target");
    g.perform_action(GameAction::CastSpell {
        card_id: at,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Player(1)],
        mode: None,
        x_value: None,
    })
    .expect("a creature and a player");
}

// ── "Whenever you discard / sacrifice one or more …" (CR 603.2c) ───────────

/// Resolve `effect` as seat 0's ability of `src`, then dispatch its events —
/// one batch.
fn resolve_batch(g: &mut GameState, src: CardId, effect: crabomination::effect::Effect) {
    let ctx = crabomination::game::effects::EffectContext::for_ability(src, 0, None);
    let events = g.resolve_effect(&effect, &ctx).expect("resolves");
    g.dispatch_triggers_for_events(&events);
    drain_stack(g);
}

fn discard_two() -> crabomination::effect::Effect {
    crabomination::effect::Effect::Discard {
        who: crabomination::effect::Selector::You,
        amount: crabomination::effect::Value::Const(2),
        random: false,
    }
}

/// Inti, Seneschal of the Sun: a two-card discard exiles ONE library card.
#[test]
fn cr_603_2c_inti_exiles_one_card_for_a_two_card_discard() {
    let mut g = main_phase();
    let inti = g.add_card_to_battlefield(0, catalog::inti_seneschal_of_the_sun());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let exiled = g.exile.len();
    resolve_batch(&mut g, inti, discard_two());
    assert_eq!(g.players[0].graveyard.len(), 2, "both discarded");
    assert_eq!(g.exile.len(), exiled + 1, "one trigger for the batch");
}

/// Captain Howler, Sea Scourge: one trigger, +2/+0 for each card discarded.
#[test]
fn cr_603_2c_captain_howler_pumps_by_the_batch_once() {
    let mut g = main_phase();
    let howler = g.add_card_to_battlefield(0, catalog::captain_howler_sea_scourge());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    resolve_batch(&mut g, howler, discard_two());
    let power: i32 = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .map(|c| g.computed_permanent(c.id).unwrap().power - c.definition.power)
        .sum();
    assert_eq!(power, 4, "+2/+0 per card, once");
}

/// Conspiracy Theorist: a two-card nonland discard is ONE "exile one of them"
/// trigger.
#[test]
fn cr_603_2c_conspiracy_theorist_exiles_one_of_them() {
    let mut g = main_phase();
    let ct = g.add_card_to_battlefield(0, catalog::conspiracy_theorist());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_ability(ct, 0, None);
    let events = g.resolve_effect(&discard_two(), &ctx).expect("resolves");
    g.dispatch_triggers_for_events(&events);
    assert_eq!(g.players[0].graveyard.len(), 2, "both discarded");
    assert_eq!(g.stack.len(), 1, "one trigger for the batch, not one per card");
}

/// Camellia, the Seedmiser: two Foods sacrificed at once make one Squirrel.
#[test]
fn cr_603_2c_camellia_makes_one_squirrel_per_food_batch() {
    let mut g = main_phase();
    let cam = g.add_card_to_battlefield(0, catalog::camellia_the_seedmiser());
    let food = crabomination::game::effects::food_token();
    g.add_token_to_battlefield(0, &food);
    g.add_token_to_battlefield(0, &food);
    resolve_batch(
        &mut g,
        cam,
        crabomination::effect::Effect::SacrificeAllMatching {
            who: crabomination::effect::Selector::You,
            filter: crabomination::card::SelectionRequirement::HasArtifactSubtype(
                crabomination::card::ArtifactSubtype::Food,
            ),
        },
    );
    let squirrels = g.battlefield.iter().filter(|c| c.definition.name == "Squirrel").count();
    assert_eq!(squirrels, 1);
}

/// Power thresholds that shipped dropped (P/T-threshold scan): Zurgo
/// Bellstriker can't block a creature with power 2 or greater; O-Naginata
/// attaches only to a creature with power 3 or greater (CR 301.5c).
#[test]
fn zurgo_and_o_naginata_power_thresholds() {
    use crabomination::game::types::{Attack, AttackTarget, GameAction};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let zurgo = g.add_card_to_battlefield(1, catalog::zurgo_bellstriker());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(zurgo, bear)])).is_err(), "power 2 attacker");

    let mut g = main_phase();
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let bear = g.add_card_to_battlefield(0, catalog::serra_angel());
    let nag = g.add_card_to_battlefield(0, catalog::o_naginata());
    g.players[0].mana_pool.add_colorless(4);
    assert!(g.perform_action(GameAction::Equip { equipment: nag, target: elf }).is_err(), "power 1");
    g.perform_action(GameAction::Equip { equipment: nag, target: bear }).expect("power 4");
}

/// Self-restriction scan: Brazen Borrower "can block only creatures with
/// flying" (dropped), Welkin Tern the same (shipped as "can't block"); and
/// Boseiju, Who Shelters All enters tapped as a replacement (CR 614.1c), not
/// by an ETB trigger that left a window to tap it first.
#[test]
fn block_only_flyers_and_boseiju_enters_tapped() {
    use crabomination::game::types::{Attack, AttackTarget, GameAction};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    for id in [bear, angel] {
        g.clear_sickness(id);
    }
    let bb = g.add_card_to_battlefield(1, catalog::brazen_borrower());
    let tern = g.add_card_to_battlefield(1, catalog::welkin_tern());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: bear, target: AttackTarget::Player(1) },
        Attack { attacker: angel, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(bb, bear)])).is_err(), "a ground creature");
    g.perform_action(GameAction::DeclareBlockers(vec![(bb, angel), (tern, angel)])).expect("both block the flyer");

    let mut g = main_phase();
    let bos = g.add_card_to_hand(0, catalog::boseiju_who_shelters_all());
    g.perform_action(GameAction::PlayLand(bos)).expect("land drop");
    assert!(g.battlefield_find(bos).unwrap().tapped, "tapped as it entered");
    assert!(g.stack.is_empty(), "no enters-tapped trigger");
}

/// Search filters read against the oracle (`scripts/audit_search_filters.py`):
/// Farseek takes a nonbasic Swamp–Mountain and never a Forest; Gift of Estates
/// any Plains card; Gatecreeper Vine a Gate; Shefet Monitor a Desert; Invasion
/// of Theros an Aura. The library holds only the bait and the printed type.
#[test]
fn search_filters_match_the_printed_card_types() {
    // Farseek: Blood Crypt fetched, the Forest left behind.
    let mut g = main_phase();
    let forest = g.add_card_to_library(0, catalog::forest());
    let crypt = g.add_card_to_library(0, catalog::blood_crypt());
    let id = g.add_card_to_hand(0, catalog::farseek());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, id, None).expect("Farseek");
    drain_stack(&mut g);
    assert!(g.battlefield_find(crypt).is_some(), "a nonbasic Swamp Mountain");
    assert!(g.players[0].library.iter().any(|c| c.id == forest), "never a Forest");

    // Gift of Estates: a nonbasic Plains (Hallowed Fountain) is a Plains card.
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::forest());
    let fountain = g.add_card_to_library(0, catalog::hallowed_fountain());
    let id = g.add_card_to_hand(0, catalog::gift_of_estates());
    g.players[0].mana_pool.add(Color::White, 2);
    cast(&mut g, id, None).expect("Gift of Estates");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == fountain), "a nonbasic Plains");

    // Gatecreeper Vine: a Gate card.
    let mut g = main_phase();
    let gate = g.add_card_to_library(0, catalog::azorius_guildgate());
    let id = g.add_card_to_hand(0, catalog::gatecreeper_vine());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, id, None).expect("Gatecreeper Vine");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == gate), "a Gate card");

    // Shefet Monitor: cycling fetches a Desert.
    let mut g = main_phase();
    // Cycling draws first: an Island on either end of the library.
    g.add_card_to_library(0, catalog::island());
    let desert = g.add_card_to_library(0, catalog::desert_of_the_fervent());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::shefet_monitor());
    g.players[0].mana_pool.add(Color::Green, 4);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
        crabomination::decision::DecisionAnswer::Search(Some(desert)),
    ]));
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert!(g.battlefield_find(desert).is_some(), "a Desert");

    // Invasion of Theros: an Aura card.
    let mut g = main_phase();
    let aura = g.add_card_to_library(0, catalog::pacifism());
    let id = g.add_card_to_hand(0, catalog::invasion_of_theros());
    g.players[0].mana_pool.add(Color::White, 3);
    cast(&mut g, id, None).expect("Invasion of Theros");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == aura), "an Aura card");
}

/// "Target artifact, creature, or enchantment" is not "nonland permanent":
/// Divine Gambit can't exile a planeswalker, Trickster Mage's "artifact,
/// creature, or land" can't tap one either.
#[test]
fn artifact_creature_or_enchantment_is_not_any_nonland_permanent() {
    let mut g = main_phase();
    let jace = g.add_card_to_battlefield(1, catalog::jace_beleren());
    let id = g.add_card_to_hand(0, catalog::divine_gambit());
    g.players[0].mana_pool.add(Color::White, 2);
    assert!(cast(&mut g, id, Some(Target::Permanent(jace))).is_err(), "a planeswalker");
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    cast(&mut g, id, Some(Target::Permanent(bear))).expect("a creature");

    let mut g = main_phase();
    let jace = g.add_card_to_battlefield(1, catalog::jace_beleren());
    let mage = g.add_card_to_battlefield(0, catalog::trickster_mage());
    g.clear_sickness(mage);
    g.add_card_to_hand(0, catalog::island());
    g.players[0].mana_pool.add(Color::Blue, 1);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: mage, ability_index: 0, target: Some(Target::Permanent(jace)),
        additional_targets: vec![], x_value: None, mode: None,
    }).is_err(), "Trickster Mage can't tap a planeswalker");
}

/// Printed target type lists the definitions had cut short: Get Lost's
/// enchantment, Strangle's planeswalker, Fade into Antiquity's artifact.
#[test]
fn target_type_lists_carry_every_printed_type() {
    let mut g = main_phase();
    let aura = g.add_card_to_battlefield(1, catalog::pacifism());
    let id = g.add_card_to_hand(0, catalog::get_lost());
    g.players[0].mana_pool.add(Color::White, 2);
    cast(&mut g, id, Some(Target::Permanent(aura))).expect("Get Lost on an enchantment");

    let mut g = main_phase();
    let jace = g.add_card_to_battlefield(1, catalog::jace_beleren());
    let id = g.add_card_to_hand(0, catalog::strangle());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, id, Some(Target::Permanent(jace))).expect("Strangle on a planeswalker");

    let mut g = main_phase();
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::fade_into_antiquity());
    g.players[0].mana_pool.add(Color::Green, 3);
    cast(&mut g, id, Some(Target::Permanent(ring))).expect("Fade into Antiquity on an artifact");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_none(), "exiled");
}

/// Negated target qualifiers that shipped dropped (target-negation scan):
/// Shriekmaw's "nonartifact", Crush's "noncreature", Izzet Charm's
/// "noncreature spell".
#[test]
fn target_negations_are_enforced() {
    let mut g = main_phase();
    let thopter = g.add_card_to_battlefield(1, catalog::ornithopter());
    let crush = g.add_card_to_hand(0, catalog::crush());
    g.players[0].mana_pool.add(Color::Red, 1);
    assert!(cast(&mut g, crush, Some(Target::Permanent(thopter))).is_err(), "an artifact creature");
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    cast(&mut g, crush, Some(Target::Permanent(ring))).expect("a noncreature artifact");
    drain_stack(&mut g);

    // Shriekmaw's ETB has no legal target when the only other creature is an
    // artifact one (it is black itself).
    let maw = g.add_card_to_hand(0, catalog::shriekmaw());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, maw, None).expect("shriekmaw");
    drain_stack(&mut g);
    assert!(g.battlefield_find(thopter).is_some(), "Shriekmaw can't destroy an artifact creature");

    let mut g = main_phase();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell { card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("bears");
    g.priority.player_with_priority = 0;
    let charm = g.add_card_to_hand(0, catalog::izzet_charm());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let r = g.perform_action(GameAction::CastSpell {
        card_id: charm,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    });
    assert!(r.is_err(), "Izzet Charm can't counter a creature spell");
}

/// "Whenever you attack" is one trigger per attack declaration, whoever
/// attacks: Adeline listened only to her own attack (a cube card), Living
/// History fired once per attacker and pumped each.
#[test]
fn whenever_you_attack_fires_once_per_declaration() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = main_phase();
    let adeline = g.add_card_to_battlefield(0, catalog::adeline_resplendent_cathar());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let _ = adeline;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack without Adeline");
    drain_stack(&mut g);
    let humans = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Human").count();
    assert_eq!(humans, 1, "one Human for the one opponent");

    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::living_history());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [a, b] {
        g.clear_sickness(id);
    }
    g.players[0].cards_left_graveyard_this_turn = 1;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    let power: i32 = [a, b].iter().map(|&id| g.computed_permanent(id).unwrap().power).sum();
    assert_eq!(power, 2 + 2 + 2, "one +2/+0, not one per attacker");
}

/// "Whenever you attack with three or more creatures" doesn't need the card
/// itself among them (it was written as battalion): Karlov Watchdog stays
/// home and still pumps the team.
#[test]
fn attack_with_three_or_more_is_not_battalion() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = main_phase();
    let dog = g.add_card_to_battlefield(0, catalog::karlov_watchdog());
    let ids: Vec<_> = (0..3).map(|_| g.add_card_to_battlefield(0, catalog::grizzly_bears())).collect();
    for &id in &ids {
        g.clear_sickness(id);
    }
    g.step = TurnStep::DeclareAttackers;
    let attacks = ids.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(1) }).collect();
    g.perform_action(GameAction::DeclareAttackers(attacks)).expect("three attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(ids[0]).unwrap().power, 3, "+1/+1");
    assert_eq!(g.computed_permanent(dog).unwrap().power, 4, "the Watchdog too, at home");
}

/// CR 115.4 — "any target" includes a battle. `SelectionRequirement::any_target`
/// replaced ~290 hand-written creature/player/planeswalker filters that left
/// battles out; Lightning Bolt now takes three defense counters off a Siege.
#[test]
fn cr_115_4_any_target_includes_a_battle() {
    use crabomination::card::CounterType;
    let mut g = main_phase();
    let siege = g.add_card_to_battlefield(1, catalog::invasion_of_theros());
    let before = g.battlefield_find(siege).unwrap().counter_count(CounterType::Defense);
    assert!(before >= 4, "a Siege enters with its defense");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(siege))).expect("a battle is any target");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(siege).unwrap().counter_count(CounterType::Defense), before - 3);
}

/// CR 115.4 — the other spelling of the same class: ~80 "any target"
/// damage effects named a BARE slot (`Target(0)` / `target()`), which no
/// filter bounds at all, so Lightning Bolt could target a Forest or a Sol
/// Ring. Each now names `target_any()`; a land is refused, a creature and a
/// player are not.
#[test]
fn cr_115_4_bare_any_target_slots_refuse_a_land() {
    type Factory = fn() -> crabomination::card::CardDefinition;
    let spells: [Factory; 8] = [
        catalog::lightning_bolt,
        catalog::shock,
        catalog::lightning_helix,
        catalog::hornet_sting,
        catalog::volcanic_hammer,
        catalog::searing_spear,
        catalog::incinerate,
        catalog::wild_slash,
    ];
    for spell in spells {
        let name = spell().name;
        let mut g = main_phase();
        let land = g.add_card_to_battlefield(1, catalog::forest());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        for (target, legal) in [(Target::Permanent(land), false), (Target::Permanent(bear), true), (Target::Player(1), true)] {
            let id = g.add_card_to_hand(0, spell());
            for c in [Color::Red, Color::White, Color::Green] {
                g.players[0].mana_pool.add(c, 3);
            }
            g.players[0].mana_pool.add_colorless(6);
            g.priority.player_with_priority = 0;
            assert_eq!(cast(&mut g, id, Some(target.clone())).is_ok(), legal, "{name} at {target:?}");
            drain_stack(&mut g);
        }
    }
}

/// CR 115.1 — the rest of the bare-slot class: each printed target now
/// bounds its slot. Flare of Faith and Cytoshape name a creature (a land is
/// refused), Searing Flesh an opponent or planeswalker (a creature is
/// refused), and Cannibalize's two creatures share a controller.
#[test]
fn cr_115_1_bare_target_slots_name_their_filter() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let try_cast = |g: &mut GameState, f: fn() -> crabomination::card::CardDefinition, t: Target, extra: Vec<Target>| {
        let id = g.add_card_to_hand(0, f());
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 3);
        }
        g.players[0].mana_pool.add_colorless(8);
        g.priority.player_with_priority = 0;
        let ok = g
            .perform_action(GameAction::CastSpell { card_id: id, target: Some(t), additional_targets: extra, mode: None, x_value: None })
            .is_ok();
        drain_stack(g);
        ok
    };
    assert!(!try_cast(&mut g, catalog::flare_of_faith, Target::Permanent(land), vec![]));
    assert!(!try_cast(&mut g, catalog::cytoshape, Target::Permanent(land), vec![Target::Permanent(bear)]));
    assert!(!try_cast(&mut g, catalog::searing_flesh, Target::Permanent(bear), vec![]));
    assert!(!try_cast(&mut g, catalog::cannibalize, Target::Permanent(bear), vec![Target::Permanent(mine)]), "different controllers");
    assert!(try_cast(&mut g, catalog::searing_flesh, Target::Player(1), vec![]));
}

/// CR 115.1 — an activation's printed target bounds its slot: Soul Sculptor
/// ("target creature") and Militant Monk ("any target") refuse a land.
/// Soul Sculptor's filter existed and the per-slot walker never read it.
#[test]
fn cr_115_1_activated_slots_refuse_a_land() {
    for f in [catalog::soul_sculptor as fn() -> crabomination::card::CardDefinition, catalog::militant_monk] {
        let mut g = main_phase();
        let land = g.add_card_to_battlefield(1, catalog::forest());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let src = g.add_card_to_battlefield(0, f());
        g.clear_sickness(src);
        for c in [Color::White, Color::Blue, Color::Green] {
            g.players[0].mana_pool.add(c, 3);
        }
        g.players[0].mana_pool.add_colorless(3);
        let act = |t: Target| GameAction::ActivateAbility {
            card_id: src,
            ability_index: 0,
            target: Some(t),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        };
        assert!(g.perform_action(act(Target::Permanent(land))).is_err(), "{}: a land", f().name);
        assert!(g.perform_action(act(Target::Permanent(bear))).is_ok(), "{}: a creature", f().name);
    }
}
