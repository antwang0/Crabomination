//! Gatecrash (GTC) — 2013. A first wave of the set: vanilla/French-vanilla
//! beaters, simple keyword-granters, combat spells, and Auras. Tests in
//! `classic_sets/gtc`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect,
    EnchantmentSubtype, Keyword, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    TokenDefinition, Value,
};
use crate::effect::shortcut::{on_dies, target_filtered};
use crate::effect::{Duration, LibraryPosition, PlayerRef, Selector, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, hybrid, r, u, w, x};

fn creatures(t: Vec<CreatureType>) -> Subtypes {
    Subtypes {
        creature_types: t,
        ..Default::default()
    }
}
fn aura() -> Subtypes {
    Subtypes {
        enchantment_subtypes: vec![EnchantmentSubtype::Aura],
        ..Default::default()
    }
}

// ── Vanilla / French-vanilla beaters ────────────────────────────────────────

/// Gutter Skulk — {1}{B} 2/2 Zombie Rat.
pub fn gutter_skulk() -> CardDefinition {
    CardDefinition {
        name: "Gutter Skulk",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Zombie, CreatureType::Rat]),
        power: 2,
        toughness: 2,
        ..Default::default()
    }
}

/// Ruination Wurm — {4}{R}{G} 7/6 Wurm.
pub fn ruination_wurm() -> CardDefinition {
    CardDefinition {
        name: "Ruination Wurm",
        cost: cost(&[generic(4), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Wurm]),
        power: 7,
        toughness: 6,
        ..Default::default()
    }
}

/// Assault Griffin — {3}{W} 3/2 Griffin with flying.
pub fn assault_griffin() -> CardDefinition {
    CardDefinition {
        name: "Assault Griffin",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Griffin]),
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Millennial Gargoyle — {4} 2/2 Artifact Gargoyle with flying.
pub fn millennial_gargoyle() -> CardDefinition {
    CardDefinition {
        name: "Millennial Gargoyle",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: creatures(vec![CreatureType::Gargoyle]),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Drakewing Krasis — {1}{G}{U} 3/1 Lizard Drake with flying and trample.
pub fn drakewing_krasis() -> CardDefinition {
    CardDefinition {
        name: "Drakewing Krasis",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Lizard, CreatureType::Drake]),
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        ..Default::default()
    }
}

/// Ember Beast — {2}{R} 3/4 Beast that can't attack or block alone.
pub fn ember_beast() -> CardDefinition {
    CardDefinition {
        name: "Ember Beast",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Beast]),
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::CantAttackOrBlockAlone],
        ..Default::default()
    }
}

// ── Keyword-granting activated abilities ─────────────────────────────────────

fn grant_self_keyword_eot(mana: crate::mana::ManaCost, kw: Keyword) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        effect: Effect::GrantKeyword {
            what: Selector::This,
            keyword: kw,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Disciple of the Old Ways — {1}{G} 2/2 Human Warrior. {R}: first strike EOT.
pub fn disciple_of_the_old_ways() -> CardDefinition {
    CardDefinition {
        name: "Disciple of the Old Ways",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Warrior]),
        power: 2,
        toughness: 2,
        activated_abilities: vec![grant_self_keyword_eot(cost(&[r()]), Keyword::FirstStrike)],
        ..Default::default()
    }
}

/// Towering Thunderfist — {4}{R} 4/4 Giant Soldier. {W}: vigilance EOT.
pub fn towering_thunderfist() -> CardDefinition {
    CardDefinition {
        name: "Towering Thunderfist",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Giant, CreatureType::Soldier]),
        power: 4,
        toughness: 4,
        activated_abilities: vec![grant_self_keyword_eot(cost(&[w()]), Keyword::Vigilance)],
        ..Default::default()
    }
}

/// Metropolis Sprite — {1}{U} 1/2 Faerie Rogue with flying. {U}: +1/-1 EOT.
pub fn metropolis_sprite() -> CardDefinition {
    CardDefinition {
        name: "Metropolis Sprite",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Faerie, CreatureType::Rogue]),
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Zarichi Tiger — {3}{W} 2/3 Cat. {1}{W}, {T}: You gain 2 life.
pub fn zarichi_tiger() -> CardDefinition {
    CardDefinition {
        name: "Zarichi Tiger",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Cat]),
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            tap_cost: true,
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dutiful Thrull — {W} 1/1 Thrull. {B}: Regenerate this creature.
pub fn dutiful_thrull() -> CardDefinition {
    CardDefinition {
        name: "Dutiful Thrull",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Thrull]),
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::Regenerate {
                what: Selector::This,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Dies triggers ────────────────────────────────────────────────────────────

/// Mortus Strider — {1}{U}{B} 1/1 Skeleton. Dies: return it to its owner's hand.
pub fn mortus_strider() -> CardDefinition {
    CardDefinition {
        name: "Mortus Strider",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Skeleton]),
        power: 1,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::Move {
            what: Selector::This,
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        })],
        ..Default::default()
    }
}

/// Mindeye Drake — {4}{U} 2/5 Drake with flying. Dies: target player mills five.
pub fn mindeye_drake() -> CardDefinition {
    CardDefinition {
        name: "Mindeye Drake",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Drake]),
        power: 2,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::Mill {
            who: target_filtered(R::Player),
            amount: Value::Const(5),
        })],
        ..Default::default()
    }
}

/// Nimbus Swimmer — {X}{G}{U} 0/0 Leviathan with flying, enters with X counters.
pub fn nimbus_swimmer() -> CardDefinition {
    CardDefinition {
        name: "Nimbus Swimmer",
        cost: cost(&[x(), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Leviathan]),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        ..Default::default()
    }
}

// ── Spells ───────────────────────────────────────────────────────────────────

/// Shattering Blow — {1}{R/W} Instant. Exile target artifact.
pub fn shattering_blow() -> CardDefinition {
    CardDefinition {
        name: "Shattering Blow",
        cost: cost(&[generic(1), hybrid(Color::Red, Color::White)]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(R::Artifact),
            to: ZoneDest::Exile,
        },
        ..Default::default()
    }
}

/// Smite — {W} Instant. Destroy target blocked creature.
pub fn smite() -> CardDefinition {
    CardDefinition {
        name: "Smite",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(R::Creature.and(R::IsBlocked)),
        },
        ..Default::default()
    }
}

/// Killing Glare — {X}{B} Instant. Destroy target creature with power X or less.
pub fn killing_glare() -> CardDefinition {
    CardDefinition {
        name: "Killing Glare",
        cost: cost(&[x(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(R::Creature.and(R::PowerAtMostXFromCost)),
        },
        ..Default::default()
    }
}

/// Scatter Arc — {3}{U} Instant. Counter target noncreature spell. Draw a card.
pub fn scatter_arc() -> CardDefinition {
    CardDefinition {
        name: "Scatter Arc",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack.and(R::Creature.negate())),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Psychic Strike — {1}{U}{B} Instant. Counter target spell. Its controller
/// mills two cards.
pub fn psychic_strike() -> CardDefinition {
    CardDefinition {
        name: "Psychic Strike",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Mill {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(2),
            },
            Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack),
            },
        ]),
        ..Default::default()
    }
}

/// Righteous Charge — {1}{W}{W} Sorcery. Creatures you control get +2/+2 EOT.
pub fn righteous_charge() -> CardDefinition {
    CardDefinition {
        name: "Righteous Charge",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Burst of Strength — {G} Instant. Put a +1/+1 counter on target creature and
/// untap it.
pub fn burst_of_strength() -> CardDefinition {
    CardDefinition {
        name: "Burst of Strength",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::Untap {
                what: Selector::Target(0),
                up_to: None,
            },
        ]),
        ..Default::default()
    }
}

/// Purge the Profane — {2}{W}{B} Sorcery. Target opponent discards two cards
/// and you gain 2 life.
pub fn purge_the_profane() -> CardDefinition {
    CardDefinition {
        name: "Purge the Profane",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: target_filtered(R::OpponentPlayer),
                amount: Value::Const(2),
                random: false,
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Knight Watch — {4}{W} Sorcery. Create two 2/2 white Knight tokens with
/// vigilance.
pub fn knight_watch() -> CardDefinition {
    let knight = TokenDefinition {
        name: "Knight".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: creatures(vec![CreatureType::Knight]),
        keywords: vec![Keyword::Vigilance],
        ..Default::default()
    };
    CardDefinition {
        name: "Knight Watch",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: Box::new(knight),
        },
        ..Default::default()
    }
}

/// Totally Lost — {4}{U} Instant. Put target nonland permanent on top of its
/// owner's library.
pub fn totally_lost() -> CardDefinition {
    CardDefinition {
        name: "Totally Lost",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(R::Permanent.and(R::Land.negate())),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOfMoved,
                pos: LibraryPosition::Top,
            },
        },
        ..Default::default()
    }
}

/// Urban Evolution — {3}{G}{U} Sorcery. Draw three cards. You may play an
/// additional land this turn.
pub fn urban_evolution() -> CardDefinition {
    CardDefinition {
        name: "Urban Evolution",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(3),
            },
            Effect::GrantExtraLandPlay {
                who: PlayerRef::You,
                count: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Tower Defense — {1}{G} Instant. Creatures you control get +0/+5 and gain
/// reach until end of turn.
pub fn tower_defense() -> CardDefinition {
    let team = Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Tower Defense",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: team.clone(),
                power: Value::ZERO,
                toughness: Value::Const(5),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: team,
                keyword: Keyword::Reach,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

// ── Auras ────────────────────────────────────────────────────────────────────

fn buff_aura(
    name: &'static str,
    mana: crate::mana::ManaCost,
    pt: (i32, i32),
    kws: Vec<Keyword>,
) -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: aura(),
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: pt.0,
            toughness: pt.1,
            keywords: kws,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Primal Visitation — {3}{R}{G} Aura. Enchanted creature gets +3/+3 and has
/// haste.
pub fn primal_visitation() -> CardDefinition {
    buff_aura(
        "Primal Visitation",
        cost(&[generic(3), r(), g()]),
        (3, 3),
        vec![Keyword::Haste],
    )
}

/// Madcap Skills — {1}{R} Aura. Enchanted creature gets +3/+0 and has menace.
pub fn madcap_skills() -> CardDefinition {
    buff_aura(
        "Madcap Skills",
        cost(&[generic(1), r()]),
        (3, 0),
        vec![Keyword::Menace],
    )
}

/// Guildscorn Ward — {W} Aura. Enchanted creature has protection from
/// multicolored.
pub fn guildscorn_ward() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "Guildscorn Ward",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: aura(),
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::ProtectionFromMulticolored],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Debtor's Pulpit — {4}{W} Aura. Enchant land. Enchanted land has
/// "{T}: Tap target creature."
pub fn debtors_pulpit() -> CardDefinition {
    CardDefinition {
        name: "Debtor's Pulpit",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: aura(),
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Land),
        },
        static_abilities: vec![StaticAbility {
            description: "Enchanted land has \"{T}: Tap target creature.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::Tap {
                        what: target_filtered(R::Creature),
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..Default::default()
    }
}

/// Illness in the Ranks — {B} Enchantment. Creature tokens get -1/-1.
pub fn illness_in_the_ranks() -> CardDefinition {
    CardDefinition {
        name: "Illness in the Ranks",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creature tokens get -1/-1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::IsToken)),
                power: -1,
                toughness: -1,
            },
        }],
        ..Default::default()
    }
}

/// Smog Elemental — {4}{B}{B} 3/3 Elemental with flying. Creatures with flying
/// your opponents control get -1/-1.
pub fn smog_elemental() -> CardDefinition {
    CardDefinition {
        name: "Smog Elemental",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Elemental]),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Creatures with flying your opponents control get -1/-1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByOpponent)
                        .and(R::HasKeyword(Keyword::Flying)),
                ),
                power: -1,
                toughness: -1,
            },
        }],
        ..Default::default()
    }
}
