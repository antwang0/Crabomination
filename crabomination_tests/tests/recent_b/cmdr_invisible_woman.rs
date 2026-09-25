//! Commander: The Fantastic Four precon (MSC, Invisible Woman,
//! `decks::cmdr_invisible_woman`), and its primitives: a forced attack on a
//! player for a duration (CR 508.1d), a pump for the attackers on the same
//! player, a random linked graveyard exile, owners regaining control, and
//! "the opponent with the most life".

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, Selector};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::grizzly_bears());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_hand(0, def);
    cast_at(g, id, &[], None).expect("castable");
    id
}

fn activate(g: &mut GameState, id: CardId, index: usize, targets: &[Target]) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
    g.step = TurnStep::PreCombatMain;
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

/// A cheap noncreature spell.
fn noncreature(g: &mut GameState) {
    cast(g, catalog::sol_ring());
}

fn declare(g: &mut GameState, attacks: &[(CardId, usize)]) -> Result<(), String> {
    for (a, _) in attacks {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    let r = g
        .perform_action(GameAction::DeclareAttackers(
            attacks.iter().map(|&(attacker, p)| Attack { attacker, target: AttackTarget::Player(p) }).collect(),
        ))
        .map(|_| ())
        .map_err(|e| format!("{e:?}"));
    drain_stack(g);
    r
}

/// Attack, no blocks, through combat damage.
fn connect(g: &mut GameState, attacks: &[(CardId, usize)]) {
    declare(g, attacks).expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = attacks[0].1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

// ── The commander ────────────────────────────────────────────────────────────

/// CR 903.4 — mana symbols in rules text count: Invisible Woman's
/// {R}{G}{W}{U} makes a {2}{W} card a four-color commander.
#[test]
fn invisible_woman_identity_reads_her_rules_text() {
    let id = crabomination::color_identity::color_identity(&catalog::invisible_woman());
    for c in [Color::White, Color::Blue, Color::Red, Color::Green] {
        assert!(id.contains(c), "{c:?}");
    }
    assert!(!id.contains(Color::Black));
}

/// CR 603.4 — the Wall comes only on a turn you cast a noncreature spell.
#[test]
fn invisible_woman_walls_after_a_noncreature_spell() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::invisible_woman());
    step(&mut g, TurnStep::BeginCombat);
    assert_eq!(named(&g, 0, "Wall"), 0);
    noncreature(&mut g);
    step(&mut g, TurnStep::BeginCombat);
    assert_eq!(named(&g, 0, "Wall"), 1);
}

// ── Multiplayer primitives ───────────────────────────────────────────────────

/// Galactus must attack an opponent with the most life — not another one —
/// unless you control Silver Surfer.
#[test]
fn galactus_hunts_the_richest_opponent() {
    let mut g = pod(3);
    g.players[2].life = 60;
    let gal = g.add_card_to_battlefield(0, catalog::galactus_devourer_of_worlds());
    step(&mut g, TurnStep::BeginCombat);
    g.step = TurnStep::BeginCombat;
    assert!(declare(&mut g, &[(gal, 1)]).is_err(), "seat 1 has less life");
    let mut g2 = pod(3);
    g2.players[2].life = 60;
    let gal2 = g2.add_card_to_battlefield(0, catalog::galactus_devourer_of_worlds());
    step(&mut g2, TurnStep::BeginCombat);
    declare(&mut g2, &[(gal2, 2)]).expect("the richest opponent");
    let mut g3 = pod(3);
    g3.players[2].life = 60;
    let gal3 = g3.add_card_to_battlefield(0, catalog::galactus_devourer_of_worlds());
    g3.add_card_to_battlefield(0, catalog::silver_surfer_galactuss_herald());
    step(&mut g3, TurnStep::BeginCombat);
    declare(&mut g3, &[(gal3, 1)]).expect("the Herald frees him");
}

/// CR 508.1d — Silver Surfer's connecting hit makes a creature attack that
/// player each combat until the end of your next turn.
#[test]
fn silver_surfer_directs_an_attacker() {
    let mut g = pod(3);
    let surfer = g.add_card_to_battlefield(0, catalog::silver_surfer_galactuss_herald());
    let theirs = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(theirs))]));
    connect(&mut g, &[(surfer, 1)]);
    let c = g.battlefield_find(theirs).unwrap();
    assert_eq!(c.chosen_player, Some(1));
    assert!(g.computed_permanent(theirs).unwrap().keywords().contains(&Keyword::MustAttackChosenPlayer));
}

/// Namor pumps only the other attackers on the player he attacks, and only
/// when that player has more life than you.
#[test]
fn namor_pumps_attackers_on_the_same_richer_player() {
    let mut g = pod(3);
    g.players[1].life = 30;
    let namor = g.add_card_to_battlefield(0, catalog::namor_atlantean_king());
    let with = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let elsewhere = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    declare(&mut g, &[(namor, 1), (with, 1), (elsewhere, 2)]).expect("attack");
    assert_eq!(pt(&g, with), (4, 2));
    assert_eq!(pt(&g, elsewhere), (2, 2));
    assert_eq!(pt(&g, namor), (2, 2), "other creatures");
}

/// Alicia Masters: at your end step every creature goes home to its owner,
/// a stolen token included.
#[test]
fn alicia_returns_every_creature_to_its_owner() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::alicia_masters_skilled_sculptor());
    let stolen = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g
        .resolve_effect(
            &Effect::GainControl {
                what: Selector::ExactObjects(vec![stolen]),
                to: None,
                duration: crabomination::effect::Duration::Permanent,
            },
            &ctx,
        )
        .expect("steal");
    g.dispatch_triggers_for_events(&evs);
    assert_eq!(g.battlefield_find(stolen).unwrap().controller, 0);
    step(&mut g, TurnStep::End);
    assert_eq!(g.battlefield_find(stolen).unwrap().controller, 1);
}

/// Power Pack exiles a random instant or sorcery from your graveyard and
/// casts it free at your next upkeep.
#[test]
fn power_pack_recasts_a_random_spell() {
    let mut g = pod(2);
    let pack = g.add_card_to_battlefield(0, catalog::power_pack());
    g.add_card_to_graveyard(0, catalog::divination());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    connect(&mut g, &[(pack, 1)]);
    assert!(g.exile.iter().any(|c| c.definition.name == "Divination"), "the only instant or sorcery");
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true); 8]));
    // Through the other seat's turn to our next upkeep, where the delayed
    // trigger fires.
    g.step = TurnStep::Cleanup;
    g.active_player_idx = 1;
    for _ in 0..40 {
        if g.active_player_idx == 0 && g.step == TurnStep::Upkeep && g.stack.is_empty() {
            break;
        }
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.delayed_triggers.is_empty(), "the rebound-like trigger fired");
    assert_eq!(g.players[0].hand.len(), hand + 2, "Divination cast free");
    assert!(g.exile.iter().any(|c| c.definition.name == "Divination"), "exiled again, not binned");
}

/// Willie Lumpkin: the damaged player may draw, and then can't attack you.
#[test]
fn willie_lumpkins_mail_buys_peace() {
    let mut g = pod(3);
    let willie = g.add_card_to_battlefield(0, catalog::willie_lumpkin_postman());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let (mine, others) = (g.players[0].hand.len(), g.players[1].hand.len());
    connect(&mut g, &[(willie, 1)]);
    assert_eq!((g.players[0].hand.len(), g.players[1].hand.len()), (mine + 1, others + 1));
    let kws = g.computed_permanent(theirs).unwrap().keywords().to_vec();
    assert!(kws.iter().any(|k| matches!(k, Keyword::CantAttackPlayer(0))), "{kws:?}");
}

// ── Noncreature-spell payoffs ────────────────────────────────────────────────

/// First Family counts the colors among your permanents and the spells you
/// cast this turn.
#[test]
fn first_family_counts_colors_of_permanents_and_spells() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let hand = g.players[0].hand.len();
    let life = g.players[0].life;
    cast(&mut g, catalog::first_family());
    // Green (the Elves) and green + blue (First Family itself).
    assert_eq!(g.players[0].hand.len(), hand + 2);
    assert_eq!(g.players[0].life, life + 2);
}

/// Crystal deals damage equal to the cast spell's number of colors.
#[test]
fn crystal_burns_for_the_spells_colors() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::crystal_inhuman_princess());
    cast(&mut g, catalog::first_family());
    assert_eq!((g.players[1].life, g.players[2].life), (18, 18));
}

/// Valeria draws for the first noncreature spell each turn only.
#[test]
fn valeria_draws_on_the_first_noncreature_spell() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::valeria_richards_precocious());
    let hand = g.players[0].hand.len();
    noncreature(&mut g);
    noncreature(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// The Fantasticar trades itself for four Constructs on the fourth
/// noncreature spell.
#[test]
fn the_fantasticar_becomes_four_constructs() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::the_fantasticar());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(false),
        DecisionAnswer::Bool(false),
        DecisionAnswer::Bool(false),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
    ]));
    for _ in 0..4 {
        noncreature(&mut g);
    }
    assert_eq!(named(&g, 0, "Construct"), 4);
    assert_eq!(named(&g, 0, "The Fantasticar"), 0);
}

/// Dragon Man's power is the greatest mana value among your noncreature
/// permanents and noncreature graveyard cards.
#[test]
fn dragon_man_power_reads_both_zones() {
    let mut g = pod(2);
    let dm = g.add_card_to_battlefield(0, catalog::dragon_man_reformed_robot());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    assert_eq!(pt(&g, dm), (1, 5));
    g.add_card_to_graveyard(0, catalog::cosmic_crucible());
    assert_eq!(pt(&g, dm), (6, 5));
}

/// Nova Flame: X counters on your creature, then it deals its power to each
/// other creature.
#[test]
fn nova_flame_burns_with_the_grown_creature() {
    let mut g = pod(2);
    let mine = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let theirs = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let nf = g.add_card_to_hand(0, catalog::nova_flame());
    cast_at(&mut g, nf, &[Target::Permanent(mine)], Some(2)).expect("X = 2");
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert!(g.battlefield_find(theirs).is_none(), "8 damage to a 6/4");
    assert!(g.battlefield_find(mine).is_some(), "not to itself");
}

/// Ultimate Nullification: a legendary creature as the cost, then every
/// creature and graveyard is exiled and the card goes to the library bottom.
#[test]
fn ultimate_nullification_exiles_creatures_and_graveyards() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::council_of_reeds());
    let theirs = g.add_card_to_battlefield(1, catalog::craw_wurm());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let un = g.add_card_to_hand(0, catalog::ultimate_nullification());
    cast_at(&mut g, un, &[], None).expect("a legend to sacrifice");
    assert!(g.battlefield_find(theirs).is_none());
    assert!(g.players[1].graveyard.is_empty() && g.players[0].graveyard.is_empty());
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(un), "the bottom of its owner's library");
}

/// Council of Reeds copies itself, and the legend rule leaves both.
#[test]
fn council_of_reeds_copies_itself_legend_rule_aside() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::council_of_reeds());
    noncreature(&mut g);
    step(&mut g, TurnStep::BeginCombat);
    assert_eq!(named(&g, 0, "Council of Reeds"), 2);
}

/// Black Bolt's Lethal Voice: an opponent targeting it loses a nonland
/// permanent.
#[test]
fn black_bolt_answers_a_targeting_opponent() {
    let mut g = pod(2);
    let bolt = g.add_card_to_battlefield(0, catalog::black_bolt_inhuman_king());
    let theirs = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let shock = g.add_card_to_hand(1, catalog::shock());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: shock,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("shock");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none());
}

/// Negative Zone Portal exiles a card from an opponent's graveyard, linked to
/// it, drawing only for a creature card.
#[test]
fn negative_zone_portal_draws_for_a_creature_card() {
    let mut g = pod(2);
    let portal = g.add_card_to_battlefield(0, catalog::negative_zone_portal());
    let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let ring = g.add_card_to_graveyard(1, catalog::sol_ring());
    let hand = g.players[0].hand.len();
    activate(&mut g, portal, 0, &[Target::Permanent(ring)]).expect("exile a Sol Ring");
    assert_eq!(g.players[0].hand.len(), hand);
    g.battlefield_find_mut(portal).unwrap().tapped = false;
    activate(&mut g, portal, 0, &[Target::Permanent(bear)]).expect("exile a creature card");
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert!(g.exile.iter().all(|c| c.exiled_with == Some(portal)));
}
