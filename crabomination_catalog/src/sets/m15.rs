//! Magic 2015 (M15) — the convoke shell, the Paragon cycle, the Soul cycle,
//! and the common/uncommon core. Tests in `classic_sets/m15`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    DynamicPt, EnchantmentSubtype, EquipBonus, Keyword, LandType, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, on_dies, target_any, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector, ZoneDest,
    ZoneRef,
};
use crate::mana::{b, cost, g, generic, r, u, w, x, Color, ManaCost};

fn creature(
    name: &'static str,
    mana: ManaCost,
    p: i32,
    t: i32,
    ct: Vec<CreatureType>,
    kw: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: ct, ..Default::default() },
        power: p,
        toughness: t,
        keywords: kw,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

/// A convoke spell (CR 702.51).
fn convoke_spell(
    name: &'static str,
    mana: ManaCost,
    kind: CardType,
    effect: Effect,
) -> CardDefinition {
    CardDefinition { keywords: vec![Keyword::Convoke], ..spell(name, mana, kind, effect) }
}

/// "You control a [land type]" — the M15 "gets +1/+1 as long as you control a
/// Plains/Island/…" cycle's gate.
fn controls_land(lt: LandType) -> Predicate {
    Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(R::HasLandType(lt).and(R::ControlledByYou)),
        n: Value::Const(1),
    }
}

/// A plain "enchant creature" Aura with a static bonus.
fn aura(name: &'static str, mana: ManaCost, enchant: R, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchant) },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

/// An Equipment with a flat bonus and an equip cost.
fn equipment(
    name: &'static str,
    mana: ManaCost,
    equip: ManaCost,
    bonus: EquipBonus,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

// ── Convoke ──────────────────────────────────────────────────────────────────

/// Triplicate Spirits — {4}{W}{W} Sorcery with convoke. Three 1/1 flying
/// Spirits.
pub fn triplicate_spirits() -> CardDefinition {
    convoke_spell(
        "Triplicate Spirits",
        cost(&[generic(4), w(), w()]),
        CardType::Sorcery,
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(3),
            definition: spirit_token(),
        },
    )
}

fn spirit_token() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::White],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Ephemeral Shields — {1}{W} Instant with convoke. Indestructible.
pub fn ephemeral_shields() -> CardDefinition {
    convoke_spell(
        "Ephemeral Shields",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::Indestructible,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Meditation Puzzle — {3}{W}{W} Instant with convoke. Gain 8 life.
pub fn meditation_puzzle() -> CardDefinition {
    convoke_spell(
        "Meditation Puzzle",
        cost(&[generic(3), w(), w()]),
        CardType::Instant,
        Effect::GainLife { who: Selector::You, amount: Value::Const(8) },
    )
}

/// Crowd's Favor — {R} Instant with convoke. +1/+0 and first strike.
pub fn crowds_favor() -> CardDefinition {
    convoke_spell(
        "Crowd's Favor",
        cost(&[r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Covenant of Blood — {6}{B} Sorcery with convoke. Drain 4.
pub fn covenant_of_blood() -> CardDefinition {
    convoke_spell(
        "Covenant of Blood",
        cost(&[generic(6), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage { to: target_any(), amount: Value::Const(4) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
        ]),
    )
}

/// Endless Obedience — {4}{B}{B} Sorcery with convoke. Reanimate from any
/// graveyard under your control.
pub fn endless_obedience() -> CardDefinition {
    convoke_spell(
        "Endless Obedience",
        cost(&[generic(4), b(), b()]),
        CardType::Sorcery,
        Effect::Move {
            what: target_filtered(R::Creature.and(R::InGraveyard)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
    )
}

/// Unmake the Graves — {4}{B} Instant with convoke. Up to two creature cards
/// back to hand.
pub fn unmake_the_graves() -> CardDefinition {
    convoke_spell(
        "Unmake the Graves",
        cost(&[generic(4), b()]),
        CardType::Instant,
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature.and(R::InGraveyard).and(R::OwnedByYou),
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        },
    )
}

/// Feral Incarnation — {8}{G} Sorcery with convoke. Three 3/3 Beasts.
pub fn feral_incarnation() -> CardDefinition {
    convoke_spell(
        "Feral Incarnation",
        cost(&[generic(8), g()]),
        CardType::Sorcery,
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(3),
            definition: beast_token(),
        },
    )
}

fn beast_token() -> TokenDefinition {
    TokenDefinition {
        name: "Beast".into(),
        power: 3,
        toughness: 3,
        colors: vec![Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        ..Default::default()
    }
}

/// Nissa's Expedition — {4}{G} Sorcery with convoke. Two basics tapped.
pub fn nissas_expedition() -> CardDefinition {
    convoke_spell(
        "Nissa's Expedition",
        cost(&[generic(4), g()]),
        CardType::Sorcery,
        Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            count: Value::Const(2),
        },
    )
}

/// Will-Forged Golem — {6} 4/4 Golem with convoke.
pub fn will_forged_golem() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..creature(
            "Will-Forged Golem",
            cost(&[generic(6)]),
            4,
            4,
            vec![CreatureType::Golem],
            vec![Keyword::Convoke],
        )
    }
}

/// Living Totem — {3}{G} 2/3 Plant Elemental with convoke. ETB: a +1/+1
/// counter on another target creature.
pub fn living_totem() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Put a +1/+1 counter on another target creature?".into(),
            body: Box::new(Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        })],
        ..creature(
            "Living Totem",
            cost(&[generic(3), g()]),
            2,
            3,
            vec![CreatureType::Plant, CreatureType::Elemental],
            vec![Keyword::Convoke],
        )
    }
}

/// Seraph of the Masses — {5}{W}{W} */* Angel with convoke and flying; its
/// stats are the creatures you control.
pub fn seraph_of_the_masses() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::CreaturesControlled { base: 0 }),
        ..creature(
            "Seraph of the Masses",
            cost(&[generic(5), w(), w()]),
            0,
            0,
            vec![CreatureType::Angel],
            vec![Keyword::Convoke, Keyword::Flying],
        )
    }
}

// ── The Paragon cycle ────────────────────────────────────────────────────────

/// A Paragon: a 2/2 lord for its own color plus a tap-to-grant ability.
fn paragon(
    name: &'static str,
    mana: ManaCost,
    ct: Vec<CreatureType>,
    color: Color,
    ability_cost: ManaCost,
    granted: Keyword,
) -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control of this Paragon's color get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::HasColor(color))
                        .and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ability_cost,
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::HasColor(color))
                        .and(R::OtherThanSource),
                ),
                keyword: granted,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(name, mana, 2, 2, ct, vec![])
    }
}

/// Paragon of New Dawns — {3}{W} 2/2. {W}, {T}: another white creature gains
/// vigilance.
pub fn paragon_of_new_dawns() -> CardDefinition {
    paragon(
        "Paragon of New Dawns",
        cost(&[generic(3), w()]),
        vec![CreatureType::Human, CreatureType::Soldier],
        Color::White,
        cost(&[w()]),
        Keyword::Vigilance,
    )
}

/// Paragon of Gathering Mists — {3}{U} 2/2. {U}, {T}: another blue creature
/// gains flying.
pub fn paragon_of_gathering_mists() -> CardDefinition {
    paragon(
        "Paragon of Gathering Mists",
        cost(&[generic(3), u()]),
        vec![CreatureType::Human, CreatureType::Wizard],
        Color::Blue,
        cost(&[u()]),
        Keyword::Flying,
    )
}

/// Paragon of Open Graves — {3}{B} 2/2. {2}{B}, {T}: another black creature
/// gains deathtouch.
pub fn paragon_of_open_graves() -> CardDefinition {
    paragon(
        "Paragon of Open Graves",
        cost(&[generic(3), b()]),
        vec![CreatureType::Skeleton, CreatureType::Warrior],
        Color::Black,
        cost(&[generic(2), b()]),
        Keyword::Deathtouch,
    )
}

/// Paragon of Fierce Defiance — {3}{R} 2/2. {R}, {T}: another red creature
/// gains haste.
pub fn paragon_of_fierce_defiance() -> CardDefinition {
    paragon(
        "Paragon of Fierce Defiance",
        cost(&[generic(3), r()]),
        vec![CreatureType::Human, CreatureType::Warrior],
        Color::Red,
        cost(&[r()]),
        Keyword::Haste,
    )
}

/// Paragon of Eternal Wilds — {3}{G} 2/2. {G}, {T}: another green creature
/// gains trample.
pub fn paragon_of_eternal_wilds() -> CardDefinition {
    paragon(
        "Paragon of Eternal Wilds",
        cost(&[generic(3), g()]),
        vec![CreatureType::Human, CreatureType::Druid],
        Color::Green,
        cost(&[g()]),
        Keyword::Trample,
    )
}

// ── The Soul cycle ───────────────────────────────────────────────────────────

/// A Soul: a 6/6 Avatar whose ability works from the battlefield and, for an
/// exile cost, from the graveyard.
fn soul(
    name: &'static str,
    mana: ManaCost,
    kw: Vec<Keyword>,
    ability_cost: ManaCost,
    effect: Effect,
) -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: ability_cost.clone(),
                effect: effect.clone(),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: ability_cost,
                from_graveyard: true,
                exile_self_cost: true,
                effect,
                ..Default::default()
            },
        ],
        ..creature(name, mana, 6, 6, vec![CreatureType::Avatar], kw)
    }
}

/// Soul of Theros — {4}{W}{W} 6/6 with vigilance. {4}{W}{W}: team +2/+2,
/// first strike, lifelink.
pub fn soul_of_theros() -> CardDefinition {
    soul(
        "Soul of Theros",
        cost(&[generic(4), w(), w()]),
        vec![Keyword::Vigilance],
        cost(&[generic(4), w(), w()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: crate::effect::shortcut::each_your_creature(),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeywords {
                what: crate::effect::shortcut::each_your_creature(),
                keywords: vec![Keyword::FirstStrike, Keyword::Lifelink],
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Soul of Ravnica — {4}{U}{U} 6/6 with flying. {5}{U}{U}: draw a card per
/// color among your permanents.
pub fn soul_of_ravnica() -> CardDefinition {
    soul(
        "Soul of Ravnica",
        cost(&[generic(4), u(), u()]),
        vec![Keyword::Flying],
        cost(&[generic(5), u(), u()]),
        Effect::Draw {
            who: Selector::You,
            amount: Value::DistinctColorsAmong(Box::new(Selector::EachPermanent(
                R::ControlledByYou,
            ))),
        },
    )
}

/// Soul of Innistrad — {4}{B}{B} 6/6 with deathtouch. {3}{B}{B}: up to three
/// creature cards from your graveyard to hand.
pub fn soul_of_innistrad() -> CardDefinition {
    soul(
        "Soul of Innistrad",
        cost(&[generic(4), b(), b()]),
        vec![Keyword::Deathtouch],
        cost(&[generic(3), b(), b()]),
        Effect::MoveChosen {
            from: Selector::EachMatching {
                zone: ZoneRef::Graveyard(PlayerRef::You),
                filter: R::Creature,
            },
            filter: None,
            count: Value::Const(3),
            up_to: true,
            to: ZoneDest::Hand(PlayerRef::You),
        },
    )
}

/// Soul of Shandalar — {4}{R}{R} 6/6 with first strike. {3}{R}{R}: 3 damage
/// to a player or planeswalker.
pub fn soul_of_shandalar() -> CardDefinition {
    soul(
        "Soul of Shandalar",
        cost(&[generic(4), r(), r()]),
        vec![Keyword::FirstStrike],
        cost(&[generic(3), r(), r()]),
        Effect::DealDamage {
            to: target_filtered(R::Player.or(R::Planeswalker)),
            amount: Value::Const(3),
        },
    )
}

/// Soul of Zendikar — {4}{G}{G} 6/6 with reach. {3}{G}{G}: a 3/3 Beast.
pub fn soul_of_zendikar() -> CardDefinition {
    soul(
        "Soul of Zendikar",
        cost(&[generic(4), g(), g()]),
        vec![Keyword::Reach],
        cost(&[generic(3), g(), g()]),
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: beast_token(),
        },
    )
}

/// Soul of New Phyrexia — {6} 6/6 artifact creature with trample. {5}: your
/// permanents gain indestructible.
pub fn soul_of_new_phyrexia() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Avatar],
            ..Default::default()
        },
        ..soul(
            "Soul of New Phyrexia",
            cost(&[generic(6)]),
            vec![Keyword::Trample],
            cost(&[generic(5)]),
            Effect::GrantKeyword {
                what: Selector::EachPermanent(R::ControlledByYou),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        )
    }
}

// ── The land-type cycle ──────────────────────────────────────────────────────

/// A creature that grows while you control a `lt` land, plus an off-color
/// activated ability.
fn land_gated(
    name: &'static str,
    mana: ManaCost,
    pt: (i32, i32),
    ct: Vec<CreatureType>,
    kw: Vec<Keyword>,
    lt: LandType,
    ability: ActivatedAbility,
) -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+1 as long as you control the gating land type.",
            effect: StaticEffect::PumpSelfIf {
                condition: controls_land(lt),
                power: 1,
                toughness: 1,
                keywords: vec![],
            },
        }],
        activated_abilities: vec![ability],
        ..creature(name, mana, pt.0, pt.1, ct, kw)
    }
}

/// Sunblade Elf — {G} 1/1. +1/+1 with a Plains; {4}{W}: team +1/+1.
pub fn sunblade_elf() -> CardDefinition {
    land_gated(
        "Sunblade Elf",
        cost(&[g()]),
        (1, 1),
        vec![CreatureType::Elf, CreatureType::Warrior],
        vec![],
        LandType::Plains,
        ActivatedAbility {
            mana_cost: cost(&[generic(4), w()]),
            effect: Effect::PumpPT {
                what: crate::effect::shortcut::each_your_creature(),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        },
    )
}

/// Dauntless River Marshal — {1}{W} 2/1. +1/+1 with an Island; {3}{U}: tap a
/// creature.
pub fn dauntless_river_marshal() -> CardDefinition {
    land_gated(
        "Dauntless River Marshal",
        cost(&[generic(1), w()]),
        (2, 1),
        vec![CreatureType::Human, CreatureType::Soldier],
        vec![],
        LandType::Island,
        ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        },
    )
}

/// Jorubai Murk Lurker — {2}{U} 1/3. +1/+1 with a Swamp; {1}{B}: lifelink.
pub fn jorubai_murk_lurker() -> CardDefinition {
    land_gated(
        "Jorubai Murk Lurker",
        cost(&[generic(2), u()]),
        (1, 3),
        vec![CreatureType::Leech],
        vec![],
        LandType::Swamp,
        ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        },
    )
}

/// Nightfire Giant — {4}{B} 4/3. +1/+1 with a Mountain; {4}{R}: 2 damage.
pub fn nightfire_giant() -> CardDefinition {
    land_gated(
        "Nightfire Giant",
        cost(&[generic(4), b()]),
        (4, 3),
        vec![CreatureType::Zombie, CreatureType::Giant],
        vec![],
        LandType::Mountain,
        ActivatedAbility {
            mana_cost: cost(&[generic(4), r()]),
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
            ..Default::default()
        },
    )
}

/// Kird Chieftain — {3}{R} 3/3. +1/+1 with a Forest; {4}{G}: +2/+2 and
/// trample.
pub fn kird_chieftain() -> CardDefinition {
    land_gated(
        "Kird Chieftain",
        cost(&[generic(3), r()]),
        (3, 3),
        vec![CreatureType::Ape],
        vec![],
        LandType::Forest,
        ActivatedAbility {
            mana_cost: cost(&[generic(4), g()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        },
    )
}

// ── Creatures ────────────────────────────────────────────────────────────────

/// Aeronaut Tinkerer — {2}{U} 2/3. Flying while you control an artifact.
pub fn aeronaut_tinkerer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature has flying as long as you control an artifact.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::Flying,
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)),
                    n: Value::Const(1),
                },
            },
        }],
        ..creature(
            "Aeronaut Tinkerer",
            cost(&[generic(2), u()]),
            2,
            3,
            vec![CreatureType::Human, CreatureType::Artificer],
            vec![],
        )
    }
}

/// Scrapyard Mongrel — {3}{R} 3/3. +2/+0 and trample while you control an
/// artifact.
pub fn scrapyard_mongrel() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as you control an artifact, this gets +2/+0 and has trample.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)),
                    n: Value::Const(1),
                },
                power: 2,
                toughness: 0,
                keywords: vec![Keyword::Trample],
            },
        }],
        ..creature(
            "Scrapyard Mongrel",
            cost(&[generic(3), r()]),
            3,
            3,
            vec![CreatureType::Dog],
            vec![],
        )
    }
}

/// Warden of the Beyond — {2}{W} 2/2 with vigilance. +2/+2 while an opponent
/// owns a card in exile.
pub fn warden_of_the_beyond() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This gets +2/+2 as long as an opponent owns a card in exile.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachMatching {
                        zone: ZoneRef::Exile,
                        filter: R::OwnedByYou.negate(),
                    },
                    n: Value::Const(1),
                },
                power: 2,
                toughness: 2,
                keywords: vec![],
            },
        }],
        ..creature(
            "Warden of the Beyond",
            cost(&[generic(2), w()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![Keyword::Vigilance],
        )
    }
}

/// Kalonian Twingrove — {5}{G} */* Treefolk Warrior. Stats and a token copy
/// both track your Forests.
pub fn kalonian_twingrove() -> CardDefinition {
    let forest_count = || {
        Value::count(Selector::EachPermanent(
            R::HasLandType(LandType::Forest).and(R::ControlledByYou),
        ))
    };
    CardDefinition {
        dynamic_pt: Some(DynamicPt::PermanentsControlledMatching {
            base_p: 0,
            base_t: 0,
            filter: Box::new(R::HasLandType(LandType::Forest).and(R::ControlledByYou)),
        }),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Treefolk Warrior".into(),
                colors: vec![Color::Green],
                card_types: vec![CardType::Creature],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Treefolk, CreatureType::Warrior],
                    ..Default::default()
                },
                dynamic_pt: Some((forest_count(), forest_count())),
                ..Default::default()
            },
        })],
        ..creature(
            "Kalonian Twingrove",
            cost(&[generic(5), g()]),
            0,
            0,
            vec![CreatureType::Treefolk, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Undergrowth Scavenger — {3}{G} 0/0 Fungus Horror. Enters with a +1/+1
/// counter per creature card in all graveyards.
pub fn undergrowth_scavenger() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::count(Selector::EachMatching {
                zone: ZoneRef::Graveyard(PlayerRef::EachPlayer),
                filter: R::Creature,
            }),
        )),
        ..creature(
            "Undergrowth Scavenger",
            cost(&[generic(3), g()]),
            0,
            0,
            vec![CreatureType::Fungus, CreatureType::Horror],
            vec![],
        )
    }
}

/// Netcaster Spider — {2}{G} 2/3 with reach; +2/+0 when it blocks a flyer.
pub fn netcaster_spider() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource).with_filter(
                Predicate::EntityMatches {
                    what: Selector::BlockedAttacker,
                    filter: R::HasKeyword(Keyword::Flying),
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Netcaster Spider",
            cost(&[generic(2), g()]),
            2,
            3,
            vec![CreatureType::Spider],
            vec![Keyword::Reach],
        )
    }
}

/// Phytotitan — {4}{G}{G} 7/2. It returns tapped at your next upkeep when it
/// dies.
pub fn phytotitan() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::ReturnSelfAtNextUpkeepTapped)],
        ..creature(
            "Phytotitan",
            cost(&[generic(4), g(), g()]),
            7,
            2,
            vec![CreatureType::Plant, CreatureType::Elemental],
            vec![],
        )
    }
}

/// Invasive Species — {2}{G} 3/3. ETB: bounce another permanent you control.
pub fn invasive_species() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MoveChosen {
            from: Selector::EachPermanent(R::ControlledByYou),
            filter: None,
            count: Value::Const(1),
            up_to: false,
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        })],
        ..creature(
            "Invasive Species",
            cost(&[generic(2), g()]),
            3,
            3,
            vec![CreatureType::Insect],
            vec![],
        )
    }
}

/// Shaman of Spring — {3}{G} 2/2. ETB: draw a card.
pub fn shaman_of_spring() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::etb_draw(1)],
        ..creature(
            "Shaman of Spring",
            cost(&[generic(3), g()]),
            2,
            2,
            vec![CreatureType::Elf, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Rotfeaster Maggot — {4}{B} 3/5. ETB: exile a creature card from a
/// graveyard and gain life equal to its toughness.
pub fn rotfeaster_maggot() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::ToughnessOf(Box::new(Selector::Target(0))),
            },
            Effect::Exile { what: target_filtered(R::Creature.and(R::InGraveyard)) },
        ]))],
        ..creature(
            "Rotfeaster Maggot",
            cost(&[generic(4), b()]),
            3,
            5,
            vec![CreatureType::Insect],
            vec![],
        )
    }
}

/// Resolute Archangel — {5}{W}{W} 4/4 with flying. ETB: your life total
/// becomes your starting life total.
pub fn resolute_archangel() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::SetLifeTotal {
            who: Selector::You,
            amount: Value::StartingLifeTotal,
        })],
        ..creature(
            "Resolute Archangel",
            cost(&[generic(5), w(), w()]),
            4,
            4,
            vec![CreatureType::Angel],
            vec![Keyword::Flying],
        )
    }
}

/// Nimbus of the Isles — {4}{U} 3/3 flier.
pub fn nimbus_of_the_isles() -> CardDefinition {
    creature(
        "Nimbus of the Isles",
        cost(&[generic(4), u()]),
        3,
        3,
        vec![CreatureType::Elemental],
        vec![Keyword::Flying],
    )
}

/// Geist of the Moors — {1}{W}{W} 3/1 flier.
pub fn geist_of_the_moors() -> CardDefinition {
    creature(
        "Geist of the Moors",
        cost(&[generic(1), w(), w()]),
        3,
        1,
        vec![CreatureType::Spirit],
        vec![Keyword::Flying],
    )
}

/// Sungrace Pegasus — {1}{W} 1/2 with flying and lifelink.
pub fn sungrace_pegasus() -> CardDefinition {
    creature(
        "Sungrace Pegasus",
        cost(&[generic(1), w()]),
        1,
        2,
        vec![CreatureType::Pegasus],
        vec![Keyword::Flying, Keyword::Lifelink],
    )
}

/// Krenko's Enforcer — {1}{R}{R} 2/2 with intimidate.
pub fn krenkos_enforcer() -> CardDefinition {
    creature(
        "Krenko's Enforcer",
        cost(&[generic(1), r(), r()]),
        2,
        2,
        vec![CreatureType::Goblin, CreatureType::Warrior],
        vec![Keyword::Intimidate],
    )
}

/// Xathrid Slyblade — {2}{B} 2/1 with hexproof.
pub fn xathrid_slyblade() -> CardDefinition {
    creature(
        "Xathrid Slyblade",
        cost(&[generic(2), b()]),
        2,
        1,
        vec![CreatureType::Human, CreatureType::Assassin],
        vec![Keyword::Hexproof],
    )
}

/// Witch's Familiar — {2}{B} 2/3 vanilla.
pub fn witchs_familiar() -> CardDefinition {
    creature(
        "Witch's Familiar",
        cost(&[generic(2), b()]),
        2,
        3,
        vec![CreatureType::Frog],
        vec![],
    )
}

/// Carrion Crow — {2}{B} 2/2 flier that enters tapped.
pub fn carrion_crow() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        ..creature(
            "Carrion Crow",
            cost(&[generic(2), b()]),
            2,
            2,
            vec![CreatureType::Zombie, CreatureType::Bird],
            vec![Keyword::Flying],
        )
    }
}

/// Shadowcloak Vampire — {4}{B} 4/3. Pay 2 life: gains flying.
pub fn shadowcloak_vampire() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            life_cost: 2,
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Shadowcloak Vampire",
            cost(&[generic(4), b()]),
            4,
            3,
            vec![CreatureType::Vampire],
            vec![],
        )
    }
}

/// Amphin Pathmage — {3}{U} 3/2. {2}{U}: target creature can't be blocked.
pub fn amphin_pathmage() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Amphin Pathmage",
            cost(&[generic(3), u()]),
            3,
            2,
            vec![CreatureType::Salamander, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Research Assistant — {1}{U} 1/3. {3}{U}, {T}: loot.
pub fn research_assistant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Research Assistant",
            cost(&[generic(1), u()]),
            1,
            3,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Carnivorous Moss-Beast — {4}{G}{G} 4/5. {5}{G}{G}: a +1/+1 counter.
pub fn carnivorous_moss_beast() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), g(), g()]),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..creature(
            "Carnivorous Moss-Beast",
            cost(&[generic(4), g(), g()]),
            4,
            5,
            vec![CreatureType::Plant, CreatureType::Elemental, CreatureType::Beast],
            vec![],
        )
    }
}

/// Miner's Bane — {4}{R}{R} 6/3. {2}{R}: +1/+0 and trample.
pub fn miners_bane() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Miner's Bane",
            cost(&[generic(4), r(), r()]),
            6,
            3,
            vec![CreatureType::Elemental],
            vec![],
        )
    }
}

/// Wall of Limbs — {2}{B} 0/3 with defender. Life gain grows it; sac it to
/// drain for its power.
pub fn wall_of_limbs() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), b(), b()]),
            sac_cost: true,
            effect: Effect::LoseLife {
                who: target_filtered(R::Player),
                amount: Value::PowerOf(Box::new(Selector::This)),
            },
            ..Default::default()
        }],
        ..creature(
            "Wall of Limbs",
            cost(&[generic(2), b()]),
            0,
            3,
            vec![CreatureType::Zombie, CreatureType::Wall],
            vec![Keyword::Defender],
        )
    }
}

/// Cruel Sadist — {B} 1/1. Grow it with life, then throw the counters as
/// damage.
pub fn cruel_sadist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                tap_cost: true,
                life_cost: 1,
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), b()]),
                tap_cost: true,
                remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
                effect: Effect::DealDamage {
                    to: target_filtered(R::Creature),
                    amount: Value::Const(1),
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Cruel Sadist",
            cost(&[b()]),
            1,
            1,
            vec![CreatureType::Human, CreatureType::Assassin],
            vec![],
        )
    }
}

/// Altac Bloodseeker — {1}{R} 2/1. An opponent's creature dying pumps it.
pub fn altac_bloodseeker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeywords {
                    what: Selector::This,
                    keywords: vec![Keyword::FirstStrike, Keyword::Haste],
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..creature(
            "Altac Bloodseeker",
            cost(&[generic(1), r()]),
            2,
            1,
            vec![CreatureType::Human, CreatureType::Berserker],
            vec![],
        )
    }
}

/// Blood Host — {3}{B}{B} 3/3. {1}{B}, Sacrifice another creature: a +1/+1
/// counter and 2 life.
pub fn blood_host() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Blood Host",
            cost(&[generic(3), b(), b()]),
            3,
            3,
            vec![CreatureType::Vampire],
            vec![],
        )
    }
}

/// Siege Dragon — {5}{R}{R} 5/5 flier. ETB wipes Walls; attacking, it sweeps
/// the ground.
pub fn siege_dragon() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Destroy {
                what: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Wall).and(R::ControlledByOpponent),
                ),
            }),
            on_attack(Effect::DealDamage {
                to: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByOpponent)
                        .and(R::HasKeyword(Keyword::Flying).negate()),
                ),
                amount: Value::Const(2),
            }),
        ],
        ..creature(
            "Siege Dragon",
            cost(&[generic(5), r(), r()]),
            5,
            5,
            vec![CreatureType::Dragon],
            vec![Keyword::Flying],
        )
    }
}

// ── Spells ───────────────────────────────────────────────────────────────────

/// Void Snare — {U} Sorcery. Bounce a nonland permanent.
pub fn void_snare() -> CardDefinition {
    spell(
        "Void Snare",
        cost(&[u()]),
        CardType::Sorcery,
        Effect::Move {
            what: target_filtered(R::Nonland),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
    )
}

/// Chronostutter — {5}{U} Instant. Target creature goes second from the top.
pub fn chronostutter() -> CardDefinition {
    spell(
        "Chronostutter",
        cost(&[generic(5), u()]),
        CardType::Instant,
        Effect::PutIntoLibraryBeneathTop {
            what: target_filtered(R::Creature),
            count: Value::Const(1),
        },
    )
}

/// Flesh to Dust — {3}{B}{B} Instant. Destroy a creature; no regeneration.
pub fn flesh_to_dust() -> CardDefinition {
    spell(
        "Flesh to Dust",
        cost(&[generic(3), b(), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::CantBeRegeneratedThisTurn { what: target_filtered(R::Creature) },
            Effect::Destroy { what: Selector::Target(0) },
        ]),
    )
}

/// Pillar of Light — {2}{W} Instant. Exile a creature with toughness 4+.
pub fn pillar_of_light() -> CardDefinition {
    spell(
        "Pillar of Light",
        cost(&[generic(2), w()]),
        CardType::Instant,
        Effect::Exile { what: target_filtered(R::Creature.and(R::ToughnessAtLeast(4))) },
    )
}

/// Blastfire Bolt — {5}{R} Instant. 5 damage to a creature; blow up its
/// Equipment.
pub fn blastfire_bolt() -> CardDefinition {
    spell(
        "Blastfire Bolt",
        cost(&[generic(5), r()]),
        CardType::Instant,
        // The Equipment is destroyed first so it still resolves when the 5
        // damage is lethal (CR 608.2: SBAs wait for the whole resolution).
        Effect::Seq(vec![
            Effect::Destroy {
                what: Selector::AttachedToMe(Box::new(target_filtered(R::Creature))),
            },
            Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(5) },
        ]),
    )
}

/// In Garruk's Wake — {7}{B}{B} Sorcery. Wipe every creature and planeswalker
/// you don't control.
pub fn in_garruks_wake() -> CardDefinition {
    spell(
        "In Garruk's Wake",
        cost(&[generic(7), b(), b()]),
        CardType::Sorcery,
        Effect::Destroy {
            what: Selector::EachPermanent(
                (R::Creature.or(R::Planeswalker)).and(R::ControlledByOpponent),
            ),
        },
    )
}

/// Sanctified Charge — {4}{W} Instant. Team +2/+1; white creatures also get
/// first strike.
pub fn sanctified_charge() -> CardDefinition {
    spell(
        "Sanctified Charge",
        cost(&[generic(4), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: crate::effect::shortcut::each_your_creature(),
                power: Value::Const(2),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::HasColor(Color::White)),
                ),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Act on Impulse — {2}{R} Sorcery. Exile the top three; play them this turn.
pub fn act_on_impulse() -> CardDefinition {
    spell(
        "Act on Impulse",
        cost(&[generic(2), r()]),
        CardType::Sorcery,
        Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::Const(3),
            duration: crate::card::MayPlayDuration::EndOfThisTurn,
            pay_any_color: false,
            pay_own_cost: true,
            uncast_penalty: None,
        },
    )
}

/// Statute of Denial — {2}{U}{U} Instant. Counter a spell; loot if you control
/// a blue creature.
pub fn statute_of_denial() -> CardDefinition {
    spell(
        "Statute of Denial",
        cost(&[generic(2), u(), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            crate::effect::shortcut::counter_target_spell(),
            Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::HasColor(Color::Blue)),
                    ),
                    n: Value::Const(1),
                },
                then: Box::new(Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(1),
                        random: false,
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Hunter's Ambush — {2}{G} Instant. Fog everything nongreen.
pub fn hunters_ambush() -> CardDefinition {
    spell(
        "Hunter's Ambush",
        cost(&[generic(2), g()]),
        CardType::Instant,
        Effect::PreventAllCombatDamageByMatchingThisTurn {
            filter: R::HasColor(Color::Green).negate(),
        },
    )
}

// ── Enchantments and artifacts ───────────────────────────────────────────────

/// Marked by Honor — {3}{W} Aura. +2/+2 and vigilance.
pub fn marked_by_honor() -> CardDefinition {
    aura(
        "Marked by Honor",
        cost(&[generic(3), w()]),
        R::Creature,
        EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Vigilance],
            ..Default::default()
        },
    )
}

/// Vineweft — {G} Aura. +1/+1; {4}{G}: return it from your graveyard.
pub fn vineweft() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g()]),
            from_graveyard: true,
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..aura(
            "Vineweft",
            cost(&[g()]),
            R::Creature,
            EquipBonus { power: 1, toughness: 1, ..Default::default() },
        )
    }
}

/// Eternal Thirst — {1}{B} Aura. Lifelink plus a counter whenever an
/// opponent's creature dies.
pub fn eternal_thirst() -> CardDefinition {
    aura(
        "Eternal Thirst",
        cost(&[generic(1), b()]),
        R::Creature,
        EquipBonus {
            keywords: vec![Keyword::Lifelink],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            }],
            ..Default::default()
        },
    )
}

/// Inferno Fist — {1}{R} Aura. +2/+0; {R}, sacrifice it: 2 damage.
pub fn inferno_fist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_cost: true,
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
            ..Default::default()
        }],
        ..aura(
            "Inferno Fist",
            cost(&[generic(1), r()]),
            R::Creature.and(R::ControlledByYou),
            EquipBonus { power: 2, ..Default::default() },
        )
    }
}

/// Military Intelligence — {1}{U} Enchantment. Attacking with two or more
/// draws a card.
pub fn military_intelligence() -> CardDefinition {
    CardDefinition {
        name: "Military Intelligence",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl).with_filter(
                Predicate::AttackedWithCountAtLeast { who: PlayerRef::You, at_least: 2 },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Profane Memento — {1} Artifact. A creature card hitting an opponent's
/// graveyard gains you 1 life.
pub fn profane_memento() -> CardDefinition {
    CardDefinition {
        name: "Profane Memento",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Sacred Armory — {2} Artifact. {2}: +1/+0.
pub fn sacred_armory() -> CardDefinition {
    CardDefinition {
        name: "Sacred Armory",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tyrant's Machine — {2} Artifact. {4}, {T}: tap a creature.
pub fn tyrants_machine() -> CardDefinition {
    CardDefinition {
        name: "Tyrant's Machine",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Perilous Vault — {4} Artifact. {5}, {T}, Exile it: exile all nonland
/// permanents.
pub fn perilous_vault() -> CardDefinition {
    CardDefinition {
        name: "Perilous Vault",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            tap_cost: true,
            exile_self_cost: true,
            effect: Effect::Exile { what: Selector::EachPermanent(R::Nonland) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Brawler's Plate — {3} Equipment. +2/+2 and trample; equip {4}.
pub fn brawlers_plate() -> CardDefinition {
    equipment(
        "Brawler's Plate",
        cost(&[generic(3)]),
        cost(&[generic(4)]),
        EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Trample],
            ..Default::default()
        },
    )
}

/// Hot Soup — {1} Equipment. Unblockable, but any damage kills the bearer;
/// equip {3}.
pub fn hot_soup() -> CardDefinition {
    equipment(
        "Hot Soup",
        cost(&[generic(1)]),
        cost(&[generic(3)]),
        EquipBonus {
            keywords: vec![Keyword::Unblockable],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
                effect: Effect::Destroy { what: Selector::This },
            }],
            ..Default::default()
        },
    )
}

/// Sliver Hive — a Sliver-matters land.
pub fn sliver_hive() -> CardDefinition {
    CardDefinition {
        name: "Sliver Hive",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::AnyOneColor(Value::Const(1)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(5)]),
                tap_cost: true,
                condition: Some(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Sliver).and(R::ControlledByYou),
                    ),
                    n: Value::Const(1),
                }),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: TokenDefinition {
                        name: "Sliver".into(),
                        power: 1,
                        toughness: 1,
                        card_types: vec![CardType::Creature],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Sliver],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Genesis Hydra — {X}{G}{G} 0/0. Cast trigger digs X deep for a nonland
/// permanent; it enters with X counters.
pub fn genesis_hydra() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        triggered_abilities: vec![crate::effect::shortcut::on_cast(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::XFromCost,
            rest_to_graveyard: false,
            pick_filter: Some(R::Nonland.and(R::ManaValueAtMostXFromCost)),
            take: Some(Value::Const(1)),
            to_battlefield: true,
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: false,
            optional: true,
            picked_lands_to_battlefield: false,
            rest_bottom_random: true,
        })],
        ..creature(
            "Genesis Hydra",
            cost(&[x(), g(), g()]),
            0,
            0,
            vec![CreatureType::Plant, CreatureType::Hydra],
            vec![],
        )
    }
}

// ── The M15 tail ─────────────────────────────────────────────────────────────

/// Burning Anger — {4}{R} Aura. Enchanted creature has "{T}: deal damage
/// equal to its power to any target."
pub fn burning_anger() -> CardDefinition {
    aura(
        "Burning Anger",
        cost(&[generic(4), r()]),
        R::Creature,
        EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::DealDamage {
                    to: target_any(),
                    amount: Value::PowerOf(Box::new(Selector::This)),
                },
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

/// Ensoul Artifact — {1}{U} Aura on an artifact. The host becomes a 5/5
/// creature in addition to its other types.
pub fn ensoul_artifact() -> CardDefinition {
    aura(
        "Ensoul Artifact",
        cost(&[generic(1), u()]),
        R::Artifact,
        EquipBonus {
            set_base_pt: Some((5, 5)),
            set_card_types: Some(vec![CardType::Artifact, CardType::Creature]),
            ..Default::default()
        },
    )
}

/// Brood Keeper — {3}{R} 2/3 Human Shaman. An Aura landing on it mints a
/// firebreathing 2/2 Dragon.
pub fn brood_keeper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AuraAttached, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Dragon".into(),
                    power: 2,
                    toughness: 2,
                    colors: vec![Color::Red],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Dragon],
                        ..Default::default()
                    },
                    keywords: vec![Keyword::Flying],
                    activated_abilities: vec![ActivatedAbility {
                        mana_cost: cost(&[r()]),
                        effect: Effect::PumpPT {
                            what: Selector::This,
                            power: Value::Const(1),
                            toughness: Value::Const(0),
                            duration: Duration::EndOfTurn,
                        },
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            },
        }],
        ..creature(
            "Brood Keeper",
            cost(&[generic(3), r()]),
            2,
            3,
            vec![CreatureType::Human, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Necromancer's Stockpile — {1}{B} Enchantment. {1}{B}, discard a creature
/// card: draw; a discarded Zombie also mints a tapped 2/2 Zombie.
pub fn necromancers_stockpile() -> CardDefinition {
    CardDefinition {
        name: "Necromancer's Stockpile",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            discard_cost: Some((R::Creature, 1)),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::If {
                    cond: Predicate::LastDiscardedHasCreatureType(CreatureType::Zombie),
                    then: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::Const(1),
                        definition: TokenDefinition {
                            name: "Zombie".into(),
                            power: 2,
                            toughness: 2,
                            colors: vec![Color::Black],
                            card_types: vec![CardType::Creature],
                            subtypes: Subtypes {
                                creature_types: vec![CreatureType::Zombie],
                                ..Default::default()
                            },
                            static_abilities: vec![StaticAbility {
                                description: "This token enters tapped.",
                                effect: StaticEffect::EntersTapped {
                                    applies_to: Selector::This,
                                },
                            }],
                            ..Default::default()
                        },
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// First Response — {3}{W} Enchantment. Losing life mints a Soldier at the
/// next upkeep. (The engine's life-loss tally is turn-scoped, so the check
/// reads "this turn" rather than the printed "last turn".)
pub fn first_response() -> CardDefinition {
    CardDefinition {
        name: "First Response",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::AnyPlayer,
            )
            .with_filter(Predicate::PlayerLostLifeThisTurn { who: PlayerRef::You }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Soldier".into(),
                    power: 1,
                    toughness: 1,
                    colors: vec![Color::White],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Soldier],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// Feast on the Fallen — {2}{B} Enchantment. An opponent's life loss grows a
/// creature you control at the next upkeep. (Same turn-scoped tally caveat as
/// First Response.)
pub fn feast_on_the_fallen() -> CardDefinition {
    CardDefinition {
        name: "Feast on the Fallen",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::AnyPlayer,
            )
            .with_filter(Predicate::PlayerLostLifeThisTurn { who: PlayerRef::EachOpponent }),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Avarice Amulet — {4} Equipment. +2/+0, vigilance, and an upkeep draw;
/// equip {2}.
pub fn avarice_amulet() -> CardDefinition {
    equipment(
        "Avarice Amulet",
        cost(&[generic(4)]),
        cost(&[generic(2)]),
        EquipBonus {
            power: 2,
            keywords: vec![Keyword::Vigilance],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            }],
            ..Default::default()
        },
    )
}

/// Kapsho Kitefins — {4}{U}{U} 3/3 flier. Any creature you control entering
/// taps one of theirs.
pub fn kapsho_kitefins() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::Tap {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            },
        }],
        ..creature(
            "Kapsho Kitefins",
            cost(&[generic(4), u(), u()]),
            3,
            3,
            vec![CreatureType::Fish],
            vec![Keyword::Flying],
        )
    }
}

/// Spirit Bonds — {1}{W} Enchantment. Pay {W} when a nontoken creature enters
/// for a Spirit; sacrifice a Spirit to make a non-Spirit indestructible.
pub fn spirit_bonds() -> CardDefinition {
    CardDefinition {
        name: "Spirit Bonds",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::NotToken),
                }),
            effect: Effect::MayPay {
                description: "Pay {W} to create a 1/1 Spirit?".into(),
                mana_cost: cost(&[w()]),
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: spirit_token(),
                }),
                else_: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Spirit), 1)),
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    R::Creature.and(R::HasCreatureType(CreatureType::Spirit).negate()),
                ),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── The M15 tail: planeswalkers, rares, and the last uncommons ───────────────

/// Aetherspouts — {3}{U}{U} Instant. Each attacking creature's owner puts it
/// on top or bottom of their library.
pub fn aetherspouts() -> CardDefinition {
    spell(
        "Aetherspouts",
        cost(&[generic(3), u(), u()]),
        CardType::Instant,
        Effect::Move {
            what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOfMoved,
                pos: crate::effect::LibraryPosition::OwnerChoice,
            },
        },
    )
}

/// Aggressive Mining — {3}{R} Enchantment. You can't play lands; once each
/// turn, sacrifice a land to draw two cards.
pub fn aggressive_mining() -> CardDefinition {
    CardDefinition {
        name: "Aggressive Mining",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "You can't play lands.",
            effect: StaticEffect::ControllerCantPlayLands,
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Land.and(R::ControlledByYou), 1)),
            once_per_turn: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ajani Steadfast — {3}{W} planeswalker, loyalty 4.
pub fn ajani_steadfast() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype, Supertype};
    CardDefinition {
        name: "Ajani Steadfast",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Ajani],
            ..Default::default()
        },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Seq(vec![
                        Effect::PumpPT {
                            what: Selector::Target(0),
                            power: Value::ONE,
                            toughness: Value::ONE,
                            duration: Duration::EndOfTurn,
                        },
                        Effect::GrantKeyword {
                            what: Selector::Target(0),
                            keyword: Keyword::FirstStrike,
                            duration: Duration::EndOfTurn,
                        },
                        Effect::GrantKeyword {
                            what: Selector::Target(0),
                            keyword: Keyword::Vigilance,
                            duration: Duration::EndOfTurn,
                        },
                        Effect::GrantKeyword {
                            what: Selector::Target(0),
                            keyword: Keyword::Lifelink,
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                    Effect::AddCounter {
                        what: Selector::EachPermanent(
                            R::Planeswalker.and(R::ControlledByYou).and(R::OtherThanSource),
                        ),
                        kind: CounterType::Loyalty,
                        amount: Value::ONE,
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Ajani Steadfast".into(),
                    triggered: vec![],
                    statics: vec![StaticAbility {
                        description: "Prevent all but 1 damage to you and your planeswalkers.",
                        effect: StaticEffect::PreventAllButOneDamageToYouAndYourPlaneswalkers,
                    }],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Avacyn, Guardian Angel — {2}{W}{W}{W} 5/4. Two color-of-your-choice
/// prevention shields, one for a creature and one for a player/planeswalker.
pub fn avacyn_guardian_angel() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), w()]),
                effect: Effect::PreventAllDamageFromChosenColorThisTurn {
                    target: target_filtered(R::Creature),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(5), w(), w()]),
                effect: Effect::PreventAllDamageFromChosenColorThisTurn {
                    target: target_filtered(R::Player.or(R::Planeswalker)),
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Avacyn, Guardian Angel",
            cost(&[generic(2), w(), w(), w()]),
            5,
            4,
            vec![CreatureType::Angel],
            vec![Keyword::Flying, Keyword::Vigilance],
        )
    }
}

/// Boonweaver Giant — {6}{W} 4/4. Enters searching graveyard, hand, and
/// library for an Aura to attach to itself.
pub fn boonweaver_giant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search for an Aura to attach?".into(),
            body: Box::new(Effect::SearchAuraAttachToSource),
        })],
        ..creature(
            "Boonweaver Giant",
            cost(&[generic(6), w()]),
            4,
            4,
            vec![CreatureType::Giant, CreatureType::Monk],
            vec![],
        )
    }
}

/// Chief Engineer — {1}{U} 1/3. Artifact spells you cast have convoke.
pub fn chief_engineer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Artifact spells you cast have convoke.",
            effect: StaticEffect::GrantConvokeToSpells { filter: R::Artifact },
        }],
        ..creature(
            "Chief Engineer",
            cost(&[generic(1), u()]),
            1,
            3,
            vec![CreatureType::Vedalken, CreatureType::Artificer],
            vec![],
        )
    }
}

/// Constricting Sliver — {5}{W} 3/3. Sliver creatures you control gain an
/// exile-until-this-leaves ETB.
pub fn constricting_sliver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Sliver creatures you control have an exile-on-enter trigger.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature
                    .and(R::ControlledByYou)
                    .and(R::HasCreatureType(CreatureType::Sliver)),
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                    effect: Effect::MayDo {
                        description: "Exile target creature an opponent controls?".into(),
                        body: Box::new(Effect::ExileUntilSourceLeaves {
                            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                            return_to: crate::card::ExileReturnZone::Battlefield,
                        }),
                    },
                }),
            },
        }],
        ..creature(
            "Constricting Sliver",
            cost(&[generic(5), w()]),
            3,
            3,
            vec![CreatureType::Sliver],
            vec![],
        )
    }
}

/// Garruk, Apex Predator — {5}{B}{G} planeswalker, loyalty 5.
pub fn garruk_apex_predator() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype, Supertype};
    CardDefinition {
        name: "Garruk, Apex Predator",
        cost: cost(&[generic(5), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Garruk],
            ..Default::default()
        },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Destroy { what: target_filtered(R::Planeswalker) },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Beast".into(),
                        power: 3,
                        toughness: 3,
                        colors: vec![Color::Black],
                        card_types: vec![CardType::Creature],
                        keywords: vec![Keyword::Deathtouch],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Beast],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Seq(vec![
                    Effect::GainLife {
                        who: Selector::You,
                        amount: Value::ToughnessOf(Box::new(Selector::Target(0))),
                    },
                    Effect::Destroy { what: target_filtered(R::Creature) },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::EachOpponent,
                    name: "Garruk, Apex Predator".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::Attacks, EventScope::OpponentControl),
                        effect: Effect::Seq(vec![
                            Effect::PumpPT {
                                what: Selector::TriggerSource,
                                power: Value::Const(5),
                                toughness: Value::Const(5),
                                duration: Duration::EndOfTurn,
                            },
                            Effect::GrantKeyword {
                                what: Selector::TriggerSource,
                                keyword: Keyword::Trample,
                                duration: Duration::EndOfTurn,
                            },
                        ]),
                    }],
                    statics: vec![],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Generator Servant — {1}{R} 2/1. Sacrifices for {C}{C} that hastes the
/// creature spell it funds.
pub fn generator_servant() -> CardDefinition {
    use crate::mana::SpendRestriction;
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: crate::effect::ManaPayload::Restricted(
                    Box::new(crate::effect::ManaPayload::Colorless(Value::Const(2))),
                    SpendRestriction::CreatureHaste,
                ),
            },
            ..Default::default()
        }],
        ..creature(
            "Generator Servant",
            cost(&[generic(1), r()]),
            2,
            1,
            vec![CreatureType::Elemental],
            vec![],
        )
    }
}

/// Glacial Crasher — {4}{U}{U} 5/5 trample that can't attack without a
/// Mountain on the battlefield.
pub fn glacial_crasher() -> CardDefinition {
    creature(
        "Glacial Crasher",
        cost(&[generic(4), u(), u()]),
        5,
        5,
        vec![CreatureType::Elemental],
        vec![
            Keyword::Trample,
            Keyword::CantAttackUnlessLandTypeOnBattlefield(LandType::Mountain),
        ],
    )
}

/// Goblin Kaboomist — {1}{R} 1/2. Mints a Land Mine each upkeep, then risks
/// two damage to itself on a lost coin flip.
pub fn goblin_kaboomist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crabomination_base::turn_step::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Land Mine".into(),
                        card_types: vec![CardType::Artifact],
                        activated_abilities: vec![ActivatedAbility {
                            mana_cost: cost(&[r()]),
                            sac_cost: true,
                            effect: Effect::DealDamage {
                                to: target_filtered(
                                    R::Creature
                                        .and(R::IsAttacking)
                                        .and(R::HasKeyword(Keyword::Flying).negate()),
                                ),
                                amount: Value::Const(2),
                            },
                            ..Default::default()
                        }],
                        ..Default::default()
                    },
                },
                Effect::FlipCoin {
                    count: Value::ONE,
                    on_heads: Box::new(Effect::Noop),
                    on_tails: Box::new(Effect::DealDamage {
                        to: Selector::This,
                        amount: Value::Const(2),
                    }),
                },
            ]),
        }],
        ..creature(
            "Goblin Kaboomist",
            cost(&[generic(1), r()]),
            1,
            2,
            vec![CreatureType::Goblin, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Jace, the Living Guildpact — {2}{U}{U} planeswalker, loyalty 5.
pub fn jace_the_living_guildpact() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype, Supertype};
    CardDefinition {
        name: "Jace, the Living Guildpact",
        cost: cost(&[generic(2), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Jace],
            ..Default::default()
        },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::LookTopKeepOneRestToGraveyard {
                    count: Value::Const(2),
                    who: None,
                    exile_rest: false,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Move {
                    what: target_filtered(
                        R::Permanent.and(R::Nonland).and(R::OtherThanSource),
                    ),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::Seq(vec![
                    Effect::ShuffleHandAndGraveyardIntoLibrary { who: PlayerRef::EachPlayer },
                    Effect::Draw { who: Selector::You, amount: Value::Const(7) },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Jalira, Master Polymorphist — {3}{U} 2/2. Polymorphs a sacrificed creature
/// into the first nonlegendary creature card off your library.
pub fn jalira_master_polymorphist() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            tap_cost: true,
            sac_other_filter: Some((R::Creature.and(R::ControlledByYou), 1)),
            effect: Effect::RevealUntilFind {
                who: PlayerRef::You,
                find: R::Creature.and(R::HasSupertype(Supertype::Legendary).negate()),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                cap: Value::Const(500),
                life_per_revealed: 0,
                miss_dest: crate::effect::RevealMissDest::BottomRandom,
            },
            ..Default::default()
        }],
        ..creature(
            "Jalira, Master Polymorphist",
            cost(&[generic(3), u()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Kurkesh, Onakke Ancient — {2}{R}{R} 4/3. Pay {R} to copy a nonmana
/// activated ability of an artifact you control.
pub fn kurkesh_onakke_ancient() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AbilityActivated, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact,
                }),
            effect: Effect::MayPay {
                description: "Pay {R} to copy that ability?".into(),
                mana_cost: cost(&[r()]),
                body: Box::new(Effect::CopyActivatedAbilityMayChooseTargets),
                else_: None,
            },
        }],
        ..creature(
            "Kurkesh, Onakke Ancient",
            cost(&[generic(2), r(), r()]),
            4,
            3,
            vec![CreatureType::Ogre, CreatureType::Spirit],
            vec![Keyword::Flying],
        )
    }
}

/// Master of Predicaments — {3}{U}{U} 4/4 flying. Its combat damage makes the
/// damaged player guess a hand card's mana value.
pub fn master_of_predicaments() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::GuessManaValueAboveElseCastFree {
                who: PlayerRef::DefendingPlayer,
                threshold: 4,
            },
        }],
        ..creature(
            "Master of Predicaments",
            cost(&[generic(3), u(), u()]),
            4,
            4,
            vec![CreatureType::Sphinx],
            vec![Keyword::Flying],
        )
    }
}

/// Mercurial Pretender — {4}{U} Shapeshifter. Enters as a copy of a creature
/// you control that can bounce itself for {2}{U}{U}.
pub fn mercurial_pretender() -> CardDefinition {
    use crate::card::EntersAsCopy;
    CardDefinition {
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature.and(R::ControlledByYou),
            extra_activated: vec![ActivatedAbility {
                mana_cost: cost(&[generic(2), u(), u()]),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..creature(
            "Mercurial Pretender",
            cost(&[generic(4), u()]),
            0,
            0,
            vec![CreatureType::Shapeshifter],
            vec![],
        )
    }
}

/// Might Makes Right — {5}{R} Enchantment. While you control every creature
/// tied for greatest power, steal one at each of your combats.
pub fn might_makes_right() -> CardDefinition {
    CardDefinition {
        name: "Might Makes Right",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crabomination_base::turn_step::TurnStep::BeginCombat),
                EventScope::YourControl,
            )
            .with_filter(Predicate::ControlsEachGreatestPowerCreature { who: PlayerRef::You }),
            effect: Effect::Seq(vec![
                Effect::GainControl {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    to: None,
                    duration: Duration::EndOfTurn,
                },
                Effect::Untap { what: Selector::Target(0), up_to: None },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Nissa, Worldwaker — {3}{G}{G} planeswalker, loyalty 3.
pub fn nissa_worldwaker() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype, Supertype};
    let animate = |what: Selector| Effect::BecomeCreature {
        what,
        power: Value::Const(4),
        toughness: Value::Const(4),
        creature_types: vec![CreatureType::Elemental],
        keywords: vec![Keyword::Trample],
        duration: Duration::Permanent,
    };
    CardDefinition {
        name: "Nissa, Worldwaker",
        cost: cost(&[generic(3), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Nissa],
            ..Default::default()
        },
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: animate(Selector::Target(0)),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Untap {
                    what: target_filtered(R::HasLandType(LandType::Forest)),
                    up_to: Some(Value::Const(4)),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::Seq(vec![
                    Effect::SearchUpToN {
                        who: PlayerRef::You,
                        filter: R::IsBasicLand,
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                        count: Value::Const(5),
                    },
                    animate(Selector::EachPermanent(R::Land.and(R::ControlledByYou))),
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ob Nixilis, Unshackled — {4}{B}{B} 4/4. Punishes opponents' tutors and
/// grows on every other creature's death.
pub fn ob_nixilis_unshackled() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::PlayerSearchedLibrary,
                    EventScope::OpponentControl,
                ),
                effect: Effect::Seq(vec![
                    Effect::Sacrifice {
                        who: Selector::Player(PlayerRef::Triggerer),
                        filter: R::Creature,
                        count: Value::ONE,
                    },
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::Triggerer),
                        amount: Value::Const(10),
                    },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::OtherThanSource,
                    }),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..creature(
            "Ob Nixilis, Unshackled",
            cost(&[generic(4), b(), b()]),
            4,
            4,
            vec![CreatureType::Demon],
            vec![Keyword::Flying, Keyword::Trample],
        )
    }
}

/// Shield of the Avatar — {1} Equipment. Prevents damage to the equipped
/// creature equal to your creature count.
pub fn shield_of_the_avatar() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Prevent damage to equipped creature equal to your creature count.",
            effect: StaticEffect::PreventDamageToAttachedPerPermanent {
                filter: R::Creature.and(R::ControlledByYou),
            },
        }],
        ..equipment(
            "Shield of the Avatar",
            cost(&[generic(1)]),
            cost(&[generic(2)]),
            EquipBonus::default(),
        )
    }
}

/// Spectra Ward — {3}{W}{W} Aura. +2/+2 and protection from each color.
pub fn spectra_ward() -> CardDefinition {
    aura(
        "Spectra Ward",
        cost(&[generic(3), w(), w()]),
        R::Creature,
        EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![
                Keyword::Protection(Color::White),
                Keyword::Protection(Color::Blue),
                Keyword::Protection(Color::Black),
                Keyword::Protection(Color::Red),
                Keyword::Protection(Color::Green),
            ],
            protection_keeps_auras: true,
            ..Default::default()
        },
    )
}

/// Stain the Mind — {4}{B} Sorcery with convoke. Name a nonland card and exile
/// every copy from target player's graveyard, hand, and library.
pub fn stain_the_mind() -> CardDefinition {
    convoke_spell(
        "Stain the Mind",
        cost(&[generic(4), b()]),
        CardType::Sorcery,
        Effect::NameCardExileMatchingAllZones,
    )
}

/// The Chain Veil — {4} Legendary Artifact. Buys an extra loyalty activation
/// per planeswalker; taxes you 2 life on turns you don't use one.
pub fn the_chain_veil() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        name: "The Chain Veil",
        cost: cost(&[generic(4)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crabomination_base::turn_step::TurnStep::End),
                EventScope::YourControl,
            )
            .with_filter(Predicate::Not(Box::new(Predicate::ActivatedLoyaltyThisTurn {
                who: PlayerRef::You,
            }))),
            effect: Effect::LoseLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            effect: Effect::GrantExtraLoyaltyActivations,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Waste Not — {1}{B} Enchantment. Each opponent discard pays off by card
/// type: Zombies, black mana, or a card.
pub fn waste_not() -> CardDefinition {
    let on_discard = |filter: R, effect: Effect| TriggeredAbility {
        event: EventSpec::new(EventKind::CardDiscarded, EventScope::OpponentControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter }),
        effect,
    };
    CardDefinition {
        name: "Waste Not",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            on_discard(
                R::Creature,
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Zombie".into(),
                        power: 2,
                        toughness: 2,
                        colors: vec![Color::Black],
                        card_types: vec![CardType::Creature],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Zombie],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
            ),
            on_discard(
                R::Land,
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colors(vec![Color::Black, Color::Black]),
                },
            ),
            on_discard(
                R::Noncreature.and(R::Nonland),
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ),
        ],
        ..Default::default()
    }
}

/// Yisan, the Wanderer Bard — {2}{G} 2/3. Each verse counter tutors a creature
/// with that exact mana value onto the battlefield.
pub fn yisan_the_wanderer_bard() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            tap_cost: true,
            add_counter_cost: Some((CounterType::Verse, 1)),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::Creature.and(R::ManaValueEqualsSourceCounters(CounterType::Verse)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature(
            "Yisan, the Wanderer Bard",
            cost(&[generic(2), g()]),
            2,
            3,
            vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Bard],
            vec![],
        )
    }
}
