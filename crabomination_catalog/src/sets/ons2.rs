//! Onslaught (ONS) wave 2 — the Morph commons, the tap-a-tribe activations,
//! the Avatar cycle and the cycling utility spells. Tests in
//! `classic_sets/ons`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, DynamicPt, Keyword, LandType,
    Predicate, SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility,
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
