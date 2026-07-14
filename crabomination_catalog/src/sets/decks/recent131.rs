//! A Wilds of Eldraine (WOE) wave: enchantment-matters triggers, Role tokens,
//! and a `*/*` nonland-permanent CDA (Regal Bunnicorn). All ride existing
//! primitives plus `DynamicPt::NonlandPermanentsControlled`. Tests in
//! `crabomination/src/tests/recent131.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, Selector, Subtypes, TokenDefinition, TriggeredAbility, Value,
    WardCost,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef};
use crate::game::effects::treasure_token;
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, Color};
use super::woe_roles::{cursed_role, sorcerer_role};



/// 1/1 black Rat token with "This token can't block."
fn rat_token() -> TokenDefinition {
    TokenDefinition {
        name: "Rat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rat], ..Default::default() },
        keywords: vec![Keyword::CantBlock],
        ..Default::default()
    }
}

// ── White ─────────────────────────────────────────────────────────────────────

/// Regal Bunnicorn — {1}{W} Rabbit Unicorn. Power and toughness are each equal
/// to the number of nonland permanents you control.
pub fn regal_bunnicorn() -> CardDefinition {
    CardDefinition {
        name: "Regal Bunnicorn",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit, CreatureType::Unicorn],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        dynamic_pt: Some(DynamicPt::NonlandPermanentsControlled { base: 0 }),
        ..Default::default()
    }
}

/// Rimefur Reindeer — {3}{W} 3/4 Elk. Whenever an enchantment you control
/// enters, tap target creature an opponent controls.
pub fn rimefur_reindeer() -> CardDefinition {
    CardDefinition {
        name: "Rimefur Reindeer",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elk], ..Default::default() },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Enchantment,
                }),
            effect: Effect::Tap {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            },
        }],
        ..Default::default()
    }
}

/// Frostbridge Guard — {1}{W} 2/2 Elemental Soldier. {2}{W}, {T}: Tap target
/// creature.
pub fn frostbridge_guard() -> CardDefinition {
    CardDefinition {
        name: "Frostbridge Guard",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            tap_cost: true,
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Savior of the Sleeping — {2}{W} 2/3 Human Knight with vigilance. Whenever an
/// enchantment you control is put into a graveyard from the battlefield, put a
/// +1/+1 counter on this creature.
pub fn savior_of_the_sleeping() -> CardDefinition {
    CardDefinition {
        name: "Savior of the Sleeping",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Enchantment,
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Unassuming Sage — {1}{W} 2/2 Human Peasant Wizard. ETB: you may pay {2}; if
/// you do, create a Sorcerer Role token attached to it.
pub fn unassuming_sage() -> CardDefinition {
    CardDefinition {
        name: "Unassuming Sage",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Peasant, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::MayPay {
            description: "Pay {2} to create a Sorcerer Role attached to this creature?".into(),
            mana_cost: cost(&[generic(2)]),
            body: Box::new(Effect::CreateTokenAttachedTo {
                target: Selector::This,
                definition: sorcerer_role(),
            }),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Spellbook Vendor — {1}{W} 2/2 Human Peasant with vigilance. At the beginning
/// of combat on your turn, you may pay {1}; if you do, create a Sorcerer Role
/// token attached to target creature you control.
pub fn spellbook_vendor() -> CardDefinition {
    CardDefinition {
        name: "Spellbook Vendor",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Peasant],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer),
            effect: Effect::MayPay {
                description: "Pay {1} to create a Sorcerer Role attached to a creature you control?"
                    .into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::CreateTokenAttachedTo {
                    target: target_filtered(R::Creature.and(R::ControlledByYou)),
                    definition: sorcerer_role(),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

// ── Blue ──────────────────────────────────────────────────────────────────────

/// Snaremaster Sprite — {U} 1/1 Faerie Wizard with flying. ETB: you may pay {2};
/// if you do, tap target creature an opponent controls and put a stun counter
/// on it.
pub fn snaremaster_sprite() -> CardDefinition {
    CardDefinition {
        name: "Snaremaster Sprite",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie, CreatureType::Wizard], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::MayPay {
            description: "Pay {2} to tap and stun a creature an opponent controls?".into(),
            mana_cost: cost(&[generic(2)]),
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Creature.and(R::ControlledByOpponent),
                effect: Box::new(Effect::Seq(vec![
                    Effect::Tap { what: Selector::Target(0) },
                    Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::Stun,
                        amount: Value::ONE,
                    },
                ])),
            }),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Stormkeld Prowler — {1}{U} 2/1 Human Rogue. Whenever you cast a spell with
/// mana value 5 or greater, put two +1/+1 counters on this creature.
pub fn stormkeld_prowler() -> CardDefinition {
    CardDefinition {
        name: "Stormkeld Prowler",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Rogue], ..Default::default() },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::ManaValueAtLeast(5),
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

// ── Black ─────────────────────────────────────────────────────────────────────

/// Warehouse Tabby — {B} 1/1 Cat. Whenever an enchantment you control is put
/// into a graveyard from the battlefield, create a Rat token. {1}{B}: gains
/// deathtouch until end of turn.
pub fn warehouse_tabby() -> CardDefinition {
    CardDefinition {
        name: "Warehouse Tabby",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Enchantment,
                }),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: rat_token() },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wicked Visitor — {1}{B} 2/2 Nightmare. Whenever an enchantment you control is
/// put into a graveyard from the battlefield, each opponent loses 1 life.
pub fn wicked_visitor() -> CardDefinition {
    CardDefinition {
        name: "Wicked Visitor",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Nightmare], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Enchantment,
                }),
            effect: Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Spiteful Hexmage — {B} 3/2 Human Warlock. ETB: create a Cursed Role token
/// attached to target creature you control.
pub fn spiteful_hexmage() -> CardDefinition {
    CardDefinition {
        name: "Spiteful Hexmage",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Warlock], ..Default::default() },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateTokenAttachedTo {
            target: target_filtered(R::Creature.and(R::ControlledByYou)),
            definition: cursed_role(),
        })],
        ..Default::default()
    }
}

// ── Red ───────────────────────────────────────────────────────────────────────

/// Harried Spearguard — {R} 1/1 Human Soldier with haste. When this creature
/// dies, create a Rat token.
pub fn harried_spearguard() -> CardDefinition {
    CardDefinition {
        name: "Harried Spearguard",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Soldier], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: rat_token() },
        }],
        ..Default::default()
    }
}

/// Skewer Slinger — {1}{R} 1/3 Dwarf Knight with reach. Whenever it blocks or
/// becomes blocked by a creature, it deals 1 damage to that creature.
pub fn skewer_slinger() -> CardDefinition {
    let ping_blocked = || Effect::DealDamage { to: Selector::BlockedAttacker, amount: Value::ONE };
    let ping_blockers = || Effect::DealDamage { to: Selector::BlockingCreatures, amount: Value::ONE };
    CardDefinition {
        name: "Skewer Slinger",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dwarf, CreatureType::Knight], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![
            TriggeredAbility { event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource), effect: ping_blocked() },
            TriggeredAbility { event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource), effect: ping_blockers() },
        ],
        ..Default::default()
    }
}

/// Redcap Thief — {2}{R} 2/3 Goblin Rogue. ETB: create a Treasure token.
pub fn redcap_thief() -> CardDefinition {
    CardDefinition {
        name: "Redcap Thief",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin, CreatureType::Rogue], ..Default::default() },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: treasure_token(),
        })],
        ..Default::default()
    }
}

// ── Green ─────────────────────────────────────────────────────────────────────

/// Rootrider Faun — {1}{G} 1/3 Satyr Scout. {T}: Add {G}. {1}, {T}: Add one
/// mana of any color.
pub fn rootrider_faun() -> CardDefinition {
    CardDefinition {
        name: "Rootrider Faun",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Satyr, CreatureType::Scout], ..Default::default() },
        power: 1,
        toughness: 3,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Green]) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Toadstool Admirer — {G} 1/1 Ouphe with ward {2}. {3}{G}: put a +1/+1 counter
/// on this creature.
pub fn toadstool_admirer() -> CardDefinition {
    CardDefinition {
        name: "Toadstool Admirer",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ouphe], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Territorial Witchstalker — {1}{G} 2/3 Wolf with defender. At the beginning of
/// combat on your turn, if you control a creature with power 4 or greater, it
/// gets +1/+0 until end of turn and can attack this turn as though it didn't
/// have defender.
pub fn territorial_witchstalker() -> CardDefinition {
    CardDefinition {
        name: "Territorial Witchstalker",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer)
                .with_filter(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4)),
                    ),
                    n: Value::ONE,
                }),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::AttackDespiteDefenderThisTurn { what: Selector::This },
            ]),
        }],
        ..Default::default()
    }
}

/// Tanglespan Lookout — {2}{G} 2/3 Satyr. Whenever an Aura you control enters,
/// draw a card.
pub fn tanglespan_lookout() -> CardDefinition {
    CardDefinition {
        name: "Tanglespan Lookout",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Satyr], ..Default::default() },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Aura),
                }),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}
