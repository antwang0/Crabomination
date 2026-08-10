//! Urza's Saga (USG) gap closure, second wave.

use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::game::types::{Attack, AttackTarget};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target};
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

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 10);
    }
    g.players[seat].mana_pool.add_colorless(10);
}

/// The cycling lands tap for their colour and enter tapped (Blasted
/// Landscape, the colourless one, does not).
#[test]
fn usg_cycling_lands_enter_tapped_except_the_colorless_one() {
    for f in [
        catalog::drifting_meadow as fn() -> crabomination::card::CardDefinition,
        catalog::polluted_mire,
        catalog::remote_isle,
        catalog::slippery_karst,
        catalog::smoldering_crater,
    ] {
        let def = f();
        assert!(
            def.keywords.iter().any(|k| matches!(k, Keyword::Cycling(c) if c.cmc() == 2)),
            "{} is missing Cycling {{2}}",
            def.name
        );
        assert!(def.subtypes.land_types.is_empty(), "{} has no basic land type", def.name);
        let mut g = two_player_game();
        let land = g.add_card_to_hand(0, f());
        g.perform_action(GameAction::PlayLand(land)).expect("play");
        assert!(g.battlefield_find(land).unwrap().tapped, "{} enters tapped", def.name);
    }
    let mut g = two_player_game();
    let land = g.add_card_to_hand(0, catalog::blasted_landscape());
    g.perform_action(GameAction::PlayLand(land)).expect("play");
    assert!(!g.battlefield_find(land).unwrap().tapped);
}

/// The echo wave all carry their printed echo cost.
#[test]
fn usg2_echo_bodies_carry_echo() {
    for (f, cmc) in [
        (catalog::citanul_centaurs as fn() -> crabomination::card::CardDefinition, 4),
        (catalog::herald_of_serra, 4),
        (catalog::lightning_dragon, 4),
        (catalog::shivan_raptor, 3),
        (catalog::viashino_outrider, 3),
        (catalog::vug_lizard, 3),
        (catalog::winding_wurm, 5),
    ] {
        let def = f();
        assert!(
            def.keywords.iter().any(|k| matches!(k, Keyword::Echo(c) if c.cmc() == cmc)),
            "{} is missing Echo {{{cmc}}}",
            def.name
        );
    }
}

/// Serra Avatar's body tracks its controller's life total (CR 604.3).
#[test]
fn serra_avatar_is_your_life_total() {
    let mut g = two_player_game();
    let avatar = g.add_card_to_battlefield(0, catalog::serra_avatar());
    let cp = g.computed_permanent(avatar).unwrap();
    assert_eq!((cp.power, cp.toughness), (g.players[0].life, g.players[0].life));
    g.players[0].life = 7;
    let cp = g.computed_permanent(avatar).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 7));
}

/// Treefolk Seedlings keeps its printed power and counts Forests for toughness.
#[test]
fn treefolk_seedlings_toughness_is_your_forests() {
    let mut g = two_player_game();
    let seedlings = g.add_card_to_battlefield(0, catalog::treefolk_seedlings());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let cp = g.computed_permanent(seedlings).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3));
}

/// Acidic Soil burns each player for their own land count.
#[test]
fn acidic_soil_scales_per_player() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    g.add_card_to_battlefield(1, catalog::island());
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    let spell = g.add_card_to_hand(0, catalog::acidic_soil());
    mana(&mut g, 0);
    cast(&mut g, spell, None);
    assert_eq!(g.players[0].life, l0 - 4);
    assert_eq!(g.players[1].life, l1 - 1);
}

/// Disorder hits every white creature and only the players who control one.
#[test]
fn disorder_spares_players_without_white_creatures() {
    let mut g = two_player_game();
    let white = g.add_card_to_battlefield(1, catalog::serra_zealot()); // 1/1 white
    let red = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    let spell = g.add_card_to_hand(0, catalog::disorder());
    mana(&mut g, 0);
    cast(&mut g, spell, None);
    assert!(g.battlefield_find(white).is_none());
    assert!(g.battlefield_find(red).is_some());
    assert_eq!(g.players[0].life, l0, "no white creature, no damage");
    assert_eq!(g.players[1].life, l1 - 2);
}

/// Falter's restriction is continuous (CR 611.2c) — a creature that arrives
/// after it resolves still can't block.
#[test]
fn falter_covers_creatures_that_arrive_later() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::falter());
    mana(&mut g, 0);
    cast(&mut g, spell, None);
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let flier = g.add_card_to_battlefield(1, catalog::pegasus_charger());
    assert!(g.computed_permanent(ground).unwrap().keywords.contains(&Keyword::CantBlock));
    assert!(!g.computed_permanent(flier).unwrap().keywords.contains(&Keyword::CantBlock));
}

/// Curfew makes every player bounce one of their own creatures.
#[test]
fn curfew_bounces_one_creature_per_player() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spare = g.add_card_to_battlefield(1, catalog::serra_zealot());
    let spell = g.add_card_to_hand(0, catalog::curfew());
    mana(&mut g, 0);
    cast(&mut g, spell, None);
    assert!(g.battlefield_find(mine).is_none());
    assert_eq!(
        [theirs, spare].iter().filter(|id| g.battlefield_find(**id).is_some()).count(),
        1,
        "exactly one of theirs went home"
    );
}

/// Bravado counts the enchanted creature's *other* friends only.
#[test]
fn bravado_excludes_its_own_host() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.add_card_to_battlefield(0, catalog::serra_zealot());
    let aura = g.add_card_to_hand(0, catalog::bravado());
    mana(&mut g, 0);
    cast(&mut g, aura, Some(Target::Permanent(host)));
    let cp = g.computed_permanent(host).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "one other creature, not two");
}

/// The Halo/Embrace recursion cycle returns to hand when the Aura dies.
#[test]
fn brilliant_halo_returns_to_its_owners_hand() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::brilliant_halo());
    mana(&mut g, 0);
    cast(&mut g, aura, Some(Target::Permanent(host)));
    assert_eq!(g.computed_permanent(host).unwrap().toughness, 4);
    g.destroy_permanent(aura, false, &mut vec![]);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Brilliant Halo"));
}

/// Torch Song banks verse counters each upkeep and cashes them in for damage.
#[test]
fn torch_song_burns_for_its_verse_counters() {
    let mut g = two_player_game();
    let song = g.add_card_to_battlefield(0, catalog::torch_song());
    for _ in 0..3 {
        g.battlefield_find_mut(song).unwrap().add_counters(CounterType::Verse, 1);
    }
    mana(&mut g, 0);
    let life = g.players[1].life;
    activate(&mut g, song, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 3);
    assert!(g.battlefield_find(song).is_none(), "the sacrifice was paid");
}

/// Wirecat sulks while any enchantment is on the battlefield (CR 611.2 — the
/// gate is live, so removing the enchantment frees it).
#[test]
fn wirecat_is_gated_on_a_live_enchantment() {
    let mut g = two_player_game();
    let cat = g.add_card_to_battlefield(0, catalog::wirecat());
    assert!(!g.computed_permanent(cat).unwrap().keywords.contains(&Keyword::CantAttack));
    let ench = g.add_card_to_battlefield(1, catalog::bedlam());
    let kws = g.computed_permanent(cat).unwrap().keywords.clone();
    assert!(kws.contains(&Keyword::CantAttack) && kws.contains(&Keyword::CantBlock));
    g.destroy_permanent(ench, false, &mut vec![]);
    assert!(!g.computed_permanent(cat).unwrap().keywords.contains(&Keyword::CantAttack));
}

/// The Opal-style enchantments' cousin: Bedlam stops every blocker.
#[test]
fn bedlam_stops_every_blocker() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bedlam());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(g.computed_permanent(blocker).unwrap().keywords.contains(&Keyword::CantBlock));
}

/// Absolute Grace hands protection from black to everything, both sides.
#[test]
fn absolute_grace_protects_everyone_from_black() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::absolute_grace());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(
        g.computed_permanent(theirs)
            .unwrap()
            .keywords
            .contains(&Keyword::Protection(Color::Black))
    );
}

/// Endless Wurm eats an enchantment each upkeep — the new filtered ward cost
/// takes an enchantment, not a creature.
#[test]
fn endless_wurm_sacrifices_an_enchantment() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let wurm = g.add_card_to_battlefield(0, catalog::endless_wurm());
    let food = g.add_card_to_battlefield(0, catalog::bedlam());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(wurm).is_some(), "the wurm stayed");
    assert!(g.battlefield_find(food).is_none(), "the enchantment paid for it");
}

/// Citanul Hierophants turns the team into mana creatures.
#[test]
fn citanul_hierophants_grants_a_green_tap_ability() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::citanul_hierophants());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let abilities = g.granted_abilities_for(bear);
    assert_eq!(abilities.len(), 1, "one granted mana ability");
    activate(&mut g, bear, 0, None);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
}

/// Vebulid enters with a counter and dies at end of combat once it fights.
#[test]
fn vebulid_dies_at_end_of_combat() {
    let mut g = two_player_game();
    let vebulid = g.add_card_to_hand(0, catalog::vebulid());
    mana(&mut g, 0);
    cast(&mut g, vebulid, None);
    assert_eq!(g.battlefield_find(vebulid).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    g.clear_sickness(vebulid);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: vebulid,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(vebulid).is_some(), "it survives until end of combat");
    g.step = TurnStep::EndCombat;
    g.fire_step_triggers(TurnStep::EndCombat);
    drain_stack(&mut g);
    assert!(g.battlefield_find(vebulid).is_none());
}

/// Phyrexian Colossus stays tapped through untap and pays 8 life to stand up.
#[test]
fn phyrexian_colossus_pays_life_to_untap() {
    let mut g = two_player_game();
    let colossus = g.add_card_to_battlefield(0, catalog::phyrexian_colossus());
    g.battlefield_find_mut(colossus).unwrap().tapped = true;
    g.do_untap();
    assert!(g.battlefield_find(colossus).unwrap().tapped, "it doesn't untap normally");
    let life = g.players[0].life;
    activate(&mut g, colossus, 0, None);
    assert!(!g.battlefield_find(colossus).unwrap().tapped);
    assert_eq!(g.players[0].life, life - 8);
}

/// Chimeric Staff animates itself at whatever size you paid for.
#[test]
fn chimeric_staff_animates_at_x() {
    let mut g = two_player_game();
    let staff = g.add_card_to_battlefield(0, catalog::chimeric_staff());
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: staff,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(4),
    })
    .expect("activate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(staff).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature));
    assert_eq!((cp.power, cp.toughness), (4, 4));
}

/// Faith Healer trades an enchantment for its mana value in life.
#[test]
fn faith_healer_gains_the_sacrificed_enchantments_mana_value() {
    let mut g = two_player_game();
    let healer = g.add_card_to_battlefield(0, catalog::faith_healer());
    let food = g.add_card_to_battlefield(0, catalog::bedlam()); // {2}{R}{R} = 4
    let life = g.players[0].life;
    activate(&mut g, healer, 0, None);
    assert!(g.battlefield_find(food).is_none());
    assert_eq!(g.players[0].life, life + 4);
}

/// Fertile Ground's enchanted land yields an extra mana of any colour.
#[test]
fn fertile_ground_adds_an_extra_mana() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let aura = g.add_card_to_hand(0, catalog::fertile_ground());
    mana(&mut g, 0);
    cast(&mut g, aura, Some(Target::Permanent(land)));
    let before = g.players[0].mana_pool.total();
    activate(&mut g, land, 0, None);
    assert_eq!(g.players[0].mana_pool.total(), before + 2, "the land's mana plus one more");
}

/// Recantation's bounce is capped by its verse counters (CR 601.2c-style
/// resolution-time cap).
#[test]
fn recantation_bounces_up_to_its_verse_count() {
    let mut g = two_player_game();
    let rec = g.add_card_to_battlefield(0, catalog::recantation());
    g.battlefield_find_mut(rec).unwrap().add_counters(CounterType::Verse, 1);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::serra_zealot());
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rec,
        ability_index: 0,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(
        [a, b].iter().filter(|id| g.battlefield_find(**id).is_some()).count(),
        1,
        "one verse counter, one bounce"
    );
}

// ── Batch 3 ─────────────────────────────────────────────────────────────────

/// Karn animates a noncreature artifact at its mana value.
#[test]
fn karn_animates_an_artifact_at_its_mana_value() {
    let mut g = two_player_game();
    let karn = g.add_card_to_battlefield(0, catalog::karn_silver_golem());
    let target = g.add_card_to_battlefield(0, catalog::metrognome()); // {4}
    mana(&mut g, 0);
    activate(&mut g, karn, 0, Some(Target::Permanent(target)));
    let cp = g.computed_permanent(target).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature));
    assert_eq!((cp.power, cp.toughness), (4, 4));
}

/// Wall of Junk goes home at end of combat once it blocks.
#[test]
fn wall_of_junk_bounces_after_blocking() {
    let mut g = two_player_game();
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_junk());
    let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, atk)])).expect("block");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wall).is_some(), "it blocks first");
    g.step = TurnStep::EndCombat;
    g.fire_step_triggers(TurnStep::EndCombat);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == wall));
}

/// Gilded Drake trades itself for one of theirs.
#[test]
fn gilded_drake_swaps_itself_for_a_creature() {
    let mut g = two_player_game();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let drake = g.add_card_to_hand(0, catalog::gilded_drake());
    mana(&mut g, 0);
    cast(&mut g, drake, Some(Target::Permanent(theirs)));
    assert_eq!(g.battlefield_find(drake).unwrap().controller, 1);
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0);
}

/// Endoskeleton's boost lasts exactly as long as it stays tapped (CR 611.2c).
#[test]
fn endoskeleton_boost_ends_when_it_untaps() {
    let mut g = two_player_game();
    let skel = g.add_card_to_battlefield(0, catalog::endoskeleton());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    activate(&mut g, skel, 0, Some(Target::Permanent(bear)));
    assert_eq!(g.computed_permanent(bear).unwrap().toughness, 5);
    g.battlefield_find_mut(skel).unwrap().tapped = false;
    g.check_state_based_actions();
    assert_eq!(g.computed_permanent(bear).unwrap().toughness, 2);
}

/// Tainted Aether taxes every creature that lands, whoever plays it.
#[test]
fn tainted_aether_taxes_each_arrival() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tainted_aether());
    let fodder = g.add_card_to_battlefield(1, catalog::forest());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    mana(&mut g, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    cast(&mut g, bear, None);
    assert!(g.battlefield_find(fodder).is_none(), "their land paid the tax");
}

/// Retaliation hands the "becomes blocked" pump to your whole team.
#[test]
fn retaliation_grants_the_block_trigger() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::retaliation());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, bear)])).expect("block");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
}

/// Catastrophe's two modes each sweep their half of the board.
#[test]
fn catastrophe_sweeps_the_chosen_half() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::catastrophe());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "creature mode");
    assert!(g.battlefield_find(land).is_some(), "lands untouched");
}

/// Rain of Filth turns every land you control into a one-shot Swamp.
#[test]
fn rain_of_filth_grants_a_sacrifice_for_black() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::rain_of_filth());
    mana(&mut g, 0);
    cast(&mut g, spell, None);
    let before = g.players[0].mana_pool.amount(Color::Black);
    let idx = g.granted_abilities_for(land).len();
    assert_eq!(idx, 1, "one granted ability");
    activate(&mut g, land, 1, None);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), before + 1);
    assert!(g.battlefield_find(land).is_none(), "the land was sacrificed");
}

/// Urza's Armor shaves one off every hit its controller takes, and nothing off
/// damage to their permanents.
#[test]
fn urzas_armor_shaves_one_off_damage_to_you() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::urzas_armor());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let src = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[0].life;
    let mut ev = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(0),
        3,
        Some(src),
        &mut ev,
    );
    assert_eq!(g.players[0].life, life - 2);
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(bear),
        1,
        Some(src),
        &mut ev,
    );
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 1, "permanents aren't shielded");
}

/// Darkest Hour paints every creature black (CR 613 layer 5), both sides.
#[test]
fn darkest_hour_makes_every_creature_black() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::darkest_hour());
    let theirs = g.add_card_to_battlefield(1, catalog::serra_zealot()); // white
    assert_eq!(g.computed_permanent(theirs).unwrap().colors.to_vec(), vec![Color::Black]);
}
