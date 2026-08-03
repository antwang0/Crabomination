//! Onslaught (ONS) wave 7 — the set's remaining rares and utility shell.
//! Tests in `classic_sets/ons2`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, ConditionalEquipBonus, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, Keyword, SelectionRequirement as R, StaticAbility, Subtypes,
    Supertype, TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, StaticEffect, Value,
    ZoneDest,
    shortcut::{draw, each_your_creature, etb, target_any, target_filtered},
};
use crate::game::TurnStep;
use crate::mana::{ManaCost, b, cost, g, generic, r, u, w, x};

fn creature(
    name: &'static str,
    c: ManaCost,
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

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn aura(name: &'static str, c: ManaCost, enchant: R) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchant) },
        ..Default::default()
    }
}

fn attached() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

/// "At the beginning of your upkeep, [effect]."
fn on_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
        effect,
    }
}

// ── White ───────────────────────────────────────────────────────────────────

/// Astral Slide — every cycled card can blink a creature until end of turn.
pub fn astral_slide() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardCycled, EventScope::AnyPlayer),
            effect: Effect::MayDo {
                description: "Exile target creature until end of turn?".into(),
                body: Box::new(Effect::ExileReturnToOwnerNextEndStep {
                    what: target_filtered(R::Creature),
                    tapped: false,
                }),
            },
        }],
        ..enchantment("Astral Slide", cost(&[generic(2), w()]))
    }
}

/// Glarecaster — {5}{W} bounces the next damage headed your way at any target.
pub fn glarecaster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), w()]),
            effect: Effect::RedirectNextDamageTo {
                what: Selector::Both(
                    Box::new(Selector::This),
                    Box::new(Selector::Player(PlayerRef::You)),
                ),
                to: target_any(),
            },
            ..Default::default()
        }],
        ..creature(
            "Glarecaster",
            cost(&[generic(4), w(), w()]),
            vec![CreatureType::Bird, CreatureType::Cleric],
            3,
            3,
        )
    }
}

/// Shieldmage Elder — two Clerics buy a damage-prevention shield.
pub fn shieldmage_elder() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((R::Creature.and(R::HasCreatureType(CreatureType::Cleric)), 2)),
            effect: Effect::PreventAllDamageByTargetThisTurn {
                target: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..creature(
            "Shieldmage Elder",
            cost(&[generic(5), w()]),
            vec![CreatureType::Human, CreatureType::Cleric, CreatureType::Wizard],
            2,
            3,
        )
    }
}

// ── Blue ────────────────────────────────────────────────────────────────────

/// Annex — an Aura that hands you the enchanted land.
pub fn annex() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains { what: attached() })],
        ..aura("Annex", cost(&[generic(2), u(), u()]), R::Land)
    }
}

/// Fleeting Aven — a flier that flickers home whenever anyone cycles.
pub fn fleeting_aven() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardCycled, EventScope::AnyPlayer),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        }],
        ..creature(
            "Fleeting Aven",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Bird, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Future Sight — play with the top card revealed, and cast off it.
pub fn future_sight() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Play with the top card of your library revealed.",
                effect: StaticEffect::TopOfLibraryRevealed,
            },
            StaticAbility {
                description: "You may play lands and cast spells from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop { filter: R::Any },
            },
        ],
        ..enchantment("Future Sight", cost(&[generic(2), u(), u(), u()]))
    }
}

/// Psychic Trance — your Wizards each become a one-shot counterspell.
pub fn psychic_trance() -> CardDefinition {
    instant(
        "Psychic Trance",
        cost(&[generic(2), u(), u()]),
        Effect::GainActivatedAbility {
            what: Selector::EachPermanent(
                R::Creature.and(R::HasCreatureType(CreatureType::Wizard)).and(R::ControlledByYou),
            ),
            ability: Box::new(ActivatedAbility {
                tap_cost: true,
                effect: crate::effect::shortcut::counter_target_spell(),
                ..Default::default()
            }),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Quicksilver Dragon — {U} shrugs a single-target spell onto another creature.
pub fn quicksilver_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Morph(cost(&[generic(4), u()]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::ChangeSpellTarget {
                what: target_filtered(R::IsSpellOnStack.and(R::SpellTargetsOnlySource)),
            },
            ..Default::default()
        }],
        ..creature(
            "Quicksilver Dragon",
            cost(&[generic(4), u(), u()]),
            vec![CreatureType::Dragon],
            5,
            5,
        )
    }
}

/// Read the Runes — draw X, then pay for each card with a permanent or a discard.
pub fn read_the_runes() -> CardDefinition {
    instant(
        "Read the Runes",
        cost(&[x(), u()]),
        Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::XFromCost },
            Effect::Repeat {
                count: Value::XFromCost,
                body: Box::new(Effect::MaySacrifice {
                    description: "Sacrifice a permanent instead of discarding?".into(),
                    filter: R::Permanent,
                    count: Value::ONE,
                    then: Box::new(Effect::Noop),
                    else_: Some(Box::new(crate::effect::shortcut::discard(
                        Selector::You,
                        1,
                        false,
                    ))),
                }),
            },
        ]),
    )
}

/// Riptide Entrancer — connect, sacrifice it, and keep a creature for good.
pub fn riptide_entrancer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[u(), u()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MaySacrificeSource {
                description: "Sacrifice Riptide Entrancer to steal a creature?".into(),
                else_: None,
                then: Box::new(Effect::GainControl {
                    what: target_filtered(R::Creature.and(R::ControlledByTriggerPlayer)),
                    to: None,
                    duration: Duration::Permanent,
                }),
            },
        }],
        ..creature(
            "Riptide Entrancer",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Wheel and Deal — refill any number of opponents, and yourself for one card.
pub fn wheel_and_deal() -> CardDefinition {
    instant(
        "Wheel and Deal",
        cost(&[generic(3), u()]),
        Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 4,
                min_targets: 0,
                filter: R::OpponentPlayer,
                effect: Box::new(Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::Target(0),
                        amount: Value::HandSizeOf(PlayerRef::Target(0)),
                        random: false,
                    },
                    Effect::Draw { who: Selector::Target(0), amount: Value::Const(7) },
                ])),
            },
            draw(1),
        ]),
    )
}

// ── Black ───────────────────────────────────────────────────────────────────

/// Entrails Feaster — feed it a graveyard creature each upkeep or it stays down.
pub fn entrails_feaster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_upkeep(Effect::MayDoElse {
            description: "Exile a creature card from a graveyard?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::ExileFromGraveyard {
                    who: PlayerRef::EachPlayer,
                    count: Value::ONE,
                    filter: R::Creature,
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ])),
            else_: Box::new(Effect::Tap { what: Selector::This }),
        })],
        ..creature(
            "Entrails Feaster",
            cost(&[b()]),
            vec![CreatureType::Zombie, CreatureType::Cat],
            1,
            1,
        )
    }
}

/// Prowling Pangolin — a 6/5 anyone can buy off with two creatures.
pub fn prowling_pangolin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::PlayersMayAccept {
            who: PlayerRef::EachPlayer,
            description: "Sacrifice two creatures to kill Prowling Pangolin?".into(),
            on_accept: Box::new(Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::Target(0),
                    count: Value::Const(2),
                    filter: R::Creature,
                },
                Effect::SacrificeSource,
            ])),
            otherwise: Box::new(Effect::Noop),
        })],
        ..creature(
            "Prowling Pangolin",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Pangolin, CreatureType::Beast],
            6,
            5,
        )
    }
}

/// Thrashing Mudspawn — every point it takes, you take too.
pub fn thrashing_mudspawn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(1), b(), b()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::You),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..creature(
            "Thrashing Mudspawn",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Beast],
            4,
            4,
        )
    }
}

// ── Red ─────────────────────────────────────────────────────────────────────

/// Blistering Firecat — three mana, seven trampling damage, then it's gone.
pub fn blistering_firecat() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Trample,
            Keyword::Haste,
            Keyword::Morph(cost(&[r(), r()])),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::SacrificeSource,
        }],
        ..creature(
            "Blistering Firecat",
            cost(&[generic(1), r(), r(), r()]),
            vec![CreatureType::Elemental, CreatureType::Cat],
            7,
            1,
        )
    }
}

/// Commando Raid — your creature's combat damage doubles as a snipe.
pub fn commando_raid() -> CardDefinition {
    instant(
        "Commando Raid",
        cost(&[generic(2), r()]),
        Effect::GrantTriggeredAbility {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            trigger: Box::new(TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Deal damage equal to its power to a creature?".into(),
                    body: Box::new(Effect::DealDamageEqualToPower {
                        source: Selector::This,
                        target: target_filtered(R::Creature.and(R::ControlledByTriggerPlayer)),
                    }),
                },
            }),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Custody Battle — the enchanted creature costs a land a turn to keep.
pub fn custody_battle() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![on_upkeep(Effect::MaySacrifice {
                description: "Sacrifice a land to keep this creature?".into(),
                filter: R::Land,
                count: Value::ONE,
                then: Box::new(Effect::Noop),
                else_: Some(Box::new(Effect::GainControl {
                    what: Selector::This,
                    to: Some(PlayerRef::Target(0)),
                    duration: Duration::Permanent,
                })),
            })],
            ..Default::default()
        }),
        ..aura("Custody Battle", cost(&[generic(1), r()]), R::Creature)
    }
}

/// Erratic Explosion — the top of your deck picks the damage.
pub fn erratic_explosion() -> CardDefinition {
    sorcery(
        "Erratic Explosion",
        cost(&[generic(2), r()]),
        Effect::RevealUntilNonlandDamage { to: target_any() },
    )
}

/// Lavamancer's Skill — a pinger, and a better one on a Wizard.
pub fn lavamancers_skill() -> CardDefinition {
    let ping = |amount: i32| ActivatedAbility {
        tap_cost: true,
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature),
            amount: Value::Const(amount),
        },
        ..Default::default()
    };
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ping(1)],
            conditional: vec![ConditionalEquipBonus {
                host_filter: R::HasCreatureType(CreatureType::Wizard),
                activated_abilities: vec![ping(2)],
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..aura("Lavamancer's Skill", cost(&[generic(1), r()]), R::Creature)
    }
}

/// Skittish Valesk — half the upkeeps it spends face down again.
pub fn skittish_valesk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(5), r()]))],
        triggered_abilities: vec![on_upkeep(Effect::FlipCoin {
            count: Value::ONE,
            on_heads: Box::new(Effect::Noop),
            on_tails: Box::new(Effect::TurnFaceDown { what: Selector::This }),
        })],
        ..creature("Skittish Valesk", cost(&[generic(6), r()]), vec![CreatureType::Beast], 5, 5)
    }
}

/// Tephraderm — it hands back every point it takes.
pub fn tephraderm() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::LastDamagerOf(Box::new(Selector::This)),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..creature("Tephraderm", cost(&[generic(4), r()]), vec![CreatureType::Beast], 4, 5)
    }
}

// ── Green ───────────────────────────────────────────────────────────────────

/// Gigapede — an untargetable 6/1 that buys its way back each upkeep.
pub fn gigapede() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shroud],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::FromYourGraveyard,
            ),
            effect: Effect::MayDiscard {
                description: "Discard a card to return Gigapede to your hand?".into(),
                count: Value::ONE,
                then: Box::new(Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) }),
                else_: None,
            },
        }],
        ..creature("Gigapede", cost(&[generic(3), g(), g()]), vec![CreatureType::Insect], 6, 1)
    }
}

/// Kamahl, Fist of Krosa — lands become creatures, and the team goes huge.
pub fn kamahl_fist_of_krosa() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[g()]),
                effect: Effect::Seq(vec![
                    Effect::AnimateAsCreature {
                        what: target_filtered(R::Land),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::SetBasePT {
                        what: Selector::Target(0),
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), g(), g(), g()]),
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: each_your_creature(),
                        power: Value::Const(3),
                        toughness: Value::Const(3),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: each_your_creature(),
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..creature(
            "Kamahl, Fist of Krosa",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            4,
            3,
        )
    }
}

/// Tempting Wurm — a cheap fatty that unloads everyone else's hand.
pub fn tempting_wurm() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::EachPlayerMayPutPermanentFromHand {
            filter: R::Artifact.or(R::Creature).or(R::Enchantment).or(R::Land),
            others_only: true,
        })],
        ..creature("Tempting Wurm", cost(&[generic(1), g()]), vec![CreatureType::Wurm], 5, 5)
    }
}

/// Weird Harvest — every player tutors up to X creatures.
pub fn weird_harvest() -> CardDefinition {
    sorcery(
        "Weird Harvest",
        cost(&[x(), g(), g()]),
        Effect::ForEach {
            selector: Selector::Player(PlayerRef::EachPlayer),
            body: Box::new(Effect::SearchUpToN {
                who: PlayerRef::Triggerer,
                filter: R::Creature,
                to: ZoneDest::Hand(PlayerRef::Triggerer),
                count: Value::XFromCost,
            }),
        },
    )
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Cryptic Gateway — two tapped creatures cheat a tribal sibling into play.
pub fn cryptic_gateway() -> CardDefinition {
    CardDefinition {
        name: "Cryptic Gateway",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((R::Creature, 2)),
            effect: Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Creature.and(R::SharesCreatureTypeWithTapped),
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Doom Cannon — a tribal sacrifice outlet that throws 3 damage.
pub fn doom_cannon() -> CardDefinition {
    CardDefinition {
        name: "Doom Cannon",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            sac_other_filter: Some((R::Creature.and(R::IsSourceChosenCreatureType), 1)),
            effect: Effect::DealDamage {
                to: target_any(),
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Supreme Inquisitor — five Wizards strip five cards from a library.
pub fn supreme_inquisitor() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((R::Creature.and(R::HasCreatureType(CreatureType::Wizard)), 5)),
            effect: Effect::SearchUpToN {
                who: PlayerRef::Target(0),
                filter: R::Any,
                to: ZoneDest::Exile,
                count: Value::Const(5),
            },
            ..Default::default()
        }],
        ..creature(
            "Supreme Inquisitor",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            3,
        )
    }
}

// ── "Choose a creature type" ────────────────────────────────────────────────

/// Bloodline Shaman — a tribal cantrip that bins its misses.
pub fn bloodline_shaman() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::ChooseCreatureTypeThen {
                who: PlayerRef::You,
                then: Box::new(Effect::RevealTopTakeMatchingRestToGraveyard {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    filter: R::Creature.and(R::IsSourceChosenCreatureType),
                }),
            },
            ..Default::default()
        }],
        ..creature(
            "Bloodline Shaman",
            cost(&[generic(1), g()]),
            vec![CreatureType::Elf, CreatureType::Wizard, CreatureType::Shaman],
            1,
            1,
        )
    }
}

/// Callous Oppressor — an opponent names the tribe it can't steal.
pub fn callous_oppressor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MayChooseNotToUntap],
        as_enters_effect: Some(Effect::NameCreatureTypeBy {
            what: Selector::This,
            who: PlayerRef::EachOpponent,
        }),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GainControlWhileSourceTapped {
                what: target_filtered(R::Creature.and(R::IsSourceChosenCreatureType.negate())),
            },
            ..Default::default()
        }],
        ..creature(
            "Callous Oppressor",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Octopus],
            1,
            2,
        )
    }
}

/// Harsh Mercy — every player saves one tribe; everything else dies.
pub fn harsh_mercy() -> CardDefinition {
    sorcery(
        "Harsh Mercy",
        cost(&[generic(2), w()]),
        Effect::EachPlayerChoosesCreatureTypeThen {
            then: Box::new(Effect::DestroyNoRegen {
                what: Selector::EachPermanent(R::Creature.and(R::IsTypeChosenThisWay.negate())),
            }),
        },
    )
}

/// Patriarch's Bidding — every player reanimates one tribe.
pub fn patriarchs_bidding() -> CardDefinition {
    sorcery(
        "Patriarch's Bidding",
        cost(&[generic(3), b(), b()]),
        Effect::EachPlayerChoosesCreatureTypeThen {
            then: Box::new(Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachPlayer),
                body: Box::new(Effect::Move {
                    what: Selector::CardsInZone {
                        who: PlayerRef::Triggerer,
                        zone: crate::card::Zone::Graveyard,
                        filter: R::Creature.and(R::IsTypeChosenThisWay),
                    },
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::OwnerOfMoved,
                        tapped: false,
                    },
                }),
            }),
        },
    )
}

/// Peer Pressure — control the biggest tribe and you take all of it.
pub fn peer_pressure() -> CardDefinition {
    sorcery(
        "Peer Pressure",
        cost(&[generic(3), u()]),
        Effect::ChooseCreatureTypeThen {
            who: PlayerRef::You,
            then: Box::new(Effect::If {
                cond: crate::effect::Predicate::PlayerControlsMostOf {
                    who: PlayerRef::You,
                    filter: R::Creature.and(R::IsSourceChosenCreatureType),
                },
                then: Box::new(Effect::GainControl {
                    what: Selector::EachPermanent(
                        R::Creature.and(R::IsSourceChosenCreatureType),
                    ),
                    to: Some(PlayerRef::You),
                    duration: Duration::Permanent,
                }),
                else_: Box::new(Effect::Noop),
            }),
        },
    )
}

/// Riptide Chronologist — untap a whole tribe at instant speed.
pub fn riptide_chronologist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            sac_cost: true,
            effect: Effect::ChooseCreatureTypeThen {
                who: PlayerRef::You,
                then: Box::new(Effect::Untap {
                    what: Selector::EachPermanent(
                        R::Creature.and(R::IsSourceChosenCreatureType),
                    ),
                    up_to: None,
                }),
            },
            ..Default::default()
        }],
        ..creature(
            "Riptide Chronologist",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            3,
        )
    }
}

/// Riptide Shapeshifter — trade a 3/3 for any creature in your deck.
pub fn riptide_shapeshifter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u(), u()]),
            sac_cost: true,
            effect: Effect::ChooseCreatureTypeThen {
                who: PlayerRef::You,
                then: Box::new(Effect::RevealUntilFind {
                    who: PlayerRef::You,
                    find: R::Creature.and(R::IsSourceChosenCreatureType),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                    cap: Value::Const(200),
                    life_per_revealed: 0,
                    miss_dest: crate::effect::RevealMissDest::ShuffleIntoLibrary,
                }),
            },
            ..Default::default()
        }],
        ..creature(
            "Riptide Shapeshifter",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Shapeshifter],
            3,
            3,
        )
    }
}

/// Walking Desecration — force a whole tribe into a bad attack.
pub fn walking_desecration() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            tap_cost: true,
            effect: Effect::ChooseCreatureTypeThen {
                who: PlayerRef::You,
                then: Box::new(Effect::GrantKeyword {
                    what: Selector::EachPermanent(
                        R::Creature.and(R::IsSourceChosenCreatureType),
                    ),
                    keyword: Keyword::MustAttack,
                    duration: Duration::EndOfTurn,
                }),
            },
            ..Default::default()
        }],
        ..creature(
            "Walking Desecration",
            cost(&[generic(2), b()]),
            vec![CreatureType::Zombie],
            1,
            1,
        )
    }
}

/// Endemic Plague — sacrifice one creature to sweep its whole tribe.
pub fn endemic_plague() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Creature,
            count: 1,
        }],
        ..sorcery(
            "Endemic Plague",
            cost(&[generic(3), b()]),
            Effect::DestroyNoRegen {
                what: Selector::EachPermanent(
                    R::Creature.and(R::SharesCreatureTypeWithSacrificed),
                ),
            },
        )
    }
}

// ── Wave C — the remaining utility shell ────────────────────────────────────

/// Backslide — a cycling trick that flips a morph back down.
pub fn backslide() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[u()]))],
        ..instant(
            "Backslide",
            cost(&[generic(1), u()]),
            Effect::TurnFaceDown { what: target_filtered(R::Creature.and(R::HasMorphAbility)) },
        )
    }
}

/// Shade's Breath — the whole team becomes pumpable Shades.
pub fn shades_breath() -> CardDefinition {
    let team = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    instant(
        "Shade's Breath",
        cost(&[generic(1), b()]),
        Effect::Seq(vec![
            Effect::BecomeColor {
                what: team(),
                colors: vec![crate::mana::Color::Black],
                duration: Duration::EndOfTurn,
                additive: false,
            },
            Effect::BecomeCreatureType {
                what: team(),
                creature_types: vec![CreatureType::Shade],
                duration: Duration::EndOfTurn,
            },
            Effect::GainActivatedAbility {
                what: team(),
                ability: Box::new(ActivatedAbility {
                    mana_cost: cost(&[b()]),
                    effect: Effect::PumpPT {
                        what: Selector::This,
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    },
                    ..Default::default()
                }),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Goblin Machinist — the top of your deck sets its power.
pub fn goblin_machinist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            effect: Effect::RevealUntilNonlandThen {
                then: Box::new(Effect::PumpPT {
                    what: Selector::This,
                    power: Value::LastRevealedManaValue,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                }),
            },
            ..Default::default()
        }],
        ..creature("Goblin Machinist", cost(&[generic(4), r()]), vec![CreatureType::Goblin], 0, 5)
    }
}

/// Kaboom! — one deck-flip of damage per chosen player or planeswalker.
pub fn kaboom() -> CardDefinition {
    sorcery(
        "Kaboom!",
        cost(&[generic(4), r()]),
        Effect::ApplyToTargets {
            max_targets: 4,
            min_targets: 0,
            filter: R::Player.or(R::Planeswalker),
            effect: Box::new(Effect::RevealUntilNonlandThen {
                then: Box::new(Effect::DealDamage {
                    to: Selector::Target(0),
                    amount: Value::LastRevealedManaValue,
                }),
            }),
        },
    )
}

/// Death Match — every creature that enters can shrink something.
pub fn death_match() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(crate::effect::Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::MayDoBy {
                who: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                description: "Give a creature -3/-3 until end of turn?".into(),
                body: Box::new(Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(-3),
                    toughness: Value::Const(-3),
                    duration: Duration::EndOfTurn,
                }),
            },
        }],
        ..enchantment("Death Match", cost(&[generic(3), b()]))
    }
}

/// Spy Network — read a hand, then set up your own draws.
pub fn spy_network() -> CardDefinition {
    instant(
        "Spy Network",
        cost(&[u()]),
        Effect::Seq(vec![
            Effect::LookAtHand { who: target_filtered(R::Player) },
            Effect::RearrangeTop { who: PlayerRef::You, amount: Value::Const(4) },
        ]),
    )
}

/// Aurification — creatures that hit you turn into gold-plated Walls.
pub fn aurification() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDamaged, EventScope::AnyPlayer)
                .with_filter(crate::effect::Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::Gold,
                amount: Value::ONE,
            },
        },
        TriggeredAbility {
            event: EventSpec::new(
                EventKind::PermanentLeavesBattlefield,
                EventScope::SelfSource,
            ),
            effect: Effect::RemoveCounter {
                what: Selector::EachPermanent(R::Creature.and(R::WithCounter(CounterType::Gold))),
                kind: CounterType::Gold,
                amount: Value::Const(99),
            },
        }],
        static_abilities: vec![
            StaticAbility {
                description: "Each creature with a gold counter on it is a Wall in addition to its other creature types.",
                effect: StaticEffect::AddCreatureTypeToMatching {
                    applies_to: Selector::EachPermanent(
                        R::Creature.and(R::WithCounter(CounterType::Gold)),
                    ),
                    creature_type: CreatureType::Wall,
                },
            },
            StaticAbility {
                description: "Each creature with a gold counter on it has defender.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        R::Creature.and(R::WithCounter(CounterType::Gold)),
                    ),
                    keyword: Keyword::Defender,
                },
            },
        ],

        ..enchantment("Aurification", cost(&[generic(2), w(), w()]))
    }
}
