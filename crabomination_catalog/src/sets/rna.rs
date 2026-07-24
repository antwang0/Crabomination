//! Ravnica Allegiance (RNA) — 2019. Commons/uncommons on existing primitives.
//! Tests in `classic_sets/rna`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, Subtypes, Value,
};
use crate::card::SelectionRequirement as R;
use crate::effect::shortcut::{deal, draw, etb, spectacle, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

fn creatures(t: Vec<CreatureType>) -> Subtypes {
    Subtypes { creature_types: t, ..Default::default() }
}

fn body(name: &'static str, mana: crate::mana::ManaCost, p: i32, t: i32, ct: Vec<CreatureType>, kw: Vec<Keyword>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: creatures(ct),
        power: p,
        toughness: t,
        keywords: kw,
        ..Default::default()
    }
}

/// Catacomb Crocodile — {4}{B} 3/7 Crocodile.
pub fn catacomb_crocodile() -> CardDefinition {
    body("Catacomb Crocodile", cost(&[generic(4), b()]), 3, 7, vec![CreatureType::Crocodile], vec![])
}

/// Azorius Knight-Arbiter — {3}{W}{U} 2/5 Human Knight. Vigilance; can't be
/// blocked.
pub fn azorius_knight_arbiter() -> CardDefinition {
    body("Azorius Knight-Arbiter", cost(&[generic(3), w(), u()]), 2, 5, vec![CreatureType::Human, CreatureType::Knight], vec![Keyword::Vigilance, Keyword::Unblockable])
}

/// Carrion Imp — {3}{B} 2/3 Imp with flying. ETB may exile a creature card from
/// a graveyard; if you do, gain 2 life.
pub fn carrion_imp() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Exile target creature card from a graveyard; gain 2 life.".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Move { what: target_filtered(R::Creature.and(R::InGraveyard)), to: ZoneDest::Exile },
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ])),
        })],
        ..body("Carrion Imp", cost(&[generic(3), b()]), 2, 3, vec![CreatureType::Imp], vec![Keyword::Flying])
    }
}

/// Civic Stalwart — {3}{W} 3/3 Elephant Soldier. ETB creatures you control get
/// +1/+1 until end of turn.
pub fn civic_stalwart() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        })],
        ..body("Civic Stalwart", cost(&[generic(3), w()]), 3, 3, vec![CreatureType::Elephant, CreatureType::Soldier], vec![])
    }
}

/// Blade Juggler — {4}{B} 3/2 Human Rogue with Spectacle {2}{B}. ETB deals 1
/// damage to you and you draw a card.
pub fn blade_juggler() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(spectacle(cost(&[generic(2), b()]))),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            deal(1, Selector::You),
            draw(1),
        ]))],
        ..body("Blade Juggler", cost(&[generic(4), b()]), 3, 2, vec![CreatureType::Human, CreatureType::Rogue], vec![])
    }
}

/// Devkarin Dissident — {1}{G} 2/2 Elf Warrior. {4}{G}: +2/+2 until end of turn.
pub fn devkarin_dissident() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..body("Devkarin Dissident", cost(&[generic(1), g()]), 2, 2, vec![CreatureType::Elf, CreatureType::Warrior], vec![])
    }
}

/// Passwall Adept — {1}{U} 1/3 Human Wizard. {2}{U}: target creature can't be
/// blocked this turn.
pub fn passwall_adept() -> CardDefinition {
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
        ..body("Passwall Adept", cost(&[generic(1), u()]), 1, 3, vec![CreatureType::Human, CreatureType::Wizard], vec![])
    }
}

/// Rakdos Firewheeler — {B}{B}{R}{R} 4/3 Human Rogue. ETB deals 2 to target
/// opponent and 2 to up to one target creature or planeswalker.
pub fn rakdos_firewheeler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::TargetFiltered { slot: 0, filter: R::OpponentPlayer },
                    amount: Value::Const(2),
                },
                Effect::DealDamage {
                    to: Selector::TargetFiltered { slot: 1, filter: R::Creature.or(R::HasCardType(CardType::Planeswalker)) },
                    amount: Value::Const(2),
                },
            ])),
        })],
        ..body("Rakdos Firewheeler", cost(&[b(), b(), r(), r()]), 4, 3, vec![CreatureType::Human, CreatureType::Rogue], vec![])
    }
}

/// Gyre Engineer — {1}{G}{U} 1/1 Vedalken Wizard. {T}: Add {G}{U}. (The adapt
/// payoff rider is engine-supported but omitted here.)
pub fn gyre_engineer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Green, Color::Blue]) },
            ..Default::default()
        }],
        ..body("Gyre Engineer", cost(&[generic(1), g(), u()]), 1, 1, vec![CreatureType::Vedalken, CreatureType::Wizard], vec![])
    }
}

/// Bring to Trial — {2}{W} Sorcery. Exile target creature with power 4 or
/// greater.
pub fn bring_to_trial() -> CardDefinition {
    CardDefinition {
        name: "Bring to Trial",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: target_filtered(R::Creature.and(R::PowerAtLeast(4))),
            to: ZoneDest::Exile,
        },
        ..Default::default()
    }
}

/// Burn Bright — {2}{R} Instant. Creatures you control get +2/+0 until end of
/// turn.
pub fn burn_bright() -> CardDefinition {
    CardDefinition {
        name: "Burn Bright",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: Value::Const(2),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Applied Biomancy — {G}{U} Instant. Choose one or both — target creature gets
/// +1/+1 until end of turn; and/or return target creature to its owner's hand.
pub fn applied_biomancy() -> CardDefinition {
    CardDefinition {
        name: "Applied Biomancy",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseModesCast {
            modes: vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::Move { what: target_filtered(R::Creature), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
            ],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
        ..Default::default()
    }
}
