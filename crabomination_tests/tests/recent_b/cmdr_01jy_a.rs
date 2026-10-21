//! Commander: the 2026-10-21 (`01Jywzky`) pair-deck closers
//! (`decks::cmdr_01jy_a`) and the primitives they need.

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
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
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn cast_as(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), GameError> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn attack_with(g: &mut GameState, attackers: &[CardId]) {
    for &a in attackers {
        g.clear_sickness(a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

/// Through the next turn's upkeep, resolving whatever triggers there.
fn to_next_upkeep(g: &mut GameState) {
    let turn = g.turn_number;
    while !(g.step == TurnStep::Upkeep && g.turn_number > turn) {
        let _ = g.advance_step(Vec::new());
        if !(g.step == TurnStep::Upkeep && g.turn_number > turn) {
            drain_stack(g);
        }
    }
    drain_stack(g);
}

/// Burnout may target any instant spell and counters it only if it's blue;
/// the delayed draw waits for the next turn's upkeep either way (CR 603.7).
#[test]
fn burnout_counters_only_a_blue_instant_and_draws_next_upkeep() {
    for (blue, spell) in [(true, catalog::opt as fn() -> _), (false, catalog::lightning_bolt)] {
        let mut g = pod(2);
        let instant = g.add_card_to_hand(1, spell());
        cast_as(&mut g, 1, instant, (!blue).then_some(Target::Player(0)));
        let burnout = g.add_card_to_hand(0, catalog::burnout());
        cast_as(&mut g, 0, burnout, Some(Target::Permanent(instant)));
        let life = g.players[0].life;
        drain_stack(&mut g);
        assert!(g.players[1].graveyard.iter().any(|c| c.id == instant));
        assert_eq!(g.players[0].life == life, blue, "only the blue instant is countered");
        let hand = g.players[0].hand.len();
        to_next_upkeep(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "the draw lands at the next turn's upkeep");
    }
}

/// Mana Short taps every land the target player controls and empties their
/// pool — even mana Upwelling would keep past a step's end (CR 500.4 governs
/// only the step-end empty, not an effect).
#[test]
fn mana_short_taps_lands_and_empties_the_pool() {
    let mut g = pod(3);
    let land = g.add_card_to_battlefield(1, catalog::island());
    let other = g.add_card_to_battlefield(2, catalog::island());
    g.add_card_to_battlefield(1, catalog::upwelling());
    g.players[1].mana_pool.add(Color::Green, 4);
    let short = g.add_card_to_hand(0, catalog::mana_short());
    cast_as(&mut g, 0, short, Some(Target::Player(1)));
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).unwrap().tapped);
    assert!(!g.battlefield_find(other).unwrap().tapped, "only the target player's lands");
    assert_eq!(g.players[1].mana_pool.total(), 0);
}

/// Energy Storm prevents a spell's damage to every recipient (CR 615) but not
/// a permanent's, and a flier stays tapped through its untap step (CR 502.3).
#[test]
fn energy_storm_stops_spell_damage_and_untapping_fliers() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::energy_storm());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let life = g.players[1].life;
    cast_as(&mut g, 0, bolt, Some(Target::Player(1)));
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life, "the instant's damage is prevented");
    let flier = g.add_card_to_battlefield(0, catalog::serra_angel());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [flier, bears] {
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    g.active_player_idx = 1;
    to_next_upkeep(&mut g);
    assert_eq!(g.active_player_idx, 0);
    assert!(g.battlefield_find(flier).unwrap().tapped, "the flier doesn't untap");
    assert!(!g.battlefield_find(bears).unwrap().tapped);
}

/// Ambitious Farmhand transforms only with coven — three creatures of
/// different powers — and its back is a 3/3 lifelinker.
#[test]
fn ambitious_farmhand_transforms_only_with_coven() {
    let mut g = pod(2);
    let farmhand = g.add_card_to_battlefield(0, catalog::ambitious_farmhand());
    assert!(activate(&mut g, farmhand, 0, None).is_err(), "no coven yet");
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::hill_giant());
    activate(&mut g, farmhand, 0, None).expect("coven: powers 1, 2, 3");
    assert_eq!(pt(&g, farmhand), (3, 3));
    assert!(g.computed_permanent(farmhand).unwrap().keywords().contains(&Keyword::Lifelink));
}

/// Pit Trap destroys an attacker without flying, never a flier or a creature
/// that isn't attacking.
#[test]
fn pit_trap_takes_only_a_grounded_attacker() {
    let mut g = pod(2);
    let trap = g.add_card_to_battlefield(1, catalog::pit_trap());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    attack_with(&mut g, &[bears, angel]);
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    let aim = |g: &mut GameState, t: CardId| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: trap,
            ability_index: 0,
            target: Some(Target::Permanent(t)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(aim(&mut g, angel).is_err(), "a flier is out of reach");
    aim(&mut g, bears).expect("a grounded attacker");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none());
    assert!(g.battlefield_find(angel).is_some());
}

/// Covenant of Minds: the target opponent picks — the three revealed cards to
/// your hand, or into your graveyard while you draw five.
#[test]
fn covenant_of_minds_gives_three_or_five() {
    let mut g = pod(2);
    let covenant = g.add_card_to_hand(0, catalog::covenant_of_minds());
    let hand = g.players[0].hand.len();
    let library = g.players[0].library.len();
    cast_as(&mut g, 0, covenant, Some(Target::Player(1)));
    drain_stack(&mut g);
    let drew = g.players[0].hand.len() + 1 - hand;
    let binned = g.players[0].graveyard.len() - 1;
    assert!((drew, binned) == (3, 0) || (drew, binned) == (5, 3), "drew {drew}, binned {binned}");
    assert_eq!(library - g.players[0].library.len(), drew + binned);
}

/// Impetuous Protege attacks for the greatest power among its opponents'
/// TAPPED creatures; an untapped bigger one doesn't count.
#[test]
fn impetuous_protege_reads_tapped_opposing_power() {
    let mut g = pod(2);
    let protege = g.add_card_to_battlefield(0, catalog::impetuous_protege());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.add_card_to_battlefield(1, catalog::serra_angel());
    g.battlefield_find_mut(giant).unwrap().tapped = true;
    attack_with(&mut g, &[protege]);
    assert_eq!(pt(&g, protege), (3, 4));
}

/// Inscribed Tablet takes a land from the top five; with none there it draws.
#[test]
fn inscribed_tablet_finds_a_land_or_draws() {
    for land in [true, false] {
        let mut g = multi_player_game(2);
        if land {
            g.add_card_to_library(0, catalog::forest());
        }
        for _ in 0..8 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let tablet = g.add_card_to_battlefield(0, catalog::inscribed_tablet());
        let hand = g.players[0].hand.len();
        activate(&mut g, tablet, 0, None).expect("activate");
        assert_eq!(g.players[0].hand.len(), hand + 1);
        let got_forest = g.players[0].hand.iter().any(|c| c.definition.name == "Forest");
        assert_eq!(got_forest, land);
    }
}

/// Bloodhaze Wolverine's trigger is the second draw of the turn, once.
#[test]
fn bloodhaze_wolverine_wakes_on_the_second_draw() {
    let mut g = pod(2);
    let wolverine = g.add_card_to_battlefield(0, catalog::bloodhaze_wolverine());
    let one = g.add_card_to_hand(0, catalog::opt());
    cast_as(&mut g, 0, one, None);
    drain_stack(&mut g);
    assert_eq!(pt(&g, wolverine), (2, 1), "one card drawn");
    let two = g.add_card_to_hand(0, catalog::divination());
    cast_as(&mut g, 0, two, None);
    drain_stack(&mut g);
    assert_eq!(pt(&g, wolverine), (3, 2), "the second and third draws trigger once");
    assert!(g.computed_permanent(wolverine).unwrap().keywords().contains(&Keyword::FirstStrike));
}

/// Weathered Runestone: a nonland permanent card can't be reanimated and
/// nothing is cast from a graveyard; a land card still enters from one.
#[test]
fn weathered_runestone_locks_graveyards_for_nonland_permanents() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::weathered_runestone());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let land = g.add_card_to_graveyard(0, catalog::forest());
    let back = crabomination::effect::Effect::Move {
        what: crabomination::effect::Selector::Target(0),
        to: crabomination::effect::ZoneDest::Battlefield {
            controller: crabomination::effect::PlayerRef::You,
            tapped: false,
        },
    };
    for what in [bears, land] {
        let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(what)), 0, 0);
        g.resolve_effect(&back, &ctx).unwrap();
    }
    assert!(g.battlefield_find(bears).is_none(), "a creature card stays in the graveyard");
    assert!(g.battlefield_find(land).is_some(), "a land card isn't covered");
    let current = g.add_card_to_graveyard(0, catalog::direct_current());
    flood(&mut g, 0);
    g.add_card_to_hand(0, catalog::island());
    let jump = g.perform_action(GameAction::CastSpell {
        card_id: current,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(jump.is_err(), "no jump-start from the graveyard");
}

/// Knight of Cliffhaven levels: 2/2, then 2/3 flier at 1, then 4/4 flying
/// vigilance at 4 (CR 702.87 / 711).
#[test]
fn knight_of_cliffhaven_levels_up() {
    let mut g = pod(2);
    let knight = g.add_card_to_battlefield(0, catalog::knight_of_cliffhaven());
    assert_eq!(pt(&g, knight), (2, 2));
    activate(&mut g, knight, 0, None).unwrap();
    assert_eq!(pt(&g, knight), (2, 3));
    for _ in 0..3 {
        activate(&mut g, knight, 0, None).unwrap();
    }
    assert_eq!(pt(&g, knight), (4, 4));
    let knight = g.computed_permanent(knight).unwrap();
    let kws = knight.keywords();
    assert!(kws.contains(&Keyword::Flying) && kws.contains(&Keyword::Vigilance));
}

/// Flower // Flourish: the left half fetches a basic Forest or Plains, the
/// right half pumps the team (CR 709.3).
#[test]
fn flower_flourish_halves() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::plains());
    let flower = g.add_card_to_hand(0, catalog::flower_flourish());
    cast_as(&mut g, 0, flower, None);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Plains"));
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let flourish = g.add_card_to_hand(0, catalog::flower_flourish());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSplitRight {
        card_id: flourish,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Flourish");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bears), (4, 4));
}

/// Tenacity untaps the creatures it pumps.
#[test]
fn tenacity_pumps_and_untaps() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().tapped = true;
    let tenacity = g.add_card_to_hand(0, catalog::tenacity());
    cast_as(&mut g, 0, tenacity, None);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bears).unwrap().tapped);
    assert_eq!(pt(&g, bears), (3, 3));
    assert!(g.computed_permanent(bears).unwrap().keywords().contains(&Keyword::Lifelink));
}

/// Dancing Sword: once its creature dies it may stand up as a 2/1 flying
/// Construct, and it is no longer an Equipment (CR 301.5).
#[test]
fn dancing_sword_dances_when_its_creature_dies() {
    let mut g = pod(2);
    let sword = g.add_card_to_battlefield(0, catalog::dancing_sword());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: sword, target: bears }).expect("equip");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bears), (4, 3));
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.remove_to_graveyard_with_triggers(bears);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none());
    let sword = g.computed_permanent(sword).expect("the sword stays");
    assert_eq!((sword.power, sword.toughness), (2, 1));
    assert!(sword.keywords().contains(&Keyword::Flying));
    assert!(sword.subtypes().artifact_subtypes.is_empty(), "it isn't an Equipment");
}

/// Tromp the Domains pumps by the basic land types among your lands.
#[test]
fn tromp_the_domains_counts_basic_types() {
    let mut g = pod(2);
    for land in [catalog::forest(), catalog::island(), catalog::island()] {
        g.add_card_to_battlefield(0, land);
    }
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let tromp = g.add_card_to_hand(0, catalog::tromp_the_domains());
    cast_as(&mut g, 0, tromp, None);
    drain_stack(&mut g);
    assert_eq!(pt(&g, bears), (4, 4));
    assert!(g.computed_permanent(bears).unwrap().keywords().contains(&Keyword::Trample));
}

/// Check for Traps exiles the chosen nonland card; a flash card costs its
/// owner a life, anything else costs the caster one.
#[test]
fn check_for_traps_charges_by_what_it_finds() {
    for (card, theirs) in [(catalog::opt as fn() -> _, true), (catalog::grizzly_bears, false)] {
        let mut g = pod(2);
        let found = g.add_card_to_hand(1, card());
        g.add_card_to_hand(1, catalog::forest());
        let traps = g.add_card_to_hand(0, catalog::check_for_traps());
        let (mine, opp) = (g.players[0].life, g.players[1].life);
        cast_as(&mut g, 0, traps, Some(Target::Player(1)));
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == found), "the nonland card is exiled");
        assert_eq!((g.players[0].life, g.players[1].life), if theirs { (mine, opp - 1) } else { (mine - 1, opp) });
    }
}

/// Stunning Strike taps a creature and keeps a nonlegendary one tapped
/// through its untap step.
#[test]
fn stunning_strike_holds_a_nonlegendary_creature_down() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let strike = g.add_card_to_hand(0, catalog::stunning_strike());
    cast_as(&mut g, 0, strike, Some(Target::Permanent(bears)));
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).unwrap().tapped);
    g.active_player_idx = 0;
    to_next_upkeep(&mut g);
    assert_eq!(g.active_player_idx, 1);
    assert!(g.battlefield_find(bears).unwrap().tapped, "doesn't untap");
}

/// Clay Champion counts {G}{G} pairs for its own counters and {W}{W} pairs
/// for the creatures it targets.
#[test]
fn clay_champion_counts_colored_pairs() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let champion = g.add_card_to_hand(0, catalog::clay_champion());
    g.players[0].mana_pool.add(Color::Green, 4);
    g.players[0].mana_pool.add(Color::White, 2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: champion,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast for X = 2 with GGGG WW");
    drain_stack(&mut g);
    assert_eq!(pt(&g, champion), (8, 8), "two {{G}}{{G}} pairs: six counters");
    assert_eq!(pt(&g, bears), (3, 3), "one {{W}}{{W}} pair: one counter");
}

/// Prepare // Fight: Fight is cast from the graveyard only (CR 702.127).
#[test]
fn prepare_fight_fights_from_the_graveyard() {
    let mut g = pod(2);
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let card = g.add_card_to_hand(0, catalog::prepare_fight());
    flood(&mut g, 0);
    let fight = |g: &mut GameState| {
        g.perform_action(GameAction::CastAftermath {
            card_id: card,
            target: Some(Target::Permanent(giant)),
            additional_targets: vec![Target::Permanent(bears)],
            mode: None,
            x_value: None,
        })
    };
    assert!(fight(&mut g).is_err(), "aftermath: not from the hand");
    cast_as(&mut g, 0, card, Some(Target::Permanent(giant)));
    drain_stack(&mut g);
    assert_eq!(pt(&g, giant), (5, 5));
    flood(&mut g, 0);
    fight(&mut g).expect("Fight from the graveyard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none());
    assert!(g.exile.iter().any(|c| c.id == card), "then exiled");
}

/// Tezzeret the Schemer's −2 is +X/−X for the artifacts you control.
#[test]
fn tezzeret_the_schemer_minus_two_scales_with_artifacts() {
    let mut g = pod(2);
    let tez = g.add_card_to_battlefield(0, catalog::tezzeret_the_schemer());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    g.add_card_to_battlefield(0, catalog::mimic());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: tez,
        ability_index: 1,
        target: Some(Target::Permanent(giant)),
        x_value: None,
    })
    .expect("−2");
    drain_stack(&mut g);
    assert_eq!(pt(&g, giant), (5, 1));
}

/// Dovin Baan's emblem caps each opponent at two untaps in their untap step
/// (CR 502.3); its owner untaps everything.
#[test]
fn dovin_baan_emblem_caps_opponents_untaps() {
    let mut g = pod(3);
    let dovin = g.add_card_to_battlefield(0, catalog::dovin_baan());
    g.battlefield_find_mut(dovin).unwrap().add_counters(crabomination::card::CounterType::Loyalty, 4);
    let theirs: Vec<CardId> = (0..4).map(|_| g.add_card_to_battlefield(1, catalog::forest())).collect();
    let mine: Vec<CardId> = (0..3).map(|_| g.add_card_to_battlefield(0, catalog::forest())).collect();
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: dovin, ability_index: 2, target: None, x_value: None })
        .expect("−7");
    drain_stack(&mut g);
    for &id in theirs.iter().chain(&mine) {
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    to_next_upkeep(&mut g);
    assert_eq!(g.active_player_idx, 1);
    let untapped = theirs.iter().filter(|&&id| !g.battlefield_find(id).unwrap().tapped).count();
    assert_eq!(untapped, 2, "the opponent untaps two of four");
    g.active_player_idx = 2;
    to_next_upkeep(&mut g);
    assert_eq!(g.active_player_idx, 0);
    assert!(mine.iter().all(|&id| !g.battlefield_find(id).unwrap().tapped), "the owner isn't capped");
}

/// Druidic Ritual returns at most one creature and one land card.
#[test]
fn druidic_ritual_returns_a_creature_and_a_land() {
    let mut g = pod(2);
    for card in [catalog::grizzly_bears(), catalog::hill_giant(), catalog::forest(), catalog::forest()] {
        g.add_card_to_graveyard(0, card);
    }
    let ritual = g.add_card_to_hand(0, catalog::druidic_ritual());
    let hand = g.players[0].hand.len();
    cast_as(&mut g, 0, ritual, None);
    drain_stack(&mut g);
    let hand_now = &g.players[0].hand;
    assert_eq!(hand_now.len(), hand + 1, "two cards back, the ritual gone");
    assert_eq!(hand_now.iter().filter(|c| c.definition.is_land()).count(), 1);
}

/// Nevermaker cast for evoke is sacrificed and its leaves-the-battlefield
/// trigger tops a nonland permanent.
#[test]
fn nevermaker_evoked_tops_a_permanent() {
    let mut g = pod(2);
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let never = g.add_card_to_hand(0, catalog::nevermaker());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: never,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("evoke");
    drain_stack(&mut g);
    assert!(g.battlefield_find(never).is_none(), "evoked: sacrificed");
    assert!(g.battlefield_find(giant).is_none());
    assert_eq!(g.players[1].library.iter().next().map(|c| c.id), Some(giant), "on top of its owner's library");
}

/// Voracious Dragon deals twice the Goblins it devoured — a devoured
/// non-Goblin counts for its counters, not its damage (CR 702.82).
#[test]
fn voracious_dragon_counts_devoured_goblins() {
    use crabomination::card::{CardType, CounterType, CreatureType, Subtypes, TokenDefinition};
    use crabomination::effect::{Effect, PlayerRef, Value};
    let mut g = pod(2);
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    for kind in [CreatureType::Goblin, CreatureType::Goblin, CreatureType::Soldier] {
        let token = TokenDefinition {
            name: "Token".into(),
            power: 1,
            toughness: 1,
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![kind], ..Default::default() },
            ..Default::default()
        };
        let make = Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: std::sync::Arc::new(token) };
        g.resolve_effect(&make, &ctx).unwrap();
    }
    let dragon = g.add_card_to_hand(0, catalog::voracious_dragon());
    let life = g.players[1].life;
    cast_as(&mut g, 0, dragon, None);
    drain_stack(&mut g);
    let me = g.battlefield_find(dragon).expect("on the battlefield");
    assert_eq!(me.counter_count(CounterType::PlusOnePlusOne), 3, "devour 1: all three tokens");
    assert_eq!(g.players[1].life, life - 4, "two Goblins: 4 damage at the opponent");
}

/// Academy Loremaster: the active player may draw an extra card, and if they
/// do their spells cost {2} more this turn only.
#[test]
fn academy_loremaster_taxes_the_extra_draw() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::academy_loremaster());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.active_player_idx = 1;
    let hand = g.players[0].hand.len();
    while !(g.step == TurnStep::PreCombatMain && g.active_player_idx == 0) {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), hand + 2, "the draw and the extra card");
    assert_eq!(g.players[0].spell_tax_this_turn, 2);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    let cast = g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(cast.is_err(), "{{R}} no longer pays for a taxed Bolt");
}

/// Jace's Sentinel is a 2/3 that can't be blocked only beside a Jace.
#[test]
fn jaces_sentinel_wakes_beside_a_jace() {
    let mut g = pod(2);
    let sentinel = g.add_card_to_battlefield(0, catalog::jaces_sentinel());
    assert_eq!(pt(&g, sentinel), (1, 3));
    g.add_card_to_battlefield(1, catalog::jace_arcane_strategist());
    assert_eq!(pt(&g, sentinel), (1, 3), "an opponent's Jace is no help");
    g.add_card_to_battlefield(0, catalog::jace_the_living_guildpact());
    assert_eq!(pt(&g, sentinel), (2, 3));
    assert!(g.computed_permanent(sentinel).unwrap().keywords().contains(&Keyword::Unblockable));
}

/// Thanos's power-up destroys every other creature of the chosen parity —
/// zero is even (CR 702.193).
#[test]
fn thanos_snaps_one_parity() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let thanos = g.add_card_to_battlefield(0, catalog::thanos_the_mad_titan());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let token = g.add_card_to_battlefield(1, catalog::ornithopter());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: thanos,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: Some(1),
    })
    .expect("power-up, even");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(thanos).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert!(g.battlefield_find(bears).is_none(), "mana value 2 is even");
    assert!(g.battlefield_find(token).is_none(), "mana value 0 is even");
    assert!(g.battlefield_find(angel).is_some(), "mana value 5 is odd");
}

/// Tombstone Stairwell: each upkeep every player gets a Tombspawn per creature
/// card in their graveyard; each end step destroys them.
#[test]
fn tombstone_stairwell_raises_and_razes_tombspawn() {
    let mut g = pod(3);
    // Seat 1's, so seat 0's upkeep carries no cumulative-upkeep payment.
    g.add_card_to_battlefield(1, catalog::tombstone_stairwell());
    for (seat, n) in [(0, 2), (2, 1)] {
        for _ in 0..n {
            g.add_card_to_graveyard(seat, catalog::grizzly_bears());
        }
    }
    let tombspawn = |g: &GameState, seat: usize| {
        g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == "Tombspawn").count()
    };
    g.active_player_idx = 2;
    to_next_upkeep(&mut g);
    assert_eq!(g.active_player_idx, 0);
    assert_eq!((tombspawn(&g, 0), tombspawn(&g, 1), tombspawn(&g, 2)), (2, 0, 1));
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    drain_stack(&mut g);
    assert_eq!(tombspawn(&g, 0) + tombspawn(&g, 2), 0, "destroyed at the end step");
}

fn cast_kicked(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellKicked { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

/// Vine Gecko discounts only the turn's first kicked spell, and grows on each;
/// Hallar grows and burns each opponent for its counters.
#[test]
fn vine_gecko_and_hallar_feed_on_kicker() {
    use crabomination::card::CounterType;
    let mut g = pod(3);
    let gecko = g.add_card_to_battlefield(0, catalog::vine_gecko());
    let hallar = g.add_card_to_battlefield(0, catalog::hallar_the_firefletcher());
    let first = g.add_card_to_hand(0, catalog::reclaim_the_wastes());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_kicked(&mut g, first, None).expect("{G} + kicker {3}, one less");
    let second = g.add_card_to_hand(0, catalog::reclaim_the_wastes());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    assert!(cast_kicked(&mut g, second, None).is_err(), "the discount is the turn's first only");
    assert_eq!(g.battlefield_find(gecko).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(hallar).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!((g.players[1].life, g.players[2].life), (19, 19));
}

/// Murasa Sproutling, kicked, returns a card with kicker; unkicked it does
/// nothing (CR 603.4 intervening if).
#[test]
fn murasa_sproutling_returns_a_kicker_card() {
    let mut g = pod(2);
    let chronicler = g.add_card_to_graveyard(0, catalog::ghitu_chronicler());
    let sprout = g.add_card_to_hand(0, catalog::murasa_sproutling());
    flood(&mut g, 0);
    cast_kicked(&mut g, sprout, None).expect("kicked");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == chronicler));
}

/// Dragon Hunter blocks an attacking Dragon with flying as though it had
/// reach — and only a Dragon (CR 702.17b).
#[test]
fn dragon_hunter_reaches_only_for_dragons() {
    let mut g = pod(2);
    let hunter = g.add_card_to_battlefield(1, catalog::dragon_hunter());
    let dragon = g.add_card_to_battlefield(0, catalog::alabaster_dragon());
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    attack_with(&mut g, &[dragon, angel]);
    assert!(g.blocker_can_block_attacker(hunter, dragon));
    assert!(!g.blocker_can_block_attacker(hunter, angel));
}

/// Grand Master of Flowers is a 7/7 flying, indestructible Dragon God only
/// at seven or more loyalty.
#[test]
fn grand_master_of_flowers_rises_at_seven_loyalty() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let gm = g.add_card_to_battlefield(0, catalog::grand_master_of_flowers());
    let creature = |g: &GameState| g.computed_permanent(gm).unwrap().card_types().contains(&crabomination::card::CardType::Creature);
    assert!(!creature(&g));
    g.battlefield_find_mut(gm).unwrap().add_counters(CounterType::Loyalty, 7);
    assert!(creature(&g));
    let cp = g.computed_permanent(gm).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 7));
    assert!(cp.keywords().contains(&Keyword::Indestructible));
}

/// Alabaster Dragon dying goes into its owner's library, not the graveyard.
#[test]
fn alabaster_dragon_shuffles_home() {
    let mut g = pod(2);
    let dragon = g.add_card_to_battlefield(0, catalog::alabaster_dragon());
    g.remove_to_graveyard_with_triggers(dragon);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().all(|c| c.id != dragon));
    assert!(g.players[0].library.iter().any(|c| c.id == dragon));
}

/// Minion of Leshrac with no other creature to feed it bites its controller
/// for 5 at upkeep and taps; {T} destroys a creature or land.
#[test]
fn minion_of_leshrac_bites_without_fodder() {
    let mut g = pod(2);
    let minion = g.add_card_to_battlefield(0, catalog::minion_of_leshrac());
    g.clear_sickness(minion);
    let life = g.players[0].life;
    g.active_player_idx = 1;
    to_next_upkeep(&mut g);
    assert_eq!(g.active_player_idx, 0);
    assert_eq!(g.players[0].life, life - 5);
    assert!(g.battlefield_find(minion).unwrap().tapped, "dealt damage this way: tapped");
}

/// Hateflayer's {Q} ability needs it tapped; it untaps and deals its power.
#[test]
fn hateflayer_untaps_to_fire() {
    let mut g = pod(2);
    let flayer = g.add_card_to_battlefield(0, catalog::hateflayer());
    g.clear_sickness(flayer);
    assert!(activate(&mut g, flayer, 0, Some(Target::Player(1))).is_err(), "{{Q}} needs it tapped");
    g.battlefield_find_mut(flayer).unwrap().tapped = true;
    let life = g.players[1].life;
    activate(&mut g, flayer, 0, Some(Target::Player(1))).expect("{{2}}{{R}}, {{Q}}");
    assert!(!g.battlefield_find(flayer).unwrap().tapped);
    assert_eq!(g.players[1].life, life - 5);
}
