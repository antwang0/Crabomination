//! Functionality tests for `catalog::sets::decks::recent225`.

use crabomination::card::ArtifactSubtype;
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game, GameAction};
use crabomination::mana::Color;

/// Savage Ventmaw's attack mana survives step/phase emptying this turn but is
/// gone at cleanup (CR 500.4 exception).
#[test]
fn savage_ventmaw_mana_persists_this_turn() {
    let mut g = two_player_game();
    let vent = g.add_card_to_battlefield(0, catalog::savage_ventmaw());
    let effect = catalog::savage_ventmaw().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_trigger(vent, 0, None, 0)).unwrap();
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 3);
    // A step/phase boundary empties pools — the kept mana re-seeds.
    g.empty_mana_pools();
    assert_eq!(g.players[0].mana_pool.total(), 6, "kept mana survives the empty");
    // Cleanup clears the kept record, so the turn's final empty removes it.
    g.players[0].kept_mana_this_turn.empty();
    g.empty_mana_pools();
    assert_eq!(g.players[0].mana_pool.total(), 0, "kept mana gone at end of turn");
}

/// Fake Your Own Death: the pumped creature returns tapped and mints a Treasure
/// when it dies this turn.
#[test]
fn fake_your_own_death_revives_with_treasure() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::fake_your_own_death());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("castable for {1}{B}");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+0");
    // Kill the bear; its granted trigger should return it tapped + make a Treasure.
    let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_ability(bear, 0, None) };
    g.resolve_effect(&crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::Target(0) }, &ctx).unwrap();
    drain_stack(&mut g);
    let revived = g.battlefield.iter().find(|c| c.id == bear).expect("bear back on battlefield");
    assert!(revived.tapped, "returned tapped");
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0
            && c.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Treasure)),
        "a Treasure was created",
    );
}

fn count_named(g: &crabomination::game::GameState, ctrl: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == ctrl && c.definition.name == name).count()
}

/// Dread Summons makes a Zombie for each creature card milled.
#[test]
fn dread_summons_mills_and_makes_zombies() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let effect = catalog::dread_summons().effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_spell(0, None, 0, 2)).unwrap();
    assert_eq!(count_named(&g, 0, "Zombie"), 2, "one Zombie per creature card milled");
}

/// On the Job pumps your team and investigates.
#[test]
fn on_the_job_pumps_and_investigates() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.resolve_effect(&catalog::on_the_job().effect.clone(), &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+1");
    assert_eq!(count_named(&g, 0, "Clue"), 1, "investigated");
}

/// Makeshift Binding exiles an opponent's creature and gains 2 life.
#[test]
fn makeshift_binding_exiles_and_gains() {
    let mut g = two_player_game();
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let src = g.add_card_to_battlefield(0, catalog::makeshift_binding());
    let effect = catalog::makeshift_binding().triggered_abilities[0].effect.clone();
    let ctx = EffectContext { targets: vec![Target::Permanent(enemy)], ..EffectContext::for_trigger(src, 0, None, 0) };
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.exile.iter().any(|c| c.id == enemy), "enemy exiled");
    assert_eq!(g.players[0].life, 22, "gained 2 life");
}

/// Long Goodbye destroys a cheap creature.
#[test]
fn long_goodbye_destroys_cheap_creature() {
    let mut g = two_player_game();
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = EffectContext { targets: vec![Target::Permanent(enemy)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&catalog::long_goodbye().effect.clone(), &ctx).unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == enemy), "destroyed");
    assert!(catalog::long_goodbye().keywords.contains(&crabomination::card::Keyword::CantBeCountered));
}

/// It Doesn't Add Up reanimates a creature and suspects it.
#[test]
fn it_doesnt_add_up_reanimates_and_suspects() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let ctx = EffectContext { targets: vec![Target::Permanent(dead)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&catalog::it_doesnt_add_up().effect.clone(), &ctx).unwrap();
    let back = g.battlefield.iter().find(|c| c.id == dead).expect("reanimated");
    assert!(back.suspected, "suspected");
}

/// Eliminate the Impossible shrinks the opponent's board and investigates.
#[test]
fn eliminate_the_impossible_shrinks_opponents() {
    let mut g = two_player_game();
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.resolve_effect(&catalog::eliminate_the_impossible().effect.clone(), &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    assert_eq!(g.computed_permanent(enemy).unwrap().power, 0, "-2/-0");
    assert_eq!(count_named(&g, 0, "Clue"), 1, "investigated");
}

/// Unauthorized Exit bounces a nonland permanent.
#[test]
fn unauthorized_exit_bounces() {
    let mut g = two_player_game();
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = EffectContext { targets: vec![Target::Permanent(enemy)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&catalog::unauthorized_exit().effect.clone(), &ctx).unwrap();
    assert!(g.players[1].hand.iter().any(|c| c.id == enemy), "returned to owner's hand");
}

/// Seraphic Steed's saddled attack mints a flying Angel.
#[test]
fn seraphic_steed_makes_angel() {
    let mut g = two_player_game();
    let steed = g.add_card_to_battlefield(0, catalog::seraphic_steed());
    let effect = catalog::seraphic_steed().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_trigger(steed, 0, None, 0)).unwrap();
    let angel = g.battlefield.iter().find(|c| c.definition.name == "Angel").expect("angel token");
    assert!(g.computed_permanent(angel.id).unwrap().keywords.contains(&crabomination::card::Keyword::Flying));
}

/// Sandstorm Salvager's ETB makes a 3/3 Golem.
#[test]
fn sandstorm_salvager_makes_golem() {
    let mut g = two_player_game();
    let sal = g.add_card_to_battlefield(0, catalog::sandstorm_salvager());
    let effect = catalog::sandstorm_salvager().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_trigger(sal, 0, None, 0)).unwrap();
    assert_eq!(count_named(&g, 0, "Golem"), 1, "a Golem entered");
}

/// Nightdrinker Moroii costs 3 life on entry.
#[test]
fn nightdrinker_moroii_loses_life() {
    let mut g = two_player_game();
    let m = g.add_card_to_battlefield(0, catalog::nightdrinker_moroii());
    let effect = catalog::nightdrinker_moroii().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_trigger(m, 0, None, 0)).unwrap();
    assert_eq!(g.players[0].life, 17, "lost 3 life");
}

/// Mirage Mesa taps for its chosen color.
#[test]
fn mirage_mesa_taps_for_chosen_color() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::mirage_mesa());
    g.resolve_effect(&crabomination::effect::Effect::ChooseColorForSelf, &EffectContext::for_ability(land, 0, None)).unwrap();
    let mana = catalog::mirage_mesa().activated_abilities[0].effect.clone();
    g.resolve_effect(&mana, &EffectContext::for_ability(land, 0, None)).unwrap();
    assert_eq!(g.players[0].mana_pool.total(), 1, "produced one mana of the chosen color");
}

/// Valgavoth's Lair is hexproof.
#[test]
fn valgavoths_lair_is_hexproof() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::valgavoths_lair());
    assert!(g.computed_permanent(land).unwrap().keywords.contains(&crabomination::card::Keyword::Hexproof));
}

/// Sandstorm Verge can stop a blocker.
#[test]
fn sandstorm_verge_prevents_block() {
    let mut g = two_player_game();
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(0, catalog::sandstorm_verge());
    let effect = catalog::sandstorm_verge().activated_abilities[1].effect.clone();
    let ctx = EffectContext { targets: vec![Target::Permanent(enemy)], ..EffectContext::for_ability(land, 0, None) };
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.computed_permanent(enemy).unwrap().keywords.contains(&crabomination::card::Keyword::CantBlock));
}

/// Pitiless Carnage is plottable.
#[test]
fn pitiless_carnage_is_plottable() {
    assert!(catalog::pitiless_carnage().plot_cost.is_some());
}

/// Seasoned Consultant's battalion trigger pumps it.
#[test]
fn seasoned_consultant_battalion_pumps() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::seasoned_consultant());
    let effect = catalog::seasoned_consultant().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_trigger(c, 0, None, 0)).unwrap();
    assert_eq!(g.computed_permanent(c).unwrap().power, 3, "1 + 2 = 3");
}

/// Terramorphic Expanse fetches a basic land tapped.
#[test]
fn terramorphic_expanse_fetches_basic() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    let land = g.add_card_to_battlefield(0, catalog::terramorphic_expanse());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(Some(forest)),
    ]));
    let effect = catalog::terramorphic_expanse().activated_abilities[0].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_ability(land, 0, None)).unwrap();
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Forest" && c.tapped), "fetched Forest tapped");
}

/// Wojek Investigator investigates on the trigger.
#[test]
fn wojek_investigator_investigates() {
    let mut g = two_player_game();
    let w = g.add_card_to_battlefield(0, catalog::wojek_investigator());
    let effect = catalog::wojek_investigator().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_trigger(w, 0, None, 0)).unwrap();
    assert_eq!(count_named(&g, 0, "Clue"), 1, "investigated");
}
