//! Commander: the Fae Dominion precon (WOC, Tegwyll, `decks::cmdr_fae`) and
//! the primitives it needed.

use crabomination::card::{AdditionalCastCost, CardDefinition, CardId, CardType, CounterType, CreatureType, Keyword, SelectionRequirement as R};
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::mana::Color;
use crabomination::catalog;
use crabomination::effect::{Duration, Effect, Selector};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn run(g: &mut GameState, seat: usize, e: &Effect) {
    let ctx = EffectContext::for_spell(seat, None, 0, 0);
    g.resolve_effect(e, &ctx).expect("resolve");
    drain_stack(g);
}

/// CR 701.38 — goad lasts until the goader's next turn; "goaded for the rest
/// of the game" survives it, while a plain goad beside it lapses.
#[test]
fn cr_701_38_a_goad_for_the_game_outlives_the_goaders_turn() {
    let mut g = pod(3);
    let forever = g.add_card_to_battlefield(1, catalog::serra_angel());
    let plain = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    run(&mut g, 0, &Effect::GoadForTheGame { what: Selector::EachPermanent(R::HasName("Serra Angel".into())) });
    run(&mut g, 0, &Effect::Goad { what: Selector::EachPermanent(R::HasName("Grizzly Bears".into())) });
    g.active_player_idx = 0;
    g.do_untap();
    assert!(g.battlefield_find(forever).unwrap().goaded_by.contains(&0), "still goaded");
    assert!(g.battlefield_find(plain).unwrap().goaded_by.is_empty(), "a plain goad lapsed");
}

/// CR 508.1a — "they can't attack you or planeswalkers you control": the
/// named seat is refused as a defender, another opponent isn't.
#[test]
fn cr_508_1a_a_creature_that_cant_attack_you_attacks_someone_else() {
    let mut g = pod(3);
    g.active_player_idx = 1;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    run(
        &mut g,
        0,
        &Effect::GrantCantAttackYou {
            what: Selector::EachPermanent(R::HasName("Grizzly Bears".into())),
            duration: Duration::EndOfTurn,
        },
    );
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    let at = |p| GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(p) }]);
    assert!(g.perform_action(at(0)).is_err(), "can't attack the granting player");
    g.perform_action(at(2)).expect("another opponent is fine");
}

/// CR 601.2b — "you may cast this spell as though it had flash by tapping
/// three untapped creatures you control with flying": off-turn it needs the
/// three fliers and taps them; in your main phase it costs nothing extra.
#[test]
fn cr_601_2b_flash_by_tapping_fliers() {
    let spell = || CardDefinition {
        name: "Test Scouring",
        card_types: vec![CardType::Sorcery],
        flash_additional_cost: Some(AdditionalCastCost::TapPermanents {
            filter: R::Creature.and(R::HasKeyword(Keyword::Flying)),
            count: 3,
        }),
        effect: Effect::Noop,
        ..Default::default()
    };
    let cast = |g: &mut GameState, id| {
        g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
    };
    let mut g = pod(2);
    g.active_player_idx = 1;
    g.step = TurnStep::End;
    let fliers: Vec<_> = (0..2).map(|_| g.add_card_to_battlefield(0, catalog::serra_angel())).collect();
    let s = g.add_card_to_hand(0, spell());
    assert!(cast(&mut g, s).is_err(), "two fliers can't pay");
    let third = g.add_card_to_battlefield(0, catalog::serra_angel());
    cast(&mut g, s).expect("three fliers pay");
    for id in fliers.iter().chain([&third]) {
        assert!(g.battlefield_find(*id).unwrap().tapped);
    }

    let mut g = pod(2);
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    let s = g.add_card_to_hand(0, spell());
    cast(&mut g, s).expect("sorcery timing needs nothing extra");
    assert!(!g.battlefield_find(angel).unwrap().tapped);
}

// ── The cards ──

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::island());
    }
}

/// Attack seat 1 with `attackers`, no blocks, through combat damage.
fn connect(g: &mut GameState, attackers: &[CardId]) {
    for &a in attackers {
        g.clear_sickness(a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

/// Cast a spell as seat 0 during seat 1's turn.
fn cast_on_their_turn(g: &mut GameState, id: CardId) {
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    cast(g, 0, id, None).expect("instant-speed cast");
}

fn cheap_instant(g: &mut GameState) -> CardId {
    g.add_card_to_hand(0, catalog::opt())
}

/// Other Faeries get +1/+1; another Faerie dying draws and costs a life.
#[test]
fn tegwyll_leads_and_mourns_faeries() {
    let mut g = pod(2);
    let teg = g.add_card_to_battlefield(0, catalog::tegwyll_duke_of_splendor());
    let fae = g.add_card_to_battlefield(0, catalog::nettling_nuisance());
    assert_eq!(g.computed_permanent(fae).unwrap().power, 4);
    assert_eq!(g.computed_permanent(teg).unwrap().power, 2, "other");
    library(&mut g, 0, 2);
    let (hand, life) = (g.players[0].hand.len(), g.players[0].life);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(fae))).expect("bolt");
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.players[0].life, life - 1);
}

/// First spell on an opponent's turn: a Faerie Rogue (not the second);
/// Faeries connecting goad one of that player's creatures.
#[test]
fn alela_makes_rogues_and_goads() {
    let mut g = pod(2);
    let alela = g.add_card_to_battlefield(0, catalog::alela_cunning_conqueror());
    library(&mut g, 0, 4);
    let a = cheap_instant(&mut g);
    let b = cheap_instant(&mut g);
    cast_on_their_turn(&mut g, a);
    cast_on_their_turn(&mut g, b);
    assert_eq!(named(&g, 0, "Faerie Rogue").len(), 1);
    g.active_player_idx = 0;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    connect(&mut g, &[alela]);
    assert!(g.battlefield_find(bear).unwrap().goaded_by.contains(&0));
}

/// A Faerie permanent spell is copied into a token.
#[test]
fn archmage_of_echoes_copies_faerie_spells() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::archmage_of_echoes());
    let f = g.add_card_to_hand(0, catalog::faerie_bladecrafter());
    cast(&mut g, 0, f, None).expect("cast");
    let copies = named(&g, 0, "Faerie Bladecrafter");
    assert_eq!(copies.len(), 2);
    assert!(copies.iter().any(|&id| g.battlefield_find(id).unwrap().is_token));
}

/// The first spell on an opponent's turn steals their top card to play.
#[test]
fn blightwing_bandit_steals_the_top_card() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::blightwing_bandit());
    library(&mut g, 0, 2);
    g.add_card_to_library(1, catalog::grizzly_bears());
    let a = cheap_instant(&mut g);
    cast_on_their_turn(&mut g, a);
    assert!(g.exile.iter().any(|c| c.definition.name == "Grizzly Bears" && c.owner == 1));
}

/// Faeries connecting grow it; dying drains each opponent for its power.
#[test]
fn faerie_bladecrafter_grows_and_drains() {
    let mut g = pod(3);
    let blade = g.add_card_to_battlefield(0, catalog::faerie_bladecrafter());
    connect(&mut g, &[blade]);
    assert_eq!(g.battlefield_find(blade).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let (me, you, them) = (g.players[0].life, g.players[1].life, g.players[2].life);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    cast(&mut g, 1, bolt, Some(Target::Permanent(blade))).expect("bolt");
    assert_eq!((g.players[1].life, g.players[2].life), (you - 3, them - 3), "each opponent, its power 3");
    assert_eq!(g.players[0].life, me + 3, "you gain X once");
}

/// {3}{U}: a Faerie and a card.
#[test]
fn faerie_formation_builds_the_flock() {
    let mut g = pod(2);
    let f = g.add_card_to_battlefield(0, catalog::faerie_formation());
    library(&mut g, 0, 1);
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: f,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Faerie").len(), 1);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Blue and black lords stack on a blue-black creature; the Liege is "other".
#[test]
fn glen_elendra_liege_pumps_by_color() {
    let mut g = pod(2);
    let liege = g.add_card_to_battlefield(0, catalog::glen_elendra_liege());
    let alela = g.add_card_to_battlefield(0, catalog::alela_cunning_conqueror());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(alela).unwrap().power, 4, "blue and black");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "green");
    assert_eq!(g.computed_permanent(liege).unwrap().power, 2);
}

/// Paying X = its mana value casts a graveyard instant free, then exiles it.
#[test]
fn halo_forager_recasts_from_a_graveyard() {
    let mut g = pod(2);
    let bolt = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Amount(1), DecisionAnswer::Bool(true)]));
    let life = g.players[1].life;
    let forager = g.add_card_to_hand(0, catalog::halo_forager());
    cast(&mut g, 0, forager, None).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == bolt), "exiled after");
    assert_eq!(g.players[1].life, life - 3, "the Bolt went at the opponent");
}

/// In the declare blockers step on an opponent's turn: the attackers untap,
/// leave combat, and can't attack the caster in the extra combat.
#[test]
fn illusionists_gambit_turns_the_attack() {
    let mut g = pod(3);
    g.active_player_idx = 1;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let gambit = g.add_card_to_hand(0, catalog::illusionists_gambit());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::PreCombatMain;
    assert!(cast(&mut g, 0, gambit, None).is_err(), "only in the declare blockers step");
    g.step = TurnStep::DeclareBlockers;
    cast(&mut g, 0, gambit, None).expect("cast");
    let b = g.battlefield_find(bear).unwrap();
    assert!(!b.tapped, "untapped");
    assert!(!g.attacking_ids().contains(&bear), "removed from combat");
    let kws = g.computed_permanent(bear).unwrap().keywords().to_vec();
    assert!(kws.contains(&Keyword::CantAttackPlayer(0)) && kws.contains(&Keyword::MustAttack));
}

/// Enters as a copy of an opponent's creature, a Faerie Shapeshifter flier.
#[test]
fn malleable_impostor_copies_and_flies() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let imp = g.add_card_to_hand(0, catalog::malleable_impostor());
    cast(&mut g, 0, imp, None).expect("cast");
    let copy = named(&g, 0, "Grizzly Bears");
    assert_eq!(copy.len(), 1);
    let cp = g.computed_permanent(copy[0]).unwrap();
    assert!(cp.keywords().contains(&Keyword::Flying));
    assert!(g.battlefield_find(copy[0]).unwrap().definition.subtypes.creature_types.contains(&CreatureType::Faerie));
}

/// Faeries connecting hand that player a goaded 4/2 Pirate that can't block.
#[test]
fn nettling_nuisance_saddles_them_with_a_pirate() {
    let mut g = pod(3);
    let n = g.add_card_to_battlefield(0, catalog::nettling_nuisance());
    connect(&mut g, &[n]);
    let p = named(&g, 1, "Pirate");
    assert_eq!(p.len(), 1);
    let pirate = g.battlefield_find(p[0]).unwrap();
    assert!(pirate.goaded_by.contains(&0) && pirate.goad_for_the_game);
}

/// First spell on an opponent's turn: look at two, keep one.
#[test]
fn nymris_filters_two() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::nymris_oonas_trickster());
    library(&mut g, 0, 4);
    let a = cheap_instant(&mut g);
    cast_on_their_turn(&mut g, a);
    // Opt scries 1 and draws 1; Nymris puts one of two in hand, one in the yard.
    assert_eq!(g.players[0].hand.len(), 2);
    assert_eq!(g.players[0].graveyard.len(), 2, "Opt and the other card");
}

/// Entering: an opponent's creature card fights for you, hasty, until the
/// end step.
#[test]
fn puppeteer_clique_borrows_the_dead() {
    let mut g = pod(2);
    let dead = g.add_card_to_graveyard(1, catalog::serra_angel());
    let clique = g.add_card_to_hand(0, catalog::puppeteer_clique());
    cast(&mut g, 0, clique, Some(Target::Permanent(dead))).expect("cast");
    let angel = named(&g, 0, "Serra Angel");
    assert_eq!(angel.len(), 1);
    assert!(g.computed_permanent(angel[0]).unwrap().keywords().contains(&Keyword::Haste));
}

/// Connecting runs all three modes: discard, lose-and-draw, sacrifice.
#[test]
fn rankle_pranks_everyone() {
    let mut g = pod(2);
    let rankle = g.add_card_to_battlefield(0, catalog::rankle_master_of_pranks());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::island());
    library(&mut g, 0, 2);
    library(&mut g, 1, 2);
    let life = g.players[1].life;
    connect(&mut g, &[rankle]);
    assert!(g.battlefield_find(bear).is_none(), "sacrificed");
    assert_eq!(g.players[1].life, life - 3 - 1);
    assert!(g.battlefield_find(rankle).is_none(), "Rankle was your only creature to sacrifice");
}

/// Two Faerie Rogues on entry; an attacking flier may become a 4/4 Dragon.
#[test]
fn shadow_puppeteers_dress_up_fliers() {
    let mut g = pod(2);
    let p = g.add_card_to_hand(0, catalog::shadow_puppeteers());
    cast(&mut g, 0, p, None).expect("cast");
    let rogues = named(&g, 0, "Faerie Rogue");
    assert_eq!(rogues.len(), 2);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.clear_sickness(rogues[0]);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: rogues[0], target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let cp = g.computed_permanent(rogues[0]).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
}

/// A wrath that leaves three Faerie Rogues; flash by tapping three fliers.
#[test]
fn tegwylls_scouring_wipes_and_rebuilds() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let fliers: Vec<_> = (0..3).map(|_| g.add_card_to_battlefield(0, catalog::serra_angel())).collect();
    let s = g.add_card_to_hand(0, catalog::tegwylls_scouring());
    cast_on_their_turn(&mut g, s);
    for f in fliers {
        assert!(g.battlefield_find(f).is_none(), "destroyed");
    }
    assert_eq!(named(&g, 0, "Faerie Rogue").len(), 3);
    assert!(named(&g, 1, "Grizzly Bears").is_empty());
}

/// Every creature card that died this turn, anyone's, returns for you; an
/// older one stays.
#[test]
fn thrilling_encore_takes_the_fallen() {
    let mut g = pod(3);
    let old = g.add_card_to_graveyard(2, catalog::serra_angel());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Permanent(bear))).expect("bolt");
    let enc = g.add_card_to_hand(0, catalog::thrilling_encore());
    cast(&mut g, 0, enc, None).expect("cast");
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 1);
    assert!(g.players[2].graveyard.iter().any(|c| c.id == old));
}
