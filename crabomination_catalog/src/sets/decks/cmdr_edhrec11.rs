//! Commander: the three-card EDHREC gaps of the top 600, eleventh file
//! (`cmdr_edhrec10` is the tenth). Tests in `tests/recent_b/cmdr_edhrec11.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, LoyaltyAbility, PlaneswalkerSubtype, Predicate, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_dies, target_filtered};
use crate::effect::{ActivatedAbility, Duration, Effect, ManaPayload, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, x, Color, ManaCost};
use std::sync::Arc;
fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn static_ab(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

fn token(name: &str, p: i32, t: i32, color: Color, types: Vec<CreatureType>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        colors: vec![color],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn make(n: i32, def: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: Arc::new(def) }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn at_your(step: TurnStep, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl), effect }
}

fn mine() -> R {
    R::Creature.and(R::ControlledByYou)
}


// ── Goro-Goro and Satoru ─────────────────────────────────────────────────────

/// Alora, Merry Thief — {2}{U} 3/2 Halfling Rogue, choose a Background.
/// Whenever you attack, up to one attacker can't be blocked this turn and
/// returns to hand at the next end step.
pub fn alora_merry_thief() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::ChooseABackground],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
            effect: Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: target_filtered(R::Creature.and(R::IsAttacking)),
                        keyword: Keyword::Unblockable,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::AtNextEndStep {
                        body: Box::new(Effect::Move {
                            what: Selector::Target(0),
                            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                        }),
                    },
                ])),
            },
        }],
        ..legend("Alora, Merry Thief", cost(&[generic(2), u()]), vec![CreatureType::Halfling, CreatureType::Rogue], 3, 2)
    }
}

/// Sakashima's Student — {2}{U}{U} 0/0 Human Ninja, ninjutsu {1}{U}. May
/// enter as a copy of any creature, a Ninja too.
pub fn sakashimas_student() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(1), u()]))],
        enters_as_copy: Some(crate::card::EntersAsCopy {
            filter: R::Creature,
            extra_creature_types: vec![CreatureType::Ninja],
            ..Default::default()
        }),
        ..creature("Sakashima's Student", cost(&[generic(2), u(), u()]), vec![CreatureType::Human, CreatureType::Ninja], 0, 0)
    }
}

/// Daring Piracy — {2}{R} Enchantment. Each of your combats makes a 1/1
/// menace, haste Pirate exiled at the next end step.
pub fn daring_piracy() -> CardDefinition {
    let pirate = TokenDefinition {
        keywords: vec![Keyword::Menace, Keyword::Haste],
        ..token("Pirate", 1, 1, Color::Red, vec![CreatureType::Pirate])
    };
    CardDefinition {
        name: "Daring Piracy",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![at_your(
            TurnStep::BeginCombat,
            Effect::Seq(vec![make(1, pirate), Effect::ExileAtNextEndStep { what: Selector::LastCreatedTokens }]),
        )],
        ..Default::default()
    }
}

// ── Neriv, Heart of the Storm ────────────────────────────────────────────────

/// Duke Ulder Ravengard — {4}{R}{W} 5/5 Human Noble Soldier. Each of your
/// combats, another creature of yours gains haste and myriad.
pub fn duke_ulder_ravengard() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![at_your(
            TurnStep::BeginCombat,
            Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(mine().and(R::OtherThanSource)),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantTriggeredAbility {
                    what: Selector::Target(0),
                    trigger: Box::new(TriggeredAbility {
                        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                        effect: Effect::Myriad,
                    }),
                    duration: Duration::EndOfTurn,
                },
            ]),
        )],
        ..legend(
            "Duke Ulder Ravengard",
            cost(&[generic(4), r(), w()]),
            vec![CreatureType::Human, CreatureType::Noble, CreatureType::Soldier],
            5,
            5,
        )
    }
}

/// Red Dragon — {4}{R}{R} 4/4 flying Dragon. ETB 4 damage to each opponent.
pub fn red_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(4),
        })],
        ..creature("Red Dragon", cost(&[generic(4), r(), r()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Nahiri's Resolve — {3}{R}{W} Enchantment. Your creatures get +1/+0 and
/// haste; your end step blinks any number of your nontoken artifacts and
/// creatures until your next upkeep.
pub fn nahiris_resolve() -> CardDefinition {
    CardDefinition {
        name: "Nahiri's Resolve",
        cost: cost(&[generic(3), r(), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            static_ab("Creatures you control get +1/+0.", StaticEffect::PumpPT { applies_to: Selector::EachPermanent(mine()), power: 1, toughness: 0 }),
            static_ab(
                "Creatures you control have haste.",
                StaticEffect::GrantKeyword { applies_to: Selector::EachPermanent(mine()), keyword: Keyword::Haste },
            ),
        ],
        triggered_abilities: vec![at_your(
            TurnStep::End,
            Effect::ChooseSomeAmong {
                what: Selector::EachPermanent(
                    R::Artifact.or(R::Creature).and(R::ControlledByYou).and(R::IsToken.negate()),
                ),
                chooser: PlayerRef::You,
                count: Value::Const(99),
                up_to: true,
                chosen: Box::new(Effect::ExileReturnAtYourNextUpkeep { what: Selector::SeparatedPile { chosen: true } }),
                other: Box::new(Effect::Noop),
            },
        )],
        ..Default::default()
    }
}

// ── Xavier Sal, Infested Captain ─────────────────────────────────────────────

/// Xavier Sal, Infested Captain — {B}{G}{U} 3/3 Human Fungus Pirate. {T},
/// remove a counter from another permanent of yours: populate; {T},
/// sacrifice another creature: proliferate. Both as sorceries.
pub fn xavier_sal_infested_captain() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                sorcery_speed: true,
                remove_counter_among_filter: Some((None, 1, R::Permanent.and(R::OtherThanSource))),
                effect: Effect::Populate { who: PlayerRef::You },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sorcery_speed: true,
                sac_other_filter: Some((R::Creature.and(R::OtherThanSource), 1)),
                effect: Effect::Proliferate,
                ..Default::default()
            },
        ],
        ..legend(
            "Xavier Sal, Infested Captain",
            cost(&[b(), g(), u()]),
            vec![CreatureType::Human, CreatureType::Fungus, CreatureType::Pirate],
            3,
            3,
        )
    }
}

fn spore_thallid(name: &'static str, mana: ManaCost, types: Vec<CreatureType>) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![at_your(
            TurnStep::Upkeep,
            Effect::AddCounter { what: Selector::This, kind: CounterType::Spore, amount: Value::ONE },
        )],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Spore, 3)),
            effect: make(1, token("Saproling", 1, 1, Color::Green, vec![CreatureType::Saproling])),
            ..Default::default()
        }],
        ..creature(name, mana, types, 1, 1)
    }
}

/// Thallid — {G} 1/1 Fungus. Spore counters each upkeep; three make a
/// Saproling.
pub fn thallid() -> CardDefinition {
    spore_thallid("Thallid", cost(&[g()]), vec![CreatureType::Fungus])
}

/// Deathspore Thallid — {1}{B} 1/1 Zombie Fungus. Thallid's spores, and
/// sacrificing a Saproling gives a creature -1/-1.
pub fn deathspore_thallid() -> CardDefinition {
    let mut d = spore_thallid("Deathspore Thallid", cost(&[generic(1), b()]), vec![CreatureType::Zombie, CreatureType::Fungus]);
    d.activated_abilities.push(ActivatedAbility {
        sac_other_filter: Some((R::HasCreatureType(CreatureType::Saproling), 1)),
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    });
    d
}

// ── Myrkul, Lord of Bones ────────────────────────────────────────────────────

/// Myrkul, Lord of Bones — {4}{W}{B}{G} 7/5 God. Indestructible at half your
/// starting life or less; another nontoken creature of yours dying may be
/// exiled for an enchantment-only token copy of it.
pub fn myrkul_lord_of_bones() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "As long as your life total is less than or equal to half your starting life total, Myrkul has indestructible.",
            StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Indestructible,
                condition: Predicate::PlayerLifeAtMostHalfStarting { who: PlayerRef::You },
            },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::IsToken.negate().and(R::OtherThanSource),
            }),
            effect: Effect::MayDo {
                description: "Exile it for an enchantment token copy?".into(),
                body: Box::new(Effect::Seq(vec![
                    crate::effect::shortcut::token_copy_of(PlayerRef::You, Value::ONE, Selector::TriggerSource),
                    Effect::AmendCopiableValues {
                        what: Selector::LastCreatedToken,
                        name: None,
                        set_creature_types: None,
                        add_creature_types: vec![],
                        legendary: false,
                        add_colors: vec![],
                        set_card_types: Some(vec![CardType::Enchantment]),
                    },
                    Effect::Exile { what: Selector::TriggerSource },
                ])),
            },
        }],
        ..legend("Myrkul, Lord of Bones", cost(&[generic(4), w(), b(), g()]), vec![CreatureType::God], 7, 5)
    }
}

fn untap_for_a_counter() -> ActivatedAbility {
    ActivatedAbility {
        add_counter_cost: Some((CounterType::MinusOneMinusOne, 1)),
        effect: Effect::Untap { what: Selector::This, up_to: None },
        ..Default::default()
    }
}

/// Barrenton Medic — {4}{W} 0/4 Kithkin Cleric. {T}: prevent the next 1
/// damage to any target; a -1/-1 counter untaps it.
pub fn barrenton_medic() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::PreventNextDamage { target: crate::effect::shortcut::target_any(), amount: Value::ONE },
                ..Default::default()
            },
            untap_for_a_counter(),
        ],
        ..creature("Barrenton Medic", cost(&[generic(4), w()]), vec![CreatureType::Kithkin, CreatureType::Cleric], 0, 4)
    }
}

/// Cinderhaze Wretch — {4}{B} 3/2 Elemental Shaman. {T}: a target player
/// discards (your turn only); a -1/-1 counter untaps it.
pub fn cinderhaze_wretch() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
                effect: Effect::Discard { who: target_filtered(R::Player), amount: Value::ONE, random: false },
                ..Default::default()
            },
            untap_for_a_counter(),
        ],
        ..creature("Cinderhaze Wretch", cost(&[generic(4), b()]), vec![CreatureType::Elemental, CreatureType::Shaman], 3, 2)
    }
}

// ── The Beamtown Bullies ─────────────────────────────────────────────────────

/// Despotic Scepter — {1} Artifact. {T}: destroy a permanent you own; it
/// can't be regenerated.
pub fn despotic_scepter() -> CardDefinition {
    CardDefinition {
        name: "Despotic Scepter",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::CantBeRegeneratedThisTurn { what: target_filtered(R::Permanent.and(R::OwnedByYou)) },
                Effect::Destroy { what: Selector::Target(0) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Boldwyr Heavyweights — {2}{R}{R} 8/8 trample Giant Warrior. ETB each
/// opponent may tutor a creature onto the battlefield.
pub fn boldwyr_heavyweights() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::EachPlayerDoes {
            who: PlayerRef::EachOpponent,
            body: Box::new(Effect::MayDo {
                description: "Search your library for a creature card to put onto the battlefield?".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Creature,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            }),
        })],
        ..creature("Boldwyr Heavyweights", cost(&[generic(2), r(), r()]), vec![CreatureType::Giant, CreatureType::Warrior], 8, 8)
    }
}

/// Hellcarver Demon — {3}{B}{B}{B} 6/6 flying Demon. Its combat damage to a
/// player sacrifices everything else and your hand, then casts any number of
/// spells from your top six free.
pub fn hellcarver_demon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::SacrificeAllMatching { who: Selector::You, filter: R::Permanent.and(R::OtherThanSource) },
                Effect::Discard { who: Selector::You, amount: Value::HandSizeOf(PlayerRef::You), random: false },
                Effect::ExileTopOfLibrary { who: Selector::You, amount: Value::Const(6), link_to_source: false, face_down: false },
                Effect::CastAnyOrderWithoutPaying {
                    what: Selector::ExiledThisResolution { filter: R::Nonland },
                    source_zone: Zone::Exile,
                    filter: None,
                    cap: None,
                    total_mana_value: None,
                    exile_after: false,
                },
            ]),
        }],
        ..creature("Hellcarver Demon", cost(&[generic(3), b(), b(), b()]), vec![CreatureType::Demon], 6, 6)
    }
}

// ── Kwain, Itinerant Meddler ─────────────────────────────────────────────────

/// Folio of Fancies — {1}{U} Artifact — Book. No maximum hand sizes; {X}{X},
/// {T}: each player draws X; {2}{U}, {T}: each opponent mills their hand
/// size.
pub fn folio_of_fancies() -> CardDefinition {
    CardDefinition {
        name: "Folio of Fancies",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Book], ..Default::default() },
        static_abilities: vec![static_ab("Players have no maximum hand size.", StaticEffect::AllPlayersNoMaximumHandSize)],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[x(), x()]),
                tap_cost: true,
                effect: Effect::Draw { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::XFromCost },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), u()]),
                tap_cost: true,
                effect: Effect::ForEach {
                    selector: Selector::Player(PlayerRef::EachOpponent),
                    body: Box::new(Effect::Mill {
                        who: Selector::Player(PlayerRef::Triggerer),
                        amount: Value::HandSizeOf(PlayerRef::Triggerer),
                    }),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Flumph — {1}{W} 0/4 defender, flying Jellyfish. Damage to it draws you and
/// a target opponent a card.
pub fn flumph() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::Seq(vec![draw(1), Effect::Draw { who: target_filtered(R::OpponentPlayer), amount: Value::ONE }]),
        }],
        ..creature("Flumph", cost(&[generic(1), w()]), vec![CreatureType::Jellyfish], 0, 4)
    }
}

/// The Council of Four — {3}{W}{U} 0/8 Human Noble. A player's second draw
/// on their turn draws you a card; their second spell on their turn makes
/// you a 2/2 Knight.
pub fn the_council_of_four() -> CardDefinition {
    let their_turn = || Predicate::IsTurnOf(PlayerRef::Triggerer);
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::AnyPlayer).with_filter(their_turn()),
                effect: draw(1),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(Predicate::All(vec![
                    their_turn(),
                    Predicate::SpellsCastThisTurnEquals { who: PlayerRef::Triggerer, count: Value::Const(2) },
                ])),
                effect: make(1, token("Knight", 2, 2, Color::White, vec![CreatureType::Knight])),
            },
        ],
        ..legend("The Council of Four", cost(&[generic(3), w(), u()]), vec![CreatureType::Human, CreatureType::Noble], 0, 8)
    }
}

// ── Ruric Thar, the Unbowed ──────────────────────────────────────────────────

/// Prowling Serpopard — {1}{G}{G} 4/3 Cat Snake, can't be countered; your
/// creature spells can't be countered.
pub fn prowling_serpopard() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeCountered],
        static_abilities: vec![static_ab(
            "Creature spells you control can't be countered.",
            StaticEffect::SpellsUncounterable { filter: R::Creature.and(R::ControlledByYou) },
        )],
        ..creature("Prowling Serpopard", cost(&[generic(1), g(), g()]), vec![CreatureType::Cat, CreatureType::Snake], 4, 3)
    }
}

/// Shaman of Forgotten Ways — {2}{G} 2/3 Human Shaman. {T}: two mana in any
/// colors for creature spells; formidable — {9}{G}{G}, {T}: each player's
/// life becomes their creature count.
pub fn shaman_of_forgotten_ways() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyColors(Value::Const(2))),
                        crate::mana::SpendRestriction::CreatureOnly,
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(9), g(), g()]),
                tap_cost: true,
                condition: Some(Predicate::FormidableActive { who: PlayerRef::You }),
                effect: Effect::ForEach {
                    selector: Selector::Player(PlayerRef::EachPlayer),
                    body: Box::new(Effect::SetLifeTotal {
                        who: Selector::Player(PlayerRef::Triggerer),
                        amount: Value::PermanentCountControlledByMatching(PlayerRef::Triggerer, R::Creature),
                    }),
                },
                ..Default::default()
            },
        ],
        ..creature("Shaman of Forgotten Ways", cost(&[generic(2), g()]), vec![CreatureType::Human, CreatureType::Shaman], 2, 3)
    }
}


// ── Zimone and Dina, Selvala ─────────────────────────────────────────────────

/// Scryb Ranger — {1}{G} 1/1 flash, flying Faerie Ranger, protection from
/// blue. Return a Forest you control to hand: untap a creature, once each
/// turn.
pub fn scryb_ranger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying, Keyword::Protection(Color::Blue)],
        activated_abilities: vec![ActivatedAbility {
            return_permanent_cost: Some((R::HasLandType(LandType::Forest).and(R::ControlledByYou), 1)),
            once_per_turn: true,
            effect: Effect::Untap { what: target_filtered(R::Creature), up_to: None },
            ..Default::default()
        }],
        ..creature("Scryb Ranger", cost(&[generic(1), g()]), vec![CreatureType::Faerie, CreatureType::Ranger], 1, 1)
    }
}

/// Zimone and Dina — {B}{G}{U} 3/4 Human Dryad. Your second draw each turn
/// drains a target opponent for 2; {T}, sacrifice another creature: draw and
/// may drop a land tapped, twice with eight lands.
pub fn zimone_and_dina() -> CardDefinition {
    let dig = || Effect::Seq(vec![draw(1), Effect::PutLandsFromHandOntoBattlefieldTapped { count: Value::ONE }]);
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::LoseLife { who: target_filtered(R::OpponentPlayer), amount: Value::Const(2) },
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Creature.and(R::OtherThanSource), 1)),
            effect: Effect::Seq(vec![
                dig(),
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::PermanentCountControlledByMatching(PlayerRef::You, R::Land),
                        Value::Const(8),
                    ),
                    then: Box::new(dig()),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..legend("Zimone and Dina", cost(&[b(), g(), u()]), vec![CreatureType::Human, CreatureType::Dryad], 3, 4)
    }
}

/// Elven Raft-Steerer — {2}{U} 3/2 Elf Pilot. Landfall: tap an opponent's
/// creature or untap one of yours.
pub fn elven_raft_steerer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
            effect: Effect::ChooseMode(vec![
                Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
                Effect::Untap { what: target_filtered(mine()), up_to: None },
            ]),
        }],
        ..creature("Elven Raft-Steerer", cost(&[generic(2), u()]), vec![CreatureType::Elf, CreatureType::Pilot], 3, 2)
    }
}

/// Woodland Bellower — {4}{G}{G} 6/5 Beast. ETB may tutor a nonlegendary
/// green creature with mana value 3 or less onto the battlefield.
pub fn woodland_bellower() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search for a nonlegendary green creature with mana value 3 or less?".into(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::Creature
                    .and(R::HasColor(Color::Green))
                    .and(R::HasSupertype(Supertype::Legendary).negate())
                    .and(R::ManaValueAtMost(3)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
        })],
        ..creature("Woodland Bellower", cost(&[generic(4), g(), g()]), vec![CreatureType::Beast], 6, 5)
    }
}

/// Shared Summons — {3}{G}{G} Instant. Tutor up to two creature cards to
/// hand.
pub fn shared_summons() -> CardDefinition {
    spell(
        "Shared Summons",
        cost(&[generic(3), g(), g()]),
        CardType::Instant,
        // "with different names" — the second pick differs from the first.
        Effect::Seq(vec![
            Effect::ClearLastMoved,
            Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::Creature.and(R::NameDiffersFromLastMoved),
                to: ZoneDest::Hand(PlayerRef::You),
                count: Value::Const(2),
            },
        ]),
    )
}

// ── Brudiclad, Riku ──────────────────────────────────────────────────────────

/// Worldwalker Helm — {2}{U} Artifact. Artifact token creations add a Map;
/// {1}{U}, {T}: copy an artifact token of yours.
pub fn worldwalker_helm() -> CardDefinition {
    CardDefinition {
        name: "Worldwalker Helm",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![static_ab(
            "If you would create one or more artifact tokens, instead create those tokens plus an additional Map token.",
            StaticEffect::ArtifactTokenCreationAddsToken { definition: crabomination_base::tokens::map_token() },
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            tap_cost: true,
            effect: crate::effect::shortcut::token_copy_of(
                PlayerRef::You,
                Value::ONE,
                target_filtered(R::Artifact.and(R::IsToken).and(R::ControlledByYou)),
            ),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// See Double — {2}{U}{U} Instant, can't be copied. Copy a spell or copy a
/// creature; both when an opponent has eight or more graveyard cards.
pub fn see_double() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeCopied],
        modes_widen: Some(crate::card::ModesWiden::If(Predicate::ForAnyPlayer {
            who: PlayerRef::EachOpponent,
            pred: Box::new(Predicate::ValueAtLeast(
                Value::CardsInGraveyardMatching { who: PlayerRef::Triggerer, filter: R::Any },
                Value::Const(8),
            )),
        })),
        ..spell(
            "See Double",
            cost(&[generic(2), u(), u()]),
            CardType::Instant,
            Effect::ChooseModesCast {
                modes: vec![
                    Effect::CopySpellMayChooseTargets { what: target_filtered(R::IsSpellOnStack), count: Value::ONE },
                    crate::effect::shortcut::token_copy_of(PlayerRef::You, Value::ONE, target_filtered(R::Creature)),
                ],
                min: 1,
                max: 1,
                allow_repeats: false,
            },
        )
    }
}

/// Quasiduplicate — {1}{U}{U} Sorcery, jump-start. Copy a creature you
/// control.
pub fn quasiduplicate() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::JumpStart],
        ..spell(
            "Quasiduplicate",
            cost(&[generic(1), u(), u()]),
            CardType::Sorcery,
            crate::effect::shortcut::token_copy_of(PlayerRef::You, Value::ONE, target_filtered(mine())),
        )
    }
}

/// Twinferno — {1}{R} Instant. Copy your next instant or sorcery this turn,
/// or give a creature of yours double strike.
pub fn twinferno() -> CardDefinition {
    spell(
        "Twinferno",
        cost(&[generic(1), r()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::OnYourNextInstantSorceryThisTurn {
                body: Box::new(Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE }),
            },
            Effect::GrantKeyword { what: target_filtered(mine()), keyword: Keyword::DoubleStrike, duration: Duration::EndOfTurn },
        ]),
    )
}

/// Titania's Command — {4}{G}{G} Sorcery. Choose two: exile a player's
/// graveyard for life; two lands tapped; two 2/2 Bears; two +1/+1 counters
/// on each of your creatures.
pub fn titanias_command() -> CardDefinition {
    spell(
        "Titania's Command",
        cost(&[generic(4), g(), g()]),
        CardType::Sorcery,
        Effect::ChooseModesCast {
            modes: vec![
                Effect::TargetPlayerThen {
                    filter: R::Player,
                    then: Box::new(Effect::Seq(vec![
                        Effect::GainLife {
                            who: Selector::You,
                            amount: Value::CardsInGraveyardMatching { who: PlayerRef::Target(0), filter: R::Any },
                        },
                        Effect::Move {
                            what: Selector::CardsInZone { who: PlayerRef::Target(0), zone: Zone::Graveyard, filter: R::Any },
                            to: ZoneDest::Exile,
                        },
                    ])),
                },
                Effect::SearchUpToN {
                    who: PlayerRef::You,
                    filter: R::Land,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    count: Value::Const(2),
                },
                make(2, token("Bear", 2, 2, Color::Green, vec![CreatureType::Bear])),
                Effect::AddCounter { what: Selector::EachPermanent(mine()), kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
            ],
            min: 2,
            max: 2,
            allow_repeats: false,
        },
    )
}

// ── Narset, Enlightened Master ───────────────────────────────────────────────

/// Karn's Temporal Sundering — {4}{U}{U} legendary Sorcery. A player takes an
/// extra turn; bounce up to one nonland permanent; exile this.
pub fn karns_temporal_sundering() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        exile_on_resolve: true,
        ..spell(
            "Karn's Temporal Sundering",
            cost(&[generic(4), u(), u()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::TargetPlayerThen {
                    filter: R::Player,
                    then: Box::new(Effect::TakeExtraTurn { who: PlayerRef::Target(0), count: Value::ONE }),
                },
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Move {
                        what: Selector::TargetFiltered { slot: 1, filter: R::Permanent.and(R::Nonland) },
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(1)))),
                    }),
                },
            ]),
        )
    }
}

/// Temporal Trespass — {8}{U}{U}{U} Sorcery, delve. Take an extra turn; exile
/// this.
pub fn temporal_trespass() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Delve],
        exile_on_resolve: true,
        ..spell(
            "Temporal Trespass",
            cost(&[generic(8), u(), u(), u()]),
            CardType::Sorcery,
            Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE },
        )
    }
}

/// Walk the Aeons — {4}{U}{U} Sorcery, buyback—sacrifice three Islands. A
/// target player takes an extra turn.
pub fn walk_the_aeons() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Buyback(ManaCost::new(vec![]))],
        buyback_additional_cost: Some(crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::HasLandType(LandType::Island),
            count: 3,
        }),
        ..spell(
            "Walk the Aeons",
            cost(&[generic(4), u(), u()]),
            CardType::Sorcery,
            Effect::TargetPlayerThen {
                filter: R::Player,
                then: Box::new(Effect::TakeExtraTurn { who: PlayerRef::Target(0), count: Value::ONE }),
            },
        )
    }
}

// ── Syr Konrad, the Grim ─────────────────────────────────────────────────────

fn creatures_to_top(name: &'static str, mana: ManaCost, kind: CardType) -> CardDefinition {
    spell(
        name,
        mana,
        kind,
        Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 20,
                min_targets: 0,
                filter: R::Creature.and(R::InYourGraveyard),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Top },
                }),
            },
            draw(1),
        ]),
    )
}

/// Gravepurge — {2}{B} Instant. Any number of creature cards from your
/// graveyard to the top of your library; draw a card.
pub fn gravepurge() -> CardDefinition {
    creatures_to_top("Gravepurge", cost(&[generic(2), b()]), CardType::Instant)
}

/// Forever Young — {1}{B} Sorcery. Gravepurge at sorcery speed.
pub fn forever_young() -> CardDefinition {
    creatures_to_top("Forever Young", cost(&[generic(1), b()]), CardType::Sorcery)
}

/// Songs of the Damned — {B} Instant. {B} per creature card in your
/// graveyard.
pub fn songs_of_the_damned() -> CardDefinition {
    spell(
        "Songs of the Damned",
        cost(&[b()]),
        CardType::Instant,
        Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::OfColor(Color::Black, Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Creature }),
        },
    )
}

// ── Runo Stromkirk ───────────────────────────────────────────────────────────

/// Icebreaker Kraken — {10}{U}{U} 8/8 snow Kraken, affinity for snow lands.
/// ETB freezes a target opponent's artifacts and creatures for their next
/// untap; return three snow lands: back to hand.
pub fn icebreaker_kraken() -> CardDefinition {
    let snow_land = || R::Land.and(R::IsSnow).and(R::ControlledByYou);
    CardDefinition {
        supertypes: vec![Supertype::Snow],
        affinity_filter: Some(snow_land()),
        triggered_abilities: vec![etb(Effect::TargetPlayerThen {
            filter: R::OpponentPlayer,
            then: Box::new(Effect::SkipNextUntap {
                what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Artifact.or(R::Creature) },
            }),
        })],
        activated_abilities: vec![ActivatedAbility {
            bounce_other_filter: Some((snow_land(), 3)),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))) },
            ..Default::default()
        }],
        ..creature("Icebreaker Kraken", cost(&[generic(10), u(), u()]), vec![CreatureType::Kraken], 8, 8)
    }
}

/// Lochmere Serpent — {4}{U}{B} 7/7 flash Serpent. Sacrifice an Island:
/// unblockable; sacrifice a Swamp: gain 1 and draw; from your graveyard,
/// {U}{B}: exile five cards from an opponent's graveyard and return it to
/// hand, as a sorcery.
pub fn lochmere_serpent() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                sac_other_filter: Some((R::HasLandType(LandType::Island), 1)),
                effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Unblockable, duration: Duration::EndOfTurn },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                sac_other_filter: Some((R::HasLandType(LandType::Swamp), 1)),
                effect: Effect::Seq(vec![Effect::GainLife { who: Selector::You, amount: Value::ONE }, draw(1)]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[u(), b()]),
                from_graveyard: true,
                sorcery_speed: true,
                effect: Effect::Seq(vec![
                    Effect::ApplyToTargets {
                        // Exactly five, all in one opponent's graveyard (2019-10-04 ruling).
                        max_targets: 5,
                        min_targets: 5,
                        filter: R::InOpponentGraveyard.and(R::SameGraveyardAsTargetSlot(0)),
                        effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile }),
                    },
                    Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
                ]),
                ..Default::default()
            },
        ],
        ..creature("Lochmere Serpent", cost(&[generic(4), u(), b()]), vec![CreatureType::Serpent], 7, 7)
    }
}

/// Striped Riverwinder — {6}{U} 5/5 hexproof Serpent, cycling {U}.
pub fn striped_riverwinder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Hexproof, Keyword::Cycling(cost(&[u()]))],
        ..creature("Striped Riverwinder", cost(&[generic(6), u()]), vec![CreatureType::Serpent], 5, 5)
    }
}

// ── Altaïr Ibn-La'Ahad ───────────────────────────────────────────────────────

/// Raffine's Silencer — {2}{B} 1/1 Human Assassin. ETB connives; dying gives
/// an opponent's creature -X/-X, X its power.
pub fn raffines_silencer() -> CardDefinition {
    let x = || Value::Diff(Box::new(Value::Const(0)), Box::new(Value::PowerOf(Box::new(Selector::This))));
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Connive { what: Selector::This, amount: Value::ONE }),
            on_dies(Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                power: x(),
                toughness: x(),
                duration: Duration::EndOfTurn,
            }),
        ],
        ..creature("Raffine's Silencer", cost(&[generic(2), b()]), vec![CreatureType::Human, CreatureType::Assassin], 1, 1)
    }
}

/// Roshan, Hidden Magister — {3}{B} 4/4 Human Assassin. Your other creatures
/// (and creature spells and cards of yours elsewhere) are Assassins; your
/// face-down creatures have menace; turning a permanent
/// of yours face up draws a card for 1 life.
pub fn roshan_hidden_magister() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            static_ab(
                "Other creatures you control are Assassins in addition to their other types.",
                StaticEffect::AddCreatureTypeToMatching {
                    applies_to: Selector::EachPermanent(mine().and(R::OtherThanSource)),
                    creature_type: CreatureType::Assassin,
                },
            ),
            static_ab(
                "The same is true for creature spells you control and creature cards you own that aren't on the battlefield.",
                StaticEffect::OwnedCardsOffBattlefieldAreTypeToo { filter: R::Creature, creature_type: CreatureType::Assassin },
            ),
            static_ab(
                "Face-down creatures you control have menace.",
                StaticEffect::GrantKeyword { applies_to: Selector::EachPermanent(mine().and(R::FaceDown)), keyword: Keyword::Menace },
            ),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::YourControl),
            effect: Effect::Seq(vec![draw(1), Effect::LoseLife { who: Selector::You, amount: Value::ONE }]),
        }],
        ..legend("Roshan, Hidden Magister", cost(&[generic(3), b()]), vec![CreatureType::Human, CreatureType::Assassin], 4, 4)
    }
}

/// Virtus the Veiled — {2}{B} 1/1 Azra Assassin, partner with Gorm,
/// deathtouch. Its combat damage halves that player's life, rounded up.
pub fn virtus_the_veiled() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::PartnerWith("Gorm the Great".into()), Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::LoseHalfLife { who: Selector::Player(PlayerRef::Target(0)), rounded_up: true },
        }],
        ..legend("Virtus the Veiled", cost(&[generic(2), b()]), vec![CreatureType::Azra, CreatureType::Assassin], 1, 1)
    }
}

// ── Liliana, Heretical Healer ────────────────────────────────────────────────

/// Liliana's Scrounger — {2}{B} 3/2 Human Wizard. Each end step after a
/// death may add loyalty to a Liliana of yours.
pub fn lilianas_scrounger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::ONE }),
            effect: Effect::MayDo {
                description: "Put a loyalty counter on a Liliana you control?".into(),
                body: Box::new(Effect::AddCounter {
                    what: Selector::one_of(Selector::EachPermanent(
                        R::Planeswalker.and(R::HasPlaneswalkerType(PlaneswalkerSubtype::Liliana)).and(R::ControlledByYou),
                    )),
                    kind: CounterType::Loyalty,
                    amount: Value::ONE,
                }),
            },
        }],
        ..creature("Liliana's Scrounger", cost(&[generic(2), b()]), vec![CreatureType::Human, CreatureType::Wizard], 3, 2)
    }
}

/// Oath of Liliana — {2}{B} legendary Enchantment. ETB each opponent
/// sacrifices a creature; each end step after a planeswalker of yours
/// entered makes a 2/2 Zombie.
pub fn oath_of_liliana() -> CardDefinition {
    CardDefinition {
        name: "Oath of Liliana",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::Sacrifice { who: Selector::Player(PlayerRef::EachOpponent), count: Value::ONE, filter: R::Creature }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                    .with_filter(Predicate::PlaneswalkerEnteredThisTurn { who: PlayerRef::You }),
                effect: make(1, token("Zombie", 2, 2, Color::Black, vec![CreatureType::Zombie])),
            },
        ],
        ..Default::default()
    }
}

/// Liliana Vess — {3}{B}{B} planeswalker, loyalty 5. +1: a player discards;
/// −2: tutor to the top; −8: every graveyard's creature cards are yours.
pub fn liliana_vess() -> CardDefinition {
    CardDefinition {
        name: "Liliana Vess",
        cost: cost(&[generic(3), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Liliana], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Discard { who: target_filtered(R::Player), amount: Value::ONE, random: false },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Any,
                    to: ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Top },
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::Move {
                    what: Selector::CardsInZone { who: PlayerRef::EachPlayer, zone: Zone::Graveyard, filter: R::Creature },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Kumena, Tyrant of Orazca ─────────────────────────────────────────────────

fn merfolk_to_top() -> Effect {
    Effect::MayDo {
        description: "Search your library for a Merfolk card to put on top?".into(),
        body: Box::new(Effect::Search {
            who: PlayerRef::You,
            filter: R::HasCreatureType(CreatureType::Merfolk),
            to: ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Top },
        }),
    }
}

/// Forerunner of the Heralds — {3}{G} 3/2 Merfolk Scout. ETB may tutor a
/// Merfolk to the top; another Merfolk of yours entering grows it.
pub fn forerunner_of_the_heralds() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(merfolk_to_top()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasCreatureType(CreatureType::Merfolk) },
                ),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
        ],
        ..creature("Forerunner of the Heralds", cost(&[generic(3), g()]), vec![CreatureType::Merfolk, CreatureType::Scout], 3, 2)
    }
}

/// Merrow Harbinger — {3}{U} 2/3 islandwalk Merfolk Wizard. ETB may tutor a
/// Merfolk to the top.
pub fn merrow_harbinger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Island)],
        triggered_abilities: vec![etb(merfolk_to_top())],
        ..creature("Merrow Harbinger", cost(&[generic(3), u()]), vec![CreatureType::Merfolk, CreatureType::Wizard], 2, 3)
    }
}

/// Merrow Commerce — {1}{U} Kindred Enchantment — Merfolk. Your end step
/// untaps your Merfolk.
pub fn merrow_commerce() -> CardDefinition {
    CardDefinition {
        name: "Merrow Commerce",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Kindred, CardType::Enchantment],
        subtypes: Subtypes { creature_types: vec![CreatureType::Merfolk], ..Default::default() },
        triggered_abilities: vec![at_your(
            TurnStep::End,
            Effect::Untap { what: Selector::EachPermanent(mine().and(R::HasCreatureType(CreatureType::Merfolk))), up_to: None },
        )],
        ..Default::default()
    }
}

// ── Kediss // Malcolm ────────────────────────────────────────────────────────

/// HYDRA Assault Robot — {1}{R} 1/3 artifact Robot Villain. Another Villain
/// or artifact of yours entering pings a target opponent.
pub fn hydra_assault_robot() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasCreatureType(CreatureType::Villain).or(R::Artifact),
            }),
            effect: Effect::DealDamage { to: target_filtered(R::OpponentPlayer), amount: Value::ONE },
        }],
        ..creature("HYDRA Assault Robot", cost(&[generic(1), r()]), vec![CreatureType::Robot, CreatureType::Villain], 1, 3)
    }
}

/// Lightning-Rig Crew — {2}{R} 0/5 Goblin Pirate. {T}: 1 damage to each
/// opponent; your Pirate spells untap it.
pub fn lightning_rig_crew() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::HasCreatureType(CreatureType::Pirate))),
            effect: Effect::Untap { what: Selector::This, up_to: None },
        }],
        ..creature("Lightning-Rig Crew", cost(&[generic(2), r()]), vec![CreatureType::Goblin, CreatureType::Pirate], 0, 5)
    }
}

/// Wings of Velis Vel — {1}{U} Kindred Instant — Shapeshifter, changeling. A
/// creature becomes a 4/4 flier with every creature type this turn.
pub fn wings_of_velis_vel() -> CardDefinition {
    let grant = |keyword| Effect::GrantKeyword { what: Selector::Target(0), keyword, duration: Duration::EndOfTurn };
    CardDefinition {
        card_types: vec![CardType::Kindred, CardType::Instant],
        subtypes: Subtypes { creature_types: vec![CreatureType::Shapeshifter], ..Default::default() },
        keywords: vec![Keyword::Changeling],
        ..spell(
            "Wings of Velis Vel",
            cost(&[generic(1), u()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::SetBasePT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(4),
                    toughness: Value::Const(4),
                    duration: Duration::EndOfTurn,
                },
                grant(Keyword::Changeling),
                grant(Keyword::Flying),
            ]),
        )
    }
}

// ── Vraska, Soul of Stone ────────────────────────────────────────────────────

/// Craterclaw Colossus — {4}{R}{R}{R} 5/5 haste artifact Beast Construct. ETB
/// your creatures get +X/+0 (X your artifacts) and trample.
pub fn craterclaw_colossus() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(mine()),
                power: Value::PermanentCountControlledByMatching(PlayerRef::You, R::Artifact),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::EachPermanent(mine()), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
        ]))],
        ..creature(
            "Craterclaw Colossus",
            cost(&[generic(4), r(), r(), r()]),
            vec![CreatureType::Beast, CreatureType::Construct],
            5,
            5,
        )
    }
}

/// Saheeli, Jewel of Avishkar — {2}{U}{R} 2/4 Human Artificer. Your Thopters
/// have haste; noncreature spells make a 1/1 flying Thopter.
pub fn saheeli_jewel_of_avishkar() -> CardDefinition {
    let thopter = TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        static_abilities: vec![static_ab(
            "Thopters you control have haste.",
            StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(mine().and(R::HasCreatureType(CreatureType::Thopter))),
                keyword: Keyword::Haste,
            },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Creature.negate())),
            effect: make(1, thopter),
        }],
        ..legend(
            "Saheeli, Jewel of Avishkar",
            cost(&[generic(2), u(), r()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            2,
            4,
        )
    }
}

/// Fateful Discovery — {3}{U}{U} Enchantment. An artifact of yours entering
/// draws a card.
pub fn fateful_discovery() -> CardDefinition {
    CardDefinition {
        name: "Fateful Discovery",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact }),
            effect: draw(1),
        }],
        ..Default::default()
    }
}

// ── Darien, King of Kjeldor ──────────────────────────────────────────────────

/// Jade Monolith — {4} Artifact. {1}: the next damage a source of your choice
/// would deal to target creature this turn is dealt to you instead.
pub fn jade_monolith() -> CardDefinition {
    CardDefinition {
        name: "Jade Monolith",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::RedirectNextDamageTo {
                what: target_filtered(R::Creature),
                to: Selector::You,
                chosen_source: true,
                whole_turn: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Healer of the Pride — {3}{W} 2/3 Cat Cleric. Another creature of yours
/// entering gains you 2.
pub fn healer_of_the_pride() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        }],
        ..creature("Healer of the Pride", cost(&[generic(3), w()]), vec![CreatureType::Cat, CreatureType::Cleric], 2, 3)
    }
}

/// Knight-Captain of Eos — {4}{W} 2/2 Human Knight. ETB two 1/1 Soldiers;
/// {W}, sacrifice a Soldier: fog.
pub fn knight_captain_of_eos() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(make(2, token("Soldier", 1, 1, Color::White, vec![CreatureType::Soldier])))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Soldier), 1)),
            effect: Effect::PreventAllCombatDamageThisTurn,
            ..Default::default()
        }],
        ..creature("Knight-Captain of Eos", cost(&[generic(4), w()]), vec![CreatureType::Human, CreatureType::Knight], 2, 2)
    }
}

// ── Ratadrabik of Urborg ─────────────────────────────────────────────────────

/// Gandalf the White — {3}{W}{W} 4/5 flash Avatar Wizard. Legendary and
/// artifact spells have flash; a legendary permanent or artifact entering or
/// leaving doubles your permanents' triggers.
pub fn gandalf_the_white() -> CardDefinition {
    let legend_or_artifact = || R::HasSupertype(Supertype::Legendary).or(R::Artifact);
    CardDefinition {
        keywords: vec![Keyword::Flash],
        static_abilities: vec![
            static_ab(
                "You may cast legendary spells and artifact spells as though they had flash.",
                StaticEffect::ControllerSpellsHaveFlash { filter: legend_or_artifact() },
            ),
            static_ab(
                "If a legendary permanent or an artifact entering the battlefield causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time.",
                StaticEffect::DoubleControllerEtbTriggers { entering: legend_or_artifact() },
            ),
            static_ab(
                "If a legendary permanent or an artifact leaving the battlefield causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time.",
                StaticEffect::DoubleControllerLeaveTriggers { leaving: legend_or_artifact() },
            ),
        ],
        ..legend("Gandalf the White", cost(&[generic(3), w(), w()]), vec![CreatureType::Avatar, CreatureType::Wizard], 4, 5)
    }
}

/// Gollum, Patient Plotter — {1}{B} 3/1 Halfling Horror. Leaving tempts you
/// with the Ring; from your graveyard, {B}, sacrifice a creature: back to
/// hand, as a sorcery.
pub fn gollum_patient_plotter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::RingTempts { who: PlayerRef::You },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            from_graveyard: true,
            sorcery_speed: true,
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..legend("Gollum, Patient Plotter", cost(&[generic(1), b()]), vec![CreatureType::Halfling, CreatureType::Horror], 3, 1)
    }
}

/// Samwise the Stouthearted — {1}{W} 2/1 flash Halfling Peasant. ETB returns
/// up to one permanent card put into your graveyard from the battlefield
/// this turn; the Ring tempts you.
pub fn samwise_the_stouthearted() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Move {
                    what: target_filtered(R::Permanent.and(R::InYourGraveyard).and(R::PutIntoGraveyardFromBattlefieldThisTurn)),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
            Effect::RingTempts { who: PlayerRef::You },
        ]))],
        ..legend("Samwise the Stouthearted", cost(&[generic(1), w()]), vec![CreatureType::Halfling, CreatureType::Peasant], 2, 1)
    }
}

// ── Nahiri, Forged in Fury ───────────────────────────────────────────────────

/// Nahiri, Forged in Fury — {4}{R}{W} 5/4 Kor Artificer, affinity for
/// Equipment. An equipped creature of yours attacking exiles your top card,
/// playable this turn; an Equipment spell cast this way is free.
pub fn nahiri_forged_in_fury() -> CardDefinition {
    CardDefinition {
        affinity_filter: Some(R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::ControlledByYou)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsEquipped }),
            // "You may cast Equipment spells this way without paying their
            // mana costs": an Equipment's grant is free, anything else pays.
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
                    to: ZoneDest::Exile,
                },
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::ExiledThisResolution { filter: R::Any },
                        filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment),
                    },
                    then: Box::new(Effect::GrantMayPlay {
                        what: Selector::ExiledThisResolution { filter: R::Any },
                        duration: crate::card::MayPlayDuration::EndOfThisTurn,
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: false,
                        any_color: false,
                    }),
                    else_: Box::new(Effect::GrantMayPlay {
                        what: Selector::ExiledThisResolution { filter: R::Any },
                        duration: crate::card::MayPlayDuration::EndOfThisTurn,
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: true,
                        any_color: false,
                    }),
                },
            ]),
        }],
        ..legend("Nahiri, Forged in Fury", cost(&[generic(4), r(), w()]), vec![CreatureType::Kor, CreatureType::Artificer], 5, 4)
    }
}

/// Citizen's Crowbar — {1}{W} Equipment, equip {2}. ETB makes a 1/1 Citizen
/// and suits it up; +1/+1 and "{W}, {T}, sacrifice the Crowbar: destroy an
/// artifact or enchantment".
pub fn citizens_crowbar() -> CardDefinition {
    let citizen = TokenDefinition {
        colors: vec![Color::Green, Color::White],
        ..token("Citizen", 1, 1, Color::Green, vec![CreatureType::Citizen])
    };
    CardDefinition {
        name: "Citizen's Crowbar",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            make(1, citizen),
            Effect::AttachSourceTo { host: Selector::LastCreatedToken },
        ]))],
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 1,
            toughness: 1,
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[w()]),
                tap_cost: true,
                sac_attachment_cost: true,
                effect: Effect::Destroy {
                    what: target_filtered(R::Artifact.or(R::HasCardType(CardType::Enchantment))),
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Nahiri, Heir of the Ancients — {2}{R}{W} planeswalker, loyalty 4. +1: a
/// 1/1 Kor Warrior that may take an Equipment; −2: dig six for a Warrior or
/// Equipment; −3: twice your Equipment count to a creature or planeswalker.
pub fn nahiri_heir_of_the_ancients() -> CardDefinition {
    let equipment = || R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::ControlledByYou);
    CardDefinition {
        name: "Nahiri, Heir of the Ancients",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Nahiri], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    make(1, token("Kor Warrior", 1, 1, Color::White, vec![CreatureType::Kor, CreatureType::Warrior])),
                    Effect::OptionalTargets {
                        min: 0,
                        body: Box::new(Effect::Attach { what: target_filtered(equipment()), to: Selector::LastCreatedToken }),
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::LookPickToHand(Box::new(crate::effect::LookPick {
                    who: PlayerRef::You,
                    count: Value::Const(6),
                    pick_filter: Some(
                        R::HasCreatureType(CreatureType::Warrior).or(R::HasArtifactSubtype(ArtifactSubtype::Equipment)),
                    ),
                    take: Some(Value::ONE),
                    optional: true,
                    ..Default::default()
                })),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::DealDamage {
                    to: target_filtered(R::Creature.or(R::Planeswalker)),
                    amount: Value::Times(
                        Box::new(Value::Const(2)),
                        Box::new(Value::PermanentCountControlledByMatching(PlayerRef::You, equipment())),
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
