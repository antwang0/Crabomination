//! Commander: the 2026-10-19 (`01GwrDKP`) closers (`decks::cmdr_01gw_a`..`_e`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].hostile_player_targets = true;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
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

/// Drain the stack and hand priority back to the active player (a three-seat
/// `drain_stack` ends on the next seat's priority).
fn settle(g: &mut GameState) {
    drain_stack(g);
    g.priority.player_with_priority = g.active_player_idx;
}

fn cast_x(g: &mut GameState, id: CardId, x: u32) {
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(x) })
        .expect("cast");
    drain_stack(g);
}

/// Opponents' total life lost from the 20 a test pod starts at.
fn opp_loss(g: &GameState) -> i32 {
    (1..g.players.len()).map(|s| 20 - g.players[s].life).sum()
}

/// CR 700.2 — "choose up to X": X = 0 runs no mode, X = 1 the first default
/// mode (target opponent loses 2), X = 2 also destroys the targeted token.
#[test]
fn ruinous_wrecking_crew_chooses_up_to_x_targeted_modes() {
    for (x, loss, token_dies) in [(0u32, 0, false), (1, 2, false), (2, 2, true)] {
        let mut g = pod(3);
        let food = g.add_token_to_battlefield(1, &crabomination::game::effects::food_token());
        let crew = g.add_card_to_hand(0, catalog::the_ruinous_wrecking_crew());
        flood(&mut g, 0);
        cast_x(&mut g, crew, x);
        let c = g.battlefield_find(crew).expect("on the battlefield");
        assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), x, "X={x} counters");
        assert_eq!(opp_loss(&g), loss, "X={x} drain");
        assert_eq!(g.battlefield_find(food).is_none(), token_dies, "X={x} token");
    }
}

/// CR 700.2 — an answer naming more modes than X is cut to X: with X = 1 a
/// three-mode answer runs only the first (the rummage), not the edict.
#[test]
fn ruinous_wrecking_crew_caps_an_overlong_answer_at_x() {
    let mut g = pod(3);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let crew = g.add_card_to_hand(0, catalog::the_ruinous_wrecking_crew());
    g.add_card_to_hand(0, catalog::forest());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Modes(vec![0, 3, 1])]));
    cast_x(&mut g, crew, 1);
    assert!(g.battlefield_find(bear).is_some(), "the edict did not run");
    assert!(g.battlefield_find(crew).is_some());
    assert_eq!(opp_loss(&g), 0, "the drain did not run");
    assert_eq!(g.players[0].graveyard.len(), 1, "rummaged a card");
}

/// CR 123.3 — Wee Champion pumps when its controller places a sticker
/// (_____ Goblin's ETB), not when the sticker is declined or an opponent
/// places one.
#[test]
fn wee_champion_grows_when_you_place_a_sticker() {
    let mut g = pod(2);
    g.set_sticker_sheets(0, [0, 1, 2]);
    g.set_sticker_sheets(1, [3, 4, 5]);
    let wee = g.add_card_to_battlefield(0, catalog::wee_champion());
    let power = |g: &GameState| g.computed_permanent(wee).unwrap().power;
    let goblin = |g: &mut GameState, seat: usize| {
        let id = g.add_card_to_hand(seat, catalog::blank_goblin());
        g.active_player_idx = seat;
        g.priority.player_with_priority = seat;
        flood(g, seat);
        g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
            .expect("goblin");
        drain_stack(g);
    };
    goblin(&mut g, 1);
    assert_eq!(power(&g), 0, "an opponent's sticker");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(99)]));
    goblin(&mut g, 0);
    assert_eq!(power(&g), 0, "declined");
    g.decider = Box::new(crabomination::decision::AutoDecider);
    goblin(&mut g, 0);
    assert_eq!(power(&g), 1, "placed a sticker");
}

fn treasures(g: &GameState, seat: usize) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == "Treasure").map(|c| c.id).collect()
}

/// Vazi hands an opponent one Treasure per Treasure its controller made this
/// turn; that opponent spending Treasure mana on a spell, or on an ability,
/// gives Vazi's controller a +1/+1 counter and a card.
#[test]
fn vazi_rewards_treasure_spent_by_an_opponent() {
    let mut g = pod(3);
    let vazi = g.add_card_to_battlefield(0, catalog::vazi_keen_negotiator());
    for _ in 0..2 {
        let sir = g.add_card_to_hand(0, catalog::strike_it_rich());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell { card_id: sir, target: None, additional_targets: vec![], mode: None, x_value: None })
            .expect("strike it rich");
        settle(&mut g);
    }
    g.perform_action(GameAction::ActivateAbility {
        card_id: vazi,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("vazi");
    settle(&mut g);
    let theirs = treasures(&g, 1);
    assert_eq!(theirs.len(), 2, "one per Treasure seat 0 made this turn");

    // Seat 1 spends one Treasure's mana on Sol Ring.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: theirs[0],
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("treasure mana");
    let ring = g.add_card_to_hand(1, catalog::sol_ring());
    g.perform_action(GameAction::CastSpell { card_id: ring, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("sol ring");
    settle(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    assert_eq!(g.battlefield_find(vazi).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);

    // Mana not from a Treasure rewards nothing.
    g.players[1].mana_pool.add_colorless(1);
    let ring2 = g.add_card_to_hand(1, catalog::sol_ring());
    g.perform_action(GameAction::CastSpell { card_id: ring2, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("sol ring 2");
    settle(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// CR 614 — Draugr Necromancer exiles an opponent's dying nontoken creature
/// with an ice counter instead, and its controller may cast that card; its own
/// creatures still die.
#[test]
fn draugr_necromancer_exiles_with_ice_and_casts_them() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::draugr_necromancer());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::savannah_lions());
    flood(&mut g, 0);
    for t in [bear, mine] {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(t)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("bolt");
        settle(&mut g);
    }
    assert!(g.players[0].graveyard.iter().any(|c| c.id == mine), "its controller's own creature dies");
    let iced = g.exile.iter().find(|c| c.id == bear).expect("exiled instead");
    assert_eq!(iced.counter_count(CounterType::Ice), 1);
    assert!(iced.may_play_until.is_some_and(|m| m.player == 0));
    let cast_bear = |g: &mut GameState| {
        g.perform_action(GameAction::CastFromZoneWithoutPaying {
            card_id: bear,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    g.players[0].mana_pool = Default::default();
    assert!(cast_bear(&mut g).is_err(), "it costs its own mana");
    flood(&mut g, 0);
    cast_bear(&mut g).expect("cast from exile");
    settle(&mut g);
    let b = g.battlefield_find(bear).expect("resolved");
    assert_eq!((b.owner, b.controller), (1, 0));
}

/// Vivid — Sanar (blue and red) reveals until two nonland cards and exiles one
/// of each color, castable this turn paying their costs; the rest are shuffled
/// back.
#[test]
fn sanar_vivid_exiles_one_card_per_color() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::sanar_innovative_first_year());
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    let counter = g.add_card_to_library(0, catalog::counterspell());
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.step = TurnStep::Upkeep;
    while g.step != TurnStep::PreCombatMain {
        g.perform_action(GameAction::PassPriority).ok();
    }
    settle(&mut g);
    for id in [bolt, counter] {
        let c = g.exile.iter().find(|c| c.id == id).expect("exiled");
        assert!(c.may_play_until.is_some_and(|m| m.player == 0 && m.cast_only));
    }
    assert!(g.players[0].library.iter().any(|c| c.id == bears), "past the reveal");
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the exiled bolt");
    settle(&mut g);
    assert_eq!(g.players[1].life, 17);
}

/// Grub's Command: destroy target creature, and target player mills five and
/// takes the Goblin cards among them into their hand.
#[test]
fn grubs_command_destroys_and_mills_goblins_home() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].library.clear();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::goblin_guide());
    }
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let cmd = g.add_card_to_hand(0, catalog::grubs_command());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: cmd,
        spree_modes: vec![2, 3],
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Player(0)],
        x_value: None,
    })
    .expect("grub's command");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "destroyed");
    let guides = g.players[0].hand.iter().filter(|c| c.definition.name == "Goblin Guide").count();
    assert_eq!(guides, 2, "the milled Goblins go to hand");
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Forest").count(), 3);
}

/// Grub returns a Goblin card on entry and transforms for {R} at the first main
/// phase; Notorious Auntie's attack blights 1 for a tapped, attacking copy of
/// the blighted creature, sacrificed at the next end step (CR 701.68).
#[test]
fn grub_returns_a_goblin_and_its_auntie_copies_the_blighted() {
    let mut g = pod(2);
    let guide = g.add_card_to_graveyard(0, catalog::goblin_guide());
    let grub = g.add_card_to_hand(0, catalog::grub_storied_matriarch());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: grub, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("grub");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == guide), "returned from the graveyard");
    let ogre = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.clear_sickness(ogre);
    g.clear_sickness(grub);
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(grub).unwrap().definition.name, "Grub, Notorious Auntie", "paid {{R}}");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Target(Target::Permanent(ogre))]));
    let before = g.battlefield.len();
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: grub,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.len(), before + 1, "a token copy");
    let copy = g.battlefield.iter().find(|c| c.is_token).expect("the copy");
    assert_eq!(copy.definition.name, "Hill Giant");
    assert!(copy.tapped);
    assert_eq!(g.battlefield_find(ogre).unwrap().counter_count(CounterType::MinusOneMinusOne), 1);
}

// ── `decks::cmdr_01gw_b` ────────────────────────────────────────────────────
/// Nobody bounces another artifact of yours; Rock Soldiers destroys an
/// Welcome to the Fold steals a toughness-2 creature but not a toughness-3 one
/// Boggart Mob: a Goblin of yours connecting makes a 1/1 Goblin Rogue.
#[test]
fn boggart_mob_makes_rogues_on_goblin_hits() {
    let mut g = pod(2);
    let guide = g.add_card_to_battlefield(0, catalog::goblin_guide());
    g.add_card_to_battlefield(0, catalog::boggart_mob());
    g.clear_sickness(guide);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: guide,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::EndCombat && g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).ok();
    }
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Goblin Rogue"));
}

/// Hei Bai digs a Shrine out of the library onto the battlefield; its WUBRG
/// ability makes a Spirit per legendary enchantment you control.
#[test]
fn hei_bai_finds_a_shrine_and_makes_spirits() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::sanctum_of_stone_fangs());
    let hb = g.add_card_to_hand(0, catalog::hei_bai_forest_guardian());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastSpell { card_id: hb, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("hei bai");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Sanctum of Stone Fangs"), "the Shrine came out");
    g.add_card_to_battlefield(0, catalog::sanctum_of_all());
    g.clear_sickness(hb);
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hb,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("spirits");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Spirit").count(), 2, "two legendary enchantments");
}

/// Sanctum of All: with six Shrines, another Shrine's trigger fires twice
/// (Stone Fangs drains six, twice); with five Shrines, once.
#[test]
fn sanctum_of_all_doubles_shrine_triggers_at_six() {
    for (others, drained) in [(2, 5), (3, 2 * 6)] {
        let mut g = pod(2);
        g.add_card_to_battlefield(0, catalog::sanctum_of_all());
        g.add_card_to_battlefield(0, catalog::sanctum_of_stone_fangs());
        g.add_card_to_battlefield(0, catalog::northern_air_temple());
        let extra = [catalog::southern_air_temple(), catalog::crescent_island_temple(), catalog::kyoshi_island_plaza()];
        for def in extra.into_iter().take(others) {
            g.add_card_to_battlefield(0, def);
        }
        let life = g.players[1].life;
        g.step = TurnStep::PreCombatMain;
        g.fire_step_triggers(TurnStep::PreCombatMain);
        drain_stack(&mut g);
        assert_eq!(life - g.players[1].life, drained, "{} Shrines", others + 3);
    }
}

// ── `decks::cmdr_01gw_c` ────────────────────────────────────────────────────

/// CR 702.62b — Curse of the Cabal's upkeep trigger functions from exile while
/// it is suspended: a player may sacrifice a permanent at their upkeep to put
/// two more time counters on it.
#[test]
fn curse_of_the_cabal_delays_for_a_sacrifice() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let curse = g.add_card_to_hand(0, catalog::curse_of_the_cabal());
    flood(&mut g, 0);
    g.perform_action(GameAction::Suspend { card_id: curse }).expect("suspend");
    drain_stack(&mut g);
    let time = |g: &GameState| g.exile.iter().find(|c| c.id == curse).map(|c| c.counter_count(CounterType::Time));
    assert_eq!(time(&g), Some(2));
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "sacrificed");
    assert_eq!(time(&g), Some(4));
}

/// Neglected Heirloom transforms with the werewolf it equips: Afflicted
/// Deserter flips at a spell-less upkeep and the Heirloom becomes Ashmouth
/// Blade (+3/+3, first strike).
#[test]
fn neglected_heirloom_flips_with_its_werewolf() {
    let mut g = pod(2);
    let wolf = g.add_card_to_battlefield(0, catalog::afflicted_deserter());
    let heirloom = g.add_card_to_battlefield(0, catalog::neglected_heirloom());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: heirloom, target: wolf }).expect("equip");
    drain_stack(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(wolf).unwrap().definition.name, "Werewolf Ransacker");
    assert_eq!(g.battlefield_find(heirloom).unwrap().definition.name, "Ashmouth Blade");
    let cp = g.computed_permanent(wolf).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 7));
}

// ── `decks::cmdr_01gw_d` ────────────────────────────────────────────────────

/// Taster of Wares: the opponent reveals one card per Goblin, the controller
/// picks the instant among them, and may cast it while Taster stays, paying
/// with mana of any type.
#[test]
fn taster_of_wares_steals_a_revealed_instant() {
    let mut g = pod(2);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::forest());
    let taster = g.add_card_to_hand(0, catalog::taster_of_wares());
    g.add_card_to_battlefield(0, catalog::goblin_guide());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: taster,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("taster");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt), "two Goblins: both cards shown, the bolt chosen");
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast it with green");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
}

/// Champion of the Weird: pay 1 life and blight 2 to make an opponent blight 2.
#[test]
fn champion_of_the_weird_makes_an_opponent_blight() {
    let mut g = pod(2);
    let champ = g.add_card_to_battlefield(0, catalog::champion_of_the_weird());
    let theirs = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.perform_action(GameAction::ActivateAbility {
        card_id: champ,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("blight");
    drain_stack(&mut g);
    use crabomination::card::CounterType;
    assert_eq!(g.players[0].life, 19);
    assert_eq!(g.battlefield_find(champ).unwrap().counter_count(CounterType::MinusOneMinusOne), 2);
    assert_eq!(g.battlefield_find(theirs).unwrap().counter_count(CounterType::MinusOneMinusOne), 2);
}

/// CR 508.2 / 802.2 — Noctis's warp-strike exiles it; at its controller's
/// next declare attackers step — even one with no attackers declared — it
/// returns tapped and attacking an opponent (every opponent is a defending
/// player in a multiplayer game), and can't be blocked that combat.
#[test]
fn noctis_warp_strikes_back_into_combat() {
    let mut g = pod(3);
    let noctis = g.add_card_to_battlefield(0, catalog::noctis_heir_apparent());
    g.clear_sickness(noctis);
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: noctis,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("warp-strike");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == noctis), "exiled");
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![])).expect("no attackers");
    drain_stack(&mut g);
    let n = g.battlefield_find(noctis).expect("returned");
    assert!(n.tapped);
    assert!(g.attacking.iter().any(|a| a.attacker == noctis), "attacking");
    assert!(g.computed_permanent(noctis).unwrap().keywords().contains(&crabomination::card::Keyword::Unblockable));
}

// ── `decks::cmdr_01gw_e` ────────────────────────────────────────────────────

/// Warren Pilferers returns a creature card; a Goblin card hastes it.
#[test]
fn warren_pilferers_hastes_for_a_goblin() {
    for (def, haste) in [(catalog::goblin_guide(), true), (catalog::grizzly_bears(), false)] {
        let mut g = pod(2);
        let back = g.add_card_to_graveyard(0, def);
        let wp = g.add_card_to_hand(0, catalog::warren_pilferers());
        flood(&mut g, 0);
        g.perform_action(GameAction::CastSpell { card_id: wp, target: None, additional_targets: vec![], mode: None, x_value: None })
            .expect("pilferers");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == back));
        let kw = g.computed_permanent(wp).unwrap().keywords().contains(&crabomination::card::Keyword::Haste);
        assert_eq!(kw, haste);
    }
}

/// Scrapper Champion's attack trigger turns two energy into a +1/+1 counter.
#[test]
fn scrapper_champion_spends_energy_for_a_counter() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let sc = g.add_card_to_hand(0, catalog::scrapper_champion());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: sc, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("champion");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 2);
    g.clear_sickness(sc);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: sc,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 0);
    assert_eq!(g.battlefield_find(sc).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Deadlock Trap taps a creature and locks its activated abilities this turn.
#[test]
fn deadlock_trap_taps_and_locks() {
    let mut g = pod(2);
    let trap = g.add_card_to_battlefield(0, catalog::deadlock_trap());
    g.battlefield_find_mut(trap).unwrap().tapped = false;
    g.players[0].energy = 1;
    let elf = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    g.perform_action(GameAction::ActivateAbility {
        card_id: trap,
        ability_index: 0,
        target: Some(Target::Permanent(elf)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("trap");
    drain_stack(&mut g);
    assert!(g.battlefield_find(elf).unwrap().tapped);
    assert_eq!(g.players[0].energy, 0);
}

// ── `decks::cmdr_01gw_f` ────────────────────────────────────────────────────

/// Sentinel of Lost Lore's three modes: an exiled Adventure card of yours
/// comes home, an opponent's goes to the bottom of their library, and the
/// opponent's graveyard is exiled. Regressions: the trigger's extra-slot fill
/// never searched exile, and `slot_owner` read a `ChooseN`'s third slot as no
/// mode's, so its hostile player slot aimed at the controller's own graveyard.
#[test]
fn sentinel_of_lost_lore_runs_its_modes() {
    let mut g = pod(2);
    let mine = g.add_card_to_exile(0, catalog::bonecrusher_giant());
    let theirs = g.add_card_to_exile(1, catalog::bonecrusher_giant());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::sentinel_of_lost_lore());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: s, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("sentinel");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == mine), "home");
    assert_eq!(g.players[1].library.last().map(|c| c.id), Some(theirs), "bottom of its owner's library");
    assert!(g.players[1].graveyard.is_empty(), "graveyard exiled");
}

// ── `decks::cmdr_01gw_g` ────────────────────────────────────────────────────

/// Righteous Fury destroys only tapped creatures and gains 2 life apiece.
#[test]
fn righteous_fury_punishes_the_tapped() {
    let mut g = pod(2);
    let tapped = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::hill_giant());
    let untapped = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for id in [tapped, mine] {
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    let rf = g.add_card_to_hand(0, catalog::righteous_fury());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: rf, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("fury");
    drain_stack(&mut g);
    assert!(g.battlefield_find(tapped).is_none() && g.battlefield_find(mine).is_none());
    assert!(g.battlefield_find(untapped).is_some());
    assert_eq!(g.players[0].life, 24);
}
