//! CR 400.7 / 607 — a card that leaves exile is a new object: the exiler's
//! "until this leaves" link (`exiled_by`) does not follow it into its next
//! zone, nor into a later exile. Plus a CR 608.2 answer-log leak.

use crabomination::catalog;
use crabomination::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Target, TurnStep};
use crabomination::game::*;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g
}

fn resolve(g: &mut GameState, source: crabomination::card::CardId, target: Option<Target>, e: &Effect) {
    let events = g.resolve_effect(e, &EffectContext::for_ability(source, 0, target)).expect("resolves");
    g.dispatch_triggers_for_events(&events);
    drain_stack(g);
}

/// Foreboding Steamboat exiles a Bear, its attack trigger puts the Bear into
/// the graveyard, and a later exile of that card is a plain exile: the
/// Steamboat leaving doesn't bring it back. (Two 6-seat debug pods, seed
/// 130026 games 10 and 27, found the card still "exiled until the Steamboat
/// leaves" with the Steamboat long gone.)
#[test]
fn cr_400_7_a_card_out_of_exile_loses_its_exilers_link() {
    let mut g = main_phase();
    let boat = g.add_card_to_battlefield(0, catalog::foreboding_steamboat());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let etb = catalog::foreboding_steamboat().triggered_abilities[0].effect.clone();
    resolve(&mut g, boat, None, &etb);
    assert!(g.exile.iter().any(|c| c.id == bear && c.exiled_by.is_some()));
    let to_gy = Effect::Move { what: Selector::CardExiledWithSource, to: ZoneDest::Graveyard };
    resolve(&mut g, boat, None, &to_gy);
    let in_gy = g.players[1].graveyard.iter().find(|c| c.id == bear).expect("in the graveyard");
    assert!(in_gy.exiled_by.is_none() && in_gy.exiled_with.is_none());
    resolve(&mut g, boat, None, &Effect::Move { what: Selector::ExactObjects(vec![bear]), to: ZoneDest::Exile });
    resolve(&mut g, boat, None, &Effect::Destroy { what: Selector::ExactObjects(vec![boat]) });
    assert!(g.exile.iter().any(|c| c.id == bear), "a plain exile: the Steamboat leaving doesn't return it");
}

/// Hostage Taker exiles a Bear; the Bear comes back by another route, is
/// exiled again by something else, and Hostage Taker leaving must not return
/// it (CR 610.3c reads only the object it exiled).
#[test]
fn cr_400_7_a_reexiled_card_does_not_return_with_its_old_exiler() {
    let mut g = main_phase();
    let taker = g.add_card_to_battlefield(0, catalog::hostage_taker());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let etb = catalog::hostage_taker().triggered_abilities[0].effect.clone();
    resolve(&mut g, taker, Some(Target::Permanent(bear)), &etb);
    assert!(g.exile.iter().any(|c| c.id == bear), "the Taker holds it");
    resolve(
        &mut g,
        taker,
        None,
        &Effect::Move {
            what: Selector::ExactObjects(vec![bear]),
            to: ZoneDest::Battlefield { controller: PlayerRef::Seat(1), tapped: false },
        },
    );
    assert!(g.battlefield_find(bear).is_some_and(|c| c.exiled_by.is_none()));
    resolve(&mut g, taker, None, &Effect::Move { what: Selector::ExactObjects(vec![bear]), to: ZoneDest::Exile });
    resolve(&mut g, taker, None, &Effect::Destroy { what: Selector::ExactObjects(vec![taker]) });
    assert!(g.battlefield_find(bear).is_none(), "the Taker's leave returns only what it holds");
}

/// CR 400.7 / 607 — a card cast from exile leaves the link behind with the
/// exile: the Bear a Steamboat held resolves as a new permanent, and when a
/// later effect exiles it, it is not "exiled with" the Steamboat again.
#[test]
fn cr_400_7_a_card_cast_from_exile_is_not_exiled_with_its_old_exiler() {
    use crabomination::game::types::StackItem;
    let mut g = main_phase();
    let boat = g.add_card_to_battlefield(0, catalog::foreboding_steamboat());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let mut card = g.players[0].hand.pop().unwrap();
    assert_eq!(card.id, bear);
    card.exiled_with = Some(boat);
    card.cast_from_exile = true;
    g.stack.push(StackItem::Spell {
        card: Box::new(card),
        caster: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: 0,
        converged_value: 0,
        mana_spent: 2,
        uncounterable: false,
    });
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "the Bear resolved");
    g.remove_from_battlefield_to_exile(bear);
    let exiled = g.exile.iter().find(|c| c.id == bear).expect("exiled");
    assert_eq!(exiled.exiled_with, None, "a plain exile, not the Steamboat's");
}

/// CR 608.2 — an asked pick whose candidates all left while it waited spends
/// its answer. Yuffie took Lightning Greaves "for as long as you control
/// Yuffie" and asked which Equipment to attach; the steal ended before the
/// answer came back (Clever Concealment phased Yuffie out), the re-run found
/// no Equipment, and the stashed pick leaked (a strict 6-seat debug pod, seed
/// 201019 game 18). Nextest runs each test in its own process, so the strict
/// leak check (read once) is switched on here.
#[test]
fn cr_608_2_a_pick_whose_candidates_left_spends_its_answer() {
    use crabomination::decision::{Decision, DecisionAnswer};
    use crabomination::game::types::GameAction;
    // SAFETY: set before any thread reads the environment.
    unsafe { std::env::set_var("CRAB_ANSWER_LOG", "strict") };
    let mut g = main_phase();
    g.players[0].wants_ui = true;
    let greaves = g.add_card_to_battlefield(1, catalog::lightning_greaves());
    let yuffie = g.add_card_to_hand(0, catalog::yuffie_materia_hunter());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell { card_id: yuffie, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast Yuffie");
    let mut asked = false;
    for _ in 0..20 {
        if let Some(d) = g.pending_decision.as_ref() {
            let answer = match &d.decision {
                Decision::ChooseTarget { .. } => DecisionAnswer::Target(Target::Permanent(greaves)),
                Decision::OptionalTrigger { .. } => DecisionAnswer::Bool(true),
                Decision::ChooseCards { .. } => {
                    // The steal ends before the answer comes back.
                    g.battlefield_find_mut(greaves).unwrap().controller = 1;
                    asked = true;
                    DecisionAnswer::Cards(vec![greaves])
                }
                other => panic!("unexpected ask {other:?}"),
            };
            g.submit_decision(answer).expect("answer");
            continue;
        }
        if g.stack.is_empty() {
            break;
        }
        g.resolve_top_of_stack().expect("resolve");
    }
    assert!(asked, "the Equipment pick was asked");
    assert!(g.pending_decision.is_none());
    assert_eq!(g.battlefield_find(greaves).and_then(|c| c.attached_to), None, "nothing to attach");
}

/// CR 608.2 / 800.4a — Florian's look counts the life the caster's opponents
/// lost; a seat that leaves while the pick waits takes its loss out of the
/// count, the re-run looks at nothing, and the logged pick is spent rather
/// than leaked (a strict 5-seat debug pod with concessions, seed 203023
/// game 5).
#[test]
fn cr_608_2_a_look_that_shrinks_to_nothing_spends_its_pick() {
    use crabomination::card::{CardDefinition, CardType};
    use crabomination::decision::{Decision, DecisionAnswer};
    use crabomination::effect::{LookExileGrant, Value};
    use crabomination::game::types::GameAction;
    // SAFETY: set before any thread reads the environment.
    unsafe { std::env::set_var("CRAB_ANSWER_LOG", "strict") };
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].wants_ui = true;
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.players[1].life_lost_this_turn = 2;
    let look = g.add_card_to_hand(0, CardDefinition {
        name: "Florian's Look",
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookTopExileOneMayPlay {
            count: Value::TotalLifeLostThisTurn(PlayerRef::EachOpponent),
            who: PlayerRef::You,
            grant: LookExileGrant::PlayThisTurn,
        },
        ..Default::default()
    });
    g.perform_action(GameAction::CastSpell { card_id: look, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    g.resolve_top_of_stack().expect("resolve");
    assert!(matches!(g.pending_decision.as_ref().map(|d| &d.decision), Some(Decision::ChooseCards { .. })));
    g.concede(1);
    g.submit_decision(DecisionAnswer::Cards(vec![top])).expect("answer");
    assert!(g.pending_decision.is_none());
    assert!(g.exile.iter().all(|c| c.id != top), "nothing was looked at, so nothing is exiled");
}

/// The cast route: Hostage Taker exiles a Bear and its controller casts it.
/// The permanent it becomes carries no exile link, so exiled again later and
/// the Taker gone, it stays exiled.
#[test]
fn cr_400_7_a_card_cast_out_of_exile_sheds_its_exile_links() {
    use crabomination::game::types::GameAction;
    use crabomination::mana::Color;
    let mut g = main_phase();
    let taker = g.add_card_to_battlefield(0, catalog::hostage_taker());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let etb = catalog::hostage_taker().triggered_abilities[0].effect.clone();
    resolve(&mut g, taker, Some(Target::Permanent(bear)), &etb);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the hostage");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("on the battlefield");
    assert!(c.exiled_by.is_none() && c.exiled_with.is_none(), "{:?} / {:?}", c.exiled_by, c.exiled_with);
    resolve(&mut g, taker, None, &Effect::Move { what: Selector::ExactObjects(vec![bear]), to: ZoneDest::Exile });
    resolve(&mut g, taker, None, &Effect::Destroy { what: Selector::ExactObjects(vec![taker]) });
    assert!(g.exile.iter().any(|c| c.id == bear));
}

/// CR 608.2 — a resumed asker whose question went moot while it waited spends
/// its stashed answer, whatever arm it is. "You may discard a card. If you do,
/// draw" was answered yes after the hand emptied; the re-run's may stops
/// offering (an unpayable cost) and the yes used to leak into the next may in
/// the same resolution, which then gained 5 life without asking.
#[test]
fn cr_608_2_a_moot_asker_does_not_hand_its_answer_to_the_next() {
    use crabomination::card::{CardDefinition, CardType};
    use crabomination::decision::{Decision, DecisionAnswer};
    use crabomination::effect::Value;
    use crabomination::game::types::GameAction;
    let mut g = main_phase();
    g.players[0].wants_ui = true;
    let fodder = g.add_card_to_hand(0, catalog::grizzly_bears());
    let may = |body: Effect| Effect::MayDo { description: "may".into(), body: Box::new(body) };
    let spell = g.add_card_to_hand(0, CardDefinition {
        name: "Two Mays",
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            may(Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ])),
            may(Effect::GainLife { who: Selector::You, amount: Value::Const(5) }),
        ]),
        ..Default::default()
    });
    g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    g.resolve_top_of_stack().expect("resolve");
    assert!(matches!(g.pending_decision.as_ref().map(|d| &d.decision), Some(Decision::OptionalTrigger { .. })));
    // The hand empties before the answer comes back.
    let pos = g.players[0].hand.iter().position(|c| c.id == fodder).unwrap();
    let card = g.players[0].hand.remove(pos);
    g.players[0].graveyard.push(card);
    let life = g.players[0].life;
    g.submit_decision(DecisionAnswer::Bool(true)).expect("answer");
    assert!(
        matches!(g.pending_decision.as_ref().map(|d| &d.decision), Some(Decision::OptionalTrigger { .. })),
        "the second may asks for itself"
    );
    g.submit_decision(DecisionAnswer::Bool(false)).expect("decline");
    assert!(g.pending_decision.is_none());
    assert_eq!(g.players[0].life, life, "declined, so no life");
}

/// CR 115.3 / 601.2c — "destroy X target artifacts and/or enchantments" is
/// one instance of "target": X slots, each a different object. A prompting
/// seat's Heliod's Intervention at X = 2 was offered a slot past X, and the
/// first artifact again, without end — a default-pilot 4-seat strict pod
/// (seed 250011 game 21) appended the same Sol Ring 255 times and overflowed
/// the slot count.
#[test]
fn cr_115_3_x_target_slots_stop_at_x_and_never_repeat() {
    use crabomination::decision::{Decision, DecisionAnswer};
    use crabomination::game::types::GameAction;
    let mut g = main_phase();
    g.players[0].wants_ui = true;
    let rocks: Vec<_> = (0..3).map(|_| g.add_card_to_battlefield(1, catalog::sol_ring())).collect();
    let heliod = g.add_card_to_hand(0, catalog::heliods_intervention());
    let cast = |g: &mut GameState, extra: Vec<Target>| {
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: heliod,
            target: Some(Target::Permanent(rocks[0])),
            additional_targets: extra,
            mode: Some(0),
            x_value: Some(2),
        })
    };
    assert!(cast(&mut g, vec![Target::Permanent(rocks[0])]).is_err(), "the same Sol Ring twice is not two targets");
    g.players[0].mana_pool = Default::default();
    cast(&mut g, vec![]).expect("cast");
    let Some(Decision::ChooseTarget { legal, .. }) = g.pending_decision.as_ref().map(|d| d.decision.clone()) else {
        panic!("the second slot is asked");
    };
    assert!(!legal.contains(&Target::Permanent(rocks[0])), "the first pick is not offered again");
    g.submit_decision(DecisionAnswer::Target(Target::Permanent(rocks[1]))).expect("second target");
    assert!(g.pending_decision.is_none(), "no third slot at X = 2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rocks[0]).is_none() && g.battlefield_find(rocks[1]).is_none());
    assert!(g.battlefield_find(rocks[2]).is_some());
}

/// CR 601.2c / 115.3 — "destroy X target artifacts" takes exactly X
/// different targets: X = 1 with three is not a cast. The target walker that
/// fills a bot's cast filled all sixteen slots, repeating the first artifact
/// once fresh ones ran out, and the cast path took any count at any X; once
/// repeats were refused, the bot stopped casting By Force at all.
#[test]
fn cr_601_2c_destroy_x_targets_takes_exactly_x_and_a_bot_still_casts_it() {
    use crabomination::game::types::GameAction;
    use crabomination::server::bot::{Bot, HeuristicBot};
    let mut g = main_phase();
    let rocks: Vec<_> = (0..3).map(|_| g.add_card_to_battlefield(1, catalog::sol_ring())).collect();
    let force = g.add_card_to_hand(0, catalog::by_force());
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    let all: Vec<Target> = rocks.iter().map(|&r| Target::Permanent(r)).collect();
    let mut h = g.clone();
    let over = h.perform_action(GameAction::CastSpell {
        card_id: force, target: Some(all[0].clone()), additional_targets: all[1..].to_vec(), mode: None, x_value: Some(1),
    });
    assert!(over.is_err(), "three targets at X = 1");
    let mut cast = None;
    for _ in 0..12 {
        let Some(a) = HeuristicBot::new().next_action(&g, 0) else { break };
        if let GameAction::CastSpell { card_id, ref target, ref additional_targets, x_value, .. } = a
            && card_id == force
        {
            cast = Some((target.iter().chain(additional_targets).cloned().collect::<Vec<_>>(), x_value));
        }
        g.perform_action(a).expect("the bot's action is legal");
        if cast.is_some() {
            break;
        }
    }
    let (targets, x) = cast.expect("the bot casts By Force");
    assert_eq!(Some(targets.len() as u32), x, "one target per X");
    assert!(targets.iter().all(|t| targets.iter().filter(|u| *u == t).count() == 1), "no repeats");
}

/// CR 709.5 / 105.2 — a Room permanent has only its unlocked doors' mana
/// cost, so a locked Room is colorless and an unlocked red door makes it red.
/// The layer system seeded colors from both doors (a locked Spiked Corridor
/// // Torture Pit read red, and the requirement walker's colorless answer
/// tripped the colour gate's ratchet in a default-pilot 2-seat strict pod,
/// seed 261036 game 7).
#[test]
fn cr_709_5_a_locked_room_is_colorless_and_a_door_colors_it() {
    use crabomination::card::SelectionRequirement as R;
    use crabomination::mana::Color;
    let mut g = main_phase();
    let room = g.add_card_to_battlefield(0, catalog::spiked_corridor_torture_pit());
    let red = |g: &GameState| {
        (
            g.computed_permanent(room).expect("on the battlefield").colors.contains(Color::Red),
            g.evaluate_requirement_static(&R::HasColor(Color::Red), &Target::Permanent(room), 0, None),
        )
    };
    assert_eq!(red(&g), (false, false), "locked: no mana cost, no color");
    assert!(g.battlefield_find_mut(room).unwrap().unlock_room_door(false));
    assert_eq!(red(&g), (true, true), "the Corridor's {{R}} colors it");
}

/// CR 709.5 — a Room permanent has only its unlocked doors' names: a locked
/// Room has no name, and unlocking a door gives it that door's name only.
#[test]
fn cr_709_5_a_locked_door_has_no_name() {
    use crabomination::card::SelectionRequirement as R;
    let mut g = main_phase();
    let room = g.add_card_to_battlefield(0, catalog::spiked_corridor_torture_pit());
    let named = |g: &GameState, n: &str| {
        g.evaluate_requirement_static(&R::HasName(n.into()), &Target::Permanent(room), 0, None)
    };
    assert!(!named(&g, "Spiked Corridor") && !named(&g, "Torture Pit"), "locked: no name");
    assert!(g.battlefield_find_mut(room).unwrap().unlock_room_door(false));
    assert!(named(&g, "Spiked Corridor"), "the unlocked door's name");
    assert!(!named(&g, "Torture Pit"), "the locked door's name is still gone");
}

/// CR 709.4a — a split card has each half's name: Meddling Mage naming
/// "Hit" stops Hit // Run, where comparing the whole printed name let it
/// through.
#[test]
fn cr_709_4a_naming_one_half_of_a_split_card_names_the_card() {
    use crabomination::game::types::GameAction;
    let mut g = main_phase();
    let mage = g.add_card_to_battlefield(1, catalog::meddling_mage());
    g.battlefield_find_mut(mage).unwrap().named_card = Some("Hit".into());
    let hit = g.add_card_to_hand(0, catalog::hit_run());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let cast = g.perform_action(GameAction::CastSpell { card_id: hit, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None });
    assert!(matches!(cast, Err(GameError::SpellNameLocked)), "{cast:?}");
}

/// CR 400.1 / 608.2 — an answer of the wrong shape to a paused resolution is
/// rejected and the ask stands: the resolving card is held off-zone until the
/// resolution resumes, and dropping the ask deleted it (a fuzzed 8-seat pod,
/// seed 487016, answered Trap the Trespassers' option with a mode).
#[test]
fn cr_400_1_a_wrong_shape_answer_leaves_the_resolution_paused() {
    use crabomination::card::{CardDefinition, CardType};
    use crabomination::decision::{Decision, DecisionAnswer};
    use crabomination::effect::Value;
    use crabomination::game::types::GameAction;
    let mut g = main_phase();
    g.players[0].wants_ui = true;
    let spell = g.add_card_to_hand(0, CardDefinition {
        name: "A May",
        card_types: vec![CardType::Sorcery],
        effect: Effect::MayDo {
            description: "may".into(),
            body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(3) }),
        },
        ..Default::default()
    });
    g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    g.resolve_top_of_stack().expect("resolve");
    assert!(g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Mode(0))).is_err(), "a mode is not a yes/no");
    assert!(
        matches!(g.pending_decision.as_ref().map(|d| &d.decision), Some(Decision::OptionalTrigger { .. })),
        "the ask is posed again"
    );
    let life = g.players[0].life;
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Bool(true))).expect("answer");
    assert_eq!(g.players[0].life, life + 3);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == spell), "the spell finished resolving");
}

/// CR 603.2 — a cast trigger fires the moment the spell is cast, so a listener
/// the same resolution destroys after casting still triggered: a "whenever you
/// cast a spell, gain 2" creature sees the free cast before the sweep.
#[test]
fn cr_603_2_a_cast_listener_destroyed_later_in_the_resolution_saw_the_cast() {
    use crabomination::card::{CardDefinition, CardType, TriggeredAbility};
    use crabomination::effect::{EventKind, EventScope, EventSpec, Value};
    let mut g = main_phase();
    let watcher = g.add_card_to_battlefield(0, CardDefinition {
        name: "Cast Watcher",
        card_types: vec![CardType::Creature],
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        }],
        ..Default::default()
    });
    let freebie = g.add_card_to_exile(0, CardDefinition {
        name: "Freebie",
        card_types: vec![CardType::Sorcery],
        effect: Effect::Noop,
        ..Default::default()
    });
    let life = g.players[0].life;
    let effect = Effect::Seq(vec![
        Effect::CastExiledFree { what: Selector::ExactObjects(vec![freebie]) },
        Effect::Destroy { what: Selector::ExactObjects(vec![watcher]) },
    ]);
    resolve(&mut g, watcher, None, &effect);
    assert!(g.battlefield_find(watcher).is_none(), "destroyed");
    assert_eq!(g.players[0].life, life + 2, "the cast was seen before the sweep");
}

/// CR 122.2 — Me, the Immortal keeps its counters outside a hand or library,
/// so cast from the graveyard it enters already carrying a keyword counter,
/// and the whole-board keyword gate must see it (its debug audit fired in a
/// bot's look-ahead in a fuzzed 5-seat pod, seed 469022: a vigilance counter
/// on Me after a cleanup had cleared the gate).
#[test]
fn cr_122_2_a_card_entering_with_its_keyword_counters_arms_the_board_gate() {
    use crabomination::card::Keyword;
    use crabomination::game::types::GameAction;
    use crabomination::mana::Color;
    let mut g = main_phase();
    let me = g.add_card_to_graveyard(0, catalog::me_the_immortal());
    g.players[0].graveyard.iter_mut().find(|c| c.id == me).unwrap().keyword_counters.add(Keyword::Vigilance, 1);
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let _ = g.do_cleanup(&mut Vec::new());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for c in [Color::Green, Color::Blue, Color::Red] {
        g.players[0].mana_pool.add(c, 1);
    }
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastFlashback { card_id: me, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast Me from the graveyard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(me).is_some_and(|c| !c.keyword_counters.is_empty()), "entered with its counter");
    // A whole-board keyword ask (the cumulative-upkeep gate).
    let _ = g.process_cumulative_upkeep();
}
