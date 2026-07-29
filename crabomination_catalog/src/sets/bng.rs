//! Born of the Gods (BNG) — the common/uncommon core. Tests in
//! `classic_sets/bng`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, heroic, target_any, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest};
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

/// "Inspired — Whenever this creature becomes untapped, …" (CR 702.108).
fn inspired(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::BecomesUntapped, EventScope::SelfSource),
        effect,
    }
}

// ── Creatures ────────────────────────────────────────────────────────────────

/// Akroan Skyguard — {1}{W} 1/1 flier. Heroic: a +1/+1 counter.
pub fn akroan_skyguard() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..creature(
            "Akroan Skyguard",
            cost(&[generic(1), w()]),
            1,
            1,
            vec![CreatureType::Human, CreatureType::Soldier],
            vec![Keyword::Flying],
        )
    }
}

/// Chorus of the Tides — {3}{U} 3/2 flier. Heroic: scry 1.
pub fn chorus_of_the_tides() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::ONE,
        })],
        ..creature(
            "Chorus of the Tides",
            cost(&[generic(3), u()]),
            3,
            2,
            vec![CreatureType::Siren],
            vec![Keyword::Flying],
        )
    }
}

/// Elite Skirmisher — {2}{W} 3/1. Heroic: you may tap target creature.
pub fn elite_skirmisher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::MayDo {
            description: "Tap target creature".into(),
            body: Box::new(Effect::Tap { what: Selector::TargetFiltered { slot: 1, filter: R::Creature } }),
        })],
        ..creature(
            "Elite Skirmisher",
            cost(&[generic(2), w()]),
            3,
            1,
            vec![CreatureType::Human, CreatureType::Soldier],
            vec![],
        )
    }
}

/// Cyclops of One-Eyed Pass — {2}{R}{R} 5/2 vanilla.
pub fn cyclops_of_one_eyed_pass() -> CardDefinition {
    creature(
        "Cyclops of One-Eyed Pass",
        cost(&[generic(2), r(), r()]),
        5,
        2,
        vec![CreatureType::Cyclops],
        vec![],
    )
}

/// Great Hart — {3}{W} 2/4 vanilla.
pub fn great_hart() -> CardDefinition {
    creature("Great Hart", cost(&[generic(3), w()]), 2, 4, vec![CreatureType::Elk], vec![])
}

/// Deepwater Hypnotist — {1}{U} 2/1. Inspired: an opposing creature gets -3/-0.
pub fn deepwater_hypnotist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![inspired(Effect::PumpPT {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            power: Value::Const(-3),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Deepwater Hypnotist",
            cost(&[generic(1), u()]),
            2,
            1,
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Kragma Butcher — {2}{R} 2/3. Inspired: +2/+0 until end of turn.
pub fn kragma_butcher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![inspired(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Kragma Butcher",
            cost(&[generic(2), r()]),
            2,
            3,
            vec![CreatureType::Minotaur, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Felhide Brawler — {1}{B} 2/2 that can't block without another Minotaur.
pub fn felhide_brawler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantAttackOrBlockUnlessYouControlCount {
            filter: Box::new(R::HasCreatureType(CreatureType::Minotaur)),
            min: 1,
            attack_only: false,
            block_only: true,
            exclude_self: true,
        }],
        ..creature(
            "Felhide Brawler",
            cost(&[generic(1), b()]),
            2,
            2,
            vec![CreatureType::Minotaur],
            vec![],
        )
    }
}

/// Forsaken Drifters — {3}{B} 4/2. Dies: mill four.
pub fn forsaken_drifters() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Mill { who: Selector::You, amount: Value::Const(4) },
        }],
        ..creature("Forsaken Drifters", cost(&[generic(3), b()]), 4, 2, vec![CreatureType::Zombie], vec![])
    }
}

/// Griffin Dreamfinder — {3}{W}{W} 1/4 flier. ETB: return an enchantment card
/// from your graveyard to hand.
pub fn griffin_dreamfinder() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::Enchantment.and(R::InGraveyard)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..creature(
            "Griffin Dreamfinder",
            cost(&[generic(3), w(), w()]),
            1,
            4,
            vec![CreatureType::Griffin],
            vec![Keyword::Flying],
        )
    }
}

/// Impetuous Sunchaser — {1}{R} 1/1 flying haste that must attack.
pub fn impetuous_sunchaser() -> CardDefinition {
    creature(
        "Impetuous Sunchaser",
        cost(&[generic(1), r()]),
        1,
        1,
        vec![CreatureType::Human, CreatureType::Soldier],
        vec![Keyword::Flying, Keyword::Haste, Keyword::MustAttack],
    )
}

/// Loyal Pegasus — {W} 2/1 flier that can't attack or block alone.
pub fn loyal_pegasus() -> CardDefinition {
    creature(
        "Loyal Pegasus",
        cost(&[w()]),
        2,
        1,
        vec![CreatureType::Pegasus],
        vec![Keyword::Flying, Keyword::CantAttackOrBlockAlone],
    )
}

/// Marshmist Titan — {6}{B} 4/5 that costs {X} less, X = devotion to black.
pub fn marshmist_titan() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This spell costs {X} less to cast, where X is your devotion to black.",
            effect: StaticEffect::SelfCostReducedByDevotion { colors: vec![Color::Black] },
        }],
        ..creature("Marshmist Titan", cost(&[generic(6), b()]), 4, 5, vec![CreatureType::Giant], vec![])
    }
}

/// Nyxborn Eidolon — {1}{B} 2/1 enchantment creature; bestow {3}{B} grants
/// +2/+1.
pub fn nyxborn_eidolon() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        bestow: Some(cost(&[generic(3), b()])),
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 2,
            toughness: 1,
            ..Default::default()
        }),
        ..creature("Nyxborn Eidolon", cost(&[generic(1), b()]), 2, 1, vec![CreatureType::Spirit], vec![])
    }
}

// ── Spells ───────────────────────────────────────────────────────────────────

/// Asphyxiate — {1}{B}{B} Sorcery. Destroy target untapped creature.
pub fn asphyxiate() -> CardDefinition {
    spell(
        "Asphyxiate",
        cost(&[generic(1), b(), b()]),
        CardType::Sorcery,
        Effect::Destroy { what: target_filtered(R::Creature.and(R::Untapped)) },
    )
}

/// Excoriate — {3}{W} Sorcery. Exile target tapped creature.
pub fn excoriate() -> CardDefinition {
    spell(
        "Excoriate",
        cost(&[generic(3), w()]),
        CardType::Sorcery,
        Effect::Exile { what: target_filtered(R::Creature.and(R::Tapped)) },
    )
}

/// Bolt of Keranos — {1}{R}{R} Sorcery. 3 damage to any target, then scry 1.
pub fn bolt_of_keranos() -> CardDefinition {
    spell(
        "Bolt of Keranos",
        cost(&[generic(1), r(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
            Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        ]),
    )
}

/// Eye Gouge — {B} Instant. -1/-1, and a Cyclops dies outright.
pub fn eye_gouge() -> CardDefinition {
    spell(
        "Eye Gouge",
        cost(&[b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: crate::card::Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::HasCreatureType(CreatureType::Cyclops),
                },
                then: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Fall of the Hammer — {1}{R} Instant. Your creature deals its power to
/// another target creature.
pub fn fall_of_the_hammer() -> CardDefinition {
    spell(
        "Fall of the Hammer",
        cost(&[generic(1), r()]),
        CardType::Instant,
        Effect::DealDamageEqualToPower {
            source: target_filtered(R::Creature.and(R::ControlledByYou)),
            target: Selector::TargetFiltered { slot: 1, filter: R::Creature },
        },
    )
}

/// Culling Mark — {2}{G} Sorcery. Target creature blocks this turn if able.
pub fn culling_mark() -> CardDefinition {
    spell(
        "Culling Mark",
        cost(&[generic(2), g()]),
        CardType::Sorcery,
        Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::MustBlock,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Mortal's Ardor — {W} Instant. +1/+1 and lifelink.
pub fn mortals_ardor() -> CardDefinition {
    spell(
        "Mortal's Ardor",
        cost(&[w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Mortal's Resolve — {1}{G} Instant. +1/+1 and indestructible.
pub fn mortals_resolve() -> CardDefinition {
    spell(
        "Mortal's Resolve",
        cost(&[generic(1), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Crypsis — {1}{U} Instant. Protection from your opponents' creatures, and
/// untap. (Modeled as protection from creatures — the "your opponents control"
/// narrowing costs nothing in practice, since your own creatures don't block
/// or damage it.)
pub fn crypsis() -> CardDefinition {
    spell(
        "Crypsis",
        cost(&[generic(1), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::ProtectionFromCreatures,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
        ]),
    )
}

/// Hold at Bay — {1}{W} Instant. Prevent the next 7 damage to any target.
pub fn hold_at_bay() -> CardDefinition {
    spell(
        "Hold at Bay",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::PreventNextDamage { target: target_any(), amount: Value::Const(7) },
    )
}

/// Nullify — {U}{U} Instant. Counter target creature or Aura spell.
pub fn nullify() -> CardDefinition {
    spell(
        "Nullify",
        cost(&[u(), u()]),
        CardType::Instant,
        Effect::CounterSpell {
            what: target_filtered(R::IsSpellOnStack.and(
                R::Creature.or(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)),
            )),
        },
    )
}

/// Glimpse the Sun God — {X}{W} Instant. Tap X target creatures.
pub fn glimpse_the_sun_god() -> CardDefinition {
    spell(
        "Glimpse the Sun God",
        cost(&[x(), w()]),
        CardType::Instant,
        Effect::CapTargetsAtX {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 1,
                filter: R::Creature,
                effect: Box::new(Effect::Tap { what: Selector::Target(0) }),
            }),
        },
    )
}

// ── Auras, Equipment, and the bestow cycle ───────────────────────────────────

use crate::card::{ActivatedAbility, EquipBonus};

/// An "enchant creature" Aura whose only text is a granted activated ability.
fn granting_aura(
    name: &'static str,
    mana: ManaCost,
    ability: ActivatedAbility,
    etb_draw: bool,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { activated_abilities: vec![ability], ..Default::default() }),
        triggered_abilities: if etb_draw {
            vec![etb(Effect::Draw { who: Selector::You, amount: Value::ONE })]
        } else {
            vec![]
        },
        ..Default::default()
    }
}

/// A bestow creature that grants a flat `+p/+t` while attached.
fn nyxborn(
    name: &'static str,
    mana: ManaCost,
    bestow_cost: ManaCost,
    pt: (i32, i32),
    ct: Vec<CreatureType>,
) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        bestow: Some(bestow_cost),
        equipped_bonus: Some(EquipBonus {
            power: pt.0,
            toughness: pt.1,
            ..Default::default()
        }),
        ..creature(name, mana, pt.0, pt.1, ct, vec![])
    }
}

/// Nyxborn Shieldmate — {W} 1/2; bestow {2}{W} for +1/+2.
pub fn nyxborn_shieldmate() -> CardDefinition {
    nyxborn(
        "Nyxborn Shieldmate",
        cost(&[w()]),
        cost(&[generic(2), w()]),
        (1, 2),
        vec![CreatureType::Human, CreatureType::Soldier],
    )
}

/// Nyxborn Triton — {2}{U} 2/3; bestow {4}{U} for +2/+3.
pub fn nyxborn_triton() -> CardDefinition {
    nyxborn(
        "Nyxborn Triton",
        cost(&[generic(2), u()]),
        cost(&[generic(4), u()]),
        (2, 3),
        vec![CreatureType::Merfolk],
    )
}

/// Nyxborn Wolf — {2}{G} 3/1; bestow {4}{G} for +3/+1.
pub fn nyxborn_wolf() -> CardDefinition {
    nyxborn(
        "Nyxborn Wolf",
        cost(&[generic(2), g()]),
        cost(&[generic(4), g()]),
        (3, 1),
        vec![CreatureType::Wolf],
    )
}

/// Ephara's Radiance — {W} Aura granting "{1}{W}, {T}: You gain 3 life."
pub fn epharas_radiance() -> CardDefinition {
    granting_aura(
        "Ephara's Radiance",
        cost(&[w()]),
        ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            tap_cost: true,
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            ..Default::default()
        },
        false,
    )
}

/// Claim of Erebos — {1}{B} Aura granting "{1}{B}, {T}: Target player loses 2
/// life."
pub fn claim_of_erebos() -> CardDefinition {
    granting_aura(
        "Claim of Erebos",
        cost(&[generic(1), b()]),
        ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            tap_cost: true,
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
            },
            ..Default::default()
        },
        false,
    )
}

/// Evanescent Intellect — {U} Aura granting "{1}{U}, {T}: Target player mills
/// three cards."
pub fn evanescent_intellect() -> CardDefinition {
    granting_aura(
        "Evanescent Intellect",
        cost(&[u()]),
        ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            tap_cost: true,
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(3),
            },
            ..Default::default()
        },
        false,
    )
}

/// Epiphany Storm — {R} Aura granting "{R}, {T}, Discard a card: Draw a card."
pub fn epiphany_storm() -> CardDefinition {
    granting_aura(
        "Epiphany Storm",
        cost(&[r()]),
        ActivatedAbility {
            mana_cost: cost(&[r()]),
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        },
        false,
    )
}

/// Karametra's Favor — {1}{G} Aura. ETB draw; the host taps for any color.
pub fn karametras_favor() -> CardDefinition {
    granting_aura(
        "Karametra's Favor",
        cost(&[generic(1), g()]),
        ActivatedAbility {
            tap_cost: true,
            effect: crate::effect::shortcut::add_any_one_color(1),
            ..Default::default()
        },
        true,
    )
}

/// Grisly Transformation — {2}{B} Aura. ETB draw; the host gains intimidate.
pub fn grisly_transformation() -> CardDefinition {
    CardDefinition {
        name: "Grisly Transformation",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Intimidate],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::ONE })],
        ..Default::default()
    }
}

/// Fearsome Temper — {2}{R} Aura granting +2/+2 and a can't-block activation.
pub fn fearsome_temper() -> CardDefinition {
    CardDefinition {
        name: "Fearsome Temper",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(2), r()]),
                effect: Effect::CantBlockSourceThisTurn {
                    target: target_filtered(R::Creature),
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Gorgon's Head — {1} Equipment. Equipped creature has deathtouch. Equip {2}.
pub fn gorgons_head() -> CardDefinition {
    CardDefinition {
        name: "Gorgon's Head",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Deathtouch],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Akroan Phalanx — {3}{W} 3/3 vigilance. {2}{R}: your creatures get +1/+0.
pub fn akroan_phalanx() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Akroan Phalanx",
            cost(&[generic(3), w()]),
            3,
            3,
            vec![CreatureType::Human, CreatureType::Soldier],
            vec![Keyword::Vigilance],
        )
    }
}

/// Ashiok's Adept — {2}{B} 1/3. Heroic: each opponent discards a card.
pub fn ashioks_adept() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::ONE,
            random: false,
        })],
        ..creature(
            "Ashiok's Adept",
            cost(&[generic(2), b()]),
            1,
            3,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Graverobber Spider — {3}{G} 2/4 reach. {3}{B}: +X/+X for the creature cards
/// in your graveyard, once each turn.
pub fn graverobber_spider() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            once_per_turn: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::CardsInGraveyardMatching {
                    who: PlayerRef::You,
                    filter: R::Creature,
                },
                toughness: Value::CardsInGraveyardMatching {
                    who: PlayerRef::You,
                    filter: R::Creature,
                },
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Graverobber Spider",
            cost(&[generic(3), g()]),
            2,
            4,
            vec![CreatureType::Spider],
            vec![Keyword::Reach],
        )
    }
}
