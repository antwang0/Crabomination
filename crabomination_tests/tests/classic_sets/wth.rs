//! Weatherlight (WTH) — `catalog::sets::wth`.

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

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

/// Abyssal Gatekeeper's death is a symmetric edict.
#[test]
fn abyssal_gatekeeper_edicts_the_table_on_death() {
    let mut g = two_player_game();
    let keeper = g.add_card_to_battlefield(0, catalog::abyssal_gatekeeper());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut events = Vec::new();
    g.destroy_permanent(keeper, false, &mut events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none());
    assert!(g.battlefield_find(mine).is_none());
}

/// Cinder Giant scorches its own team but never itself.
#[test]
fn cinder_giant_spares_only_itself() {
    let mut g = two_player_game();
    let giant = ready(&mut g, 0, catalog::cinder_giant());
    let friend = ready(&mut g, 0, catalog::grizzly_bears());
    let foe = ready(&mut g, 1, catalog::grizzly_bears());
    let upkeep = catalog::cinder_giant().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_ability(giant, 0, None);
    g.resolve_effect(&upkeep, &ctx).expect("upkeep");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_some());
    assert!(g.battlefield_find(friend).is_none());
    assert!(g.battlefield_find(foe).is_some(), "only your own board burns");
}

/// Cinder Wall burns out at end of combat once it blocks.
#[test]
fn cinder_wall_dies_after_one_block() {
    let mut g = two_player_game();
    let wall = ready(&mut g, 0, catalog::cinder_wall());
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
    assert!(g.battlefield_find(wall).is_none());
}

/// Bubble Matrix shuts damage off for every creature, not just yours.
#[test]
fn bubble_matrix_protects_both_boards() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bubble_matrix());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(theirs))).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(theirs).unwrap().damage, 0);
    assert!(g.battlefield_find(mine).is_some());
}

/// Dingus Staff bills the dying creature's controller.
#[test]
fn dingus_staff_bills_the_owner_of_the_dead() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dingus_staff());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(bear))).expect("bolt it");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
    assert_eq!(g.players[0].life, 20);
}

/// Empyrial Armor scales with the cards in your hand.
#[test]
fn empyrial_armor_scales_with_your_hand() {
    let mut g = two_player_game();
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let armor = g.add_card_to_battlefield(0, catalog::empyrial_armor());
    g.battlefield_find_mut(armor).unwrap().attached_to = Some(bear);
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::forest());
    }
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
}

/// Familiar Ground stops gang blocks on your side only.
#[test]
fn familiar_ground_only_helps_your_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::familiar_ground());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    assert!(
        g.computed_permanent(mine)
            .unwrap()
            .keywords
            .contains(&Keyword::CantBeBlockedByMoreThanOne)
    );
    assert!(
        !g.computed_permanent(theirs)
            .unwrap()
            .keywords
            .contains(&Keyword::CantBeBlockedByMoreThanOne)
    );
}

/// Fatal Blow only finishes a creature that's already been hit.
#[test]
fn fatal_blow_needs_prior_damage() {
    let mut g = two_player_game();
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let blow = g.add_card_to_hand(0, catalog::fatal_blow());
    g.players[0].mana_pool.add(Color::Black, 1);
    assert!(cast(&mut g, blow, Some(Target::Permanent(bear))).is_err(), "undamaged");

    g.battlefield_find_mut(bear).unwrap().damage = 1;
    g.battlefield_find_mut(bear).unwrap().dealt_damage_this_turn = true;
    cast(&mut g, blow, Some(Target::Permanent(bear))).expect("finish it");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
}

/// Abjure needs a blue permanent to feed it.
#[test]
fn abjure_eats_a_blue_permanent() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let abjure = g.add_card_to_hand(0, catalog::abjure());
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
    g.players[0].mana_pool.add(Color::Blue, 1);
    assert!(
        cast(&mut g, abjure, Some(Target::Permanent(bolt))).is_err(),
        "no blue permanent to sacrifice"
    );

    // A land is colorless — it has to be an actually blue permanent.
    let djinn = g.add_card_to_battlefield(0, catalog::cloud_djinn());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, abjure, Some(Target::Permanent(bolt))).expect("counter");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20);
    assert!(g.battlefield_find(djinn).is_none(), "the Djinn was the cost");
}

/// Jangling Automaton unlocks the defender's whole board when it attacks.
#[test]
fn jangling_automaton_untaps_the_defenders() {
    let mut g = two_player_game();
    let automaton = ready(&mut g, 0, catalog::jangling_automaton());
    let blocker = ready(&mut g, 1, catalog::grizzly_bears());
    g.battlefield_find_mut(blocker).unwrap().tapped = true;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: automaton,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(blocker).unwrap().tapped);
}

/// Downdraft grounds a flier, then sweeps the rest when cashed in.
#[test]
fn downdraft_grounds_then_sweeps() {
    let mut g = two_player_game();
    let downdraft = ready(&mut g, 0, catalog::downdraft());
    let flier = ready(&mut g, 1, catalog::cloud_djinn());
    let grounded = ready(&mut g, 1, catalog::fledgling_djinn());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Green, 1);
    activate(&mut g, downdraft, 0, Some(Target::Permanent(flier))).expect("ground it");
    drain_stack(&mut g);
    assert!(!g.computed_permanent(flier).unwrap().keywords.contains(&Keyword::Flying));

    activate(&mut g, downdraft, 1, None).expect("sweep");
    drain_stack(&mut g);
    assert!(g.battlefield_find(grounded).is_none(), "the 2/2 flier died");
    assert!(g.battlefield_find(flier).is_some(), "the grounded one was spared");
}

// ── Closing waves (`catalog::sets::wth2`) ───────────────────────────────────

use crabomination::card::{CounterType, CumulativeUpkeepCost};

/// Step the game to the given seat's upkeep so the turn-based actions run.
fn to_upkeep(g: &mut GameState, seat: usize) {
    g.active_player_idx = seat;
    g.step = TurnStep::Untap;
    g.priority.player_with_priority = seat;
    advance_to(g, TurnStep::Upkeep);
    drain_stack(g);
}

/// CR 702.24 — the age counter goes on first, and the cost scales with it.
/// The upkeep step starts with an empty pool, so payment auto-taps lands.
#[test]
fn cumulative_upkeep_scales_with_age_counters() {
    let mut g = two_player_game();
    let wolves = ready(&mut g, 0, catalog::arctic_wolves());
    let forests: Vec<CardId> = (0..6).map(|_| ready(&mut g, 0, catalog::forest())).collect();
    to_upkeep(&mut g, 0);
    assert_eq!(g.battlefield_find(wolves).unwrap().counter_count(CounterType::Age), 1);
    assert_eq!(forests.iter().filter(|&&f| g.battlefield_find(f).unwrap().tapped).count(), 2);
    for f in &forests {
        g.battlefield_find_mut(*f).unwrap().tapped = false;
    }
    to_upkeep(&mut g, 0);
    assert_eq!(g.battlefield_find(wolves).unwrap().counter_count(CounterType::Age), 2);
    assert_eq!(
        forests.iter().filter(|&&f| g.battlefield_find(f).unwrap().tapped).count(),
        4,
        "two mana per age counter"
    );
}

/// An unpayable cumulative upkeep sacrifices the permanent.
#[test]
fn cumulative_upkeep_sacrifices_when_unpaid() {
    let mut g = two_player_game();
    let efreet = ready(&mut g, 0, catalog::uktabi_efreet());
    to_upkeep(&mut g, 0);
    assert!(g.battlefield_find(efreet).is_none(), "no green mana, no Efreet");
}

/// Gallowbraid's life upkeep never kills its controller (CR 118.4 — you can't
/// pay more life than you have).
#[test]
fn gallowbraid_is_sacrificed_rather_than_paid_lethally() {
    let mut g = two_player_game();
    let brute = ready(&mut g, 0, catalog::gallowbraid());
    g.players[0].life = 1;
    to_upkeep(&mut g, 0);
    assert!(g.battlefield_find(brute).is_none());
    assert_eq!(g.players[0].life, 1);
}

/// Aboroth pays its cumulative upkeep with its own body.
#[test]
fn aboroth_shrinks_itself_each_upkeep() {
    let mut g = two_player_game();
    let aboroth = ready(&mut g, 0, catalog::aboroth());
    to_upkeep(&mut g, 0);
    let cp = g.computed_permanent(aboroth).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 8));
    to_upkeep(&mut g, 0);
    let cp = g.computed_permanent(aboroth).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 6), "two more counters on the second tick");
}

/// Psychic Vortex's "cumulative upkeep—draw a card" is always payable.
#[test]
fn psychic_vortex_draws_instead_of_paying_mana() {
    let mut g = two_player_game();
    let vortex = ready(&mut g, 0, catalog::psychic_vortex());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let before = g.players[0].hand.len();
    to_upkeep(&mut g, 0);
    assert!(g.battlefield_find(vortex).is_some());
    assert_eq!(g.players[0].hand.len(), before + 1);
}

/// Mwonvuli Ooze's body is 1 plus twice its age counters.
#[test]
fn mwonvuli_ooze_grows_two_per_age_counter() {
    let mut g = two_player_game();
    let ooze = ready(&mut g, 0, catalog::mwonvuli_ooze());
    for _ in 0..4 {
        ready(&mut g, 0, catalog::forest());
    }
    to_upkeep(&mut g, 0);
    let cp = g.computed_permanent(ooze).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
}

/// Revered Unicorn cashes its age counters in as life on the way out.
#[test]
fn revered_unicorn_pays_out_its_age_counters() {
    let mut g = two_player_game();
    let unicorn = ready(&mut g, 0, catalog::revered_unicorn());
    let forests: Vec<CardId> = (0..3).map(|_| ready(&mut g, 0, catalog::forest())).collect();
    to_upkeep(&mut g, 0);
    for f in &forests {
        g.battlefield_find_mut(*f).unwrap().tapped = false;
    }
    to_upkeep(&mut g, 0);
    let life = g.players[0].life;
    let mut events = Vec::new();
    g.destroy_permanent(unicorn, false, &mut events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "two age counters, two life");
}

/// Wave of Terror kills exactly the mana value its age counters name.
#[test]
fn wave_of_terror_kills_the_current_curve_slot() {
    let mut g = two_player_game();
    let wave = ready(&mut g, 0, catalog::wave_of_terror());
    g.battlefield_find_mut(wave).unwrap().add_counters(CounterType::Age, 2);
    let two_drop = ready(&mut g, 1, catalog::grizzly_bears());
    let one_drop = ready(&mut g, 1, catalog::rogue_elephant());
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    advance_to(&mut g, TurnStep::Draw);
    drain_stack(&mut g);
    assert!(g.battlefield_find(two_drop).is_none(), "Grizzly Bears is MV 2");
    assert!(g.battlefield_find(one_drop).is_some(), "the one-drop lives");
}

/// Barrow Ghoul eats the *top* creature card of your graveyard, not any of them.
#[test]
fn barrow_ghoul_eats_the_top_creature_card() {
    let mut g = two_player_game();
    let ghoul = ready(&mut g, 0, catalog::barrow_ghoul());
    let old = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let fresh = g.add_card_to_graveyard(0, catalog::cinder_wall());
    to_upkeep(&mut g, 0);
    assert!(g.battlefield_find(ghoul).is_some(), "it paid");
    assert!(g.exile.iter().any(|c| c.id == fresh), "the freshest corpse went");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == old));
}

/// With no creature in the graveyard, Barrow Ghoul is sacrificed.
#[test]
fn barrow_ghoul_starves_on_an_empty_graveyard() {
    let mut g = two_player_game();
    let ghoul = ready(&mut g, 0, catalog::barrow_ghoul());
    g.add_card_to_graveyard(0, catalog::forest());
    to_upkeep(&mut g, 0);
    assert!(g.battlefield_find(ghoul).is_none());
}

/// Harvest Wurm buys a basic land back instead of dying.
#[test]
fn harvest_wurm_returns_a_basic_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_graveyard(0, catalog::forest());
    let wurm = g.add_card_to_battlefield(0, catalog::harvest_wurm());
    let etb = catalog::harvest_wurm().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_ability(wurm, 0, None);
    g.resolve_effect(&etb, &ctx).expect("etb");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wurm).is_some());
    assert!(g.players[0].hand.iter().any(|c| c.id == land));
}

/// Necratog's cost takes the top creature card of your graveyard.
#[test]
fn necratog_pumps_off_the_graveyard_top() {
    let mut g = two_player_game();
    let atog = ready(&mut g, 0, catalog::necratog());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, atog, 0, None).expect("eat a corpse");
    drain_stack(&mut g);
    let cp = g.computed_permanent(atog).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4));
    assert!(g.players[0].graveyard.is_empty());
    assert!(activate(&mut g, atog, 0, None).is_err(), "nothing left to eat");
}

/// Roc Hatchling grows up only once the last shell counter comes off.
#[test]
fn roc_hatchling_hatches_after_four_upkeeps() {
    let mut g = two_player_game();
    let roc = g.add_card_to_battlefield_with_counters(0, catalog::roc_hatchling());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(roc).unwrap().counter_count(CounterType::Shell), 4);
    for _ in 0..3 {
        to_upkeep(&mut g, 0);
    }
    let cp = g.computed_permanent(roc).unwrap();
    assert_eq!((cp.power, cp.toughness), (0, 1), "still shelled");
    to_upkeep(&mut g, 0);
    let cp = g.computed_permanent(roc).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords.contains(&Keyword::Flying));
}

/// Pendrell Mists taxes every creature on the table, not just yours.
#[test]
fn pendrell_mists_taxes_the_whole_table() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::pendrell_mists());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    to_upkeep(&mut g, 0);
    assert!(g.battlefield_find(mine).is_none(), "the generic-1 upkeep went unpaid");
    assert!(g.battlefield_find(theirs).is_some(), "it's not their upkeep");
}

/// Dense Foliage stops spells from targeting creatures — abilities still work.
#[test]
fn dense_foliage_blanks_targeted_removal() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::dense_foliage());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    assert!(cast(&mut g, bolt, Some(Target::Permanent(bear))).is_err());
}

/// Steel Golem locks its own controller out of creature spells.
#[test]
fn steel_golem_locks_out_creature_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::steel_golem());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 3);
    assert!(cast(&mut g, bears, None).is_err());
}

/// Timid Drake bounces itself when anything else lands, including an opponent's.
#[test]
fn timid_drake_flees_from_any_new_creature() {
    let mut g = two_player_game();
    let drake = ready(&mut g, 0, catalog::timid_drake());
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 3);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast bears");
    drain_stack(&mut g);
    assert!(g.battlefield_find(drake).is_none());
    assert!(g.players[0].hand.iter().any(|c| c.id == drake));
}

/// Heat Stroke sweeps everything that met a blocker, both sides.
#[test]
fn heat_stroke_kills_both_sides_of_a_block() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::heat_stroke());
    let attacker = ready(&mut g, 0, catalog::benalish_infantry());
    let blocker = ready(&mut g, 1, catalog::volunteer_reserves());
    let bystander = ready(&mut g, 1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).expect("block");
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert!(g.battlefield_find(attacker).is_none());
    assert!(g.battlefield_find(blocker).is_none());
    assert!(g.battlefield_find(bystander).is_some(), "it stayed home");
}

/// Phyrexian Furnace eats the *bottom* (oldest) card of a graveyard.
#[test]
fn phyrexian_furnace_eats_the_oldest_card() {
    let mut g = two_player_game();
    let furnace = ready(&mut g, 0, catalog::phyrexian_furnace());
    let oldest = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let newest = g.add_card_to_graveyard(1, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, furnace, 0, Some(Target::Player(1))).expect("nibble");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == oldest));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == newest));
}

/// Well of Knowledge is only live during the activating player's draw step.
#[test]
fn well_of_knowledge_is_draw_step_only() {
    let mut g = two_player_game();
    let well = ready(&mut g, 0, catalog::well_of_knowledge());
    g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    assert!(activate(&mut g, well, 0, None).is_err(), "wrong step");
    g.step = TurnStep::Draw;
    activate(&mut g, well, 0, None).expect("draw step");
    drain_stack(&mut g);
}

/// Xanthic Statue stands up as an 8/8 trampler for the turn.
#[test]
fn xanthic_statue_animates() {
    let mut g = two_player_game();
    let statue = ready(&mut g, 0, catalog::xanthic_statue());
    g.players[0].mana_pool.add_colorless(5);
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, statue, 0, None).expect("stand up");
    drain_stack(&mut g);
    let cp = g.computed_permanent(statue).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 8));
    assert!(cp.keywords.contains(&Keyword::Trample));
    assert!(cp.card_types.contains(&crabomination::card::CardType::Creature));
}

/// Nature's Resurgence pays each player for their own graveyard.
#[test]
fn natures_resurgence_pays_each_player_separately() {
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::forest());
    }
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::natures_resurgence());
    g.players[0].mana_pool.add(Color::Green, 4);
    let (h0, h1) = (g.players[0].hand.len(), g.players[1].hand.len());
    cast(&mut g, spell, None).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h0 - 1 + 2);
    assert_eq!(g.players[1].hand.len(), h1 + 1);
}

/// A `..Default::default()` sanity pass over the wave's simpler bodies.
#[test]
fn wth2_stat_lines_match_print() {
    let rows: Vec<(crabomination::card::CardDefinition, i32, i32)> = vec![
        (catalog::benalish_infantry(), 1, 3),
        (catalog::razortooth_rats(), 2, 1),
        (catalog::shadow_rider(), 3, 3),
        (catalog::striped_bears(), 2, 2),
        (catalog::merfolk_traders(), 1, 2),
        (catalog::lava_hounds(), 4, 4),
        (catalog::tolarian_serpent(), 7, 7),
        (catalog::odylic_wraith(), 2, 2),
        (catalog::morinfen(), 5, 4),
        (catalog::master_of_arms(), 2, 2),
        (catalog::southern_paladin(), 3, 3),
        (catalog::llanowar_behemoth(), 4, 4),
        (catalog::llanowar_druid(), 1, 2),
        (catalog::serrated_biskelion(), 2, 2),
        (catalog::soul_shepherd(), 2, 1),
        (catalog::zombie_scavengers(), 3, 1),
        (catalog::circling_vultures(), 3, 2),
    ];
    for (def, p, t) in rows {
        assert_eq!((def.power, def.toughness), (p, t), "{}", def.name);
    }
    assert!(matches!(
        catalog::volunteer_reserves().keywords[1],
        Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Mana(_))
    ));
}
