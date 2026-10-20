//! Commander: the top 600's six-card gaps — Maha, Its Feathers Night;
//! General Ferrous Rokiric; Greasefang, Okiba Boss (`cmdr_edhrec19` is the
//! previous batch). Tests in `tests/recent_b/cmdr_edhrec20.rs`.

use crate::card::{
    AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{ActivatedAbility, Duration, Effect, PlayerRef};
use crate::mana::{b, cost, generic, hybrid, r, w, Color, ManaCost};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn static_ab(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

fn theirs() -> Selector {
    Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent))
}

/// "Creatures your opponents control get -1/-1 until end of turn."
fn shrink_theirs() -> Effect {
    Effect::PumpPT { what: theirs(), power: Value::Const(-1), toughness: Value::Const(-1), duration: Duration::EndOfTurn }
}

fn vehicle() -> R {
    R::HasArtifactSubtype(ArtifactSubtype::Vehicle)
}

fn rw() -> crate::mana::ManaSymbol {
    hybrid(Color::Red, Color::White)
}

// ── Maha, Its Feathers Night ─────────────────────────────────────────────────

/// M.O.D.O.K. — {3}{B}{B} 2/2 legendary artifact creature Villain, flying,
/// lifelink. Pay 3 life: it connives (your turn only). Creatures your
/// opponents control get -1/-1.
pub fn m_o_d_o_k() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            life_cost: 3,
            condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::Connive { what: Selector::This, amount: Value::ONE },
            ..Default::default()
        }],
        static_abilities: vec![static_ab(
            "Creatures your opponents control get -1/-1.",
            StaticEffect::PumpPT { applies_to: theirs(), power: -1, toughness: -1 },
        )],
        ..creature("M.O.D.O.K.", cost(&[generic(3), b(), b()]), vec![CreatureType::Villain], 2, 2)
    }
}

/// Night Clubber — {1}{B}{B} 2/2 Human Warrior. ETB: opponents' creatures get
/// -1/-1 until end of turn. Blitz {2}{B}.
pub fn night_clubber() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(shrink_theirs())],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(2), b()]),
            blitz: true,
            ..Default::default()
        }),
        ..creature("Night Clubber", cost(&[generic(1), b(), b()]), vec![CreatureType::Human, CreatureType::Warrior], 2, 2)
    }
}

/// Plague Mare — {1}{B}{B} 2/2 Nightmare Horse. Can't be blocked by white
/// creatures. ETB: opponents' creatures get -1/-1 until end of turn.
pub fn plague_mare() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedBy(Box::new(R::HasColor(Color::White)))],
        triggered_abilities: vec![etb(shrink_theirs())],
        ..creature("Plague Mare", cost(&[generic(1), b(), b()]), vec![CreatureType::Nightmare, CreatureType::Horse], 2, 2)
    }
}

/// Make Obsolete — {2}{B} instant. Opponents' creatures get -1/-1 until end
/// of turn.
pub fn make_obsolete() -> CardDefinition {
    spell("Make Obsolete", cost(&[generic(2), b()]), CardType::Instant, shrink_theirs())
}

/// Gruesome Realization — {1}{B}{B} sorcery. Choose one: draw two and lose 2
/// life; or opponents' creatures get -1/-1 until end of turn.
pub fn gruesome_realization() -> CardDefinition {
    spell(
        "Gruesome Realization",
        cost(&[generic(1), b(), b()]),
        CardType::Sorcery,
        Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
            ]),
            shrink_theirs(),
        ]),
    )
}

/// Shrivel — {1}{B} sorcery. All creatures get -1/-1 until end of turn.
pub fn shrivel() -> CardDefinition {
    spell(
        "Shrivel",
        cost(&[generic(1), b()]),
        CardType::Sorcery,
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        },
    )
}

// ── General Ferrous Rokiric ──────────────────────────────────────────────────

/// Figure of Destiny — {R/W} 1/1 Kithkin. {R/W}: a Kithkin Spirit with base
/// 2/2. {R/W}x3: if a Spirit, a Kithkin Spirit Warrior with base 4/4.
/// {R/W}x6: if a Warrior, a Kithkin Spirit Warrior Avatar with base 8/8,
/// flying and first strike.
pub fn figure_of_destiny() -> CardDefinition {
    use CreatureType::{Avatar, Kithkin, Spirit, Warrior};
    let become_ = |types: Vec<CreatureType>, pt: i32, keywords: Vec<Keyword>| {
        let mut steps = vec![
            Effect::BecomeCreatureType { what: Selector::This, creature_types: types, duration: Duration::Permanent },
            Effect::SetBasePT {
                what: Selector::This,
                power: Value::Const(pt),
                toughness: Value::Const(pt),
                duration: Duration::Permanent,
            },
        ];
        for keyword in keywords {
            steps.push(Effect::GrantKeyword { what: Selector::This, keyword, duration: Duration::Permanent });
        }
        Effect::Seq(steps)
    };
    let if_is = |t: CreatureType, then: Effect| Effect::If {
        cond: Predicate::EntityMatches { what: Selector::This, filter: R::HasCreatureType(t) },
        then: Box::new(then),
        else_: Box::new(Effect::Noop),
    };
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: ManaCost { symbols: vec![rw()] },
                effect: become_(vec![Kithkin, Spirit], 2, vec![]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: ManaCost { symbols: vec![rw(); 3] },
                effect: if_is(Spirit, become_(vec![Kithkin, Spirit, Warrior], 4, vec![])),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: ManaCost { symbols: vec![rw(); 6] },
                effect: if_is(
                    Warrior,
                    become_(vec![Kithkin, Spirit, Warrior, Avatar], 8, vec![Keyword::Flying, Keyword::FirstStrike]),
                ),
                ..Default::default()
            },
        ],
        ..creature("Figure of Destiny", ManaCost { symbols: vec![rw()] }, vec![Kithkin], 1, 1)
    }
}

/// Rem Karolus, Stalwart Slayer — {1}{R}{W} 2/3 legendary Human Knight,
/// flying, haste. Spell damage to you or another permanent you control is
/// prevented; spell damage to an opponent or their permanents is +1.
pub fn rem_karolus_stalwart_slayer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Haste],
        static_abilities: vec![
            static_ab(
                "If a spell would deal damage to you or another permanent you control, prevent that damage.",
                StaticEffect::PreventSpellDamageToYouAndYourOtherPermanents,
            ),
            static_ab(
                "If a spell would deal damage to an opponent or a permanent an opponent controls, it deals that much damage plus 1 instead.",
                StaticEffect::SpellDamageToOpponentsBonus { amount: 1 },
            ),
        ],
        ..creature(
            "Rem Karolus, Stalwart Slayer",
            cost(&[generic(1), r(), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            3,
        )
    }
}

/// Intimidation Bolt — {1}{R}{W} instant. 3 damage to target creature; other
/// creatures can't attack this turn.
pub fn intimidation_bolt() -> CardDefinition {
    spell(
        "Intimidation Bolt",
        cost(&[generic(1), r(), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(3) },
            Effect::CantAttackThisTurn {
                what: Selector::EachPermanentExceptTargets(R::Creature),
            },
        ]),
    )
}

/// Justice Strike — {R}{W} instant. Target creature deals damage to itself
/// equal to its power.
pub fn justice_strike() -> CardDefinition {
    spell(
        "Justice Strike",
        cost(&[r(), w()]),
        CardType::Instant,
        Effect::DealDamageFrom {
            source: target_filtered(R::Creature),
            to: Selector::Target(0),
            amount: Value::PowerOf(Box::new(Selector::Target(0))),
        },
    )
}

/// Ride Down — {R}{W} instant. Destroy target blocking creature; the creatures
/// it blocked gain trample until end of turn.
pub fn ride_down() -> CardDefinition {
    spell(
        "Ride Down",
        cost(&[r(), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature.and(R::IsBlocking)) },
            Effect::GrantKeyword {
                what: Selector::CreaturesBlockedByThisTurn(Box::new(Selector::Target(0))),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Deafening Clarion — {1}{R}{W} sorcery. Choose one or both: 3 damage to each
/// creature; creatures you control gain lifelink until end of turn.
pub fn deafening_clarion() -> CardDefinition {
    spell(
        "Deafening Clarion",
        cost(&[generic(1), r(), w()]),
        CardType::Sorcery,
        Effect::ChooseModesCast {
            modes: vec![
                Effect::DealDamage { to: Selector::EachPermanent(R::Creature), amount: Value::Const(3) },
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::Lifelink,
                    duration: Duration::EndOfTurn,
                },
            ],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
    )
}

// ── Greasefang, Okiba Boss ───────────────────────────────────────────────────

/// Giant Ox — {1}{W} 0/6 Ox. Crews Vehicles using its toughness.
pub fn giant_ox() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "This creature crews Vehicles using its toughness rather than its power.",
            StaticEffect::SelfCrewsSaddlesWithToughness,
        )],
        ..creature("Giant Ox", cost(&[generic(1), w()]), vec![CreatureType::Ox], 0, 6)
    }
}

/// Hotshot Mechanic — {W} 2/1 artifact creature Fox Pilot. Crews Vehicles as
/// though its power were 2 greater.
pub fn hotshot_mechanic() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![static_ab(
            "This creature crews Vehicles as though its power were 2 greater.",
            StaticEffect::CrewSaddlePowerBonus { applies_to: Selector::This, amount: 2 },
        )],
        ..creature("Hotshot Mechanic", cost(&[w()]), vec![CreatureType::Fox, CreatureType::Pilot], 2, 1)
    }
}

/// Kitsune Ace — {1}{W} 2/2 Fox Pilot. Whenever a Vehicle you control attacks,
/// choose one: that Vehicle gains first strike until end of turn; or untap
/// this creature.
pub fn kitsune_ace() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: vehicle() }),
            effect: Effect::ChooseMode(vec![
                Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
                Effect::Untap { what: Selector::This, up_to: None },
            ]),
        }],
        ..creature("Kitsune Ace", cost(&[generic(1), w()]), vec![CreatureType::Fox, CreatureType::Pilot], 2, 2)
    }
}

/// Born to Drive — {2}{W} Aura, enchant artifact or creature. The enchanted
/// creature gets +1/+1 for each creature and/or Vehicle you control. Channel —
/// {2}{W}, discard this card: two 1/1 Pilots that crew as though 2 greater.
pub fn born_to_drive() -> CardDefinition {
    let pilot = TokenDefinition {
        name: "Pilot".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Pilot], ..Default::default() },
        static_abilities: vec![static_ab(
            "This token crews Vehicles as though its power were 2 greater.",
            StaticEffect::CrewSaddlePowerBonus { applies_to: Selector::This, amount: 2 },
        )],
        ..Default::default()
    };
    let n = Value::CountOf(Box::new(Selector::EachPermanent(R::Creature.or(vehicle()).and(R::ControlledByYou))));
    CardDefinition {
        name: "Born to Drive",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.or(R::Artifact)) },
        static_abilities: vec![static_ab(
            "As long as enchanted permanent is a creature, it gets +1/+1 for each creature and/or Vehicle you control.",
            StaticEffect::PumpPTByValue {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                power: n.clone(),
                toughness: n,
            },
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: Arc::new(pilot) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Nezumi Road Captain — 2/2 enchantment creature Rat Rogue, menace. Vehicles
/// you control have menace.
fn nezumi_road_captain() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        keywords: vec![Keyword::Menace],
        static_abilities: vec![static_ab(
            "Vehicles you control have menace.",
            StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(vehicle().and(R::ControlledByYou)),
                keyword: Keyword::Menace,
            },
        )],
        ..creature("Nezumi Road Captain", ManaCost::default(), vec![CreatureType::Rat, CreatureType::Rogue], 2, 2)
    }
}

/// Okiba Reckoner Raid // Nezumi Road Captain — {B} Saga. I, II: each opponent
/// loses 1 life and you gain 1. III: exile it, then return it transformed.
pub fn okiba_reckoner_raid() -> CardDefinition {
    let drain = || {
        Effect::Seq(vec![
            Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
            Effect::GainLife { who: Selector::You, amount: Value::ONE },
        ])
    };
    CardDefinition {
        name: "Okiba Reckoner Raid",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![(1, drain()), (2, drain()), (3, Effect::ExileSelfReturnTransformed)],
        back_face: Some(Box::new(nezumi_road_captain())),
        ..Default::default()
    }
}
