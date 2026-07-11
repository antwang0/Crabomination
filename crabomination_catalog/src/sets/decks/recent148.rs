//! A wave spanning WOE and Duskmourn (DSK): a Faerie-token trick, a sacrifice
//! aristocrat, and two manifest-dread Nightmares (one an Equipment). All ride
//! existing primitives. Tests in `crabomination/src/tests/recent148.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EquipBonus, Keyword,
    SelectionRequirement as R, Selector, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef};
use crate::mana::{b, cost, generic, u, Color};

/// 1/1 blue Faerie token with flying.
fn faerie_token() -> TokenDefinition {
    TokenDefinition {
        name: "Faerie".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Faebloom Trick — {2}{U} Instant. Make two 1/1 blue Faerie flyers, then tap
/// an opponent's creature.
pub fn faebloom_trick() -> CardDefinition {
    CardDefinition {
        name: "Faebloom Trick",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: faerie_token() },
            Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
        ]),
        ..Default::default()
    }
}

/// Popular Egotist — {2}{B} 3/2 Human Rogue. {1}{B}, sac another creature or
/// enchantment: indestructible until end of turn, tapped. Sacrifices drain an
/// opponent for 1.
pub fn popular_egotist() -> CardDefinition {
    CardDefinition {
        name: "Popular Egotist",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_other_filter: Some((R::Creature.or(R::Enchantment), 1)),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
                Effect::Tap { what: Selector::This },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::LoseLife { who: target_filtered(R::OpponentPlayer), amount: Value::ONE },
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..Default::default()
    }
}

/// Overwhelmed Apprentice — {U} 1/2 Human Wizard. ETB each opponent mills two,
/// then you scry 2.
pub fn overwhelmed_apprentice() -> CardDefinition {
    CardDefinition {
        name: "Overwhelmed Apprentice",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        ]))],
        ..Default::default()
    }
}

/// Fear of Impostors — {1}{U}{U} 3/2 Nightmare with flash. ETB counters a spell;
/// its controller manifests dread.
pub fn fear_of_impostors() -> CardDefinition {
    CardDefinition {
        name: "Fear of Impostors",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Nightmare], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::ManifestDread { who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))) },
        ]))],
        ..Default::default()
    }
}

/// Cursed Windbreaker — {2}{U} Equipment. ETB manifests dread and attaches to
/// that creature; equipped creature has flying. Equip {3}.
pub fn cursed_windbreaker() -> CardDefinition {
    CardDefinition {
        name: "Cursed Windbreaker",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ManifestDread { who: PlayerRef::You },
            // The manifest is the (only) face-down creature you control; attach
            // to it. `LastMoved` is unreliable here — ManifestDread's second
            // card goes to the graveyard after the manifest.
            Effect::Attach {
                what: Selector::This,
                to: Selector::take(
                    Selector::EachPermanent(R::FaceDown.and(R::ControlledByYou)),
                    Value::ONE,
                ),
            },
        ]))],
        ..Default::default()
    }
}
