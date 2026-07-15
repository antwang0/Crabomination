//! Functionality tests for `catalog::sets::decks::recent230`.

use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::effects::EffectContext;
use crate::game::types::Target;
use crate::game::{drain_stack, two_player_game};

/// Wickerfolk Thresher digs the top land onto the battlefield when delirium is
/// active.
#[test]
fn wickerfolk_thresher_digs_a_land_with_delirium() {
    let mut g = two_player_game();
    let thresher = g.add_card_to_battlefield(0, catalog::wickerfolk_thresher());
    // Put a land on top of the library.
    let land_id = crate::card::CardId(9001);
    g.players[0].add_to_library_top(land_id, catalog::forest());
    let before = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Forest").count();
    let effect = catalog::wickerfolk_thresher().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_trigger(thresher, 0, None, 0)).unwrap();
    drain_stack(&mut g);
    let after = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Forest").count();
    assert_eq!(after, before + 1, "the top land is put onto the battlefield");
}

/// The attack trigger only fires with delirium active (four+ card types in gy).
#[test]
fn wickerfolk_thresher_gated_on_delirium() {
    use crate::effect::{PlayerRef, Predicate};
    let def = catalog::wickerfolk_thresher();
    let filter = def.triggered_abilities[0].event.filter.clone();
    assert!(matches!(filter, Some(Predicate::DeliriumActive { who: PlayerRef::You })));
}

/// Resilient Roadrunner has haste and protection from Coyotes; its {3} makes it
/// blockable only by haste creatures.
#[test]
fn resilient_roadrunner_evasion() {
    let mut g = two_player_game();
    let rr = g.add_card_to_battlefield(0, catalog::resilient_roadrunner());
    let cp = g.computed_permanent(rr).unwrap();
    assert!(cp.keywords.contains(&Keyword::Haste));
    assert!(cp.keywords.contains(&Keyword::ProtectionFromCreatureType(CreatureType::Coyote)));
    let effect = catalog::resilient_roadrunner().activated_abilities[0].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_ability(rr, 0, None)).unwrap();
    assert!(
        g.computed_permanent(rr)
            .unwrap()
            .keywords
            .iter()
            .any(|k| matches!(k, Keyword::CantBeBlockedExceptBy(_))),
        "gains can't-be-blocked-except-by-haste"
    );
}

/// Giant Beaver's saddled-attack trigger puts a +1/+1 counter on a creature you
/// control.
#[test]
fn giant_beaver_counter_when_saddled() {
    let mut g = two_player_game();
    let beaver = g.add_card_to_battlefield(0, catalog::giant_beaver());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let effect = catalog::giant_beaver().triggered_abilities[0].effect.clone();
    let ctx = EffectContext {
        targets: vec![Target::Permanent(ally)],
        ..EffectContext::for_trigger(beaver, 0, None, 0)
    };
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.battlefield_find(ally).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(catalog::giant_beaver().keywords.iter().filter(|k| matches!(k, Keyword::Saddle(3))).count(), 1);
}

/// Ornery Tumblewagg's saddled-attack trigger doubles the +1/+1 counters on a
/// target creature.
#[test]
fn ornery_tumblewagg_doubles_counters() {
    let mut g = two_player_game();
    let wagg = g.add_card_to_battlefield(0, catalog::ornery_tumblewagg());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(ally).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    // second ability = the saddled-attack doubler
    let effect = catalog::ornery_tumblewagg().triggered_abilities[1].effect.clone();
    let ctx = EffectContext {
        targets: vec![Target::Permanent(ally)],
        ..EffectContext::for_trigger(wagg, 0, None, 0)
    };
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.battlefield_find(ally).unwrap().counter_count(CounterType::PlusOnePlusOne), 6, "3 → 6");
}
