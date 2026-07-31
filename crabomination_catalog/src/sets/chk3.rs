//! Champions of Kamigawa (CHK) closure — the last six gap cards, each on a
//! primitive added with it. Tests in `classic_sets/chk::gaps2`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, SpellSubtype, StaticAbility, Subtypes, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, PlayerRef, Selector, StaticEffect};
use crate::mana::{b, cost, generic, r, u};

fn types(t: Vec<CreatureType>) -> Subtypes {
    Subtypes {
        creature_types: t,
        ..Default::default()
    }
}

/// Hisoka's Guard — {1}{U} 1/1. You may choose not to untap it during your
/// untap step. {1}{U}, {T}: another creature you control has shroud for as
/// long as this creature remains tapped.
pub fn hisokas_guard() -> CardDefinition {
    CardDefinition {
        name: "Hisoka's Guard",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Human, CreatureType::Wizard]),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::MayChooseNotToUntap],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            tap_cost: true,
            effect: Effect::GrantKeywordWhileSourceTapped {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                keyword: Keyword::Shroud,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mindblaze — {5}{R} Sorcery. Name a nonland card and a number greater than
/// 0; target player reveals their library and takes 8 damage if it holds
/// exactly that many cards with that name. Then they shuffle.
pub fn mindblaze() -> CardDefinition {
    CardDefinition {
        name: "Mindblaze",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::NameCard { what: Selector::This, restrict_to: None },
            Effect::RevealLibraryNamedCountPunish {
                who: target_filtered(R::Player),
                damage: Value::Const(8),
            },
        ]),
        ..Default::default()
    }
}

/// Moonring Mirror — {5} Artifact. Whenever you draw a card, exile the top
/// card of your library face down. At the beginning of your upkeep, you may
/// exile your hand face down to take back everything already stashed here.
pub fn moonring_mirror() -> CardDefinition {
    CardDefinition {
        name: "Moonring Mirror",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
                effect: Effect::ExileTopOfLibrary {
                    who: Selector::Player(PlayerRef::You),
                    amount: Value::ONE,
                    link_to_source: true,
                    face_down: true,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::ExileHandThenReclaimLinked,
            },
        ],
        ..Default::default()
    }
}

/// Reweave — {5}{U} Instant — Arcane. Target permanent's controller sacrifices
/// it, then reveals until a permanent card sharing a card type with it, puts
/// that onto the battlefield, and shuffles. Splice onto Arcane {2}{U}{U}.
pub fn reweave() -> CardDefinition {
    CardDefinition {
        name: "Reweave",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Arcane],
            ..Default::default()
        },
        keywords: vec![Keyword::Splice(
            cost(&[generic(2), u(), u()]),
            SpellSubtype::Arcane,
        )],
        effect: Effect::SacrificeThenRevealUntilSharedType {
            what: target_filtered(R::Permanent),
        },
        ..Default::default()
    }
}

/// Struggle for Sanity — {2}{B}{B} Sorcery. Target opponent reveals their
/// hand; they and you alternate exiling from it (they pick first). Their picks
/// go back to hand, yours to the graveyard.
pub fn struggle_for_sanity() -> CardDefinition {
    CardDefinition {
        name: "Struggle for Sanity",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::AlternatingExileFromHand {
            who: target_filtered(R::OpponentPlayer),
        },
        ..Default::default()
    }
}

/// Swirl the Mists — {2}{U}{U} Enchantment. As it enters, choose a color word;
/// every color word in the text of spells and permanents becomes that word.
pub fn swirl_the_mists() -> CardDefinition {
    CardDefinition {
        name: "Swirl the Mists",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Enchantment],
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        static_abilities: vec![StaticAbility {
            description: "All instances of color words are changed to the chosen color word.",
            effect: StaticEffect::AllColorWordsBecomeChosen,
        }],
        ..Default::default()
    }
}
