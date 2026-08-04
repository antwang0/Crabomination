//! Legends (LEG) wave 8 — the set's last nine cards. Tests in
//! `classic_sets/leg7`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, ExileCountdown, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, StaticEffect, Value, ZoneDest,
    shortcut::{target_filtered},
};
use crate::mana::{ManaCost, b, cost, g, generic, r, u, w};

fn enchantment(name: &'static str, c: ManaCost, effect: StaticEffect, text: &'static str) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility { description: text, effect }],
        ..Default::default()
    }
}

/// All Hallow's Eve — exiles itself with two scream counters; when the last
/// comes off, every graveyard empties its creatures onto the battlefield.
pub fn all_hallows_eve() -> CardDefinition {
    CardDefinition {
        name: "All Hallow's Eve",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ExileSelfWithCountdown,
        exile_countdown: Some(ExileCountdown {
            counter: CounterType::Scream,
            count: 2,
            effect: Effect::Move {
                what: Selector::CardsInZone {
                    who: PlayerRef::EachPlayer,
                    zone: crate::card::Zone::Graveyard,
                    filter: R::Creature,
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved, tapped: false },
            },
        }),
        ..Default::default()
    }
}

/// Arboria — a player who sat out their own last turn can't be attacked.
pub fn arboria() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::World],
        ..enchantment(
            "Arboria",
            cost(&[generic(2), g(), g()]),
            StaticEffect::PlayersCantBeAttackedUnlessTheyActedLastTurn,
            "Creatures can't attack a player unless that player cast a spell or put a nontoken \
             permanent onto the battlefield during their last turn.",
        )
    }
}

/// Backdraft — half of what a sorcery already burned this turn, aimed back at
/// the player who cast it.
pub fn backdraft() -> CardDefinition {
    CardDefinition {
        name: "Backdraft",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(R::Player.and(R::CastSorceryThisTurn)),
            amount: Value::HalfGreatestSorceryDamageThisTurn { who: PlayerRef::Target(0) },
        },
        ..Default::default()
    }
}

/// Chains of Mephistopheles — every draw past the draw step's first becomes
/// discard-then-draw, or a mill if the hand is empty.
pub fn chains_of_mephistopheles() -> CardDefinition {
    enchantment(
        "Chains of Mephistopheles",
        cost(&[generic(1), b()]),
        StaticEffect::ChainsOfMephistopheles,
        "If a player would draw a card except the first one they draw in each of their draw \
         steps, that player discards a card instead. If the player discards a card this way, \
         they draw a card. If the player doesn't discard a card this way, they mill a card.",
    )
}

/// Equinox — the enchanted land taps to counter a spell that would blow up one
/// of your lands.
pub fn equinox() -> CardDefinition {
    CardDefinition {
        name: "Equinox",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted land has \"{T}: Counter target spell if it would destroy a \
                          land you control.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::CounterSpell {
                        what: target_filtered(
                            R::IsSpellOnStack.and(R::SpellWouldDestroyALandYouControl),
                        ),
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..Default::default()
    }
}

/// Knowledge Vault — bank the top of your library face down, then trade your
/// hand for the whole stash. It burns if the Vault leaves any other way.
pub fn knowledge_vault() -> CardDefinition {
    CardDefinition {
        name: "Knowledge Vault",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::ExileTopOfLibrary {
                    who: Selector::You,
                    amount: Value::Const(1),
                    link_to_source: true,
                    face_down: true,
                },
                ..Default::default()
            },
            ActivatedAbility {
                sac_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(100),
                        random: false,
                    },
                    Effect::Move {
                        what: Selector::CardExiledWithSource,
                        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                    },
                ]),
                ..Default::default()
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::CardExiledWithSource,
                to: ZoneDest::Graveyard,
            },
        }],
        ..Default::default()
    }
}

/// Land Equilibrium — an opponent who's caught up on lands pays for each new
/// one with an old one.
pub fn land_equilibrium() -> CardDefinition {
    enchantment(
        "Land Equilibrium",
        cost(&[generic(2), u(), u()]),
        StaticEffect::OpponentLandsEnterThenSacrifice,
        "If an opponent who controls at least as many lands as you do would put a land onto the \
         battlefield, that player instead puts that land onto the battlefield then sacrifices a \
         land of their choice.",
    )
}

/// Reverberation — the targeted sorcery spends the rest of the turn burning
/// its own caster.
pub fn reverberation() -> CardDefinition {
    CardDefinition {
        name: "Reverberation",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::RedirectSpellDamageToItsController {
            what: target_filtered(R::IsSpellOnStack.and(R::HasCardType(CardType::Sorcery))),
        },
        ..Default::default()
    }
}

/// Wall of Caltrops — bands with the other Walls gang-blocking one attacker.
pub fn wall_of_caltrops() -> CardDefinition {
    CardDefinition {
        name: "Wall of Caltrops",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wall],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource).with_filter(
                Predicate::SourceCoBlockersAllMatch {
                    filter: R::HasCreatureType(CreatureType::Wall),
                },
            ),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Banding,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}
