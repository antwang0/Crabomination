//! Commander: two-card gaps of EDHREC's top-2000 commanders (the 2026-10-13
//! rescan), twelfth file — Ishkanah and Gandalf, Party Guest. Tests in
//! `tests/recent_b/cmdr_rank2k_l.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, Selector, Subtypes, Supertype, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{ActivatedAbility, Effect, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, r, u, w, ManaCost};

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

/// Arachnus Spinner — {5}{G} 5/7 reach Spider. Tap an untapped Spider you
/// control: put a card named Arachnus Web from your graveyard and/or library
/// onto the battlefield attached to target creature (shuffling if the library
/// was searched).
/// Like Invasion of Ikoria, the library is always searched (and shuffled)
/// alongside the graveyard.
pub fn arachnus_spinner() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            tap_others_cost: Some((R::Creature.and(R::HasCreatureType(CreatureType::Spider)), 1)),
            effect: Effect::Seq(vec![
                Effect::PutOntoBattlefieldAttached {
                    zones: vec![Zone::Graveyard, Zone::Library],
                    filter: R::HasName("Arachnus Web".into()),
                    host: Some(target_filtered(R::Creature)),
                    max: Some(Value::ONE),
                    creatures_only: true,
                    equipment_unattached: false,
                },
                Effect::ShuffleLibrary { who: PlayerRef::You },
            ]),
            ..Default::default()
        }],
        ..creature("Arachnus Spinner", cost(&[generic(5), g()]), vec![CreatureType::Spider], 5, 7)
    }
}

/// Gandalf, Party Guest — {1}{U}{R}{W} legendary 3/4 Avatar Wizard. At the
/// beginning of combat on your turn, you may cast an instant or sorcery
/// spell with mana value up to twice your legendary Wizards from your hand
/// without paying its mana cost.
pub fn gandalf_party_guest() -> CardDefinition {
    let wizards = || {
        Value::count(Selector::EachPermanent(
            R::Creature
                .and(R::HasCreatureType(CreatureType::Wizard))
                .and(R::HasSupertype(Supertype::Legendary))
                .and(R::ControlledByYou),
        ))
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::MayCastFromHandFreeMatching {
                filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                max_mv: Value::Sum(vec![wizards(), wizards()]),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature(
            "Gandalf, Party Guest",
            cost(&[generic(1), u(), r(), w()]),
            vec![CreatureType::Avatar, CreatureType::Wizard],
            3,
            4,
        )
    }
}

/// Multiversal Incursion — {5}{U}{U} sorcery: for each nontoken creature you
/// control, create a token copy of it, except it isn't legendary.
pub fn multiversal_incursion() -> CardDefinition {
    spell(
        "Multiversal Incursion",
        cost(&[generic(5), u(), u()]),
        CardType::Sorcery,
        Effect::ForEach {
            selector: Selector::EachPermanent(R::Creature.and(R::NotToken).and(R::ControlledByYou)),
            body: Box::new(Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: Selector::TriggerSource,
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: true,
                legendary: false,
                extra_keywords: vec![],
                no_mana_cost: false,
                enters_with_counters: None,
                remove_keywords: vec![],
            }),
        },
    )
}

/// Traverse the Ulvenwald — {G} sorcery: a basic land card to hand; with
/// delirium, a creature or land card instead.
pub fn traverse_the_ulvenwald() -> CardDefinition {
    spell(
        "Traverse the Ulvenwald",
        cost(&[g()]),
        CardType::Sorcery,
        Effect::If {
            cond: Predicate::DeliriumActive { who: PlayerRef::You },
            then: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::Creature.or(R::Land),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            else_: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::Land.and(R::IsBasicLand),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        },
    )
}
