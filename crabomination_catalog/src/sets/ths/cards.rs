//! Theros (THS) — assorted commons/uncommons used as devotion-shell
//! filler. Simple bodies / ETBs / one instant.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, Keyword,
    SelectionRequirement, Selector, Subtypes, Value,
};
use crate::effect::{PlayerRef, ZoneDest, shortcut::etb, shortcut::target_filtered};
use crate::mana::{b, cost, g, generic, r, u, w};

/// Sedge Scorpion — {G} Creature — Scorpion 1/1. Deathtouch.
pub fn sedge_scorpion() -> CardDefinition {
    CardDefinition {
        name: "Sedge Scorpion",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Scorpion], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        ..Default::default()
    }
}

/// Pharika's Chosen — {B} Creature — Snake 1/1. Deathtouch.
pub fn pharikas_chosen() -> CardDefinition {
    CardDefinition {
        name: "Pharika's Chosen",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Snake], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        ..Default::default()
    }
}

/// Yoked Ox — {W} Creature — Ox 0/4.
pub fn yoked_ox() -> CardDefinition {
    CardDefinition {
        name: "Yoked Ox",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ox], ..Default::default() },
        power: 0,
        toughness: 4,
        ..Default::default()
    }
}

/// Two-Headed Cerberus — {2}{R} Creature — Dog 2/2. Double strike.
pub fn two_headed_cerberus() -> CardDefinition {
    CardDefinition {
        name: "Two-Headed Cerberus",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dog], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::DoubleStrike],
        ..Default::default()
    }
}

/// Voyaging Satyr — {1}{G} Creature — Satyr Druid 1/2. {T}: Untap target land.
pub fn voyaging_satyr() -> CardDefinition {
    CardDefinition {
        name: "Voyaging Satyr",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Satyr, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            tap_cost: true,
            effect: Effect::Untap { what: target_filtered(SelectionRequirement::Land), up_to: None },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Leonin Snarecaster — {1}{W} Creature — Cat Soldier 2/1. When it enters,
/// you may tap target creature. (The "may" is taken — collapsed to a tap.)
pub fn leonin_snarecaster() -> CardDefinition {
    CardDefinition {
        name: "Leonin Snarecaster",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Tap {
            what: target_filtered(SelectionRequirement::Creature),
        })],
        ..Default::default()
    }
}

/// Voyage's End — {1}{U} Instant. Return target creature to its owner's
/// hand. Scry 1.
pub fn voyages_end() -> CardDefinition {
    CardDefinition {
        name: "Voyage's End",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Nessian Courser — {2}{G} Creature — Centaur Warrior 3/3.
pub fn nessian_courser() -> CardDefinition {
    CardDefinition {
        name: "Nessian Courser",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Centaur, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        ..Default::default()
    }
}

/// Vulpine Goliath — {5}{G} Creature — Fox 4/4. Trample.
pub fn vulpine_goliath() -> CardDefinition {
    CardDefinition {
        name: "Vulpine Goliath",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Fox], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        ..Default::default()
    }
}

/// Felhide Minotaur — {2}{R} Creature — Minotaur 3/2.
pub fn felhide_minotaur() -> CardDefinition {
    CardDefinition {
        name: "Felhide Minotaur",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Minotaur], ..Default::default() },
        power: 3,
        toughness: 2,
        ..Default::default()
    }
}

/// Griptide — {2}{U} Instant. Put target creature on top of its owner's
/// library.
pub fn griptide() -> CardDefinition {
    use crate::effect::LibraryPosition;
    CardDefinition {
        name: "Griptide",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: LibraryPosition::Top,
            },
        },
        ..Default::default()
    }
}

/// Lash of the Whip — {4}{B} Instant. Target creature gets -4/-4 until end
/// of turn.
pub fn lash_of_the_whip() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Lash of the Whip",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-4),
            toughness: Value::Const(-4),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Pharika's Cure — {1}{B} Instant. Deal 2 damage to target creature and
/// you gain 2 life.
pub fn pharikas_cure() -> CardDefinition {
    CardDefinition {
        name: "Pharika's Cure",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(2),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Fade into Antiquity — {2}{G} Sorcery. Exile target enchantment.
pub fn fade_into_antiquity() -> CardDefinition {
    CardDefinition {
        name: "Fade into Antiquity",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Exile { what: target_filtered(SelectionRequirement::Enchantment) },
        ..Default::default()
    }
}

/// Nylea's Disciple — {3}{G} Creature — Centaur Shaman 2/3. ETB: you gain
/// life equal to your devotion to green (CR 700.5).
pub fn nyleas_disciple() -> CardDefinition {
    CardDefinition {
        name: "Nylea's Disciple",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Centaur, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::DevotionTo(vec![crate::mana::Color::Green]),
        })],
        ..Default::default()
    }
}

/// Traveling Philosopher — {2}{W} Creature — Human Advisor 1/4.
pub fn traveling_philosopher() -> CardDefinition {
    CardDefinition {
        name: "Traveling Philosopher",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Advisor],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        ..Default::default()
    }
}

/// Cavalry Pegasus — {1}{W} Creature — Pegasus 1/1. Flying. (The "Humans
/// you control gain flying when it attacks" rider is omitted.)
pub fn cavalry_pegasus() -> CardDefinition {
    CardDefinition {
        name: "Cavalry Pegasus",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Pegasus], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Mnemonic Wall — {4}{U} Creature — Wall 0/4. Defender. ETB: return target
/// instant or sorcery card from your graveyard to your hand.
pub fn mnemonic_wall() -> CardDefinition {
    use crate::card::CardType as Ct;
    CardDefinition {
        name: "Mnemonic Wall",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wall], ..Default::default() },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                SelectionRequirement::InGraveyard.and(
                    SelectionRequirement::HasCardType(Ct::Instant)
                        .or(SelectionRequirement::HasCardType(Ct::Sorcery)),
                ),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Horizon Scholar — {4}{U} Creature — Sphinx 4/4. Flying. ETB: scry 2.
pub fn horizon_scholar() -> CardDefinition {
    CardDefinition {
        name: "Horizon Scholar",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Sphinx], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) })],
        ..Default::default()
    }
}

/// Anvilwrought Raptor — {4} Artifact Creature — Bird 2/2. Flying, first strike.
pub fn anvilwrought_raptor() -> CardDefinition {
    CardDefinition {
        name: "Anvilwrought Raptor",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        ..Default::default()
    }
}

/// Bronze Sable — {2} Artifact Creature — Construct 2/1.
pub fn bronze_sable() -> CardDefinition {
    CardDefinition {
        name: "Bronze Sable",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 2,
        toughness: 1,
        ..Default::default()
    }
}

/// Guardians of Meletis — {3} Artifact Creature — Golem 0/6. Defender.
pub fn guardians_of_meletis() -> CardDefinition {
    CardDefinition {
        name: "Guardians of Meletis",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: 0,
        toughness: 6,
        keywords: vec![Keyword::Defender],
        ..Default::default()
    }
}

/// Opaline Unicorn — {3} Artifact Creature — Unicorn 2/2. {T}: Add one mana
/// of any color.
pub fn opaline_unicorn() -> CardDefinition {
    CardDefinition {
        name: "Opaline Unicorn",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Unicorn], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![crate::sets::tap_add_any_color()],
        ..Default::default()
    }
}

/// Borderland Minotaur — {3}{R} Creature — Minotaur Warrior 3/3.
pub fn borderland_minotaur() -> CardDefinition {
    CardDefinition {
        name: "Borderland Minotaur",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        ..Default::default()
    }
}

/// Deathbellow Raider — {1}{R} Creature — Minotaur Berserker 3/1.
pub fn deathbellow_raider() -> CardDefinition {
    CardDefinition {
        name: "Deathbellow Raider",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Berserker],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        ..Default::default()
    }
}

/// Asphodel Wanderer — {1}{B} Creature — Zombie 1/1.
pub fn asphodel_wanderer() -> CardDefinition {
    CardDefinition {
        name: "Asphodel Wanderer",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 1,
        toughness: 1,
        ..Default::default()
    }
}

/// Returned Centaur — {3}{B} Creature — Zombie Centaur 3/3. ETB: put the
/// top four cards of your library into your graveyard.
pub fn returned_centaur() -> CardDefinition {
    CardDefinition {
        name: "Returned Centaur",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Centaur],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Mill { who: Selector::You, amount: Value::Const(4) })],
        ..Default::default()
    }
}

/// Baleful Eidolon — {2}{B} Enchantment Creature — Zombie 1/1. Deathtouch.
/// Bestow {4}{B} (CR 702.103): cast as an Aura granting +1/+1 and deathtouch.
pub fn baleful_eidolon() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "Baleful Eidolon",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        bestow: Some(cost(&[generic(4), b()])),
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Deathtouch], scale: None, triggered_abilities: vec![] }),
        ..Default::default()
    }
}
