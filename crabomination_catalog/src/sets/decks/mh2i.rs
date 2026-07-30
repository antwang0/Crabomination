//! Modern Horizons 2 sweep, batch 10 (final) — linked phasing (CR 702.26),
//! suspend-only spells (CR 601.3e), Warp-World shuffles, off-battlefield
//! CDA types (CR 604.3). Tests in `tests/mh2i.rs`. Completes the MH2 set.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, Keyword,
    LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement, Selector, Subtypes, Supertype,
    Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, ZoneDest};
use crate::mana::{ManaCost, b, cost, g, generic, r, u, w, x};

use SelectionRequirement as R;

/// Out of Time — {1}{W}{W} enchantment, vanishing. ETB: untap all creatures,
/// then they phase out until this leaves; a time counter per creature phased.
pub fn out_of_time() -> CardDefinition {
    CardDefinition {
        name: "Out of Time",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Vanishing(0)],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Untap {
                what: Selector::EachPermanent(R::Creature),
                up_to: None,
            },
            Effect::PhaseOut {
                what: Selector::EachPermanent(R::Creature),
                until_source_leaves: true,
            },
        ]))],
        ..Default::default()
    }
}

/// Gaea's Will — suspend-only sorcery (Suspend 4—{G}): play lands and cast
/// spells from your graveyard this turn; your graveyard-bound cards are
/// exiled this turn.
pub fn gaeas_will() -> CardDefinition {
    CardDefinition {
        name: "Gaea's Will",
        cost: ManaCost::default(),
        card_types: vec![CardType::Sorcery],
        no_mana_cost: true,
        keywords: vec![Keyword::Suspend(4, cost(&[g()]))],
        effect: Effect::Seq(vec![
            Effect::PlayFromGraveyardThisTurn,
            Effect::ExileYourGraveyardBoundThisTurn,
        ]),
        ..Default::default()
    }
}

/// Inevitable Betrayal — suspend-only sorcery (Suspend 3—{1}{U}{U}): search
/// target opponent's library for a creature, put it under your control.
pub fn inevitable_betrayal() -> CardDefinition {
    CardDefinition {
        name: "Inevitable Betrayal",
        cost: ManaCost::default(),
        card_types: vec![CardType::Sorcery],
        no_mana_cost: true,
        keywords: vec![Keyword::Suspend(3, cost(&[generic(1), u(), u()]))],
        effect: Effect::SearchPickedBy {
            who: PlayerRef::Target(0),
            picker: PlayerRef::You,
            filter: R::Creature,
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        },
        ..Default::default()
    }
}

/// Glimpse of Tomorrow — suspend-only sorcery (Suspend 3—{R}{R}): shuffle
/// your permanents into your library and flip that many into play.
pub fn glimpse_of_tomorrow() -> CardDefinition {
    CardDefinition {
        name: "Glimpse of Tomorrow",
        cost: ManaCost::default(),
        card_types: vec![CardType::Sorcery],
        no_mana_cost: true,
        keywords: vec![Keyword::Suspend(3, cost(&[r(), r()]))],
        effect: Effect::GlimpseOfTomorrow,
        ..Default::default()
    }
}

/// Braingeyser — {X}{U}{U} sorcery: target player draws X cards.
pub fn braingeyser() -> CardDefinition {
    CardDefinition {
        name: "Braingeyser",
        cost: cost(&[x(), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::XFromCost,
        },
        ..Default::default()
    }
}

/// Garth One-Eye — {W}{U}{B}{R}{G} 5/5. {T}: choose an unchosen classic among
/// Disenchant, Braingeyser, Terror, Shivan Dragon, Regrowth, Black Lotus;
/// create a copy of it that you may cast.
pub fn garth_one_eye() -> CardDefinition {
    CardDefinition {
        name: "Garth One-Eye",
        cost: cost(&[w(), u(), b(), r(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GarthOneEye {
                names: vec![
                    "Disenchant".into(),
                    "Braingeyser".into(),
                    "Terror".into(),
                    "Shivan Dragon".into(),
                    "Regrowth".into(),
                    "Black Lotus".into(),
                ],
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dermotaxi — {2} 0/0 Vehicle. Imprint a creature card from a graveyard as
/// it enters; tap two untapped creatures: it becomes a copy of the imprinted
/// card until end of turn (a Vehicle artifact in addition).
pub fn dermotaxi() -> CardDefinition {
    CardDefinition {
        name: "Dermotaxi",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        triggered_abilities: vec![etb(Effect::ExileWithSource {
            what: target_filtered(R::Creature.and(R::InGraveyard)),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((R::Creature, 2)),
            effect: Effect::BecomeCopyOfFor {
                what: Selector::This,
                source: Selector::CardExiledWithSource,
                duration: Duration::EndOfTurn,
                non_legendary: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Chef's Kiss — {1}{R}{R} instant: gain control of target single-target
/// spell, copy it, and retarget both at random (never at you or yours).
pub fn chefs_kiss() -> CardDefinition {
    CardDefinition {
        name: "Chef's Kiss",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChefsKiss,
        ..Default::default()
    }
}

/// Grist, the Hunger Tide — {1}{B}{G} planeswalker (3); a 1/1 Insect creature
/// while not on the battlefield (CR 604.3).
pub fn grist_the_hunger_tide() -> CardDefinition {
    CardDefinition {
        name: "Grist, the Hunger Tide",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Planeswalker],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Grist],
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        base_loyalty: 3,
        creature_off_battlefield: true,
        loyalty_abilities: vec![
            LoyaltyAbility {
                x_cost: false,
                loyalty_cost: 1,
                effect: Effect::GristPlusOne,
            },
            LoyaltyAbility {
                x_cost: false,
                loyalty_cost: -2,
                effect: Effect::MaySacrifice {
                    description: "Sacrifice a creature to destroy?".into(),
                    filter: R::Creature,
                    count: Value::ONE,
                    then: Box::new(Effect::Reflexive {
                        body: Box::new(Effect::Destroy {
                            what: target_filtered(R::Creature.or(R::Planeswalker)),
                        }),
                    }),
                    else_: None,
                },
            },
            LoyaltyAbility {
                x_cost: false,
                loyalty_cost: -5,
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::count(Selector::EachMatching {
                        zone: crate::effect::ZoneRef::Graveyard(PlayerRef::You),
                        filter: R::Creature,
                    }),
                },
            },
        ],
        ..Default::default()
    }
}
