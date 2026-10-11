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
