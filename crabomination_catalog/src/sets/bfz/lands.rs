//! BFZ lands — the "battle land" duals, the Blighted utility cycle, and the
//! enters-tapped ETB-trigger commons.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Value,
};
use crate::effect::shortcut::{add_colorless, etb, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, ZoneDest};
use crate::mana::{b, cost, generic, r, u, w, Color, SpendRestriction};

fn land(name: &'static str) -> CardDefinition {
    CardDefinition { name, card_types: vec![CardType::Land], ..Default::default() }
}

fn tap_for(colors: Vec<Color>) -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(colors) },
        ..Default::default()
    }
}

fn tap_for_colorless() -> ActivatedAbility {
    ActivatedAbility { tap_cost: true, effect: add_colorless(1), ..Default::default() }
}

// ── Battle lands ────────────────────────────────────────────────────────────

/// The BFZ "battle land" dual: a typed dual that enters tapped unless you
/// control two or more basic lands.
fn battle_land(name: &'static str, types: Vec<LandType>, colors: Vec<Color>) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { land_types: types, ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "Enters tapped unless you control two or more basic lands.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::IsBasicLand.and(R::ControlledByYou)),
                    n: Value::Const(2),
                },
            },
        }],
        activated_abilities: vec![tap_for(colors)],
        ..land(name)
    }
}

/// Canopy Vista — {T}: {G} or {W}.
pub fn canopy_vista() -> CardDefinition {
    battle_land(
        "Canopy Vista",
        vec![LandType::Forest, LandType::Plains],
        vec![Color::Green, Color::White],
    )
}

/// Cinder Glade — {T}: {R} or {G}.
pub fn cinder_glade() -> CardDefinition {
    battle_land(
        "Cinder Glade",
        vec![LandType::Mountain, LandType::Forest],
        vec![Color::Red, Color::Green],
    )
}

/// Prairie Stream — {T}: {W} or {U}.
pub fn prairie_stream() -> CardDefinition {
    battle_land(
        "Prairie Stream",
        vec![LandType::Plains, LandType::Island],
        vec![Color::White, Color::Blue],
    )
}

/// Smoldering Marsh — {T}: {B} or {R}.
pub fn smoldering_marsh() -> CardDefinition {
    battle_land(
        "Smoldering Marsh",
        vec![LandType::Swamp, LandType::Mountain],
        vec![Color::Black, Color::Red],
    )
}

/// Sunken Hollow — {T}: {U} or {B}.
pub fn sunken_hollow() -> CardDefinition {
    battle_land(
        "Sunken Hollow",
        vec![LandType::Island, LandType::Swamp],
        vec![Color::Blue, Color::Black],
    )
}

// ── Blighted utility cycle ──────────────────────────────────────────────────

/// The Blighted cycle: {T} for {C}, plus a sac-for-value activation.
fn blighted(name: &'static str, sac_cost: crate::mana::ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            tap_for_colorless(),
            ActivatedAbility {
                mana_cost: sac_cost,
                tap_cost: true,
                sac_cost: true,
                effect,
                ..Default::default()
            },
        ],
        ..land(name)
    }
}

/// Blighted Cataract — {5}{U}, {T}, Sacrifice: draw two cards.
pub fn blighted_cataract() -> CardDefinition {
    blighted(
        "Blighted Cataract",
        cost(&[generic(5), u()]),
        crate::effect::shortcut::draw(2),
    )
}

/// Blighted Fen — {4}{B}, {T}, Sacrifice: target opponent sacrifices a creature.
pub fn blighted_fen() -> CardDefinition {
    blighted(
        "Blighted Fen",
        cost(&[generic(4), b()]),
        Effect::Sacrifice {
            who: Selector::Player(PlayerRef::Target(0)),
            count: Value::Const(1),
            filter: R::Creature,
        },
    )
}

/// Blighted Gorge — {4}{R}, {T}, Sacrifice: 2 damage to any target.
pub fn blighted_gorge() -> CardDefinition {
    blighted(
        "Blighted Gorge",
        cost(&[generic(4), r()]),
        Effect::DealDamage {
            to: crate::effect::shortcut::target_any(),
            amount: Value::Const(2),
        },
    )
}

/// Blighted Steppe — {3}{W}, {T}, Sacrifice: gain 2 life per creature you
/// control.
pub fn blighted_steppe() -> CardDefinition {
    blighted(
        "Blighted Steppe",
        cost(&[generic(3), w()]),
        Effect::GainLife {
            who: Selector::You,
            amount: Value::Times(
                Box::new(Value::Const(2)),
                Box::new(Value::count(Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou),
                ))),
            ),
        },
    )
}

// ── Enters-tapped ETB commons ───────────────────────────────────────────────

fn tapped_etb_land(name: &'static str, color: Color, trigger: Effect) -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        triggered_abilities: vec![etb(trigger)],
        activated_abilities: vec![tap_for(vec![color])],
        ..land(name)
    }
}

/// Looming Spires — ETB: target creature gets +1/+1 and first strike EOT.
pub fn looming_spires() -> CardDefinition {
    tapped_etb_land(
        "Looming Spires",
        Color::Red,
        Effect::Seq(vec![
            crate::effect::shortcut::pump_target(1, 1),
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Sandstone Bridge — ETB: target creature gets +1/+1 and vigilance EOT.
pub fn sandstone_bridge() -> CardDefinition {
    tapped_etb_land(
        "Sandstone Bridge",
        Color::White,
        Effect::Seq(vec![
            crate::effect::shortcut::pump_target(1, 1),
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Skyline Cascade — ETB: a creature an opponent controls doesn't untap during
/// its controller's next untap step.
pub fn skyline_cascade() -> CardDefinition {
    tapped_etb_land(
        "Skyline Cascade",
        Color::Blue,
        Effect::SkipNextUntap {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
        },
    )
}

/// Fertile Thicket — ETB: look at the top five, reveal up to one basic land and
/// put it on top, the rest on the bottom.
pub fn fertile_thicket() -> CardDefinition {
    tapped_etb_land(
        "Fertile Thicket",
        Color::Green,
        Effect::LookTopKeepMatchingOnTop {
            who: PlayerRef::You,
            count: Value::Const(5),
            take: Value::Const(1),
            filter: R::IsBasicLand,
        },
    )
}

// ── Utility rares ───────────────────────────────────────────────────────────

/// Ally Encampment — {T}: {C}; {T}: one mana of any color for an Ally spell;
/// {1}, {T}, Sacrifice: return target Ally you control to its owner's hand.
pub fn ally_encampment() -> CardDefinition {
    use crate::card::CreatureType;
    CardDefinition {
        activated_abilities: vec![
            tap_for_colorless(),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyOneColor(Value::Const(1))),
                        SpendRestriction::CreatureOfType(CreatureType::Ally),
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Move {
                    what: target_filtered(
                        R::HasCreatureType(CreatureType::Ally).and(R::ControlledByYou),
                    ),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                ..Default::default()
            },
        ],
        ..land("Ally Encampment")
    }
}

/// Shrine of the Forsaken Gods — {T}: {C}; {T}: {C}{C} for colorless spells,
/// only with seven or more lands.
pub fn shrine_of_the_forsaken_gods() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            tap_for_colorless(),
            ActivatedAbility {
                tap_cost: true,
                condition: Some(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
                    n: Value::Const(7),
                }),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colorless(Value::Const(2))),
                        SpendRestriction::ColorlessSpellsOrAbilities,
                    ),
                },
                ..Default::default()
            },
        ],
        ..land("Shrine of the Forsaken Gods")
    }
}

/// Sanctum of Ugin — {T}: {C}. Casting a colorless spell of mana value 7+ may
/// sacrifice it to tutor a colorless creature card to hand.
pub fn sanctum_of_ugin() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        activated_abilities: vec![tap_for_colorless()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Colorless.and(R::ManaValueAtLeast(7)),
                },
            ),
            effect: Effect::MayDo {
                description: "Sacrifice Sanctum of Ugin to search for a colorless creature?"
                    .into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::SacrificePermanent { what: Selector::This },
                    Effect::Search {
                        who: PlayerRef::You,
                        filter: R::Creature.and(R::Colorless),
                        to: ZoneDest::Hand(PlayerRef::You),
                    },
                ])),
            },
        }],
        ..land("Sanctum of Ugin")
    }
}
