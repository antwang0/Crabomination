//! A Wilds of Eldraine (WOE) wave: Adventures, Roles, Food/Treasure value, and
//! tap-down control. Introduces `Predicate::CastSpellIsAdventure` (Chancellor
//! of Tales) and `Effect::CreateTokenAttachedToEach` (Asinine Antics). Other
//! cards ride existing primitives. Tests in `crabomination/src/tests/recent135.rs`.

use crate::card::{
    ActivatedAbility, Adventure, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement as R, Selector, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, ZoneDest, ZoneRef};
use crate::game::effects::{food_token, treasure_token};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, g, generic, r, u, w};

use super::woe_roles::{cursed_role, monster_role, young_hero_role};

// ── White ─────────────────────────────────────────────────────────────────────

/// Cooped Up — {1}{W} Aura. Enchant creature; it can't attack or block. {2}{W}:
/// Exile enchanted creature.
pub fn cooped_up() -> CardDefinition {
    CardDefinition {
        name: "Cooped Up",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::CantAttack, Keyword::CantBlock],
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            effect: Effect::Exile {
                what: Selector::AttachedTo(Box::new(Selector::This)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Blue ──────────────────────────────────────────────────────────────────────

/// Chancellor of Tales — {3}{U} 2/3 Faerie Advisor with flying. Whenever you
/// cast an Adventure spell, copy it (you may choose new targets).
pub fn chancellor_of_tales() -> CardDefinition {
    CardDefinition {
        name: "Chancellor of Tales",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Advisor],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellIsAdventure),
            effect: Effect::CopySpellMayChooseTargets {
                what: Selector::TriggerSource,
                count: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Asinine Antics — {2}{U}{U} Sorcery. For each creature your opponents control,
/// create a Cursed Role token attached to that creature. (The flash-for-{2}-more
/// mode is omitted.)
pub fn asinine_antics() -> CardDefinition {
    CardDefinition {
        name: "Asinine Antics",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateTokenAttachedToEach {
            target: Selector::EachMatching {
                zone: ZoneRef::Battlefield,
                filter: R::Creature.and(R::ControlledByOpponent),
            },
            definition: cursed_role(),
        },
        ..Default::default()
    }
}

// ── Black ─────────────────────────────────────────────────────────────────────

/// Dream Spoilers — {3}{B} 2/2 Faerie Warlock with flying. Whenever you cast a
/// spell during an opponent's turn, up to one target creature an opponent
/// controls gets -1/-1 until end of turn.
pub fn dream_spoilers() -> CardDefinition {
    CardDefinition {
        name: "Dream Spoilers",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You))),
            ),
            effect: Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Creature.and(R::ControlledByOpponent),
                effect: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Feed the Cauldron — {2}{B} Instant. Destroy target creature with mana value 3
/// or less. If it's your turn, create a Food token.
pub fn feed_the_cauldron() -> CardDefinition {
    CardDefinition {
        name: "Feed the Cauldron",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Creature.and(R::ManaValueAtMost(3))),
            },
            Effect::If {
                cond: Predicate::IsTurnOf(PlayerRef::You),
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: food_token(),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Experimental Confectioner — {2}{B} 2/3 Human Peasant. ETB create a Food.
/// Whenever you sacrifice a Food, create a 1/1 black Rat token that can't block.
pub fn experimental_confectioner() -> CardDefinition {
    CardDefinition {
        name: "Experimental Confectioner",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Peasant],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: food_token(),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasArtifactSubtype(crate::card::ArtifactSubtype::Food),
                    }),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: rat_token(),
                },
            },
        ],
        ..Default::default()
    }
}

// ── Red ───────────────────────────────────────────────────────────────────────

/// Embereth Veteran — {R} 2/1 Human Knight. {1}, Sacrifice this creature: Create
/// a Young Hero Role token attached to another target creature.
pub fn embereth_veteran() -> CardDefinition {
    CardDefinition {
        name: "Embereth Veteran",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::CreateTokenAttachedTo {
                target: target_filtered(R::Creature.and(R::OtherThanSource)),
                definition: young_hero_role(),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Green ─────────────────────────────────────────────────────────────────────

/// Bestial Bloodline — {1}{G} Aura. Enchant creature; it gets +2/+2. {4}{G}:
/// Return this card from your graveyard to your hand.
pub fn bestial_bloodline() -> CardDefinition {
    CardDefinition {
        name: "Bestial Bloodline",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g()]),
            from_graveyard: true,
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Elvish Archivist — {1}{G} 0/1 Elf Artificer. Once each turn, when one or more
/// artifacts you control enter, put two +1/+1 counters on it; once each turn,
/// when one or more enchantments you control enter, draw a card.
pub fn elvish_archivist() -> CardDefinition {
    CardDefinition {
        name: "Elvish Archivist",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Artificer],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec {
                    once_per_turn: true,
                    ..EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                        .with_filter(Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: R::Artifact,
                        })
                },
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
            },
            TriggeredAbility {
                event: EventSpec {
                    once_per_turn: true,
                    ..EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                        .with_filter(Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: R::Enchantment,
                        })
                },
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Curse of the Werefox — {2}{G} Sorcery. Create a Monster Role token attached
/// to target creature you control; when you do, that creature fights up to one
/// target creature you don't control.
pub fn curse_of_the_werefox() -> CardDefinition {
    CardDefinition {
        name: "Curse of the Werefox",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateTokenAttachedTo {
                target: target_filtered(R::Creature.and(R::ControlledByYou)),
                definition: monster_role(),
            },
            Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Creature.and(R::ControlledByOpponent),
                effect: Box::new(Effect::Fight {
                    attacker: Selector::Target(0),
                    defender: Selector::Target(1),
                }),
            },
        ]),
        ..Default::default()
    }
}

// ── Multicolor / Artifact ──────────────────────────────────────────────────────

/// Bespoke Battlegarb — {1}{R} Equipment. Equipped creature gets +2/+0.
/// Celebration — at combat on your turn, if two or more nonland permanents
/// entered under your control this turn, attach to up to one target creature
/// you control. Equip {2}.
pub fn bespoke_battlegarb() -> CardDefinition {
    CardDefinition {
        name: "Bespoke Battlegarb",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        equipped_bonus: Some(EquipBonus {
            power: 2,
            ..Default::default()
        }),
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            )
            .with_filter(Predicate::CelebrationActive {
                who: PlayerRef::You,
            }),
            effect: Effect::Attach {
                what: Selector::This,
                to: target_filtered(R::Creature.and(R::ControlledByYou)),
            },
        }],
        ..Default::default()
    }
}

/// Collector's Vault — {2} Artifact. {2}, {T}: Draw a card, then discard a card.
/// Create a Treasure token.
pub fn collectors_vault() -> CardDefinition {
    CardDefinition {
        name: "Collector's Vault",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: treasure_token(),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Eriette's Tempting Apple — {4} Legendary Artifact — Food. ETB gain control of
/// target creature until end of turn; untap it and it gains haste. {2}, {T},
/// Sacrifice: gain 3 life. {2}, {T}, Sacrifice: target opponent loses 3 life.
pub fn eriettes_tempting_apple() -> CardDefinition {
    CardDefinition {
        name: "Eriette's Tempting Apple",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Food],
            ..Default::default()
        },
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(R::Creature),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap {
                what: Selector::Target(0),
                up_to: None,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]))],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(3),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::LoseLife {
                    who: target_filtered(R::OpponentPlayer),
                    amount: Value::Const(3),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Tempest Hart // Scan the Clouds — {3}{G} 3/4 Elemental Elk with trample;
/// whenever you cast a spell with mana value 5+, put a +1/+1 counter on it.
/// Adventure {1}{U} Instant: draw two cards, then discard two cards.
pub fn tempest_hart() -> CardDefinition {
    CardDefinition {
        name: "Tempest Hart",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Elk],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        adventure: Some(Box::new(Adventure {
            name: "Scan the Clouds",
            cost: cost(&[generic(1), u()]),
            card_types: vec![CardType::Instant],
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(2),
                    random: false,
                },
            ]),
        })),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::ManaValueAtLeast(5))),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

// ── Additional wave-8 cards (tokens, Roles, dies-value, Adventures) ─────────────

/// 1/1 black Rat token with "This token can't block."
fn rat_token() -> crate::card::TokenDefinition {
    crate::card::TokenDefinition {
        name: "Rat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat],
            ..Default::default()
        },
        keywords: vec![Keyword::CantBlock],
        ..Default::default()
    }
}

/// 1/1 white Bird token with flying (Knight of Doves).
fn bird_token() -> crate::card::TokenDefinition {
    crate::card::TokenDefinition {
        name: "Bird".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// 1/1 blue Faerie token with flying that can block only fliers (Into the Fae Court).
fn faerie_token() -> crate::card::TokenDefinition {
    crate::card::TokenDefinition {
        name: "Faerie".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying, Keyword::CanBlockOnlyFlying],
        ..Default::default()
    }
}

/// Twisted Sewer-Witch — {3}{B}{B} 3/4 Human Warlock. ETB create a Rat, then for
/// each Rat you control create a Wicked Role token attached to it.
pub fn twisted_sewer_witch() -> CardDefinition {
    CardDefinition {
        name: "Twisted Sewer-Witch",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: rat_token(),
            },
            Effect::CreateTokenAttachedToEach {
                target: Selector::EachMatching {
                    zone: ZoneRef::Battlefield,
                    filter: R::ControlledByYou.and(R::HasCreatureType(CreatureType::Rat)),
                },
                definition: super::woe_roles::wicked_role(),
            },
        ]))],
        ..Default::default()
    }
}

/// Mintstrosity — {1}{B} 3/1 Horror. When it dies, create a Food token.
pub fn mintstrosity() -> CardDefinition {
    CardDefinition {
        name: "Mintstrosity",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Horror],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: food_token(),
            },
        }],
        ..Default::default()
    }
}

/// Protective Parents — {2}{W} 3/2 Human Peasant. When it dies, create a Young
/// Hero Role token attached to up to one target creature you control.
pub fn protective_parents() -> CardDefinition {
    CardDefinition {
        name: "Protective Parents",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Peasant],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateTokenAttachedTo {
                target: target_filtered(R::Creature.and(R::ControlledByYou)),
                definition: young_hero_role(),
            },
        }],
        ..Default::default()
    }
}

/// Merry Bards — {2}{R} 3/2 Human Bard. ETB you may pay {1}; if you do, create a
/// Young Hero Role token attached to target creature you control.
pub fn merry_bards() -> CardDefinition {
    CardDefinition {
        name: "Merry Bards",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Bard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::MayPay {
            description: "Pay {1} to create a Young Hero Role?".into(),
            mana_cost: cost(&[generic(1)]),
            body: Box::new(Effect::CreateTokenAttachedTo {
                target: target_filtered(R::Creature.and(R::ControlledByYou)),
                definition: young_hero_role(),
            }),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Monstrous Rage — {R} Instant. Target creature gets +2/+0 until end of turn.
/// Create a Monster Role token attached to it.
pub fn monstrous_rage() -> CardDefinition {
    CardDefinition {
        name: "Monstrous Rage",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::CreateTokenAttachedTo {
                target: Selector::Target(0),
                definition: monster_role(),
            },
        ]),
        ..Default::default()
    }
}

/// Leaping Ambush — {G} Instant. Target creature gets +1/+3 and gains reach until
/// end of turn. Untap it.
pub fn leaping_ambush() -> CardDefinition {
    CardDefinition {
        name: "Leaping Ambush",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(1),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Reach,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap {
                what: Selector::Target(0),
                up_to: None,
            },
        ]),
        ..Default::default()
    }
}

/// Plunge into Winter — {1}{W} Instant. Tap up to one target creature. Scry 1,
/// then draw a card.
pub fn plunge_into_winter() -> CardDefinition {
    CardDefinition {
        name: "Plunge into Winter",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::Tap {
                    what: Selector::Target(0),
                }),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::ONE,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Into the Fae Court — {3}{U}{U} Sorcery. Draw three cards. Create a 1/1 blue
/// Faerie token with flying that can block only creatures with flying.
pub fn into_the_fae_court() -> CardDefinition {
    CardDefinition {
        name: "Into the Fae Court",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(3),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: faerie_token(),
            },
        ]),
        ..Default::default()
    }
}

/// Knight of Doves — {2}{W} 1/3 Human Knight. Whenever an enchantment you control
/// is put into a graveyard from the battlefield, create a 1/1 white flying Bird.
pub fn knight_of_doves() -> CardDefinition {
    CardDefinition {
        name: "Knight of Doves",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Enchantment,
                },
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: bird_token(),
            },
        }],
        ..Default::default()
    }
}

/// Fae Flight — {1}{U} Aura with flash. ETB grants the enchanted creature
/// hexproof until end of turn; it gets +1/+0 and has flying.
pub fn fae_flight() -> CardDefinition {
    CardDefinition {
        name: "Fae Flight",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Flash],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: 1,
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: Selector::AttachedTo(Box::new(Selector::This)),
            keyword: Keyword::Hexproof,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Gingerbread Hunter // Puny Snack — {4}{G} 5/5 Giant; ETB create a Food.
/// Adventure {2}{B} Instant: target creature gets -2/-2 until end of turn.
pub fn gingerbread_hunter() -> CardDefinition {
    CardDefinition {
        name: "Gingerbread Hunter",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        adventure: Some(Box::new(Adventure {
            name: "Puny Snack",
            cost: cost(&[generic(2), b()]),
            card_types: vec![CardType::Instant],
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
        })),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: food_token(),
        })],
        ..Default::default()
    }
}
