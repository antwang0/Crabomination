//! Invasion (INV) gap-closing wave 4: the Dragon Legends, the last kicker
//! cards and the enchantment shell. Tests in `classic_sets/inv_gaps4`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, ZoneDest,
    shortcut::{draw, etb, target_filtered},
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

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

fn saproling() -> TokenDefinition {
    TokenDefinition {
        name: "Saproling".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Saproling], ..Default::default() },
        ..Default::default()
    }
}

/// Permanents of the source's ETB/resolution-chosen colour.
fn of_chosen_color(base: R) -> Selector {
    Selector::EachPermanent(base.and(R::HasChosenColorOfSource))
}

// ── The Dragon Legends ──────────────────────────────────────────────────────

/// The shared shape: "Whenever ~ deals combat damage to a player, you may pay
/// [cost]. If you do, choose a color, then [body]."
fn dragon_legend(
    name: &'static str,
    c: ManaCost,
    pay: ManaCost,
    prompt: &'static str,
    body: Effect,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: prompt.into(),
                mana_cost: pay,
                body: Box::new(Effect::Seq(vec![Effect::ChooseColorForSelf, body])),
                else_: None,
            },
        }],
        ..creature(name, c, vec![CreatureType::Dragon], 6, 6)
    }
}

/// Crosis, the Purger — {3}{U}{B}{R}. Strips the damaged player's colour.
pub fn crosis_the_purger() -> CardDefinition {
    dragon_legend(
        "Crosis, the Purger",
        cost(&[generic(3), u(), b(), r()]),
        cost(&[generic(2), b()]),
        "Pay {2}{B} to strip a colour from their hand?",
        Effect::RevealHandDiscardAllMatching {
            who: PlayerRef::TriggerEventPlayer,
            filter: R::HasChosenColorOfSource,
        },
    )
}

/// Dromar, the Banisher — {3}{W}{U}{B}. Bounces a whole colour.
pub fn dromar_the_banisher() -> CardDefinition {
    dragon_legend(
        "Dromar, the Banisher",
        cost(&[generic(3), w(), u(), b()]),
        cost(&[generic(2), u()]),
        "Pay {2}{U} to bounce a colour?",
        Effect::Move {
            what: of_chosen_color(R::Creature),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
    )
}

/// Treva, the Renewer — {3}{G}{W}{U}. Life per permanent of a colour.
pub fn treva_the_renewer() -> CardDefinition {
    dragon_legend(
        "Treva, the Renewer",
        cost(&[generic(3), g(), w(), u()]),
        cost(&[generic(2), w()]),
        "Pay {2}{W} to gain life per permanent of a colour?",
        Effect::GainLife {
            who: Selector::You,
            amount: Value::CountOf(Box::new(of_chosen_color(R::Permanent))),
        },
    )
}

/// Rith, the Awakener — {3}{R}{G}{W}. A Saproling per permanent of a colour.
pub fn rith_the_awakener() -> CardDefinition {
    dragon_legend(
        "Rith, the Awakener",
        cost(&[generic(3), r(), g(), w()]),
        cost(&[generic(2), g()]),
        "Pay {2}{G} to make Saprolings?",
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::CountOf(Box::new(of_chosen_color(R::Permanent))),
            definition: saproling(),
        },
    )
}

/// Darigaaz, the Igniter — {3}{B}{R}{G}. Burn per card of a colour in hand.
/// (The reveal is folded into the count — the hand is read live rather than
/// snapshotted, which is only visible if the hand changes mid-resolution.)
pub fn darigaaz_the_igniter() -> CardDefinition {
    dragon_legend(
        "Darigaaz, the Igniter",
        cost(&[generic(3), b(), r(), g()]),
        cost(&[generic(2), r()]),
        "Pay {2}{R} to burn for their colour count?",
        Effect::DealDamage {
            to: Selector::Player(PlayerRef::TriggerEventPlayer),
            amount: Value::CardsInHandMatching {
                who: PlayerRef::TriggerEventPlayer,
                filter: R::HasChosenColorOfSource,
            },
        },
    )
}

// ── The last kicker cards ───────────────────────────────────────────────────

/// Skizzik — {3}{R} 5/3 trampling haste that leaves unless kicked.
pub fn skizzik() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste, Keyword::Kicker(cost(&[r()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::Not(Box::new(Predicate::SpellWasKicked))),
            effect: Effect::AtNextEndStep {
                body: Box::new(Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::IsSource,
                }),
            },
        }],
        ..creature("Skizzik", cost(&[generic(3), r()]), vec![CreatureType::Elemental], 5, 3)
    }
}

/// Prison Barricade — {1}{W} 1/3 Wall. Kicked, it enters with a counter.
/// (The kicked "can attack as though it didn't have defender" rider needs a
/// defender-bypass grant the engine doesn't have — tracked in TODO.md.)
pub fn prison_barricade() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender, Keyword::Kicker(cost(&[generic(1), w()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ])),
            else_: Box::new(Effect::Noop),
        })],
        ..creature("Prison Barricade", cost(&[generic(1), w()]), vec![CreatureType::Wall], 1, 3)
    }
}

/// Probe — {2}{U}. Draw three and pitch two; kicked, they pitch two too.
pub fn probe() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(1), b()]))],
        ..sorcery(
            "Probe",
            cost(&[generic(2), u()]),
            Effect::Seq(vec![
                draw(3),
                Effect::Discard { who: Selector::You, amount: Value::Const(2), random: false },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::Discard {
                        who: Selector::Player(PlayerRef::Target(0)),
                        amount: Value::Const(2),
                        random: false,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Vigorous Charge — {G}. Trample, plus lifelink for the turn when kicked.
pub fn vigorous_charge() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[w()]))],
        ..instant(
            "Vigorous Charge",
            cost(&[g()]),
            Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Lifelink,
                        duration: Duration::EndOfTurn,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Thicket Elemental — {3}{G}{G} 4/4. Kicked, it digs out a free creature.
pub fn thicket_elemental() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(1), g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SpellWasKicked),
            effect: Effect::MayDo {
                description: "Reveal until a creature and put it onto the battlefield?".into(),
                body: Box::new(Effect::RevealUntilFind {
                    who: PlayerRef::You,
                    find: R::Creature,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    cap: Value::Const(60),
                    life_per_revealed: 0,
                    miss_dest: crate::effect::RevealMissDest::ShuffleIntoLibrary,
                }),
            },
        }],
        ..creature(
            "Thicket Elemental",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Elemental],
            4,
            4,
        )
    }
}

/// Verdeloth the Ancient — {4}{G}{G} 4/7 lord. Kicked, it brings X Saprolings.
pub fn verdeloth_the_ancient() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Saproling creatures and other Treefolk creatures get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(
                        R::HasCreatureType(CreatureType::Saproling).or(R::HasCreatureType(
                            CreatureType::Treefolk,
                        )
                        .and(R::OtherThanSource)),
                    ),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        keywords: vec![Keyword::Kicker(cost(&[x()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SpellWasKicked),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::XFromCost,
                definition: saproling(),
            },
        }],
        ..creature(
            "Verdeloth the Ancient",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Treefolk],
            4,
            7,
        )
    }
}

/// Kangee, Aerie Keeper — {2}{W}{U} 2/2 Bird lord scaled by its kicked X.
/// (Feather counters are modelled as charge counters — the engine has no
/// dedicated kind and nothing else reads them.)
pub fn kangee_aerie_keeper() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Kicker(cost(&[x(), generic(2)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SpellWasKicked),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Charge,
                amount: Value::XFromCost,
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "Other Bird creatures get +1/+1 for each feather counter on Kangee.",
            effect: StaticEffect::PumpPTPerCounterOnSource {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::HasCreatureType(CreatureType::Bird))
                        .and(R::OtherThanSource),
                ),
                kind: CounterType::Charge,
                per_power: 1,
                per_toughness: 1,
            },
        }],
        ..creature(
            "Kangee, Aerie Keeper",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Bird, CreatureType::Wizard],
            2,
            2,
        )
    }
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Devouring Strossus — {5}{B}{B}{B} 9/9 that eats a creature every upkeep.
pub fn devouring_strossus() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Sacrifice {
                who: Selector::You,
                count: Value::ONE,
                filter: R::Creature,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature(
            "Devouring Strossus",
            cost(&[generic(5), b(), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Horror],
            9,
            9,
        )
    }
}

/// Pyre Zombie — {1}{B}{R} 2/1 that buys itself back and goes out shooting.
pub fn pyre_zombie() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::FromYourGraveyard,
            ),
            effect: Effect::MayPay {
                description: "Pay {1}{B}{B} to return Pyre Zombie to your hand?".into(),
                mana_cost: cost(&[generic(1), b(), b()]),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
                else_: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r(), r()]),
            sac_cost: true,
            effect: Effect::DealDamage {
                to: crate::effect::shortcut::target_any(),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..creature("Pyre Zombie", cost(&[generic(1), b(), r()]), vec![CreatureType::Zombie], 2, 1)
    }
}

/// Metathran Transport — {1}{U}{U} 1/3 flier that dodges and recolours.
pub fn metathran_transport() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Flying,
            Keyword::CantBeBlockedBy(Box::new(R::HasColor(Color::Blue))),
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::BecomeColor {
                what: target_filtered(R::Creature),
                colors: vec![Color::Blue],
                duration: Duration::EndOfTurn,
                additive: false,
            },
            ..Default::default()
        }],
        ..creature(
            "Metathran Transport",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Metathran],
            1,
            3,
        )
    }
}

/// Phyrexian Infiltrator — {2}{B} 2/2 that trades itself for something better.
pub fn phyrexian_infiltrator() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u(), u()]),
            effect: Effect::ExchangeControl {
                a: Selector::This,
                b: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..creature(
            "Phyrexian Infiltrator",
            cost(&[generic(2), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Minion],
            2,
            2,
        )
    }
}

/// Hunting Kavu — {1}{R}{G} 2/3 that trades itself for a ground attacker.
pub fn hunting_kavu() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r(), g()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Exile { what: Selector::This },
                Effect::Exile {
                    what: target_filtered(
                        R::Creature
                            .and(R::IsAttacking)
                            .and(R::HasKeyword(Keyword::Flying).negate()),
                    ),
                },
            ]),
            ..Default::default()
        }],
        ..creature("Hunting Kavu", cost(&[generic(1), r(), g()]), vec![CreatureType::Kavu], 2, 3)
    }
}

/// Slimy Kavu — {2}{R} 2/2. {T}: a land is a Swamp for the turn.
pub fn slimy_kavu() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GainLandType {
                what: target_filtered(R::Land),
                land_type: crate::card::LandType::Swamp,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Slimy Kavu", cost(&[generic(2), r()]), vec![CreatureType::Kavu], 2, 2)
    }
}

/// Vodalian Hypnotist — {1}{U} 1/1. A sorcery-speed discard tap.
pub fn vodalian_hypnotist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
                random: false,
            },
            ..Default::default()
        }],
        ..creature(
            "Vodalian Hypnotist",
            cost(&[generic(1), u()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Tek — {5} 2/2 artifact Dragon whose stat line reads your basic types.
pub fn tek() -> CardDefinition {
    use crate::card::LandType;
    let controls = |lt: LandType| {
        Predicate::SelectorExists(Selector::EachPermanent(
            R::HasLandType(lt).and(R::ControlledByYou),
        ))
    };
    let clause = |lt: LandType, p: i32, t: i32, kws: Vec<Keyword>| StaticAbility {
        description: "Tek's bonuses track the basic land types you control.",
        effect: StaticEffect::PumpSelfIf {
            condition: controls(lt),
            power: p,
            toughness: t,
            keywords: kws,
        },
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![
            clause(LandType::Plains, 0, 2, vec![]),
            clause(LandType::Island, 0, 0, vec![Keyword::Flying]),
            clause(LandType::Swamp, 2, 0, vec![]),
            clause(LandType::Mountain, 0, 0, vec![Keyword::FirstStrike]),
            clause(LandType::Forest, 0, 0, vec![Keyword::Trample]),
        ],
        ..creature("Tek", cost(&[generic(5)]), vec![CreatureType::Dragon], 2, 2)
    }
}

// ── Enchantments ────────────────────────────────────────────────────────────

/// Teferi's Moat — {3}{W}{U}. A colour can't attack you on the ground.
pub fn teferis_moat() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        static_abilities: vec![StaticAbility {
            description: "Creatures of the chosen color without flying can't attack you.",
            effect: StaticEffect::CreaturesCantAttackController {
                filter: Some(
                    R::HasChosenColorOfSource.and(R::HasKeyword(Keyword::Flying).negate()),
                ),
                protect_planeswalkers: false,
            },
        }],
        ..enchantment("Teferi's Moat", cost(&[generic(3), w(), u()]))
    }
}

/// Teferi's Care — {2}{W}. Trades enchantments, or counters one outright.
pub fn teferis_care() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                sac_other_filter: Some((R::Enchantment, 1)),
                effect: Effect::Destroy { what: target_filtered(R::Enchantment) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), u(), u()]),
                effect: Effect::CounterSpell {
                    what: target_filtered(R::IsSpellOnStack.and(R::Enchantment)),
                },
                ..Default::default()
            },
        ],
        ..enchantment("Teferi's Care", cost(&[generic(2), w()]))
    }
}

/// Spirit of Resistance — {2}{W}. Five colours out means you take no damage.
pub fn spirit_of_resistance() -> CardDefinition {
    let of = |k: Color| {
        Predicate::SelectorExists(Selector::EachPermanent(
            R::HasColor(k).and(R::ControlledByYou),
        ))
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as you control a permanent of each color, prevent all damage \
                          that would be dealt to you.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::All(vec![
                    of(Color::White),
                    of(Color::Blue),
                    of(Color::Black),
                    of(Color::Red),
                    of(Color::Green),
                ]),
                inner: Box::new(StaticEffect::PreventAllDamageToController),
            },
        }],
        ..enchantment("Spirit of Resistance", cost(&[generic(2), w()]))
    }
}

/// Tectonic Instability — {2}{R}. Every land drop taps its controller out.
pub fn tectonic_instability() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Land,
                }),
            effect: Effect::Tap {
                what: Selector::ControlledBy {
                    who: PlayerRef::TriggerEventPlayer,
                    filter: R::Land,
                },
            },
        }],
        ..enchantment("Tectonic Instability", cost(&[generic(2), r()]))
    }
}

/// Saproling Infestation — {1}{G}. A Saproling every time anyone kicks.
pub fn saproling_infestation() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::CastSpellWasKicked),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: saproling(),
            },
        }],
        ..enchantment("Saproling Infestation", cost(&[generic(1), g()]))
    }
}

/// Saproling Symbiosis — {3}{G}. A Saproling per creature you control.
pub fn saproling_symbiosis() -> CardDefinition {
    CardDefinition {
        flash_surcharge: Some(cost(&[generic(2)])),
        ..sorcery(
        "Saproling Symbiosis",
        cost(&[generic(3), g()]),
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::CountMatching {
                sel: Box::new(Selector::EachPermanent(R::ControlledByYou)),
                filter: R::Creature,
            },
            definition: saproling(),
        },
    )
    }
}

/// Seer's Vision — {2}{U}{B}. Open hands, and one surgical discard.
pub fn seers_vision() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Your opponents play with their hands revealed.",
            effect: StaticEffect::OpponentsPlayWithHandsRevealed,
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::LookAtHand { who: Selector::Player(PlayerRef::Target(0)) },
                Effect::Discard {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::ONE,
                    random: false,
                },
            ]),
            ..Default::default()
        }],
        ..enchantment("Seer's Vision", cost(&[generic(2), u(), b()]))
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────


// ── The "pay {2} more for flash" cycle (CR 601.2b) ──────────────────────────

/// Ghitu Fire — {X}{R}. X damage anywhere, at instant speed for {2} more.
pub fn ghitu_fire() -> CardDefinition {
    CardDefinition {
        flash_surcharge: Some(cost(&[generic(2)])),
        ..sorcery(
            "Ghitu Fire",
            cost(&[x(), r()]),
            Effect::DealDamage {
                to: crate::effect::shortcut::target_any(),
                amount: Value::XFromCost,
            },
        )
    }
}

/// Breaking Wave — {2}{U}{U}. Every creature flips its tapped state.
pub fn breaking_wave() -> CardDefinition {
    CardDefinition {
        flash_surcharge: Some(cost(&[generic(2)])),
        ..sorcery(
            "Breaking Wave",
            cost(&[generic(2), u(), u()]),
            Effect::SwapTappedState { what: Selector::EachPermanent(R::Creature) },
        )
    }
}


/// Void — {3}{B}{R}. Name a number; wipe it off the board and out of a hand.
pub fn void() -> CardDefinition {
    sorcery(
        "Void",
        cost(&[generic(3), b(), r()]),
        Effect::PlayerChoosesNumber {
            who: Selector::You,
            prompt: "Choose a mana value".into(),
            max: Value::Const(15),
            then: Box::new(Effect::Seq(vec![
                Effect::DestroyEachMatchingWithManaValue {
                    filter: R::Artifact.or(R::Creature),
                    value: Value::ChosenNumber,
                },
                Effect::RevealHandDiscardAllMatching {
                    who: PlayerRef::Target(0),
                    filter: R::Nonland.and(R::ManaValueEqualsChosenNumber),
                },
            ])),
        },
    )
}
