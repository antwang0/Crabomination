//! Weatherlight (WTH) — the Mirage block's third set. Tests in
//! `classic_sets/wth`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CreatureType, EnchantmentSubtype,
    EquipBonus, EquipScale, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R,
    StaticAbility, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{deal, draw, target_any, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect, Value, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

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

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn aura(name: &'static str, c: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..enchantment(name, c)
    }
}

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { card_types: vec![CardType::Sorcery], ..instant(name, c, effect) }
}

fn artifact(name: &'static str, c: ManaCost, abilities: Vec<ActivatedAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Artifact],
        activated_abilities: abilities,
        ..Default::default()
    }
}

fn your_upkeep() -> EventSpec {
    EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
}

fn dies() -> EventSpec {
    EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource)
}

/// "When this creature enters, sacrifice it unless you [cost]."
fn etb_sacrifice_unless(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
    keep: Effect,
) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::MayDoElse {
            description: format!("Pay {name}'s upkeep cost?"),
            body: Box::new(keep),
            else_: Box::new(Effect::SacrificeSource),
        })],
        ..creature(name, c, types, p, t)
    }
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Abyssal Gatekeeper — {1}{B} 1/1. Its death is an edict for the table.
pub fn abyssal_gatekeeper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: dies(),
            effect: Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachPlayer),
                count: Value::ONE,
                filter: R::Creature,
            },
        }],
        ..creature("Abyssal Gatekeeper", cost(&[generic(1), b()]), vec![CreatureType::Horror], 1, 1)
    }
}

/// Barishi — {2}{G}{G} 4/3 that recycles the graveyard's creatures on death.
pub fn barishi() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: dies(),
            effect: Effect::Seq(vec![
                Effect::Exile { what: Selector::This },
                Effect::ShuffleFilteredGraveyardIntoLibrary {
                    who: PlayerRef::You,
                    filter: R::Creature,
                },
            ]),
        }],
        ..creature("Barishi", cost(&[generic(2), g(), g()]), vec![CreatureType::Elemental], 4, 3)
    }
}

/// Benalish Knight — {2}{W} 2/2 flash first striker.
pub fn benalish_knight() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::FirstStrike],
        ..creature(
            "Benalish Knight",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Bogardan Firefiend — {2}{R} 2/1 that burns a creature on the way out.
pub fn bogardan_firefiend() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: dies(),
            effect: deal(2, target_filtered(R::Creature)),
        }],
        ..creature(
            "Bogardan Firefiend",
            cost(&[generic(2), r()]),
            vec![CreatureType::Elemental, CreatureType::Spirit],
            2,
            1,
        )
    }
}

/// Cinder Giant — {3}{R} 5/3 that scorches your own board each upkeep.
pub fn cinder_giant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::DealDamage {
                to: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                amount: Value::Const(2),
            },
        }],
        ..creature("Cinder Giant", cost(&[generic(3), r()]), vec![CreatureType::Giant], 5, 3)
    }
}

/// Cinder Wall — {R} 3/3 defender that burns out after one block.
pub fn cinder_wall() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
            effect: Effect::AtEndOfCombat {
                body: Box::new(Effect::Destroy { what: Selector::This }),
            },
        }],
        ..creature("Cinder Wall", cost(&[r()]), vec![CreatureType::Wall], 3, 3)
    }
}

/// Cloud Djinn — {5}{U} 5/4 flier that only answers other fliers.
pub fn cloud_djinn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::CanBlockOnlyFlying],
        ..creature("Cloud Djinn", cost(&[generic(5), u()]), vec![CreatureType::Djinn], 5, 4)
    }
}

/// Duskrider Falcon — {1}{W} 1/1 flier with protection from black.
pub fn duskrider_falcon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Protection(Color::Black)],
        ..creature("Duskrider Falcon", cost(&[generic(1), w()]), vec![CreatureType::Bird], 1, 1)
    }
}

/// Dwarven Thaumaturgist — {2}{R} 1/2 that flips a creature's stats.
pub fn dwarven_thaumaturgist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::SwitchPowerToughness {
                what: target_filtered(R::Creature),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Dwarven Thaumaturgist",
            cost(&[generic(2), r()]),
            vec![CreatureType::Dwarf, CreatureType::Shaman],
            1,
            2,
        )
    }
}

/// Fallow Wurm — {2}{G} 4/4 that costs a land card out of your hand.
pub fn fallow_wurm() -> CardDefinition {
    etb_sacrifice_unless(
        "Fallow Wurm",
        cost(&[generic(2), g()]),
        vec![CreatureType::Wurm],
        4,
        4,
        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
    )
}

/// Fledgling Djinn — {1}{B} 2/2 flier that bites you each upkeep.
pub fn fledgling_djinn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: deal(1, Selector::You),
        }],
        ..creature("Fledgling Djinn", cost(&[generic(1), b()]), vec![CreatureType::Djinn], 2, 2)
    }
}

/// Fog Elemental — {2}{U} 4/4 flier that evaporates after one combat.
pub fn fog_elemental() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::SacrificeAtEndOfCombat { what: Selector::This },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: Effect::SacrificeAtEndOfCombat { what: Selector::This },
            },
        ],
        ..creature("Fog Elemental", cost(&[generic(2), u()]), vec![CreatureType::Elemental], 4, 4)
    }
}

/// Heavy Ballista — {3}{W} 2/3 that shoots into combat.
pub fn heavy_ballista() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: deal(2, target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking)))),
            ..Default::default()
        }],
        ..creature(
            "Heavy Ballista",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            3,
        )
    }
}

/// Hidden Horror — {1}{B}{B} 4/4 that eats a creature card on arrival.
pub fn hidden_horror() -> CardDefinition {
    etb_sacrifice_unless(
        "Hidden Horror",
        cost(&[generic(1), b(), b()]),
        vec![CreatureType::Horror],
        4,
        4,
        Effect::MayDiscardMatching {
            description: "Discard a creature card?".to_string(),
            count: Value::ONE,
            filter: R::Creature,
            then: Box::new(Effect::Noop),
            else_: Some(Box::new(Effect::SacrificeSource)),
        },
    )
}

/// Hurloon Shaman — {1}{R}{R} 2/3 whose death costs everyone a land.
pub fn hurloon_shaman() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: dies(),
            effect: Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachPlayer),
                count: Value::ONE,
                filter: R::Land,
            },
        }],
        ..creature(
            "Hurloon Shaman",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Minotaur, CreatureType::Shaman],
            2,
            3,
        )
    }
}

/// Jangling Automaton — {3} 3/2 that unlocks the defender's blockers.
pub fn jangling_automaton() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Untap {
                what: Selector::ControlledBy {
                    who: PlayerRef::DefendingPlayer,
                    filter: R::Creature,
                },
                up_to: None,
            },
        }],
        ..creature("Jangling Automaton", cost(&[generic(3)]), vec![CreatureType::Construct], 3, 2)
    }
}

// ── Enchantments & Auras ────────────────────────────────────────────────────

/// Coils of the Medusa — {1}{B} Aura. +1/-1, and it can be cashed in to wipe
/// the host's blockers.
pub fn coils_of_the_medusa() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Destroy {
                what: Selector::EachPermanent(
                    R::Creature
                        .and(R::IsBlocking)
                        .and(R::HasCreatureType(CreatureType::Wall).negate()),
                ),
            },
            ..Default::default()
        }],
        ..aura(
            "Coils of the Medusa",
            cost(&[generic(1), b()]),
            EquipBonus { power: 1, toughness: -1, ..Default::default() },
        )
    }
}

/// Empyrial Armor — {1}{W}{W} Aura. +1/+1 for each card in your hand.
pub fn empyrial_armor() -> CardDefinition {
    aura(
        "Empyrial Armor",
        cost(&[generic(1), w(), w()]),
        EquipBonus {
            scale: Some(EquipScale {
                filter: R::Any,
                per_power: 1,
                per_toughness: 1,
                count_source_controller_hand: true,
                ..Default::default()
            }),
            ..Default::default()
        },
    )
}

/// Familiar Ground — {2}{G}. Your creatures can only ever be single-blocked.
pub fn familiar_ground() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each creature you control can't be blocked by more than one creature.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::CantBeBlockedByMoreThanOne,
            },
        }],
        ..enchantment("Familiar Ground", cost(&[generic(2), g()]))
    }
}

/// Festering Evil — {3}{B}{B}. A slow board sweeper that can be cashed in.
pub fn festering_evil() -> CardDefinition {
    let sweep = |n: i32| {
        Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature),
                amount: Value::Const(n),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(n),
            },
        ])
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility { event: your_upkeep(), effect: sweep(1) }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), b()]),
            sac_cost: true,
            effect: sweep(3),
            ..Default::default()
        }],
        ..enchantment("Festering Evil", cost(&[generic(3), b(), b()]))
    }
}

/// Fire Whip — {1}{R} Aura on your own creature: a repeatable ping that can
/// also be cashed in for one more.
pub fn fire_whip() -> CardDefinition {
    CardDefinition {
        attach_only_filter: Some(R::Creature.and(R::ControlledByYou)),
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        },
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: deal(1, target_any()),
            ..Default::default()
        }],
        ..aura(
            "Fire Whip",
            cost(&[generic(1), r()]),
            EquipBonus {
                activated_abilities: vec![ActivatedAbility {
                    tap_cost: true,
                    effect: deal(1, target_any()),
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
    }
}

/// Downdraft — {2}{G}. Grounds a flier at a time, or all of them at once.
pub fn downdraft() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[g()]),
                effect: Effect::LoseKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                sac_cost: true,
                effect: Effect::DealDamage {
                    to: Selector::EachPermanent(
                        R::Creature.and(R::HasKeyword(Keyword::Flying)),
                    ),
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
        ],
        ..enchantment("Downdraft", cost(&[generic(2), g()]))
    }
}

/// Infernal Tribute — {B}{B}{B}. Turns spare permanents into cards.
pub fn infernal_tribute() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((R::Permanent.and(R::IsToken.negate()), 1)),
            effect: draw(1),
            ..Default::default()
        }],
        ..enchantment("Infernal Tribute", cost(&[b(), b(), b()]))
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Bubble Matrix — {4}. Nothing takes damage while it's out.
pub fn bubble_matrix() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Prevent all damage that would be dealt to creatures.",
            effect: StaticEffect::PreventAllDamageToCreatures,
        }],
        ..artifact("Bubble Matrix", cost(&[generic(4)]), vec![])
    }
}

/// Dingus Staff — {4}. Every death bills its controller for 2.
pub fn dingus_staff() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
            effect: deal(2, Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)))),
        }],
        ..artifact("Dingus Staff", cost(&[generic(4)]), vec![])
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Abjure — {U}. A blue permanent buys a hard counter.
pub fn abjure() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::Permanent.and(R::HasColor(Color::Blue)),
            count: 1,
        }],
        ..instant("Abjure", cost(&[u()]), crate::effect::shortcut::counter_target_spell())
    }
}

/// Argivian Find — {W}. Buy back an artifact or enchantment card.
pub fn argivian_find() -> CardDefinition {
    instant(
        "Argivian Find",
        cost(&[w()]),
        Effect::Move {
            what: target_filtered(R::Artifact.or(R::Enchantment).and(R::InYourGraveyard)),
            to: ZoneDest::Hand(PlayerRef::You),
        },
    )
}

/// Argivian Restoration — {2}{U}{U}. Reanimate an artifact.
pub fn argivian_restoration() -> CardDefinition {
    sorcery(
        "Argivian Restoration",
        cost(&[generic(2), u(), u()]),
        Effect::Move {
            what: target_filtered(R::Artifact.and(R::InYourGraveyard)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
    )
}

/// Blossoming Wreath — {G}. Life for every creature you've lost.
pub fn blossoming_wreath() -> CardDefinition {
    instant(
        "Blossoming Wreath",
        cost(&[g()]),
        Effect::GainLife {
            who: Selector::You,
            amount: Value::CardsInGraveyardMatching {
                who: PlayerRef::You,
                filter: R::Creature,
            },
        },
    )
}

/// Boiling Blood — {2}{R}. Force a creature into combat and replace itself.
pub fn boiling_blood() -> CardDefinition {
    instant(
        "Boiling Blood",
        cost(&[generic(2), r()]),
        Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::MustAttack,
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
    )
}

/// Disrupt — {U}. A soft counter for spells that replaces itself.
pub fn disrupt() -> CardDefinition {
    instant(
        "Disrupt",
        cost(&[u()]),
        Effect::Seq(vec![
            Effect::CounterUnlessPaid {
                what: target_filtered(
                    R::IsSpellOnStack
                        .and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
                ),
                mana_cost: cost(&[generic(1)]),
                exile: false,
                extra_generic: None,
            },
            draw(1),
        ]),
    )
}

/// Fatal Blow — {B}. Finishes off anything that's already been hit.
pub fn fatal_blow() -> CardDefinition {
    instant(
        "Fatal Blow",
        cost(&[b()]),
        Effect::DestroyNoRegen {
            what: target_filtered(R::Creature.and(R::DealtDamageThisTurn)),
        },
    )
}

/// Fit of Rage — {1}{R}. +3/+3 and first strike.
pub fn fit_of_rage() -> CardDefinition {
    sorcery(
        "Fit of Rage",
        cost(&[generic(1), r()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
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

/// Guided Strike — {1}{W}. A combat trick that replaces itself.
pub fn guided_strike() -> CardDefinition {
    instant(
        "Guided Strike",
        cost(&[generic(1), w()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
    )
}
