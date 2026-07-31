//! Betrayers of Kamigawa (BOK) gap closure. Tests in `classic_sets/bok`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, StaticAbility, Subtypes,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::{
    Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Selector, StaticEffect, ZoneDest,
    shortcut::{deal, lose_life, on_attack, on_dies, soulshift, spiritcraft, target_any, target_filtered},
};
use crate::mana::{Color, b, cost, g, generic, r, u, w, x};

fn creature(
    name: &'static str,
    c: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(
    name: &'static str,
    c: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        ..creature(name, c, types, p, t)
    }
}

fn instant(name: &'static str, c: crate::mana::ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

/// An Arcane instant — the splice-onto-Arcane host type.
fn arcane_instant(
    name: &'static str,
    c: crate::mana::ManaCost,
    effect: Effect,
) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Arcane],
            ..Default::default()
        },
        ..instant(name, c, effect)
    }
}

fn sorcery(name: &'static str, c: crate::mana::ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

fn arcane_sorcery(
    name: &'static str,
    c: crate::mana::ManaCost,
    effect: Effect,
) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Arcane],
            ..Default::default()
        },
        ..sorcery(name, c, effect)
    }
}

/// A 1/1 colorless Spirit token — the Kamigawa staple.
fn spirit_token() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        ..Default::default()
    }
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Akki Blizzard-Herder — {1}{R} 1/1. When it dies, each player sacrifices a
/// land.
pub fn akki_blizzard_herder() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachPlayer),
            count: Value::ONE,
            filter: R::Land,
        })],
        ..creature(
            "Akki Blizzard-Herder",
            cost(&[generic(1), r()]),
            vec![CreatureType::Goblin, CreatureType::Shaman],
            1,
            1,
        )
    }
}

/// Ashen Monstrosity — {5}{R}{R} 7/4 with haste that has to swing.
pub fn ashen_monstrosity() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste, Keyword::MustAttack],
        ..creature(
            "Ashen Monstrosity",
            cost(&[generic(5), r(), r()]),
            vec![CreatureType::Spirit],
            7,
            4,
        )
    }
}

/// Body of Jukai — {7}{G}{G} 8/5 trample with soulshift 8.
pub fn body_of_jukai() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![soulshift(8)],
        ..creature(
            "Body of Jukai",
            cost(&[generic(7), g(), g()]),
            vec![CreatureType::Spirit],
            8,
            5,
        )
    }
}

/// Forked-Branch Garami — {3}{G}{G} 4/4 with soulshift 4 twice.
pub fn forked_branch_garami() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![soulshift(4), soulshift(4)],
        ..creature(
            "Forked-Branch Garami",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Spirit],
            4,
            4,
        )
    }
}

/// Harbinger of Spring — {4}{G} 2/1 with protection from non-Spirit creatures
/// and soulshift 4.
pub fn harbinger_of_spring() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::ProtectionFromMatching(Box::new(
            R::Creature.and(R::Not(Box::new(R::HasCreatureType(CreatureType::Spirit)))),
        ))],
        triggered_abilities: vec![soulshift(4)],
        ..creature(
            "Harbinger of Spring",
            cost(&[generic(4), g()]),
            vec![CreatureType::Spirit],
            2,
            1,
        )
    }
}

/// Kami of the Honored Dead — {5}{W}{W} 3/5 flier that converts damage it takes
/// into life, with soulshift 6.
pub fn kami_of_the_honored_dead() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
                effect: Effect::GainLife { who: Selector::You, amount: Value::TriggerEventAmount },
            },
            soulshift(6),
        ],
        ..creature(
            "Kami of the Honored Dead",
            cost(&[generic(5), w(), w()]),
            vec![CreatureType::Spirit],
            3,
            5,
        )
    }
}

/// Shinka Gatekeeper — {2}{R} 3/2 that reflects the damage it takes onto you.
pub fn shinka_gatekeeper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::You,
                amount: Value::TriggerEventAmount,
            },
        }],
        ..creature(
            "Shinka Gatekeeper",
            cost(&[generic(2), r()]),
            vec![CreatureType::Ogre, CreatureType::Warrior],
            3,
            2,
        )
    }
}

/// Scourge of Numai — {3}{B} 4/4 that bleeds you 2 each upkeep without an Ogre.
pub fn scourge_of_numai() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::If {
                cond: Predicate::Not(Box::new(Predicate::SelectorExists(
                    Selector::EachPermanent(
                        R::ControlledByYou.and(R::HasCreatureType(CreatureType::Ogre)),
                    ),
                ))),
                then: Box::new(lose_life(2, Selector::You)),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature(
            "Scourge of Numai",
            cost(&[generic(3), b()]),
            vec![CreatureType::Demon, CreatureType::Spirit],
            4,
            4,
        )
    }
}

/// Takenuma Bleeder — {2}{B} 3/3 that costs you 1 life per swing or block
/// unless you control a Demon.
pub fn takenuma_bleeder() -> CardDefinition {
    let bleed = || Effect::If {
        cond: Predicate::Not(Box::new(Predicate::SelectorExists(Selector::EachPermanent(
            R::ControlledByYou.and(R::HasCreatureType(CreatureType::Demon)),
        )))),
        then: Box::new(lose_life(1, Selector::You)),
        else_: Box::new(Effect::Noop),
    };
    CardDefinition {
        triggered_abilities: vec![
            on_attack(bleed()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: bleed(),
            },
        ],
        ..creature(
            "Takenuma Bleeder",
            cost(&[generic(2), b()]),
            vec![CreatureType::Ogre, CreatureType::Shaman],
            3,
            3,
        )
    }
}

/// Scaled Hulk — {5}{G} 4/4 that grows on every Spirit or Arcane spell.
pub fn scaled_hulk() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![spiritcraft(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Scaled Hulk",
            cost(&[generic(5), g()]),
            vec![CreatureType::Spirit],
            4,
            4,
        )
    }
}

/// Oyobi, Who Split the Heavens — {6}{W} 3/6 flier minting a 3/3 flying Spirit
/// on each Spirit or Arcane spell.
pub fn oyobi_who_split_the_heavens() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![spiritcraft(Effect::CreateToken {
            definition: TokenDefinition {
                name: "Spirit".into(),
                power: 3,
                toughness: 3,
                colors: vec![Color::White],
                card_types: vec![CardType::Creature],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Spirit],
                    ..Default::default()
                },
                keywords: vec![Keyword::Flying],
                ..Default::default()
            },
            count: Value::ONE,
            who: PlayerRef::You,
        })],
        ..legend(
            "Oyobi, Who Split the Heavens",
            cost(&[generic(6), w()]),
            vec![CreatureType::Spirit],
            3,
            6,
        )
    }
}

/// Kyoki, Sanity's Eclipse — {4}{B}{B} 6/4 exiling a card from an opponent's
/// hand on each Spirit or Arcane spell.
pub fn kyoki_sanitys_eclipse() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![spiritcraft(Effect::ExileFromHand {
            who: target_filtered(R::OpponentPlayer),
            amount: Value::ONE,
        })],
        ..legend(
            "Kyoki, Sanity's Eclipse",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Demon, CreatureType::Spirit],
            6,
            4,
        )
    }
}

/// Ishi-Ishi, Akki Crackshot — {1}{R} 1/1 that burns an opponent for 2 on each
/// Spirit or Arcane spell they cast.
pub fn ishi_ishi_akki_crackshot() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Spirit)
                        .or(R::HasSpellSubtype(crate::card::SpellSubtype::Arcane)),
                },
            ),
            effect: deal(2, Selector::Player(PlayerRef::Triggerer)),
        }],
        ..legend(
            "Ishi-Ishi, Akki Crackshot",
            cost(&[generic(1), r()]),
            vec![CreatureType::Goblin, CreatureType::Warrior],
            1,
            1,
        )
    }
}

/// Ogre Recluse — {3}{R} 5/4 that taps whenever anyone casts a spell.
pub fn ogre_recluse() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
            effect: Effect::Tap { what: Selector::This },
        }],
        ..creature(
            "Ogre Recluse",
            cost(&[generic(3), r()]),
            vec![CreatureType::Ogre, CreatureType::Warrior],
            5,
            4,
        )
    }
}

/// Indebted Samurai — {3}{W} 2/3 bushido 1 that grows when a Samurai dies.
pub fn indebted_samurai() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Bushido(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Samurai),
                }),
            effect: Effect::MayDo {
                description: "Put a +1/+1 counter on Indebted Samurai?".into(),
                body: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
        }],
        ..creature(
            "Indebted Samurai",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Samurai],
            2,
            3,
        )
    }
}

/// Silverstorm Samurai — {4}{W}{W} 3/3 with flash and bushido 1.
pub fn silverstorm_samurai() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Bushido(1)],
        ..creature(
            "Silverstorm Samurai",
            cost(&[generic(4), w(), w()]),
            vec![CreatureType::Fox, CreatureType::Samurai],
            3,
            3,
        )
    }
}

/// Takeno's Cavalry — {3}{W} 1/1 bushido 1 that snipes attacking or blocking
/// Spirits.
pub fn takenos_cavalry() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Bushido(1)],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: deal(
                1,
                target_filtered(
                    R::HasCreatureType(CreatureType::Spirit)
                        .and(R::IsAttacking.or(R::IsBlocking)),
                ),
            ),
            ..Default::default()
        }],
        ..creature(
            "Takeno's Cavalry",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Samurai, CreatureType::Archer],
            1,
            1,
        )
    }
}

/// Isao, Enlightened Bushi — {2}{G} 2/1 uncounterable bushido 2 that
/// regenerates Samurai.
pub fn isao_enlightened_bushi() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeCountered, Keyword::Bushido(2)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Regenerate {
                what: target_filtered(R::HasCreatureType(CreatureType::Samurai)),
            },
            ..Default::default()
        }],
        ..legend(
            "Isao, Enlightened Bushi",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Samurai],
            2,
            1,
        )
    }
}

/// Mannichi, the Fevered Dream — {2}{R} 1/2 that flips everyone's power and
/// toughness.
pub fn mannichi_the_fevered_dream() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::SwitchPT {
                what: Selector::EachPermanent(R::Creature),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend(
            "Mannichi, the Fevered Dream",
            cost(&[generic(2), r()]),
            vec![CreatureType::Spirit],
            1,
            2,
        )
    }
}

/// Minamo Sightbender — {1}{U} 1/2 that slips a small creature past blockers.
pub fn minamo_sightbender() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::PowerAtMostXFromCost)),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Minamo Sightbender",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            2,
        )
    }
}

/// Split-Tail Miko — {1}{W} 1/1 that prevents the next 2 damage to any target.
pub fn split_tail_miko() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            effect: Effect::PreventNextDamage {
                target: target_any(),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..creature(
            "Split-Tail Miko",
            cost(&[generic(1), w()]),
            vec![CreatureType::Fox, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Sakura-Tribe Springcaller — {3}{G} 2/4 that banks {G} each upkeep.
pub fn sakura_tribe_springcaller() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::AddManaKeptThisTurn { who: PlayerRef::You, colors: vec![Color::Green] },
        }],
        ..creature(
            "Sakura-Tribe Springcaller",
            cost(&[generic(3), g()]),
            vec![CreatureType::Snake, CreatureType::Shaman],
            2,
            4,
        )
    }
}

/// Sakiko, Mother of Summer — {4}{G}{G} 3/3. Combat damage your creatures deal
/// to a player banks that much {G} for the turn.
pub fn sakiko_mother_of_summer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
            effect: Effect::AddManaKeptThisTurnCount {
                who: PlayerRef::You,
                color: Color::Green,
                amount: Value::TriggerEventAmount,
            },
        }],
        ..legend(
            "Sakiko, Mother of Summer",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Snake, CreatureType::Shaman],
            3,
            3,
        )
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Hero's Demise — {1}{B} Instant. Destroy target legendary creature.
pub fn heros_demise() -> CardDefinition {
    instant(
        "Hero's Demise",
        cost(&[generic(1), b()]),
        Effect::Destroy {
            what: target_filtered(
                R::Creature.and(R::HasSupertype(crate::card::Supertype::Legendary)),
            ),
        },
    )
}

/// Terashi's Verdict — {1}{W} Arcane instant. Destroy target attacking creature
/// with power 3 or less.
pub fn terashis_verdict() -> CardDefinition {
    arcane_instant(
        "Terashi's Verdict",
        cost(&[generic(1), w()]),
        Effect::Destroy {
            what: target_filtered(R::Creature.and(R::IsAttacking).and(R::PowerAtMost(3))),
        },
    )
}

/// First Volley — {1}{R} Arcane instant. 1 damage to a creature and 1 to its
/// controller.
pub fn first_volley() -> CardDefinition {
    arcane_instant(
        "First Volley",
        cost(&[generic(1), r()]),
        Effect::Seq(vec![
            deal(1, target_filtered(R::Creature)),
            deal(1, Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0))))),
        ]),
    )
}

/// Three Tragedies — {3}{B}{B} Arcane sorcery. Target player discards three.
pub fn three_tragedies() -> CardDefinition {
    arcane_sorcery(
        "Three Tragedies",
        cost(&[generic(3), b(), b()]),
        Effect::Discard {
            who: target_filtered(R::Player),
            amount: Value::Const(3),
            random: false,
        },
    )
}

/// Reduce to Dreams — {3}{U}{U} Sorcery. Bounce every artifact and enchantment.
pub fn reduce_to_dreams() -> CardDefinition {
    sorcery(
        "Reduce to Dreams",
        cost(&[generic(3), u(), u()]),
        Effect::Move {
            what: Selector::EachPermanent(R::Artifact.or(R::Enchantment)),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
    )
}

/// Ribbons of the Reikai — {4}{U} Arcane sorcery. Draw a card for each Spirit
/// you control.
pub fn ribbons_of_the_reikai() -> CardDefinition {
    arcane_sorcery(
        "Ribbons of the Reikai",
        cost(&[generic(4), u()]),
        Effect::Draw {
            who: Selector::You,
            amount: Value::count(Selector::EachPermanent(
                R::ControlledByYou.and(R::HasCreatureType(CreatureType::Spirit)),
            )),
        },
    )
}

/// Uproot — {3}{G} Arcane sorcery. Put target land on top of its owner's
/// library.
pub fn uproot() -> CardDefinition {
    arcane_sorcery(
        "Uproot",
        cost(&[generic(3), g()]),
        Effect::Move {
            what: target_filtered(R::Land),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOfMoved,
                pos: LibraryPosition::Top,
            },
        },
    )
}

/// Stir the Grave — {X}{B} Sorcery. Reanimate a creature card with mana value X
/// or less.
pub fn stir_the_grave() -> CardDefinition {
    sorcery(
        "Stir the Grave",
        cost(&[x(), b()]),
        Effect::Move {
            what: target_filtered(
                R::InYourGraveyard.and(R::Creature).and(R::ManaValueAtMostXFromCost),
            ),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
    )
}

/// Unchecked Growth — {2}{G} Arcane instant. +4/+4, plus trample for a Spirit.
pub fn unchecked_growth() -> CardDefinition {
    arcane_instant(
        "Unchecked Growth",
        cost(&[generic(2), g()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(4),
                toughness: Value::Const(4),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::HasCreatureType(CreatureType::Spirit),
                },
                then: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Enshrined Memories — {X}{G} Sorcery. Reveal the top X; creatures to hand,
/// the rest to the bottom.
pub fn enshrined_memories() -> CardDefinition {
    sorcery(
        "Enshrined Memories",
        cost(&[x(), g()]),
        Effect::RevealTopTakeMatchingToHand {
            who: PlayerRef::You,
            count: Value::XFromCost,
            filter: R::Creature,
        },
    )
}

/// Sosuke's Summons — {2}{G} Sorcery. Two Snakes now, and it climbs back out of
/// the graveyard whenever a nontoken Snake you control enters.
pub fn sosukes_summons() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::FromYourGraveyard)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Snake)
                        .and(R::Not(Box::new(R::IsToken))),
                }),
            effect: Effect::MayDo {
                description: "Return Sosuke's Summons to your hand?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..sorcery(
            "Sosuke's Summons",
            cost(&[generic(2), g()]),
            Effect::CreateToken {
                definition: TokenDefinition {
                    name: "Snake".into(),
                    power: 1,
                    toughness: 1,
                    colors: vec![Color::Green],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Snake],
                        ..Default::default()
                    },
                    ..Default::default()
                },
                count: Value::Const(2),
                who: PlayerRef::You,
            },
        )
    }
}

// ── Noncreature permanents ──────────────────────────────────────────────────

/// Day of Destiny — {3}{W} Legendary Enchantment. Your legends get +2/+2.
pub fn day_of_destiny() -> CardDefinition {
    CardDefinition {
        name: "Day of Destiny",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![crate::card::Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Legendary creatures you control get +2/+2.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::HasSupertype(crate::card::Supertype::Legendary)),
                power: 2,
                toughness: 2,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

/// In the Web of War — {3}{R}{R} Enchantment. Every creature you deploy comes
/// in swinging.
pub fn in_the_web_of_war() -> CardDefinition {
    CardDefinition {
        name: "In the Web of War",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(2),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Orb of Dreams — {3} Artifact. Permanents enter tapped.
pub fn orb_of_dreams() -> CardDefinition {
    CardDefinition {
        name: "Orb of Dreams",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Permanents enter tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::EachPermanent(R::Any) },
        }],
        ..Default::default()
    }
}

/// Mirror Gallery — {5} Artifact. The legend rule doesn't apply.
pub fn mirror_gallery() -> CardDefinition {
    CardDefinition {
        name: "Mirror Gallery",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "The \"legend rule\" doesn't apply.",
            effect: StaticEffect::LegendRuleDoesntApply,
        }],
        ..Default::default()
    }
}

/// Gods' Eye, Gate to the Reikai — Legendary Land that leaves a Spirit behind.
pub fn gods_eye_gate_to_the_reikai() -> CardDefinition {
    CardDefinition {
        name: "Gods' Eye, Gate to the Reikai",
        card_types: vec![CardType::Land],
        supertypes: vec![crate::card::Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource),
            effect: Effect::CreateToken {
                definition: spirit_token(),
                count: Value::ONE,
                who: PlayerRef::You,
            },
        }],
        ..Default::default()
    }
}

/// Tendo Ice Bridge — Land. Taps for {C}, or spends its charge counter for any
/// color.
pub fn tendo_ice_bridge() -> CardDefinition {
    CardDefinition {
        name: "Tendo Ice Bridge",
        card_types: vec![CardType::Land],
        enters_with_counters: Some((CounterType::Charge, Value::ONE)),
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                remove_counter_cost: Some((CounterType::Charge, 1)),
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Yomiji, Who Bars the Way — {5}{W}{W} 4/4. Other legendary permanents bounce
/// to their owner's hand instead of staying dead.
pub fn yomiji_who_bars_the_way() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasSupertype(crate::card::Supertype::Legendary)
                        .and(R::OtherThanSource),
                },
            ),
            effect: Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        }],
        ..legend(
            "Yomiji, Who Bars the Way",
            cost(&[generic(5), w(), w()]),
            vec![CreatureType::Spirit],
            4,
            4,
        )
    }
}

/// Heed the Mists — {3}{U}{U} Arcane sorcery. Mill one, then draw that card's
/// mana value.
pub fn heed_the_mists() -> CardDefinition {
    arcane_sorcery(
        "Heed the Mists",
        cost(&[generic(3), u(), u()]),
        Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::ONE },
            Effect::Draw {
                who: Selector::You,
                amount: Value::ManaValueOf(Box::new(Selector::LastMoved)),
            },
        ]),
    )
}

/// Ward of Piety — {1}{W} Aura. {1}{W}: shunt the next 1 damage off the
/// enchanted creature onto any target.
pub fn ward_of_piety() -> CardDefinition {
    CardDefinition {
        name: "Ward of Piety",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::RedirectNextDamage {
                target: Selector::AttachedTo(Box::new(Selector::This)),
                to: target_any(),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Heart of Light — {2}{W} Aura. The enchanted creature neither deals nor takes
/// damage.
pub fn heart_of_light() -> CardDefinition {
    CardDefinition {
        name: "Heart of Light",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![StaticAbility {
            description: "Prevent all damage that would be dealt to and dealt by enchanted creature.",
            effect: StaticEffect::PreventAllDamageToAndFromEnchanted,
        }],
        ..Default::default()
    }
}

// ── The Genju cycle ─────────────────────────────────────────────────────────

/// A Genju: an Aura on a land that can animate its host for {2} and climbs back
/// out of the graveyard when the land dies.
fn genju(
    name: &'static str,
    c: crate::mana::ManaCost,
    land: LandType,
    (p, t): (i32, i32),
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Land.and(R::HasLandType(land))),
        },
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::BecomeCreature {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                power: Value::Const(p),
                toughness: Value::Const(t),
                creature_types: vec![CreatureType::Spirit],
                keywords,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        // The printed trigger is "when enchanted land is put into a graveyard";
        // modeled as the Aura's own LTB, which is the same event in practice
        // (an orphaned Aura is swept the moment its land leaves).
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::PermanentLeavesBattlefield,
                EventScope::SelfSource,
            ),
            effect: Effect::MayDo {
                description: "Return the Genju to your hand?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Genju of the Cedars — {G}. The Forest swings as a 4/4.
pub fn genju_of_the_cedars() -> CardDefinition {
    genju(
        "Genju of the Cedars",
        cost(&[g()]),
        LandType::Forest,
        (4, 4),
        vec![],
    )
}

/// Genju of the Falls — {U}. The Island swings as a 3/2 flier.
pub fn genju_of_the_falls() -> CardDefinition {
    genju(
        "Genju of the Falls",
        cost(&[u()]),
        LandType::Island,
        (3, 2),
        vec![Keyword::Flying],
    )
}

/// Genju of the Spires — {R}. The Mountain swings as a 6/1.
pub fn genju_of_the_spires() -> CardDefinition {
    genju(
        "Genju of the Spires",
        cost(&[r()]),
        LandType::Mountain,
        (6, 1),
        vec![],
    )
}

/// Genju of the Fens — {B}. The Swamp swings as a 2/2. (Its granted
/// "{B}: +1/+1" is dropped — `BecomeCreature` grants keywords, not abilities.)
pub fn genju_of_the_fens() -> CardDefinition {
    genju(
        "Genju of the Fens",
        cost(&[b()]),
        LandType::Swamp,
        (2, 2),
        vec![],
    )
}

/// Genju of the Fields — {W}. The Plains swings as a 2/5 with lifelink (the
/// printed "gain that much life" trigger, modeled as the keyword).
pub fn genju_of_the_fields() -> CardDefinition {
    genju(
        "Genju of the Fields",
        cost(&[w()]),
        LandType::Plains,
        (2, 5),
        vec![Keyword::Lifelink],
    )
}

/// Genju of the Realm — {W}{U}{B}{R}{G}. Any land swings as a legendary 8/12
/// trampler.
pub fn genju_of_the_realm() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        ..genju(
            "Genju of the Realm",
            cost(&[w(), u(), b(), r(), g()]),
            LandType::Plains,
            (8, 12),
            vec![Keyword::Trample],
        )
    }
}

// ── Moonfolk ────────────────────────────────────────────────────────────────

/// Floodbringer — {1}{U} 1/2 flier. Bounce a land to tap one down.
pub fn floodbringer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            return_permanent_cost: Some(R::Land),
            effect: Effect::Tap { what: target_filtered(R::Land) },
            ..Default::default()
        }],
        ..creature(
            "Floodbringer",
            cost(&[generic(1), u()]),
            vec![CreatureType::Moonfolk, CreatureType::Wizard],
            1,
            2,
        )
    }
}

/// Soratami Mindsweeper — {3}{U} 1/4 flier. Bounce a land to mill two.
pub fn soratami_mindsweeper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            return_permanent_cost: Some(R::Land),
            effect: Effect::Mill {
                who: target_filtered(R::Player),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..creature(
            "Soratami Mindsweeper",
            cost(&[generic(3), u()]),
            vec![CreatureType::Moonfolk, CreatureType::Wizard],
            1,
            4,
        )
    }
}

// ── More creatures ──────────────────────────────────────────────────────────

/// Kaijin of the Vanishing Touch — {1}{U} 0/3 Defender that bounces whatever it
/// blocks at end of combat.
pub fn kaijin_of_the_vanishing_touch() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
            effect: Effect::DelayUntilWithCapture {
                kind: crate::effect::DelayedTriggerKind::EndOfCombat,
                capture: Selector::BlockedAttacker,
                body: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
            },
        }],
        ..creature(
            "Kaijin of the Vanishing Touch",
            cost(&[generic(1), u()]),
            vec![CreatureType::Spirit],
            0,
            3,
        )
    }
}

/// Kitsune Palliator — {2}{W} 0/2 that soaks 1 off every creature and player.
pub fn kitsune_palliator() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PreventNextDamage {
                target: Selector::EachPermanent(R::Creature),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Kitsune Palliator",
            cost(&[generic(2), w()]),
            vec![CreatureType::Fox, CreatureType::Cleric],
            0,
            2,
        )
    }
}

/// Lifespinner — {3}{G} 3/3. Feed it three Spirits for a legendary Spirit
/// permanent off the top of your library.
pub fn lifespinner() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Spirit), 3)),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::HasSupertype(crate::card::Supertype::Legendary)
                    .and(R::HasCreatureType(CreatureType::Spirit)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature(
            "Lifespinner",
            cost(&[generic(3), g()]),
            vec![CreatureType::Spirit],
            3,
            3,
        )
    }
}

/// Shizuko, Caller of Autumn — {1}{G}{G} 2/3. Every player's upkeep banks
/// {G}{G}{G} for them.
pub fn shizuko_caller_of_autumn() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::AddManaKeptThisTurnCount {
                who: PlayerRef::ActivePlayer,
                color: Color::Green,
                amount: Value::Const(3),
            },
        }],
        ..legend(
            "Shizuko, Caller of Autumn",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Snake, CreatureType::Shaman],
            2,
            3,
        )
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// That Which Was Taken — {5} Legendary Artifact. Divinity counters hand out
/// indestructibility.
pub fn that_which_was_taken() -> CardDefinition {
    CardDefinition {
        name: "That Which Was Taken",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        supertypes: vec![crate::card::Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            effect: Effect::AddCounter {
                what: target_filtered(R::OtherThanSource),
                kind: CounterType::Divinity,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "Each permanent with a divinity counter on it has indestructible.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::WithCounter(CounterType::Divinity)),
                keyword: Keyword::Indestructible,
            },
        }],
        ..Default::default()
    }
}

/// Ronin Warclub — {3} Equipment. +2/+1, and it leaps onto each creature you
/// deploy.
pub fn ronin_warclub() -> CardDefinition {
    CardDefinition {
        name: "Ronin Warclub",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(5)]))],
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 2,
            toughness: 1,
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::Attach { what: Selector::This, to: Selector::TriggerSource },
        }],
        ..Default::default()
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Stream of Consciousness — {1}{U} Arcane instant. Shuffle up to four cards
/// from a graveyard back into its owner's library.
pub fn stream_of_consciousness() -> CardDefinition {
    arcane_instant(
        "Stream of Consciousness",
        cost(&[generic(1), u()]),
        Effect::ApplyToTargets {
            max_targets: 4,
            min_targets: 0,
            filter: R::InGraveyard,
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOfMoved,
                    pos: LibraryPosition::Shuffled,
                },
            }),
        },
    )
}

/// Toils of Night and Day — {2}{U} Arcane instant. Tap or untap two permanents.
pub fn toils_of_night_and_day() -> CardDefinition {
    let flip = |slot: u8| {
        Effect::ChooseMode(vec![
            Effect::Tap { what: Selector::Target(slot) },
            Effect::Untap { what: Selector::Target(slot), up_to: None },
        ])
    };
    arcane_instant(
        "Toils of Night and Day",
        cost(&[generic(2), u()]),
        Effect::Seq(vec![flip(0), flip(1)]),
    )
}

/// Call for Blood — {4}{B} Arcane instant. Feed it a creature to shrink another
/// by the eaten creature's power.
pub fn call_for_blood() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Creature,
            count: 1,
        }],
        ..arcane_instant(
            "Call for Blood",
            cost(&[generic(4), b()]),
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Times(
                    Box::new(Value::SacrificedPower),
                    Box::new(Value::Const(-1)),
                ),
                toughness: Value::Times(
                    Box::new(Value::SacrificedPower),
                    Box::new(Value::Const(-1)),
                ),
                duration: Duration::EndOfTurn,
            },
        )
    }
}

// ── The Baku cycle ──────────────────────────────────────────────────────────

/// Spiritcraft: "you may put a ki counter on this" — the Baku cycle's engine.
fn ki_charger() -> TriggeredAbility {
    spiritcraft(Effect::MayDo {
        description: "Put a ki counter on this permanent?".into(),
        body: Box::new(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::Ki,
            amount: Value::ONE,
        }),
    })
}

/// "{1}[, {T}], Remove X ki counters from this: `payoff`" — the Baku payoff.
fn ki_payoff(tap: bool, payoff: Effect) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost(&[generic(1)]),
        tap_cost: tap,
        remove_counter_x: Some(CounterType::Ki),
        effect: payoff,
        ..Default::default()
    }
}

/// Blademane Baku — {1}{R} 1/1. Each ki counter spent is +2/+0.
pub fn blademane_baku() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![ki_charger()],
        activated_abilities: vec![ki_payoff(
            false,
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Times(Box::new(Value::XFromCost), Box::new(Value::Const(2))),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        )],
        ..creature("Blademane Baku", cost(&[generic(1), r()]), vec![CreatureType::Spirit], 1, 1)
    }
}

/// Petalmane Baku — {1}{G} 1/2. Spend ki for that much mana of one color.
pub fn petalmane_baku() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![ki_charger()],
        activated_abilities: vec![ki_payoff(
            false,
            Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::XFromCost),
            },
        )],
        ..creature("Petalmane Baku", cost(&[generic(1), g()]), vec![CreatureType::Spirit], 1, 2)
    }
}

/// Quillmane Baku — {4}{U} 3/3. Spend ki to bounce a creature of that mana
/// value or less.
pub fn quillmane_baku() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![ki_charger()],
        activated_abilities: vec![ki_payoff(
            true,
            Effect::Move {
                what: target_filtered(R::Creature.and(R::ManaValueAtMostXFromCost)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        )],
        ..creature("Quillmane Baku", cost(&[generic(4), u()]), vec![CreatureType::Spirit], 3, 3)
    }
}

/// Skullmane Baku — {3}{B}{B} 2/1. Spend ki for that much -X/-X.
pub fn skullmane_baku() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![ki_charger()],
        activated_abilities: vec![ki_payoff(
            true,
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Times(Box::new(Value::XFromCost), Box::new(Value::Const(-1))),
                toughness: Value::Times(Box::new(Value::XFromCost), Box::new(Value::Const(-1))),
                duration: Duration::EndOfTurn,
            },
        )],
        ..creature(
            "Skullmane Baku",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Spirit],
            2,
            1,
        )
    }
}

/// Waxmane Baku — {2}{W} 2/2. Spend ki to tap that many creatures.
pub fn waxmane_baku() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![ki_charger()],
        activated_abilities: vec![ki_payoff(
            false,
            Effect::TargetsExactlyX {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 6,
                    min_targets: 1,
                    filter: R::Creature,
                    effect: Box::new(Effect::Tap { what: Selector::Target(0) }),
                }),
            },
        )],
        ..creature("Waxmane Baku", cost(&[generic(2), w()]), vec![CreatureType::Spirit], 2, 2)
    }
}

/// Baku Altar — {2} Artifact. The same ki engine, spending counters for 1/1
/// Spirit tokens.
pub fn baku_altar() -> CardDefinition {
    CardDefinition {
        name: "Baku Altar",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![ki_charger()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Ki, 1)),
            effect: Effect::CreateToken {
                definition: spirit_token(),
                count: Value::ONE,
                who: PlayerRef::You,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── The Shoal cycle ─────────────────────────────────────────────────────────

/// A Shoal: an {X}{C}{C} Arcane instant you may cast by exiling a `color` card
/// with mana value X from your hand instead of paying its cost.
fn shoal(
    name: &'static str,
    c: crate::mana::ManaCost,
    color: Color,
    effect: Effect,
) -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(crate::card::AlternativeCost {
            exile_filter: Some(R::HasColor(color).and(R::ManaValueExactlyXFromCost)),
            ..Default::default()
        }),
        ..arcane_instant(name, c, effect)
    }
}

/// Blazing Shoal — {X}{R}{R}. Target creature gets +X/+0.
pub fn blazing_shoal() -> CardDefinition {
    shoal(
        "Blazing Shoal",
        cost(&[x(), r(), r()]),
        Color::Red,
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::XFromCost,
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Sickening Shoal — {X}{B}{B}. Target creature gets -X/-X.
pub fn sickening_shoal() -> CardDefinition {
    shoal(
        "Sickening Shoal",
        cost(&[x(), b(), b()]),
        Color::Black,
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Times(Box::new(Value::XFromCost), Box::new(Value::Const(-1))),
            toughness: Value::Times(Box::new(Value::XFromCost), Box::new(Value::Const(-1))),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Nourishing Shoal — {X}{G}{G}. Gain X life.
pub fn nourishing_shoal() -> CardDefinition {
    shoal(
        "Nourishing Shoal",
        cost(&[x(), g(), g()]),
        Color::Green,
        Effect::GainLife { who: Selector::You, amount: Value::XFromCost },
    )
}

/// Disrupting Shoal — {X}{U}{U}. Counter target spell if its mana value is X.
pub fn disrupting_shoal() -> CardDefinition {
    shoal(
        "Disrupting Shoal",
        cost(&[x(), u(), u()]),
        Color::Blue,
        Effect::CounterSpell { what: target_filtered(R::ManaValueExactlyXFromCost) },
    )
}
