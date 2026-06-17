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

/// Shared "deals combat damage to a creature → tap it and it doesn't untap
/// next turn" trigger (the Orochi/Matsu snake-tribe ability).
fn snake_tap_lock() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToCreature, EventScope::SelfSource),
        effect: Effect::Seq(vec![
            Effect::Tap { what: Selector::Target(0) },
            Effect::SkipNextUntap { what: Selector::Target(0) },
        ]),
    }
}

/// Orochi Ranger — {1}{G} Snake Warrior Ranger 2/1. Combat damage to a
/// creature taps it and it doesn't untap during its controller's next untap.
pub fn orochi_ranger() -> CardDefinition {
    CardDefinition {
        name: "Orochi Ranger",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Snake, CreatureType::Warrior, CreatureType::Ranger]),
        power: 2,
        toughness: 1,
        triggered_abilities: vec![snake_tap_lock()],
        ..Default::default()
    }
}

/// Kashi-Tribe Reaver — {3}{G} Snake Warrior 3/2. The snake tap-lock; {1}{G}:
/// Regenerate this creature.
pub fn kashi_tribe_reaver() -> CardDefinition {
    CardDefinition {
        name: "Kashi-Tribe Reaver",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Snake, CreatureType::Warrior]),
        power: 3,
        toughness: 2,
        triggered_abilities: vec![snake_tap_lock()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
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

// ── Legendary Dragon Spirits (the CHK rare cycle) ────────────────────────────

/// Konda, Lord of Eiganjo — {5}{W}{W} Legendary Human Samurai 3/3. Vigilance,
/// indestructible, Bushido 5.
pub fn konda_lord_of_eiganjo() -> CardDefinition {
    CardDefinition {
        name: "Konda, Lord of Eiganjo",
        cost: cost(&[generic(5), w(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: spirit(vec![CreatureType::Human, CreatureType::Samurai]),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::Indestructible, Keyword::Bushido(5)],
        ..Default::default()
    }
}

/// Keiga, the Tide Star — {5}{U} Legendary Dragon Spirit 5/5. Flying; when it
/// dies, gain control of target creature.
pub fn keiga_the_tide_star() -> CardDefinition {
    CardDefinition {
        name: "Keiga, the Tide Star",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: spirit(vec![CreatureType::Dragon, CreatureType::Spirit]),
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::GainControl {
            what: target_filtered(SelectionRequirement::Creature),
            to: Some(PlayerRef::You),
            duration: Duration::Permanent,
        })],
        ..Default::default()
    }
}

/// Jugan, the Rising Star — {3}{G}{G}{G} Legendary Dragon Spirit 5/5. Flying;
/// when it dies, distribute five +1/+1 counters among any number of target
/// creatures.
pub fn jugan_the_rising_star() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Jugan, the Rising Star",
        cost: cost(&[generic(3), g(), g(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: spirit(vec![CreatureType::Dragon, CreatureType::Spirit]),
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::DistributeCounters {
            total: Value::Const(5),
            counter: CounterType::PlusOnePlusOne,
            filter: SelectionRequirement::Creature,
            max_targets: 5,
        })],
        ..Default::default()
    }
}

/// Ryusei, the Falling Star — {5}{R} Legendary Dragon Spirit 5/5. Flying; when
/// it dies, it deals 5 damage to each creature without flying.
pub fn ryusei_the_falling_star() -> CardDefinition {
    CardDefinition {
        name: "Ryusei, the Falling Star",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: spirit(vec![CreatureType::Dragon, CreatureType::Spirit]),
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::DealDamage {
            to: Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasKeyword(Keyword::Flying).negate()),
            ),
            amount: Value::Const(5),
        })],
        ..Default::default()
    }
}

// ── Moonfolk land-bounce utility ─────────────────────────────────────────────

/// Meloku the Clouded Mirror — {4}{U} Legendary Moonfolk Wizard 2/4. Flying;
/// {1}, Return a land you control to its owner's hand: Create a 1/1 blue
/// Illusion creature token with flying.
pub fn meloku_the_clouded_mirror() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    CardDefinition {
        name: "Meloku the Clouded Mirror",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: spirit(vec![CreatureType::Moonfolk, CreatureType::Wizard]),
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            bounce_other_filter: Some((SelectionRequirement::Land, 1)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Illusion".into(),
                    power: 1,
                    toughness: 1,
                    keywords: vec![Keyword::Flying],
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Blue],
                    subtypes: spirit(vec![CreatureType::Illusion]),
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Soratami Cloudskater — {1}{U} Moonfolk Rogue 1/1. Flying; {2}, Return a land
/// you control to its owner's hand: Draw a card, then discard a card.
pub fn soratami_cloudskater() -> CardDefinition {
    CardDefinition {
        name: "Soratami Cloudskater",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Moonfolk, CreatureType::Rogue]),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            bounce_other_filter: Some((SelectionRequirement::Land, 1)),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hana Kami — {G} Spirit 1/1. {1}{G}, Sacrifice this creature: Return target
/// Arcane card from your graveyard to your hand.
pub fn hana_kami() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Hana Kami",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            sac_cost: true,
            effect: Effect::Move {
                what: target_filtered(
                    SelectionRequirement::HasSpellSubtype(SpellSubtype::Arcane)
                        .and(SelectionRequirement::InYourGraveyard),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Kami of the Crescent Moon — {U}{U} Legendary Spirit 1/3. At the beginning of
/// each player's draw step, that player draws an additional card.
pub fn kami_of_the_crescent_moon() -> CardDefinition {
    use crate::game::TurnStep;
    CardDefinition {
        name: "Kami of the Crescent Moon",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Draw), EventScope::AnyPlayer),
            effect: Effect::Draw {
                who: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

// ── The Honden shrine cycle (upkeep triggers scaling with Shrines) ───────────

/// Number of Shrines you control — the "for each Shrine you control" rider that
/// scales the Honden cycle (CR — enchantment subtype Shrine).
fn shrines_you_control() -> Value {
    use crate::card::EnchantmentSubtype;
    Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(SelectionRequirement::ControlledByYou)),
        filter: SelectionRequirement::HasEnchantmentSubtype(EnchantmentSubtype::Shrine),
    }
}

fn shrine() -> Subtypes {
    use crate::card::EnchantmentSubtype;
    Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Shrine], ..Default::default() }
}

fn honden_upkeep(effect: Effect) -> TriggeredAbility {
    use crate::game::TurnStep;
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
        effect,
    }
}

/// Honden of Cleansing Fire — {2}{W} Legendary Enchantment — Shrine. At the
/// beginning of your upkeep, you gain 2 life for each Shrine you control.
pub fn honden_of_cleansing_fire() -> CardDefinition {
    CardDefinition {
        name: "Honden of Cleansing Fire",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![Supertype::Legendary],
        subtypes: shrine(),
        triggered_abilities: vec![honden_upkeep(Effect::GainLife {
            who: Selector::You,
            amount: Value::Times(Box::new(Value::Const(2)), Box::new(shrines_you_control())),
        })],
        ..Default::default()
    }
}

/// Honden of Life's Web — {2}{G} Legendary Enchantment — Shrine. At the
/// beginning of your upkeep, create a 1/1 colorless Spirit creature token for
/// each Shrine you control.
pub fn honden_of_lifes_web() -> CardDefinition {
    use crate::card::TokenDefinition;
    CardDefinition {
        name: "Honden of Life's Web",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![Supertype::Legendary],
        subtypes: shrine(),
        triggered_abilities: vec![honden_upkeep(Effect::CreateToken {
            who: PlayerRef::You,
            count: shrines_you_control(),
            definition: TokenDefinition {
                name: "Spirit".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                subtypes: spirit(vec![CreatureType::Spirit]),
                ..Default::default()
            },
        })],
        ..Default::default()
    }
}

/// Honden of Night's Reach — {2}{B} Legendary Enchantment — Shrine. At the
/// beginning of your upkeep, each opponent discards a card for each Shrine you
/// control.
pub fn honden_of_nights_reach() -> CardDefinition {
    CardDefinition {
        name: "Honden of Night's Reach",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![Supertype::Legendary],
        subtypes: shrine(),
        triggered_abilities: vec![honden_upkeep(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: shrines_you_control(),
            random: false,
        })],
        ..Default::default()
    }
}

/// Honden of Infinite Rage — {2}{R} Legendary Enchantment — Shrine. At the
/// beginning of your upkeep, Honden of Infinite Rage deals 2 damage to any
/// target for each Shrine you control.
pub fn honden_of_infinite_rage() -> CardDefinition {
    CardDefinition {
        name: "Honden of Infinite Rage",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![Supertype::Legendary],
        subtypes: shrine(),
        triggered_abilities: vec![honden_upkeep(Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Any),
            amount: Value::Times(Box::new(Value::Const(2)), Box::new(shrines_you_control())),
        })],
        ..Default::default()
    }
}

/// Honden of Seeing Winds — {4}{U} Legendary Enchantment — Shrine. At the
/// beginning of your upkeep, draw a card for each Shrine you control.
pub fn honden_of_seeing_winds() -> CardDefinition {
    CardDefinition {
        name: "Honden of Seeing Winds",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![Supertype::Legendary],
        subtypes: shrine(),
        triggered_abilities: vec![honden_upkeep(Effect::Draw {
            who: Selector::You,
            amount: shrines_you_control(),
        })],
        ..Default::default()
    }
}

// ── Aggro odds & ends ────────────────────────────────────────────────────────

/// Battle-Mad Ronin — {1}{R} Human Samurai 3/1. Bushido 1; attacks each combat
/// if able.
pub fn battle_mad_ronin() -> CardDefinition {
    CardDefinition {
        name: "Battle-Mad Ronin",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Human, CreatureType::Samurai]),
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Bushido(1), Keyword::MustAttack],
        ..Default::default()
    }
}

// ── Snake tribal (Seshiro / Sosuke) + red sac-pingers ────────────────────────

/// Seshiro the Anointed — {4}{G}{G} Legendary Snake Monk 3/4. Other Snakes you
/// control get +2/+2; whenever a Snake you control deals combat damage to a
/// player, you may draw a card.
pub fn seshiro_the_anointed() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Seshiro the Anointed",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: spirit(vec![CreatureType::Snake, CreatureType::Monk]),
        power: 3,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Other Snakes you control get +2/+2.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Snake)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 2,
                toughness: 2,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Snake),
                }),
            effect: Effect::MayDo {
                description: "Draw a card".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            },
        }],
        ..Default::default()
    }
}

/// Sosuke, Son of Seshiro — {2}{G}{G} Legendary Snake Warrior 3/4. Other Snakes
/// you control get +1/+0; whenever a Warrior you control deals combat damage to
/// a creature, destroy that creature. (Printed "at end of combat"; modeled as an
/// immediate destroy of the just-damaged creature.)
pub fn sosuke_son_of_seshiro() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Sosuke, Son of Seshiro",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: spirit(vec![CreatureType::Snake, CreatureType::Warrior]),
        power: 3,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Other Snakes you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Snake)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToCreature, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Warrior),
                }),
            effect: Effect::Destroy { what: Selector::Target(0) },
        }],
        ..Default::default()
    }
}

/// Kashi-Tribe Warriors — {3}{G}{G} Snake Warrior 2/4. Combat damage to a
/// creature taps it and stops its next untap.
pub fn kashi_tribe_warriors() -> CardDefinition {
    CardDefinition {
        name: "Kashi-Tribe Warriors",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Snake, CreatureType::Warrior]),
        power: 2,
        toughness: 4,
        triggered_abilities: vec![snake_tap_lock()],
        ..Default::default()
    }
}

/// Frostling — {R} Spirit 1/1. Sacrifice this creature: It deals 1 damage to
/// target creature.
pub fn frostling() -> CardDefinition {
    CardDefinition {
        name: "Frostling",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hearth Kami — {1}{R} Spirit 2/1. {X}, Sacrifice this creature: Destroy
/// target artifact with mana value X.
pub fn hearth_kami() -> CardDefinition {
    use crate::mana::x;
    CardDefinition {
        name: "Hearth Kami",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact
                        .and(SelectionRequirement::ManaValueExactlyXFromCost),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Quiet Purity — {W} Instant — Arcane. Destroy target enchantment.
pub fn quiet_purity() -> CardDefinition {
    CardDefinition {
        name: "Quiet Purity",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        subtypes: arcane(),
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::HasCardType(CardType::Enchantment)),
        },
        ..Default::default()
    }
}

/// Soratami Mirror-Guard — {3}{U} Moonfolk Wizard 3/1. Flying; {2}, Return a
/// land you control to its owner's hand: Target creature with power 2 or less
/// can't be blocked this turn.
pub fn soratami_mirror_guard() -> CardDefinition {
    CardDefinition {
        name: "Soratami Mirror-Guard",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Moonfolk, CreatureType::Wizard]),
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            bounce_other_filter: Some((SelectionRequirement::Land, 1)),
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(2)),
                ),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Akki Coalflinger — {1}{R}{R} Goblin Shaman 2/2. First strike; {R}, {T}:
/// Attacking creatures gain first strike until end of turn.
pub fn akki_coalflinger() -> CardDefinition {
    CardDefinition {
        name: "Akki Coalflinger",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Goblin, CreatureType::Shaman]),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[r()]),
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(SelectionRequirement::IsAttacking),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Yosei, the Morning Star — {4}{W}{W} Legendary Dragon Spirit 5/5. Flying;
/// when it dies, target player skips their next untap step and you tap the
/// permanents they control. (Printed "tap up to five"; modeled as tapping all
/// of that player's permanents, which the skipped untap then keeps locked.)
pub fn yosei_the_morning_star() -> CardDefinition {
    use crate::effect::Selector as Sel;
    CardDefinition {
        name: "Yosei, the Morning Star",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: spirit(vec![CreatureType::Dragon, CreatureType::Spirit]),
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::Seq(vec![
            Effect::SkipPlayerUntapStep { player: PlayerRef::Target(0) },
            Effect::Tap {
                what: Sel::ControlledBy {
                    who: PlayerRef::Target(0),
                    filter: SelectionRequirement::Permanent,
                },
            },
        ]))],
        ..Default::default()
    }
}

// ── More CHK commons/uncommons (modern_decks batch 3) ────────────────────────

/// Mothrider Patrol — {W} Fox Warrior 1/1. Flying; {3}{W}, {T}: Tap target
/// creature.
pub fn mothrider_patrol() -> CardDefinition {
    CardDefinition {
        name: "Mothrider Patrol",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Fox, CreatureType::Warrior]),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(3), w()]),
            effect: Effect::Tap { what: target_filtered(SelectionRequirement::Creature) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Strength of Cedars — {4}{G} Instant — Arcane. Target creature gets +X/+X
/// until end of turn, where X is the number of lands you control.
pub fn strength_of_cedars() -> CardDefinition {
    let lands = Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(SelectionRequirement::ControlledByYou)),
        filter: SelectionRequirement::Land,
    };
    CardDefinition {
        name: "Strength of Cedars",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Instant],
        subtypes: arcane(),
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: lands.clone(),
            toughness: lands,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Vine Kami — {6}{G} Spirit 4/4. Menace, Soulshift 6.
pub fn vine_kami() -> CardDefinition {
    CardDefinition {
        name: "Vine Kami",
        cost: cost(&[generic(6), g()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![crate::effect::shortcut::soulshift(6)],
        ..Default::default()
    }
}

/// Sokenzan Spellblade — {4}{R} Ogre Samurai Shaman 2/3. Bushido 1; {1}{R}:
/// This creature gets +X/+0 until end of turn, where X is the number of cards
/// in your hand.
pub fn sokenzan_spellblade() -> CardDefinition {
    CardDefinition {
        name: "Sokenzan Spellblade",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Ogre, CreatureType::Samurai, CreatureType::Shaman]),
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Bushido(1)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::HandSizeOf(PlayerRef::You),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wear Away — {G}{G} Instant — Arcane. Destroy target artifact or enchantment.
/// Splice onto Arcane {3}{G}.
pub fn wear_away() -> CardDefinition {
    CardDefinition {
        name: "Wear Away",
        cost: cost(&[g(), g()]),
        card_types: vec![CardType::Instant],
        subtypes: arcane(),
        keywords: vec![Keyword::Splice(cost(&[generic(3), g()]), SpellSubtype::Arcane)],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
            ),
        },
        ..Default::default()
    }
}

/// Soulless Revival — {1}{B} Instant — Arcane. Return target creature card from
/// your graveyard to your hand. Splice onto Arcane {1}{B}.
pub fn soulless_revival() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Soulless Revival",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        subtypes: arcane(),
        keywords: vec![Keyword::Splice(cost(&[generic(1), b()]), SpellSubtype::Arcane)],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}

/// Burr Grafter — {3}{G} Spirit 2/2. Sacrifice this creature: Target creature
/// gets +2/+2 until end of turn. Soulshift 3.
pub fn burr_grafter() -> CardDefinition {
    CardDefinition {
        name: "Burr Grafter",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 2,
        toughness: 2,
        triggered_abilities: vec![crate::effect::shortcut::soulshift(3)],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Crack the Earth — {R} Sorcery — Arcane. Each player sacrifices a permanent
/// of their choice.
pub fn crack_the_earth() -> CardDefinition {
    CardDefinition {
        name: "Crack the Earth",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        subtypes: arcane(),
        effect: Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachPlayer),
            count: Value::ONE,
            filter: SelectionRequirement::Permanent,
        },
        ..Default::default()
    }
}

// ── CHK batch 4 (red/goblin + threaten) ──────────────────────────────────────

/// Akki Underminer — {3}{R} Goblin Rogue Shaman 1/1. Whenever it deals combat
/// damage to a player, that player sacrifices a permanent of their choice.
pub fn akki_underminer() -> CardDefinition {
    CardDefinition {
        name: "Akki Underminer",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Goblin, CreatureType::Rogue, CreatureType::Shaman]),
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Sacrifice {
                who: Selector::Player(PlayerRef::DefendingPlayer),
                count: Value::ONE,
                filter: SelectionRequirement::Permanent,
            },
        }],
        ..Default::default()
    }
}

/// Ronin Cliffrider — {3}{R}{R} Human Samurai 2/2. Bushido 1; whenever it
/// attacks, you may have it deal 1 damage to each creature the defending player
/// controls.
pub fn ronin_cliffrider() -> CardDefinition {
    CardDefinition {
        name: "Ronin Cliffrider",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Human, CreatureType::Samurai]),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Bushido(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Deal 1 damage to each creature the defending player controls".into(),
                body: Box::new(Effect::DealDamage {
                    to: Selector::ControlledBy {
                        who: PlayerRef::DefendingPlayer,
                        filter: SelectionRequirement::Creature,
                    },
                    amount: Value::Const(1),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Akki Avalanchers — {R} Goblin Warrior 1/1. Sacrifice a land: This creature
/// gets +2/+0 until end of turn. Activate only once each turn.
pub fn akki_avalanchers() -> CardDefinition {
    CardDefinition {
        name: "Akki Avalanchers",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Goblin, CreatureType::Warrior]),
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            once_per_turn: true,
            sac_other_filter: Some((SelectionRequirement::Land, 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Frost Ogre — {3}{R}{R} Ogre Warrior 5/3 (vanilla).
pub fn frost_ogre() -> CardDefinition {
    CardDefinition {
        name: "Frost Ogre",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Ogre, CreatureType::Warrior]),
        power: 5,
        toughness: 3,
        ..Default::default()
    }
}

/// Blind with Anger — {3}{R} Instant — Arcane. Untap target nonlegendary
/// creature and gain control of it until end of turn; it gains haste.
pub fn blind_with_anger() -> CardDefinition {
    CardDefinition {
        name: "Blind with Anger",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Instant],
        subtypes: arcane(),
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasSupertype(Supertype::Legendary).negate()),
                ),
                to: Some(PlayerRef::You),
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Veteran's Reflexes — {W} Instant. Target creature gets +1/+1 until end of
/// turn. Untap that creature.
pub fn veterans_reflexes() -> CardDefinition {
    CardDefinition {
        name: "Veteran's Reflexes",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
        ]),
        ..Default::default()
    }
}

// ── CHK batch 5 (black rats & spirits) ───────────────────────────────────────

/// Nezumi Ronin — {2}{B} Rat Samurai 3/1. Bushido 1.
pub fn nezumi_ronin() -> CardDefinition {
    CardDefinition {
        name: "Nezumi Ronin",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Rat, CreatureType::Samurai]),
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Bushido(1)],
        ..Default::default()
    }
}

/// Kami of Empty Graves — {3}{B} Spirit 4/1. Soulshift 3.
pub fn kami_of_empty_graves() -> CardDefinition {
    CardDefinition {
        name: "Kami of Empty Graves",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 4,
        toughness: 1,
        triggered_abilities: vec![crate::effect::shortcut::soulshift(3)],
        ..Default::default()
    }
}

/// Scuttling Death — {4}{B} Spirit 4/2. Sacrifice this creature: Target
/// creature gets -1/-1 until end of turn. Soulshift 4.
pub fn scuttling_death() -> CardDefinition {
    CardDefinition {
        name: "Scuttling Death",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 4,
        toughness: 2,
        triggered_abilities: vec![crate::effect::shortcut::soulshift(4)],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Bile Urchin — {B} Spirit 1/1. Sacrifice this creature: Target player loses
/// 1 life.
pub fn bile_urchin() -> CardDefinition {
    CardDefinition {
        name: "Bile Urchin",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Cursed Ronin — {3}{B} Human Samurai 1/1. Bushido 1; {B}: This creature gets
/// +1/+1 until end of turn.
pub fn cursed_ronin() -> CardDefinition {
    CardDefinition {
        name: "Cursed Ronin",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Human, CreatureType::Samurai]),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Bushido(1)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Nezumi Bone-Reader — {1}{B} Rat Shaman 1/1. {B}, Sacrifice a creature:
/// Target player discards a card. Activate only as a sorcery.
pub fn nezumi_bone_reader() -> CardDefinition {
    CardDefinition {
        name: "Nezumi Bone-Reader",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Rat, CreatureType::Shaman]),
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            sorcery_speed: true,
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
                random: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Patron of the Nezumi — {5}{B}{B} Legendary Spirit 6/6. Rat offering.
/// Whenever a permanent is put into an opponent's graveyard, that player
/// loses 1 life. (Filtered to permanent-type cards; also fires for a
/// permanent card milled/discarded into an opponent's graveyard.)
pub fn patron_of_the_nezumi() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Patron of the Nezumi",
        cost: cost(&[generic(5), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 6,
        toughness: 6,
        alternative_cost: Some(offering(cost(&[generic(5), b(), b()]), CreatureType::Rat)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Permanent,
                }),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::Triggerer),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Cage of Hands — {2}{W} Aura. Enchant creature. Enchanted creature can't
/// attack or block. {1}{W}: Return this Aura to its owner's hand.
pub fn cage_of_hands() -> CardDefinition {
    use crate::card::EnchantmentSubtype;
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Cage of Hands",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(crate::card::EquipBonus {
            keywords: vec![Keyword::CantAttack, Keyword::CantBlock],
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Heartless Hidetsugu — {3}{R}{R} Legendary Ogre Shaman 4/3. {T}: Deals
/// damage to each player equal to half that player's life total, rounded down.
pub fn heartless_hidetsugu() -> CardDefinition {
    CardDefinition {
        name: "Heartless Hidetsugu",
        cost: cost(&[generic(3), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Ogre, CreatureType::Shaman]),
        power: 4,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealHalfLifeDamage {
                to: Selector::Player(PlayerRef::EachPlayer),
                rounded_up: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Horobi, Death's Wail — {2}{B}{B} Legendary Spirit 4/4, Flying. Whenever a
/// creature becomes the target of a spell or ability, destroy that creature.
pub fn horobi_deaths_wail() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Horobi, Death's Wail",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature,
                }),
            effect: Effect::Destroy { what: Selector::TriggerSource },
        }],
        ..Default::default()
    }
}

/// Time of Need — {1}{G} Sorcery. Search your library for a legendary creature
/// card, reveal it, put it into your hand, then shuffle.
pub fn time_of_need() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Time of Need",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Creature.and(SelectionRequirement::HasSupertype(
                Supertype::Legendary,
            )),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}

/// Yukora, the Prisoner — {2}{B}{B} Legendary Demon Spirit 5/5. When Yukora
/// leaves the battlefield, sacrifice all non-Ogre creatures you control.
pub fn yukora_the_prisoner() -> CardDefinition {
    CardDefinition {
        name: "Yukora, the Prisoner",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Demon, CreatureType::Spirit]),
        power: 5,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::SacrificeAllMatching {
                who: Selector::You,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(crate::card::SelectionRequirement::Not(Box::new(
                        SelectionRequirement::HasCreatureType(CreatureType::Ogre),
                    ))),
            },
        }],
        ..Default::default()
    }
}

/// He Who Hungers — {4}{B} Legendary Spirit 3/2, Flying. Soulshift 4.
/// {1}, Sacrifice a Spirit: Target opponent reveals their hand. You choose a
/// card from it; that player discards it. Activate only as a sorcery.
pub fn he_who_hungers() -> CardDefinition {
    CardDefinition {
        name: "He Who Hungers",
        cost: cost(&[generic(4), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sorcery_speed: true,
            sac_other_filter: Some((SelectionRequirement::HasCreatureType(CreatureType::Spirit), 1)),
            effect: Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::Target(0)),
                count: Value::ONE,
                filter: SelectionRequirement::Any,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![crate::effect::shortcut::soulshift(4)],
        ..Default::default()
    }
}

/// Kami of the Painted Road — {4}{W} Spirit 3/3. Whenever you cast a Spirit or
/// Arcane spell, this creature gains protection from the color of your choice
/// until end of turn.
pub fn kami_of_the_painted_road() -> CardDefinition {
    CardDefinition {
        name: "Kami of the Painted Road",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 3,
        toughness: 3,
        triggered_abilities: vec![crate::effect::shortcut::spiritcraft(
            Effect::GrantProtectionFromChosenColor {
                what: Selector::This,
                duration: Duration::EndOfTurn,
            },
        )],
        ..Default::default()
    }
}

// ── Zubera cycle: 1/2 Zubera Spirits whose death triggers scale with the ──────
// number of Zubera that died this turn (`Value::ZuberasDiedThisTurnTotal`).

fn zubera(name: &'static str, color_pip: crate::mana::ManaSymbol, dies: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(1), color_pip]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Zubera, CreatureType::Spirit]),
        power: 1,
        toughness: 2,
        triggered_abilities: vec![crate::effect::shortcut::on_dies(dies)],
        ..Default::default()
    }
}

/// Ember-Fist Zubera — {1}{R} Zubera Spirit 1/2. When it dies, it deals damage
/// to any target equal to the number of Zubera that died this turn.
pub fn ember_fist_zubera() -> CardDefinition {
    zubera("Ember-Fist Zubera", r(), Effect::DealDamage {
        to: crate::effect::shortcut::target(),
        amount: Value::ZuberasDiedThisTurnTotal,
    })
}

/// Floating-Dream Zubera — {1}{U} Zubera Spirit 1/2. When it dies, draw a card
/// for each Zubera that died this turn.
pub fn floating_dream_zubera() -> CardDefinition {
    zubera("Floating-Dream Zubera", u(), Effect::Draw {
        who: Selector::You,
        amount: Value::ZuberasDiedThisTurnTotal,
    })
}

/// Ashen-Skin Zubera — {1}{B} Zubera Spirit 1/2. When it dies, target opponent
/// discards a card for each Zubera that died this turn.
pub fn ashen_skin_zubera() -> CardDefinition {
    zubera("Ashen-Skin Zubera", b(), Effect::Discard {
        who: Selector::Player(PlayerRef::Target(0)),
        amount: Value::ZuberasDiedThisTurnTotal,
        random: false,
    })
}

/// Dripping-Tongue Zubera — {1}{G} Zubera Spirit 1/2. When it dies, create a
/// 1/1 colorless Spirit token for each Zubera that died this turn.
pub fn dripping_tongue_zubera() -> CardDefinition {
    use crate::card::TokenDefinition;
    zubera("Dripping-Tongue Zubera", g(), Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ZuberasDiedThisTurnTotal,
        definition: TokenDefinition {
            name: "Spirit".into(),
            power: 1,
            toughness: 1,
            card_types: vec![CardType::Creature],
            subtypes: spirit(vec![CreatureType::Spirit]),
            ..Default::default()
        },
    })
}

/// Silent-Chant Zubera — {1}{W} Zubera Spirit 1/2. When it dies, you gain 2
/// life for each Zubera that died this turn.
pub fn silent_chant_zubera() -> CardDefinition {
    zubera("Silent-Chant Zubera", w(), Effect::GainLife {
        who: Selector::You,
        amount: Value::Times(
            Box::new(Value::ZuberasDiedThisTurnTotal),
            Box::new(Value::Const(2)),
        ),
    })
}

/// Orochi Leafcaller — {G} Snake Shaman 1/1. {G}: Add one mana of any color.
pub fn orochi_leafcaller() -> CardDefinition {
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Orochi Leafcaller",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Snake, CreatureType::Shaman]),
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Joyous Respite — {3}{G} Sorcery — Arcane. You gain 1 life for each land
/// you control.
pub fn joyous_respite() -> CardDefinition {
    CardDefinition {
        name: "Joyous Respite",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        subtypes: arcane(),
        effect: Effect::GainLife {
            who: Selector::You,
            amount: Value::count(Selector::ControlledBy {
                who: PlayerRef::You,
                filter: SelectionRequirement::Land,
            }),
        },
        ..Default::default()
    }
}

/// Kiku, Night's Flower — {B}{B} Legendary Human Assassin 1/1. {2}{B}{B}, {T}:
/// Target creature deals damage to itself equal to its power.
pub fn kiku_nights_flower() -> CardDefinition {
    CardDefinition {
        name: "Kiku, Night's Flower",
        cost: cost(&[b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Human, CreatureType::Assassin]),
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b(), b()]),
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hanabi Blast — {1}{R}{R} Instant. Deals 2 damage to any target. Return
/// Hanabi Blast to its owner's hand, then discard a card at random.
pub fn hanabi_blast() -> CardDefinition {
    CardDefinition {
        name: "Hanabi Blast",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            crate::effect::shortcut::deal(2, crate::effect::shortcut::target()),
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: true },
            Effect::ReturnResolvingSpellToHand,
        ]),
        ..Default::default()
    }
}

/// Frostwielder — {2}{R}{R} Human Shaman 1/2. {T}: Deals 1 damage to any
/// target. A creature it deals damage to is exiled instead of dying.
pub fn frostwielder() -> CardDefinition {
    CardDefinition {
        name: "Frostwielder",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Human, CreatureType::Shaman]),
        power: 1,
        toughness: 2,
        damage_exiles_if_dies: true,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: crate::effect::shortcut::deal(1, crate::effect::shortcut::target()),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Counsel of the Soratami — {2}{U} Sorcery. Draw two cards.
pub fn counsel_of_the_soratami() -> CardDefinition {
    CardDefinition {
        name: "Counsel of the Soratami",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ..Default::default()
    }
}

/// Ghostly Visit — {2}{B} Sorcery. Destroy target nonblack creature.
pub fn ghostly_visit() -> CardDefinition {
    CardDefinition {
        name: "Ghostly Visit",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::Creature.and(
                crate::card::SelectionRequirement::Not(Box::new(SelectionRequirement::HasColor(
                    crate::mana::Color::Black,
                ))),
            )),
        },
        ..Default::default()
    }
}

/// Lifegift — {2}{G} Enchantment. Whenever a land enters, you may gain 1 life.
pub fn lifegift() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Lifegift",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Land,
                }),
            effect: Effect::MayDo {
                description: "Gain 1 life".into(),
                body: Box::new(gain_life(1)),
            },
        }],
        ..Default::default()
    }
}

/// Dampen Thought — {1}{U} Instant — Arcane. Target player mills four cards.
/// Splice onto Arcane {1}{U}.
pub fn dampen_thought() -> CardDefinition {
    CardDefinition {
        name: "Dampen Thought",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        subtypes: arcane(),
        keywords: vec![Keyword::Splice(cost(&[generic(1), u()]), SpellSubtype::Arcane)],
        effect: Effect::Mill {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Const(4),
        },
        ..Default::default()
    }
}

/// Consuming Vortex — {1}{U} Instant — Arcane. Return target creature to its
/// owner's hand. Splice onto Arcane {3}{U}.
pub fn consuming_vortex() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Consuming Vortex",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        subtypes: arcane(),
        keywords: vec![Keyword::Splice(cost(&[generic(3), u()]), SpellSubtype::Arcane)],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        },
        ..Default::default()
    }
}

/// Psychic Puppetry — {1}{U} Instant — Arcane. You may tap or untap target
/// permanent. Splice onto Arcane {U}.
pub fn psychic_puppetry() -> CardDefinition {
    CardDefinition {
        name: "Psychic Puppetry",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        subtypes: arcane(),
        keywords: vec![Keyword::Splice(cost(&[u()]), SpellSubtype::Arcane)],
        effect: Effect::ChooseMode(vec![
            Effect::Tap { what: target_filtered(SelectionRequirement::Permanent) },
            Effect::Untap { what: target_filtered(SelectionRequirement::Permanent), up_to: None },
        ]),
        ..Default::default()
    }
}

/// Pull Under — {5}{B} Instant — Arcane. Target creature gets -5/-5 until end
/// of turn.
pub fn pull_under() -> CardDefinition {
    CardDefinition {
        name: "Pull Under",
        cost: cost(&[generic(5), b()]),
        card_types: vec![CardType::Instant],
        subtypes: arcane(),
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-5),
            toughness: Value::Const(-5),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Kiku's Shadow — {B}{B} Sorcery. Target creature deals damage to itself equal
/// to its power.
pub fn kikus_shadow() -> CardDefinition {
    CardDefinition {
        name: "Kiku's Shadow",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::PowerOf(Box::new(Selector::Target(0))),
        },
        ..Default::default()
    }
}

/// Swallowing Plague — {X}{B}{B} Sorcery — Arcane. Deals X damage to target
/// creature and you gain X life.
pub fn swallowing_plague() -> CardDefinition {
    use crate::mana::x;
    CardDefinition {
        name: "Swallowing Plague",
        cost: cost(&[x(), b(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: arcane(),
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::XFromCost,
            },
            Effect::GainLife { who: Selector::You, amount: Value::XFromCost },
        ]),
        ..Default::default()
    }
}

/// Innocence Kami — {3}{W}{W} Spirit 2/3. {W}, {T}: Tap target creature.
/// Whenever you cast a Spirit or Arcane spell, untap this creature.
pub fn innocence_kami() -> CardDefinition {
    CardDefinition {
        name: "Innocence Kami",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            effect: Effect::Tap { what: target_filtered(SelectionRequirement::Creature) },
            ..Default::default()
        }],
        triggered_abilities: vec![crate::effect::shortcut::spiritcraft(Effect::Untap {
            what: Selector::This,
            up_to: None,
        })],
        ..Default::default()
    }
}

/// Villainous Ogre — {2}{B} Ogre Warrior 3/2. Can't block. (The Demon-gated
/// regeneration grant is omitted.)
pub fn villainous_ogre() -> CardDefinition {
    CardDefinition {
        name: "Villainous Ogre",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Ogre, CreatureType::Warrior]),
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::CantBlock],
        ..Default::default()
    }
}

/// Kumano, Master Yamabushi — {3}{R}{R} Legendary Human Shaman 4/4. {1}{R}:
/// Deals 1 damage to any target. Creatures it damages are exiled instead of
/// dying.
pub fn kumano_master_yamabushi() -> CardDefinition {
    CardDefinition {
        name: "Kumano, Master Yamabushi",
        cost: cost(&[generic(3), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Human, CreatureType::Shaman]),
        power: 4,
        toughness: 4,
        damage_exiles_if_dies: true,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: crate::effect::shortcut::deal(1, crate::effect::shortcut::target()),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Teardrop Kami — {U} Spirit 1/1. Sacrifice this creature: You may tap or
/// untap target creature.
pub fn teardrop_kami() -> CardDefinition {
    CardDefinition {
        name: "Teardrop Kami",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::ChooseMode(vec![
                Effect::Tap { what: target_filtered(SelectionRequirement::Creature) },
                Effect::Untap { what: target_filtered(SelectionRequirement::Creature), up_to: None },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Soratami Savant — {2}{U}{U} Moonfolk Wizard 2/2, Flying. {3}, Return a land
/// you control to its owner's hand: Counter target spell unless its controller
/// pays {3}.
pub fn soratami_savant() -> CardDefinition {
    CardDefinition {
        name: "Soratami Savant",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Moonfolk, CreatureType::Wizard]),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            bounce_other_filter: Some((SelectionRequirement::Land, 1)),
            effect: Effect::CounterUnlessPaid {
                what: crate::effect::shortcut::target(),
                mana_cost: cost(&[generic(3)]),
                exile: false,
                extra_generic: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Goblin Cohort — {R} Goblin Warrior 2/2. Can't attack unless you've cast a
/// creature spell this turn.
pub fn goblin_cohort() -> CardDefinition {
    CardDefinition {
        name: "Goblin Cohort",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Goblin, CreatureType::Warrior]),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::CantAttackUnlessCastCreatureThisTurn],
        ..Default::default()
    }
}

/// Rend Spirit — {2}{B} Instant. Destroy target Spirit.
pub fn rend_spirit() -> CardDefinition {
    CardDefinition {
        name: "Rend Spirit",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::HasCreatureType(CreatureType::Spirit)),
        },
        ..Default::default()
    }
}

/// Eye of Nowhere — {U}{U} Sorcery — Arcane. Return target permanent to its
/// owner's hand.
pub fn eye_of_nowhere() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Eye of Nowhere",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: arcane(),
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Permanent),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        },
        ..Default::default()
    }
}

/// Thief of Hope — {2}{B} Spirit 2/2. Whenever you cast a Spirit or Arcane
/// spell, target opponent loses 1 life and you gain 1 life. Soulshift 2.
pub fn thief_of_hope() -> CardDefinition {
    CardDefinition {
        name: "Thief of Hope",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            crate::effect::shortcut::spiritcraft(Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::ONE,
            }),
            crate::effect::shortcut::soulshift(2),
        ],
        ..Default::default()
    }
}

/// Soratami Rainshaper — {2}{U} Moonfolk Wizard 2/1, Flying. {3}, Return a
/// land you control to its owner's hand: Target creature you control gains
/// shroud until end of turn.
pub fn soratami_rainshaper() -> CardDefinition {
    CardDefinition {
        name: "Soratami Rainshaper",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Moonfolk, CreatureType::Wizard]),
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            bounce_other_filter: Some((SelectionRequirement::Land, 1)),
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Shroud,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mystic Restraints — {2}{U}{U} Aura with Flash. ETB taps the enchanted
/// creature; it doesn't untap during its controller's untap step.
pub fn mystic_restraints() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, StaticAbility};
    CardDefinition {
        name: "Mystic Restraints",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Tap {
            what: Selector::AttachedTo(Box::new(Selector::This)),
        })],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        ..Default::default()
    }
}

/// Hokori, Dust Drinker — {2}{W}{W} Legendary Spirit 2/2. Lands don't untap
/// during their controllers' untap steps; at each player's upkeep, that player
/// untaps a land they control.
pub fn hokori_dust_drinker() -> CardDefinition {
    use crate::card::StaticAbility;
    CardDefinition {
        name: "Hokori, Dust Drinker",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Spirit]),
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Lands don't untap during their controllers' untap steps.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::EachPermanent(SelectionRequirement::Land),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::Untap {
                what: Selector::ControlledBy {
                    who: PlayerRef::ActivePlayer,
                    filter: SelectionRequirement::Land,
                },
                up_to: Some(Value::ONE),
            },
        }],
        ..Default::default()
    }
}

/// Throat Slitter — {4}{B} Rat Ninja 2/2 with Ninjutsu {2}{B}. Whenever this
/// deals combat damage to a player, destroy target nonblack creature that
/// player controls. (Modeled as a nonblack creature an opponent controls.)
pub fn throat_slitter() -> CardDefinition {
    CardDefinition {
        name: "Throat Slitter",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: spirit(vec![CreatureType::Rat, CreatureType::Ninja]),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(2), b()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent)
                        .and(crate::card::SelectionRequirement::Not(Box::new(
                            SelectionRequirement::HasColor(crate::mana::Color::Black),
                        ))),
                ),
            },
        }],
        ..Default::default()
    }
}
