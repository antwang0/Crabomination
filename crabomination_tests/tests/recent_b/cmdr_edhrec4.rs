//! Commander: EDHREC average-deck gaps, fourth batch (`decks::cmdr_edhrec4`):
//! Shroofus Sproutsire's Saprolings; Miirym's and Tiamat's Dragons.

use crabomination::card::{CardId, CounterType, Keyword, Value};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast(g: &mut GameState, id: CardId) {
    cast_at(g, id, None);
}

fn cast_at(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, ability_index: usize) -> Result<(), crabomination::game::GameError> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == name).count()
}

fn kill(g: &mut GameState, id: CardId) {
    let ctx = EffectContext::for_spell(1, None, 0, 0);
    let evs = g.resolve_effect(&Effect::Destroy { what: Selector::ExactObjects(vec![id]) }, &ctx).expect("destroy");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

/// Sporecrown Thallid pumps other Fungi and Saprolings only; the Thallid
/// cycle banks a spore counter each upkeep and cashes three for a
/// Saproling; Utopia Mycon's Saproling is a mana ability.
#[test]
fn thallids_bank_spores_into_saprolings() {
    let mut g = pod(2);
    let crown = ready(&mut g, 0, catalog::sporecrown_thallid());
    let mycon = ready(&mut g, 0, catalog::utopia_mycon());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(mycon).unwrap().power, 1, "a Fungus");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "not a Fungus");
    assert_eq!(g.computed_permanent(crown).unwrap().power, 2, "not itself");
    for turn in 0..3 {
        assert!(activate(&mut g, mycon, 0).is_err() || turn == 3, "not yet three spores");
        g.step = TurnStep::Upkeep;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(mycon).unwrap().counter_count(CounterType::Spore), 3);
    activate(&mut g, mycon, 0).expect("remove three");
    assert_eq!(named(&g, "Saproling"), 1);
    let sap = g.battlefield.iter().find(|c| c.definition.name == "Saproling").unwrap().id;
    assert_eq!(g.computed_permanent(sap).unwrap().power, 2, "Sporecrown");
    g.players[0].mana_pool = Default::default();
    activate(&mut g, mycon, 1).expect("sacrifice for mana");
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

/// Fungal Sprouting counts the greatest power; Might of the Masses counts
/// creatures you control; Undercellar Myconid makes one entering and one dying.
#[test]
fn saproling_spells_scale_with_the_board() {
    let mut g = pod(2);
    let myconid = g.add_card_to_hand(0, catalog::undercellar_myconid());
    flood(&mut g);
    cast(&mut g, myconid);
    assert_eq!(named(&g, "Saproling"), 1);
    ready(&mut g, 0, catalog::colossal_dreadmaw());
    let sprouting = g.add_card_to_hand(0, catalog::fungal_sprouting());
    cast(&mut g, sprouting);
    assert_eq!(named(&g, "Saproling"), 7, "the Dreadmaw's 6");
    kill(&mut g, myconid);
    assert_eq!(named(&g, "Saproling"), 8, "and one on death");
}

/// Fungal Plots: the exile is a creature card from your graveyard, and the
/// two-Saproling sacrifice gains 2 and draws.
#[test]
fn fungal_plots_turns_corpses_into_cards() {
    let mut g = pod(2);
    let plots = ready(&mut g, 0, catalog::fungal_plots());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    flood(&mut g);
    assert!(activate(&mut g, plots, 0).is_err(), "no creature card to exile");
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    activate(&mut g, plots, 0).expect("exile one");
    activate(&mut g, plots, 0).expect("exile the other");
    let hand = g.players[0].hand.len();
    activate(&mut g, plots, 1).expect("two Saprolings");
    assert_eq!((g.players[0].life, g.players[0].hand.len()), (22, hand + 1));
}

fn advance_to_turn_of(g: &mut GameState, seat: usize) {
    loop {
        let ev = g.advance_step(Vec::new()).expect("step");
        g.dispatch_triggers_for_events(&ev);
        drain_stack(g);
        if g.active_player_idx == seat && g.step == TurnStep::PreCombatMain {
            break;
        }
    }
}

fn has_kw(g: &GameState, id: CardId, kw: Keyword) -> bool {
    g.computed_permanent(id).is_some_and(|c| c.keywords().contains(&kw))
}

/// CR 106.6 — Jade Orb's rider: the Dragon creature spell its {G} paid for
/// enters with a +1/+1 counter and hexproof that lasts until its caster's
/// next turn; Scaled Nurturer's gains 2 life. Mana from elsewhere does neither.
#[test]
fn dragon_mana_riders_name_the_dragon_they_paid_for() {
    let mut g = pod(2);
    let orb = ready(&mut g, 0, catalog::jade_orb_of_dragonkind());
    activate(&mut g, orb, 0).expect("Orb's {G}");
    g.players[0].mana_pool.add_colorless(1);
    let whelp = g.add_card_to_hand(0, catalog::scaled_nurturer());
    cast(&mut g, whelp);
    assert_eq!(g.battlefield_find(whelp).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(has_kw(&g, whelp, Keyword::Hexproof));
    let nurturer = ready(&mut g, 0, catalog::scaled_nurturer());
    activate(&mut g, nurturer, 0).expect("Nurturer's {G}");
    g.players[0].mana_pool.add_colorless(1);
    let second = g.add_card_to_hand(0, catalog::scaled_nurturer());
    cast(&mut g, second);
    assert_eq!(g.players[0].life, 22, "a Dragon paid for by the Nurturer");
    assert!(!has_kw(&g, second, Keyword::Hexproof), "not the Orb's mana");
    // The hexproof ends as its caster's next turn begins.
    stock_libraries(&mut g, 5);
    advance_to_turn_of(&mut g, 1);
    assert!(has_kw(&g, whelp, Keyword::Hexproof), "through the opponent's turn");
    advance_to_turn_of(&mut g, 0);
    assert!(!has_kw(&g, whelp, Keyword::Hexproof));
}

/// Earthquake Dragon costs {X} less for the total mana value of your
/// Dragons; Korlessa casts Dragon spells off the top of the library.
#[test]
fn dragons_make_dragons_cheaper_and_castable_from_the_top() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::thrakkus_the_butcher());
    ready(&mut g, 0, catalog::korlessa_scale_singer());
    let quake = g.add_card_to_library(0, catalog::earthquake_dragon());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(7);
    cast(&mut g, quake);
    assert!(g.battlefield_find(quake).is_some(), "{{14}}{{G}} less 5 + 2, from the top");
    assert_eq!(g.players[0].mana_pool.total(), 0);
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
            .is_err(),
        "not a Dragon"
    );
}

/// Rith: an opponent's creature dealt excess damage (CR 120.10) makes the
/// end-step Dragon; lethal-exact damage does not.
#[test]
fn rith_rewards_excess_damage() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::rith_liberated_primeval());
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 2, catalog::grizzly_bears());
    let burn = |g: &mut GameState, id: CardId, n: i32| {
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        let evs = g
            .resolve_effect(&Effect::DealDamage { to: Selector::ExactObjects(vec![id]), amount: Value::Const(n) }, &ctx)
            .expect("burn");
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    };
    let end_step = |g: &mut GameState| {
        g.step = TurnStep::End;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(g);
    };
    burn(&mut g, a, 2);
    end_step(&mut g);
    assert_eq!(named(&g, "Dragon"), 0, "exactly lethal");
    burn(&mut g, b, 5);
    end_step(&mut g);
    assert_eq!(named(&g, "Dragon"), 1);
}

/// Rivaz: one Dragon creature spell a turn from the graveyard, and it comes
/// back with "When this creature dies, exile it."
#[test]
fn rivaz_recasts_a_dragon_that_dies_into_exile() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::rivaz_of_the_claw());
    let first = g.add_card_to_graveyard(0, catalog::scaled_nurturer());
    let second = g.add_card_to_graveyard(0, catalog::scaled_nurturer());
    flood(&mut g);
    cast(&mut g, first);
    assert!(g.battlefield_find(first).is_some());
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell { card_id: second, target: None, additional_targets: vec![], mode: None, x_value: None })
            .is_err(),
        "once a turn"
    );
    kill(&mut g, first);
    assert!(g.exile.iter().any(|c| c.id == first), "its granted trigger exiled it");
}

/// Dragon's Fire: 3 damage, or a beheld Dragon's power when that is more
/// (CR 701.4a — a Dragon card revealed from hand counts).
#[test]
fn dragons_fire_reads_the_beheld_dragons_power() {
    let mut g = pod(2);
    let maw = ready(&mut g, 1, catalog::colossal_dreadmaw());
    flood(&mut g);
    let fire = g.add_card_to_hand(0, catalog::dragons_fire());
    cast_at(&mut g, fire, Some(Target::Permanent(maw)));
    assert_eq!(g.battlefield_find(maw).unwrap().damage, 3);
    g.add_card_to_hand(0, catalog::wrathful_red_dragon());
    let maw2 = ready(&mut g, 1, catalog::colossal_dreadmaw());
    let fire2 = g.add_card_to_hand(0, catalog::dragons_fire());
    cast_at(&mut g, fire2, Some(Target::Permanent(maw2)));
    assert_eq!(g.battlefield_find(maw2).unwrap().damage, 5, "the revealed 5/5");
}

/// Thrakkus doubles each Dragon's power as it attacks; Sarkhan Unbroken's
/// −8 puts every Dragon creature card in the library onto the battlefield.
#[test]
fn thrakkus_and_sarkhan_scale_the_dragons() {
    let mut g = pod(2);
    let thrakkus = ready(&mut g, 0, catalog::thrakkus_the_butcher());
    let wrath = ready(&mut g, 0, catalog::wrathful_red_dragon());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: thrakkus, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(wrath).unwrap().power, 10);
    assert_eq!(g.computed_permanent(thrakkus).unwrap().power, 6);

    let mut g = pod(2);
    let sarkhan = ready(&mut g, 0, catalog::sarkhan_unbroken());
    g.battlefield_find_mut(sarkhan).unwrap().add_counters(CounterType::Loyalty, 4);
    for def in [catalog::wrathful_red_dragon(), catalog::grizzly_bears(), catalog::scaled_nurturer()] {
        g.add_card_to_library(0, def);
    }
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: sarkhan, ability_index: 2, target: None, x_value: None })
        .expect("-8");
    drain_stack(&mut g);
    assert_eq!(named(&g, "Wrathful Red Dragon") + named(&g, "Scaled Nurturer"), 2);
    assert_eq!(named(&g, "Grizzly Bears"), 0);
}

/// Insidious Fungus' draw mode also puts a land from hand onto the
/// battlefield tapped (it shipped without that half).
#[test]
fn insidious_fungus_draws_then_drops_a_land() {
    let mut g = pod(2);
    let fungus = ready(&mut g, 0, catalog::insidious_fungus());
    g.add_card_to_library(0, catalog::island());
    let forest = g.add_card_to_hand(0, catalog::forest());
    flood(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: fungus,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: Some(2),
    })
    .expect("draw mode");
    drain_stack(&mut g);
    let land = g.battlefield_find(forest).expect("the Forest entered");
    assert!(land.tapped);
    assert_eq!(g.players[0].hand.len(), 1, "the Island drawn");
}

fn attack_with(g: &mut GameState, attackers: &[CardId]) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

/// Wolverine, Best There Is doubles its own damage (CR 614.2) and grows at
/// the end step only after damaging another creature — one that died
/// counts.
#[test]
fn wolverine_doubles_and_grows() {
    let mut g = pod(2);
    let wolverine = ready(&mut g, 0, catalog::wolverine_best_there_is());
    let wall = ready(&mut g, 1, catalog::wall_of_wood());
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(wolverine).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    let ctx = EffectContext::for_ability(wolverine, 0, None);
    let evs = g
        .resolve_effect(&Effect::Fight { attacker: Selector::ExactObjects(vec![wolverine]), defender: Selector::ExactObjects(vec![wall]) }, &ctx)
        .expect("fight");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(wall).is_none(), "2 power, doubled, kills the 0/3 wall");
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(wolverine).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Mjölnir: equip worthy is a legendary non-Villain that's red or white;
/// the equipped creature's damage is doubled, combat or not.
#[test]
fn mjolnir_doubles_the_worthy() {
    let mut g = pod(2);
    let hammer = ready(&mut g, 0, catalog::mjolnir_hammer_of_thor());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let sarkhan = ready(&mut g, 0, catalog::thrakkus_the_butcher());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::Equip { equipment: hammer, target: bear }).is_err(), "not legendary");
    g.perform_action(GameAction::Equip { equipment: hammer, target: sarkhan }).expect("equip the worthy");
    let maw = ready(&mut g, 1, catalog::colossal_dreadmaw());
    let ctx = EffectContext::for_ability(sarkhan, 0, None);
    let evs = g
        .resolve_effect(&Effect::DealDamageFrom { source: Selector::This, to: Selector::ExactObjects(vec![maw]), amount: Value::Const(2) }, &ctx)
        .expect("ping");
    g.dispatch_triggers_for_events(&evs);
    assert_eq!(g.battlefield_find(maw).unwrap().damage, 4);
}

/// Wolverine, Fierce Fighter: new damage heals the old (CR 614.1a), so two
/// 3-damage hits leave 3 marked, not 6 — it survives what would kill a 3/5.
#[test]
fn fierce_fighter_heals_as_it_is_hit() {
    let mut g = pod(2);
    let w = ready(&mut g, 0, catalog::wolverine_fierce_fighter());
    for _ in 0..2 {
        let ctx = EffectContext::for_spell(1, None, 0, 0);
        let evs = g
            .resolve_effect(&Effect::DealDamage { to: Selector::ExactObjects(vec![w]), amount: Value::Const(3) }, &ctx)
            .expect("burn");
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(w).map(|c| c.damage), Some(3));
}

/// Wolverine, Claws Out doubles an attacking Mutant's power; Epic Fight's
/// two modes double P/T and fight.
#[test]
fn claws_out_and_epic_fight() {
    let mut g = pod(2);
    let claws = ready(&mut g, 0, catalog::wolverine_claws_out());
    attack_with(&mut g, &[claws]);
    assert_eq!(g.computed_permanent(claws).unwrap().power, 4);
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let victim = ready(&mut g, 1, catalog::grizzly_bears());
    flood(&mut g);
    let spell = g.add_card_to_hand(0, catalog::epic_fight());
    g.perform_action(GameAction::CastSpellSpree {
        card_id: spell,
        spree_modes: vec![0, 1],
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Permanent(bear), Target::Permanent(victim)],
        x_value: None,
    })
    .expect("both modes");
    drain_stack(&mut g);
    let b = g.computed_permanent(bear).unwrap();
    assert_eq!((b.power, b.toughness), (4, 4));
    assert!(g.battlefield_find(victim).is_none(), "fought by a 4/4");
}

// ── Target-deck residuals (Terra's and Eddie Brock's seats) ─────────────────

/// Rydia's Summon: {X}, {T} returns a Saga card of mana value exactly X with
/// a finality counter and haste; X must match.
#[test]
fn rydia_summons_a_saga_back() {
    let mut g = pod(2);
    let rydia = ready(&mut g, 0, catalog::rydia_summoner_of_mist());
    let saga = g.add_card_to_graveyard(0, catalog::summon_brynhildr());
    stock_libraries(&mut g, 3);
    flood(&mut g);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: rydia,
            ability_index: 0,
            target: Some(Target::Permanent(saga)),
            additional_targets: vec![],
            x_value: Some(3),
            mode: None,
        })
        .is_err(),
        "the Saga's mana value is 2"
    );
    g.perform_action(GameAction::ActivateAbility {
        card_id: rydia,
        ability_index: 0,
        target: Some(Target::Permanent(saga)),
        additional_targets: vec![],
        x_value: Some(2),
        mode: None,
    })
    .expect("X = 2");
    drain_stack(&mut g);
    let back = g.battlefield_find(saga).expect("returned");
    assert_eq!(back.counter_count(CounterType::Finality), 1);
    assert!(has_kw(&g, saga, Keyword::Haste));
}

/// Xu-Ifit's returned creature is a Skeleton with no abilities (CR 613.1d /
/// 613.1f): a flier comes back grounded.
#[test]
fn xu_ifit_returns_a_skeleton_without_abilities() {
    let mut g = pod(2);
    let xu = ready(&mut g, 0, catalog::xu_ifit_osteoharmonist());
    let bird = g.add_card_to_graveyard(0, catalog::serra_angel());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: xu,
        ability_index: 0,
        target: Some(Target::Permanent(bird)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bird).expect("returned");
    assert!(cp.subtypes().creature_types.contains(&crabomination::card::CreatureType::Skeleton));
    assert!(!cp.keywords().contains(&Keyword::Flying));
}

/// Garnet's lore-counter cash-in is a pick among the Sagas: each one chosen
/// gives up a lore counter and pays a +1/+1 counter.
#[test]
fn garnet_trades_lore_for_counters() {
    let mut g = pod(2);
    let garnet = ready(&mut g, 0, catalog::garnet_princess_of_alexandria());
    let saga = ready(&mut g, 0, catalog::summon_brynhildr());
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 1);
    attack_with(&mut g, &[garnet]);
    let pluses = g.battlefield_find(garnet).unwrap().counter_count(CounterType::PlusOnePlusOne);
    let lore = g.battlefield_find(saga).map_or(0, |c| c.counter_count(CounterType::Lore));
    assert_eq!(pluses + lore, 1, "a removed lore counter became a +1/+1 counter, or neither moved");
}

/// Glint-Horn Buccaneer and Ivora read each discarded card: Change of
/// Fortune's whole-hand discard pings every opponent per card, grows Ivora
/// per card, and draws for every card discarded this turn.
#[test]
fn discard_payoffs_count_each_card() {
    let mut g = pod(3);
    stock_libraries(&mut g, 10);
    ready(&mut g, 0, catalog::glint_horn_buccaneer());
    let ivora = ready(&mut g, 0, catalog::ivora_insatiable_heir());
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::island());
    }
    let spell = g.add_card_to_hand(0, catalog::change_of_fortune());
    flood(&mut g);
    cast(&mut g, spell);
    assert_eq!((g.players[1].life, g.players[2].life), (17, 17));
    assert_eq!(g.battlefield_find(ivora).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    assert_eq!(g.players[0].hand.len(), 3);
}

/// Mary Read and Anne Bonny: a discarded Island (or Pirate, or Vehicle)
/// makes a tapped Treasure; another card doesn't.
#[test]
fn mary_and_anne_treasure_on_islands() {
    let mut g = pod(2);
    stock_libraries(&mut g, 3);
    let pair = ready(&mut g, 0, catalog::mary_read_and_anne_bonny());
    let ctx = EffectContext::for_ability(pair, 0, None);
    let discard_one = |g: &mut GameState| {
        let evs = g
            .resolve_effect(&Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }, &ctx)
            .expect("discard");
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    };
    g.add_card_to_hand(0, catalog::lightning_bolt());
    discard_one(&mut g);
    assert_eq!(named(&g, "Treasure"), 0);
    g.add_card_to_hand(0, catalog::island());
    discard_one(&mut g);
    assert_eq!(named(&g, "Treasure"), 1);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure" && c.tapped));
}

/// Brass's Tunnel-Grinder: a bore counter only on an end step after you
/// descended; the third transforms it into Tecutlan, whose mana discovers
/// for a permanent spell (CR 106.6 rider, CR 701.57).
#[test]
fn tunnel_grinder_bores_into_tecutlan() {
    let mut g = pod(2);
    let grinder = ready(&mut g, 0, catalog::brasss_tunnel_grinder());
    let end_step = |g: &mut GameState| {
        g.step = TurnStep::End;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(g);
    };
    end_step(&mut g);
    assert_eq!(g.battlefield_find(grinder).unwrap().counter_count(CounterType::Bore), 0, "no descent");
    for _ in 0..3 {
        let bear = ready(&mut g, 0, catalog::grizzly_bears());
        kill(&mut g, bear);
        end_step(&mut g);
        g.players[0].descended_this_turn = false;
    }
    let tecutlan = g.battlefield_find(grinder).unwrap();
    assert_eq!(tecutlan.definition.name, "Tecutlan, the Searing Rift");
}

/// Clown Car: X dice, each odd result a Clown Robot and each even one a
/// counter — X results in all.
#[test]
fn clown_car_splits_its_dice() {
    let mut g = pod(2);
    let car = g.add_card_to_hand(0, catalog::clown_car());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: car, target: None, additional_targets: vec![], mode: None, x_value: Some(4) })
        .expect("cast X = 4");
    drain_stack(&mut g);
    let counters = g.battlefield_find(car).unwrap().counter_count(CounterType::PlusOnePlusOne) as usize;
    assert_eq!(counters + named(&g, "Clown Robot"), 4);
}

/// Storied: Óin's enduring story latches at three artifacts, legendaries
/// and/or Sagas, and stays when they leave (+1/+0 and haste).
#[test]
fn oin_keeps_his_enduring_story() {
    let mut g = pod(2);
    let oin = ready(&mut g, 0, catalog::oin_the_brave());
    assert_eq!(g.computed_permanent(oin).unwrap().power, 1, "Óin alone is one legendary");
    ready(&mut g, 0, catalog::sol_ring());
    assert!(!g.players[0].enduring_story);
    let ring = ready(&mut g, 0, catalog::sol_ring());
    assert!(g.players[0].enduring_story);
    kill(&mut g, ring);
    assert_eq!(g.computed_permanent(oin).unwrap().power, 2);
    assert!(has_kw(&g, oin, Keyword::Haste));
}

/// Flywheel Racer taps for mana only while crewed into a creature; Dwarven
/// Mauler makes an equip aimed at it {2} cheaper; Vault Robber cashes a
/// creature card from the graveyard into a Treasure.
#[test]
fn magdas_dwarves_and_vehicles() {
    let mut g = pod(2);
    let racer = ready(&mut g, 0, catalog::flywheel_racer());
    assert!(activate(&mut g, racer, 0).is_err(), "not a creature yet");
    let mauler = ready(&mut g, 0, catalog::dwarven_mauler());
    let sword = ready(&mut g, 0, catalog::bonesplitter());
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: sword, target: mauler }).expect("equip {1} for free");
    assert_eq!(g.battlefield_find(sword).unwrap().attached_to, Some(mauler));
    let robber = ready(&mut g, 0, catalog::vault_robber());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, robber, 0).expect("exile the Bear");
    assert_eq!(named(&g, "Treasure"), 1);
}

/// Berserk can't be cast once a combat damage step has begun (its printed
/// timing restriction); Rielle draws only on the turn's FIRST discard batch,
/// even one made before she entered.
#[test]
fn berserk_timing_and_rielles_first_discard() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g);
    g.step = TurnStep::CombatDamage;
    let berserk = g.add_card_to_hand(0, catalog::berserk());
    g.priority.player_with_priority = 0;
    assert!(g
        .perform_action(GameAction::CastSpell { card_id: berserk, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None })
        .is_err());
    g.step = TurnStep::DeclareBlockers;
    cast_at(&mut g, berserk, Some(Target::Permanent(bear)));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4);

    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let discard_one = |g: &mut GameState| {
        g.add_card_to_hand(0, catalog::island());
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        let evs = g
            .resolve_effect(&Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }, &ctx)
            .expect("discard");
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    };
    discard_one(&mut g);
    ready(&mut g, 0, catalog::rielle_the_everwise());
    discard_one(&mut g);
    assert_eq!(g.players[0].hand.len(), 0, "not the first discard this turn");
    advance_to_turn_of(&mut g, 1);
    advance_to_turn_of(&mut g, 0);
    let hand = g.players[0].hand.len();
    discard_one(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "a new turn's first discard draws one");
}

/// Summon: Brynhildr's chapter I card is playable only during a turn a lore
/// counter goes on the Saga: this turn, not the next until chapter II.
#[test]
fn brynhildrs_chain_card_wakes_with_each_lore_counter() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let saga = g.add_card_to_hand(0, catalog::summon_brynhildr());
    flood(&mut g);
    cast(&mut g, saga);
    let playable = |g: &GameState| g.exile.iter().any(|c| c.may_play_until.is_some_and(|p| p.player == 0));
    assert!(playable(&g), "chapter I's turn");
    advance_to_turn_of(&mut g, 1);
    assert!(!playable(&g), "parked on a turn with no lore counter");
    advance_to_turn_of(&mut g, 0);
    assert!(playable(&g), "chapter II's lore counter wakes it");
}

/// Secret Tunnel's second target must share a creature type with the first
/// (CR 601.2c): two Bears may tunnel, a Bear and a Wall may not.
#[test]
fn secret_tunnel_needs_a_shared_creature_type() {
    let mut g = pod(2);
    let tunnel = ready(&mut g, 0, catalog::secret_tunnel());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let wall = ready(&mut g, 0, catalog::wall_of_wood());
    let other_bear = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g);
    let go = |g: &mut GameState, second: CardId| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: tunnel,
            ability_index: 1,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![Target::Permanent(second)],
            x_value: None,
            mode: None,
        })
    };
    assert!(go(&mut g, wall).is_err(), "a Wall shares no type with a Bear");
    go(&mut g, other_bear).expect("two Bears");
    drain_stack(&mut g);
    assert!(has_kw(&g, bear, Keyword::Unblockable) && has_kw(&g, other_bear, Keyword::Unblockable));
}

/// Eladamri, Korvecdal: {G}, {T} and two more untapped creatures put a
/// creature card from hand (mode 0) or the library's top (mode 1) onto the
/// battlefield; a noncreature top card stays put.
#[test]
fn eladamri_drops_a_creature_from_hand_or_top() {
    let mut g = pod(2);
    let eladamri = ready(&mut g, 0, catalog::eladamri_korvecdal());
    for _ in 0..2 {
        ready(&mut g, 0, catalog::grizzly_bears());
    }
    let angel = g.add_card_to_hand(0, catalog::serra_angel());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: eladamri,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: Some(0),
    })
    .expect("{G}, {T}, tap two");
    drain_stack(&mut g);
    assert!(g.battlefield_find(angel).is_some(), "the Angel from hand");
    assert_eq!(g.battlefield.iter().filter(|c| c.tapped).count(), 3, "Eladamri and both Bears");
}

// ── Lorehold, the Historian's list ──────────────────────────────────────────

/// Goliath Daydreamer: an instant cast from hand is exiled with a dream
/// counter as it resolves; attacking recasts it free from exile, and that
/// cast (not from hand) goes to the graveyard.
#[test]
fn goliath_daydreamer_dreams_a_bolt_and_recasts_it() {
    let mut g = pod(2);
    let goliath = ready(&mut g, 0, catalog::goliath_daydreamer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g);
    let life = g.players[1].life;
    cast_at(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 3);
    let exiled = g.exile.iter().find(|c| c.id == bolt).expect("exiled, not in the graveyard");
    assert_eq!(exiled.counter_count(CounterType::Dream), 1);
    attack_with(&mut g, &[goliath]);
    assert_eq!(g.players[1].life, life - 6, "the attack recast the dreamed Bolt");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt), "a cast from exile isn't dreamed again");
}

/// Invoke Calamity: up to two instants/sorceries with total mana value 6 or
/// less, from graveyard and hand, cast free and exiled afterwards; a third
/// doesn't fit.
#[test]
fn invoke_calamity_casts_two_within_six() {
    let mut g = pod(2);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let axe = g.add_card_to_hand(0, catalog::lava_axe());
    let divination = g.add_card_to_hand(0, catalog::divination());
    let invoke = g.add_card_to_hand(0, catalog::invoke_calamity());
    flood(&mut g);
    let life = g.players[1].life;
    cast(&mut g, invoke);
    assert_eq!(g.players[1].life, life - 8, "Lava Axe (5) and Bolt (1) total six");
    assert!(g.players[0].hand.iter().any(|c| c.id == divination), "Divination (3) doesn't fit beside Lava Axe");
    for id in [bolt, axe, invoke] {
        assert!(g.exile.iter().any(|c| c.id == id), "exiled instead of the graveyard");
    }
}

/// Olórin's Searing Light: each opponent exiles their greatest-power
/// creature; with spell mastery each takes that creature's power.
#[test]
fn olorins_searing_light_exiles_the_biggest_and_burns() {
    let mut g = pod(3);
    ready(&mut g, 1, catalog::grizzly_bears());
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let ogre = ready(&mut g, 2, catalog::gray_ogre());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::shock());
    let light = g.add_card_to_hand(0, catalog::olorins_searing_light());
    flood(&mut g);
    let (l1, l2) = (g.players[1].life, g.players[2].life);
    cast(&mut g, light);
    assert!(g.exile.iter().any(|c| c.id == angel) && g.exile.iter().any(|c| c.id == ogre));
    assert_eq!(named(&g, "Grizzly Bears"), 1);
    assert_eq!((g.players[1].life, g.players[2].life), (l1 - 4, l2 - 2));
}

/// Soulfire Eruption: per target, the top card of the library is exiled and
/// deals its mana value; the exiled cards stay playable.
#[test]
fn soulfire_eruption_burns_by_mana_value() {
    let mut g = pod(2);
    let angel = g.add_card_to_library(0, catalog::serra_angel());
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    let ogre = ready(&mut g, 1, catalog::gray_ogre());
    let eruption = g.add_card_to_hand(0, catalog::soulfire_eruption());
    flood(&mut g);
    let life = g.players[1].life;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: eruption,
        target: Some(Target::Player(1)),
        additional_targets: vec![Target::Permanent(ogre)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 5, "Serra Angel's mana value");
    assert!(g.battlefield_find(ogre).is_none(), "Grizzly Bears' 2 kills the Ogre");
    for id in [angel, bears] {
        assert!(g.exile.iter().any(|c| c.id == id && c.may_play_until.is_some()));
    }
}

// ── High Perfect Morcant's list ─────────────────────────────────────────────

fn to_upkeep(g: &mut GameState) {
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(g);
}

/// Eclipsed Elf takes an Elf off the top four; Elvish Harbinger stacks an Elf
/// on top and taps for any color.
#[test]
fn eclipsed_elf_and_harbinger_find_elves() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let elves = g.add_card_to_library(0, catalog::llanowar_elves());
    let eclipsed = g.add_card_to_hand(0, catalog::eclipsed_elf());
    flood(&mut g);
    cast(&mut g, eclipsed);
    assert!(g.players[0].hand.iter().any(|c| c.id == elves), "an Elf among the top four");
    let deep = g.add_card_to_library(0, catalog::elvish_mystic());
    let harbinger = g.add_card_to_hand(0, catalog::elvish_harbinger());
    flood(&mut g);
    cast(&mut g, harbinger);
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(deep), "the Elf is on top");
    g.clear_sickness(harbinger);
    g.players[0].mana_pool.empty();
    activate(&mut g, harbinger, 0).expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

/// Crumbling Ashes destroys a creature with a -1/-1 counter at your upkeep;
/// Morcant's Eyes surveils then and cashes Elf cards in for 2/2 Elves.
#[test]
fn crumbling_ashes_and_morcants_eyes_upkeep() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    ready(&mut g, 0, catalog::crumbling_ashes());
    let eyes = ready(&mut g, 0, catalog::morcants_eyes());
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().add_counters(CounterType::MinusOneMinusOne, 1);
    let ogre = ready(&mut g, 1, catalog::gray_ogre());
    to_upkeep(&mut g);
    assert!(g.battlefield_find(bears).is_none(), "the countered creature is destroyed");
    assert!(g.battlefield_find(ogre).is_some());
    g.step = TurnStep::PreCombatMain;
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::llanowar_elves());
    }
    flood(&mut g);
    activate(&mut g, eyes, 0).expect("sacrifice for Elves");
    assert!(g.battlefield_find(eyes).is_none());
    // Morcant's Eyes is a Kindred Elf card, sacrificed as a cost, so it counts
    // itself: three Llanowar Elves and the Eyes.
    assert_eq!(named(&g, "Elf"), 4, "one 2/2 per Elf card in the graveyard");
}

/// Tyvar Kell: Elves tap for {B}; +1 grows, untaps and gives an Elf deathtouch.
#[test]
fn tyvar_kell_feeds_and_arms_elves() {
    let mut g = pod(2);
    let tyvar = ready(&mut g, 0, catalog::tyvar_kell());
    let elf = ready(&mut g, 0, catalog::llanowar_elves());
    g.players[0].mana_pool.empty();
    let printed = catalog::llanowar_elves().activated_abilities.len();
    activate(&mut g, elf, printed).expect("the granted {T}: Add {B}");
    assert_eq!(g.players[0].mana_pool.total(), 1);
    assert!(g.battlefield_find(elf).unwrap().tapped);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: tyvar,
        ability_index: 0,
        target: Some(Target::Permanent(elf)),
        x_value: None,
    })
    .expect("+1");
    drain_stack(&mut g);
    let c = g.battlefield_find(elf).unwrap();
    assert!(!c.tapped && c.counter_count(CounterType::PlusOnePlusOne) == 1);
    assert!(has_kw(&g, elf, Keyword::Deathtouch));
}

/// Trystan's Command: destroy a creature and pump-and-untap a player's
/// creatures (modes 2 and 3).
#[test]
fn trystans_command_destroys_and_pumps() {
    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(mine).unwrap().tapped = true;
    let theirs = ready(&mut g, 1, catalog::serra_angel());
    let command = g.add_card_to_hand(0, catalog::trystans_command());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellSpree {
        spree_modes: vec![2, 3],
        card_id: command,
        target: Some(Target::Permanent(theirs)),
        additional_targets: vec![Target::Player(0)],
        x_value: None,
    })
    .expect("cast two modes");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none());
    let bears = g.battlefield_find(mine).unwrap();
    assert!(!bears.tapped);
    assert_eq!(g.computed_permanent(mine).unwrap().power, 5);
}

/// Chakra Meditation: the ETB returns an instant; a cast draws and then
/// discards unless three Lessons sit in the graveyard.
#[test]
fn chakra_meditation_loots_until_three_lessons() {
    let mut g = pod(2);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let med = g.add_card_to_hand(0, catalog::chakra_meditation());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell { card_id: med, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "the ETB returned Lightning Bolt");
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1, "cast one, drew one, discarded one");
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::overwhelming_victory());
    }
    let shock = g.add_card_to_hand(0, catalog::shock());
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell { card_id: shock, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None })
        .expect("shock");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "three Lessons: the draw is kept");
}

/// Elemental Teachings: four lands with different names found; two go to the
/// graveyard, two enter tapped.
#[test]
fn elemental_teachings_splits_four_lands() {
    let mut g = pod(2);
    g.players[0].library.clear();
    for def in [catalog::forest(), catalog::forest(), catalog::island(), catalog::mountain(), catalog::plains()] {
        g.add_card_to_library(0, def);
    }
    let et = g.add_card_to_hand(0, catalog::elemental_teachings());
    flood(&mut g);
    let lands = g.battlefield.len();
    g.perform_action(GameAction::CastSpell { card_id: et, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    let entered: Vec<_> = g.battlefield.iter().skip(lands).collect();
    assert_eq!(entered.len(), 2);
    assert!(entered.iter().all(|c| c.tapped));
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.is_land()).count(), 2);
    assert_eq!(g.players[0].library.len(), 1, "one Forest is left: names differ");
}

/// Overwhelming Victory: 5 damage to a 2/2 is 3 excess — each creature you
/// control gets +3/+0 and trample.
#[test]
fn overwhelming_victory_pumps_by_the_excess() {
    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let ov = g.add_card_to_hand(0, catalog::overwhelming_victory());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell { card_id: ov, target: Some(Target::Permanent(theirs)), additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none());
    assert_eq!(g.computed_permanent(mine).unwrap().power, 5);
    assert!(has_kw(&g, mine, Keyword::Trample));
}

/// Secret of Bloodbending: unpaid, the caster controls the opponent's next
/// combat phase (CR 723.2); with waterbend {10}, their whole next turn
/// (CR 723.1). Either way the card is exiled.
#[test]
fn secret_of_bloodbending_controls_a_combat_or_a_turn() {
    for paid in [false, true] {
        let mut g = pod(3);
        let secret = g.add_card_to_hand(0, catalog::secret_of_bloodbending());
        flood(&mut g);
        let cast = if paid {
            GameAction::CastSpellWaterbend {
                card_id: secret, target: Some(Target::Player(2)), additional_targets: vec![], mode: None, x_value: None,
                helpers: vec![],
            }
        } else {
            GameAction::CastSpell { card_id: secret, target: Some(Target::Player(2)), additional_targets: vec![], mode: None, x_value: None }
        };
        g.perform_action(cast).expect("cast");
        drain_stack(&mut g);
        if paid {
            assert_eq!(g.pending_player_control, vec![(2, 0)]);
            assert!(g.pending_combat_control.is_empty());
        } else {
            assert_eq!(g.pending_combat_control, vec![(2, 0)]);
            assert!(g.pending_player_control.is_empty());
        }
        assert!(g.exile.iter().any(|c| c.id == secret), "exiled on resolution");
    }
}

fn counters(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

/// Aragorn and Arwen, Wed: entering puts a counter on each OTHER creature of
/// yours and gains a life per other creature.
#[test]
fn aragorn_and_arwen_counters_and_life_for_the_others() {
    let mut g = pod(2);
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::llanowar_elves());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let wed = g.add_card_to_hand(0, catalog::aragorn_and_arwen_wed());
    flood(&mut g);
    let life = g.players[0].life;
    cast(&mut g, wed);
    let wed = g.battlefield.iter().find(|c| c.definition.name == "Aragorn and Arwen, Wed").unwrap().id;
    assert_eq!((counters(&g, a), counters(&g, b), counters(&g, wed), counters(&g, theirs)), (1, 1, 0, 0));
    assert_eq!(g.players[0].life, life + 2);
}

/// Arwen Undómiel + Elrond, Master of Healing: one scry 2 is one counter from
/// Arwen and two from Elrond (X = the cards looked at); a one-card library
/// makes Elrond's X 1 (2023-06-16 ruling).
#[test]
fn arwen_and_elrond_count_the_cards_scried() {
    for (library, total) in [(5, 3), (1, 2)] {
        let mut g = pod(2);
        for _ in 0..library {
            g.add_card_to_library(0, catalog::island());
        }
        let arwen = ready(&mut g, 0, catalog::arwen_undomiel());
        let elrond = ready(&mut g, 0, catalog::elrond_master_of_healing());
        flood(&mut g);
        activate(&mut g, arwen, 0).expect("scry 2");
        assert_eq!(counters(&g, arwen) + counters(&g, elrond), total, "library {library}");
    }
}

/// Galadriel of Lothlórien: a scry reveals the top card and a land enters tapped.
#[test]
fn galadriel_of_lothlorien_deploys_a_scried_land_tapped() {
    let mut g = pod(2);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let arwen = ready(&mut g, 0, catalog::arwen_undomiel());
    ready(&mut g, 0, catalog::galadriel_of_lothlorien());
    flood(&mut g);
    activate(&mut g, arwen, 0).expect("scry 2");
    let forests: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Forest").collect();
    assert_eq!(forests.len(), 1);
    assert!(forests[0].tapped);
}

/// Arboreal Alliance: X = 3 is a 3/3 Treefolk.
#[test]
fn arboreal_alliance_makes_an_x_x_treefolk() {
    let mut g = pod(2);
    let aa = g.add_card_to_hand(0, catalog::arboreal_alliance());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell { card_id: aa, target: None, additional_targets: vec![], mode: None, x_value: Some(3) })
        .expect("cast");
    drain_stack(&mut g);
    let tf = g.battlefield.iter().find(|c| c.definition.name == "Treefolk").expect("token").id;
    let cp = g.computed_permanent(tf).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
}

/// Galadriel's Dismissal: unkicked, one creature phases out; kicked, every
/// creature the target player controls does.
#[test]
fn galadriels_dismissal_one_creature_or_a_whole_side() {
    let mut g = pod(2);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::serra_angel());
    let gd = g.add_card_to_hand(0, catalog::galadriels_dismissal());
    flood(&mut g);
    cast_at(&mut g, gd, Some(Target::Permanent(a)));
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_some());
    let gd = g.add_card_to_hand(0, catalog::galadriels_dismissal());
    g.perform_action(GameAction::CastSpellKicked { card_id: gd, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None })
        .expect("kicked");
    drain_stack(&mut g);
    assert!(g.battlefield_find(b).is_none());
    assert_eq!(g.phased_out.len(), 2);
}

fn equip_to(g: &mut GameState, eq: CardId, host: CardId) {
    g.battlefield_find_mut(eq).unwrap().attached_to = Some(host);
}

/// The Ring cycle's activations: Xathrid regenerates its bearer, Evos Isle
/// gives it hexproof; Kalonia's trample is static.
#[test]
fn rings_of_kalonia_xathrid_and_evos_isle() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let kalonia = ready(&mut g, 0, catalog::ring_of_kalonia());
    equip_to(&mut g, kalonia, bear);
    assert!(has_kw(&g, bear, Keyword::Trample));
    let xathrid = ready(&mut g, 0, catalog::ring_of_xathrid());
    equip_to(&mut g, xathrid, bear);
    let evos = ready(&mut g, 0, catalog::ring_of_evos_isle());
    equip_to(&mut g, evos, bear);
    flood(&mut g);
    activate(&mut g, xathrid, 0).expect("regenerate");
    assert_eq!(g.battlefield_find(bear).unwrap().regeneration_shields, 1);
    activate(&mut g, evos, 0).expect("hexproof");
    assert!(has_kw(&g, bear, Keyword::Hexproof));
}

/// Duskfang Mentor: the lifelink counter skips Humans (itself), and its
/// activation grows every lifelinker.
#[test]
fn duskfang_mentor_marks_a_non_human_then_grows_lifelinkers() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let mentor = g.add_card_to_hand(0, catalog::duskfang_mentor());
    flood(&mut g);
    cast(&mut g, mentor);
    assert!(has_kw(&g, bear, Keyword::Lifelink));
    let mentor = g.battlefield.iter().find(|c| c.definition.name == "Duskfang Mentor").unwrap().id;
    assert!(!has_kw(&g, mentor, Keyword::Lifelink));
    g.clear_sickness(mentor);
    activate(&mut g, mentor, 0).expect("grow");
    assert_eq!((counters(&g, bear), counters(&g, mentor)), (1, 0));
}

/// Thundering Mightmare: paired, an opponent's spell grows both of the pair.
#[test]
fn thundering_mightmare_grows_the_pair_on_an_opponents_spell() {
    let mut g = pod(2);
    let horse = ready(&mut g, 0, catalog::thundering_mightmare());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(horse).unwrap().soulbond_partner = Some(bear);
    g.battlefield_find_mut(bear).unwrap().soulbond_partner = Some(horse);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    drain_stack(&mut g);
    assert_eq!((counters(&g, horse), counters(&g, bear)), (1, 1));
}

/// Call of the Death-Dweller: two cards totalling MV 3 return; one gets a
/// deathtouch counter and one a menace counter.
#[test]
fn call_of_the_death_dweller_returns_two_and_marks_them() {
    let mut g = pod(2);
    let elves = g.add_card_to_graveyard(0, catalog::llanowar_elves());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let call = g.add_card_to_hand(0, catalog::call_of_the_death_dweller());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell { card_id: call, target: Some(Target::Permanent(elves)), additional_targets: vec![Target::Permanent(bears)], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(elves).is_some() && g.battlefield_find(bears).is_some());
    assert!(has_kw(&g, elves, Keyword::Deathtouch) || has_kw(&g, bears, Keyword::Deathtouch));
    assert!(has_kw(&g, elves, Keyword::Menace) || has_kw(&g, bears, Keyword::Menace));
}

/// Increasing Savagery: five counters from hand, ten flashed back.
#[test]
fn increasing_savagery_doubles_from_the_graveyard() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let sav = g.add_card_to_hand(0, catalog::increasing_savagery());
    flood(&mut g);
    cast_at(&mut g, sav, Some(Target::Permanent(bear)));
    assert_eq!(counters(&g, bear), 5);
    g.perform_action(GameAction::CastFlashback { card_id: sav, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None })
        .expect("flashback");
    drain_stack(&mut g);
    assert_eq!(counters(&g, bear), 15);
}

/// Visions of Dominance: one counter, then double — 2 becomes 6.
#[test]
fn visions_of_dominance_adds_then_doubles() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let vis = g.add_card_to_hand(0, catalog::visions_of_dominance());
    flood(&mut g);
    cast_at(&mut g, vis, Some(Target::Permanent(bear)));
    assert_eq!(counters(&g, bear), 6);
}

/// CR 704.5d / 117.3c — the auto-tapper never leaves a sacrificed Spawn in a
/// graveyard: a nested payment for Cascading Cataracts' {5} that sacrificed
/// Spawns and then failed rolled back only the pool and the taps (an audit
/// pod, seed 38200260); a Spawn that does pay is swept once the caster gets
/// priority.
#[test]
fn cr_704_5d_a_spawn_sacrificed_for_a_cast_leaves_no_token() {
    for signet in [false, true] {
        let mut g = pod(2);
        ready(&mut g, 0, catalog::ulalek_fused_atrocity());
        // The green comes off a restricted source, which the auto-tapper
        // reaches only after Cataracts' {5} "any colour" attempt fails.
        let court = ready(&mut g, 0, catalog::secluded_courtyard());
        g.battlefield_find_mut(court).unwrap().chosen_creature_type = Some(crabomination::card::CreatureType::Eldrazi);
        ready(&mut g, 0, catalog::cascading_cataracts());
        if signet {
            ready(&mut g, 0, catalog::golgari_signet());
        }
        for _ in 0..2 {
            g.add_token_to_battlefield(0, &crabomination_base::tokens::eldrazi_spawn_token());
        }
        let vile = g.add_card_to_hand(0, catalog::vile_redeemer());
        let _ = g.perform_action(GameAction::CastSpell { card_id: vile, target: None, additional_targets: vec![], mode: None, x_value: None });
        assert!(!g.players[0].graveyard.iter().any(|c| c.is_token), "signet {signet}");
    }
}

/// Step `g` forward until `step` on `seat`'s turn.
fn run_to(g: &mut GameState, seat: usize, step: TurnStep) {
    for _ in 0..200 {
        if g.active_player_idx == seat && g.step == step {
            return;
        }
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    panic!("never reached {step:?} on seat {seat}'s turn");
}

/// Rivendell: tapped without a legendary creature, untapped with one; the
/// scry needs one too.
#[test]
fn rivendell_wants_a_legend() {
    for legend in [false, true] {
        let mut g = pod(2);
        if legend {
            ready(&mut g, 0, catalog::pippin_guard_of_the_citadel());
        }
        let riv = g.add_card_to_hand(0, catalog::rivendell());
        g.perform_action(GameAction::PlayLand(riv)).expect("land drop");
        assert_eq!(g.battlefield_find(riv).unwrap().tapped, !legend);
        g.battlefield_find_mut(riv).unwrap().tapped = false;
        stock_libraries(&mut g, 3);
        flood(&mut g);
        assert_eq!(activate(&mut g, riv, 1).is_ok(), legend);
    }
}

/// Aragorn, Hornburg Hero: an attacker has first strike and renown 1; once
/// renowned, its next combat damage to a player doubles its counters.
#[test]
fn aragorn_hornburg_hero_renown_then_doubling() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    ready(&mut g, 0, catalog::aragorn_hornburg_hero());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    attack_with(&mut g, &[bear]);
    assert!(has_kw(&g, bear, Keyword::FirstStrike));
    run_to(&mut g, 0, TurnStep::PostCombatMain);
    assert_eq!(counters(&g, bear), 1, "renown 1");
    assert!(!has_kw(&g, bear, Keyword::FirstStrike), "only while attacking");
    run_to(&mut g, 0, TurnStep::PreCombatMain);
    run_to(&mut g, 0, TurnStep::BeginCombat);
    g.battlefield_find_mut(bear).unwrap().tapped = false;
    attack_with(&mut g, &[bear]);
    run_to(&mut g, 0, TurnStep::PostCombatMain);
    assert_eq!(counters(&g, bear), 2, "renowned: doubled, not renowned again");
}

/// Faramir: the chosen opponent's end step draws a card when they left you
/// alone, and makes three Soldiers when they attacked you.
#[test]
fn faramir_rewards_peace_or_punishes_attacks() {
    for attacked in [false, true] {
        let mut g = pod(2);
        stock_libraries(&mut g, 10);
        ready(&mut g, 0, catalog::faramir_prince_of_ithilien());
        run_to(&mut g, 1, TurnStep::PostCombatMain);
        if attacked {
            g.players[1].attacked_players_this_turn.push(0);
        }
        let hand = g.players[0].hand.len();
        run_to(&mut g, 1, TurnStep::End);
        let soldiers = named(&g, "Human Soldier");
        if attacked {
            assert_eq!((soldiers, g.players[0].hand.len()), (3, hand));
        } else {
            assert_eq!((soldiers, g.players[0].hand.len()), (0, hand + 1));
        }
    }
}

/// Katilda: a Human taps for a mana of its own colour.
#[test]
fn katilda_humans_tap_for_their_colors() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::katilda_dawnhart_prime());
    let lions = ready(&mut g, 0, catalog::elite_vanguard());
    activate(&mut g, lions, 0).expect("granted mana ability");
    assert_eq!(g.players[0].mana_pool.amount(Color::White), 1);
}

/// Pippin: a creature targeted by an opponent's Lightning Bolt gains
/// protection from instants, and the Bolt fizzles.
#[test]
fn pippin_blanks_the_threat_by_card_type() {
    let mut g = pod(2);
    let pippin = ready(&mut g, 0, catalog::pippin_guard_of_the_citadel());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: pippin, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: vec![], x_value: None, mode: None })
        .expect("Pippin");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "the Bolt lost its target");
}

/// Éowyn, Fearless Knight exiles a bigger opposing creature and shields your
/// legends from its colours.
#[test]
fn eowyn_fearless_knight_exiles_the_bigger_threat() {
    let mut g = pod(2);
    let pippin = ready(&mut g, 0, catalog::pippin_guard_of_the_citadel());
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let eowyn = g.add_card_to_hand(0, catalog::eowyn_fearless_knight());
    flood(&mut g);
    cast(&mut g, eowyn);
    assert!(g.battlefield_find(angel).is_none(), "4 power beats Éowyn's 3");
    assert!(has_kw(&g, pippin, Keyword::Protection(Color::White)));
}

/// Friendly Rivalry: a 2-power creature and a 4-power legend together kill a
/// 4/4 neither kills alone.
#[test]
fn friendly_rivalry_two_sources_one_victim() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let hero = ready(&mut g, 0, catalog::aragorn_hornburg_hero());
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let fr = g.add_card_to_hand(0, catalog::friendly_rivalry());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: fr,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Permanent(angel), Target::Permanent(hero)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(angel).is_none());
}

fn destroy(g: &mut GameState, id: CardId) {
    let ctx = EffectContext::for_spell(1, None, 0, 0);
    let evs = g.resolve_effect(&Effect::Destroy { what: Selector::ExactObjects(vec![id]) }, &ctx).expect("destroy");
    g.dispatch_triggers_for_events(&evs);
    let swept = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&swept);
    drain_stack(g);
}

/// Prized Statue and Nimblewright Schematic: one token entering, another
/// dying.
#[test]
fn statue_and_schematic_pay_twice() {
    for (def, token) in [(catalog::prized_statue(), "Treasure"), (catalog::nimblewright_schematic(), "Construct")] {
        let mut g = pod(2);
        let id = g.add_card_to_hand(0, def);
        flood(&mut g);
        cast(&mut g, id);
        assert_eq!(named(&g, token), 1, "{token}");
        destroy(&mut g, id);
        assert_eq!(named(&g, token), 2, "{token}");
    }
}

/// Implement of Improvement: sacrificed, 2 life and a card.
#[test]
fn implement_of_improvement_gains_and_draws() {
    let mut g = pod(2);
    stock_libraries(&mut g, 3);
    let imp = ready(&mut g, 0, catalog::implement_of_improvement());
    flood(&mut g);
    let (life, hand) = (g.players[0].life, g.players[0].hand.len());
    activate(&mut g, imp, 0).expect("sacrifice");
    assert_eq!((g.players[0].life, g.players[0].hand.len()), (life + 2, hand + 1));
}

/// Caduceus: lifelink always; at 30+ life +5/+5, indestructible and damage
/// prevented.
#[test]
fn caduceus_wakes_at_thirty_life() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let staff = ready(&mut g, 0, catalog::caduceus_staff_of_hermes());
    equip_to(&mut g, staff, bear);
    g.players[0].life = 29;
    assert!(has_kw(&g, bear, Keyword::Lifelink));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2);
    g.players[0].life = 30;
    assert_eq!(g.computed_permanent(bear).unwrap().power, 7);
    assert!(has_kw(&g, bear, Keyword::Indestructible));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0, "prevented");
}

/// Transmutation Font: {T} mints a token; three artifact tokens and {3}
/// fetch an artifact card onto the battlefield.
#[test]
fn transmutation_font_mints_then_fetches() {
    let mut g = pod(2);
    let font = ready(&mut g, 0, catalog::transmutation_font());
    g.add_card_to_library(0, catalog::sol_ring());
    flood(&mut g);
    activate(&mut g, font, 0).expect("mint");
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 1);
    for t in [crabomination_base::tokens::clue_token(), crabomination_base::tokens::food_token()] {
        g.add_token_to_battlefield(0, &t);
    }
    g.battlefield_find_mut(font).unwrap().tapped = false;
    activate(&mut g, font, 1).expect("fetch");
    assert_eq!(named(&g, "Sol Ring"), 1);
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 0);
    // "… with different names": three Treasures can't pay it.
    for _ in 0..3 {
        g.add_token_to_battlefield(0, &crabomination_base::tokens::treasure_token());
    }
    g.battlefield_find_mut(font).unwrap().tapped = false;
    assert!(activate(&mut g, font, 1).is_err());
}

/// Apothecary White: two players attacked, two Foods; then {W} and two
/// tapped Foods make two Humans.
#[test]
fn apothecary_white_feeds_per_defender() {
    let mut g = pod(3);
    let ap = ready(&mut g, 0, catalog::apothecary_white());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: ap, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(2) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(named(&g, "Food"), 2);
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    flood(&mut g);
    g.perform_action(GameAction::ActivateAbility { card_id: ap, ability_index: 0, target: None, additional_targets: vec![], x_value: Some(2), mode: None })
        .expect("tap two Foods");
    drain_stack(&mut g);
    assert_eq!(named(&g, "Human"), 2);
}

/// Ratchet, Field Medic: gaining 3 life converts it and returns a 3-mana
/// artifact from the graveyard tapped.
#[test]
fn ratchet_converts_and_rescues_on_life_gain() {
    let mut g = pod(2);
    let ratchet = ready(&mut g, 0, catalog::ratchet_field_medic());
    let font = g.add_card_to_graveyard(0, catalog::prized_statue());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(&Effect::GainLife { who: Selector::You, amount: Value::Const(3) }, &ctx).expect("gain");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(ratchet).unwrap().transformed, "converted");
    assert!(g.battlefield_find(font).is_some_and(|c| c.tapped), "the Statue returned tapped");
}

/// Spirit Loop: the enchanted creature's damage is life; the Loop comes back
/// to hand when it goes to the graveyard.
#[test]
fn spirit_loop_links_and_returns() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let lp = g.add_card_to_hand(0, catalog::spirit_loop());
    flood(&mut g);
    cast_at(&mut g, lp, Some(Target::Permanent(bear)));
    let life = g.players[0].life;
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g
        .resolve_effect(&Effect::DealDamageFrom { source: Selector::ExactObjects(vec![bear]), to: Selector::Player(crabomination::effect::PlayerRef::Seat(1)), amount: Value::Const(2) }, &ctx)
        .expect("damage");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2);
    destroy(&mut g, bear);
    assert!(g.players[0].hand.iter().any(|c| c.id == lp), "back to hand");
}

fn yes(g: &mut GameState, n: usize) {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    g.decider = Box::new(ScriptedDecider::new(std::iter::repeat_n(DecisionAnswer::Bool(true), n)));
}

/// Arc Reactor enters tapped and taps for three.
#[test]
fn arc_reactor_enters_tapped_for_three() {
    let mut g = pod(2);
    let arc = g.add_card_to_hand(0, catalog::arc_reactor());
    flood(&mut g);
    cast(&mut g, arc);
    let arc = g.battlefield.iter().find(|c| c.definition.name == "Arc Reactor").unwrap().id;
    assert!(g.battlefield_find(arc).unwrap().tapped);
    g.battlefield_find_mut(arc).unwrap().tapped = false;
    g.players[0].mana_pool = Default::default();
    activate(&mut g, arc, 0).expect("tap");
    assert_eq!(g.players[0].mana_pool.total(), 3);
}

/// Esoteric Duplicator: a sacrificed Implement of Improvement, {2} paid,
/// comes back as a token copy at the next end step.
#[test]
fn esoteric_duplicator_copies_a_sacrificed_artifact() {
    let mut g = pod(2);
    stock_libraries(&mut g, 3);
    ready(&mut g, 0, catalog::esoteric_duplicator());
    let imp = ready(&mut g, 0, catalog::implement_of_improvement());
    flood(&mut g);
    yes(&mut g, 1);
    activate(&mut g, imp, 0).expect("sacrifice");
    assert_eq!(named(&g, "Implement of Improvement"), 0);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let copy = g.battlefield.iter().find(|c| c.definition.name == "Implement of Improvement").expect("the copy");
    assert!(copy.is_token);
}

/// Ugin's Nexus: destroyed, it is exiled instead and its controller banks an
/// extra turn (CR 614.6).
#[test]
fn ugins_nexus_dies_into_an_extra_turn() {
    let mut g = pod(2);
    let nexus = ready(&mut g, 0, catalog::ugins_nexus());
    destroy(&mut g, nexus);
    assert!(g.exile.iter().any(|c| c.id == nexus));
    assert_eq!(g.players[0].extra_turns, 1);
}

/// Iron Man, Bleeding Edge copies the first artifact spell each turn; Shuri
/// makes it a {1} cheaper.
#[test]
fn iron_man_bleeding_edge_and_shuri_double_up_artifacts() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::iron_man_bleeding_edge());
    ready(&mut g, 0, catalog::shuri_wakandan_inventor());
    yes(&mut g, 4);
    let statue = g.add_card_to_hand(0, catalog::prized_statue());
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, statue);
    assert_eq!(named(&g, "Prized Statue"), 2, "cast for {{1}} and copied");
    let second = g.add_card_to_hand(0, catalog::prized_statue());
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, second);
    assert_eq!(named(&g, "Prized Statue"), 3, "once each turn");
}

/// Tony Stark digs an artifact, then becomes The Invincible Iron Man, which
/// puts an Equipment from hand onto the battlefield attached to itself.
#[test]
fn tony_stark_digs_then_suits_up() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_library(0, catalog::ring_of_kalonia());
    let tony = ready(&mut g, 0, catalog::tony_stark());
    flood(&mut g);
    activate(&mut g, tony, 0).expect("dig");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Ring of Kalonia"));
    activate(&mut g, tony, 1).expect("transform");
    assert!(g.battlefield_find(tony).unwrap().transformed);
    let ring = g.players[0].hand.iter().find(|c| c.definition.name == "Ring of Kalonia").unwrap().id;
    {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Cards(vec![ring])]));
    }
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ring).expect("put onto the battlefield").attached_to, Some(tony));
}

/// Storm the Vault transforms at an end step with five artifacts, and Vault
/// of Catlacan taps for a blue per artifact.
#[test]
fn storm_the_vault_becomes_the_vault() {
    let mut g = pod(2);
    let storm = ready(&mut g, 0, catalog::storm_the_vault());
    for _ in 0..5 {
        g.add_token_to_battlefield(0, &crabomination_base::tokens::treasure_token());
    }
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(storm).unwrap().transformed);
    activate(&mut g, storm, 1).expect("tap for blue");
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 5);
}

/// Demonic Taskmaster eats another creature each upkeep; Lord of the Pit
/// with nothing to eat bites its controller for 7.
#[test]
fn upkeep_demons_feed_or_bite() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::demonic_taskmaster());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());

    let mut g = pod(2);
    ready(&mut g, 0, catalog::lord_of_the_pit());
    let life = g.players[0].life;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 7);
}

/// Evil Eye of Urborg: your other creatures can't attack, and its blocker is
/// destroyed.
#[test]
fn evil_eye_of_urborg_locks_and_kills_its_blocker() {
    let mut g = pod(2);
    let eye = ready(&mut g, 0, catalog::evil_eye_of_urborg());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    assert!(has_kw(&g, bear, Keyword::CantAttack));
    assert!(!has_kw(&g, eye, Keyword::CantAttack));
    let wall = ready(&mut g, 1, catalog::wall_of_wood());
    attack_with(&mut g, &[eye]);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, eye)])).expect("block");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wall).is_none());
}

/// Plague Reaver: its end step clears your other creatures; sacrificed for
/// two discards, it returns under the target opponent's control at their next
/// upkeep.
#[test]
fn plague_reaver_sweeps_then_moves_house() {
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    let reaver = ready(&mut g, 0, catalog::plague_reaver());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    run_to(&mut g, 0, TurnStep::End);
    assert!(g.battlefield_find(bear).is_none() && g.battlefield_find(reaver).is_some());
    let mut g = pod(2);
    stock_libraries(&mut g, 10);
    let reaver = ready(&mut g, 0, catalog::plague_reaver());
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    g.perform_action(GameAction::ActivateAbility { card_id: reaver, ability_index: 0, target: Some(Target::Player(1)), additional_targets: vec![], x_value: None, mode: None })
        .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(reaver).is_none());
    run_to(&mut g, 1, TurnStep::Draw);
    assert_eq!(g.battlefield_find(reaver).map(|c| c.controller), Some(1));
}

/// Exchange of Words swaps two creatures' text boxes.
#[test]
fn exchange_of_words_swaps_text() {
    let mut g = pod(2);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let ex = g.add_card_to_hand(0, catalog::exchange_of_words());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell { card_id: ex, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert!(has_kw(&g, bear, Keyword::Flying) ^ has_kw(&g, angel, Keyword::Flying));
}

/// Wrong Turn and Fateful Handoff give a creature away; Modify Memory swaps
/// two opponents' creatures and draws three.
#[test]
fn donations_move_control() {
    let mut g = pod(3);
    stock_libraries(&mut g, 10);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let wt = g.add_card_to_hand(0, catalog::wrong_turn());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell { card_id: wt, target: Some(Target::Player(2)), additional_targets: vec![Target::Permanent(bear)], mode: None, x_value: None })
        .expect("wrong turn");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 2);

    let lions = ready(&mut g, 0, catalog::serra_angel());
    let fh = g.add_card_to_hand(0, catalog::fateful_handoff());
    let hand = g.players[0].hand.len();
    cast_at(&mut g, fh, Some(Target::Permanent(lions)));
    assert_ne!(g.battlefield_find(lions).unwrap().controller, 0);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 5, "Serra Angel's mana value");

    let a = ready(&mut g, 1, catalog::llanowar_elves());
    let mm = g.add_card_to_hand(0, catalog::modify_memory());
    let hand = g.players[0].hand.len();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: mm, target: Some(Target::Permanent(a)), additional_targets: vec![Target::Permanent(bear)], mode: None, x_value: None })
        .expect("modify memory");
    drain_stack(&mut g);
    assert_eq!((g.battlefield_find(a).unwrap().controller, g.battlefield_find(bear).unwrap().controller), (2, 1));
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3);
}

/// CR 601.2c — Modify Memory's two creatures must have different controllers:
/// two of one player's creatures are not a legal pair.
#[test]
fn modify_memory_wants_different_controllers() {
    let mut g = pod(3);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::llanowar_elves());
    let mm = g.add_card_to_hand(0, catalog::modify_memory());
    flood(&mut g);
    assert!(g
        .perform_action(GameAction::CastSpell { card_id: mm, target: Some(Target::Permanent(a)), additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None })
        .is_err());
}

/// Flesh Reaver hits back only off damage to a creature or an opponent: the
/// damage it deals its own controller doesn't retrigger it (with Spirit Loop
/// on it that was a mandatory loop, pod seed 38801 game 8).
#[test]
fn flesh_reaver_does_not_retrigger_off_its_controller() {
    let mut g = pod(2);
    let reaver = ready(&mut g, 0, catalog::flesh_reaver());
    let lp = g.add_card_to_hand(0, catalog::spirit_loop());
    flood(&mut g);
    cast_at(&mut g, lp, Some(Target::Permanent(reaver)));
    let (me, them) = (g.players[0].life, g.players[1].life);
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g
        .resolve_effect(&Effect::DealDamageFrom { source: Selector::ExactObjects(vec![reaver]), to: Selector::Player(crabomination::effect::PlayerRef::Seat(1)), amount: Value::Const(4) }, &ctx)
        .expect("damage");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.stack.is_empty());
    assert_eq!(g.players[1].life, them - 4);
    // 4 back to its controller, and Spirit Loop's 4 + 4 life.
    assert_eq!(g.players[0].life, me - 4 + 8);
}

/// Braids, Conjurer Adept: on another player's upkeep, THAT player may put a
/// creature from their hand onto the battlefield.
#[test]
fn braids_lets_the_active_player_deploy() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::braids_conjurer_adept());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).map(|c| c.controller), Some(1));
}

/// Jhoira of the Ghitu suspends an exiled card with four time counters.
#[test]
fn jhoira_suspends_from_hand() {
    let mut g = pod(2);
    let jhoira = ready(&mut g, 0, catalog::jhoira_of_the_ghitu());
    let angel = g.add_card_to_hand(0, catalog::serra_angel());
    flood(&mut g);
    activate(&mut g, jhoira, 0).expect("exile and suspend");
    let c = g.exile.iter().find(|c| c.id == angel).expect("exiled");
    assert_eq!(c.counter_count(CounterType::Time), 4);
}

/// Rilsa Rael takes the initiative entering, and gives an attacker
/// deathtouch.
#[test]
fn rilsa_rael_initiative_and_deathtouch() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    let rilsa = g.add_card_to_hand(0, catalog::rilsa_rael_kingpin());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g);
    cast(&mut g, rilsa);
    assert_eq!(g.initiative, Some(0));
    attack_with(&mut g, &[bear]);
    assert!(has_kw(&g, bear, Keyword::Deathtouch));
}

/// Court of Locthwain: the monarch exiles an opponent's top card and may cast
/// it free that turn.
#[test]
fn court_of_locthwain_steals_a_free_spell() {
    let mut g = pod(2);
    g.add_card_to_library(1, catalog::grizzly_bears());
    let court = g.add_card_to_hand(0, catalog::court_of_locthwain());
    flood(&mut g);
    cast(&mut g, court);
    assert_eq!(g.monarch, Some(0));
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let bear = g.exile.iter().find(|c| c.definition.name == "Grizzly Bears").expect("exiled").id;
    g.players[0].mana_pool = Default::default();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("free cast from exile");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).map(|c| c.controller), Some(0));
}

/// Court of Vantress: as the monarch, a token copy of another artifact.
#[test]
fn court_of_vantress_copies_as_monarch() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::sol_ring());
    let court = g.add_card_to_hand(0, catalog::court_of_vantress());
    flood(&mut g);
    cast(&mut g, court);
    yes(&mut g, 1);
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(named(&g, "Sol Ring"), 2);
}

/// The Lion-Turtle can't attack or block until three Lessons sit in your
/// graveyard.
#[test]
fn lion_turtle_waits_for_three_lessons() {
    let mut g = pod(2);
    let turtle = ready(&mut g, 0, catalog::the_lion_turtle());
    assert!(has_kw(&g, turtle, Keyword::CantAttack) && has_kw(&g, turtle, Keyword::CantBlock));
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::overwhelming_victory());
    }
    assert!(!has_kw(&g, turtle, Keyword::CantAttack) && !has_kw(&g, turtle, Keyword::CantBlock));
}

/// White Lotus Hideout's restricted mana pays for a Shrine as well as a
/// Lesson (Honden of Cleansing Fire: {3}{W}).
#[test]
fn white_lotus_hideout_funds_a_shrine() {
    let mut g = pod(2);
    let hideout = ready(&mut g, 0, catalog::white_lotus_hideout());
    let honden = g.add_card_to_hand(0, catalog::honden_of_cleansing_fire());
    {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::White)]));
    }
    activate(&mut g, hideout, 1).expect("restricted white");
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, honden);
    assert!(g.battlefield_find(honden).is_some());
}
