//! Foundations (FDN) gap batch 8 — reprint staples on existing primitives:
//! a no-lifegain beater, a graveyard-recycler artifact, a choose-color land,
//! attack payoffs, a graveyard-fueled 1-drop, a tutor + flashback, a wheel
//! dragon, "end the turn", a big-creature tutor, two exile-on-death burns, and
//! a changeling. Tests in `tests/recent209.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, Keyword,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{etb, on_you_attack};
use crate::effect::{
    Duration, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, PlayerStaticTarget,
    Predicate, ZoneDest, ZoneRef,
};
use crate::mana::{Color, ManaCost, ManaSymbol, cost, g, generic, r, u, w};

/// Giant Cindermaw — {2}{R} 4/3 Dinosaur Beast. Trample; players can't gain life.
pub fn giant_cindermaw() -> CardDefinition {
    CardDefinition {
        name: "Giant Cindermaw",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur, CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Players can't gain life.",
            effect: StaticEffect::PlayerCannotGainLife {
                target: PlayerStaticTarget::EachPlayer,
            },
        }],
        ..Default::default()
    }
}

/// Feldon's Cane — {1} Artifact. {T}, Exile this artifact: Shuffle your
/// graveyard into your library.
pub fn feldons_cane() -> CardDefinition {
    CardDefinition {
        name: "Feldon's Cane",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            exile_self_cost: true,
            effect: Effect::ShuffleGraveyardIntoLibrary {
                who: PlayerRef::You,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Uncharted Haven — Land. Enters tapped; as it enters, choose a color.
/// {T}: Add one mana of the chosen color.
pub fn uncharted_haven() -> CardDefinition {
    CardDefinition {
        name: "Uncharted Haven",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Tap {
                    what: Selector::This,
                },
            },
            etb(Effect::ChooseColorForSelf),
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::ChosenColorOfSource,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ancestor Dragon — {4}{W}{W} 5/6 Dragon. Flying; whenever one or more
/// creatures you control attack, gain 1 life for each attacking creature.
pub fn ancestor_dragon() -> CardDefinition {
    CardDefinition {
        name: "Ancestor Dragon",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 5,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_you_attack(Effect::GainLife {
            who: Selector::You,
            amount: Value::count(Selector::EachMatching {
                zone: ZoneRef::Battlefield,
                filter: R::Creature.and(R::ControlledByYou).and(R::IsAttacking),
            }),
        })],
        ..Default::default()
    }
}

/// Jazal Goldmane — {2}{W}{W} 4/4 Legendary Cat Warrior. First strike;
/// {3}{W}{W}: Attacking creatures you control get +X/+X until end of turn,
/// where X is the number of attacking creatures.
pub fn jazal_goldmane() -> CardDefinition {
    let attackers = || {
        Value::count(Selector::EachMatching {
            zone: ZoneRef::Battlefield,
            filter: R::Creature.and(R::ControlledByYou).and(R::IsAttacking),
        })
    };
    CardDefinition {
        name: "Jazal Goldmane",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::FirstStrike],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w(), w()]),
            effect: Effect::PumpPT {
                what: Selector::EachMatching {
                    zone: ZoneRef::Battlefield,
                    filter: R::Creature.and(R::ControlledByYou).and(R::IsAttacking),
                },
                power: attackers(),
                toughness: attackers(),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ghitu Lavarunner — {R} 1/2 Human Wizard. As long as there are two or more
/// instant and/or sorcery cards in your graveyard, it gets +1/+0 and has haste.
pub fn ghitu_lavarunner() -> CardDefinition {
    let two_spells = Predicate::SelectorCountAtLeast {
        sel: Selector::EachMatching {
            zone: ZoneRef::Graveyard(PlayerRef::You),
            filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
        },
        n: Value::Const(2),
    };
    CardDefinition {
        name: "Ghitu Lavarunner",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Gets +1/+0 and has haste while 2+ instant/sorcery cards are in your graveyard.",
            effect: StaticEffect::PumpSelfIf {
                condition: two_spells,
                power: 1,
                toughness: 0,
                keywords: vec![Keyword::Haste],
            },
        }],
        ..Default::default()
    }
}

/// Mystical Teachings — {3}{U} Instant. Search your library for an instant card
/// or a card with flash, put it into your hand, then shuffle. Flashback {5}{B}.
pub fn mystical_teachings() -> CardDefinition {
    CardDefinition {
        name: "Mystical Teachings",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Flashback(ManaCost {
            symbols: vec![ManaSymbol::Generic(5), ManaSymbol::Colored(Color::Black)],
        })],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: R::HasCardType(CardType::Instant).or(R::HasKeyword(Keyword::Flash)),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}

/// Dragon Mage — {5}{R}{R} 5/5 Dragon Wizard. Flying; whenever it deals combat
/// damage to a player, each player discards their hand, then draws seven cards.
pub fn dragon_mage() -> CardDefinition {
    CardDefinition {
        name: "Dragon Mage",
        cost: cost(&[generic(5), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Wizard],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(100),
                    random: false,
                },
                Effect::Draw {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(7),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Time Stop — {4}{U}{U} Instant. End the turn.
pub fn time_stop() -> CardDefinition {
    CardDefinition {
        name: "Time Stop",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::EndTheTurn,
        ..Default::default()
    }
}

/// Fierce Empath — {2}{G} 1/1 Elf. When it enters, you may search your library
/// for a creature card with mana value 6 or greater, put it into your hand.
pub fn fierce_empath() -> CardDefinition {
    CardDefinition {
        name: "Fierce Empath",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::Creature.and(R::ManaValueAtLeast(6)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Obliterating Bolt — {1}{R} Sorcery. Deals 4 damage to target creature or
/// planeswalker. If that permanent would die this turn, exile it instead.
pub fn obliterating_bolt() -> CardDefinition {
    CardDefinition {
        name: "Obliterating Bolt",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn {
                what: crate::effect::shortcut::target_filtered(R::Creature.or(R::Planeswalker)),
            },
            Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::Const(4),
            },
        ]),
        ..Default::default()
    }
}

/// Elspeth's Smite — {W} Instant. Deals 3 damage to target attacking or
/// blocking creature. If that creature would die this turn, exile it instead.
pub fn elspeths_smite() -> CardDefinition {
    CardDefinition {
        name: "Elspeth's Smite",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn {
                what: crate::effect::shortcut::target_filtered(
                    R::Creature.and(R::IsAttacking.or(R::IsBlocking)),
                ),
            },
            Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::Const(3),
            },
        ]),
        ..Default::default()
    }
}

/// Taurean Mauler — {2}{R} 2/2 Shapeshifter. Changeling; whenever an opponent
/// casts a spell, you may put a +1/+1 counter on it.
pub fn taurean_mauler() -> CardDefinition {
    CardDefinition {
        name: "Taurean Mauler",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shapeshifter],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Changeling],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
            effect: Effect::MayDo {
                description: "put a +1/+1 counter on Taurean Mauler".into(),
                body: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
        }],
        ..Default::default()
    }
}
