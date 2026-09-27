//! Pod-deck cards whose printed rider shipped dropped: each test asserts the
//! rider, not the card's already-covered body.

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
