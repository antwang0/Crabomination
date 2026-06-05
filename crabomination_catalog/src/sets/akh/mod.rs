//! Amonkhet block (AKH / HOU) cards.
//!
//! Showcases Embalm (CR 702.88) and Eternalize (CR 702.91) via
//! `shortcut::embalm` / `shortcut::eternalize`: exile the card from your
//! graveyard for the listed cost to mint a token copy (a Zombie; 4/4 for
//! Eternalize). The token-color override (white/black) is approximated — the
//! copy keeps the original's color.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope, EventSpec,
    ExileReturnZone, Keyword, Predicate, SelectionRequirement, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{embalm, eternalize, etb, on_attack, target_any, target_filtered};
use crate::effect::{Duration, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{cost, b, g, generic, r, u, w};

/// Body helper for a vanilla-ish Embalm/Eternalize creature: stats, creature
/// types, optional keyword, plus the graveyard-activated token-copy ability.
fn akh_body(
    name: &'static str,
    c: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
    keywords: Vec<Keyword>,
    ability: crate::card::ActivatedAbility,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        keywords,
        activated_abilities: vec![ability],
        ..Default::default()
    }
}

/// Sacred Cat — {W} 1/1 Cat, Lifelink. Embalm {W}.
pub fn sacred_cat() -> CardDefinition {
    akh_body("Sacred Cat", cost(&[w()]), vec![CreatureType::Cat], 1, 1,
        vec![Keyword::Lifelink], embalm(cost(&[w()])))
}

/// Adorned Pouncer — {1}{W} 1/1 Cat, Double strike. Eternalize {3}{W}{W}.
pub fn adorned_pouncer() -> CardDefinition {
    akh_body("Adorned Pouncer", cost(&[generic(1), w()]), vec![CreatureType::Cat], 1, 1,
        vec![Keyword::DoubleStrike], eternalize(cost(&[generic(3), w(), w()])))
}

/// Unwavering Initiate — {2}{W} 3/2 Human Warrior, Vigilance. Embalm {4}{W}.
pub fn unwavering_initiate() -> CardDefinition {
    akh_body("Unwavering Initiate", cost(&[generic(2), w()]),
        vec![CreatureType::Human, CreatureType::Warrior], 3, 2,
        vec![Keyword::Vigilance], embalm(cost(&[generic(4), w()])))
}

/// Steadfast Sentinel — {3}{W} 2/3 Human Cleric, Vigilance. Eternalize {4}{W}{W}.
pub fn steadfast_sentinel() -> CardDefinition {
    akh_body("Steadfast Sentinel", cost(&[generic(3), w()]),
        vec![CreatureType::Human, CreatureType::Cleric], 2, 3,
        vec![Keyword::Vigilance], eternalize(cost(&[generic(4), w(), w()])))
}

/// Aven Initiate — {3}{U} 3/2 Bird Warrior, Flying. Embalm {6}{U}.
pub fn aven_initiate() -> CardDefinition {
    akh_body("Aven Initiate", cost(&[generic(3), u()]),
        vec![CreatureType::Bird, CreatureType::Warrior], 3, 2,
        vec![Keyword::Flying], embalm(cost(&[generic(6), u()])))
}

/// Proven Combatant — {U} 1/1 Human Warrior. Eternalize {4}{U}{U}.
pub fn proven_combatant() -> CardDefinition {
    akh_body("Proven Combatant", cost(&[u()]),
        vec![CreatureType::Human, CreatureType::Warrior], 1, 1,
        vec![], eternalize(cost(&[generic(4), u(), u()])))
}

/// Tah-Crop Skirmisher — {1}{U} 2/1 Snake Warrior. Embalm {3}{U}.
pub fn tah_crop_skirmisher() -> CardDefinition {
    akh_body("Tah-Crop Skirmisher", cost(&[generic(1), u()]),
        vec![CreatureType::Snake, CreatureType::Warrior], 2, 1,
        vec![], embalm(cost(&[generic(3), u()])))
}

/// Honored Hydra — {5}{G} 6/6 Snake Hydra, Trample. Embalm {3}{G}{G}.
pub fn honored_hydra() -> CardDefinition {
    akh_body("Honored Hydra", cost(&[generic(5), g()]),
        vec![CreatureType::Snake, CreatureType::Hydra], 6, 6,
        vec![Keyword::Trample], embalm(cost(&[generic(3), g(), g()])))
}

/// Timeless Witness — {2}{G}{G} 2/1 Human Shaman. ETB: return target card from
/// your graveyard to hand (Eternal Witness). Embalm {3}{G}{G}.
pub fn timeless_witness() -> CardDefinition {
    let mut c = akh_body("Timeless Witness", cost(&[generic(2), g(), g()]),
        vec![CreatureType::Human, CreatureType::Shaman], 2, 1,
        vec![], embalm(cost(&[generic(3), g(), g()])));
    // ETB: return target card from your graveyard to hand (Eternal Witness).
    c.triggered_abilities = vec![etb(Effect::Move {
        what: target_filtered(SelectionRequirement::Player.negate()),
        to: ZoneDest::Hand(PlayerRef::You),
    })];
    c
}

/// Sunscourge Champion — {2}{W} 2/3 Human Wizard. ETB: gain life equal to its
/// power. Eternalize {3}{W}{W}.
pub fn sunscourge_champion() -> CardDefinition {
    let mut c = akh_body("Sunscourge Champion", cost(&[generic(2), w()]),
        vec![CreatureType::Human, CreatureType::Wizard], 2, 3,
        vec![], eternalize(cost(&[generic(3), w(), w()])));
    c.triggered_abilities = vec![etb(Effect::GainLife {
        who: Selector::You,
        amount: Value::PowerOf(Box::new(Selector::This)),
    })];
    c
}

/// Dreamstealer — {2}{B} 1/2 Human Wizard, Menace. Eternalize {5}{B}{B}.
/// (The combat-damage discard rider collapses — the body + Eternalize is the
/// gameplay-relevant attribute.)
pub fn dreamstealer() -> CardDefinition {
    akh_body("Dreamstealer", cost(&[generic(2), b()]),
        vec![CreatureType::Human, CreatureType::Wizard], 1, 2,
        vec![Keyword::Menace], eternalize(cost(&[generic(5), b(), b()])))
}

/// Oketra's Attendant — {3}{W}{W} 3/3 Bird Soldier, Flying. Cycling {2}.
/// Embalm {3}{W}{W}.
pub fn oketras_attendant() -> CardDefinition {
    akh_body("Oketra's Attendant", cost(&[generic(3), w(), w()]),
        vec![CreatureType::Bird, CreatureType::Soldier], 3, 3,
        vec![Keyword::Flying, Keyword::Cycling(cost(&[generic(2)]))],
        embalm(cost(&[generic(3), w(), w()])))
}

/// Anointer Priest — {1}{W} 1/3 Human Cleric. Whenever a creature token you
/// control enters, gain 1 life. Embalm {3}{W}.
pub fn anointer_priest() -> CardDefinition {
    let mut c = akh_body("Anointer Priest", cost(&[generic(1), w()]),
        vec![CreatureType::Human, CreatureType::Cleric], 1, 3,
        vec![], embalm(cost(&[generic(3), w()])));
    c.triggered_abilities = vec![TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::Creature.and(SelectionRequirement::IsToken),
            }),
        effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
    }];
    c
}

/// Angel of Sanctions — {3}{W}{W} 3/4 Angel, Flying. ETB: exile target nonland
/// permanent an opponent controls until this leaves. Embalm {5}{W}.
pub fn angel_of_sanctions() -> CardDefinition {
    let mut c = akh_body("Angel of Sanctions", cost(&[generic(3), w(), w()]),
        vec![CreatureType::Angel], 3, 4,
        vec![Keyword::Flying], embalm(cost(&[generic(5), w()])));
    c.triggered_abilities = vec![etb(Effect::ExileUntilSourceLeaves {
        what: target_filtered(
            SelectionRequirement::Permanent
                .and(SelectionRequirement::Nonland)
                .and(SelectionRequirement::ControlledByOpponent),
        ),
        return_to: ExileReturnZone::Battlefield,
    })];
    c
}

/// Earthshaker Khenra — {1}{R} 2/1 Jackal Warrior, Haste. ETB: target creature
/// with power ≤ this creature's power can't block this turn. Eternalize
/// {4}{R}{R}. (The "≤ its power" filter is fixed at the printed power 2.)
pub fn earthshaker_khenra() -> CardDefinition {
    use crate::effect::Duration;
    use crate::mana::r;
    let mut c = akh_body("Earthshaker Khenra", cost(&[generic(1), r()]),
        vec![CreatureType::Jackal, CreatureType::Warrior], 2, 1,
        vec![Keyword::Haste], eternalize(cost(&[generic(4), r(), r()])));
    c.triggered_abilities = vec![etb(Effect::GrantKeyword {
        what: target_filtered(
            SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(2)),
        ),
        keyword: Keyword::CantBlock,
        duration: Duration::EndOfTurn,
    })];
    c
}

/// Sinuous Striker — {2}{U} 2/2 Snake Warrior. {U}: +1/-1 until end of turn.
/// Eternalize {3}{U}{U}. (The Eternalize "discard a card" additional cost is
/// dropped.)
pub fn sinuous_striker() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::Duration;
    let mut c = akh_body("Sinuous Striker", cost(&[generic(2), u()]),
        vec![CreatureType::Snake, CreatureType::Warrior], 2, 2,
        vec![], eternalize(cost(&[generic(3), u(), u()])));
    c.activated_abilities.insert(0, ActivatedAbility {
        mana_cost: cost(&[u()]),
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    });
    c
}

/// Champion of Wits — {2}{U} 2/1 Snake Wizard. ETB: you may draw cards equal to
/// its power, then discard two. Eternalize {5}{U}{U} (token is 4/4, so it draws
/// four).
pub fn champion_of_wits() -> CardDefinition {
    let mut c = akh_body("Champion of Wits", cost(&[generic(2), u()]),
        vec![CreatureType::Snake, CreatureType::Wizard], 2, 1,
        vec![], eternalize(cost(&[generic(5), u(), u()])));
    c.triggered_abilities = vec![etb(Effect::MayDo {
        description: "Draw cards equal to power, then discard two".into(),
        body: Box::new(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::PowerOf(Box::new(Selector::This)) },
            Effect::Discard { who: Selector::You, amount: Value::Const(2), random: false },
        ])),
    })];
    c
}

// ── Exert (CR 702.137) ───────────────────────────────────────────────────────
// The engine auto-exerts an attacking creature with `Keyword::Exert` (it skips
// its next untap) and fires its SelfSource Attacks trigger as the exert bonus.

/// Tah-Crop Elite — {3}{W} 2/2 Bird Warrior, Flying. Exert as it attacks:
/// creatures you control get +1/+1 until end of turn.
pub fn tah_crop_elite() -> CardDefinition {
    CardDefinition {
        name: "Tah-Crop Elite",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Exert],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Glory-Bound Initiate — {1}{W} 3/1 Human Warrior. Exert as it attacks: it
/// gets +1/+3 and gains lifelink until end of turn.
pub fn glory_bound_initiate() -> CardDefinition {
    CardDefinition {
        name: "Glory-Bound Initiate",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Exert],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Bloodrage Brawler — {1}{R} 4/3 Minotaur Warrior. ETB: discard a card.
pub fn bloodrage_brawler() -> CardDefinition {
    CardDefinition {
        name: "Bloodrage Brawler",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Discard {
            who: Selector::You,
            amount: Value::Const(1),
            random: false,
        })],
        ..Default::default()
    }
}

/// Nimble-Blade Khenra — {1}{R} 1/3 Jackal Warrior, Prowess.
pub fn nimble_blade_khenra() -> CardDefinition {
    CardDefinition {
        name: "Nimble-Blade Khenra",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Jackal, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Prowess],
        ..Default::default()
    }
}

/// Hooded Brawler — {2}{G} 3/2 Snake Warrior. Exert as it attacks: it gets
/// +2/+2 until end of turn.
pub fn hooded_brawler() -> CardDefinition {
    CardDefinition {
        name: "Hooded Brawler",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Exert],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Greater Sandwurm — {5}{G}{G} 7/7 Wurm. Can't be blocked by creatures with
/// power 2 or less. Cycling {2}.
pub fn greater_sandwurm() -> CardDefinition {
    CardDefinition {
        name: "Greater Sandwurm",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wurm], ..Default::default() },
        power: 7,
        toughness: 7,
        keywords: vec![
            Keyword::Cycling(cost(&[generic(2)])),
            Keyword::CantBeBlockedBy(Box::new(SelectionRequirement::PowerAtMost(2))),
        ],
        ..Default::default()
    }
}

/// Pouncing Cheetah — {2}{G} 3/2 Cat, Flash.
pub fn pouncing_cheetah() -> CardDefinition {
    CardDefinition {
        name: "Pouncing Cheetah",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        ..Default::default()
    }
}

/// Defiant Khenra — {1}{R} 2/2 Jackal Warrior.
pub fn defiant_khenra() -> CardDefinition {
    CardDefinition {
        name: "Defiant Khenra",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Jackal, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        ..Default::default()
    }
}

/// Naga Vitalist — {1}{G} 1/2 Snake Druid. {T}: Add one mana of any type a land
/// you control could produce (modeled via `AnyColorYouCouldProduce`).
pub fn naga_vitalist() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Naga Vitalist",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyColorYouCouldProduce,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Spells / enchantments ────────────────────────────────────────────────────

/// Cast Out — {3}{W} Enchantment, Flash. ETB: exile target nonland permanent an
/// opponent controls until this leaves. Cycling {W}.
pub fn cast_out() -> CardDefinition {
    CardDefinition {
        name: "Cast Out",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Flash, Keyword::Cycling(cost(&[w()]))],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(
                SelectionRequirement::Permanent
                    .and(SelectionRequirement::Nonland)
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}

/// Open Fire — {2}{R} Instant. Deal 3 damage to any target.
pub fn open_fire() -> CardDefinition {
    CardDefinition {
        name: "Open Fire",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
        ..Default::default()
    }
}

/// Gideon's Reproach — {1}{W} Instant. Deal 4 damage to target attacking or
/// blocking creature.
pub fn gideons_reproach() -> CardDefinition {
    CardDefinition {
        name: "Gideon's Reproach",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature.and(
                    SelectionRequirement::IsAttacking.or(SelectionRequirement::IsBlocking),
                ),
            ),
            amount: Value::Const(4),
        },
        ..Default::default()
    }
}

/// Every AKH factory, for snapshot name→factory registration and the cube.
pub fn all_akh_card_factories() -> &'static [crate::CardFactory] {
    &[
        sacred_cat,
        adorned_pouncer,
        unwavering_initiate,
        steadfast_sentinel,
        aven_initiate,
        proven_combatant,
        tah_crop_skirmisher,
        honored_hydra,
        timeless_witness,
        sunscourge_champion,
        dreamstealer,
        oketras_attendant,
        anointer_priest,
        angel_of_sanctions,
        earthshaker_khenra,
        sinuous_striker,
        champion_of_wits,
        tah_crop_elite,
        glory_bound_initiate,
        bloodrage_brawler,
        nimble_blade_khenra,
        cast_out,
        open_fire,
        gideons_reproach,
        hooded_brawler,
        greater_sandwurm,
        pouncing_cheetah,
        defiant_khenra,
        naga_vitalist,
    ]
}
