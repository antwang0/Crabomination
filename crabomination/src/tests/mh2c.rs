//! Functionality tests for `catalog::sets::decks::mh2c` — MH2 sweep batch 4.

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::{Target, TurnStep};
use crate::game::*;
use crate::mana::Color;

fn resolve_spell(g: &mut GameState, def: crate::card::CardDefinition, targets: Vec<Target>) {
    let mut ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = targets;
    let events = g.resolve_effect(&def.effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
}

fn activate(g: &mut GameState, id: crate::card::CardId, idx: usize, target: Option<Target>) {
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: idx, target, additional_targets: vec![], x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Aeromoeba swaps its P/T for the turn off a discard.
#[test]
fn aeromoeba_switches_pt() {
    let mut g = two_player_game();
    let moeba = g.add_card_to_battlefield(0, catalog::aeromoeba());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    activate(&mut g, moeba, 0, None);
    let cp = g.computed_permanent(moeba).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 2), "2/4 switched to 4/2");
}

/// Archfiend of Sorrows shrinks the opposing board on entry.
#[test]
fn archfiend_shrinks_opponents() {
    let mut g = two_player_game();
    let theirs = g.add_card_to_battlefield(1, catalog::serra_angel());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let fiend = g.add_card_to_hand(0, catalog::archfiend_of_sorrows());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.priority.player_with_priority = 0;
    cast(&mut g, fiend);
    assert_eq!(g.computed_permanent(theirs).unwrap().power, 2, "4/4 → 2/2");
    assert_eq!(g.computed_permanent(mine).unwrap().power, 2, "mine untouched");
}

/// Archfiend unearths from the graveyard for {3}{B}{B}.
#[test]
fn archfiend_unearths() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::archfiend_of_sorrows());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    activate(&mut g, dead, 0, None);
    let fiend = g.battlefield_find(dead).expect("unearthed onto the battlefield");
    assert!(fiend.has_keyword(&Keyword::Haste), "unearth grants haste");
}

/// Batterbone lands as a living weapon: a 0/0 Germ wearing a +1/+1 stick.
#[test]
fn batterbone_living_weapon() {
    let mut g = two_player_game();
    let bone = g.add_card_to_hand(0, catalog::batterbone());
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    cast(&mut g, bone);
    let germ = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Phyrexian Germ")
        .expect("germ minted");
    let cp = g.computed_permanent(germ.id).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "0/0 + equip bonus");
    assert!(cp.keywords.contains(&Keyword::Lifelink));
}

/// Blacksmith's Skill: artifact creatures get the +2/+2 rider, others don't.
#[test]
fn blacksmiths_skill_rider() {
    let mut g = two_player_game();
    let golem = g.add_card_to_battlefield(0, catalog::bottle_golems());
    resolve_spell(&mut g, catalog::blacksmiths_skill(), vec![Target::Permanent(golem)]);
    let cp = g.computed_permanent(golem).unwrap();
    assert!(cp.keywords.contains(&Keyword::Indestructible));
    assert_eq!(cp.power, 5, "3/3 artifact creature got +2/+2");

    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    resolve_spell(&mut g, catalog::blacksmiths_skill(), vec![Target::Permanent(bear)]);
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::Hexproof));
    assert_eq!(cp.power, 2, "non-artifact skips the pump");
}

/// Blessed Respite reshuffles the graveyard and fogs the turn.
#[test]
fn blessed_respite_shuffles_and_fogs() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let before_lib = g.players[0].library.len();
    resolve_spell(&mut g, catalog::blessed_respite(), vec![Target::Player(0)]);
    assert!(g.players[0].graveyard.is_empty());
    assert_eq!(g.players[0].library.len(), before_lib + 1);
    assert!(g.prevent_combat_damage_this_turn, "fog armed");
}

/// Bottle Golems' death pays out its (buffed) power in life.
#[test]
fn bottle_golems_dies_gains_power_life() {
    let mut g = two_player_game();
    let golem = g.add_card_to_battlefield(0, catalog::bottle_golems());
    let life = g.players[0].life;
    resolve_spell(&mut g, catalog::blacksmiths_skill(), vec![Target::Permanent(golem)]);
    // Indestructible from the pump — kill via sacrifice instead.
    let ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g
        .resolve_effect(
            &crate::effect::Effect::Sacrifice {
                who: crate::effect::Selector::You,
                count: crate::effect::Value::ONE,
                filter: crate::card::SelectionRequirement::Creature,
            },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 5, "gained buffed power (5)");
}

/// Cabal Initiate: threshold turns on the +1/+2.
#[test]
fn cabal_initiate_threshold() {
    let mut g = two_player_game();
    let init = g.add_card_to_battlefield(0, catalog::cabal_initiate());
    assert_eq!(g.computed_permanent(init).unwrap().power, 2);
    for _ in 0..7 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let cp = g.computed_permanent(init).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "threshold +1/+2");
}

/// Clattering Augur returns itself from the graveyard.
#[test]
fn clattering_augur_recurs() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::clattering_augur());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, dead, 0, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "back to hand");
}

/// Crack Open pops an artifact and mints a Treasure.
#[test]
fn crack_open() {
    let mut g = two_player_game();
    let relic = g.add_card_to_battlefield(1, catalog::batterbone());
    resolve_spell(&mut g, catalog::crack_open(), vec![Target::Permanent(relic)]);
    assert!(g.battlefield_find(relic).is_none(), "artifact destroyed");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"));
}

/// Etherium Spinner mints a Thopter only off MV-4+ spells.
#[test]
fn etherium_spinner_mv_gate() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::etherium_spinner());
    let cheap = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.priority.player_with_priority = 0;
    cast(&mut g, cheap);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Thopter"), "MV 2: no Thopter");
    let big = g.add_card_to_hand(0, catalog::serra_angel());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, big);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Thopter"), "MV 5: Thopter");
}

/// Furious (the right half) sweeps the ground but spares flyers.
#[test]
fn furious_spares_flyers() {
    let mut g = two_player_game();
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let flyer = g.add_card_to_battlefield(1, catalog::serra_angel());
    let def = catalog::fast_furious();
    let right = def.split.as_ref().unwrap().right.effect.clone();
    let ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g.resolve_effect(&right, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    g.check_state_based_actions();
    assert!(g.battlefield_find(ground).is_none(), "2/2 dies to 3");
    assert!(g.battlefield_find(flyer).is_some(), "flyer untouched");
}

/// Feast of Sanity converts each discard into a ping + 1 life.
#[test]
fn feast_of_sanity_discard_payoff() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::feast_of_sanity());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let life = g.players[0].life;
    let opp_life = g.players[1].life;
    let ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g
        .resolve_effect(
            &crate::effect::Effect::Discard {
                who: crate::effect::Selector::You,
                amount: crate::effect::Value::ONE,
                random: false,
            },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "gained 1");
    assert_eq!(g.players[1].life, opp_life - 1, "ping auto-aimed at the opponent");
}

/// Filigree Attendant's power tracks your artifact count.
#[test]
fn filigree_attendant_cda() {
    let mut g = two_player_game();
    let fil = g.add_card_to_battlefield(0, catalog::filigree_attendant());
    // Counts itself (an artifact creature).
    assert_eq!(g.computed_permanent(fil).unwrap().power, 1);
    g.add_card_to_battlefield(0, catalog::batterbone());
    g.add_card_to_battlefield(0, catalog::parcel_myr());
    let cp = g.computed_permanent(fil).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "3 artifacts / fixed 3 toughness");
}

/// Flame Blitz torches every planeswalker at your end step.
#[test]
fn flame_blitz_hits_planeswalkers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::flame_blitz());
    let pw = g.add_card_to_battlefield(1, catalog::vivien_reid());
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(pw).is_none(), "5 damage kills a 5-loyalty walker");
}

/// Fodder Tosser turns spare cards into 2-damage lobs.
#[test]
fn fodder_tosser() {
    let mut g = two_player_game();
    let tosser = g.add_card_to_battlefield(0, catalog::fodder_tosser());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let opp_life = g.players[1].life;
    activate(&mut g, tosser, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, opp_life - 2);
    assert!(g.players[0].hand.is_empty(), "discard paid");
}

/// Foundation Breaker's evoke cost is registered and the ETB pops an artifact.
#[test]
fn foundation_breaker_etb() {
    let mut g = two_player_game();
    assert!(catalog::foundation_breaker().alternative_cost.is_some());
    let relic = g.add_card_to_battlefield(1, catalog::fodder_tosser());
    let breaker = g.add_card_to_hand(0, catalog::foundation_breaker());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, breaker);
    assert!(g.battlefield_find(relic).is_none(), "artifact destroyed via MayDo (auto-yes)");
}

/// Landscaper Colos bottoms a card from an opponent's graveyard.
#[test]
fn landscaper_colos_bottoms() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let colos = g.add_card_to_hand(0, catalog::landscaper_colos());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: colos,
        target: Some(Target::Permanent(dead)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast with graveyard target");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.is_empty(), "graveyard emptied");
    assert_eq!(g.players[1].library.first().map(|c| c.id), Some(dead), "on the bottom");
}

/// Lightning Spear sacrifices for a 3-damage bolt.
#[test]
fn lightning_spear_bolt() {
    let mut g = two_player_game();
    let spear = g.add_card_to_battlefield(0, catalog::lightning_spear());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_life = g.players[1].life;
    activate(&mut g, spear, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, opp_life - 3);
    assert!(g.battlefield_find(spear).is_none(), "sacrificed");
}

/// Loathsome Curator exploits into a small-creature kill.
#[test]
fn loathsome_curator_exploit_kill() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let curator = g.add_card_to_hand(0, catalog::loathsome_curator());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, curator);
    assert!(g.battlefield_find(fodder).is_none(), "exploited");
    assert!(g.battlefield_find(victim).is_none(), "MV≤3 creature destroyed");
}

/// Moderation locks you to one spell and pays a card for it.
#[test]
fn moderation_draw_and_lock() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::moderation());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    cast(&mut g, bear);
    assert_eq!(g.players[0].hand.len(), hand_before, "cast one, drew one");
    let second = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: second, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .is_err(),
        "second spell locked"
    );
}

/// Monoskelion converts its counter into a ping.
#[test]
fn monoskelion_ping() {
    let mut g = two_player_game();
    let skel = g.add_card_to_battlefield(0, catalog::monoskelion());
    g.battlefield_find_mut(skel).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_life = g.players[1].life;
    activate(&mut g, skel, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, opp_life - 1);
    assert_eq!(
        g.battlefield_find(skel).unwrap().counter_count(CounterType::PlusOnePlusOne),
        0,
        "counter paid"
    );
}

/// Piru sacrifices itself when the upkeep toll goes unpaid, and its death
/// torches nonlegendary creatures.
#[test]
fn piru_upkeep_and_death_sweep() {
    let mut g = two_player_game();
    let piru = g.add_card_to_battlefield(0, catalog::piru_the_volatile());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let legend = g.add_card_to_battlefield(1, catalog::chatterfang_squirrel_general());
    // No mana available: the MayPay auto-declines and Piru is sacrificed.
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(piru).is_none(), "sacrificed at upkeep");
    assert!(g.battlefield_find(bear).is_none(), "nonlegendary swept for 7");
    assert!(g.battlefield_find(legend).is_some(), "legend survives");
}

/// Rishadan Dockhand taps a land.
#[test]
fn rishadan_dockhand_taps_land() {
    let mut g = two_player_game();
    let hand = g.add_card_to_battlefield(0, catalog::rishadan_dockhand());
    g.clear_sickness(hand);
    let land = g.add_card_to_battlefield(1, catalog::island());
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, hand, 0, Some(Target::Permanent(land)));
    assert!(g.battlefield_find(land).unwrap().tapped);
}

/// Said returns an instant/sorcery from the graveyard; Done stuns two.
#[test]
fn said_done_halves() {
    let mut g = two_player_game();
    let spell = g.add_card_to_graveyard(0, catalog::blessed_respite());
    resolve_spell(&mut g, catalog::said_done(), vec![Target::Permanent(spell)]);
    assert!(g.players[0].hand.iter().any(|c| c.id == spell), "Said regrows");

    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let def = catalog::said_done();
    let right = def.split.as_ref().unwrap().right.effect.clone();
    let mut ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(a)];
    let events = g.resolve_effect(&right, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    let tapped = g.battlefield_find(a).unwrap();
    assert!(tapped.tapped, "Done taps");
    assert_eq!(tapped.counter_count(CounterType::Stun), 1, "and stuns");
}

/// Slag Strider's affinity discounts and its sac-ability pings.
#[test]
fn slag_strider() {
    let mut g = two_player_game();
    let d = catalog::slag_strider();
    assert_eq!(d.affinity_filter, Some(crate::card::SelectionRequirement::Artifact));
    let strider = g.add_card_to_battlefield(0, catalog::slag_strider());
    g.add_card_to_battlefield(0, catalog::parcel_myr());
    g.players[0].mana_pool.add_colorless(1);
    let opp_life = g.players[1].life;
    activate(&mut g, strider, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, opp_life - 1);
}

/// Storm God's Oracle pumps +1/-1 and pings 3 when it dies.
#[test]
fn storm_gods_oracle() {
    let mut g = two_player_game();
    let oracle = g.add_card_to_battlefield(0, catalog::storm_gods_oracle());
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, oracle, 0, None);
    let cp = g.computed_permanent(oracle).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "+1/-1");
}

/// Vedalken Infiltrator turns on with metalcraft.
#[test]
fn vedalken_infiltrator_metalcraft() {
    let mut g = two_player_game();
    let rogue = g.add_card_to_battlefield(0, catalog::vedalken_infiltrator());
    assert_eq!(g.computed_permanent(rogue).unwrap().power, 1);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::parcel_myr());
    }
    assert_eq!(g.computed_permanent(rogue).unwrap().power, 2, "metalcraft +1/+0");
}

/// Viashino Lashclaw hastes the team.
#[test]
fn viashino_lashclaw_haste() {
    let mut g = two_player_game();
    let lash = g.add_card_to_battlefield(0, catalog::viashino_lashclaw());
    g.clear_sickness(lash);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    activate(&mut g, lash, 0, None);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste));
}

/// World-Weary's -4/-4 kills a 4-toughness body; landcycling is registered.
#[test]
fn world_weary() {
    let mut g = two_player_game();
    assert!(catalog::world_weary()
        .keywords
        .iter()
        .any(|k| matches!(k, Keyword::Typecycling(_))));
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let aura = g.add_card_to_hand(0, catalog::world_weary());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    cast_at(&mut g, aura, Target::Permanent(angel));
    g.check_state_based_actions();
    assert!(g.battlefield_find(angel).is_none(), "4/4 at -4/-4 dies");
}

/// Batch-4 stat spot checks.
#[test]
fn batch4_stats() {
    assert!(catalog::battle_plan().keywords.iter().any(|k| matches!(k, Keyword::Typecycling(_))));
    assert!(catalog::flame_blitz().keywords.iter().any(|k| matches!(k, Keyword::Cycling(_))));
    assert!(catalog::vedalken_infiltrator().keywords.contains(&Keyword::Unblockable));
    assert!(catalog::piru_the_volatile().keywords.contains(&Keyword::Lifelink));
    let d = catalog::fast_furious();
    assert!(d.split.is_some());
    assert_eq!(catalog::monoskelion().cost.cmc(), 2);
    assert_eq!(catalog::batterbone().cost.cmc(), 2);
}
