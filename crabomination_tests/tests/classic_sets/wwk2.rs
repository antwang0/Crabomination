//! Worldwake gap closure — Traps, combat-damage riders and the rest.

use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn cast_alt(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    let r = g.perform_action(GameAction::CastSpellAlternative {
        card_id: id,
        pitch_card: None,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    drain_stack(g);
    r.map(|_| ())
}

/// Attack with `attacker` (seat 0's creature) into seat 1.
fn connect_with(g: &mut GameState, attacker: CardId, defender: usize) {
    g.clear_sickness(attacker);
    g.attacking = vec![Attack { attacker, target: AttackTarget::Player(defender) }];
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat damage");
    drain_stack(g);
}

// ── Traps ───────────────────────────────────────────────────────────────────

/// Ricochet Trap's {R} alternative cost unlocks only once an opponent has cast
/// a blue spell this turn.
#[test]
fn ricochet_trap_alt_cost_needs_a_blue_spell() {
    let try_alt = |blue_cast: bool| {
        let mut g = two_player_game();
        // Seat 1 casts a targeted spell — blue (Unsummon) or red (Bolt).
        let spell = if blue_cast {
            g.add_card_to_hand(1, catalog::unsummon())
        } else {
            g.add_card_to_hand(1, catalog::lightning_bolt())
        };
        let trap = g.add_card_to_hand(0, catalog::ricochet_trap());
        let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[1].mana_pool.add(Color::Blue, 1);
        g.players[1].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(victim)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("opponent's spell");
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Red, 1);
        cast_alt(&mut g, trap, Some(Target::Permanent(spell))).is_ok()
    };
    assert!(!try_alt(false), "a red spell doesn't turn the Trap on");
    assert!(try_alt(true), "a blue spell does");
}

/// Refraction Trap shields the whole team off one shared 3-damage pool and
/// fires the prevented damage back.
#[test]
fn refraction_trap_prevents_then_reflects() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
    let src = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let trap = g.add_card_to_hand(0, catalog::refraction_trap());
    // The prevented source is picked at resolution; reflect at `victim`.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![src])]));
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, trap, Some(Target::Permanent(victim)));
    // Two points at the bear, then two at the player: one shared 3-point pool.
    let mut ev = vec![];
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(bear), 2, Some(src), &mut ev);
    assert_eq!(g.battlefield_find(bear).map(|c| c.damage), Some(0), "the first two are prevented");
    let mut ev2 = vec![];
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Player(0), 2, Some(src), &mut ev2);
    assert_eq!(g.players[0].life, 19, "the pool had only 1 point left");
    assert_eq!(g.battlefield_find(victim).map(|c| c.damage), Some(3), "all 3 reflected")
}

/// Slingbow Trap's {G} unlocks off an attacking black flier and kills it.
#[test]
fn slingbow_trap_shoots_down_a_black_flier() {
    let mut g = two_player_game();
    let flier = g.add_card_to_battlefield(1, catalog::vampire_nighthawk());
    g.clear_sickness(flier);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: flier,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let trap = g.add_card_to_hand(0, catalog::slingbow_trap());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    cast_alt(&mut g, trap, Some(Target::Permanent(flier))).expect("trap for {G}");
    assert!(g.battlefield_find(flier).is_none(), "the Nighthawk is destroyed");
}

/// Stone Idol Trap costs {1} less per attacking creature.
#[test]
fn stone_idol_trap_discounts_per_attacker() {
    let mut g = two_player_game();
    let trap = g.add_card_to_hand(0, catalog::stone_idol_trap());
    let reduction = |g: &GameState| {
        let card = g.players[0].hand.iter().find(|c| c.id == trap).unwrap();
        crabomination::game::actions::cost_reduction_for_spell(g, 0, card, None)
    };
    assert_eq!(reduction(&g), 0, "no attackers, no discount");
    for _ in 0..3 {
        let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.attacking.push(Attack { attacker: a, target: AttackTarget::Player(0) });
    }
    assert_eq!(reduction(&g), 3, "three attackers shave three generic");
}

/// Permafrost Trap's {U} unlocks once an opponent's green creature has entered.
#[test]
fn permafrost_trap_watches_green_creatures() {
    let try_alt = |green: bool| {
        let mut g = two_player_game();
        let c = if green {
            g.add_card_to_battlefield(1, catalog::llanowar_elves())
        } else {
            g.add_card_to_battlefield(1, catalog::serra_angel())
        };
        // Record the entry the way the battlefield hop would.
        g.players[1].creatures_entered_this_turn.push(c);
        let trap = g.add_card_to_hand(0, catalog::permafrost_trap());
        g.players[0].mana_pool.add(Color::Blue, 1);
        cast_alt(&mut g, trap, Some(Target::Permanent(c))).is_ok()
    };
    assert!(!try_alt(false), "a white creature doesn't turn the Trap on");
    assert!(try_alt(true), "a green one does");
}

/// Nemesis Trap exiles the attacker and leaves you a copy.
#[test]
fn nemesis_trap_steals_a_copy() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.attacking.push(Attack { attacker, target: AttackTarget::Player(0) });
    let trap = g.add_card_to_hand(0, catalog::nemesis_trap());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, trap, Some(Target::Permanent(attacker)));
    assert!(g.battlefield_find(attacker).is_none(), "the original is exiled");
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Grizzly Bears"),
        "you get the token copy"
    );
}

// ── Combat-damage riders ────────────────────────────────────────────────────

/// Mordant Dragon repeats its combat damage onto a creature the damaged player
/// controls.
#[test]
fn mordant_dragon_repeats_its_hit() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let dragon = g.add_card_to_battlefield(0, catalog::mordant_dragon());
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
    connect_with(&mut g, dragon, 1);
    assert!(g.battlefield_find(victim).is_none(), "5 power repeated kills the 4/4");
}

/// Thada Adel exiles an artifact out of the damaged player's library and lets
/// you play it this turn.
#[test]
fn thada_adel_steals_an_artifact() {
    let mut g = two_player_game();
    let thada = g.add_card_to_battlefield(0, catalog::thada_adel_acquisitor());
    let art = g.add_card_to_library(1, catalog::sol_ring());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(art))]));
    connect_with(&mut g, thada, 1);
    assert!(g.exile.iter().any(|c| c.id == art), "the Sol Ring is exiled");
    assert!(
        g.exile.iter().find(|c| c.id == art).unwrap().may_play_until.is_some(),
        "and you may play it"
    );
}

/// Hammer of Ruin destroys an Equipment the damaged player controls.
#[test]
fn hammer_of_ruin_smashes_equipment() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hammer = g.add_card_to_battlefield(0, catalog::hammer_of_ruin());
    g.battlefield_find_mut(hammer).unwrap().attached_to = Some(bear);
    let theirs = g.add_card_to_battlefield(1, catalog::hammer_of_ruin());
    connect_with(&mut g, bear, 1);
    assert!(g.battlefield_find(theirs).is_none(), "their Equipment is destroyed");
}

/// Wrexial free-casts an instant out of the damaged player's graveyard.
#[test]
fn wrexial_casts_from_their_graveyard() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(
        std::iter::repeat_with(|| DecisionAnswer::Bool(true)).take(3),
    ));
    let wrexial = g.add_card_to_battlefield(0, catalog::wrexial_the_risen_deep());
    let bolt = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    connect_with(&mut g, wrexial, 1);
    assert!(
        !g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "the Bolt left their graveyard"
    );
}

// ── Multikicker ─────────────────────────────────────────────────────────────

/// Spell Contortion draws one card per kick (the count now reaches a resolving
/// *spell*, not just a permanent).
#[test]
fn spell_contortion_draws_per_kick() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    let spell = g.add_card_to_hand(0, catalog::spell_contortion());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.players[0].mana_pool.add_colorless(4);
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpellMultikicked {
        card_id: spell,
        times: 2,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast kicked twice");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before - 1 + 2, "spent the spell, drew two");
}

/// Voyager Drake grants flying to one creature per kick.
#[test]
fn voyager_drake_grants_flying_per_kick() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let drake = g.add_card_to_hand(0, catalog::voyager_drake());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpellMultikicked {
        card_id: drake,
        times: 2,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast kicked twice");
    drain_stack(&mut g);
    for id in [a, b] {
        assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Flying));
    }
}

/// Rumbling Aftershocks pings for the kick count of the spell you just cast.
#[test]
fn rumbling_aftershocks_scales_with_kicks() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rumbling_aftershocks());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let elite = g.add_card_to_hand(0, catalog::enclave_elite());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpellMultikicked {
        card_id: elite,
        times: 2,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast kicked twice");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "two kicks, two damage");
}

/// Marshal's Anthem is an anthem that reanimates one creature per kick.
#[test]
fn marshals_anthem_reanimates_per_kick() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let anthem = g.add_card_to_hand(0, catalog::marshals_anthem());
    g.players[0].mana_pool.add(Color::White, 3);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpellMultikicked {
        card_id: anthem,
        times: 1,
        target: Some(Target::Permanent(dead)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast kicked once");
    drain_stack(&mut g);
    let bear = g.battlefield_find(dead).expect("reanimated");
    assert_eq!(g.computed_permanent(bear.id).unwrap().power, 3, "2/2 plus the anthem");
}

// ── The rest ────────────────────────────────────────────────────────────────

/// Quest for Renewal only unlocks its off-turn untap at four quest counters.
#[test]
fn quest_for_renewal_needs_four_counters() {
    let untaps_off_turn = |counters: u32| {
        let mut g = two_player_game();
        let quest = g.add_card_to_battlefield(0, catalog::quest_for_renewal());
        g.battlefield_find_mut(quest).unwrap().add_counters(CounterType::Quest, counters);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().tapped = true;
        g.active_player_idx = 1;
        g.do_untap();
        !g.battlefield_find(bear).unwrap().tapped
    };
    assert!(!untaps_off_turn(3), "three counters isn't enough");
    assert!(untaps_off_turn(4), "four unlocks it");
}

/// Terastodon pays each victim's controller a 3/3 Elephant.
#[test]
fn terastodon_pays_for_what_it_breaks() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let theirs = g.add_card_to_battlefield(1, catalog::sol_ring());
    let tera = g.add_card_to_hand(0, catalog::terastodon());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(6);
    cast(&mut g, tera, None);
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "the Sol Ring is destroyed");
    assert!(
        g.battlefield.iter().any(|c| c.controller == 1 && c.definition.name == "Elephant"),
        "its controller gets the Elephant"
    );
}

/// Kazuul taxes each attacker {3} or mints an Ogre.
#[test]
fn kazuul_taxes_attackers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kazuul_tyrant_of_the_cliffs());
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Ogre"),
        "they declined to pay {{3}}"
    );
}

/// Treasure Hunt takes every card it revealed, lands included.
#[test]
fn treasure_hunt_takes_the_whole_reveal() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_library(0, catalog::lightning_bolt());
    let hunt = g.add_card_to_hand(0, catalog::treasure_hunt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let before = g.players[0].hand.len();
    cast(&mut g, hunt, None);
    assert_eq!(g.players[0].hand.len(), before - 1 + 4, "three lands plus the nonland");
}

/// Urge to Feed grows every Vampire you tap for it.
#[test]
fn urge_to_feed_feeds_the_vampires() {
    let mut g = two_player_game();
    let vamp = g.add_card_to_battlefield(0, catalog::vampire_nighthawk());
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
    let urge = g.add_card_to_hand(0, catalog::urge_to_feed());
    g.players[0].mana_pool.add(Color::Black, 2);
    cast(&mut g, urge, Some(Target::Permanent(victim)));
    let v = g.battlefield_find(vamp).expect("the Nighthawk lives");
    assert!(v.tapped, "it tapped to feed");
    assert_eq!(v.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.computed_permanent(victim).unwrap().power, 1, "4/4 minus 3/3");
}

/// Horizon Drake can't be targeted or damaged by lands.
#[test]
fn horizon_drake_has_protection_from_lands() {
    let mut g = two_player_game();
    let drake = g.add_card_to_battlefield(0, catalog::horizon_drake());
    let cp = g.computed_permanent(drake).unwrap();
    assert!(cp.keywords.contains(&Keyword::ProtectionFromCardType(CardType::Land)));
}

/// Summit Apes has menace only while you control a Mountain.
#[test]
fn summit_apes_needs_a_mountain() {
    let mut g = two_player_game();
    let apes = g.add_card_to_battlefield(0, catalog::summit_apes());
    assert!(!g.computed_permanent(apes).unwrap().keywords.contains(&Keyword::Menace));
    g.add_card_to_battlefield(0, catalog::mountain());
    assert!(g.computed_permanent(apes).unwrap().keywords.contains(&Keyword::Menace));
}

/// Terra Eternal makes every land indestructible, both players'.
#[test]
fn terra_eternal_protects_all_lands() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::terra_eternal());
    let theirs = g.add_card_to_battlefield(1, catalog::mountain());
    assert!(
        g.computed_permanent(theirs).unwrap().keywords.contains(&Keyword::Indestructible),
        "their land is indestructible too"
    );
}

/// Wind Zendikon animates the land and hands the card back when it dies.
#[test]
fn wind_zendikon_returns_the_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::island());
    let aura = g.add_card_to_hand(0, catalog::wind_zendikon());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, aura, Some(Target::Permanent(land)));
    let cp = g.computed_permanent(land).expect("still there");
    assert_eq!((cp.power, cp.toughness), (2, 2));
    assert!(cp.keywords.contains(&Keyword::Flying));
    let mut ev = vec![];
    g.destroy_permanent(land, false, &mut ev);
    ev.append(&mut g.check_state_based_actions());
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == land), "the land card went back to hand");
}

/// Feral Contest grows your creature and forces the other one to block it.
#[test]
fn feral_contest_forces_the_block() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let contest = g.add_card_to_hand(0, catalog::feral_contest());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: contest,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(theirs).unwrap().must_block, Some(mine));
}

/// Mire's Toll reveals one card per Swamp and takes one of them.
#[test]
fn mires_toll_scales_with_swamps() {
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::swamp());
    }
    g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::island());
    let toll = g.add_card_to_hand(0, catalog::mires_toll());
    g.players[0].mana_pool.add(Color::Black, 1);
    cast(&mut g, toll, Some(Target::Player(1)));
    assert_eq!(g.players[1].graveyard.len(), 1, "one of the two revealed is discarded");
}

/// Agadeem Occultist reanimates only up to your Ally count.
#[test]
fn agadeem_occultist_is_capped_by_allies() {
    let mut g = two_player_game();
    let occ = g.add_card_to_battlefield(0, catalog::agadeem_occultist());
    g.clear_sickness(occ);
    // One Ally (itself) — a 2-drop is out of reach, a 1-drop isn't.
    let big = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let small = g.add_card_to_graveyard(1, catalog::llanowar_elves());
    let act = |g: &mut GameState, t: CardId| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: occ,
            ability_index: 0,
            target: Some(Target::Permanent(t)),
            additional_targets: vec![],
            x_value: None,
        })
    };
    assert!(act(&mut g, big).is_err(), "mana value 2 > one Ally");
    assert!(act(&mut g, small).is_ok(), "mana value 1 is in range");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(small).map(|c| c.controller), Some(0), "stolen");
}

/// Vastwood Animist animates a land at your Ally count.
#[test]
fn vastwood_animist_scales_with_allies() {
    let mut g = two_player_game();
    let animist = g.add_card_to_battlefield(0, catalog::vastwood_animist());
    g.clear_sickness(animist);
    g.add_card_to_battlefield(0, catalog::hada_freeblade());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: animist,
        ability_index: 0,
        target: Some(Target::Permanent(land)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("animate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(land).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "two Allies");
    assert!(cp.card_types.contains(&CardType::Land), "it's still a land");
}

/// Razor Boomerang pings and bounces itself back to hand.
#[test]
fn razor_boomerang_comes_back() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let boom = g.add_card_to_battlefield(0, catalog::razor_boomerang());
    g.battlefield_find_mut(boom).unwrap().attached_to = Some(bear);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("throw it");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19);
    assert!(g.players[0].hand.iter().any(|c| c.id == boom), "it returns to hand");
}

/// Tomb Hex is −2/−2, or −4/−4 with landfall.
#[test]
fn tomb_hex_doubles_on_landfall() {
    let shrink = |landfall: bool| {
        let mut g = two_player_game();
        if landfall {
            let l = g.add_card_to_battlefield(0, catalog::swamp());
            let turn = g.turn_number;
            g.battlefield_find_mut(l).unwrap().entered_turn = Some(turn);
        }
        let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
        let hex = g.add_card_to_hand(0, catalog::tomb_hex());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, hex, Some(Target::Permanent(victim)));
        g.computed_permanent(victim).map(|c| c.power)
    };
    assert_eq!(shrink(false), Some(2), "4/4 minus 2/2");
    assert_eq!(shrink(true), None, "4/4 minus 4/4 dies to the SBA");
}

/// Sejiri Steppe and Smoldering Spires enter tapped with their ETB riders.
#[test]
fn wwk_utility_lands_enter_tapped_with_riders() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    let steppe = g.add_card_to_hand(0, catalog::sejiri_steppe());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(steppe)).expect("play");
    drain_stack(&mut g);
    assert!(g.battlefield_find(steppe).unwrap().tapped, "enters tapped");
    assert!(
        g.computed_permanent(bear).unwrap().keywords.iter().any(|k| matches!(
            k,
            Keyword::Protection(Color::Red)
        )),
        "the creature gained protection from the chosen color"
    );

    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spires = g.add_card_to_hand(0, catalog::smoldering_spires());
    g.players[0].lands_played_this_turn = 0;
    g.perform_action(GameAction::PlayLand(spires)).expect("play");
    drain_stack(&mut g);
    assert!(g.computed_permanent(theirs).unwrap().keywords.contains(&Keyword::CantBlock));
}

/// The landfall untappers (Scrib Nibblers, Tideforce Elemental) untap on a land
/// drop, and Scrib Nibblers gains life off an exiled land.
#[test]
fn wwk_landfall_untappers_untap() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(
        std::iter::repeat_with(|| DecisionAnswer::Bool(true)).take(4),
    ));
    let nibblers = g.add_card_to_battlefield(0, catalog::scrib_nibblers());
    let tideforce = g.add_card_to_battlefield(0, catalog::tideforce_elemental());
    for id in [nibblers, tideforce] {
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    g.add_card_to_library(1, catalog::island());
    let land = g.add_card_to_hand(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(land)).expect("play");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(nibblers).unwrap().tapped);
    assert!(!g.battlefield_find(tideforce).unwrap().tapped);
}

/// Vapor Snare steals the creature and asks for a land each upkeep.
#[test]
fn vapor_snare_steals_then_taxes() {
    let mut g = two_player_game();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::island());
    let snare = g.add_card_to_hand(0, catalog::vapor_snare());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, snare, Some(Target::Permanent(theirs)));
    assert_eq!(g.battlefield_find(theirs).map(|c| c.controller), Some(0), "you control it");
    // Decline the bounce: the Aura is sacrificed.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(snare).is_none(), "the Aura is sacrificed");
}

/// The vanilla-ish WWK gap creatures ship with their printed stats.
#[test]
fn wwk2_stat_lines() {
    let table: &[(crabomination::card::CardDefinition, i32, i32)] = &[
        (catalog::shoreline_salvager(), 3, 3),
        (catalog::slavering_nulls(), 2, 1),
        (catalog::surrakar_banisher(), 3, 3),
        (catalog::talus_paladin(), 2, 3),
        (catalog::tuktuk_scrapper(), 2, 2),
        (catalog::jwari_shapeshifter(), 0, 0),
    ];
    for (def, p, t) in table {
        assert_eq!((def.power, def.toughness), (*p, *t), "{}", def.name);
    }
}

