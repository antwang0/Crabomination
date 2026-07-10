//! Modern Horizons 3 (MH3), batch 5. The ten-card "Landscape" common land
//! cycle (colorless tap + sac-fetch three basics tapped + tricolor cycling),
//! plus a handful of energy/replicate/aura payoffs. Tests in `tests/mh3e.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, MayPlayDuration, Predicate, SelectionRequirement as R, Subtypes,
    TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{b, cost, generic, g, r, u, w, ManaSymbol};

/// One member of the MH3 "Landscape" cycle: `{T}: Add {C}`; `{T}, Sacrifice:
/// fetch a basic of one of three types onto the battlefield tapped`; Cycling
/// for its three colors.
fn landscape(name: &'static str, types: [LandType; 3], cycle: &[ManaSymbol]) -> CardDefinition {
    let filter = R::IsBasicLand.and(
        R::HasLandType(types[0])
            .or(R::HasLandType(types[1]))
            .or(R::HasLandType(types[2])),
    );
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        keywords: vec![Keyword::Cycling(cost(cycle))],
        activated_abilities: vec![
            super::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

use LandType::{Forest as F, Island as I, Mountain as M, Plains as P, Swamp as S};

pub fn bountiful_landscape() -> CardDefinition {
    landscape("Bountiful Landscape", [F, I, M], &[g(), u(), r()])
}
pub fn contaminated_landscape() -> CardDefinition {
    landscape("Contaminated Landscape", [P, I, S], &[w(), u(), b()])
}
pub fn deceptive_landscape() -> CardDefinition {
    landscape("Deceptive Landscape", [P, S, F], &[w(), b(), g()])
}
pub fn foreboding_landscape() -> CardDefinition {
    landscape("Foreboding Landscape", [S, F, I], &[b(), g(), u()])
}
pub fn perilous_landscape() -> CardDefinition {
    landscape("Perilous Landscape", [I, M, P], &[u(), r(), w()])
}
pub fn seething_landscape() -> CardDefinition {
    landscape("Seething Landscape", [I, S, M], &[u(), b(), r()])
}
pub fn shattered_landscape() -> CardDefinition {
    landscape("Shattered Landscape", [M, P, S], &[r(), w(), b()])
}
pub fn sheltering_landscape() -> CardDefinition {
    landscape("Sheltering Landscape", [M, F, P], &[r(), g(), w()])
}
pub fn tranquil_landscape() -> CardDefinition {
    landscape("Tranquil Landscape", [F, P, I], &[g(), w(), u()])
}
pub fn twisted_landscape() -> CardDefinition {
    landscape("Twisted Landscape", [S, M, F], &[b(), r(), g()])
}

/// Vega, the Watcher — {1}{W}{U} 2/2 Bird Spirit with flying. Whenever you cast
/// a spell from anywhere other than your hand, draw a card.
pub fn vega_the_watcher() -> CardDefinition {
    CardDefinition {
        name: "Vega, the Watcher",
        cost: cost(&[generic(1), w(), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![crate::card::Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::SpellNotCastFromHand,
                },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Argent Dais — {1}{W} artifact. Enters with two oil counters. Whenever two
/// or more creatures attack, put an oil counter on it. {2}, {T}, remove two
/// oil: exile another target nonland permanent; its controller draws two.
pub fn argent_dais() -> CardDefinition {
    CardDefinition {
        name: "Argent Dais",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        enters_with_counters: Some((CounterType::Oil, Value::Const(2))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::AnyPlayer).with_filter(
                Predicate::AttackedWithCountAtLeast { who: PlayerRef::ActivePlayer, at_least: 2 },
            ),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Oil, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Oil, 2)),
            effect: Effect::Seq(vec![
                Effect::Exile { what: target_filtered(R::Permanent.and(R::Nonland).and(R::OtherThanSource)) },
                Effect::Draw {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::Const(2),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Glimpse the Impossible — {2}{R} sorcery. Exile the top three cards; you may
/// play them this turn. At the next end step, each still-exiled card is put
/// into your graveyard and makes a 0/1 Eldrazi Spawn.
pub fn glimpse_the_impossible() -> CardDefinition {
    CardDefinition {
        name: "Glimpse the Impossible",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::Const(3),
            duration: MayPlayDuration::EndOfThisTurn,
            pay_any_color: false,
            uncast_penalty: Some(Box::new(Effect::Seq(vec![
                Effect::Move { what: Selector::Target(0), to: ZoneDest::Graveyard },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: crabomination_base::tokens::eldrazi_spawn_token(),
                },
            ]))),
        },
        ..Default::default()
    }
}

/// Chthonian Nightmare — {1}{B} enchantment. ETB: get {E}{E}{E}. Pay X {E},
/// sacrifice a creature, return this to hand: reanimate a creature card with
/// mana value X from your graveyard. Sorcery-speed.
pub fn chthonian_nightmare() -> CardDefinition {
    CardDefinition {
        name: "Chthonian Nightmare",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::AddEnergy(Value::Const(3)))],
        activated_abilities: vec![ActivatedAbility {
            sorcery_speed: true,
            energy_x_cost: true,
            sac_other_filter: Some((R::Creature, 1)),
            return_self_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::ManaValueExactlyXFromCost)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
