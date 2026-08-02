//! Judgment (JUD) — the closing wave: the Wish cycle, the blue prison
//! enchantments and the graveyard-matters rares. Tests in `classic_sets/jud`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, DynamicPt, Keyword,
    SelectionRequirement as R, StaticAbility, Subtypes, TriggeredAbility,
};
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, Selector, StaticEffect, Value,
    ZoneDest,
    shortcut::{draw, etb, target_filtered},
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};
use crabomination_base::turn_step::TurnStep;

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

/// "You may reveal a [filter] card you own from outside the game and put it
/// into your hand. Exile this." The Judgment Wish cycle.
fn wish(name: &'static str, c: ManaCost, filter: R, instant_speed: bool) -> CardDefinition {
    let mut def = if instant_speed {
        instant(name, c, Effect::WishToHand { filter })
    } else {
        sorcery(name, c, Effect::WishToHand { filter })
    };
    def.exile_on_resolve = true;
    def
}

pub fn burning_wish() -> CardDefinition {
    wish("Burning Wish", cost(&[generic(1), r()]), R::HasCardType(CardType::Sorcery), false)
}

pub fn cunning_wish() -> CardDefinition {
    wish("Cunning Wish", cost(&[generic(2), u()]), R::HasCardType(CardType::Instant), true)
}

pub fn living_wish() -> CardDefinition {
    wish("Living Wish", cost(&[generic(1), g()]), R::Creature.or(R::Land), false)
}

pub fn golden_wish() -> CardDefinition {
    wish("Golden Wish", cost(&[generic(3), w(), w()]), R::Artifact.or(R::Enchantment), false)
}

/// Death Wish — any card from outside the game, for half your life.
pub fn death_wish() -> CardDefinition {
    CardDefinition {
        exile_on_resolve: true,
        ..sorcery(
            "Death Wish",
            cost(&[generic(1), b(), b()]),
            Effect::Seq(vec![
                Effect::WishToHand { filter: R::Any },
                Effect::LoseHalfLife { who: Selector::You, rounded_up: true },
            ]),
        )
    }
}

/// Grave Consequences — everyone shrinks their graveyard or pays for it.
pub fn grave_consequences() -> CardDefinition {
    instant(
        "Grave Consequences",
        cost(&[generic(1), b()]),
        Effect::EachPlayerMayExileAnyNumberFromGraveyard {
            then: Box::new(Effect::Seq(vec![
                Effect::LoseLifePerCardInGraveyard {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    filter: R::Any,
                    per: Value::Const(1),
                },
                draw(1),
            ])),
        },
    )
}

/// Cephalid Constable — connects and bounces one permanent per damage.
pub fn cephalid_constable() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::Take {
                    inner: Box::new(Selector::ControlledBy {
                        who: PlayerRef::TriggerEventPlayer,
                        filter: R::Permanent,
                    }),
                    count: Box::new(Value::TriggerEventAmount),
                },
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        }],
        ..creature(
            "Cephalid Constable",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Octopus, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Mist of Stagnation — nothing untaps; you buy permanents back with
/// graveyard cards.
pub fn mist_of_stagnation() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Permanents don't untap during their controllers' untap steps.",
            effect: StaticEffect::PermanentsDontUntap,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::UntapChosenPerCardInGraveyard { who: PlayerRef::ActivePlayer },
        }],
        ..enchantment("Mist of Stagnation", cost(&[generic(3), u(), u()]))
    }
}

/// Planar Chaos — every spell is a coin flip, and so is keeping this around.
pub fn planar_chaos() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::FlipCoin {
                    count: Value::Const(1),
                    on_heads: Box::new(Effect::Noop),
                    on_tails: Box::new(Effect::SacrificeSource),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
                effect: Effect::FlipCoinBy {
                    flipper: PlayerRef::Triggerer,
                    on_heads: Box::new(Effect::Noop),
                    on_tails: Box::new(Effect::CounterSpell { what: Selector::TriggerSource }),
                },
            },
        ],
        ..enchantment("Planar Chaos", cost(&[generic(2), r()]))
    }
}

/// Prismatic Strands — a colour-scoped fog, castable twice off a white
/// creature's tap.
pub fn prismatic_strands() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FlashbackTap {
            count: 1,
            filter: Some(Box::new(R::Creature.and(R::HasColor(Color::White)))),
        }],
        ..instant(
            "Prismatic Strands",
            cost(&[generic(2), w()]),
            Effect::PreventAllDamageFromChosenColorGlobally,
        )
    }
}

/// Riftstone Portal — a colorless land that fixes {G}/{W} from the graveyard.
pub fn riftstone_portal() -> CardDefinition {
    CardDefinition {
        name: "Riftstone Portal",
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::Const(1)),
            },
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "As long as this card is in your graveyard, lands you control have \"{T}: Add {G} or {W}.\"",
            effect: StaticEffect::GrantActivatedAbilityFromGraveyard {
                applies_to: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
                ability: Box::new(ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::OfColors(
                            vec![Color::Green, Color::White],
                            Value::Const(1),
                        ),
                    },
                    ..Default::default()
                }),
            },
        }],
        ..Default::default()
    }
}

/// Scalpelexis — mills four at a time, again on any duplicate name.
pub fn scalpelexis() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::ExileTopRepeatOnDuplicateNames {
                who: PlayerRef::TriggerEventPlayer,
                count: Value::Const(4),
            },
        }],
        ..creature("Scalpelexis", cost(&[generic(4), u()]), vec![CreatureType::Beast], 1, 5)
    }
}

/// Shaman's Trance — every graveyard is yours for the turn, and no one
/// else's is theirs.
pub fn shamans_trance() -> CardDefinition {
    instant("Shaman's Trance", cost(&[generic(2), r()]), Effect::ShamansTrance)
}

/// Soulgorger Orgg — a 6/6 trampler mortgaged against your life total.
pub fn soulgorger_orgg() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            etb(Effect::LoseAllButLifeRemembered {
                who: PlayerRef::You,
                keep: Value::Const(1),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::RememberedAmountOfSource,
                },
            },
        ],
        ..creature(
            "Soulgorger Orgg",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Nightmare],
            6,
            6,
        )
    }
}

/// Spelljack — counter it, then cast it yourself for free.
pub fn spelljack() -> CardDefinition {
    instant(
        "Spelljack",
        cost(&[generic(3), u(), u(), u()]),
        Effect::CounterSpellExileMayPlayFree { what: Selector::Target(0) },
    )
}

/// Sutured Ghoul — a `*/*` trampler stitched from your dead creatures.
pub fn sutured_ghoul() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        as_enters_effect: Some(Effect::ExileAnyNumberFromGraveyardOnSource {
            filter: R::Creature,
        }),
        dynamic_pt: Some(DynamicPt::ExiledWithSourceTotals),
        ..creature(
            "Sutured Ghoul",
            cost(&[generic(4), b(), b(), b()]),
            vec![CreatureType::Zombie],
            0,
            0,
        )
    }
}

/// Telekinetic Bonds — every discard buys a tap or untap for {1}{U}.
pub fn telekinetic_bonds() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::AnyPlayer),
            effect: Effect::MayPay {
                description: "Pay {1}{U} to tap or untap target permanent?".into(),
                mana_cost: cost(&[generic(1), u()]),
                body: Box::new(Effect::TapOrUntap { what: target_filtered(R::Permanent) }),
                else_: None,
            },
        }],
        ..enchantment("Telekinetic Bonds", cost(&[generic(2), u(), u(), u()]))
    }
}

/// Web of Inertia — opponents pay a graveyard card each combat or can't
/// attack you.
pub fn web_of_inertia() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::OpponentControl,
            ),
            effect: Effect::MayExileFromGraveyardElse {
                who: PlayerRef::ActivePlayer,
                otherwise: Box::new(Effect::CantAttackPlayerThisTurn {
                    who: PlayerRef::ActivePlayer,
                    defender: PlayerRef::You,
                }),
            },
        }],
        ..enchantment("Web of Inertia", cost(&[generic(2), u()]))
    }
}
