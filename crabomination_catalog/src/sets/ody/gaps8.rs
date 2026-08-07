//! Odyssey (ODY) gap-closing wave 8: the Atog cycle, the tapper/prevention
//! shell and the last spells. Tests in `classic_sets/ody`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    Keyword, Predicate, SelectionRequirement as R, Subtypes, Value,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector,
    shortcut::{draw, target_any, target_filtered},
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

fn threshold() -> Predicate {
    Predicate::ThresholdActive { who: PlayerRef::You }
}

/// The Atog pump: "+1/+1 until end of turn."
fn atog_pump() -> Effect {
    Effect::PumpPT {
        what: Selector::This,
        power: Value::ONE,
        toughness: Value::ONE,
        duration: Duration::EndOfTurn,
    }
}

// ── The Odyssey Atog cycle ──────────────────────────────────────────────────

fn atog(name: &'static str, c: ManaCost, feeds: [ActivatedAbility; 2]) -> CardDefinition {
    CardDefinition {
        activated_abilities: feeds.into(),
        ..creature(name, c, vec![CreatureType::Atog], 1, 2)
    }
}

fn sac_feed(filter: R) -> ActivatedAbility {
    ActivatedAbility {
        sac_other_filter: Some((filter, 1)),
        effect: atog_pump(),
        ..Default::default()
    }
}

fn discard_feed() -> ActivatedAbility {
    ActivatedAbility {
        discard_cost: Some((R::Any, 1)),
        effect: atog_pump(),
        ..Default::default()
    }
}

fn graveyard_feed() -> ActivatedAbility {
    ActivatedAbility {
        exile_other_filter: Some((R::InYourGraveyard, 2)),
        effect: atog_pump(),
        ..Default::default()
    }
}

pub fn phantatog() -> CardDefinition {
    atog(
        "Phantatog",
        cost(&[generic(1), w(), u()]),
        [sac_feed(R::Enchantment), discard_feed()],
    )
}

pub fn psychatog() -> CardDefinition {
    atog("Psychatog", cost(&[generic(1), u(), b()]), [discard_feed(), graveyard_feed()])
}

pub fn sarcatog() -> CardDefinition {
    atog(
        "Sarcatog",
        cost(&[generic(1), b(), r()]),
        [graveyard_feed(), sac_feed(R::Artifact)],
    )
}

pub fn thaumatog() -> CardDefinition {
    atog(
        "Thaumatog",
        cost(&[generic(1), g(), w()]),
        [sac_feed(R::Land), sac_feed(R::Enchantment)],
    )
}

// ── Tappers and prevention ──────────────────────────────────────────────────

/// Puppeteer — {2}{U} 1/2 that taps or untaps a creature.
pub fn puppeteer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            effect: Effect::ChooseMode(vec![
                Effect::Tap { what: target_filtered(R::Creature) },
                Effect::Untap { what: target_filtered(R::Creature), up_to: None },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Puppeteer",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            2,
        )
    }
}

/// Nomad Decoy — {2}{W} 1/2 tapper that doubles up past Threshold.
pub fn nomad_decoy() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                tap_cost: true,
                effect: Effect::Tap { what: target_filtered(R::Creature) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[w(), w()]),
                tap_cost: true,
                condition: Some(threshold()),
                effect: Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 2,
                    filter: R::Creature,
                    effect: Box::new(Effect::Tap { what: Selector::Target(0) }),
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Nomad Decoy",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Nomad],
            1,
            2,
        )
    }
}

/// The Pilgrim pair: protection from a colour, and a one-shot shield off it.
fn pilgrim(name: &'static str, color: Color) -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(color)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            sac_cost: true,
            effect: Effect::PreventNextDamageFromChosenSource {
                filter: R::HasColor(color),
                reflect: false,
                to: None,
                gain_life: false,
                redirect_to: None,
                whole_turn: false,
            },
            ..Default::default()
        }],
        ..creature(
            name,
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            3,
        )
    }
}

pub fn pilgrim_of_justice() -> CardDefinition {
    pilgrim("Pilgrim of Justice", Color::Red)
}
pub fn pilgrim_of_virtue() -> CardDefinition {
    pilgrim("Pilgrim of Virtue", Color::Black)
}

// ── Enchantments ────────────────────────────────────────────────────────────

/// Persuasion — {3}{U}{U} steals the enchanted creature.
pub fn persuasion() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![crate::effect::shortcut::etb(
            Effect::GainControlWhileSourceRemains {
                what: Selector::attached_to(Selector::This),
            },
        )],
        ..enchantment("Persuasion", cost(&[generic(3), u(), u()]))
    }
}

/// Tattoo Ward — {2}{W} +1/+1 and protection from enchantments, keeping itself
/// on; sacrifices for an enchantment.
pub fn tattoo_ward() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::ProtectionFromCardType(CardType::Enchantment)],
            protection_keeps_self: true,
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Enchantment) },
            ..Default::default()
        }],
        ..enchantment("Tattoo Ward", cost(&[generic(2), w()]))
    }
}

/// Testament of Faith — {W} stands up as an X/X defender.
pub fn testament_of_faith() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            effect: Effect::BecomeCreature {
                what: Selector::This,
                power: Value::XFromCost,
                toughness: Value::XFromCost,
                creature_types: vec![CreatureType::Wall],
                keywords: vec![Keyword::Defender],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..enchantment("Testament of Faith", cost(&[w()]))
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Shower of Coals — {3}{R}{R} 2 damage to three targets, 4 past Threshold.
pub fn shower_of_coals() -> CardDefinition {
    let spread = |n: i32| Effect::ApplyToTargets {
        max_targets: 3,
        min_targets: 0,
        filter: R::Creature.or(R::Player).or(R::Planeswalker),
        effect: Box::new(Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::Const(n),
        }),
    };
    sorcery(
        "Shower of Coals",
        cost(&[generic(3), r(), r()]),
        Effect::If { cond: threshold(), then: Box::new(spread(4)), else_: Box::new(spread(2)) },
    )
}

/// Vivify — {2}{G} animates a land and cantrips.
pub fn vivify() -> CardDefinition {
    instant(
        "Vivify",
        cost(&[generic(2), g()]),
        Effect::Seq(vec![
            Effect::BecomeCreature {
                what: target_filtered(R::Land),
                power: Value::Const(3),
                toughness: Value::Const(3),
                creature_types: vec![],
                keywords: vec![],
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
    )
}

/// Sacred Rites — {W} pitch your hand for team toughness.
pub fn sacred_rites() -> CardDefinition {
    instant(
        "Sacred Rites",
        cost(&[w()]),
        Effect::Seq(vec![
            Effect::DiscardAnyNumber { who: Selector::You, filter: crate::card::SelectionRequirement::Any },
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(0),
                toughness: Value::CardsDiscardedThisEffect,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Demoralize — {2}{R} menace for everyone, or no blocks at all past
/// Threshold.
pub fn demoralize() -> CardDefinition {
    instant(
        "Demoralize",
        cost(&[generic(2), r()]),
        Effect::If {
            cond: threshold(),
            then: Box::new(Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature),
                keyword: Keyword::Menace,
                duration: Duration::EndOfTurn,
            }),
        },
    )
}

/// Embolden — {2}{W} four points of prevention, twice.
pub fn embolden() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(1), w()]))],
        ..instant(
            "Embolden",
            cost(&[generic(2), w()]),
            Effect::PreventNextDamage { target: target_any(), amount: Value::Const(4) },
        )
    }
}

/// Piper's Melody — {G} recycles your dead creatures into your library.
pub fn pipers_melody() -> CardDefinition {
    sorcery(
        "Piper's Melody",
        cost(&[g()]),
        Effect::Move {
            what: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: crate::card::Zone::Graveyard,
                filter: R::Creature,
            },
            to: crate::effect::ZoneDest::Library {
                who: PlayerRef::OwnerOfMoved,
                pos: crate::effect::LibraryPosition::Shuffled,
            },
        },
    )
}

/// Time Stretch — {8}{U}{U} two extra turns.
pub fn time_stretch() -> CardDefinition {
    sorcery(
        "Time Stretch",
        cost(&[generic(8), u(), u()]),
        Effect::TakeExtraTurn { who: PlayerRef::Target(0), count: Value::Const(2) },
    )
}
