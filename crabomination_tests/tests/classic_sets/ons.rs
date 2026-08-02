//! Onslaught (ONS) — the tribal block's opener: lords, tribal counts and the
//! "creature type of your choice" spells.

use crabomination::card::{CardId, CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    cast_x(g, seat, id, target, None);
}

fn cast_x(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>, x: Option<u32>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: x,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, seat: usize, card_id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Aven Brigadier lords over Birds and Soldiers at once.
#[test]
fn aven_brigadier_lords_two_tribes() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::aven_brigadier());
    let bird = g.add_card_to_battlefield(0, catalog::screeching_buzzard());
    let soldier = g.add_card_to_battlefield(0, catalog::gustcloak_runner());
    assert_eq!(g.computed_permanent(bird).unwrap().power, 3, "2/2 Bird +1/+1");
    assert_eq!(g.computed_permanent(soldier).unwrap().power, 2, "1/1 Soldier +1/+1");
}

/// Airborne Aid counts every Bird on the battlefield, not just yours.
#[test]
fn airborne_aid_counts_every_bird() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::screeching_buzzard());
    g.add_card_to_battlefield(1, catalog::aven_brigadier());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::airborne_aid());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.players[0].hand.len(), 2);
}

/// Tribal Unity pumps only the named type, by the paid X.
#[test]
fn tribal_unity_pumps_the_named_tribe() {
    let mut g = main_phase();
    let elf = g.add_card_to_battlefield(0, catalog::wirewood_elf());
    let beast = g.add_card_to_battlefield(0, catalog::barkhide_mauler());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Elf)]));
    let spell = g.add_card_to_hand(0, catalog::tribal_unity());
    cast_x(&mut g, 0, spell, None, Some(3));
    assert_eq!(g.computed_permanent(elf).unwrap().power, 4, "1/2 Elf +3/+3");
    assert_eq!(g.computed_permanent(beast).unwrap().power, 4, "Beasts untouched");
}

/// Brightstone Ritual scales with the Goblin count.
#[test]
fn brightstone_ritual_pays_per_goblin() {
    let mut g = main_phase();
    for seat in [0, 1] {
        g.add_card_to_battlefield(seat, catalog::greasewrench_goblin());
    }
    let spell = g.add_card_to_hand(0, catalog::brightstone_ritual());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 21, "20 seeded, one red paid, 2 Goblins");
}

/// Infest shrinks the whole board and kills the small.
#[test]
fn infest_sweeps_two_toughness() {
    let mut g = main_phase();
    let small = g.add_card_to_battlefield(1, catalog::wirewood_elf());
    let big = g.add_card_to_battlefield(1, catalog::barkhide_mauler());
    let spell = g.add_card_to_hand(0, catalog::infest());
    cast(&mut g, 0, spell, None);
    assert!(g.battlefield_find(small).is_none(), "1/2 dies");
    assert_eq!(g.computed_permanent(big).unwrap().toughness, 2);
}

/// Symbiotic Wurm leaves seven Insects behind.
#[test]
fn symbiotic_wurm_leaves_a_swarm() {
    let mut g = main_phase();
    let wurm = g.add_card_to_battlefield(0, catalog::symbiotic_wurm());
    let _ = g.remove_to_graveyard_with_triggers(wurm);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Insect").count(), 7);
}

/// Cabal Archon turns Clerics into a two-point drain.
#[test]
fn cabal_archon_drains_for_a_cleric() {
    let mut g = main_phase();
    let archon = g.add_card_to_battlefield(0, catalog::cabal_archon());
    g.add_card_to_battlefield(0, catalog::nova_cleric());
    activate(&mut g, 0, archon, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 18);
    assert_eq!(g.players[0].life, 22);
}

/// Wretched Anurid bleeds you for every other creature that arrives.
#[test]
fn wretched_anurid_bleeds_on_arrivals() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::wretched_anurid());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    cast(&mut g, 1, bear, None);
    assert_eq!(g.players[0].life, 19);
}

/// Visara kills anything, and it stays dead.
#[test]
fn visara_kills_without_regeneration() {
    let mut g = main_phase();
    let visara = g.add_card_to_battlefield(0, catalog::visara_the_dreadful());
    g.clear_sickness(visara);
    let victim = g.add_card_to_battlefield(1, catalog::silvos_rogue_elemental());
    g.battlefield_find_mut(victim).unwrap().regeneration_shields = 1;
    activate(&mut g, 0, visara, 0, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(victim).is_none());
}

/// Wave of Indifference sidelines exactly X blockers.
#[test]
fn wave_of_indifference_benches_x_creatures() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::wave_of_indifference());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(1),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.computed_permanent(a).unwrap().keywords.contains(&Keyword::CantBlock));
    assert!(!g.computed_permanent(b).unwrap().keywords.contains(&Keyword::CantBlock));
}

/// Gustcloak Runner ducks out of the block it walked into.
#[test]
fn gustcloak_runner_slips_the_block() {
    let mut g = main_phase();
    let runner = g.add_card_to_battlefield(0, catalog::gustcloak_runner());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(runner);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: runner,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, runner)])).expect("block");
    drain_stack(&mut g);
    assert!(
        !g.attacking().iter().any(|a| a.attacker == runner),
        "the Runner slipped out of combat"
    );
}

/// Mobilization hands Soldiers vigilance and mints more of them.
#[test]
fn mobilization_arms_the_soldiers() {
    let mut g = main_phase();
    let mob = g.add_card_to_battlefield(0, catalog::mobilization());
    let soldier = g.add_card_to_battlefield(0, catalog::gustcloak_runner());
    assert!(g.computed_permanent(soldier).unwrap().keywords.contains(&Keyword::Vigilance));
    activate(&mut g, 0, mob, 0, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Soldier").count(), 1);
}

/// Overwhelming Instinct only pays off a three-wide attack.
#[test]
fn overwhelming_instinct_needs_three_attackers() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::overwhelming_instinct());
    let attackers: Vec<CardId> = (0..3)
        .map(|_| {
            let id = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.clear_sickness(id);
            id
        })
        .collect();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// True Believer makes you untargetable.
#[test]
fn true_believer_gives_you_shroud() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::true_believer());
    assert!(g.player_has_static_shroud(0));
    assert!(!g.player_has_static_shroud(1));
}

/// Slate of Ancestry trades your hand for a card per creature.
#[test]
fn slate_of_ancestry_refills_off_the_board() {
    let mut g = main_phase();
    let slate = g.add_card_to_battlefield(0, catalog::slate_of_ancestry());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    g.add_card_to_hand(0, catalog::forest());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    activate(&mut g, 0, slate, 0, None);
    assert_eq!(g.players[0].hand.len(), 3, "hand pitched, three creatures drawn");
}

/// Aphetto Dredging buys back up to three of the named tribe.
#[test]
fn aphetto_dredging_returns_a_tribe() {
    let mut g = main_phase();
    for _ in 0..2 {
        g.add_card_to_graveyard(0, catalog::wirewood_elf());
    }
    g.add_card_to_graveyard(0, catalog::barkhide_mauler());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Elf)]));
    let spell = g.add_card_to_hand(0, catalog::aphetto_dredging());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.players[0].hand.len(), 2, "both Elves, not the Beast");
}

/// Riptide Laboratory saves a Wizard from removal.
#[test]
fn riptide_laboratory_rescues_a_wizard() {
    let mut g = main_phase();
    let lab = g.add_card_to_battlefield(0, catalog::riptide_laboratory());
    let wizard = g.add_card_to_battlefield(0, catalog::crafty_pathmage());
    activate(&mut g, 0, lab, 1, Some(Target::Permanent(wizard)));
    assert!(g.battlefield_find(wizard).is_none());
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Silklash Spider shoots down every flier for X.
#[test]
fn silklash_spider_sweeps_the_sky() {
    let mut g = main_phase();
    let spider = g.add_card_to_battlefield(0, catalog::silklash_spider());
    let flier = g.add_card_to_battlefield(1, catalog::screeching_buzzard());
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: spider,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(flier).is_none());
    assert!(g.battlefield_find(ground).is_some());
}

/// Nova Cleric wipes every enchantment, including its own team's.
#[test]
fn nova_cleric_sweeps_enchantments() {
    let mut g = main_phase();
    let cleric = g.add_card_to_battlefield(0, catalog::nova_cleric());
    g.clear_sickness(cleric);
    let mine = g.add_card_to_battlefield(0, catalog::mobilization());
    let theirs = g.add_card_to_battlefield(1, catalog::righteous_cause());
    activate(&mut g, 0, cleric, 0, None);
    assert!(g.battlefield_find(mine).is_none());
    assert!(g.battlefield_find(theirs).is_none());
}

/// Wall of Mulch turns spare Walls into cards.
#[test]
fn wall_of_mulch_cashes_in_a_wall() {
    let mut g = main_phase();
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_mulch());
    g.add_card_to_battlefield(0, catalog::wall_of_mulch());
    g.add_card_to_library(0, catalog::forest());
    activate(&mut g, 0, wall, 0, None);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Mythic Proportions is +8/+8 and trample.
#[test]
fn mythic_proportions_is_a_monster() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::mythic_proportions());
    cast(&mut g, 0, aura, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (10, 10));
    assert!(cp.keywords.contains(&Keyword::Trample));
}

/// Reminisce puts a graveyard back into its library.
#[test]
fn reminisce_recycles_a_graveyard() {
    let mut g = main_phase();
    for _ in 0..4 {
        g.add_card_to_graveyard(1, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::reminisce());
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert!(g.players[1].graveyard.is_empty());
    assert_eq!(g.players[1].library.len(), 4);
}

/// Blatant Thievery takes a permanent outright.
#[test]
fn blatant_thievery_steals_a_permanent() {
    let mut g = main_phase();
    let prize = g.add_card_to_battlefield(1, catalog::silvos_rogue_elemental());
    let spell = g.add_card_to_hand(0, catalog::blatant_thievery());
    cast(&mut g, 0, spell, Some(Target::Permanent(prize)));
    assert_eq!(g.battlefield_find(prize).unwrap().controller, 0);
}

/// Cabal Slaver strips a card off every Goblin hit.
#[test]
fn cabal_slaver_punishes_goblin_damage() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::cabal_slaver());
    let goblin = g.add_card_to_battlefield(0, catalog::greasewrench_goblin());
    g.add_card_to_hand(1, catalog::forest());
    g.clear_sickness(goblin);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: goblin,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.players[1].hand.is_empty());
}

/// Centaur Glade makes a 3/3 on demand.
#[test]
fn centaur_glade_mints_a_centaur() {
    let mut g = main_phase();
    let glade = g.add_card_to_battlefield(0, catalog::centaur_glade());
    activate(&mut g, 0, glade, 0, None);
    let token = g.battlefield.iter().find(|c| c.definition.name == "Centaur").expect("token");
    assert_eq!((token.definition.power, token.definition.toughness), (3, 3));
}

/// Aggravated Assault untaps the team and buys another combat.
#[test]
fn aggravated_assault_buys_a_combat() {
    let mut g = main_phase();
    let assault = g.add_card_to_battlefield(0, catalog::aggravated_assault());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    activate(&mut g, 0, assault, 0, None);
    assert!(!g.battlefield_find(bear).unwrap().tapped);
    assert_eq!(g.additional_post_main_combats, 1);
}

/// Wirewood Savage draws off every Beast that shows up.
#[test]
fn wirewood_savage_draws_for_beasts() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::wirewood_savage());
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let beast = g.add_card_to_hand(0, catalog::barkhide_mauler());
    cast(&mut g, 0, beast, None);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Elvish Scrapper eats an artifact.
#[test]
fn elvish_scrapper_eats_an_artifact() {
    let mut g = main_phase();
    let elf = g.add_card_to_battlefield(0, catalog::elvish_scrapper());
    g.clear_sickness(elf);
    let slate = g.add_card_to_battlefield(1, catalog::slate_of_ancestry());
    activate(&mut g, 0, elf, 0, Some(Target::Permanent(slate)));
    assert!(g.battlefield_find(slate).is_none());
}

/// Grand Melee forces everyone into combat.
#[test]
fn grand_melee_forces_attacks_and_blocks() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::grand_melee());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::MustAttack));
    assert!(cp.keywords.contains(&Keyword::MustBlock));
}

/// Krosan Groundshaker hands trample to any Beast.
#[test]
fn krosan_groundshaker_grants_trample() {
    let mut g = main_phase();
    let shaker = g.add_card_to_battlefield(0, catalog::krosan_groundshaker());
    let beast = g.add_card_to_battlefield(0, catalog::barkhide_mauler());
    activate(&mut g, 0, shaker, 0, Some(Target::Permanent(beast)));
    assert!(g.computed_permanent(beast).unwrap().keywords.contains(&Keyword::Trample));
}

/// Daru Encampment pumps a Soldier off a colorless land.
#[test]
fn daru_encampment_pumps_a_soldier() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::daru_encampment());
    let soldier = g.add_card_to_battlefield(0, catalog::gustcloak_runner());
    activate(&mut g, 0, land, 1, Some(Target::Permanent(soldier)));
    assert_eq!(g.computed_permanent(soldier).unwrap().power, 2);
}

/// Accursed Centaur eats a creature on the way in.
#[test]
fn accursed_centaur_eats_a_friend() {
    let mut g = main_phase();
    let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let centaur = g.add_card_to_hand(0, catalog::accursed_centaur());
    cast(&mut g, 0, centaur, None);
    assert!(g.battlefield_find(friend).is_none(), "the bear, not the Centaur");
}

/// Screeching Buzzard takes a card with it.
#[test]
fn screeching_buzzard_strips_on_death() {
    let mut g = main_phase();
    let bird = g.add_card_to_battlefield(0, catalog::screeching_buzzard());
    g.add_card_to_hand(1, catalog::forest());
    let _ = g.remove_to_graveyard_with_triggers(bird);
    drain_stack(&mut g);
    assert!(g.players[1].hand.is_empty());
}

/// Dispersing Orb buys a bounce with a spare permanent.
#[test]
fn dispersing_orb_bounces_for_a_permanent() {
    let mut g = main_phase();
    let orb = g.add_card_to_battlefield(0, catalog::dispersing_orb());
    g.add_card_to_battlefield(0, catalog::forest());
    let prize = g.add_card_to_battlefield(1, catalog::silvos_rogue_elemental());
    activate(&mut g, 0, orb, 0, Some(Target::Permanent(prize)));
    assert!(g.battlefield_find(prize).is_none());
    assert_eq!(g.players[1].hand.len(), 1);
}

/// Crafty Pathmage waves small creatures through.
#[test]
fn crafty_pathmage_sneaks_the_small() {
    let mut g = main_phase();
    let mage = g.add_card_to_battlefield(0, catalog::crafty_pathmage());
    g.clear_sickness(mage);
    let runner = g.add_card_to_battlefield(0, catalog::gustcloak_runner());
    activate(&mut g, 0, mage, 0, Some(Target::Permanent(runner)));
    assert!(g.computed_permanent(runner).unwrap().keywords.contains(&Keyword::Unblockable));
}

/// Righteous Cause pays a life for every attack on the table.
#[test]
fn righteous_cause_gains_off_any_attack() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::righteous_cause());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 21);
}

/// Silvos regenerates for a single green.
#[test]
fn silvos_regenerates_for_one_green() {
    let mut g = main_phase();
    let silvos = g.add_card_to_battlefield(0, catalog::silvos_rogue_elemental());
    activate(&mut g, 0, silvos, 0, None);
    assert_eq!(g.battlefield_find(silvos).unwrap().regeneration_shields, 1);
}

/// Barkhide Mauler cycles for a card.
#[test]
fn barkhide_mauler_cycles() {
    let mut g = main_phase();
    let mauler = g.add_card_to_hand(0, catalog::barkhide_mauler());
    g.add_card_to_library(0, catalog::forest());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Cycle { card_id: mauler, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == mauler));
}

/// Inspirit untaps a blocker and makes it survive.
#[test]
fn inspirit_untaps_and_grows() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::inspirit());
    cast(&mut g, 0, spell, Some(Target::Permanent(bear)));
    assert!(!g.battlefield_find(bear).unwrap().tapped);
    assert_eq!(g.computed_permanent(bear).unwrap().toughness, 6);
}

/// Crowd Favorites taps a blocker out of the way.
#[test]
fn crowd_favorites_taps_a_blocker() {
    let mut g = main_phase();
    let crowd = g.add_card_to_battlefield(0, catalog::crowd_favorites());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, crowd, 0, Some(Target::Permanent(blocker)));
    assert!(g.battlefield_find(blocker).unwrap().tapped);
}

/// Defensive Maneuvers braces only the named tribe.
#[test]
fn defensive_maneuvers_braces_one_tribe() {
    let mut g = main_phase();
    let elf = g.add_card_to_battlefield(0, catalog::wirewood_elf());
    let beast = g.add_card_to_battlefield(0, catalog::barkhide_mauler());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Elf)]));
    let spell = g.add_card_to_hand(0, catalog::defensive_maneuvers());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.computed_permanent(elf).unwrap().toughness, 6);
    assert_eq!(g.computed_permanent(beast).unwrap().toughness, 4);
}

/// Goblin Burrows swings a Goblin off a colorless land.
#[test]
fn goblin_burrows_pumps_a_goblin() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::goblin_burrows());
    let goblin = g.add_card_to_battlefield(0, catalog::greasewrench_goblin());
    let before = g.computed_permanent(goblin).unwrap().power;
    activate(&mut g, 0, land, 1, Some(Target::Permanent(goblin)));
    assert_eq!(g.computed_permanent(goblin).unwrap().power, before + 2);
}

/// Symbiotic Beast leaves four Insects behind.
#[test]
fn symbiotic_beast_leaves_four_insects() {
    let mut g = main_phase();
    let beast = g.add_card_to_battlefield(0, catalog::symbiotic_beast());
    let _ = g.remove_to_graveyard_with_triggers(beast);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Insect").count(), 4);
}

/// Searing Flesh throws seven at a face.
#[test]
fn searing_flesh_burns_for_seven() {
    let mut g = main_phase();
    let spell = g.add_card_to_hand(0, catalog::searing_flesh());
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 13);
}

/// Rorix Bladewing attacks the turn it lands.
#[test]
fn rorix_bladewing_is_hasty() {
    let mut g = main_phase();
    let rorix = g.add_card_to_hand(0, catalog::rorix_bladewing());
    cast(&mut g, 0, rorix, None);
    let id = g.battlefield.iter().find(|c| c.definition.name == "Rorix Bladewing").unwrap().id;
    let cp = g.computed_permanent(id).unwrap();
    assert!(cp.keywords.contains(&Keyword::Haste) && cp.keywords.contains(&Keyword::Flying));
}

/// Disciple of Malice can't be touched by white.
#[test]
fn disciple_of_malice_dodges_white() {
    let mut g = main_phase();
    let disciple = g.add_card_to_battlefield(0, catalog::disciple_of_malice());
    assert!(
        g.computed_permanent(disciple)
            .unwrap()
            .keywords
            .contains(&Keyword::Protection(Color::White))
    );
}

/// Wirewood Elf taps for green.
#[test]
fn wirewood_elf_taps_for_green() {
    let mut g = main_phase();
    let elf = g.add_card_to_battlefield(0, catalog::wirewood_elf());
    g.clear_sickness(elf);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: elf,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
}

/// Counters ride Onslaught's tribal bodies the same as anywhere else.
#[test]
fn onslaught_bodies_take_counters() {
    let mut g = main_phase();
    let elf = g.add_card_to_battlefield(0, catalog::wirewood_elf());
    g.battlefield_find_mut(elf).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert_eq!(g.computed_permanent(elf).unwrap().power, 2);
}
