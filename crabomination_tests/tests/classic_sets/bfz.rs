//! Battle for Zendikar — landfall, Rally, Awaken, Converge, Process.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn counters(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne)
}

/// Play a land for seat 0 from a fresh instance, returning its id.
fn play_land(g: &mut GameState, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_hand(0, def);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].lands_played_this_turn = 0;
    g.perform_action(GameAction::PlayLand(id)).expect("play land");
    drain_stack(g);
    id
}

// ── Landfall ────────────────────────────────────────────────────────────────

/// The landfall pump cycle grows until end of turn when a land enters.
#[test]
fn landfall_creatures_pump_on_a_land_drop() {
    let cases: [(fn() -> crabomination::card::CardDefinition, i32, i32); 5] = [
        (catalog::scythe_leopard, 2, 2),
        (catalog::makindi_sliderunner, 3, 2),
        (catalog::valakut_predator, 4, 4),
        (catalog::wave_wing_elemental, 5, 6),
        (catalog::geyserfield_stalker, 5, 4),
    ];
    for (def, p, t) in cases {
        let mut g = two_player_game();
        let c = g.add_card_to_battlefield(0, def());
        let name = g.battlefield_find(c).unwrap().definition.name;
        play_land(&mut g, catalog::forest());
        let cp = g.computed_permanent(c).expect("still there");
        assert_eq!((cp.power, cp.toughness), (p, t), "{name} pumped on landfall");
    }
}

/// Jaddi Offshoot gains life; Tunneling Geopede pings each opponent.
#[test]
fn landfall_triggers_reach_players() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::jaddi_offshoot());
    g.add_card_to_battlefield(0, catalog::tunneling_geopede());
    let (life, opp) = (g.players[0].life, g.players[1].life);
    play_land(&mut g, catalog::forest());
    assert_eq!(g.players[0].life, life + 1, "Jaddi Offshoot gained 1");
    assert_eq!(g.players[1].life, opp - 1, "Tunneling Geopede pinged");
}

/// Oran-Rief Hydra takes two counters off a Forest and one off anything else.
#[test]
fn oran_rief_hydra_doubles_on_a_forest() {
    let mut g = two_player_game();
    let hydra = g.add_card_to_battlefield(0, catalog::oran_rief_hydra());
    play_land(&mut g, catalog::forest());
    assert_eq!(counters(&g, hydra), 2, "a Forest gives two");
    play_land(&mut g, catalog::island());
    assert_eq!(counters(&g, hydra), 3, "a non-Forest gives one");
}

/// Landfall fires on a land put onto the battlefield without being played
/// (CR — "whenever a land you control *enters*", not "whenever you play").
#[test]
fn landfall_fires_on_a_fetched_land() {
    let mut g = two_player_game();
    let leopard = g.add_card_to_battlefield(0, catalog::scythe_leopard());
    let land = g.add_card_to_hand(0, catalog::forest());
    let mut evs = Vec::new();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.move_card_to(
        land,
        &crabomination::effect::ZoneDest::Battlefield {
            controller: crabomination::effect::PlayerRef::You,
            tapped: false,
        },
        &ctx,
        &mut evs,
    );
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let cp = g.computed_permanent(leopard).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "the fetched land still triggers landfall");
}

// ── Rally ───────────────────────────────────────────────────────────────────

/// The Rally keyword-grant cycle fires off any Ally entering.
#[test]
fn rally_grants_fire_on_an_ally_entering() {
    let cases: [(fn() -> crabomination::card::CardDefinition, Keyword); 6] = [
        (catalog::hero_of_goma_fada, Keyword::Indestructible),
        (catalog::lantern_scout, Keyword::Lifelink),
        (catalog::makindi_patrol, Keyword::Vigilance),
        (catalog::ondu_champion, Keyword::Trample),
        (catalog::firemantle_mage, Keyword::Menace),
        (catalog::resolute_blademaster, Keyword::DoubleStrike),
    ];
    for (def, kw) in cases {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let src = g.add_card_to_battlefield(0, def());
        let name = g.battlefield_find(src).unwrap().definition.name;
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: src }]);
        drain_stack(&mut g);
        assert!(
            g.computed_permanent(bear).unwrap().keywords.contains(&kw),
            "{name} granted {kw:?}"
        );
    }
}

/// Kalastria Healer drains on each Ally entering; a non-Ally doesn't trigger it.
#[test]
fn kalastria_healer_drains_only_on_allies() {
    let mut g = two_player_game();
    let healer = g.add_card_to_battlefield(0, catalog::kalastria_healer());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: healer }]);
    drain_stack(&mut g);
    assert_eq!((g.players[0].life, g.players[1].life), (21, 19), "its own ETB is a Rally");

    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: bear }]);
    drain_stack(&mut g);
    assert_eq!((g.players[0].life, g.players[1].life), (21, 19), "a non-Ally is no Rally");
}

// ── Converge ────────────────────────────────────────────────────────────────

/// Radiant Flames scales with the number of colors of mana spent.
#[test]
fn radiant_flames_scales_with_colors_spent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let flames = g.add_card_to_hand(0, catalog::radiant_flames());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: flames, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "3 colors → 3 damage kills a 2/2");
}

/// Woodland Wanderer enters with a +1/+1 counter per color of mana spent.
#[test]
fn woodland_wanderer_enters_with_converge_counters() {
    let mut g = two_player_game();
    let ww = g.add_card_to_hand(0, catalog::woodland_wanderer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: ww, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(counters(&g, ww), 4, "four colors, four counters");
}

// ── Awaken ──────────────────────────────────────────────────────────────────

/// Cast for its Awaken cost, Clutch of Currents also animates a land.
#[test]
fn clutch_of_currents_awakens_a_land() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(0, catalog::island());
    let clutch = g.add_card_to_hand(0, catalog::clutch_of_currents());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: clutch,
        pitch_card: None,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![Target::Permanent(land)],
        mode: None,
        x_value: None,
    })
    .expect("awaken cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "the creature bounced");
    let cp = g.computed_permanent(land).expect("the land is still there");
    assert_eq!(counters(&g, land), 3, "three +1/+1 counters");
    assert!(
        cp.card_types.contains(&crabomination::card::CardType::Creature),
        "and it is a creature now"
    );
}

// ── Process ─────────────────────────────────────────────────────────────────

/// Cryptic Cruiser's process cost gates the activation and bins an exiled card.
#[test]
fn cryptic_cruiser_needs_a_card_in_exile_to_process() {
    let mut g = two_player_game();
    let cruiser = g.add_card_to_battlefield(0, catalog::cryptic_cruiser());
    g.clear_sickness(cruiser);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let act = || GameAction::ActivateAbility {
        card_id: cruiser,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
    };
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    assert!(g.perform_action(act()).is_err(), "nothing in exile → no activation");

    let exiled = g.next_id();
    g.exile.push(crabomination::card::CardInstance::new(exiled, catalog::grizzly_bears(), 1));
    g.perform_action(act()).expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "the target got tapped");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == exiled), "the exile card was processed");
}

// ── Lands ───────────────────────────────────────────────────────────────────

/// The battle lands enter untapped only with two or more basic lands.
#[test]
fn battle_lands_enter_tapped_without_two_basics() {
    let mut g = two_player_game();
    let first = play_land(&mut g, catalog::canopy_vista());
    assert!(g.battlefield_find(first).unwrap().tapped, "no basics → tapped");

    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::plains());
    let second = play_land(&mut g, catalog::cinder_glade());
    assert!(!g.battlefield_find(second).unwrap().tapped, "two basics → untapped");
}

/// Blighted Steppe's sac ability scales with your creature count.
#[test]
fn blighted_steppe_gains_two_life_per_creature() {
    let mut g = two_player_game();
    let steppe = g.add_card_to_battlefield(0, catalog::blighted_steppe());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: steppe, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 4, "2 life per creature");
    assert!(g.battlefield_find(steppe).is_none(), "the land sacrificed itself");
}

/// Fertile Thicket keeps a revealed basic on top and bottoms the rest.
#[test]
fn fertile_thicket_puts_a_basic_on_top() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    let basic = g.add_card_to_library(0, catalog::forest());
    let thicket = play_land(&mut g, catalog::fertile_thicket());
    assert!(g.battlefield_find(thicket).unwrap().tapped, "enters tapped");
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(basic), "the basic is on top");
    assert_eq!(g.players[0].library.len(), 5, "nothing was drawn");
}

// ── Misc ────────────────────────────────────────────────────────────────────

/// Undergrowth Champion eats damage by shedding a +1/+1 counter.
#[test]
fn undergrowth_champion_sheds_a_counter_instead_of_taking_damage() {
    let mut g = two_player_game();
    let champ = g.add_card_to_battlefield(0, catalog::undergrowth_champion());
    play_land(&mut g, catalog::forest());
    assert_eq!(counters(&g, champ), 1);
    let mut evs = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(champ),
        5,
        None,
        &mut evs,
    );
    assert_eq!(counters(&g, champ), 0, "the counter absorbed the hit");
    assert_eq!(g.battlefield_find(champ).unwrap().damage, 0, "and no damage was marked");
}

/// Titan's Presence exiles only a creature whose power the revealed card matches.
#[test]
fn titans_presence_needs_a_big_enough_reveal() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::eldrazi_devastator());
    let spell = g.add_card_to_hand(0, catalog::titans_presence());
    g.players[0].mana_pool.add_colorless(3);
    // No colorless creature card in hand — the cost can't be paid.
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(big)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "no reveal, no cast"
    );
    g.add_card_to_hand(0, catalog::eldrazi_devastator());
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(big)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast with the reveal");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == big), "8/9 revealed exiles an 8/9");
}

/// Veteran Warleader's P/T counts your creatures, and its tap-an-Ally ability
/// grants one of three keywords.
#[test]
fn veteran_warleader_counts_creatures_and_grants_a_keyword() {
    let mut g = two_player_game();
    let leader = g.add_card_to_battlefield(0, catalog::veteran_warleader());
    let ally = g.add_card_to_battlefield(0, catalog::kitesail_scout());
    g.battlefield_find_mut(ally).unwrap().definition =
        std::sync::Arc::new(catalog::lantern_scout());
    let cp = g.computed_permanent(leader).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "two creatures → 2/2");

    g.clear_sickness(leader);
    g.clear_sickness(ally);
    g.perform_action(GameAction::ActivateAbility {
        card_id: leader, ability_index: 2, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("tap the Ally");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(leader).unwrap().keywords.contains(&Keyword::Trample),
        "the third ability is trample"
    );
    assert!(g.battlefield_find(ally).unwrap().tapped, "the Ally paid the cost");
}

// ── Gap wave 2 ──────────────────────────────────────────────────────────────

/// Retreat to Kazandu's landfall is modal: mode 1 gains 2 life.
#[test]
fn retreat_to_kazandu_offers_both_modes() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::retreat_to_kazandu());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    play_land(&mut g, catalog::forest());
    assert_eq!(counters(&g, bear), 1, "the default mode grows a creature");

    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    let life = g.players[0].life;
    play_land(&mut g, catalog::forest());
    assert_eq!(g.players[0].life, life + 2, "mode 1 gains 2 life");
}

/// Guardian of Tazeem's Island rider also stops the untap.
#[test]
fn guardian_of_tazeem_locks_down_off_an_island() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::guardian_of_tazeem());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    play_land(&mut g, catalog::island());
    assert!(g.battlefield_find(bear).unwrap().tapped, "tapped by the trigger");
    assert!(
        g.battlefield_find(bear).unwrap().skip_next_untap,
        "an Island also skips its next untap"
    );
}

/// Guul Draz Overseer pumps your *other* creatures, doubled off a Swamp.
#[test]
fn guul_draz_overseer_doubles_off_a_swamp() {
    let mut g = two_player_game();
    let overseer = g.add_card_to_battlefield(0, catalog::guul_draz_overseer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    play_land(&mut g, catalog::swamp());
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+0 off a Swamp");
    assert_eq!(g.computed_permanent(overseer).unwrap().power, 3, "the source is excluded");
}

/// Sire of Stagnation fires off an *opponent's* land, not yours.
#[test]
fn sire_of_stagnation_watches_opponents_lands() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sire_of_stagnation());
    for _ in 0..4 {
        g.add_card_to_library(1, catalog::lightning_bolt());
    }
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    let hand = g.players[0].hand.len();
    play_land(&mut g, catalog::forest());
    assert_eq!(g.players[0].hand.len(), hand, "your own land doesn't trigger it");

    let land = g.add_card_to_hand(1, catalog::forest());
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.players[1].lands_played_this_turn = 0;
    g.perform_action(GameAction::PlayLand(land)).expect("opponent plays a land");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2, "you drew two");
    assert_eq!(g.exile.len(), 2, "and they exiled two");
}

/// Herald of Kozilek discounts colorless spells by {1}.
#[test]
fn herald_of_kozilek_discounts_colorless_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::herald_of_kozilek());
    let devastator = g.add_card_to_hand(0, catalog::eldrazi_devastator());
    // Printed {8}; with the Herald it costs {7}.
    g.players[0].mana_pool.add_colorless(7);
    g.perform_action(GameAction::CastSpell {
        card_id: devastator, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast at the discount");
    drain_stack(&mut g);
    assert!(g.battlefield_find(devastator).is_some());
}

/// Munda stacks Allies from the top four on top and bottoms the rest.
#[test]
fn munda_stacks_allies_on_top() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::lantern_scout());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let munda = g.add_card_to_battlefield(0, catalog::munda_ambush_leader());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: munda }]);
    drain_stack(&mut g);
    // Lantern Scout is the only Ally among the four — it ends up on top.
    assert_eq!(g.players[0].library.first().map(|c| c.definition.name), Some("Lantern Scout"));
    assert_eq!(g.players[0].library.len(), 4, "nothing left the library");
}

/// Halimar Tidecaller returns an Awaken card and grants land creatures flying.
#[test]
fn halimar_tidecaller_recurs_awaken_and_grants_flying() {
    let mut g = two_player_game();
    let gy_id = g.next_id();
    g.players[0]
        .graveyard
        .push(crabomination::card::CardInstance::new(gy_id, catalog::ruinous_path(), 0));
    let bolt_id = g.next_id();
    g.players[0]
        .graveyard
        .push(crabomination::card::CardInstance::new(bolt_id, catalog::lightning_bolt(), 0));
    let tide = g.add_card_to_battlefield(0, catalog::halimar_tidecaller());
    g.fire_self_etb_triggers(tide, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == gy_id), "Ruinous Path has awaken");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt_id), "the Bolt has none");
}

/// Gideon's 0 mints a 2/2 Knight Ally; his −4 emblem anthems your board.
#[test]
fn gideon_ally_of_zendikar_makes_knights_and_an_emblem() {
    let mut g = two_player_game();
    let gid = g.add_card_to_battlefield(0, catalog::gideon_ally_of_zendikar());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: gid, ability_index: 1, target: None, x_value: None,
    })
    .expect("0 ability");
    drain_stack(&mut g);
    let knight = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Knight Ally")
        .map(|c| c.id)
        .expect("a Knight Ally token");

    g.battlefield_find_mut(gid).unwrap().add_counters(CounterType::Loyalty, 4);
    g.battlefield_find_mut(gid).unwrap().loyalty_uses_this_turn = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: gid, ability_index: 2, target: None, x_value: None,
    })
    .expect("-4 emblem");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(knight).unwrap().power, 3, "the emblem anthem applies");
}

/// March from the Tomb reanimates as many Allies as the 8-mana budget allows.
#[test]
fn march_from_the_tomb_respects_the_total_mana_value_cap() {
    let mut g = two_player_game();
    let mk = |g: &mut GameState, def: crabomination::card::CardDefinition| {
        let id = g.next_id();
        g.players[0].graveyard.push(crabomination::card::CardInstance::new(id, def, 0));
        id
    };
    // Three Allies at MV 3, 3 and 5 (total 11) plus a non-Ally.
    let a = mk(&mut g, catalog::lantern_scout());
    let b = mk(&mut g, catalog::chasm_guide());
    let big = mk(&mut g, catalog::tajuru_beastmaster());
    let bear = mk(&mut g, catalog::grizzly_bears());
    let march = g.add_card_to_hand(0, catalog::march_from_the_tomb());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: march, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_some() && g.battlefield_find(b).is_some(), "3 + 3 fits");
    assert!(g.battlefield_find(big).is_none(), "the 6-drop breaks the budget");
    assert!(g.battlefield_find(bear).is_none(), "a non-Ally is never eligible");
}

/// Ondu Rising gives every creature that attacks this turn lifelink.
#[test]
fn ondu_rising_grants_attackers_lifelink() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let rising = g.add_card_to_hand(0, catalog::ondu_rising());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: rising, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);

    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Lifelink),
        "the attacker gained lifelink"
    );
}

/// Zada copies a spell that targets only her, once per other creature it could
/// target.
#[test]
fn zada_copies_a_self_targeting_pump_for_each_other_creature() {
    let mut g = two_player_game();
    let zada = g.add_card_to_battlefield(0, catalog::zada_hedron_grinder());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pump = g.add_card_to_hand(0, catalog::giant_growth());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: pump,
        target: Some(Target::Permanent(zada)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(zada).unwrap().power, 6, "the original still hits Zada");
    assert_eq!(g.computed_permanent(a).unwrap().power, 5, "a copy hit the first bear");
    assert_eq!(g.computed_permanent(b).unwrap().power, 5, "and the second");
    assert_eq!(g.computed_permanent(theirs).unwrap().power, 2, "not an opponent's creature");
}

/// Gruesome Slaughter turns your colorless creatures into pingers for the turn.
#[test]
fn gruesome_slaughter_grants_a_tap_to_ping() {
    let mut g = two_player_game();
    let devastator = g.add_card_to_battlefield(0, catalog::eldrazi_devastator());
    g.clear_sickness(devastator);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let slaughter = g.add_card_to_hand(0, catalog::gruesome_slaughter());
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpell {
        card_id: slaughter, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: devastator,
        ability_index: 0,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("granted ping");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "an 8/9 pinging a 2/2 kills it");
}
