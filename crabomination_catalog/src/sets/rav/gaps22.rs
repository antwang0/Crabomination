//! Ravnica (RAV) gap wave 22 — the last of the block's rares that needed real
//! machinery. Tests in `classic_sets/rav`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, MayPlayDuration,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector, StaticEffect,
    TriggeredAbility,
};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

fn types(t: Vec<CreatureType>) -> Subtypes {
    Subtypes {
        creature_types: t,
        ..Default::default()
    }
}

/// Cloudstone Curio — {3} Artifact. Whenever a nonartifact permanent you
/// control enters, you may bounce another permanent sharing a type with it.
pub fn cloudstone_curio() -> CardDefinition {
    CardDefinition {
        name: "Cloudstone Curio",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact.negate(),
                }),
            effect: Effect::MayReturnSharingPermanentType {
                with: Selector::TriggerSource,
            },
        }],
        ..Default::default()
    }
}

/// Circu, Dimir Lobotomist — {2}{U}{B} 2/3. Blue and black casts strip the top
/// of target player's library; opponents can't cast those names.
pub fn circu_dimir_lobotomist() -> CardDefinition {
    let strip = |color: R| TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
            Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: color,
            },
        ),
        effect: Effect::ExileTopOfLibrary {
            who: target_filtered(R::Player),
            amount: Value::ONE,
            link_to_source: true,
            face_down: false,
        },
    };
    CardDefinition {
        name: "Circu, Dimir Lobotomist",
        cost: cost(&[generic(2), u(), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: types(vec![CreatureType::Human, CreatureType::Wizard]),
        power: 2,
        toughness: 3,
        triggered_abilities: vec![strip(R::HasColor(Color::Blue)), strip(R::HasColor(Color::Black))],
        static_abilities: vec![StaticAbility {
            description: "Opponents can't cast spells named like a card exiled with this.",
            effect: StaticEffect::OpponentsCantCastNamesExiledWithSource,
        }],
        ..Default::default()
    }
}

/// Sins of the Past — {4}{B}{B} Sorcery. Until end of turn you may cast target
/// instant or sorcery card from your graveyard for free; it exiles instead of
/// returning, and so does this.
pub fn sins_of_the_past() -> CardDefinition {
    CardDefinition {
        name: "Sins of the Past",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Sorcery],
        exile_on_resolve: true,
        effect: Effect::GrantMayPlay {
            what: target_filtered(
                R::HasCardType(CardType::Instant)
                    .or(R::HasCardType(CardType::Sorcery))
                    .and(R::InGraveyard),
            ),
            duration: MayPlayDuration::EndOfThisTurn,
            to_owner: false,
            exile_after: true,
            pay_own_cost: false,
            any_color: false,
        },
        ..Default::default()
    }
}

/// Mindleech Mass — {5}{U}{B}{B} 6/6 trample. Combat damage lets you raid the
/// defender's hand for a free spell.
pub fn mindleech_mass() -> CardDefinition {
    CardDefinition {
        name: "Mindleech Mass",
        cost: cost(&[generic(5), u(), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Horror]),
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::LookAtHandCastFree {
                who: Selector::Player(PlayerRef::DefendingPlayer),
            },
        }],
        ..Default::default()
    }
}

/// Reroute — {1}{R} Instant. Change the target of target activated ability
/// with a single target, then draw a card.
pub fn reroute() -> CardDefinition {
    CardDefinition {
        name: "Reroute",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ChangeTargetOfAbility {
                what: target_filtered(R::HasAbilityOnStack),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Warp World — {5}{R}{R}{R} Sorcery. Everyone shuffles their permanents away
/// and redeploys whatever the top of their library gives back.
pub fn warp_world() -> CardDefinition {
    CardDefinition {
        name: "Warp World",
        cost: cost(&[generic(5), r(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::WarpWorld,
        ..Default::default()
    }
}

/// Chorus of the Conclave — {4}{G}{G}{W}{W} 3/8 forestwalk. Your creature
/// spells may buy extra +1/+1 counters with extra mana.
pub fn chorus_of_the_conclave() -> CardDefinition {
    CardDefinition {
        name: "Chorus of the Conclave",
        cost: cost(&[generic(4), g(), g(), w(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: types(vec![CreatureType::Dryad]),
        power: 3,
        toughness: 8,
        keywords: vec![Keyword::Landwalk(crate::card::LandType::Forest)],
        static_abilities: vec![StaticAbility {
            description: "Creature spells may pay extra mana for that many +1/+1 counters.",
            effect: StaticEffect::CreatureSpellsMayPayExtraForCounters,
        }],
        ..Default::default()
    }
}

/// Flickerform — {1}{W} Aura. {2}{W}{W}: blink the enchanted creature and
/// every Aura on it, all of it coming back at the next end step.
pub fn flickerform() -> CardDefinition {
    CardDefinition {
        name: "Flickerform",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        activated_abilities: vec![crate::card::ActivatedAbility {
            mana_cost: cost(&[generic(2), w(), w()]),
            effect: Effect::FlickerHostWithAuras,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Breath of Fury — {2}{R}{R} Aura on a creature you control. Its combat
/// damage to a player buys another combat phase, at the cost of the creature.
pub fn breath_of_fury() -> CardDefinition {
    CardDefinition {
        name: "Breath of Fury",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::EnchantedBySource,
            ),
            effect: Effect::SacrificeEnchantedForExtraCombat,
        }],
        ..Default::default()
    }
}

/// Sunforger — {3} Equipment. Equipped creature gets +4/+0; unattach it to
/// fetch and cast a cheap red or white instant for free.
pub fn sunforger() -> CardDefinition {
    CardDefinition {
        name: "Sunforger",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 4,
            ..Default::default()
        }),
        activated_abilities: vec![crate::card::ActivatedAbility {
            mana_cost: cost(&[r(), w()]),
            unattach_cost: true,
            effect: Effect::SearchAndCastFree {
                filter: R::HasCardType(CardType::Instant)
                    .and(R::HasColor(Color::Red).or(R::HasColor(Color::White)))
                    .and(R::ManaValueAtMost(4)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Eye of the Storm — {5}{U}{U} Enchantment. Every instant and sorcery cast is
/// exiled under it, and its owner replays free copies of the whole pile.
pub fn eye_of_the_storm() -> CardDefinition {
    CardDefinition {
        name: "Eye of the Storm",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCardType(CardType::Instant)
                        .or(R::HasCardType(CardType::Sorcery)),
                },
            ),
            effect: Effect::EyeOfTheStorm {
                what: Selector::TriggerSource,
            },
        }],
        ..Default::default()
    }
}

/// Master Warcraft — {2}{R/W}{R/W} Instant. Cast before attackers; you choose
/// this turn's attackers and blocks instead of the players who normally do.
pub fn master_warcraft() -> CardDefinition {
    CardDefinition {
        name: "Master Warcraft",
        cost: cost(&[
            generic(2),
            crate::mana::hybrid(Color::Red, Color::White),
            crate::mana::hybrid(Color::Red, Color::White),
        ]),
        card_types: vec![CardType::Instant],
        cast_only_before_attackers: true,
        effect: Effect::ChooseCombatThisTurn,
        ..Default::default()
    }
}
