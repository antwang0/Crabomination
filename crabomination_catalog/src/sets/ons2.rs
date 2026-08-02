//! Onslaught (ONS) wave 2 — the Morph commons, the tap-a-tribe activations,
//! the Avatar cycle and the cycling utility spells. Tests in
//! `classic_sets/ons`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, DynamicPt, EnchantmentSubtype,
    EquipBonus, Keyword, LandType, Predicate, SelectionRequirement as R, StaticAbility, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, LibraryPosition, ManaPayload, PlayerRef,
    Selector, StaticEffect, Value, ZoneDest,
    shortcut::{draw, etb, target_filtered},
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

/// "Whenever this creature deals combat damage to a player, [effect]."
fn on_combat_damage(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect,
    }
}

/// The Onslaught "search your library for a card with this name" ETB
/// (Avarax, Daru Cavalier, Embermage Goblin, Screaming Seahawk).
fn fetch_sibling() -> TriggeredAbility {
    etb(Effect::MayDo {
        description: "Search your library for another copy?".into(),
        body: Box::new(Effect::SearchSameNameAs {
            who: PlayerRef::You,
            subject: Selector::This,
            to: ZoneDest::Hand(PlayerRef::You),
        }),
    })
}

// ── Morph bodies ────────────────────────────────────────────────────────────

/// Battering Craghorn — first-striking Goat with Morph {1}{R}{R}.
pub fn battering_craghorn() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::FirstStrike,
            Keyword::Morph(cost(&[generic(1), r(), r()])),
        ],
        ..creature(
            "Battering Craghorn",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Goat, CreatureType::Beast],
            3,
            1,
        )
    }
}

/// Crude Rampart — a 4/5 Wall with Morph {4}{W}.
pub fn crude_rampart() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender, Keyword::Morph(cost(&[generic(4), w()]))],
        ..creature("Crude Rampart", cost(&[generic(3), w()]), vec![CreatureType::Wall], 4, 5)
    }
}

/// Spined Basher — a 3/1 Zombie Beast with Morph {2}{B}.
pub fn spined_basher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(2), b()]))],
        ..creature(
            "Spined Basher",
            cost(&[generic(2), b()]),
            vec![CreatureType::Zombie, CreatureType::Beast],
            3,
            1,
        )
    }
}

/// Spitting Gourna — reaching Beast with Morph {4}{G}.
pub fn spitting_gourna() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Morph(cost(&[generic(4), g()]))],
        ..creature(
            "Spitting Gourna",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Beast],
            3,
            4,
        )
    }
}

/// Krosan Colossus — a 9/9 with Morph {6}{G}{G}.
pub fn krosan_colossus() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(6), g(), g()]))],
        ..creature(
            "Krosan Colossus",
            cost(&[generic(6), g(), g(), g()]),
            vec![CreatureType::Beast],
            9,
            9,
        )
    }
}

/// Towering Baloth — a 7/6 with Morph {6}{G}.
pub fn towering_baloth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(6), g()]))],
        ..creature(
            "Towering Baloth",
            cost(&[generic(6), g(), g()]),
            vec![CreatureType::Beast],
            7,
            6,
        )
    }
}

/// Treespring Lorian — a 5/4 with Morph {5}{G}.
pub fn treespring_lorian() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(5), g()]))],
        ..creature("Treespring Lorian", cost(&[generic(5), g()]), vec![CreatureType::Beast], 5, 4)
    }
}

/// Fallen Cleric — protection from Clerics, Morph {4}{B}.
pub fn fallen_cleric() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::ProtectionFromCreatureType(CreatureType::Cleric),
            Keyword::Morph(cost(&[generic(4), b()])),
        ],
        ..creature(
            "Fallen Cleric",
            cost(&[generic(4), b()]),
            vec![CreatureType::Zombie, CreatureType::Cleric],
            4,
            2,
        )
    }
}

/// Foothill Guide — protection from Goblins, Morph {W}.
pub fn foothill_guide() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::ProtectionFromCreatureType(CreatureType::Goblin),
            Keyword::Morph(cost(&[w()])),
        ],
        ..creature(
            "Foothill Guide",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Riptide Biologist — protection from Beasts, Morph {2}{U}.
pub fn riptide_biologist() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::ProtectionFromCreatureType(CreatureType::Beast),
            Keyword::Morph(cost(&[generic(2), u()])),
        ],
        ..creature(
            "Riptide Biologist",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            2,
        )
    }
}

/// Ascending Aven — flier that can block only fliers; Morph {2}{U}.
pub fn ascending_aven() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Flying,
            Keyword::CanBlockOnlyFlying,
            Keyword::Morph(cost(&[generic(2), u()])),
        ],
        ..creature(
            "Ascending Aven",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            3,
            2,
        )
    }
}

/// Anurid Murkdiver — a swampwalking 4/3.
pub fn anurid_murkdiver() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Swamp)],
        ..creature(
            "Anurid Murkdiver",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Zombie, CreatureType::Frog, CreatureType::Beast],
            4,
            3,
        )
    }
}

/// Gluttonous Zombie — a 3/3 with fear.
pub fn gluttonous_zombie() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fear],
        ..creature(
            "Gluttonous Zombie",
            cost(&[generic(4), b()]),
            vec![CreatureType::Zombie],
            3,
            3,
        )
    }
}

/// Grinning Demon — a 6/6 that bleeds you 2 a turn; Morph {2}{B}{B}.
pub fn grinning_demon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(2), b(), b()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
        }],
        ..creature(
            "Grinning Demon",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Demon],
            6,
            6,
        )
    }
}

// ── Combat-damage Morph creatures ───────────────────────────────────────────

/// Headhunter — combat damage strips a card; Morph {B}.
pub fn headhunter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[b()]))],
        triggered_abilities: vec![on_combat_damage(Effect::Discard {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::ONE,
            random: false,
        })],
        ..creature(
            "Headhunter",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Silent Specter — combat damage strips two; Morph {3}{B}{B}.
pub fn silent_specter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Morph(cost(&[generic(3), b(), b()]))],
        triggered_abilities: vec![on_combat_damage(Effect::Discard {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Const(2),
            random: false,
        })],
        ..creature(
            "Silent Specter",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Specter],
            4,
            4,
        )
    }
}

/// Cabal Executioner — combat damage edicts the defender; Morph {3}{B}{B}.
pub fn cabal_executioner() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(3), b(), b()]))],
        triggered_abilities: vec![on_combat_damage(Effect::Sacrifice {
            who: Selector::Player(PlayerRef::Target(0)),
            count: Value::ONE,
            filter: R::Creature,
        })],
        ..creature(
            "Cabal Executioner",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Skirk Commando — combat damage pings one of their creatures; Morph {2}{R}.
pub fn skirk_commando() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(2), r()]))],
        triggered_abilities: vec![on_combat_damage(Effect::MayDo {
            description: "Deal 2 damage to a creature that player controls?".into(),
            body: Box::new(Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::ControlledByTriggerPlayer)),
                amount: Value::Const(2),
            }),
        })],
        ..creature("Skirk Commando", cost(&[generic(1), r(), r()]), vec![CreatureType::Goblin], 2, 1)
    }
}

/// Snapping Thragg — the Beast-sized Skirk Commando; Morph {4}{R}{R}.
pub fn snapping_thragg() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(4), r(), r()]))],
        triggered_abilities: vec![on_combat_damage(Effect::MayDo {
            description: "Deal 3 damage to a creature that player controls?".into(),
            body: Box::new(Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::ControlledByTriggerPlayer)),
                amount: Value::Const(3),
            }),
        })],
        ..creature("Snapping Thragg", cost(&[generic(4), r()]), vec![CreatureType::Beast], 3, 3)
    }
}

/// Hystrodon — trampling card draw on connect; Morph {1}{G}{G}.
pub fn hystrodon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Morph(cost(&[generic(1), g(), g()]))],
        triggered_abilities: vec![on_combat_damage(Effect::MayDo {
            description: "Draw a card?".into(),
            body: Box::new(draw(1)),
        })],
        ..creature("Hystrodon", cost(&[generic(4), g()]), vec![CreatureType::Beast], 3, 4)
    }
}

/// Dawning Purist — combat damage cracks an enchantment; Morph {1}{W}.
pub fn dawning_purist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(1), w()]))],
        triggered_abilities: vec![on_combat_damage(Effect::MayDo {
            description: "Destroy an enchantment that player controls?".into(),
            body: Box::new(Effect::Destroy {
                what: target_filtered(R::Enchantment.and(R::ControlledByTriggerPlayer)),
            }),
        })],
        ..creature(
            "Dawning Purist",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    }
}

// ── Avatar cycle (tribal count = P/T) ───────────────────────────────────────

fn avatar(
    name: &'static str,
    c: ManaCost,
    tribe: CreatureType,
    also_graveyards: bool,
) -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::CreaturesOfTypeOnBattlefield {
            creature_type: tribe,
            also_graveyards,
        }),
        ..creature(name, c, vec![tribe, CreatureType::Avatar], 0, 0)
    }
}

/// Doubtless One — */* by Clerics; drains you life on every hit.
pub fn doubtless_one() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamage, EventScope::SelfSource),
            effect: Effect::GainLife { who: Selector::You, amount: Value::TriggerEventAmount },
        }],
        ..avatar("Doubtless One", cost(&[generic(3), w()]), CreatureType::Cleric, false)
    }
}

/// Heedless One — */* by Elves, with trample.
pub fn heedless_one() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..avatar("Heedless One", cost(&[generic(3), g()]), CreatureType::Elf, false)
    }
}

/// Nameless One — */* by Wizards; Morph {2}{U}.
pub fn nameless_one() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(2), u()]))],
        ..avatar("Nameless One", cost(&[generic(3), u()]), CreatureType::Wizard, false)
    }
}

/// Reckless One — */* by Goblins, with haste.
pub fn reckless_one() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        ..avatar("Reckless One", cost(&[generic(3), r()]), CreatureType::Goblin, false)
    }
}

/// Soulless One — */* by Zombies in play *and* in every graveyard.
pub fn soulless_one() -> CardDefinition {
    avatar("Soulless One", cost(&[generic(3), b()]), CreatureType::Zombie, true)
}

// ── Tap-a-tribe activations ─────────────────────────────────────────────────

/// Count every creature of `tribe` on the battlefield, any controller.
fn tribe_count(tribe: CreatureType) -> Value {
    Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(R::Creature)),
        filter: R::HasCreatureType(tribe),
    }
}

/// Ancestor's Prophet — tap five Clerics for 10 life.
pub fn ancestors_prophet() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_permanents_cost: Some((
                R::Creature.and(R::HasCreatureType(CreatureType::Cleric)),
                5,
            )),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(10) },
            ..Default::default()
        }],
        ..creature(
            "Ancestor's Prophet",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            5,
        )
    }
}

/// Aphetto Grifter — tap two Wizards to tap a permanent.
pub fn aphetto_grifter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_permanents_cost: Some((
                R::Creature.and(R::HasCreatureType(CreatureType::Wizard)),
                2,
            )),
            effect: Effect::Tap { what: target_filtered(R::Permanent) },
            ..Default::default()
        }],
        ..creature(
            "Aphetto Grifter",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Catapult Squad — tap two Soldiers to shoot a combatant.
pub fn catapult_squad() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_permanents_cost: Some((
                R::Creature.and(R::HasCreatureType(CreatureType::Soldier)),
                2,
            )),
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..creature(
            "Catapult Squad",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            1,
        )
    }
}

/// Birchlore Rangers — tap two Elves for a mana of any color; Morph {G}.
pub fn birchlore_rangers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[g()]))],
        activated_abilities: vec![ActivatedAbility {
            tap_permanents_cost: Some((R::Creature.and(R::HasCreatureType(CreatureType::Elf)), 2)),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::ONE),
            },
            ..Default::default()
        }],
        ..creature(
            "Birchlore Rangers",
            cost(&[g()]),
            vec![CreatureType::Elf, CreatureType::Druid, CreatureType::Ranger],
            1,
            1,
        )
    }
}

/// Gravespawn Sovereign — tap five Zombies to reanimate from any graveyard.
pub fn gravespawn_sovereign() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_permanents_cost: Some((
                R::Creature.and(R::HasCreatureType(CreatureType::Zombie)),
                5,
            )),
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature(
            "Gravespawn Sovereign",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Zombie],
            3,
            3,
        )
    }
}

/// Gangrenous Goliath — tap three Clerics to buy it back from the graveyard.
pub fn gangrenous_goliath() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            from_graveyard: true,
            tap_permanents_cost: Some((
                R::Creature.and(R::HasCreatureType(CreatureType::Cleric)),
                3,
            )),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..creature(
            "Gangrenous Goliath",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Zombie, CreatureType::Giant],
            4,
            4,
        )
    }
}

/// Skirk Fire Marshal — protection from red, and a five-Goblin Fireball.
pub fn skirk_fire_marshal() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Red)],
        activated_abilities: vec![ActivatedAbility {
            tap_permanents_cost: Some((
                R::Creature.and(R::HasCreatureType(CreatureType::Goblin)),
                5,
            )),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::EachPermanent(R::Creature),
                    amount: Value::Const(10),
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(10),
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Skirk Fire Marshal",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Goblin],
            2,
            2,
        )
    }
}

/// Spurred Wolverine — tap two Beasts to grant first strike.
pub fn spurred_wolverine() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_permanents_cost: Some((R::Creature.and(R::HasCreatureType(CreatureType::Beast)), 2)),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Spurred Wolverine",
            cost(&[generic(4), r()]),
            vec![CreatureType::Wolverine, CreatureType::Beast],
            3,
            2,
        )
    }
}

// ── Tribal-count spells ─────────────────────────────────────────────────────

/// Wirewood Pride — +X/+X for every Elf in play.
pub fn wirewood_pride() -> CardDefinition {
    CardDefinition {
        name: "Wirewood Pride",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: tribe_count(CreatureType::Elf),
            toughness: tribe_count(CreatureType::Elf),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Feeding Frenzy — -X/-X for every Zombie in play.
pub fn feeding_frenzy() -> CardDefinition {
    CardDefinition {
        name: "Feeding Frenzy",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Diff(Box::new(Value::ZERO), Box::new(tribe_count(CreatureType::Zombie))),
            toughness: Value::Diff(
                Box::new(Value::ZERO),
                Box::new(tribe_count(CreatureType::Zombie)),
            ),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Profane Prayers — a Cleric-sized drain.
pub fn profane_prayers() -> CardDefinition {
    CardDefinition {
        name: "Profane Prayers",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Any),
                amount: tribe_count(CreatureType::Cleric),
            },
            Effect::GainLife { who: Selector::You, amount: tribe_count(CreatureType::Cleric) },
        ]),
        ..Default::default()
    }
}

/// Thunder of Hooves — a Beast-sized sweeper that spares fliers.
pub fn thunder_of_hooves() -> CardDefinition {
    CardDefinition {
        name: "Thunder of Hooves",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    R::Creature.and(R::HasKeyword(Keyword::Flying).negate()),
                ),
                amount: tribe_count(CreatureType::Beast),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachPlayer),
                amount: tribe_count(CreatureType::Beast),
            },
        ]),
        ..Default::default()
    }
}

// ── Utility creatures ───────────────────────────────────────────────────────

/// Elvish Pioneer — ETB drops a basic land onto the battlefield tapped.
pub fn elvish_pioneer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::PutFromHandOntoBattlefield {
            who: PlayerRef::You,
            filter: R::IsBasicLand,
            count: Value::ONE,
            tapped: true,
            haste: false,
            sacrifice_eot: false,
            return_eot: false,
        })],
        ..creature(
            "Elvish Pioneer",
            cost(&[g()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            1,
            1,
        )
    }
}

/// Doomed Necromancer — sacrifice it to reanimate.
pub fn doomed_necromancer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature(
            "Doomed Necromancer",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Cleric, CreatureType::Mercenary],
            2,
            2,
        )
    }
}

/// Rotlung Reanimator — every dying Cleric leaves a Zombie behind.
pub fn rotlung_reanimator() -> CardDefinition {
    let zombie = TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Cleric),
                },
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: zombie,
            },
        }],
        ..creature(
            "Rotlung Reanimator",
            cost(&[generic(2), b()]),
            vec![CreatureType::Zombie, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Dive Bomber — sacrifice it to shoot a combatant.
pub fn dive_bomber() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..creature(
            "Dive Bomber",
            cost(&[generic(3), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Grassland Crusader — pumps an Elf or Soldier.
pub fn grassland_crusader() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(
                    R::HasCreatureType(CreatureType::Elf).or(R::HasCreatureType(CreatureType::Soldier)),
                )),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Grassland Crusader",
            cost(&[generic(5), w()]),
            vec![CreatureType::Human, CreatureType::Cleric, CreatureType::Soldier],
            2,
            4,
        )
    }
}

/// Elvish Pathcutter — forestwalk for an Elf.
pub fn elvish_pathcutter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Elf))),
                keyword: Keyword::Landwalk(LandType::Forest),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Elvish Pathcutter",
            cost(&[generic(3), g()]),
            vec![CreatureType::Elf, CreatureType::Scout],
            1,
            2,
        )
    }
}

/// Snarling Undorak — pumps a Beast; Morph {1}{G}{G}.
pub fn snarling_undorak() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(1), g(), g()]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Beast))),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Snarling Undorak",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Beast],
            3,
            3,
        )
    }
}

/// Aphetto Vulture — dies to stack a Zombie back on top.
pub fn aphetto_vulture() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Put a Zombie card from your graveyard on top of your library?".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(
                        R::Creature
                            .and(R::HasCreatureType(CreatureType::Zombie))
                            .and(R::InYourGraveyard),
                    ),
                    to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
                }),
            },
        }],
        ..creature(
            "Aphetto Vulture",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Zombie, CreatureType::Bird],
            3,
            2,
        )
    }
}

/// Daru Cavalier — first striker that fetches its twin.
pub fn daru_cavalier() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![fetch_sibling()],
        ..creature(
            "Daru Cavalier",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Avarax — hasty Beast that fetches its twin and pumps itself.
pub fn avarax() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![fetch_sibling()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Avarax", cost(&[generic(3), r(), r()]), vec![CreatureType::Beast], 3, 3)
    }
}

/// Embermage Goblin — fetches its twin, then pings every turn.
pub fn embermage_goblin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![fetch_sibling()],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage { to: target_filtered(R::Any), amount: Value::ONE },
            ..Default::default()
        }],
        ..creature(
            "Embermage Goblin",
            cost(&[generic(3), r()]),
            vec![CreatureType::Goblin, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Screaming Seahawk — flier that fetches its twin.
pub fn screaming_seahawk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![fetch_sibling()],
        ..creature("Screaming Seahawk", cost(&[generic(4), u()]), vec![CreatureType::Bird], 2, 2)
    }
}

/// Sage Aven — flier that stacks the top four.
pub fn sage_aven() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::RearrangeTop {
            who: PlayerRef::You,
            amount: Value::Const(4),
        })],
        ..creature(
            "Sage Aven",
            cost(&[generic(3), u()]),
            vec![CreatureType::Bird, CreatureType::Wizard],
            1,
            3,
        )
    }
}

/// Aven Fateshaper — Sage Aven's rare cousin, with a repeatable stack.
pub fn aven_fateshaper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::RearrangeTop {
            who: PlayerRef::You,
            amount: Value::Const(4),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), u()]),
            effect: Effect::RearrangeTop { who: PlayerRef::You, amount: Value::Const(4) },
            ..Default::default()
        }],
        ..creature(
            "Aven Fateshaper",
            cost(&[generic(6), u()]),
            vec![CreatureType::Bird, CreatureType::Wizard],
            4,
            5,
        )
    }
}

/// Stag Beetle — enters sized by the rest of the board.
pub fn stag_beetle() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((
            crate::card::CounterType::PlusOnePlusOne,
            Value::CountMatching {
                sel: Box::new(Selector::EachPermanent(R::Creature.and(R::OtherThanSource))),
                filter: R::Any,
            },
        )),
        ..creature(
            "Stag Beetle",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Insect],
            0,
            0,
        )
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Pinpoint Avalanche — 4 unpreventable damage to a creature.
pub fn pinpoint_avalanche() -> CardDefinition {
    CardDefinition {
        name: "Pinpoint Avalanche",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DamageCantBePreventedThisTurn,
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(4) },
        ]),
        ..Default::default()
    }
}

/// Discombobulate — counter, then stack the top four.
pub fn discombobulate() -> CardDefinition {
    CardDefinition {
        name: "Discombobulate",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::RearrangeTop { who: PlayerRef::You, amount: Value::Const(4) },
        ]),
        ..Default::default()
    }
}

/// Ixidor's Will — a Wizard-scaled Mana Leak.
pub fn ixidors_will() -> CardDefinition {
    CardDefinition {
        name: "Ixidor's Will",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(R::IsSpellOnStack),
            mana_cost: ManaCost::default(),
            exile: false,
            extra_generic: Some(Value::Times(
                Box::new(tribe_count(CreatureType::Wizard)),
                Box::new(Value::Const(2)),
            )),
        },
        ..Default::default()
    }
}

/// Unified Strike — exile an attacker no bigger than your Soldier count.
pub fn unified_strike() -> CardDefinition {
    CardDefinition {
        name: "Unified Strike",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::ValueAtMost(
                Value::PowerOf(Box::new(Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::IsAttacking),
                })),
                tribe_count(CreatureType::Soldier),
            ),
            then: Box::new(Effect::Exile {
                what: target_filtered(R::Creature.and(R::IsAttacking)),
            }),
            else_: Box::new(Effect::Noop),
        },
        ..Default::default()
    }
}

/// Blackmail — they show three, you pick the discard.
pub fn blackmail() -> CardDefinition {
    CardDefinition {
        name: "Blackmail",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DiscardChosenFromRevealed {
            from: Selector::Player(PlayerRef::Target(0)),
            reveal: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Dragon Roost — a Dragon every turn, if you can pay for it.
pub fn dragon_roost() -> CardDefinition {
    let dragon = TokenDefinition {
        name: "Dragon".into(),
        power: 5,
        toughness: 5,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Dragon Roost",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), r(), r()]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: dragon,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Starstorm — an instant-speed X sweeper with cycling.
pub fn starstorm() -> CardDefinition {
    CardDefinition {
        name: "Starstorm",
        cost: cost(&[x(), r(), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Cycling(cost(&[generic(3)]))],
        effect: Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature),
            amount: Value::XFromCost,
        },
        ..Default::default()
    }
}

// ── Lands ───────────────────────────────────────────────────────────────────

/// Unholy Grotto — recycle Zombies from the graveyard.
pub fn unholy_grotto() -> CardDefinition {
    CardDefinition {
        name: "Unholy Grotto",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                tap_cost: true,
                effect: Effect::Move {
                    what: target_filtered(
                        R::Creature
                            .and(R::HasCreatureType(CreatureType::Zombie))
                            .and(R::InYourGraveyard),
                    ),
                    to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Seaside Haven — cash in a Bird for a card.
pub fn seaside_haven() -> CardDefinition {
    CardDefinition {
        name: "Seaside Haven",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[w(), u()]),
                tap_cost: true,
                sac_other_filter: Some((
                    R::Creature.and(R::HasCreatureType(CreatureType::Bird)),
                    1,
                )),
                effect: draw(1),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Contested Cliffs — turn a Beast loose on their board.
pub fn contested_cliffs() -> CardDefinition {
    CardDefinition {
        name: "Contested Cliffs",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[r(), g()]),
                tap_cost: true,
                effect: Effect::Fight {
                    attacker: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Creature
                            .and(R::HasCreatureType(CreatureType::Beast))
                            .and(R::ControlledByYou),
                    },
                    defender: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.and(R::ControlledByOpponent),
                    },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Grand Coliseum — the painful five-color land.
pub fn grand_coliseum() -> CardDefinition {
    CardDefinition {
        name: "Grand Coliseum",
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::AnyOneColor(Value::ONE),
                    },
                    Effect::DealDamage { to: Selector::You, amount: Value::ONE },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Starlit Sanctum — trade Clerics for life or drain.
pub fn starlit_sanctum() -> CardDefinition {
    let cleric = || R::Creature.and(R::HasCreatureType(CreatureType::Cleric));
    CardDefinition {
        name: "Starlit Sanctum",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                tap_cost: true,
                sac_other_filter: Some((cleric(), 1)),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::SacrificedToughness,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                tap_cost: true,
                sac_other_filter: Some((cleric(), 1)),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::SacrificedPower,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Jareth, Leonine Titan — a blocker that grows and dodges colors.
pub fn jareth_leonine_titan() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(7),
                toughness: Value::Const(7),
                duration: Duration::EndOfTurn,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::GrantProtectionFromChosenColor {
                what: Selector::This,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Jareth, Leonine Titan",
            cost(&[generic(3), w(), w(), w()]),
            vec![CreatureType::Cat, CreatureType::Giant],
            4,
            7,
        )
    }
}

// ── The Crown cycle (sacrifice for a tribe-wide pump) ───────────────────────

/// A Crown Aura: a static bonus on the host, plus a sacrifice ability that
/// spreads the same bonus to every creature sharing a type with it.
fn crown(
    name: &'static str,
    c: ManaCost,
    power: i32,
    toughness: i32,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    let shared = || R::Creature.and(R::SharesCreatureTypeWithAttachedHost);
    let mut spread = vec![Effect::PumpPT {
        what: Selector::EachPermanent(shared()),
        power: Value::Const(power),
        toughness: Value::Const(toughness),
        duration: Duration::EndOfTurn,
    }];
    spread.extend(keywords.iter().map(|kw| Effect::GrantKeyword {
        what: Selector::EachPermanent(shared()),
        keyword: kw.clone(),
        duration: Duration::EndOfTurn,
    }));
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power,
            toughness,
            keywords: keywords.clone(),
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Seq(spread),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Crown of Ascension — flying, then flying for the whole tribe.
pub fn crown_of_ascension() -> CardDefinition {
    crown("Crown of Ascension", cost(&[generic(1), u()]), 0, 0, vec![Keyword::Flying])
}

/// Crown of Awe — protection from black and from red.
pub fn crown_of_awe() -> CardDefinition {
    crown(
        "Crown of Awe",
        cost(&[generic(1), w()]),
        0,
        0,
        vec![Keyword::Protection(Color::Black), Keyword::Protection(Color::Red)],
    )
}

/// Crown of Fury — +1/+0 and first strike.
pub fn crown_of_fury() -> CardDefinition {
    crown("Crown of Fury", cost(&[generic(1), r()]), 1, 0, vec![Keyword::FirstStrike])
}

/// Crown of Suspicion — +2/-1.
pub fn crown_of_suspicion() -> CardDefinition {
    crown("Crown of Suspicion", cost(&[generic(1), b()]), 2, -1, vec![])
}

/// Crown of Vigor — +1/+1.
pub fn crown_of_vigor() -> CardDefinition {
    crown("Crown of Vigor", cost(&[generic(1), g()]), 1, 1, vec![])
}

// ── The Courier cycle (stay tapped to lend +2/+2 and a keyword) ─────────────

fn courier(
    name: &'static str,
    c: ManaCost,
    self_types: Vec<CreatureType>,
    activation: ManaCost,
    tribe: CreatureType,
    keyword: Keyword,
) -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MayChooseNotToUntap],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: activation,
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::HasCreatureType(tribe))),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::WhileSourceTapped,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword,
                    duration: Duration::WhileSourceTapped,
                },
            ]),
            ..Default::default()
        }],
        ..creature(name, c, self_types, 2, 1)
    }
}

/// Everglove Courier — lends an Elf +2/+2 and trample.
pub fn everglove_courier() -> CardDefinition {
    courier(
        "Everglove Courier",
        cost(&[generic(2), g()]),
        vec![CreatureType::Elf],
        cost(&[generic(2), g()]),
        CreatureType::Elf,
        Keyword::Trample,
    )
}

/// Flamestick Courier — lends a Goblin +2/+2 and haste.
pub fn flamestick_courier() -> CardDefinition {
    courier(
        "Flamestick Courier",
        cost(&[generic(2), r()]),
        vec![CreatureType::Goblin],
        cost(&[generic(2), r()]),
        CreatureType::Goblin,
        Keyword::Haste,
    )
}

/// Frightshroud Courier — lends a Zombie +2/+2 and fear.
pub fn frightshroud_courier() -> CardDefinition {
    courier(
        "Frightshroud Courier",
        cost(&[generic(2), b()]),
        vec![CreatureType::Zombie],
        cost(&[generic(2), b()]),
        CreatureType::Zombie,
        Keyword::Fear,
    )
}

/// Ghosthelm Courier — lends a Wizard +2/+2 and shroud.
pub fn ghosthelm_courier() -> CardDefinition {
    courier(
        "Ghosthelm Courier",
        cost(&[generic(2), u()]),
        vec![CreatureType::Human, CreatureType::Wizard],
        cost(&[generic(2), u()]),
        CreatureType::Wizard,
        Keyword::Shroud,
    )
}

/// Pearlspear Courier — lends a Soldier +2/+2 and vigilance.
pub fn pearlspear_courier() -> CardDefinition {
    courier(
        "Pearlspear Courier",
        cost(&[generic(2), w()]),
        vec![CreatureType::Human, CreatureType::Soldier],
        cost(&[generic(2), w()]),
        CreatureType::Soldier,
        Keyword::Vigilance,
    )
}

// ── The Mistform shapeshifters ──────────────────────────────────────────────

/// `{1}: This creature becomes the creature type of your choice until end of
/// turn.`
fn mistform_ability(c: ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: c,
        effect: Effect::BecomeChosenCreatureType {
            what: Selector::This,
            duration: Duration::EndOfTurn,
            excluded: vec![],
        },
        ..Default::default()
    }
}

/// Mistform Dreamer — a 2/1 flier that shifts types.
pub fn mistform_dreamer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![mistform_ability(cost(&[generic(1)]))],
        ..creature("Mistform Dreamer", cost(&[generic(2), u()]), vec![CreatureType::Illusion], 2, 1)
    }
}

/// Mistform Shrieker — a 3/3 flier that shifts types; Morph {3}{U}{U}.
pub fn mistform_shrieker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Morph(cost(&[generic(3), u(), u()]))],
        activated_abilities: vec![mistform_ability(cost(&[generic(1)]))],
        ..creature(
            "Mistform Shrieker",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Illusion],
            3,
            3,
        )
    }
}

/// Mistform Skyreaver — a 6/6 flier that shifts types.
pub fn mistform_skyreaver() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![mistform_ability(cost(&[generic(1)]))],
        ..creature(
            "Mistform Skyreaver",
            cost(&[generic(5), u(), u()]),
            vec![CreatureType::Illusion],
            6,
            6,
        )
    }
}

/// Mistform Stalker — shifts types, and can suit up for the air.
pub fn mistform_stalker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            mistform_ability(cost(&[generic(1)])),
            ActivatedAbility {
                mana_cost: cost(&[generic(2), u(), u()]),
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::This,
                        power: Value::Const(2),
                        toughness: Value::Const(2),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::This,
                        keyword: Keyword::Flying,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..creature("Mistform Stalker", cost(&[generic(1), u()]), vec![CreatureType::Illusion], 1, 1)
    }
}

/// Mistform Wall — a 1/4 that has defender only while it's still a Wall.
pub fn mistform_wall() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature has defender as long as it's a Wall.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::HasCreatureType(CreatureType::Wall),
                },
                inner: Box::new(StaticEffect::GrantKeyword {
                    applies_to: Selector::This,
                    keyword: Keyword::Defender,
                }),
            },
        }],
        activated_abilities: vec![mistform_ability(cost(&[generic(1)]))],
        ..creature(
            "Mistform Wall",
            cost(&[generic(2), u()]),
            vec![CreatureType::Illusion, CreatureType::Wall],
            1,
            4,
        )
    }
}

/// Mistform Mutant — retypes any creature, not just itself.
pub fn mistform_mutant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::BecomeChosenCreatureType {
                what: target_filtered(R::Creature),
                duration: Duration::EndOfTurn,
                excluded: vec![CreatureType::Wall],
            },
            ..Default::default()
        }],
        ..creature(
            "Mistform Mutant",
            cost(&[generic(4), u(), u()]),
            vec![CreatureType::Illusion, CreatureType::Mutant],
            3,
            4,
        )
    }
}

/// Mistform Mask — the Aura that retypes its host.
pub fn mistform_mask() -> CardDefinition {
    CardDefinition {
        name: "Mistform Mask",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::BecomeChosenCreatureType {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                duration: Duration::EndOfTurn,
                excluded: vec![],
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Imagecrafter — retypes a creature every turn for free.
pub fn imagecrafter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::BecomeChosenCreatureType {
                what: target_filtered(R::Creature),
                duration: Duration::EndOfTurn,
                excluded: vec![CreatureType::Wall],
            },
            ..Default::default()
        }],
        ..creature(
            "Imagecrafter",
            cost(&[u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Standardize — every creature becomes one chosen type.
pub fn standardize() -> CardDefinition {
    CardDefinition {
        name: "Standardize",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::BecomeChosenCreatureType {
            what: Selector::EachPermanent(R::Creature),
            duration: Duration::EndOfTurn,
            excluded: vec![CreatureType::Wall],
        },
        ..Default::default()
    }
}

// ── Cycling spells with "when you cycle this card" riders ───────────────────

/// `When you cycle this card, [effect].`
fn on_cycle(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource),
        effect: Effect::MayDo { description: "Use the cycling trigger?".into(), body: Box::new(effect) },
    }
}

/// Solar Blast — 3 damage, or 1 on the cycle.
pub fn solar_blast() -> CardDefinition {
    CardDefinition {
        name: "Solar Blast",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Cycling(cost(&[generic(1), r(), r()]))],
        effect: Effect::DealDamage { to: target_filtered(R::Any), amount: Value::Const(3) },
        triggered_abilities: vec![on_cycle(Effect::DealDamage {
            to: target_filtered(R::Any),
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Death Pulse — -4/-4, or -1/-1 on the cycle.
pub fn death_pulse() -> CardDefinition {
    CardDefinition {
        name: "Death Pulse",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Cycling(cost(&[generic(1), b(), b()]))],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(-4),
            toughness: Value::Const(-4),
            duration: Duration::EndOfTurn,
        },
        triggered_abilities: vec![on_cycle(Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Primal Boost — +4/+4, or +1/+1 on the cycle.
pub fn primal_boost() -> CardDefinition {
    CardDefinition {
        name: "Primal Boost",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Cycling(cost(&[generic(2), g()]))],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(4),
            toughness: Value::Const(4),
            duration: Duration::EndOfTurn,
        },
        triggered_abilities: vec![on_cycle(Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Sunfire Balm — prevent 4, or 1 on the cycle.
pub fn sunfire_balm() -> CardDefinition {
    CardDefinition {
        name: "Sunfire Balm",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Cycling(cost(&[generic(1), w()]))],
        effect: Effect::PreventNextDamage {
            target: target_filtered(R::Any),
            amount: Value::Const(4),
        },
        triggered_abilities: vec![on_cycle(Effect::PreventNextDamage {
            target: target_filtered(R::Any),
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Dirge of Dread — fear for everyone, or for one on the cycle.
pub fn dirge_of_dread() -> CardDefinition {
    CardDefinition {
        name: "Dirge of Dread",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Cycling(cost(&[generic(1), b()]))],
        effect: Effect::GrantKeyword {
            what: Selector::EachPermanent(R::Creature),
            keyword: Keyword::Fear,
            duration: Duration::EndOfTurn,
        },
        triggered_abilities: vec![on_cycle(Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::Fear,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Choking Tethers — tap up to four, or one on the cycle.
pub fn choking_tethers() -> CardDefinition {
    CardDefinition {
        name: "Choking Tethers",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Cycling(cost(&[generic(1), u()]))],
        effect: Effect::ApplyToTargets {
            max_targets: 4,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::Tap { what: Selector::Target(0) }),
        },
        triggered_abilities: vec![on_cycle(Effect::Tap { what: target_filtered(R::Creature) })],
        ..Default::default()
    }
}

/// Complicate — a {3} tax, or a {1} tax on the cycle.
pub fn complicate() -> CardDefinition {
    let tax = |n| Effect::CounterUnlessPaid {
        what: target_filtered(R::IsSpellOnStack),
        mana_cost: cost(&[generic(n)]),
        exile: false,
        extra_generic: None,
    };
    CardDefinition {
        name: "Complicate",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Cycling(cost(&[generic(2), u()]))],
        effect: tax(3),
        triggered_abilities: vec![on_cycle(tax(1))],
        ..Default::default()
    }
}

/// Slice and Dice — 4 to each creature, or 1 on the cycle.
pub fn slice_and_dice() -> CardDefinition {
    CardDefinition {
        name: "Slice and Dice",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Cycling(cost(&[generic(2), r()]))],
        effect: Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature),
            amount: Value::Const(4),
        },
        triggered_abilities: vec![on_cycle(Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature),
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

// ── Plain cycling spells ────────────────────────────────────────────────────

/// Akroma's Blessing — team-wide protection from a chosen color.
pub fn akromas_blessing() -> CardDefinition {
    CardDefinition {
        name: "Akroma's Blessing",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Cycling(cost(&[w()]))],
        effect: Effect::GrantProtectionFromChosenColor {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Aura Extraction — stack an enchantment back on its owner's library.
pub fn aura_extraction() -> CardDefinition {
    CardDefinition {
        name: "Aura Extraction",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        effect: Effect::Move {
            what: target_filtered(R::Enchantment),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: LibraryPosition::Top,
            },
        },
        ..Default::default()
    }
}

/// Fade from Memory — cheap graveyard hate that cycles for {B}.
pub fn fade_from_memory() -> CardDefinition {
    CardDefinition {
        name: "Fade from Memory",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Cycling(cost(&[b()]))],
        effect: Effect::Exile { what: target_filtered(R::InGraveyard) },
        ..Default::default()
    }
}

/// Mage's Guile — shroud for a turn, or a cantrip.
pub fn mages_guile() -> CardDefinition {
    CardDefinition {
        name: "Mage's Guile",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Cycling(cost(&[u()]))],
        effect: Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::Shroud,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Essence Fracture — bounce two, or cycle it away.
pub fn essence_fracture() -> CardDefinition {
    CardDefinition {
        name: "Essence Fracture",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Cycling(cost(&[generic(2), u()]))],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 2,
            filter: R::Creature,
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            }),
        },
        ..Default::default()
    }
}

/// Improvised Armor — a +2/+5 Aura that cycles when you don't want it.
pub fn improvised_armor() -> CardDefinition {
    CardDefinition {
        name: "Improvised Armor",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Cycling(cost(&[generic(3)]))],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 5, ..Default::default() }),
        ..Default::default()
    }
}

/// Slipstream Eel — a 6/6 that needs an Island across the table.
pub fn slipstream_eel() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::CanAttackOnlyIfDefenderControls(Box::new(R::Land.and(R::HasLandType(
                LandType::Island,
            )))),
            Keyword::Cycling(cost(&[generic(1), u()])),
        ],
        ..creature(
            "Slipstream Eel",
            cost(&[generic(5), u(), u()]),
            vec![CreatureType::Fish, CreatureType::Beast],
            6,
            6,
        )
    }
}

/// Undead Gladiator — a recursive Zombie that also cycles.
pub fn undead_gladiator() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(1), b()]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            from_graveyard: true,
            discard_cost: Some((R::Any, 1)),
            condition: Some(Predicate::CurrentStepIs(crate::game::TurnStep::Upkeep)),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..creature(
            "Undead Gladiator",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Zombie, CreatureType::Barbarian],
            3,
            1,
        )
    }
}

// ── The Gustcloak cycle (CR 506.4 — untap and leave combat) ─────────────────

/// "Whenever this creature becomes blocked, you may untap it and remove it
/// from combat."
fn gustcloak_dodge(what: Selector) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
        effect: Effect::MayDo {
            description: "Untap and remove it from combat?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Untap { what: what.clone(), up_to: None },
                Effect::RemoveFromCombat { what },
            ])),
        },
    }
}

/// Gustcloak Harrier — a 2/2 flier that slips its blockers.
pub fn gustcloak_harrier() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![gustcloak_dodge(Selector::This)],
        ..creature(
            "Gustcloak Harrier",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Gustcloak Sentinel — the ground-bound 3/3.
pub fn gustcloak_sentinel() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![gustcloak_dodge(Selector::This)],
        ..creature(
            "Gustcloak Sentinel",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Gustcloak Skirmisher — a 2/3 flier that slips its blockers.
pub fn gustcloak_skirmisher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![gustcloak_dodge(Selector::This)],
        ..creature(
            "Gustcloak Skirmisher",
            cost(&[generic(3), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            2,
            3,
        )
    }
}

/// Gustcloak Savior — extends the dodge to your whole team.
pub fn gustcloak_savior() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Untap that creature and remove it from combat?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Untap { what: Selector::TriggerSource, up_to: None },
                    Effect::RemoveFromCombat { what: Selector::TriggerSource },
                ])),
            },
        }],
        ..creature(
            "Gustcloak Savior",
            cost(&[generic(4), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            3,
            4,
        )
    }
}

/// Leery Fogbeast — blocking it calls off the whole combat.
pub fn leery_fogbeast() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::PreventAllCombatDamageThisTurn,
        }],
        ..creature("Leery Fogbeast", cost(&[generic(2), g()]), vec![CreatureType::Beast], 4, 2)
    }
}

/// Shaleskin Bruiser — grows for each other attacking Beast.
pub fn shaleskin_bruiser() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Times(
                    Box::new(Value::CountMatching {
                        sel: Box::new(Selector::EachPermanent(
                            R::Creature.and(R::IsAttacking).and(R::OtherThanSource),
                        )),
                        filter: R::HasCreatureType(CreatureType::Beast),
                    }),
                    Box::new(Value::Const(3)),
                ),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Shaleskin Bruiser", cost(&[generic(6), r()]), vec![CreatureType::Beast], 4, 4)
    }
}

/// Ebonblade Reaper — halves your life to attack, halves theirs on connect.
pub fn ebonblade_reaper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(3), b(), b()]))],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::LoseLife {
                    who: Selector::You,
                    amount: Value::HalvedRoundUp(Box::new(Value::LifeOf(PlayerRef::You))),
                },
            },
            on_combat_damage(Effect::LoseLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::HalvedRoundUp(Box::new(Value::LifeOf(PlayerRef::Target(0)))),
            }),
        ],
        ..creature(
            "Ebonblade Reaper",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Haunted Cadaver — cash it in on connect to strip three cards.
pub fn haunted_cadaver() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(1), b()]))],
        triggered_abilities: vec![on_combat_damage(Effect::MayDo {
            description: "Sacrifice this creature to strip three cards?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::SacrificePermanent { what: Selector::This },
                Effect::Discard {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(3),
                    random: false,
                },
            ])),
        })],
        ..creature("Haunted Cadaver", cost(&[generic(3), b()]), vec![CreatureType::Zombie], 2, 2)
    }
}

/// Nosy Goblin — sacrifice it to blow up a face-down creature.
pub fn nosy_goblin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Creature.and(R::FaceDown)) },
            ..Default::default()
        }],
        ..creature("Nosy Goblin", cost(&[generic(2), r()]), vec![CreatureType::Goblin], 2, 1)
    }
}

/// Break Open — flip an opponent's face-down creature up.
pub fn break_open() -> CardDefinition {
    CardDefinition {
        name: "Break Open",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::TurnFaceUpFree {
            what: target_filtered(R::Creature.and(R::FaceDown).and(R::ControlledByOpponent)),
        },
        ..Default::default()
    }
}

/// Ixidor, Reality Sculptor — face-down creatures get +1/+1, and he unmasks
/// them at will.
pub fn ixidor_reality_sculptor() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Face-down creatures get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::FaceDown)),
                power: 1,
                toughness: 1,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::TurnFaceUpFree {
                what: target_filtered(R::Creature.and(R::FaceDown)),
            },
            ..Default::default()
        }],
        ..creature(
            "Ixidor, Reality Sculptor",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            4,
        )
    }
}

/// Dream Chisel — face-down creature spells cost {1} less.
pub fn dream_chisel() -> CardDefinition {
    CardDefinition {
        name: "Dream Chisel",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Face-down creature spells you cast cost {1} less to cast.",
            effect: StaticEffect::FaceDownSpellsCostLess { amount: 1 },
        }],
        ..Default::default()
    }
}

// ── Wave 5 ──────────────────────────────────────────────────────────────────

/// Aether Charge — every Beast you land can bolt an opponent for 4.
pub fn aether_charge() -> CardDefinition {
    CardDefinition {
        name: "Aether Charge",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Beast),
                }),
            effect: Effect::MayDo {
                description: "Deal 4 damage to an opponent?".into(),
                body: Box::new(Effect::DealDamage {
                    to: target_filtered(R::OpponentPlayer.or(R::Planeswalker)),
                    amount: Value::Const(4),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Airdrop Condor — throw a Goblin at anything.
pub fn airdrop_condor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            sac_other_filter: Some((
                R::Creature.and(R::HasCreatureType(CreatureType::Goblin)),
                1,
            )),
            effect: Effect::DealDamage {
                to: target_filtered(R::Any),
                amount: Value::SacrificedPower,
            },
            ..Default::default()
        }],
        ..creature("Airdrop Condor", cost(&[generic(4), r()]), vec![CreatureType::Bird], 2, 2)
    }
}

/// Aphetto Alchemist — untaps anything; Morph {U}.
pub fn aphetto_alchemist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[u()]))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Untap {
                what: target_filtered(R::Artifact.or(R::Creature)),
                up_to: None,
            },
            ..Default::default()
        }],
        ..creature(
            "Aphetto Alchemist",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            2,
        )
    }
}

/// Boneknitter — regenerates Zombies; Morph {2}{B}.
pub fn boneknitter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(2), b()]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::Regenerate {
                what: target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Zombie))),
            },
            ..Default::default()
        }],
        ..creature(
            "Boneknitter",
            cost(&[generic(1), b()]),
            vec![CreatureType::Zombie, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Battlefield Medic — a Cleric-sized damage shield.
pub fn battlefield_medic() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PreventNextDamage {
                target: target_filtered(R::Creature),
                amount: tribe_count(CreatureType::Cleric),
            },
            ..Default::default()
        }],
        ..creature(
            "Battlefield Medic",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Daru Healer — one point of prevention a turn; Morph {W}.
pub fn daru_healer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[w()]))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PreventNextDamage {
                target: target_filtered(R::Any),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Daru Healer",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            2,
        )
    }
}

/// Daunting Defender — your Clerics shrug off a point of every hit.
pub fn daunting_defender() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Prevent 1 damage dealt to Cleric creatures you control.",
            effect: StaticEffect::ReduceDamageToYourMatchingCreaturesBy {
                filter: R::HasCreatureType(CreatureType::Cleric),
                amount: 1,
            },
        }],
        ..creature(
            "Daunting Defender",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            3,
            3,
        )
    }
}

/// Broodhatch Nantuko — every point of damage buys an Insect; Morph {2}{G}.
pub fn broodhatch_nantuko() -> CardDefinition {
    let insect = TokenDefinition {
        name: "Insect".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(2), g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Create that many Insects?".into(),
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::TriggerEventAmount,
                    definition: insect,
                }),
            },
        }],
        ..creature(
            "Broodhatch Nantuko",
            cost(&[generic(1), g()]),
            vec![CreatureType::Insect, CreatureType::Druid],
            1,
            1,
        )
    }
}

/// Disruptive Pitmage — a repeatable {1} tax; Morph {U}.
pub fn disruptive_pitmage() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[u()]))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::CounterUnlessPaid {
                what: target_filtered(R::IsSpellOnStack),
                mana_cost: cost(&[generic(1)]),
                exile: false,
                extra_generic: None,
            },
            ..Default::default()
        }],
        ..creature(
            "Disruptive Pitmage",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Dwarven Blastminer — nonbasic land hate; Morph {R}.
pub fn dwarven_blastminer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[r()]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            tap_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::IsNonbasicLand) },
            ..Default::default()
        }],
        ..creature("Dwarven Blastminer", cost(&[generic(1), r()]), vec![CreatureType::Dwarf], 1, 1)
    }
}

/// Gravel Slinger — pings a combatant; Morph {1}{W}.
pub fn gravel_slinger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(1), w()]))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Gravel Slinger",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            3,
        )
    }
}

/// Venomspout Brackus — shoots fliers out of the sky; Morph {3}{G}{G}.
pub fn venomspout_brackus() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(3), g(), g()]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(
                    R::Creature
                        .and(R::IsAttacking.or(R::IsBlocking))
                        .and(R::HasKeyword(Keyword::Flying)),
                ),
                amount: Value::Const(5),
            },
            ..Default::default()
        }],
        ..creature("Venomspout Brackus", cost(&[generic(6), g()]), vec![CreatureType::Beast], 5, 5)
    }
}

/// Whipcorder — taps a creature every turn; Morph {W}.
pub fn whipcorder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[w()]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..creature(
            "Whipcorder",
            cost(&[w(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Rebel],
            2,
            2,
        )
    }
}

/// Voidmage Prodigy — feed it a Wizard to counter anything; Morph {U}.
pub fn voidmage_prodigy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[u()]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), u()]),
            sac_other_filter: Some((
                R::Creature.and(R::HasCreatureType(CreatureType::Wizard)),
                1,
            )),
            effect: Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            ..Default::default()
        }],
        ..creature(
            "Voidmage Prodigy",
            cost(&[u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            1,
        )
    }
}

/// Serpentine Basilisk — anything it touches dies; Morph {1}{G}{G}.
pub fn serpentine_basilisk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(1), g(), g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToCreature, EventScope::SelfSource),
            effect: Effect::AtEndOfCombat {
                body: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            },
        }],
        ..creature(
            "Serpentine Basilisk",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Basilisk],
            2,
            3,
        )
    }
}

/// Spitfire Handler — firebreathing that won't block up.
pub fn spitfire_handler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBlockGreaterPowerThanSelf],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Spitfire Handler", cost(&[generic(1), r()]), vec![CreatureType::Goblin], 1, 1)
    }
}

/// Rummaging Wizard — surveil at will.
pub fn rummaging_wizard() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature(
            "Rummaging Wizard",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Information Dealer — look at as many cards as you have Wizards.
pub fn information_dealer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::RearrangeTop {
                who: PlayerRef::You,
                amount: tribe_count(CreatureType::Wizard),
            },
            ..Default::default()
        }],
        ..creature(
            "Information Dealer",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Goblin Pyromancer — a one-turn Goblin overrun that eats the team.
pub fn goblin_pyromancer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::PumpPT {
                what: Selector::EachPermanent(
                    R::Creature.and(R::HasCreatureType(CreatureType::Goblin)),
                ),
                power: Value::Const(3),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::End),
                    EventScope::AnyPlayer,
                ),
                effect: Effect::Destroy {
                    what: Selector::EachPermanent(
                        R::Creature.and(R::HasCreatureType(CreatureType::Goblin)),
                    ),
                },
            },
        ],
        ..creature(
            "Goblin Pyromancer",
            cost(&[generic(3), r()]),
            vec![CreatureType::Goblin, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Cruel Revival — kill a non-Zombie, buy back a Zombie.
pub fn cruel_revival() -> CardDefinition {
    CardDefinition {
        name: "Cruel Revival",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::HasCreatureType(CreatureType::Zombie).negate()),
                },
            },
            Effect::CantBeRegeneratedThisTurn { what: Selector::Target(0) },
            Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature
                        .and(R::HasCreatureType(CreatureType::Zombie))
                        .and(R::InYourGraveyard),
                },
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        ..Default::default()
    }
}

/// Oversold Cemetery — a creature back every upkeep once the yard fills.
pub fn oversold_cemetery() -> CardDefinition {
    CardDefinition {
        name: "Oversold Cemetery",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            )
            .with_filter(Predicate::SelectorCountAtLeast {
                sel: Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: R::Creature,
                },
                n: Value::Const(4),
            }),
            effect: Effect::MayDo {
                description: "Return a creature card from your graveyard to your hand?".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Convalescent Care — a lifeline while you're low.
pub fn convalescent_care() -> CardDefinition {
    CardDefinition {
        name: "Convalescent Care",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            )
            .with_filter(Predicate::ValueAtMost(
                Value::LifeOf(PlayerRef::You),
                Value::Const(5),
            )),
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                draw(1),
            ]),
        }],
        ..Default::default()
    }
}

/// Lightning Rift — cycling turns into damage.
pub fn lightning_rift() -> CardDefinition {
    CardDefinition {
        name: "Lightning Rift",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardCycled, EventScope::AnyPlayer),
            effect: Effect::MayPay {
                description: "Pay one generic mana to deal 2 damage?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::DealDamage {
                    to: target_filtered(R::Any),
                    amount: Value::Const(2),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Invigorating Boon — cycling turns into counters.
pub fn invigorating_boon() -> CardDefinition {
    CardDefinition {
        name: "Invigorating Boon",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardCycled, EventScope::AnyPlayer),
            effect: Effect::MayDo {
                description: "Put a +1/+1 counter on a creature?".into(),
                body: Box::new(Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Sea's Claim — turn a land into an Island.
pub fn seas_claim() -> CardDefinition {
    CardDefinition {
        name: "Sea's Claim",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        equipped_bonus: Some(EquipBonus {
            set_land_types: Some(vec![LandType::Island]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Sandskin — a creature that neither deals nor takes combat damage.
pub fn sandskin() -> CardDefinition {
    CardDefinition {
        name: "Sandskin",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![StaticAbility {
            description: "Prevent all combat damage that would be dealt to and by enchanted creature.",
            effect: StaticEffect::PreventAllCombatDamageToAndFromEnchanted,
        }],
        ..Default::default()
    }
}

/// Withering Hex — an Aura that rots as the table cycles.
pub fn withering_hex() -> CardDefinition {
    CardDefinition {
        name: "Withering Hex",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardCycled, EventScope::AnyPlayer),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: crate::card::CounterType::Plague,
                amount: Value::ONE,
            },
        }],
        equipped_bonus: Some(EquipBonus {
            scale: Some(crate::card::EquipScale {
                filter: R::Any,
                per_power: -1,
                per_toughness: -1,
                count_self_counters: Some(crate::card::CounterType::Plague),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Run Wild — trample and a regeneration outlet for a turn.
pub fn run_wild() -> CardDefinition {
    CardDefinition {
        name: "Run Wild",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::GrantKeywords {
            what: target_filtered(R::Creature),
            keywords: vec![Keyword::Trample, Keyword::Regenerate(1)],
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Insurrection — take the whole board for a swing.
pub fn insurrection() -> CardDefinition {
    CardDefinition {
        name: "Insurrection",
        cost: cost(&[generic(5), r(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Untap { what: Selector::EachPermanent(R::Creature), up_to: None },
            Effect::GainControl {
                what: Selector::EachPermanent(R::Creature),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Tribal Golem — a Golem that borrows a keyword from every tribe you field.
pub fn tribal_golem() -> CardDefinition {
    let grant = |tribe: CreatureType, keyword: Keyword| StaticAbility {
        description: "Tribal Golem's conditional keyword grant.",
        effect: StaticEffect::WhileCondition {
            condition: Predicate::SelectorExists(Selector::ControlledBy {
                who: PlayerRef::You,
                filter: R::Creature.and(R::HasCreatureType(tribe)),
            }),
            inner: Box::new(StaticEffect::GrantKeyword {
                applies_to: Selector::This,
                keyword,
            }),
        },
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![
            grant(CreatureType::Beast, Keyword::Trample),
            grant(CreatureType::Goblin, Keyword::Haste),
            grant(CreatureType::Soldier, Keyword::FirstStrike),
            grant(CreatureType::Wizard, Keyword::Flying),
            grant(CreatureType::Zombie, Keyword::Regenerate(1)),
        ],
        ..creature("Tribal Golem", cost(&[generic(6)]), vec![CreatureType::Golem], 4, 4)
    }
}

// ── The Onslaught charms ────────────────────────────────────────────────────

/// Fever Charm — haste, a pump, or a dead Wizard.
pub fn fever_charm() -> CardDefinition {
    CardDefinition {
        name: "Fever Charm",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Wizard))),
                amount: Value::Const(3),
            },
        ]),
        ..Default::default()
    }
}

/// Misery Charm — kill a Cleric, raise a Cleric, or drain.
pub fn misery_charm() -> CardDefinition {
    CardDefinition {
        name: "Misery Charm",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy {
                what: target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Cleric))),
            },
            Effect::Move {
                what: target_filtered(
                    R::Creature
                        .and(R::HasCreatureType(CreatureType::Cleric))
                        .and(R::InYourGraveyard),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Piety Charm — crack an Aura, pump a Soldier, or hand out vigilance.
pub fn piety_charm() -> CardDefinition {
    CardDefinition {
        name: "Piety Charm",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy {
                what: target_filtered(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)),
            },
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Soldier))),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Trickery Charm — flying, a retype, or a look at the top four.
pub fn trickery_charm() -> CardDefinition {
    CardDefinition {
        name: "Trickery Charm",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            Effect::BecomeChosenCreatureType {
                what: target_filtered(R::Creature),
                duration: Duration::EndOfTurn,
                excluded: vec![],
            },
            Effect::RearrangeTop { who: PlayerRef::You, amount: Value::Const(4) },
        ]),
        ..Default::default()
    }
}

/// Vitality Charm — an Insect, a pump, or a saved Beast.
pub fn vitality_charm() -> CardDefinition {
    let insect = TokenDefinition {
        name: "Insect".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Vitality Charm",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: insect,
            },
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
            Effect::Regenerate {
                what: target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Beast))),
            },
        ]),
        ..Default::default()
    }
}
