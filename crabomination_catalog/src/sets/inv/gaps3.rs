//! Invasion (INV) gap-closing wave 3: the Apprentice/Master cycles, the Auras,
//! the legends and the utility shell. Tests in `classic_sets/inv_gaps3`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest,
    shortcut::{draw, etb, on_dies, target_filtered},
};
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
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Sorcery],
        effect,
        ..Default::default()
    }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn aura(name: &'static str, c: ManaCost, enchant: R) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchant) },
        ..enchantment(name, c)
    }
}

fn host() -> Selector {
    Selector::attached_to(Selector::This)
}

/// A `{cost}`: pump/grant ability aimed at a filtered creature.
fn tap_ability(mana: ManaCost, effect: Effect) -> ActivatedAbility {
    ActivatedAbility { mana_cost: mana, tap_cost: true, effect, ..Default::default() }
}

fn wizard(name: &'static str, c: ManaCost, p: i32, t: i32) -> CardDefinition {
    creature(name, c, vec![CreatureType::Human, CreatureType::Wizard], p, t)
}

// ── The Apprentice / Master cycles ──────────────────────────────────────────

fn pump_target(power: i32, toughness: i32) -> Effect {
    Effect::PumpPT {
        what: target_filtered(R::Creature),
        power: Value::Const(power),
        toughness: Value::Const(toughness),
        duration: Duration::EndOfTurn,
    }
}

fn grant_target(keyword: Keyword) -> Effect {
    Effect::GrantKeyword {
        what: target_filtered(R::Creature),
        keyword,
        duration: Duration::EndOfTurn,
    }
}

fn team_pump(power: i32, toughness: i32) -> Effect {
    Effect::PumpPT {
        what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
        power: Value::Const(power),
        toughness: Value::Const(toughness),
        duration: Duration::EndOfTurn,
    }
}

fn drain_two() -> Effect {
    Effect::Drain {
        from: Selector::Player(PlayerRef::Target(0)),
        to: Selector::You,
        amount: Value::Const(2),
    }
}

fn tuck_own_creature() -> Effect {
    Effect::Move {
        what: target_filtered(R::Creature.and(R::ControlledByYou)),
        to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: crate::effect::LibraryPosition::Top },
    }
}

/// Sunscape Apprentice — {W} 1/1. {G}: pump; {U}: tuck your own creature.
pub fn sunscape_apprentice() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            tap_ability(cost(&[g()]), pump_target(1, 1)),
            tap_ability(cost(&[u()]), tuck_own_creature()),
        ],
        ..wizard("Sunscape Apprentice", cost(&[w()]), 1, 1)
    }
}

/// Sunscape Master — {2}{W}{W} 2/2. {G}{G}: team pump; {U}{U}: bounce.
pub fn sunscape_master() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            tap_ability(cost(&[g(), g()]), team_pump(2, 2)),
            tap_ability(
                cost(&[u(), u()]),
                Effect::Move {
                    what: target_filtered(R::Creature),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
            ),
        ],
        ..wizard("Sunscape Master", cost(&[generic(2), w(), w()]), 2, 2)
    }
}

/// Nightscape Apprentice — {B} 1/1. {U}: tuck yours; {R}: grant first strike.
pub fn nightscape_apprentice() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            tap_ability(cost(&[u()]), tuck_own_creature()),
            tap_ability(cost(&[r()]), grant_target(Keyword::FirstStrike)),
        ],
        ..creature(
            "Nightscape Apprentice",
            cost(&[b()]),
            vec![CreatureType::Zombie, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Nightscape Master — {2}{B}{B} 2/2. {U}{U}: bounce; {R}{R}: 2 damage.
pub fn nightscape_master() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            tap_ability(
                cost(&[u(), u()]),
                Effect::Move {
                    what: target_filtered(R::Creature),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
            ),
            tap_ability(
                cost(&[r(), r()]),
                Effect::DealDamage {
                    to: target_filtered(R::Creature),
                    amount: Value::Const(2),
                },
            ),
        ],
        ..creature(
            "Nightscape Master",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Zombie, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Thornscape Apprentice — {G} 1/1. {R}: first strike; {W}: tap a creature.
pub fn thornscape_apprentice() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            tap_ability(cost(&[r()]), grant_target(Keyword::FirstStrike)),
            tap_ability(cost(&[w()]), Effect::Tap { what: target_filtered(R::Creature) }),
        ],
        ..wizard("Thornscape Apprentice", cost(&[g()]), 1, 1)
    }
}

/// Thornscape Master — {2}{G}{G} 2/2. {R}{R}: 2 damage; {W}{W}: protection.
pub fn thornscape_master() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            tap_ability(
                cost(&[r(), r()]),
                Effect::DealDamage {
                    to: target_filtered(R::Creature),
                    amount: Value::Const(2),
                },
            ),
            tap_ability(cost(&[w(), w()]), grant_chosen_protection()),
        ],
        ..wizard("Thornscape Master", cost(&[generic(2), g(), g()]), 2, 2)
    }
}

/// Thunderscape Apprentice — {R} 1/1. {B}: drain 1; {G}: pump.
pub fn thunderscape_apprentice() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            tap_ability(
                cost(&[b()]),
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::ONE,
                },
            ),
            tap_ability(cost(&[g()]), pump_target(1, 1)),
        ],
        ..wizard("Thunderscape Apprentice", cost(&[r()]), 1, 1)
    }
}

/// Thunderscape Master — {2}{R}{R} 2/2. {B}{B}: drain 2; {G}{G}: team pump.
pub fn thunderscape_master() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            tap_ability(cost(&[b(), b()]), drain_two()),
            tap_ability(cost(&[g(), g()]), team_pump(2, 2)),
        ],
        ..wizard("Thunderscape Master", cost(&[generic(2), r(), r()]), 2, 2)
    }
}

/// Stormscape Master — {2}{U}{U} 2/2. {W}{W}: protection; {B}{B}: drain 2.
pub fn stormscape_master() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            tap_ability(cost(&[w(), w()]), grant_chosen_protection()),
            tap_ability(cost(&[b(), b()]), drain_two()),
        ],
        ..wizard("Stormscape Master", cost(&[generic(2), u(), u()]), 2, 2)
    }
}

/// "Target creature gains protection from the color of your choice until end
/// of turn."
fn grant_chosen_protection() -> Effect {
    Effect::GrantProtectionFromChosenColor {
        what: target_filtered(R::Creature),
        duration: Duration::EndOfTurn,
    }
}

/// Armored Guardian — {3}{W}{U} 2/5 that hands out protection or takes shroud.
pub fn armored_guardian() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), w(), w()]),
                effect: Effect::GrantProtectionFromChosenColor {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u(), u()]),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Shroud,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Armored Guardian",
            cost(&[generic(3), w(), u()]),
            vec![CreatureType::Cat, CreatureType::Soldier],
            2,
            5,
        )
    }
}

// ── Colour-restricted pump wizards ──────────────────────────────────────────

fn weaver(
    name: &'static str,
    c: ManaCost,
    p: i32,
    t: i32,
    mana: ManaCost,
    colors: (Color, Color),
    effect_for: fn(R) -> Effect,
) -> CardDefinition {
    let filter = R::Creature.and(R::HasColor(colors.0).or(R::HasColor(colors.1)));
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: mana,
            effect: effect_for(filter),
            ..Default::default()
        }],
        ..wizard(name, c, p, t)
    }
}

fn grant_trample(filter: R) -> Effect {
    Effect::GrantKeyword {
        what: target_filtered(filter),
        keyword: Keyword::Trample,
        duration: Duration::EndOfTurn,
    }
}

fn grant_haste(filter: R) -> Effect {
    Effect::GrantKeyword {
        what: target_filtered(filter),
        keyword: Keyword::Haste,
        duration: Duration::EndOfTurn,
    }
}

fn grant_flying(filter: R) -> Effect {
    Effect::GrantKeyword {
        what: target_filtered(filter),
        keyword: Keyword::Flying,
        duration: Duration::EndOfTurn,
    }
}

fn toughness_pump(filter: R) -> Effect {
    Effect::PumpPT {
        what: target_filtered(filter),
        power: Value::ZERO,
        toughness: Value::ONE,
        duration: Duration::EndOfTurn,
    }
}

/// Might Weaver — {1}{G} 2/1. {2}: trample for a red or white creature.
pub fn might_weaver() -> CardDefinition {
    weaver(
        "Might Weaver",
        cost(&[generic(1), g()]),
        2,
        1,
        cost(&[generic(2)]),
        (Color::Red, Color::White),
        grant_trample,
    )
}

/// Rage Weaver — {1}{R} 2/1. {2}: haste for a black or green creature.
pub fn rage_weaver() -> CardDefinition {
    weaver(
        "Rage Weaver",
        cost(&[generic(1), r()]),
        2,
        1,
        cost(&[generic(2)]),
        (Color::Black, Color::Green),
        grant_haste,
    )
}

/// Sky Weaver — {1}{U} 2/1. {2}: flying for a white or black creature.
pub fn sky_weaver() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Metathran, CreatureType::Wizard],
            ..Default::default()
        },
        ..weaver(
            "Sky Weaver",
            cost(&[generic(1), u()]),
            2,
            1,
            cost(&[generic(2)]),
            (Color::White, Color::Black),
            grant_flying,
        )
    }
}

/// Spirit Weaver — {1}{W} 2/1. {2}: +0/+1 for a green or blue creature.
pub fn spirit_weaver() -> CardDefinition {
    weaver(
        "Spirit Weaver",
        cost(&[generic(1), w()]),
        2,
        1,
        cost(&[generic(2)]),
        (Color::Green, Color::Blue),
        toughness_pump,
    )
}

// ── Self-pumping / colour-shifting bodies ───────────────────────────────────

fn self_pump(mana: ManaCost, power: i32, toughness: i32) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(power),
            toughness: Value::Const(toughness),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

fn become_chosen_color(mana: ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        effect: Effect::BecomeChosenColor { what: Selector::This, duration: Duration::EndOfTurn },
        ..Default::default()
    }
}

/// Llanowar Cavalry — {2}{G} 1/4. {W}: vigilance.
pub fn llanowar_cavalry() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Llanowar Cavalry",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            4,
        )
    }
}

/// Llanowar Vanguard — {2}{G} 1/1. {T}: +0/+4.
pub fn llanowar_vanguard() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ZERO,
                toughness: Value::Const(4),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Llanowar Vanguard", cost(&[generic(2), g()]), vec![CreatureType::Dryad], 1, 1)
    }
}

/// Metathran Zombie — {1}{U} 1/1 that regenerates for {B}.
pub fn metathran_zombie() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature(
            "Metathran Zombie",
            cost(&[generic(1), u()]),
            vec![CreatureType::Metathran, CreatureType::Zombie],
            1,
            1,
        )
    }
}

/// Noble Panther — {1}{G}{W} 3/3. {1}: first strike.
pub fn noble_panther() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Noble Panther", cost(&[generic(1), g(), w()]), vec![CreatureType::Cat], 3, 3)
    }
}

/// Phyrexian Battleflies — {B} 0/1 flier. {B}: +1/+0, twice a turn.
pub fn phyrexian_battleflies() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        // "Activate no more than twice each turn" collapses to the engine's
        // once-per-turn gate on a second copy of the same ability.
        activated_abilities: vec![
            self_pump(cost(&[b()]), 1, 0),
            ActivatedAbility { once_per_turn: true, ..self_pump(cost(&[b()]), 1, 0) },
        ],
        ..creature(
            "Phyrexian Battleflies",
            cost(&[b()]),
            vec![CreatureType::Phyrexian, CreatureType::Insect],
            0,
            1,
        )
    }
}

/// Rainbow Crow — {3}{U} 2/2 flier. {1}: becomes the colour of your choice.
pub fn rainbow_crow() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![become_chosen_color(cost(&[generic(1)]))],
        ..creature("Rainbow Crow", cost(&[generic(3), u()]), vec![CreatureType::Bird], 2, 2)
    }
}

/// Kavu Chameleon — {3}{G}{G} 4/4 that can't be countered. {G}: colour shift.
pub fn kavu_chameleon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeCountered],
        activated_abilities: vec![become_chosen_color(cost(&[g()]))],
        ..creature(
            "Kavu Chameleon",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Kavu],
            4,
            4,
        )
    }
}

/// Tidal Visionary — {U} 1/1. {T}: recolour a creature.
pub fn tidal_visionary() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::BecomeChosenColor {
                what: target_filtered(R::Creature),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Tidal Visionary",
            cost(&[u()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Blurred Mongoose — {1}{G} 2/1 with shroud that can't be countered.
pub fn blurred_mongoose() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeCountered, Keyword::Shroud],
        ..creature("Blurred Mongoose", cost(&[generic(1), g()]), vec![CreatureType::Mongoose], 2, 1)
    }
}

/// Serpentine Kavu — {4}{G} 4/4. {R}: haste.
pub fn serpentine_kavu() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Serpentine Kavu", cost(&[generic(4), g()]), vec![CreatureType::Kavu], 4, 4)
    }
}

/// Viashino Grappler — {2}{R} 3/1. {G}: trample.
pub fn viashino_grappler() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Viashino Grappler", cost(&[generic(2), r()]), vec![CreatureType::Lizard], 3, 1)
    }
}

/// Raging Kavu — {1}{R}{G} 3/1 with flash and haste.
pub fn raging_kavu() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Haste],
        ..creature("Raging Kavu", cost(&[generic(1), r(), g()]), vec![CreatureType::Kavu], 3, 1)
    }
}

/// Urborg Drake — {1}{U}{B} 2/3 flier that must attack.
pub fn urborg_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::MustAttack],
        ..creature("Urborg Drake", cost(&[generic(1), u(), b()]), vec![CreatureType::Drake], 2, 3)
    }
}

/// Slinking Serpent — {2}{U}{B} 2/3 forestwalker.
pub fn slinking_serpent() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Forest)],
        ..creature(
            "Slinking Serpent",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::Serpent],
            2,
            3,
        )
    }
}

/// Scarred Puma — {R} 2/1 that needs a black or green partner in the attack.
pub fn scarred_puma() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CanAttackOnlyIfYouControl(Box::new(
            R::Creature
                .and(R::IsAttacking)
                .and(R::OtherThanSource)
                .and(R::HasColor(Color::Black).or(R::HasColor(Color::Green))),
        ))],
        ..creature("Scarred Puma", cost(&[r()]), vec![CreatureType::Cat], 2, 1)
    }
}

/// Urborg Phantom — {2}{B} 3/1 that can't block and can shrug off combat.
pub fn urborg_phantom() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBlock],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::Seq(vec![
                Effect::PreventCombatDamageToTargetThisTurn { target: Selector::This },
                Effect::PreventCombatDamageByTargetThisTurn { target: Selector::This },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Urborg Phantom",
            cost(&[generic(2), b()]),
            vec![CreatureType::Spirit, CreatureType::Minion],
            3,
            1,
        )
    }
}

/// Skittish Kavu — {1}{R} 1/1 that swells while the board stays off-colour.
pub fn skittish_kavu() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+1 as long as no opponent controls a white or \
                          blue creature.",
            effect: StaticEffect::PumpSelfIf {
                condition: no_opposing_white_or_blue(),
                power: 1,
                toughness: 1,
                keywords: vec![],
            },
        }],
        ..creature("Skittish Kavu", cost(&[generic(1), r()]), vec![CreatureType::Kavu], 1, 1)
    }
}

/// Kavu Runner — {3}{R} 3/3 that gets haste while the board stays off-colour.
pub fn kavu_runner() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature has haste as long as no opponent controls a white or \
                          blue creature.",
            effect: StaticEffect::PumpSelfIf {
                condition: no_opposing_white_or_blue(),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Haste],
            },
        }],
        ..creature("Kavu Runner", cost(&[generic(3), r()]), vec![CreatureType::Kavu], 3, 3)
    }
}

fn no_opposing_white_or_blue() -> Predicate {
    Predicate::Not(Box::new(Predicate::SelectorExists(Selector::EachPermanent(
        R::Creature
            .and(R::ControlledByOpponent)
            .and(R::HasColor(Color::White).or(R::HasColor(Color::Blue))),
    ))))
}

// ── Auras ───────────────────────────────────────────────────────────────────

/// Wings of Hope — {W}{U}. +1/+3 and flying.
pub fn wings_of_hope() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted creature gets +1/+3.",
                effect: StaticEffect::PumpPT { applies_to: host(), power: 1, toughness: 3 },
            },
            StaticAbility {
                description: "Enchanted creature has flying.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: host(),
                    keyword: Keyword::Flying,
                },
            },
        ],
        ..aura("Wings of Hope", cost(&[w(), u()]), R::Creature)
    }
}

/// Whip Silk — {G}. Reach, and it rebuys itself for {G}.
pub fn whip_silk() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has reach.",
            effect: StaticEffect::GrantKeyword { applies_to: host(), keyword: Keyword::Reach },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..aura("Whip Silk", cost(&[g()]), R::Creature)
    }
}

/// Mourning — {1}{B}. -2/-0, and it rebuys itself for {B}.
pub fn mourning() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature gets -2/-0.",
            effect: StaticEffect::PumpPT { applies_to: host(), power: -2, toughness: 0 },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..aura("Mourning", cost(&[generic(1), b()]), R::Creature)
    }
}

/// Scavenged Weaponry — {2}{B}. A cantripping +1/+1.
pub fn scavenged_weaponry() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(draw(1))],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature gets +1/+1.",
            effect: StaticEffect::PumpPT { applies_to: host(), power: 1, toughness: 1 },
        }],
        ..aura("Scavenged Weaponry", cost(&[generic(2), b()]), R::Creature)
    }
}

/// Sleeper's Robe — {U}{B}. Fear, and a may-draw on connect.
pub fn sleepers_robe() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has fear.",
            effect: StaticEffect::GrantKeyword { applies_to: host(), keyword: Keyword::Fear },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::EnchantedBySource),
            effect: Effect::MayDo {
                description: "Draw a card?".into(),
                body: Box::new(draw(1)),
            },
        }],
        ..aura("Sleeper's Robe", cost(&[u(), b()]), R::Creature)
    }
}

/// Tainted Well — {2}{B}. A cantrip that turns a land into a Swamp.
pub fn tainted_well() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(draw(1))],
        static_abilities: vec![StaticAbility {
            description: "Enchanted land is a Swamp.",
            effect: StaticEffect::LandTypeChanger {
                applies_to: host(),
                land_type: LandType::Swamp,
                replace: true,
            },
        }],
        ..aura("Tainted Well", cost(&[generic(2), b()]), R::Land)
    }
}

// ── Legends ─────────────────────────────────────────────────────────────────

fn legend(mut def: CardDefinition) -> CardDefinition {
    def.supertypes = vec![Supertype::Legendary];
    def
}

/// Captain Sisay — {2}{G}{W} 2/2. {T}: tutor a legendary card to hand.
pub fn captain_sisay() -> CardDefinition {
    legend(CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::HasSupertype(Supertype::Legendary),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..creature(
            "Captain Sisay",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            2,
        )
    })
}

/// Hanna, Ship's Navigator — {1}{W}{U} 1/2. Rebuys artifacts and enchantments.
pub fn hanna_ships_navigator() -> CardDefinition {
    legend(CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w(), u()]),
            tap_cost: true,
            effect: Effect::Move {
                what: target_filtered(
                    R::InYourGraveyard.and(R::Artifact.or(R::Enchantment)),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..creature(
            "Hanna, Ship's Navigator",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            1,
            2,
        )
    })
}

/// Reya Dawnbringer — {6}{W}{W}{W} 4/6 flier that reanimates every upkeep.
pub fn reya_dawnbringer() -> CardDefinition {
    legend(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::MayDo {
                description: "Return a creature card from your graveyard?".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
        }],
        ..creature(
            "Reya Dawnbringer",
            cost(&[generic(6), w(), w(), w()]),
            vec![CreatureType::Angel],
            4,
            6,
        )
    })
}

/// Empress Galina — {3}{U}{U} 1/3 that steals legends.
pub fn empress_galina() -> CardDefinition {
    legend(CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), u()]),
            tap_cost: true,
            effect: Effect::GainControl {
                what: target_filtered(R::Permanent.and(R::HasSupertype(Supertype::Legendary))),
                to: None,
                duration: Duration::Permanent,
            },
            ..Default::default()
        }],
        ..creature(
            "Empress Galina",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Noble],
            1,
            3,
        )
    })
}

/// Tsabo Tavoc — {5}{B}{R} 7/4 first striker that hunts other legends.
pub fn tsabo_tavoc() -> CardDefinition {
    legend(CardDefinition {
        keywords: vec![
            Keyword::FirstStrike,
            Keyword::ProtectionFromMatching(Box::new(
                R::Creature.and(R::HasSupertype(Supertype::Legendary)),
            )),
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), b()]),
            tap_cost: true,
            effect: Effect::DestroyNoRegen {
                what: target_filtered(R::Creature.and(R::HasSupertype(Supertype::Legendary))),
            },
            ..Default::default()
        }],
        ..creature(
            "Tsabo Tavoc",
            cost(&[generic(5), b(), r()]),
            vec![CreatureType::Phyrexian, CreatureType::Horror],
            7,
            4,
        )
    })
}

/// Blind Seer — {2}{U}{U} 3/3 that recolours a spell or permanent.
pub fn blind_seer() -> CardDefinition {
    legend(CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::BecomeChosenColor {
                what: target_filtered(R::Permanent.or(R::IsSpellOnStack)),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..wizard("Blind Seer", cost(&[generic(2), u(), u()]), 3, 3)
    })
}

// ── Utility permanents ──────────────────────────────────────────────────────

/// Keldon Necropolis — a Legendary land that eats creatures for damage.
pub fn keldon_necropolis() -> CardDefinition {
    legend(CardDefinition {
        name: "Keldon Necropolis",
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
                mana_cost: cost(&[generic(4), r()]),
                tap_cost: true,
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::DealDamage {
                    to: crate::effect::shortcut::target_any(),
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    })
}

/// Reckless Assault — {2}{B}{R}. Pay life for reach.
pub fn reckless_assault() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            life_cost: 2,
            effect: Effect::DealDamage {
                to: crate::effect::shortcut::target_any(),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..enchantment("Reckless Assault", cost(&[generic(2), b(), r()]))
    }
}

/// Meteor Storm — {R}{G}. Discard two at random for four damage.
pub fn meteor_storm() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r(), g()]),
            discard_cost: Some((R::Any, 2)),
            effect: Effect::DealDamage {
                to: crate::effect::shortcut::target_any(),
                amount: Value::Const(4),
            },
            ..Default::default()
        }],
        ..enchantment("Meteor Storm", cost(&[r(), g()]))
    }
}

/// Shivan Harvest — {1}{R}. Feed it creatures to break nonbasic lands.
pub fn shivan_harvest() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Destroy { what: target_filtered(R::IsNonbasicLand) },
            ..Default::default()
        }],
        ..enchantment("Shivan Harvest", cost(&[generic(1), r()]))
    }
}

/// Smoldering Tar — {2}{B}{R}. A slow drain you can cash in for four damage.
pub fn smoldering_tar() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Const(4),
            },
            ..Default::default()
        }],
        ..enchantment("Smoldering Tar", cost(&[generic(2), b(), r()]))
    }
}

/// Stalking Assassin — {1}{U}{B} 1/1. Taps things, then kills what's tapped.
pub fn stalking_assassin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3), u()]),
                tap_cost: true,
                effect: Effect::Tap { what: target_filtered(R::Creature) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), b()]),
                tap_cost: true,
                effect: Effect::Destroy {
                    what: target_filtered(R::Creature.and(R::Tapped)),
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Stalking Assassin",
            cost(&[generic(1), u(), b()]),
            vec![CreatureType::Human, CreatureType::Assassin],
            1,
            1,
        )
    }
}

/// Trench Wurm — {3}{B} 3/3. {2}{R}, {T}: break a nonbasic land.
pub fn trench_wurm() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            tap_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::IsNonbasicLand) },
            ..Default::default()
        }],
        ..creature("Trench Wurm", cost(&[generic(3), b()]), vec![CreatureType::Wurm], 3, 3)
    }
}

/// Samite Archer — {1}{W}{U} 1/1. Shields or pings, one tap each.
pub fn samite_archer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::PreventNextDamage {
                    target: crate::effect::shortcut::target_any(),
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::DealDamage {
                    to: crate::effect::shortcut::target_any(),
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Samite Archer",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Human, CreatureType::Cleric, CreatureType::Archer],
            1,
            1,
        )
    }
}

/// Treefolk Healer — {4}{G} 2/3. {2}{W}, {T}: a 2-point shield.
pub fn treefolk_healer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            tap_cost: true,
            effect: Effect::PreventNextDamage {
                target: crate::effect::shortcut::target_any(),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..creature(
            "Treefolk Healer",
            cost(&[generic(4), g()]),
            vec![CreatureType::Treefolk, CreatureType::Cleric],
            2,
            3,
        )
    }
}

/// Planar Portal — {6}. {6}, {T}: tutor anything.
pub fn planar_portal() -> CardDefinition {
    CardDefinition {
        name: "Planar Portal",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            tap_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::Any,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tsabo's Web — {2}. A cantrip that locks down utility lands.
pub fn tsabos_web() -> CardDefinition {
    CardDefinition {
        name: "Tsabo's Web",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(draw(1))],
        static_abilities: vec![StaticAbility {
            description: "Each land with an activated ability that isn't a mana ability doesn't \
                          untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::EachPermanent(R::Land.and(R::HasNonManaActivatedAbility)),
            },
        }],
        ..Default::default()
    }
}

/// Dueling Grounds — {1}{G}{W}. One attacker and one blocker each combat.
pub fn dueling_grounds() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "No more than one creature can attack each combat.",
                effect: StaticEffect::MaxAttackersPerCombat(1),
            },
            StaticAbility {
                description: "No more than one creature can block each combat.",
                effect: StaticEffect::MaxBlockersPerCombat(1),
            },
        ],
        ..enchantment("Dueling Grounds", cost(&[generic(1), g(), w()]))
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Restock — {3}{G}{G}. Rebuy two, then exile itself.
pub fn restock() -> CardDefinition {
    CardDefinition {
        exile_on_resolve: true,
        ..sorcery(
            "Restock",
            cost(&[generic(3), g(), g()]),
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 2,
                filter: R::InYourGraveyard,
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
            },
        )
    }
}

/// Scouting Trek — {1}{G}. Stack any number of basics on top.
pub fn scouting_trek() -> CardDefinition {
    sorcery(
        "Scouting Trek",
        cost(&[generic(1), g()]),
        Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::IsBasicLand,
            to: ZoneDest::Library {
                who: PlayerRef::You,
                pos: crate::effect::LibraryPosition::Top,
            },
            count: Value::Const(5),
        },
    )
}

/// Sway of Illusion — {1}{U}. Recolour any number of creatures, then draw.
pub fn sway_of_illusion() -> CardDefinition {
    instant(
        "Sway of Illusion",
        cost(&[generic(1), u()]),
        Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 4,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::BecomeChosenColor {
                    what: Selector::Target(0),
                    duration: Duration::EndOfTurn,
                }),
            },
            draw(1),
        ]),
    )
}

/// Obliterate — {6}{R}{R}. An uncounterable board wipe that spares nothing.
pub fn obliterate() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeCountered],
        ..sorcery(
            "Obliterate",
            cost(&[generic(6), r(), r()]),
            Effect::DestroyNoRegen {
                what: Selector::EachPermanent(R::Artifact.or(R::Creature).or(R::Land)),
            },
        )
    }
}

/// Twilight's Call — {4}{B}{B}. Everyone's graveyard creatures come back.
pub fn twilights_call() -> CardDefinition {
    CardDefinition {
        flash_surcharge: Some(cost(&[generic(2)])),
        ..sorcery(
        "Twilight's Call",
        cost(&[generic(4), b(), b()]),
        Effect::Move {
            what: Selector::CardsInZone {
                who: PlayerRef::EachPlayer,
                zone: crate::card::Zone::Graveyard,
                filter: R::Creature,
            },
            to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved, tapped: false },
        },
    )
    }
}

/// Winnow — {1}{W}. Kill a permanent whose name is already on the board twice.
pub fn winnow() -> CardDefinition {
    instant(
        "Winnow",
        cost(&[generic(1), w()]),
        Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Nonland.and(R::SharesNameWithAnotherPermanent)),
            },
            draw(1),
        ]),
    )
}

/// Searing Rays — {2}{R}. Damage each player per creature of a chosen colour.
pub fn searing_rays() -> CardDefinition {
    sorcery(
        "Searing Rays",
        cost(&[generic(2), r()]),
        Effect::Seq(vec![
            Effect::ChooseColorForSelf,
            Effect::DealDamageToEachPlayerPerPermanent {
                filter: R::Creature.and(R::HasChosenColorOfSource),
                amount: Value::ONE,
                flat: false,
            },
        ]),
    )
}

/// Rampant Elephant — {3}{W} 2/2. {G}: force a block.
pub fn rampant_elephant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            effect: Effect::BecomeBlocked { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..creature("Rampant Elephant", cost(&[generic(3), w()]), vec![CreatureType::Elephant], 2, 2)
    }
}

/// Rooting Kavu — {2}{G}{G} 4/3 that may exile itself to refill the library.
pub fn rooting_kavu() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::MayDo {
            description: "Exile this and shuffle your creature cards back in?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Exile { what: Selector::This },
                Effect::ShuffleFilteredGraveyardIntoLibrary {
                    who: PlayerRef::You,
                    filter: R::Creature,
                },
            ])),
        })],
        ..creature("Rooting Kavu", cost(&[generic(2), g(), g()]), vec![CreatureType::Kavu], 4, 3)
    }
}

/// Plague Spitter — {2}{B} 2/2 that pings the whole table, twice.
pub fn plague_spitter() -> CardDefinition {
    let sweep = || Effect::Seq(vec![
        Effect::DealDamage { to: Selector::EachPermanent(R::Creature), amount: Value::ONE },
        Effect::DealDamage { to: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE },
    ]);
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: sweep(),
            },
            on_dies(sweep()),
        ],
        ..creature(
            "Plague Spitter",
            cost(&[generic(2), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Horror],
            2,
            2,
        )
    }
}
