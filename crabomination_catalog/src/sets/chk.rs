//! Champions of Kamigawa (CHK) — Splice onto Arcane (CR 702.47), Offering
//! (CR 702.48 — the Patron cycle), plus the Legends-era World permanents
//! exercising the world rule (CR 704.5k).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, LandType, SelectionRequirement, Selector, SpellSubtype, StaticAbility, Subtypes,
    Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{gain_life, offering, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, StaticEffect};
use crate::mana::{b, cost, g, generic, r, u, w};

fn spirit(types: Vec<CreatureType>) -> Subtypes {
    Subtypes { creature_types: types, ..Default::default() }
}

fn arcane() -> Subtypes {
    Subtypes { spell_subtypes: vec![SpellSubtype::Arcane], ..Default::default() }
}

/// Glacial Ray — {1}{R} Instant — Arcane. Deals 2 damage to any target.
/// Splice onto Arcane {1}{R}.
pub fn glacial_ray() -> CardDefinition {
    CardDefinition {
        name: "Glacial Ray",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        subtypes: arcane(),
        keywords: vec![Keyword::Splice(cost(&[generic(1), r()]), SpellSubtype::Arcane)],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Any),
            amount: Value::Const(2),
        },
        ..Default::default()
    }
}

/// Reach Through Mists — {U} Instant — Arcane. Draw a card.
pub fn reach_through_mists() -> CardDefinition {
    CardDefinition {
        name: "Reach Through Mists",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        subtypes: arcane(),
        effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        ..Default::default()
    }
}

/// Kodama's Might — {G} Instant — Arcane. Target creature gets +2/+2 until
/// end of turn. Splice onto Arcane {G}.
pub fn kodamas_might() -> CardDefinition {
    CardDefinition {
        name: "Kodama's Might",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        subtypes: arcane(),
        keywords: vec![Keyword::Splice(cost(&[g()]), SpellSubtype::Arcane)],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Concordant Crossroads — {G} World Enchantment. All creatures have haste.
pub fn concordant_crossroads() -> CardDefinition {
    CardDefinition {
        name: "Concordant Crossroads",
        cost: cost(&[g()]),
        supertypes: vec![Supertype::World],
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "All creatures have haste",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(SelectionRequirement::Creature),
                keyword: Keyword::Haste,
            },
        }],
        ..Default::default()
    }
}

/// Nether Void — {3}{B} World Enchantment. Whenever a player casts a spell,
/// counter it unless that player pays {3}.
pub fn nether_void() -> CardDefinition {
    use crate::mana::b;
    CardDefinition {
        name: "Nether Void",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![Supertype::World],
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
            effect: Effect::CounterUnlessPaid {
                what: Selector::TriggerSource,
                mana_cost: cost(&[generic(3)]),
                exile: false,
                extra_generic: None,
            },
        }],
        ..Default::default()
    }
}

/// Patron of the Kitsune — {4}{W}{W} Legendary Spirit 5/6. Fox offering.
/// Whenever a creature attacks, you may gain 1 life.
pub fn patron_of_the_kitsune() -> CardDefinition {
    CardDefinition {
        name: "Patron of the Kitsune",
        cost: cost(&[generic(4), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 5,
        toughness: 6,
        alternative_cost: Some(offering(cost(&[generic(4), w(), w()]), CreatureType::Fox)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer),
            effect: gain_life(1),
        }],
        ..Default::default()
    }
}

/// Patron of the Akki — {4}{R}{R} Legendary Spirit 5/5. Goblin offering.
/// Whenever this attacks, creatures you control get +2/+0 until end of turn.
pub fn patron_of_the_akki() -> CardDefinition {
    CardDefinition {
        name: "Patron of the Akki",
        cost: cost(&[generic(4), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 5,
        toughness: 5,
        alternative_cost: Some(offering(cost(&[generic(4), r(), r()]), CreatureType::Goblin)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Patron of the Moon — {5}{U}{U} Legendary Spirit 5/4, Flying. Moonfolk
/// offering. {1}: Put up to two land cards from your hand onto the
/// battlefield tapped.
pub fn patron_of_the_moon() -> CardDefinition {
    CardDefinition {
        name: "Patron of the Moon",
        cost: cost(&[generic(5), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        alternative_cost: Some(offering(cost(&[generic(5), u(), u()]), CreatureType::Moonfolk)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: SelectionRequirement::Land,
                count: Value::Const(2),
                tapped: true,
                haste: false,
                sacrifice_eot: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Patron of the Orochi — {6}{G}{G} Legendary Spirit 7/7. Snake offering.
/// {T}: Untap all Forests and all green creatures. Activate only once each turn.
pub fn patron_of_the_orochi() -> CardDefinition {
    CardDefinition {
        name: "Patron of the Orochi",
        cost: cost(&[generic(6), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 7,
        toughness: 7,
        alternative_cost: Some(offering(cost(&[generic(6), g(), g()]), CreatureType::Snake)),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            once_per_turn: true,
            effect: Effect::Untap {
                what: Selector::EachPermanent(
                    SelectionRequirement::HasLandType(LandType::Forest).or(
                        SelectionRequirement::Creature.and(SelectionRequirement::HasColor(
                            crate::mana::Color::Green,
                        )),
                    ),
                ),
                up_to: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Kodama of the North Tree — {2}{G}{G}{G} Legendary Spirit 6/4. Trample,
/// Shroud.
pub fn kodama_of_the_north_tree() -> CardDefinition {
    CardDefinition {
        name: "Kodama of the North Tree",
        cost: cost(&[generic(2), g(), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 6,
        toughness: 4,
        keywords: vec![Keyword::Trample, Keyword::Shroud],
        ..Default::default()
    }
}

/// Kami of Ancient Law — {1}{W} Spirit 2/2. Sacrifice this creature: Destroy
/// target enchantment.
pub fn kami_of_ancient_law() -> CardDefinition {
    CardDefinition {
        name: "Kami of Ancient Law",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(SelectionRequirement::HasCardType(CardType::Enchantment)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Kitsune Blademaster — {2}{W} Fox Samurai 2/2. First strike, Bushido 1.
pub fn kitsune_blademaster() -> CardDefinition {
    CardDefinition {
        name: "Kitsune Blademaster",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Fox, CreatureType::Samurai]),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike, Keyword::Bushido(1)],
        ..Default::default()
    }
}

/// Gibbering Kami — {3}{B} Spirit 2/2. Flying, Soulshift 3.
pub fn gibbering_kami() -> CardDefinition {
    CardDefinition {
        name: "Gibbering Kami",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::soulshift(3)],
        ..Default::default()
    }
}

/// Nezumi Cutthroat — {1}{B} Rat Warrior 2/1. Fear; can't block.
pub fn nezumi_cutthroat() -> CardDefinition {
    CardDefinition {
        name: "Nezumi Cutthroat",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Rat, CreatureType::Warrior]),
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Fear, Keyword::CantBlock],
        ..Default::default()
    }
}

/// Kabuto Moth — {2}{W} Spirit 1/2. Flying; {T}: target creature gets +1/+2
/// until end of turn.
pub fn kabuto_moth() -> CardDefinition {
    CardDefinition {
        name: "Kabuto Moth",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Kami of Fire's Roar — {3}{R} Spirit 2/3. Spiritcraft: whenever you cast a
/// Spirit or Arcane spell, target creature can't block this turn.
pub fn kami_of_fires_roar() -> CardDefinition {
    CardDefinition {
        name: "Kami of Fire's Roar",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 2,
        toughness: 3,
        triggered_abilities: vec![crate::effect::shortcut::spiritcraft(Effect::GrantKeyword {
            what: target_filtered(SelectionRequirement::Creature),
            keyword: Keyword::CantBlock,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Teller of Tales — {3}{U}{U} Spirit 3/3, Flying. Spiritcraft: whenever you
/// cast a Spirit or Arcane spell, tap or untap target creature.
pub fn teller_of_tales() -> CardDefinition {
    CardDefinition {
        name: "Teller of Tales",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::spiritcraft(Effect::ChooseMode(vec![
            Effect::Tap { what: target_filtered(SelectionRequirement::Creature) },
            Effect::Untap {
                what: target_filtered(SelectionRequirement::Creature),
                up_to: None,
            },
        ]))],
        ..Default::default()
    }
}

/// Sokenzan Bruiser — {4}{R} Ogre Warrior 3/3. Mountainwalk.
pub fn sokenzan_bruiser() -> CardDefinition {
    CardDefinition {
        name: "Sokenzan Bruiser",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Ogre, CreatureType::Warrior]),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Landwalk(LandType::Mountain)],
        ..Default::default()
    }
}

/// Numai Outcast — {3}{B} Human Samurai 1/1. Bushido 2; {B}, Pay 5 life:
/// Regenerate this creature.
pub fn numai_outcast() -> CardDefinition {
    CardDefinition {
        name: "Numai Outcast",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Human, CreatureType::Samurai]),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Bushido(2)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            life_cost: 5,
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Befoul — {2}{B}{B} Sorcery. Destroy target land or nonblack creature.
pub fn befoul() -> CardDefinition {
    CardDefinition {
        name: "Befoul",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::Land.or(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasColor(crate::mana::Color::Black).negate()),
            )),
        },
        ..Default::default()
    }
}

/// Rend Flesh — {2}{B} Instant — Arcane. Destroy target non-Spirit creature.
pub fn rend_flesh() -> CardDefinition {
    CardDefinition {
        name: "Rend Flesh",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        subtypes: arcane(),
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit).negate()),
            ),
        },
        ..Default::default()
    }
}

/// Yamabushi's Flame — {2}{R} Instant. 3 damage to any target; if a creature
/// dealt damage this way would die this turn, exile it instead.
pub fn yamabushis_flame() -> CardDefinition {
    use crate::effect::shortcut::{deal, target};
    CardDefinition {
        name: "Yamabushi's Flame",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn { what: Selector::Target(0) },
            deal(3, target()),
        ]),
        ..Default::default()
    }
}

/// Moss Kami — {5}{G} Spirit 5/5. Trample.
pub fn moss_kami() -> CardDefinition {
    CardDefinition {
        name: "Moss Kami",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        ..Default::default()
    }
}

/// Order of the Sacred Bell — {3}{G} Human Monk 4/3 (vanilla).
pub fn order_of_the_sacred_bell() -> CardDefinition {
    CardDefinition {
        name: "Order of the Sacred Bell",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Human, CreatureType::Monk]),
        power: 4,
        toughness: 3,
        ..Default::default()
    }
}

/// River Kaijin — {2}{U} Spirit 1/4 (vanilla wall-body).
pub fn river_kaijin() -> CardDefinition {
    CardDefinition {
        name: "River Kaijin",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 1,
        toughness: 4,
        ..Default::default()
    }
}

/// Kami of Twisted Reflection — {1}{U}{U} Spirit 2/2. Sacrifice this creature:
/// Return target creature you control to its owner's hand.
pub fn kami_of_twisted_reflection() -> CardDefinition {
    use crate::effect::{PlayerRef, ZoneDest};
    CardDefinition {
        name: "Kami of Twisted Reflection",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Kami of the Hunt — {2}{G} Spirit 2/2. Spiritcraft: gets +1/+1 until end of
/// turn.
pub fn kami_of_the_hunt() -> CardDefinition {
    CardDefinition {
        name: "Kami of the Hunt",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 2,
        toughness: 2,
        triggered_abilities: vec![crate::effect::shortcut::spiritcraft(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Soilshaper — {1}{G} Spirit 1/1. Spiritcraft: target land becomes a 3/3
/// creature until end of turn (it's still a land).
pub fn soilshaper() -> CardDefinition {
    CardDefinition {
        name: "Soilshaper",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 1,
        toughness: 1,
        triggered_abilities: vec![crate::effect::shortcut::spiritcraft(Effect::BecomeCreature {
            what: target_filtered(SelectionRequirement::Land),
            power: Value::Const(3),
            toughness: Value::Const(3),
            creature_types: vec![],
            keywords: vec![],
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Devoted Retainer — {W} Human Samurai 1/1. Bushido 1.
pub fn devoted_retainer() -> CardDefinition {
    CardDefinition {
        name: "Devoted Retainer",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Human, CreatureType::Samurai]),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Bushido(1)],
        ..Default::default()
    }
}

/// Ronin Houndmaster — {2}{R} Human Samurai 2/2. Haste, Bushido 1.
pub fn ronin_houndmaster() -> CardDefinition {
    CardDefinition {
        name: "Ronin Houndmaster",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Human, CreatureType::Samurai]),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste, Keyword::Bushido(1)],
        ..Default::default()
    }
}

/// Mothrider Samurai — {3}{W} Human Samurai 2/2. Flying, Bushido 1.
pub fn mothrider_samurai() -> CardDefinition {
    CardDefinition {
        name: "Mothrider Samurai",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Human, CreatureType::Samurai]),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Bushido(1)],
        ..Default::default()
    }
}

/// Pain Kami — {2}{R} Spirit 2/2. {X}{R}, Sacrifice this creature: It deals X
/// damage to target creature.
pub fn pain_kami() -> CardDefinition {
    use crate::mana::x;
    CardDefinition {
        name: "Pain Kami",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), r()]),
            sac_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::XFromCost,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Kami of Old Stone — {3}{W} Spirit 1/7 (vanilla wall-body).
pub fn kami_of_old_stone() -> CardDefinition {
    CardDefinition {
        name: "Kami of Old Stone",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 1,
        toughness: 7,
        ..Default::default()
    }
}

/// Lantern Kami — {W} Spirit 1/1, Flying.
pub fn lantern_kami() -> CardDefinition {
    CardDefinition {
        name: "Lantern Kami",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Wandering Ones — {U} Spirit 1/1 (vanilla).
pub fn wandering_ones() -> CardDefinition {
    CardDefinition {
        name: "Wandering Ones",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 1,
        toughness: 1,
        ..Default::default()
    }
}

/// Kami of Lunacy — {4}{B}{B} Spirit 4/1, Flying. Soulshift 5.
pub fn kami_of_lunacy() -> CardDefinition {
    CardDefinition {
        name: "Kami of Lunacy",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 4,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::soulshift(5)],
        ..Default::default()
    }
}

/// Venerable Kumo — {4}{G} Spirit 2/3. Reach, Soulshift 4.
pub fn venerable_kumo() -> CardDefinition {
    CardDefinition {
        name: "Venerable Kumo",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![crate::effect::shortcut::soulshift(4)],
        ..Default::default()
    }
}

/// Kitsune Diviner — {W} Fox Cleric 0/1. {T}: Tap target Spirit.
pub fn kitsune_diviner() -> CardDefinition {
    CardDefinition {
        name: "Kitsune Diviner",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Fox, CreatureType::Cleric]),
        power: 0,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Tap {
                what: target_filtered(SelectionRequirement::HasCreatureType(CreatureType::Spirit)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sire of the Storm — {4}{U}{U} Spirit 3/3, Flying. Spiritcraft: you may
/// draw a card.
pub fn sire_of_the_storm() -> CardDefinition {
    CardDefinition {
        name: "Sire of the Storm",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::spiritcraft(Effect::Draw {
            who: Selector::You,
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Nagao, Bound by Honor — {3}{W} Legendary Human Samurai 3/3. Bushido 1;
/// whenever Nagao attacks, Samurai creatures you control get +1/+1 until EOT.
pub fn nagao_bound_by_honor() -> CardDefinition {
    CardDefinition {
        name: "Nagao, Bound by Honor",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Human, CreatureType::Samurai]),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Bushido(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Samurai)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Kami of the Waning Moon — {2}{B} Spirit 1/1, Flying. Spiritcraft: target
/// creature gains fear until end of turn.
pub fn kami_of_the_waning_moon() -> CardDefinition {
    CardDefinition {
        name: "Kami of the Waning Moon",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::spiritcraft(Effect::GrantKeyword {
            what: target_filtered(SelectionRequirement::Creature),
            keyword: Keyword::Fear,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Hideous Laughter — {2}{B}{B} Instant — Arcane. All creatures get -2/-2 until
/// end of turn. Splice onto Arcane {3}{B}{B}.
pub fn hideous_laughter() -> CardDefinition {
    CardDefinition {
        name: "Hideous Laughter",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: arcane(),
        keywords: vec![Keyword::Splice(cost(&[generic(3), b(), b()]), SpellSubtype::Arcane)],
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(SelectionRequirement::Creature),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Yamabushi's Storm — {1}{R} Sorcery. 1 damage to each creature; creatures it
/// would kill are exiled instead.
pub fn yamabushis_storm() -> CardDefinition {
    CardDefinition {
        name: "Yamabushi's Storm",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn {
                what: Selector::EachPermanent(SelectionRequirement::Creature),
            },
            Effect::DealDamage {
                to: Selector::EachPermanent(SelectionRequirement::Creature),
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

/// Vigilance — {W} Aura. Enchanted creature has vigilance.
pub fn vigilance_aura() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, EquipBonus};
    CardDefinition {
        name: "Vigilance",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Vigilance],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Eiganjo Castle — {T}: Add {W}. {2}{W}: Prevent the next 2 damage to a
/// legendary creature this turn. (Mana half only — the prevention targets a
/// legendary creature.)
pub fn eiganjo_castle() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::mana::w;
    CardDefinition {
        name: "Eiganjo Castle",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add(crate::mana::Color::White),
            ActivatedAbility {
                mana_cost: cost(&[generic(2), w()]),
                effect: Effect::PreventNextDamage {
                    target: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::HasSupertype(Supertype::Legendary)),
                    ),
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
