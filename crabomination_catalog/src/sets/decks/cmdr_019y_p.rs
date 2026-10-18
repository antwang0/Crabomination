//! Commander: 2026-10-18 (`019Y6J4R`) closers, sixteenth batch — each closes
//! two otherwise-buildable EDHREC average decks. Tests in
//! `tests/recent_b/cmdr_019y_p.rs`.

use crate::card::{
    AdditionalCastCost, CardDefinition, CardType, CreatureType, EnchantmentSubtype, Keyword, ModesWiden,
    Predicate, SelectionRequirement as R, SpellSubtype, Subtypes, Supertype, TokenDefinition, Value, Zone,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, generic, r, u, w};
use std::sync::Arc;

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn counter_unless(n: u32) -> Effect {
    Effect::CounterUnlessPaid {
        what: target_filtered(R::IsSpellOnStack),
        mana_cost: cost(&[generic(n)]),
        exile: false,
        extra_generic: None,
        if_paid: None,
    }
}

/// The Raven's Warning — {1}{W}{U} Saga. I — a 1/1 blue Bird token with
/// flying; gain 2 life. II — whenever one or more creatures you control with
/// flying deal combat damage to a player this turn, look at that player's
/// hand and draw a card. III — you may put a card you own from outside the
/// game on top of your library: a no-op here (INCOMPLETE_CARDS — a Commander
/// game has no sideboard to take it from).
pub fn the_ravens_warning() -> CardDefinition {
    let bird = TokenDefinition {
        name: "Bird".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "The Raven's Warning",
        cost: cost(&[generic(1), w(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::Seq(vec![
                    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(bird) },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ]),
            ),
            (
                2,
                Effect::MatchingCreaturesDealingCombatDamageBatchThisTurn {
                    filter: R::Creature.and(R::HasKeyword(Keyword::Flying)),
                    body: Box::new(Effect::Seq(vec![
                        Effect::LookAtHand { who: Selector::Player(PlayerRef::TriggerEventPlayer) },
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                    ])),
                },
            ),
            (3, Effect::Noop),
        ],
        ..Default::default()
    }
}

/// Blood on the Snow — {4}{B}{B} snow sorcery. Choose one: destroy all
/// creatures; or destroy all planeswalkers. Then return a creature or
/// planeswalker card with mana value X or less from your graveyard to the
/// battlefield, where X is the {S} spent to cast it.
pub fn blood_on_the_snow() -> CardDefinition {
    let returnable = R::InGraveyard
        .and(R::OwnedByYou)
        .and(R::Creature.or(R::Planeswalker))
        .and(R::ManaValueAtMostXFromCost);
    let then_return = Effect::MoveChosen {
        from: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Any },
        filter: Some(returnable),
        count: Value::ONE,
        up_to: false,
        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
    };
    CardDefinition {
        supertypes: vec![Supertype::Snow],
        x_as_cast: Some(Value::SnowManaSpentToCastSource),
        ..spell(
            "Blood on the Snow",
            cost(&[generic(4), b(), b()]),
            CardType::Sorcery,
            Effect::ChooseMode(vec![
                Effect::Seq(vec![Effect::Destroy { what: Selector::EachPermanent(R::Creature) }, then_return.clone()]),
                Effect::Seq(vec![Effect::Destroy { what: Selector::EachPermanent(R::Planeswalker) }, then_return]),
            ]),
        )
    }
}

/// We Say Thee Nay! — {1}{U} instant — Arcane. Teamwork 2. Counter target
/// spell unless its controller pays {2}, or {4} if it was cast using teamwork.
pub fn we_say_thee_nay() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { spell_subtypes: vec![SpellSubtype::Arcane], ..Default::default() },
        kicker_action_cost: Some(AdditionalCastCost::TapTotalPower { filter: R::Creature.and(R::ControlledByYou), power: 2 }),
        ..spell(
            "We Say Thee Nay!",
            cost(&[generic(1), u()]),
            CardType::Instant,
            Effect::If { cond: Predicate::SpellWasKicked, then: Box::new(counter_unless(4)), else_: Box::new(counter_unless(2)) },
        )
    }
}

/// HULK SMASH! — {1}{R} instant. Teamwork 4. Choose one — both if cast using
/// teamwork: destroy target noncreature artifact; target creature you control
/// deals damage equal to its power to target creature an opponent controls.
pub fn hulk_smash() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::TapTotalPower { filter: R::Creature.and(R::ControlledByYou), power: 4 }),
        modes_widen: Some(ModesWiden::Kicked),
        ..spell(
            "HULK SMASH!",
            cost(&[generic(1), r()]),
            CardType::Instant,
            Effect::ChooseModesCast {
                modes: vec![
                    Effect::Destroy { what: target_filtered(R::Artifact.and(R::Creature.negate())) },
                    Effect::DealDamageEqualToPower {
                        source: target_filtered(R::Creature.and(R::ControlledByYou)),
                        target: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
                    },
                ],
                min: 1,
                max: 1,
                allow_repeats: false,
            },
        )
    }
}
