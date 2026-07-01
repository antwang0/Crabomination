//! Value legends and recursion staples across G/B/R/W. Tests in
//! `tests/recent47.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CreatureType, DynamicPt, Effect,
    Keyword, Predicate, SelectionRequirement as R, Selector, Subtypes, Supertype, TokenDefinition,
    Value,
};
use crate::effect::shortcut::{exploit, target_filtered};
use crate::effect::{PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, r, w};

/// Multani, Yavimaya's Avatar — {4}{G}{G} 0/0 legendary Elemental Avatar. Reach,
/// trample. +1/+1 for each land you control and each land card in your
/// graveyard. {1}{G}, return two lands you control to their owners' hands:
/// return this from your graveyard to your hand.
pub fn multani_yavimayas_avatar() -> CardDefinition {
    CardDefinition {
        name: "Multani, Yavimaya's Avatar",
        cost: cost(&[generic(4), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Avatar],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Reach, Keyword::Trample],
        dynamic_pt: Some(DynamicPt::LandsControlledPlusLandsInControllerGraveyard { base: 0 }),
        activated_abilities: vec![ActivatedAbility {
            from_graveyard: true,
            mana_cost: cost(&[generic(1), g()]),
            bounce_other_filter: Some((R::Land.and(R::ControlledByYou), 2)),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Nullmage Shepherd — {3}{G} 2/4 Elf Shaman. Tap four untapped creatures you
/// control: destroy target artifact or enchantment.
pub fn nullmage_shepherd() -> CardDefinition {
    CardDefinition {
        name: "Nullmage Shepherd",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((R::Creature.and(R::ControlledByYou), 4)),
            effect: Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Magus of the Wheel — {2}{R} 3/3 Human Wizard. {1}{R}, {T}, sacrifice this:
/// each player discards their hand, then draws seven cards.
pub fn magus_of_the_wheel() -> CardDefinition {
    CardDefinition {
        name: "Magus of the Wheel",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(100),
                    random: false,
                },
                Effect::Draw { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(7) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Bankrupt in Blood — {1}{B} Sorcery. Additional cost: sacrifice two creatures.
/// Draw three cards.
pub fn bankrupt_in_blood() -> CardDefinition {
    CardDefinition {
        name: "Bankrupt in Blood",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent { filter: R::Creature, count: 2 }],
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(3) },
        ..Default::default()
    }
}

/// Sidisi, Undead Vizier — {3}{B}{B} 4/6 legendary Zombie Snake. Deathtouch,
/// exploit. When Sidisi exploits a creature, search your library for a card,
/// put it into your hand, then shuffle.
pub fn sidisi_undead_vizier() -> CardDefinition {
    CardDefinition {
        name: "Sidisi, Undead Vizier",
        cost: cost(&[generic(3), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Snake],
            ..Default::default()
        },
        power: 4,
        toughness: 6,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![exploit(Effect::Search {
            who: PlayerRef::You,
            filter: R::Any,
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Nighthawk Scavenger — {1}{B}{B} 1+*/3 Vampire Rogue. Flying, deathtouch,
/// lifelink. Power = 1 + the number of card types among cards in your opponents'
/// graveyards.
pub fn nighthawk_scavenger() -> CardDefinition {
    CardDefinition {
        name: "Nighthawk Scavenger",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch, Keyword::Lifelink],
        dynamic_pt: Some(DynamicPt::CardTypesInOpponentsGraveyards { base_p: 1, base_t: 3 }),
        ..Default::default()
    }
}

/// Speaker of the Heavens — {W} 1/1 Human Cleric. Vigilance, lifelink. {T}:
/// create a 4/4 white Angel with flying. Activate only if your life total is at
/// least 7 more than your starting total, and only as a sorcery.
pub fn speaker_of_the_heavens() -> CardDefinition {
    let angel = TokenDefinition {
        name: "Angel".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        keywords: vec![Keyword::Flying],
        colors: vec![crate::mana::Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Speaker of the Heavens",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Vigilance, Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            condition: Some(Predicate::PlayerLifeAtLeastAboveStarting { who: PlayerRef::You, delta: 7 }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: angel,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
