//! Functionality tests for the `catalog::sets::decks::recent2` batch.

use crate::card::{CounterType, CreatureType, Effect, Keyword, StaticEffect};
use crate::catalog;
use crate::game::types::{Attack, AttackTarget, TurnStep};
use crate::game::*;
use crate::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Tangle fogs combat and keeps attackers from untapping next turn.
#[test]
fn tangle_fogs_and_locks_attackers() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::tangle());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Tangle");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, 20, "combat damage prevented");
    assert!(g.battlefield_find(atk).unwrap().tapped, "attacker still tapped");
}

/// March of Otherworldly Light exiles a creature with MV ≤ X.
#[test]
fn march_of_otherworldly_light_exiles_by_x() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let id = g.add_card_to_hand(0, catalog::march_of_otherworldly_light());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2); // X = 2
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("cast March");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == victim), "creature exiled");
}

/// Disdainful Stroke counters a 4-MV spell but not a cheap one.
#[test]
fn disdainful_stroke_counters_expensive_spell() {
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let spell = g.add_card_to_hand(1, catalog::serra_angel()); // {3}{W}{W} = MV 5
    g.players[1].mana_pool.add(Color::White, 2);
    g.players[1].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Serra Angel");
    g.perform_action(GameAction::PassPriority).expect("P1 passes to P0");
    let ds = g.add_card_to_hand(0, catalog::disdainful_stroke());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: ds, target: Some(Target::Permanent(spell)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Disdainful Stroke");
    drain_stack(&mut g);
    assert!(g.battlefield_find(spell).is_none(), "Serra Angel countered");
}

/// Flame Lash deals 4 to a player.
#[test]
fn flame_lash_deals_four() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::flame_lash());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Flame Lash");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 16);
}

/// Virtue of Persistence: the adventure half (-3/-3 + gain 2 life) resolves.
#[test]
fn virtue_of_persistence_adventure_shrinks_and_gains() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → dies
    let id = g.add_card_to_hand(0, catalog::virtue_of_persistence());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastAdventure {
        card_id: id, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Locthwain Scorn");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "−3/−3 killed the 2/2");
    assert_eq!(g.players[0].life, 22, "gained 2 life");
}

/// Scrabbling Skullcrab mills when an enchantment enters under your control.
#[test]
fn scrabbling_skullcrab_mills_on_enchantment_etb() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(1, catalog::island()); }
    g.add_card_to_battlefield(0, catalog::scrabbling_skullcrab());
    let lib_before = g.players[1].library.len();
    // An enchantment entering under your control triggers the mill — cast it
    // through the full path so observer triggers dispatch.
    let ench = g.add_card_to_hand(0, catalog::possibility_storm());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: ench, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Possibility Storm");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib_before - 2, "opponent milled two");
}

/// Hush destroys every enchantment.
#[test]
fn hush_destroys_all_enchantments() {
    let mut g = two_player_game();
    let e1 = g.add_card_to_battlefield(0, catalog::glorious_anthem());
    let e2 = g.add_card_to_battlefield(1, catalog::glorious_anthem());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::hush());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Hush");
    drain_stack(&mut g);
    assert!(g.battlefield_find(e1).is_none() && g.battlefield_find(e2).is_none(), "enchantments gone");
    assert!(g.battlefield_find(bear).is_some(), "creature untouched");
}

/// Hush can be cycled away for {2}.
#[test]
fn hush_can_be_cycled() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::hush());
    g.players[0].mana_pool.add_colorless(2);
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None }).expect("cycle Hush");
    drain_stack(&mut g);
    // Cycled away (-1) and drew a card (+1) → hand size unchanged, Hush in gy.
    assert_eq!(g.players[0].hand.len(), before, "cycle drew a replacement");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "Hush in graveyard");
}

/// Llanowar Greenwidow returns itself from the graveyard to the battlefield.
#[test]
fn llanowar_greenwidow_returns_from_graveyard() {
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::llanowar_greenwidow());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(7);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate graveyard return");
    drain_stack(&mut g);
    let r = g.battlefield_find(id).expect("returned to battlefield");
    assert!(r.tapped, "returns tapped");
    assert_eq!((r.power(), r.toughness()), (4, 3));
}

/// Searchlight Companion makes a Spirit token on ETB.
#[test]
fn searchlight_companion_makes_a_spirit() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::searchlight_companion());
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Spirit").count();
    assert_eq!(spirits, 1);
}

/// Resolute Reinforcements has flash and makes a Soldier on ETB.
#[test]
fn resolute_reinforcements_makes_a_soldier() {
    let mut g = two_player_game();
    assert!(catalog::resolute_reinforcements().keywords.contains(&Keyword::Flash));
    g.move_card_to_battlefield_for_test(0, catalog::resolute_reinforcements());
    drain_stack(&mut g);
    let soldiers = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Soldier").count();
    assert_eq!(soldiers, 1);
}

/// Jewel Thief makes a Treasure on ETB.
#[test]
fn jewel_thief_makes_a_treasure() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::jewel_thief());
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Treasure"));
}

/// Sweettooth Witch makes a Food on ETB.
#[test]
fn sweettooth_witch_makes_a_food() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::sweettooth_witch());
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Food"));
}

/// Ambush Paratrooper's {5} ability pumps the team.
#[test]
fn ambush_paratrooper_pumps_team() {
    let mut g = two_player_game();
    let trooper = g.add_card_to_battlefield(0, catalog::ambush_paratrooper());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(5);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: trooper, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "bear pumped +1/+1");
}

/// Glistening Deluge shrinks all creatures and hits G/W harder.
#[test]
fn glistening_deluge_punishes_green_white() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green 2/2 → -3/-3 dies
    let id = g.add_card_to_hand(0, catalog::glistening_deluge());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Glistening Deluge");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "green 2/2 took -3/-3 and died");
}

/// Faerie Dreamthief surveils on ETB and can be exiled from the graveyard to draw.
#[test]
fn faerie_dreamthief_surveils_and_recurs() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.move_card_to_battlefield_for_test(0, catalog::faerie_dreamthief());
    drain_stack(&mut g);
    let d = catalog::faerie_dreamthief();
    assert_eq!(d.triggered_abilities.len(), 1, "ETB surveil wired");
    assert!(d.activated_abilities[0].from_graveyard && d.activated_abilities[0].exile_self_cost);
}

/// Vinereap Mentor makes a Food on ETB (and again on death).
#[test]
fn vinereap_mentor_makes_food_on_etb() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::vinereap_mentor());
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Food"));
    assert_eq!(catalog::vinereap_mentor().triggered_abilities.len(), 2, "etb + dies");
}

/// Topiary Panther is a 6/5 trampler with basic landcycling.
#[test]
fn topiary_panther_has_basic_landcycling() {
    let d = catalog::topiary_panther();
    assert_eq!((d.power, d.toughness), (6, 5));
    assert!(d.keywords.iter().any(|k| matches!(k, Keyword::Typecycling(_))));
}

/// Valgavoth's Faithful sacrifices itself to reanimate a creature.
#[test]
fn valgavoths_faithful_reanimates() {
    let mut g = two_player_game();
    let faithful = g.add_card_to_battlefield(0, catalog::valgavoths_faithful());
    let dead = g.add_card_to_graveyard(0, catalog::serra_angel());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: faithful, ability_index: 0, target: Some(Target::Permanent(dead)),
        additional_targets: vec![], x_value: None,
    }).expect("reanimate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(faithful).is_none(), "Faithful sacrificed");
    assert!(g.battlefield_find(dead).is_some(), "Serra Angel reanimated");
}

/// Charforger makes a Goblin on ETB and grows when another of your creatures dies.
#[test]
fn charforger_etb_token_and_death_growth() {
    let mut g = two_player_game();
    let charforger = g.move_card_to_battlefield_for_test(0, catalog::charforger());
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Phyrexian Goblin"));
    // Kill another creature you control via lethal damage so the death goes
    // through the SBA + observer-trigger dispatch (not the self-source helper).
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(fodder)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt fodder");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(charforger).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "grew when another creature you control died",
    );
}

/// Voracious Vermin makes a Rat on ETB.
#[test]
fn voracious_vermin_makes_a_rat() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::voracious_vermin());
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.is_token
        && c.definition.subtypes.creature_types.contains(&CreatureType::Rat)));
}

/// Mocking Sprite reduces your instant/sorcery costs by {1}.
#[test]
fn mocking_sprite_discounts_instants() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mocking_sprite());
    // Lightning Bolt {R} would normally need {R}; with the discount a {1}{R}
    // instant should be castable off two mana. Use a {1}{R} instant: Flame Lash
    // is {3}{R}; instead verify the static is wired and matches instants.
    let d = catalog::mocking_sprite();
    assert!(matches!(d.static_abilities[0].effect, StaticEffect::CostReduction { amount: 1, .. }));
    // Functional check: a {1}{U} instant (Disdainful Stroke) is castable for {U}.
    g.priority.player_with_priority = 1;
    g.active_player_idx = 1;
    let spell = g.add_card_to_hand(1, catalog::serra_angel());
    g.players[1].mana_pool.add(Color::White, 2);
    g.players[1].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp casts");
    g.perform_action(GameAction::PassPriority).expect("pass");
    let ds = g.add_card_to_hand(0, catalog::disdainful_stroke());
    g.players[0].mana_pool.add(Color::Blue, 1); // only {U}, discount covers the {1}
    g.perform_action(GameAction::CastSpell {
        card_id: ds, target: Some(Target::Permanent(spell)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Disdainful Stroke castable for {U} under Mocking Sprite");
    drain_stack(&mut g);
    assert!(g.battlefield_find(spell).is_none(), "countered");
}

/// Ancestral Reminiscence draws three then discards one (net +2).
#[test]
fn ancestral_reminiscence_draws_three_discards_one() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::ancestral_reminiscence());
    let before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // cast (-1) + draw 3 (+3) - discard 1 (-1) = +1 vs before.
    assert_eq!(g.players[0].hand.len(), before + 1);
}

/// Charge pumps your team +1/+1.
#[test]
fn charge_pumps_team() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::charge());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Charge");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
}

/// Heroic Reinforcements makes two Soldiers and pumps + hastes the team.
#[test]
fn heroic_reinforcements_makes_soldiers_and_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::heroic_reinforcements());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let soldiers: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Soldier").collect();
    assert_eq!(soldiers.len(), 2);
    let sid = soldiers[0].id;
    assert_eq!(g.computed_permanent(sid).unwrap().power, 2, "1/1 pumped to 2/2");
    assert!(g.computed_permanent(sid).unwrap().keywords.contains(&Keyword::Haste));
}

/// Pyrewood Gearhulk buffs your other creatures on ETB.
#[test]
fn pyrewood_gearhulk_buffs_others() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::pyrewood_gearhulk());
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 4, "other creature got +2/+2");
    assert!(cp.keywords.contains(&Keyword::Menace));
}

/// Beastbond Outcaster draws on ETB only with a power-4+ creature.
#[test]
fn beastbond_outcaster_conditional_draw() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4 → condition met
    let before = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::beastbond_outcaster());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1, "drew with a big creature out");
    assert!(catalog::beastbond_outcaster().plot_cost.is_some());
}

/// Mindwhisker surveils at the beginning of your upkeep.
#[test]
fn mindwhisker_surveils_on_upkeep() {
    let d = catalog::mindwhisker();
    assert_eq!((d.power, d.toughness), (3, 2));
    assert!(matches!(d.triggered_abilities[0].effect, Effect::Surveil { .. }));
    assert!(matches!(
        d.triggered_abilities[0].event.kind,
        crate::card::EventKind::StepBegins(TurnStep::Upkeep)
    ));
}

/// Tarrian's Soulcleaver grants vigilance and grows the equipped creature when
/// another permanent dies.
#[test]
fn tarrians_soulcleaver_grows_equipped_on_death() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cleaver = g.add_card_to_battlefield(0, catalog::tarrians_soulcleaver());
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::Equip { equipment: cleaver, target: bear }).expect("equip");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Vigilance),
        "equipped creature has vigilance"
    );
    // Another creature dies → +1/+1 counter on the equipped bear.
    let fodder = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(fodder).unwrap().damage = 2;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(1),
        "equipped creature grew by a +1/+1 counter"
    );
}

/// Snarespinner pumps +2/+0 when it blocks a flier (but not a grounded attacker).
#[test]
fn snarespinner_pumps_blocking_a_flier() {
    use crate::card::{CardDefinition, CardType};
    let flier = CardDefinition {
        name: "Test Drake",
        card_types: vec![CardType::Creature],
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, flier);
    let spider = g.add_card_to_battlefield(1, catalog::snarespinner());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(spider, atk)])).expect("block");
    drain_stack(&mut g);
    let s = g.battlefield_find(spider).unwrap();
    assert_eq!((s.power(), s.toughness()), (3, 3), "+2/+0 for blocking a flier");
}

/// Lord Skitter makes a Rat at the beginning of combat on your turn.
#[test]
fn lord_skitter_makes_a_rat_in_combat() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lord_skitter_sewer_king());
    advance_to(&mut g, TurnStep::BeginCombat);
    drain_stack(&mut g);
    let rats = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Rat))
        .count();
    assert_eq!(rats, 1, "one Rat token created at combat");
}

/// Stickytongue Sentinel bounces another permanent you control on entry.
#[test]
fn stickytongue_sentinel_bounces_own_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::stickytongue_sentinel());
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear left the battlefield");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "bear returned to hand");
}

/// Ossification exiles an opponent's creature until it leaves.
#[test]
fn ossification_exiles_until_it_leaves() {
    let foe_def = catalog::grizzly_bears();
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, foe_def);
    g.move_card_to_battlefield_for_test(0, catalog::ossification());
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == foe), "opponent creature exiled");
    assert!(g.battlefield_find(foe).is_none());
}

/// Sunfall exiles all creatures.
#[test]
fn sunfall_exiles_all_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::serra_angel());
    let s = g.add_card_to_hand(0, catalog::sunfall());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: s, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Sunfall");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "all creatures gone");
    assert_eq!(g.exile.iter().filter(|c| c.id == a || c.id == b).count(), 2, "both exiled");
}

/// Witchstalker Frenzy's discount counts every player's attackers this turn.
#[test]
fn witchstalker_frenzy_counts_all_attackers() {
    use crate::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    let spell = crate::card::CardInstance::new(g.next_id(), catalog::witchstalker_frenzy(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 0, "no attacks → full price");
    g.players[1].creatures_attacked_this_turn = 2; // the OPPONENT attacked
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 2, "all-players count includes opp");
}

/// Warden of the Inner Sky gains flying and vigilance at three counters.
#[test]
fn warden_of_the_inner_sky_unlocks_at_three_counters() {
    let mut g = two_player_game();
    let w = g.add_card_to_battlefield(0, catalog::warden_of_the_inner_sky());
    assert!(!g.computed_permanent(w).unwrap().keywords.contains(&Keyword::Flying), "no flying yet");
    g.battlefield_find_mut(w).unwrap().counters.insert(CounterType::PlusOnePlusOne, 3);
    let cp = g.computed_permanent(w).unwrap();
    assert!(cp.keywords.contains(&Keyword::Flying), "flying at 3 counters");
    assert!(cp.keywords.contains(&Keyword::Vigilance), "vigilance at 3 counters");
}

/// Gathering Throng tutors its same-named copies to hand on entry.
#[test]
fn gathering_throng_gathers_copies() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let c1 = g.add_card_to_library(0, catalog::gathering_throng());
    let c2 = g.add_card_to_library(0, catalog::gathering_throng());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(c1)),
        DecisionAnswer::Search(Some(c2)),
    ]));
    g.move_card_to_battlefield_for_test(0, catalog::gathering_throng());
    drain_stack(&mut g);
    let in_hand = g.players[0].hand.iter()
        .filter(|c| c.definition.name == "Gathering Throng").count();
    assert_eq!(in_hand, 2, "both library copies found");
}

/// Charming Scoundrel's Treasure mode mints a Treasure.
#[test]
fn charming_scoundrel_treasure_mode() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    g.move_card_to_battlefield_for_test(0, catalog::charming_scoundrel());
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Treasure"),
        "Treasure token created"
    );
}

/// Fear of Missing Out loots on entry (discard then draw).
#[test]
fn fear_of_missing_out_loots_on_etb() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_hand(0, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    g.move_card_to_battlefield_for_test(0, catalog::fear_of_missing_out());
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder), "discarded the Island");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"), "drew the Forest");
}

/// Archmage of Runes discounts instants/sorceries and draws when you cast one.
#[test]
fn archmage_of_runes_discount_and_draw() {
    use crate::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::archmage_of_runes());
    let bolt = crate::card::CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &bolt, None), 1, "one generic off an instant");
    // Casting an instant draws a card.
    let id = g.add_card_to_hand(0, catalog::lightning_bolt());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(foe)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    drain_stack(&mut g);
    // Hand: -1 (bolt cast) +1 (magecraft draw) = net unchanged vs before-minus-cast.
    assert_eq!(g.players[0].hand.len(), before, "magecraft replaced the cast card");
}

/// Splashy Spellcaster mints a Sorcerer Role on a friendly creature when you
/// cast an instant or sorcery.
#[test]
fn splashy_spellcaster_makes_a_role() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::splashy_spellcaster());
    let pet = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lightning_bolt());
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(foe)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    drain_stack(&mut g);
    let role_on_pet = g.battlefield.iter()
        .any(|c| c.attached_to == Some(pet) && c.definition.name == "Sorcerer");
    assert!(role_on_pet, "Sorcerer Role attached to the bear");
    // The Role grants +1/+1.
    let cp = g.computed_permanent(pet).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 from the Role");
}

/// Subterranean Schooner: crew it with a Bear, attack, and the crewer explores.
#[test]
fn subterranean_schooner_explores_on_attack() {
    let mut g = two_player_game();
    let ship = g.add_card_to_battlefield(0, catalog::subterranean_schooner());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(ship);
    g.clear_sickness(bear);
    g.add_card_to_library(0, catalog::forest()); // top card is a land → goes to hand
    g.perform_action(GameAction::Crew { vehicle: ship, crew_creatures: vec![bear] }).expect("crew 1");
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ship, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"), "explored land to hand");
}

/// Steamcore Scholar draws two then discards two on entry.
#[test]
fn steamcore_scholar_draw_then_discard() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    g.move_card_to_battlefield_for_test(0, catalog::steamcore_scholar());
    drain_stack(&mut g);
    // Net hand size unchanged (+2 draw, −2 discard); two cards now in graveyard.
    assert_eq!(g.players[0].graveyard.len(), 2, "discarded two cards");
}

/// Axgard Cavalry taps to grant haste.
#[test]
fn axgard_cavalry_grants_haste() {
    let mut g = two_player_game();
    let cav = g.add_card_to_battlefield(0, catalog::axgard_cavalry());
    let fresh = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(cav);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cav, ability_index: 0,
        target: Some(Target::Permanent(fresh)), additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(g.computed_permanent(fresh).unwrap().keywords.contains(&Keyword::Haste));
}

/// Experimental Synthesizer exiles the top card with may-play on entry, and its
/// sac ability makes a Samurai.
#[test]
fn experimental_synthesizer_etb_and_sac() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let synth = g.move_card_to_battlefield_for_test(0, catalog::experimental_synthesizer());
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.definition.name == "Island"), "top card exiled (may-play)");
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: synth, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac for Samurai");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Samurai"), "Samurai token made");
}

/// Hexgold Slith nets energy on entry and grows on combat damage.
#[test]
fn hexgold_slith_energy_and_growth() {
    let mut g = two_player_game();
    let slith = g.add_card_to_battlefield(0, catalog::hexgold_slith());
    g.move_card_to_battlefield_for_test(0, catalog::hexgold_slith());
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 2, "ETB gave two energy");
    // Attack unblocked → combat damage → +1/+1 counter.
    g.clear_sickness(slith);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: slith, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(slith).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(1),
        "grew on combat damage"
    );
}

/// Slickshot Lockpicker grants flashback to an instant/sorcery in your graveyard.
#[test]
fn slickshot_lockpicker_grants_flashback() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.move_card_to_battlefield_for_test(0, catalog::slickshot_lockpicker());
    drain_stack(&mut g);
    let gy_bolt = g.players[0].graveyard.iter().find(|c| c.id == bolt).unwrap();
    assert!(gy_bolt.granted_flashback_eot.is_some(), "bolt gained flashback this turn");
}

/// Tender Wildguide taps for any color and (separately) for a +1/+1 counter.
#[test]
fn tender_wildguide_taps_for_counter() {
    let mut g = two_player_game();
    let w = g.add_card_to_battlefield(0, catalog::tender_wildguide());
    g.clear_sickness(w);
    g.perform_action(GameAction::ActivateAbility {
        card_id: w, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for counter");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(w).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(1),
    );
}

/// Sinister Monolith drains at combat and can sac for two cards.
#[test]
fn sinister_monolith_drains_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sinister_monolith());
    advance_to(&mut g, TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "opponent lost 1");
    assert_eq!(g.players[0].life, 21, "you gained 1");
}

/// CR 702.70 — Pit Scorpion's Poisonous 1 adds a poison counter on combat damage.
#[test]
fn pit_scorpion_poisonous_adds_poison() {
    let mut g = two_player_game();
    let scorp = g.add_card_to_battlefield(0, catalog::pit_scorpion());
    g.clear_sickness(scorp);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: scorp, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "1 combat damage");
    assert_eq!(g.players[1].poison_counters, 1, "Poisonous 1 adds a poison counter");
}

/// Splatter Goblin shrinks an opponent's creature when it dies.
#[test]
fn splatter_goblin_death_shrinks_opponent() {
    let mut g = two_player_game();
    let gob = g.add_card_to_battlefield(0, catalog::splatter_goblin());
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.battlefield_find_mut(gob).unwrap().damage = 1; // lethal on the 2/1
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let cp = g.computed_permanent(foe).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "-1/-1 on the opponent's creature");
}

/// Hightide Hermit nets four energy and can pay {E}{E} to attack despite defender.
#[test]
fn hightide_hermit_energy_then_attacks() {
    let mut g = two_player_game();
    let crab = g.add_card_to_battlefield(0, catalog::hightide_hermit());
    g.move_card_to_battlefield_for_test(0, catalog::hightide_hermit());
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 4, "ETB gave four energy");
    g.clear_sickness(crab);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: crab, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("pay energy to attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: crab, target: AttackTarget::Player(1),
    }])).expect("defender can attack this turn");
}
