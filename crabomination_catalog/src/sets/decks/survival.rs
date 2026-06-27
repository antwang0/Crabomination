//! Bloomburrow **Survival** creatures (CR 702.180). "Survival — At the
//! beginning of your second main phase, if this creature is tapped, [effect]."
//! Modeled as a `StepBegins(PostCombatMain)` / `ActivePlayer` trigger whose
//! body runs under an intervening-`if` that the source is tapped. Tracked in
//! `DECK_FEATURES.md`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement, Selector, Subtypes, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{PlayerRef, RevealMissDest, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, w, Color, ManaCost};

/// A 1/1 white Glimmer enchantment creature — Glimmer Seeker's payoff token.
fn glimmer_token() -> TokenDefinition {
    TokenDefinition {
        name: "Glimmer".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Enchantment, CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Glimmer],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// "Survival — At the beginning of your second main phase, if this creature is
/// tapped, [effect]." Wraps `effect` in the tapped intervening-`if`.
fn survival(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(
            EventKind::StepBegins(TurnStep::PostCombatMain),
            EventScope::ActivePlayer,
        ),
        effect: Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::This,
                filter: SelectionRequirement::Tapped,
            },
            then: Box::new(effect),
            else_: Box::new(Effect::Noop),
        },
    }
}

fn survivor(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    power: i32,
    toughness: i32,
    effect: Effect,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power,
        toughness,
        triggered_abilities: vec![survival(effect)],
        ..Default::default()
    }
}

/// Acrobatic Cheerleader — {1}{W} 2/2 Human Survivor. Survival: put a flying
/// counter on it (the "triggers only once" rider is modeled by gating on it
/// not already having flying).
pub fn acrobatic_cheerleader() -> CardDefinition {
    survivor(
        "Acrobatic Cheerleader",
        cost(&[generic(1), w()]),
        vec![CreatureType::Human, CreatureType::Survivor],
        2,
        2,
        Effect::If {
            cond: Predicate::Not(Box::new(Predicate::EntityMatches {
                what: Selector::This,
                filter: SelectionRequirement::HasKeyword(Keyword::Flying),
            })),
            then: Box::new(Effect::AddKeywordCounter {
                what: Selector::This,
                keyword: Keyword::Flying,
                amount: Value::Const(1),
            }),
            else_: Box::new(Effect::Noop),
        },
    )
}

/// Cautious Survivor — {3}{G} 4/4. Survival: gain 2 life.
pub fn cautious_survivor() -> CardDefinition {
    survivor(
        "Cautious Survivor",
        cost(&[generic(3), g()]),
        vec![CreatureType::Elf, CreatureType::Survivor],
        4,
        4,
        Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
    )
}

/// Defiant Survivor — {2}{G} 3/2. Survival: manifest dread.
pub fn defiant_survivor() -> CardDefinition {
    survivor(
        "Defiant Survivor",
        cost(&[generic(2), g()]),
        vec![CreatureType::Human, CreatureType::Survivor],
        3,
        2,
        Effect::ManifestDread { who: PlayerRef::You },
    )
}

/// Shrewd Storyteller — {1}{G}{W} 3/3. Survival: put a +1/+1 counter on target
/// creature.
pub fn shrewd_storyteller() -> CardDefinition {
    survivor(
        "Shrewd Storyteller",
        cost(&[generic(1), g(), w()]),
        vec![CreatureType::Human, CreatureType::Survivor],
        3,
        3,
        Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        },
    )
}

/// Kona, Rescue Beastie — {3}{G} 4/3 legendary. Survival: put a permanent card
/// from your hand onto the battlefield. (The "you may" is modeled as the put.)
pub fn kona_rescue_beastie() -> CardDefinition {
    CardDefinition {
        name: "Kona, Rescue Beastie",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![crate::card::Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast, CreatureType::Survivor],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![survival(Effect::PutFromHandOntoBattlefield {
            who: PlayerRef::You,
            filter: SelectionRequirement::Permanent,
            count: Value::Const(1),
            tapped: false,
            haste: false,
            sacrifice_eot: false,
        })],
        ..Default::default()
    }
}

/// Cynical Loner — {1}{B} 3/1. Survival: search your library for a card, put it
/// into your graveyard, then shuffle. (The "can't be blocked by Glimmers" rider
/// is omitted.)
pub fn cynical_loner() -> CardDefinition {
    survivor(
        "Cynical Loner",
        cost(&[generic(1), b()]),
        vec![CreatureType::Human, CreatureType::Survivor],
        3,
        1,
        Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Any,
            to: ZoneDest::Graveyard,
        },
    )
}

/// Glimmer Seeker — {2}{W} 3/3. Survival: draw a card if you control a Glimmer
/// creature; otherwise create a 1/1 white Glimmer enchantment creature token.
pub fn glimmer_seeker() -> CardDefinition {
    survivor(
        "Glimmer Seeker",
        cost(&[generic(2), w()]),
        vec![CreatureType::Human, CreatureType::Survivor],
        3,
        3,
        Effect::If {
            cond: Predicate::SelectorExists(Selector::ControlledBy {
                who: PlayerRef::You,
                filter: SelectionRequirement::HasCreatureType(CreatureType::Glimmer),
            }),
            then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            else_: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: glimmer_token(),
            }),
        },
    )
}

/// House Cartographer — {1}{G} 2/2. Survival: reveal cards from the top of your
/// library until you reveal a land card, put it into your hand, the rest on the
/// bottom in a random order.
pub fn house_cartographer() -> CardDefinition {
    survivor(
        "House Cartographer",
        cost(&[generic(1), g()]),
        vec![CreatureType::Human, CreatureType::Scout, CreatureType::Survivor],
        2,
        2,
        Effect::RevealUntilFind {
            who: PlayerRef::You,
            find: SelectionRequirement::Land,
            to: ZoneDest::Hand(PlayerRef::You),
            cap: Value::Const(99),
            life_per_revealed: 0,
            miss_dest: RevealMissDest::BottomRandom,
        },
    )
}

/// Savior of the Small — {3}{W} 3/4. Survival: return target creature card with
/// mana value 3 or less from your graveyard to your hand.
pub fn savior_of_the_small() -> CardDefinition {
    survivor(
        "Savior of the Small",
        cost(&[generic(3), w()]),
        vec![CreatureType::Kor, CreatureType::Survivor],
        3,
        4,
        Effect::Move {
            what: target_filtered(
                SelectionRequirement::InYourGraveyard
                    .and(SelectionRequirement::Creature)
                    .and(SelectionRequirement::ManaValueAtMost(3)),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        },
    )
}
