//! One submodule per Magic set, named by the set's three-letter code.
//! Helpers shared across all set modules live here.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, EventKind, EventScope, EventSpec, LandType,
    SelectionRequirement, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::{Effect, ManaPayload, PlayerRef, Predicate};
use crate::mana::{Color, ManaCost};

pub fn tap_add(color: Color) -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        mana_cost: ManaCost::default(),
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Colors(vec![color]),
        },
        once_per_turn: false,
        sorcery_speed: false,
        sac_cost: false,
        condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
    }
}

/// Mana ability shorthand: `{T}: Add {C}.` (one true colorless pip, not
/// generic). Used by colorless-only lands (Wastes, Petrified Hamlet) and
/// Eldrazi-aligned utility lands.
pub fn tap_add_colorless() -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        mana_cost: ManaCost::default(),
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Colorless(Value::Const(1)),
        },
        once_per_turn: false,
        sorcery_speed: false,
        sac_cost: false,
        condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
    }
}

pub fn no_abilities() -> Vec<ActivatedAbility> {
    vec![]
}

// ── Land helpers shared across set modules ───────────────────────────────────

/// Triggered ability: when this permanent enters the battlefield, tap it.
pub fn etb_tap() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::Tap { what: Selector::This },
    }
}

/// Triggered ability: when this permanent enters, tap it AND surveil 1.
pub fn etb_tap_then_surveil_one() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::Seq(vec![
            Effect::Tap { what: Selector::This },
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
    }
}

/// Fastland ETB trigger: "ETB tapped unless you control two or fewer other
/// lands." Counted against the post-ETB battlefield (which already contains
/// this land), so the threshold is "≥ 4 lands you control".
pub fn fastland_etb_conditional_tap() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::Land
                        .and(SelectionRequirement::ControlledByYou),
                ),
                n: Value::Const(4),
            },
            then: Box::new(Effect::Tap { what: Selector::This }),
            else_: Box::new(Effect::Noop),
        },
    }
}

/// Shock-land ETB choice — "As this enters, you may pay 2 life. If you don't,
/// it enters tapped." Modeled as a self-source ETB `ChooseMode` trigger
/// (mode 0 = pay 2 life, mode 1 = tap self). The default `AutoDecider` and
/// the simulated bot both pick mode 0, which matches typical play (a single
/// untap is almost always worth 2 life). Note: this is a triggered ability,
/// not a true replacement effect — the land is briefly available untapped
/// before the trigger resolves. Functionally close enough for the demo decks.
pub fn shockland_pay_two_or_tap() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::ChooseMode(vec![
            // Mode 0: Pay 2 life, stay untapped.
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
            // Mode 1: enter tapped.
            Effect::Tap { what: Selector::This },
        ]),
    }
}

/// Skeleton for a non-basic land with two color-producing mana abilities and
/// optionally an ETB-tapped trigger and the corresponding `LandType`s.
pub fn dual_land_with(
    name: &'static str,
    type_a: LandType,
    type_b: LandType,
    color_a: Color,
    color_b: Color,
    triggers: Vec<TriggeredAbility>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![type_a, type_b],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add(color_a), tap_add(color_b)],
        triggered_abilities: triggers,
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

pub mod all;
pub mod ap;
pub mod arn;
pub mod dis;
pub mod fem;
pub mod gpt;
pub mod ice;
pub mod inv;
pub mod lea;
pub mod m11;
pub mod ogw;
pub mod pc2;
pub mod por;
pub mod rav;
pub mod rtr;
pub mod ths;
pub mod tmp;
pub mod zen;
pub mod decks;
pub mod mod_set;
pub mod sos;
pub mod stx;
