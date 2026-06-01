use super::tap_add;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, LandType, Subtypes, TriggeredAbility,
};
use crate::card::SelectionRequirement;
use crate::effect::shortcut::{deal, etb_gain_life, target, target_filtered};
use crate::effect::{Duration, ManaPayload, PlayerRef, Selector, Value};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

/// Savannah Lions — {W} 2/1
pub fn savannah_lions() -> CardDefinition {
    CardDefinition {
        name: "Savannah Lions",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Lion],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// White Knight — {W}{W} 2/2 First Strike
pub fn white_knight() -> CardDefinition {
    CardDefinition {
        name: "White Knight",
        cost: cost(&[w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Serra Angel — {3}{W}{W} 4/4 Flying Vigilance
pub fn serra_angel() -> CardDefinition {
    CardDefinition {
        name: "Serra Angel",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Mahamoti Djinn — {4}{U}{U} 5/6 Flying
pub fn mahamoti_djinn() -> CardDefinition {
    CardDefinition {
        name: "Mahamoti Djinn",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn],
            ..Default::default()
        },
        power: 5,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Prodigal Sorcerer — {2}{U} 1/1, {T}: Deal 1 damage to any target
pub fn prodigal_sorcerer() -> CardDefinition {
    CardDefinition {
        name: "Prodigal Sorcerer",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: deal(1, target()),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Black Knight — {B}{B} 2/2 First Strike
pub fn black_knight() -> CardDefinition {
    CardDefinition {
        name: "Black Knight",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Hypnotic Specter — {1}{B}{B} 2/2 Flying
/// Hypnotic Specter — {1}{B}{B} 2/2 Flying.
/// Whenever Hypnotic Specter deals damage to an opponent, that player
/// discards a card at random.
pub fn hypnotic_specter() -> CardDefinition {
    CardDefinition {
        name: "Hypnotic Specter",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Specter],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::SelfSource,
            ),
            // Combat fires this trigger with `target = Player(damaged)` so
            // `PlayerRef::Target(0)` resolves to exactly the player who took
            // damage — not every opponent.
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(1),
                random: true,
            },
        }],
        ..Default::default()
    }
}

/// Sengir Vampire — {3}{B}{B} 4/4 Flying
pub fn sengir_vampire() -> CardDefinition {
    CardDefinition {
        name: "Sengir Vampire",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Shivan Dragon — {4}{R}{R} 5/5 Flying
pub fn shivan_dragon() -> CardDefinition {
    CardDefinition {
        name: "Shivan Dragon",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Grizzly Bears — {1}{G} 2/2
pub fn grizzly_bears() -> CardDefinition {
    CardDefinition {
        name: "Grizzly Bears",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bear],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Birds of Paradise — {G} 0/1 Flying, {T}: Add one mana of any color
pub fn birds_of_paradise() -> CardDefinition {
    CardDefinition {
        name: "Birds of Paradise",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Llanowar Elves — {G} 1/1, {T}: Add {G}
pub fn llanowar_elves() -> CardDefinition {
    CardDefinition {
        name: "Llanowar Elves",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add(Color::Green)],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Elvish Archers — {1}{G} 2/1 First Strike (LEA).
pub fn elvish_archers() -> CardDefinition {
    CardDefinition {
        name: "Elvish Archers",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Archer],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Craw Wurm — {4}{G}{G} 6/4
pub fn craw_wurm() -> CardDefinition {
    CardDefinition {
        name: "Craw Wurm",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wurm],
            ..Default::default()
        },
        power: 6,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Samite Healer — {2}{W} 1/1 Human Cleric. "{T}: Prevent the next 1
/// damage that would be dealt to any target this turn." (CR 615.7)
pub fn samite_healer() -> CardDefinition {
    CardDefinition {
        name: "Samite Healer",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::PreventNextDamage {
                target: target_filtered(
                    SelectionRequirement::Player
                        .or(SelectionRequirement::Creature)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(1),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
            sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        ..Default::default()
    }
}

// ── Classic vanilla / keyword bodies (claude/modern_decks) ───────────────────
// Core-set staples built on existing primitives. Each has a functionality
// test in `tests/classic.rs`.

/// Gray Ogre — {2}{R} 2/2 Ogre.
pub fn gray_ogre() -> CardDefinition {
    CardDefinition {
        name: "Gray Ogre",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ogre], ..Default::default() },
        power: 2,
        toughness: 2,
        ..Default::default()
    }
}

/// Hurloon Minotaur — {1}{R}{R} 2/3 Minotaur.
pub fn hurloon_minotaur() -> CardDefinition {
    CardDefinition {
        name: "Hurloon Minotaur",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Minotaur], ..Default::default() },
        power: 2,
        toughness: 3,
        ..Default::default()
    }
}

/// Spined Wurm — {4}{G} 5/4 Wurm.
pub fn spined_wurm() -> CardDefinition {
    CardDefinition {
        name: "Spined Wurm",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wurm], ..Default::default() },
        power: 5,
        toughness: 4,
        ..Default::default()
    }
}

/// Trained Armodon — {2}{G} 3/3 Elephant.
pub fn trained_armodon() -> CardDefinition {
    CardDefinition {
        name: "Trained Armodon",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elephant], ..Default::default() },
        power: 3,
        toughness: 3,
        ..Default::default()
    }
}

/// Pearled Unicorn — {2}{W} 2/2 Unicorn.
pub fn pearled_unicorn() -> CardDefinition {
    CardDefinition {
        name: "Pearled Unicorn",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Unicorn], ..Default::default() },
        power: 2,
        toughness: 2,
        ..Default::default()
    }
}

/// Obsianus Golem — {6} 4/6 Golem.
pub fn obsianus_golem() -> CardDefinition {
    CardDefinition {
        name: "Obsianus Golem",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: 4,
        toughness: 6,
        ..Default::default()
    }
}

/// Eager Cadet — {W} 1/1 Soldier.
pub fn eager_cadet() -> CardDefinition {
    CardDefinition {
        name: "Eager Cadet",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        power: 1,
        toughness: 1,
        ..Default::default()
    }
}

/// Elite Vanguard — {W} 2/1 Soldier.
pub fn elite_vanguard() -> CardDefinition {
    CardDefinition {
        name: "Elite Vanguard",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        power: 2,
        toughness: 1,
        ..Default::default()
    }
}

/// Devoted Hero — {W} 1/2 Soldier.
pub fn devoted_hero() -> CardDefinition {
    CardDefinition {
        name: "Devoted Hero",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        power: 1,
        toughness: 2,
        ..Default::default()
    }
}

/// Giant Spider — {3}{G} 2/4 Spider with reach.
pub fn giant_spider() -> CardDefinition {
    CardDefinition {
        name: "Giant Spider",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        ..Default::default()
    }
}

/// Air Elemental — {3}{U}{U} 4/4 Elemental with flying.
pub fn air_elemental() -> CardDefinition {
    CardDefinition {
        name: "Air Elemental",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Scryb Sprites — {G} 1/1 Faerie with flying.
pub fn scryb_sprites() -> CardDefinition {
    CardDefinition {
        name: "Scryb Sprites",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Tundra Wolves — {W} 1/1 Wolf with first strike.
pub fn tundra_wolves() -> CardDefinition {
    CardDefinition {
        name: "Tundra Wolves",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::FirstStrike],
        ..Default::default()
    }
}

/// Mesa Pegasus — {1}{W} 1/1 Pegasus with flying and banding.
pub fn mesa_pegasus() -> CardDefinition {
    CardDefinition {
        name: "Mesa Pegasus",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Pegasus], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Banding],
        ..Default::default()
    }
}

/// Wall of Air — {1}{U}{U} 1/5 Wall with defender and flying.
pub fn wall_of_air() -> CardDefinition {
    CardDefinition {
        name: "Wall of Air",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wall], ..Default::default() },
        power: 1,
        toughness: 5,
        keywords: vec![Keyword::Defender, Keyword::Flying],
        ..Default::default()
    }
}

/// Wall of Swords — {3}{W} 3/5 Wall with defender and flying.
pub fn wall_of_swords() -> CardDefinition {
    CardDefinition {
        name: "Wall of Swords",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wall], ..Default::default() },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Defender, Keyword::Flying],
        ..Default::default()
    }
}

/// Wall of Wood — {G} 0/3 Wall with defender.
pub fn wall_of_wood() -> CardDefinition {
    CardDefinition {
        name: "Wall of Wood",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wall], ..Default::default() },
        power: 0,
        toughness: 3,
        keywords: vec![Keyword::Defender],
        ..Default::default()
    }
}

/// Wall of Stone — {1}{R}{R} 0/8 Wall with defender.
pub fn wall_of_stone() -> CardDefinition {
    CardDefinition {
        name: "Wall of Stone",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wall], ..Default::default() },
        power: 0,
        toughness: 8,
        keywords: vec![Keyword::Defender],
        ..Default::default()
    }
}

/// Yotian Soldier — {3} 1/4 artifact Soldier with vigilance.
pub fn yotian_soldier() -> CardDefinition {
    CardDefinition {
        name: "Yotian Soldier",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        ..Default::default()
    }
}

/// Royal Assassin — {1}{B}{B} 1/1 Assassin. "{T}: Destroy target tapped
/// creature."
pub fn royal_assassin() -> CardDefinition {
    CardDefinition {
        name: "Royal Assassin",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Assassin], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::Tapped),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wall of Fire — {1}{R}{R} 0/5 Wall with defender. "{R}: +1/+0 until EOT."
pub fn wall_of_fire() -> CardDefinition {
    CardDefinition {
        name: "Wall of Fire",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wall], ..Default::default() },
        power: 0,
        toughness: 5,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![pump_one_zero(&[r()])],
        ..Default::default()
    }
}

/// Flame Spirit — {2}{R} 2/2 Spirit. "{R}: +1/+0 until EOT."
pub fn flame_spirit() -> CardDefinition {
    CardDefinition {
        name: "Flame Spirit",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![pump_one_zero(&[r()])],
        ..Default::default()
    }
}

/// Goblin Balloon Brigade — {R} 1/1 Goblin. "{R}: gains flying until EOT."
pub fn goblin_balloon_brigade() -> CardDefinition {
    CardDefinition {
        name: "Goblin Balloon Brigade",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// "{cost}: This creature gets +1/+0 until end of turn." (Firebreathing
/// shape — Flame Spirit, Wall of Fire.)
fn pump_one_zero(cost_syms: &[crate::mana::ManaSymbol]) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost(cost_syms),
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(1),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

// ── Classic core-set bodies, batch 2 (claude/modern_decks) ───────────────────

/// "{cost}: This creature gets +n/+n until end of turn." (Shade-style pump.)
fn pump_nn(cost_syms: &[crate::mana::ManaSymbol], n: i32) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost(cost_syms),
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(n),
            toughness: Value::Const(n),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

fn body(
    name: &'static str,
    cost_syms: &[crate::mana::ManaSymbol],
    types: Vec<CreatureType>,
    power: i32,
    toughness: i32,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(cost_syms),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power,
        toughness,
        keywords,
        ..Default::default()
    }
}

// White
/// Youthful Knight — {1}{W} 2/1 Knight with first strike.
pub fn youthful_knight() -> CardDefinition {
    body("Youthful Knight", &[generic(1), w()], vec![CreatureType::Human, CreatureType::Knight], 2, 1, vec![Keyword::FirstStrike])
}
/// Standing Troops — {2}{W} 1/4 Soldier with vigilance.
pub fn standing_troops() -> CardDefinition {
    body("Standing Troops", &[generic(2), w()], vec![CreatureType::Soldier], 1, 4, vec![Keyword::Vigilance])
}
/// Benalish Hero — {W} 1/1 Soldier with banding.
pub fn benalish_hero() -> CardDefinition {
    body("Benalish Hero", &[w()], vec![CreatureType::Human, CreatureType::Soldier], 1, 1, vec![Keyword::Banding])
}
/// Skyhunter Skirmisher — {1}{W} 1/1 Cat Knight with flying and double strike.
pub fn skyhunter_skirmisher() -> CardDefinition {
    body("Skyhunter Skirmisher", &[generic(1), w()], vec![CreatureType::Cat, CreatureType::Knight], 1, 1, vec![Keyword::Flying, Keyword::DoubleStrike])
}
/// Knight Errant — {1}{W} 2/2 Human Knight.
pub fn knight_errant() -> CardDefinition {
    body("Knight Errant", &[generic(1), w()], vec![CreatureType::Human, CreatureType::Knight], 2, 2, vec![])
}
/// Venerable Monk — {2}{W} 2/2 Monk Cleric. "When this enters, you gain 2 life."
pub fn venerable_monk() -> CardDefinition {
    let mut c = body("Venerable Monk", &[generic(2), w()], vec![CreatureType::Monk, CreatureType::Cleric], 2, 2, vec![]);
    c.triggered_abilities = vec![etb_gain_life(2)];
    c
}

// Blue
/// Snapping Drake — {3}{U} 3/2 Drake with flying.
pub fn snapping_drake() -> CardDefinition {
    body("Snapping Drake", &[generic(3), u()], vec![CreatureType::Drake], 3, 2, vec![Keyword::Flying])
}
/// Phantom Warrior — {1}{U}{U} 2/2 Illusion Warrior that can't be blocked.
pub fn phantom_warrior() -> CardDefinition {
    body("Phantom Warrior", &[generic(1), u(), u()], vec![CreatureType::Illusion, CreatureType::Warrior], 2, 2, vec![Keyword::Unblockable])
}
/// Merfolk of the Pearl Trident — {U} 1/1 Merfolk.
pub fn merfolk_of_the_pearl_trident() -> CardDefinition {
    body("Merfolk of the Pearl Trident", &[u()], vec![CreatureType::Merfolk], 1, 1, vec![])
}
/// Vodalian Soldiers — {1}{U} 1/2 Merfolk Soldier.
pub fn vodalian_soldiers() -> CardDefinition {
    body("Vodalian Soldiers", &[generic(1), u()], vec![CreatureType::Merfolk, CreatureType::Soldier], 1, 2, vec![])
}
/// Sea Eagle — {1}{U} 1/1 Bird with flying.
pub fn sea_eagle() -> CardDefinition {
    body("Sea Eagle", &[generic(1), u()], vec![CreatureType::Bird], 1, 1, vec![Keyword::Flying])
}
/// Wind Spirit — {3}{U} 2/3 Spirit with flying.
pub fn wind_spirit() -> CardDefinition {
    body("Wind Spirit", &[generic(3), u()], vec![CreatureType::Spirit], 2, 3, vec![Keyword::Flying])
}

// Black
/// Scathe Zombies — {2}{B} 2/2 Zombie.
pub fn scathe_zombies() -> CardDefinition {
    body("Scathe Zombies", &[generic(2), b()], vec![CreatureType::Zombie], 2, 2, vec![])
}
/// Walking Corpse — {1}{B} 2/2 Zombie.
pub fn walking_corpse() -> CardDefinition {
    body("Walking Corpse", &[generic(1), b()], vec![CreatureType::Zombie], 2, 2, vec![])
}
/// Bog Imp — {1}{B} 1/1 Imp with flying.
pub fn bog_imp() -> CardDefinition {
    body("Bog Imp", &[generic(1), b()], vec![CreatureType::Imp], 1, 1, vec![Keyword::Flying])
}
/// Looming Shade — {2}{B} 2/2 Shade. "{B}: +1/+1 until end of turn."
pub fn looming_shade() -> CardDefinition {
    let mut c = body("Looming Shade", &[generic(2), b()], vec![CreatureType::Shade], 2, 2, vec![]);
    c.activated_abilities = vec![pump_nn(&[b()], 1)];
    c
}

// Red
/// Mons's Goblin Raiders — {R} 1/1 Goblin.
pub fn mons_goblin_raiders() -> CardDefinition {
    body("Mons's Goblin Raiders", &[r()], vec![CreatureType::Goblin], 1, 1, vec![])
}
/// Raging Goblin — {R} 1/1 Goblin with haste.
pub fn raging_goblin() -> CardDefinition {
    body("Raging Goblin", &[r()], vec![CreatureType::Goblin], 1, 1, vec![Keyword::Haste])
}
/// Goblin Piker — {1}{R} 2/1 Goblin.
pub fn goblin_piker() -> CardDefinition {
    body("Goblin Piker", &[generic(1), r()], vec![CreatureType::Goblin], 2, 1, vec![])
}
/// Goblin Chariot — {1}{R} 2/2 Goblin with haste.
pub fn goblin_chariot() -> CardDefinition {
    body("Goblin Chariot", &[generic(1), r()], vec![CreatureType::Goblin], 2, 2, vec![Keyword::Haste])
}
/// Mountain Goat — {R} 1/1 Goat with mountainwalk.
pub fn mountain_goat() -> CardDefinition {
    body("Mountain Goat", &[r()], vec![CreatureType::Goat], 1, 1, vec![Keyword::Landwalk(LandType::Mountain)])
}
/// Dragon Hatchling — {1}{R}{R} 0/1 Dragon with flying. "{R}: +1/+0 EOT."
pub fn dragon_hatchling() -> CardDefinition {
    let mut c = body("Dragon Hatchling", &[generic(1), r(), r()], vec![CreatureType::Dragon], 0, 1, vec![Keyword::Flying]);
    c.activated_abilities = vec![pump_one_zero(&[r()])];
    c
}

// Green
/// Panther Warriors — {3}{G} 6/1 Cat Warrior.
pub fn panther_warriors() -> CardDefinition {
    body("Panther Warriors", &[generic(3), g()], vec![CreatureType::Cat, CreatureType::Warrior], 6, 1, vec![])
}
/// Redwood Treefolk — {4}{G} 3/6 Treefolk.
pub fn redwood_treefolk() -> CardDefinition {
    body("Redwood Treefolk", &[generic(4), g()], vec![CreatureType::Treefolk], 3, 6, vec![])
}
/// Gorilla Chieftain — {2}{G} 3/3 Ape. "{1}{G}: Regenerate this creature."
pub fn gorilla_chieftain() -> CardDefinition {
    let mut c = body("Gorilla Chieftain", &[generic(2), g()], vec![CreatureType::Ape], 3, 3, vec![]);
    c.activated_abilities = vec![ActivatedAbility {
        mana_cost: cost(&[generic(1), g()]),
        effect: Effect::Regenerate { what: Selector::This },
        ..Default::default()
    }];
    c
}
