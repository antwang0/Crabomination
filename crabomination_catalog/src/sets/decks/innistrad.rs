//! Innistrad: Midnight Hunt (MID) / Crimson Vow (VOW) commons & uncommons that
//! round out the modern-era pool. Mechanics in play: coven, decayed, exploit,
//! Blood tokens, day/night, training, flashback. Each card has at least one
//! functionality test in `crabomination/src/tests/innistrad.rs`.

use crate::card::{
    ActivatedAbility, AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, Effect, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    Predicate, Selector, SelectionRequirement, Subtypes, TokenDefinition, TriggeredAbility, Value,
    WardCost,
};
use crate::effect::shortcut::{etb, magecraft_self_pump, on_attack, on_dies, on_other_dies, target_filtered};
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

// ── MID/VOW Daybound werewolf DFCs, disturb spirits, and supporting spells ───
// Werewolf fronts carry `Keyword::Daybound`; the engine transforms them to
// their `Keyword::Nightbound` back face when it becomes night (CR 702.146).
// Disturb cards (`Keyword::Disturb`) cast their back face from the graveyard;
// when the back is an Aura it's cast targeting a creature (engine threads the
// target through `GameAction::CastDisturb`).

/// Helper: a Daybound/Nightbound werewolf DFC built from front/back specs that
/// differ only in stats + keywords + triggers (the common MID/VOW shape).
#[allow(clippy::too_many_arguments)] // a card-shape builder; each arg is a printed face stat
fn werewolf_dfc(
    front_name: &'static str,
    front_cost: crate::mana::ManaCost,
    front_types: Vec<CreatureType>,
    front_pt: (i32, i32),
    front_kw: Vec<Keyword>,
    front_triggers: Vec<TriggeredAbility>,
    back_name: &'static str,
    back_pt: (i32, i32),
    back_kw: Vec<Keyword>,
    back_triggers: Vec<TriggeredAbility>,
) -> CardDefinition {
    let mut back_keywords = back_kw;
    back_keywords.push(Keyword::Nightbound);
    let back = CardDefinition {
        name: back_name,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Werewolf], ..Default::default() },
        power: back_pt.0,
        toughness: back_pt.1,
        keywords: back_keywords,
        triggered_abilities: back_triggers,
        ..Default::default()
    };
    let mut front_keywords = front_kw;
    front_keywords.push(Keyword::Daybound);
    CardDefinition {
        name: front_name,
        cost: front_cost,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: front_types, ..Default::default() },
        power: front_pt.0,
        toughness: front_pt.1,
        keywords: front_keywords,
        triggered_abilities: front_triggers,
        back_face: Some(Box::new(back)),
        ..Default::default()
    }
}

/// Helper: a disturb DFC whose back face is an Aura granting `bonus` to the
/// enchanted creature (`Effect::Attach` provides the enchant target slot).
fn disturb_aura_dfc(
    front: CardDefinition,
    disturb_cost: crate::mana::ManaCost,
    back_name: &'static str,
    bonus: EquipBonus,
) -> CardDefinition {
    let aura = CardDefinition {
        name: back_name,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(bonus),
        ..Default::default()
    };
    let mut front = front;
    front.keywords.push(Keyword::Disturb(disturb_cost));
    front.back_face = Some(Box::new(aura));
    front
}

/// Helper: "Whenever this creature deals combat damage to a player, `effect`."
fn on_combat_damage_to_player(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect,
    }
}

/// Suspicious Stowaway // Seafaring Werewolf — {1}{U} 1/1 unblockable Human
/// Rogue Werewolf. Combat damage → loot. Back: 2/1, combat damage → draw.
pub fn suspicious_stowaway() -> CardDefinition {
    werewolf_dfc(
        "Suspicious Stowaway",
        cost(&[generic(1), u()]),
        vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Werewolf],
        (1, 1),
        vec![Keyword::Unblockable],
        vec![on_combat_damage_to_player(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
        ]))],
        "Seafaring Werewolf",
        (2, 1),
        vec![Keyword::Unblockable],
        vec![on_combat_damage_to_player(Effect::Draw { who: Selector::You, amount: Value::Const(1) })],
    )
}

/// Tireless Hauler // Dire-Strain Brawler — {4}{G} 4/5 vigilant Werewolf.
/// Back: 6/6 vigilance.
pub fn tireless_hauler() -> CardDefinition {
    werewolf_dfc(
        "Tireless Hauler",
        cost(&[generic(4), g()]),
        vec![CreatureType::Human, CreatureType::Werewolf],
        (4, 5),
        vec![Keyword::Vigilance],
        vec![],
        "Dire-Strain Brawler",
        (6, 6),
        vec![Keyword::Vigilance],
        vec![],
    )
}

/// Bird Admirer // Wing Shredder — {2}{G} 1/4 reach Werewolf. Back: 3/5 reach.
pub fn bird_admirer() -> CardDefinition {
    werewolf_dfc(
        "Bird Admirer",
        cost(&[generic(2), g()]),
        vec![CreatureType::Human, CreatureType::Archer, CreatureType::Werewolf],
        (1, 4),
        vec![Keyword::Reach],
        vec![],
        "Wing Shredder",
        (3, 5),
        vec![Keyword::Reach],
        vec![],
    )
}

/// Hookhand Mariner // Riphook Raider — {3}{G} 4/4 Werewolf. Back: 6/4 that
/// can't be blocked by creatures with power 2 or less.
pub fn hookhand_mariner() -> CardDefinition {
    werewolf_dfc(
        "Hookhand Mariner",
        cost(&[generic(3), g()]),
        vec![CreatureType::Human, CreatureType::Werewolf],
        (4, 4),
        vec![],
        vec![],
        "Riphook Raider",
        (6, 4),
        vec![Keyword::CantBeBlockedByPowerAtMost(2)],
        vec![],
    )
}

/// Fearful Villager // Fearsome Werewolf — {2}{R} 2/3 menace Werewolf.
/// Back: 4/3 menace.
pub fn fearful_villager() -> CardDefinition {
    werewolf_dfc(
        "Fearful Villager",
        cost(&[generic(2), r()]),
        vec![CreatureType::Human, CreatureType::Werewolf],
        (2, 3),
        vec![Keyword::Menace],
        vec![],
        "Fearsome Werewolf",
        (4, 3),
        vec![Keyword::Menace],
        vec![],
    )
}

/// Lambholt Raconteur // Lambholt Ravager — {3}{R} 2/4 Werewolf. Noncreature
/// cast → 1 damage to each opponent. Back: 4/4, deals 2 instead.
pub fn lambholt_raconteur() -> CardDefinition {
    let noncreature_ping = |n: i32| TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature)),
        effect: Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(n),
        },
    };
    werewolf_dfc(
        "Lambholt Raconteur",
        cost(&[generic(3), r()]),
        vec![CreatureType::Human, CreatureType::Werewolf],
        (2, 4),
        vec![],
        vec![noncreature_ping(1)],
        "Lambholt Ravager",
        (4, 4),
        vec![],
        vec![noncreature_ping(2)],
    )
}

/// Spellrune Painter // Spellrune Howler — {2}{R} 2/3 Werewolf. Instant/sorcery
/// cast → +1/+1 until EOT. Back: 3/4, +2/+2 instead.
pub fn spellrune_painter() -> CardDefinition {
    werewolf_dfc(
        "Spellrune Painter",
        cost(&[generic(2), r()]),
        vec![CreatureType::Human, CreatureType::Shaman, CreatureType::Werewolf],
        (2, 3),
        vec![],
        vec![magecraft_self_pump(1, 1)],
        "Spellrune Howler",
        (3, 4),
        vec![],
        vec![magecraft_self_pump(2, 2)],
    )
}

/// Wolfkin Outcast // Wedding Crasher — {5}{G} 5/4 Werewolf; costs {2} less if
/// you control a Wolf or Werewolf. Back: 6/5; a Wolf/Werewolf you control
/// dying → draw a card.
pub fn wolfkin_outcast() -> CardDefinition {
    let wolf_or_werewolf =
        SelectionRequirement::HasCreatureType(CreatureType::Wolf).or(SelectionRequirement::HasCreatureType(CreatureType::Werewolf));
    let mut card = werewolf_dfc(
        "Wolfkin Outcast",
        cost(&[generic(5), g()]),
        vec![CreatureType::Human, CreatureType::Werewolf],
        (5, 4),
        vec![],
        vec![],
        "Wedding Crasher",
        (6, 5),
        vec![],
        // "this creature or another Wolf/Werewolf you control dies" — modeled
        // as the "another of yours" leave-trigger (the self case is rare).
        vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: wolf_or_werewolf.clone(),
                }),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
    );
    card.self_cost_reduction_if_control = vec![(wolf_or_werewolf, 2)];
    card
}

/// Galedrifter // Waildrifter — {3}{U} 3/2 flying Hippogriff. Disturb {4}{U}
/// into a 2/2 flying Hippogriff Spirit.
pub fn galedrifter() -> CardDefinition {
    let waildrifter = CardDefinition {
        name: "Waildrifter",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Hippogriff, CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Galedrifter",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Hippogriff], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Disturb(cost(&[generic(4), u()]))],
        back_face: Some(Box::new(waildrifter)),
        ..Default::default()
    }
}

/// Kindly Ancestor // Ancestor's Embrace — {2}{W} 2/3 lifelink Spirit. Disturb
/// {1}{W} into an Aura granting lifelink.
pub fn kindly_ancestor() -> CardDefinition {
    let front = CardDefinition {
        name: "Kindly Ancestor",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        ..Default::default()
    };
    disturb_aura_dfc(
        front,
        cost(&[generic(1), w()]),
        "Ancestor's Embrace",
        EquipBonus { keywords: vec![Keyword::Lifelink], ..Default::default() },
    )
}

/// Twinblade Geist // Twinblade Invocation — {1}{W} 1/1 double strike Spirit
/// Warrior. Disturb {2}{W} into an Aura granting double strike.
pub fn twinblade_geist() -> CardDefinition {
    let front = CardDefinition {
        name: "Twinblade Geist",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::DoubleStrike],
        ..Default::default()
    };
    disturb_aura_dfc(
        front,
        cost(&[generic(2), w()]),
        "Twinblade Invocation",
        EquipBonus { keywords: vec![Keyword::DoubleStrike], ..Default::default() },
    )
}

/// Mischievous Catgeist // Catlike Curiosity — {1}{U} 1/1 Cat Spirit; combat
/// damage → draw. Disturb {2}{U} into an Aura granting that combat-damage draw.
pub fn mischievous_catgeist() -> CardDefinition {
    let front = CardDefinition {
        name: "Mischievous Catgeist",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Spirit],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![on_combat_damage_to_player(Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    };
    disturb_aura_dfc(
        front,
        cost(&[generic(2), u()]),
        "Catlike Curiosity",
        EquipBonus {
            triggered_abilities: vec![on_combat_damage_to_player(Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            })],
            ..Default::default()
        },
    )
}

/// Olivia's Midnight Ambush — {1}{B} Instant. Target creature gets -2/-2, or
/// -13/-13 if it's night.
pub fn olivias_midnight_ambush() -> CardDefinition {
    CardDefinition {
        name: "Olivia's Midnight Ambush",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::IsNight,
            then: Box::new(Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-13),
                toughness: Value::Const(-13),
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}

/// Moonrager's Slash — {2}{R} Instant; costs {2} less if it's night. Deal 3
/// damage to any target.
pub fn moonragers_slash() -> CardDefinition {
    CardDefinition {
        name: "Moonrager's Slash",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_night: Some(2),
        effect: Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Lunar Rejection — {1}{U} Instant. Return target Wolf or Werewolf to its
/// owner's hand, then draw. Cleave {3}{U} — return any creature instead.
pub fn lunar_rejection() -> CardDefinition {
    let wolf_or_werewolf =
        SelectionRequirement::HasCreatureType(CreatureType::Wolf).or(SelectionRequirement::HasCreatureType(CreatureType::Werewolf));
    let bounce_and_draw = |filter: SelectionRequirement| {
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(filter),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ])
    };
    CardDefinition {
        name: "Lunar Rejection",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: bounce_and_draw(wolf_or_werewolf),
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(3), u()]),
            effect_override: Some(bounce_and_draw(SelectionRequirement::Creature)),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Shipwreck Sifters — {1}{U} 1/2 Spirit. ETB loot. Discarding a Spirit card
/// or a card with disturb → +1/+1 counter.
pub fn shipwreck_sifters() -> CardDefinition {
    CardDefinition {
        name: "Shipwreck Sifters",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Spirit)
                            .or(SelectionRequirement::HasDisturb),
                    }),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Gryffwing Cavalry — {3}{W} 2/2 flying Knight with Training. When it attacks,
/// you may pay {1}{W}; if you do, target creature can't block this turn.
pub fn gryffwing_cavalry() -> CardDefinition {
    CardDefinition {
        name: "Gryffwing Cavalry",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::training(), on_attack(Effect::MayPay {
            description: "Pay {1}{W}: target creature can't block this turn".into(),
            mana_cost: cost(&[generic(1), w()]),
            body: Box::new(Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            }),
            else_: None,
        })],
        ..Default::default()
    }
}

// ── token helpers ──────────────────────────────────────────────────────────

fn white_human_token() -> TokenDefinition {
    TokenDefinition {
        name: "Human".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        ..Default::default()
    }
}

fn flying_bat_token() -> TokenDefinition {
    TokenDefinition {
        name: "Bat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bat], ..Default::default() },
        ..Default::default()
    }
}

fn reach_spider_token() -> TokenDefinition {
    TokenDefinition {
        name: "Spider".into(),
        power: 1,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        keywords: vec![Keyword::Reach],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        ..Default::default()
    }
}

fn boar_3_1_token() -> TokenDefinition {
    TokenDefinition {
        name: "Boar".into(),
        power: 3,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Boar], ..Default::default() },
        ..Default::default()
    }
}

fn decayed_zombie_token() -> TokenDefinition {
    TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        keywords: vec![Keyword::Decayed],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    }
}

// ── White ──────────────────────────────────────────────────────────────────

/// Unruly Mob — {1}{W} 1/1 Human. Whenever another creature you control dies,
/// put a +1/+1 counter on this creature.
pub fn unruly_mob() -> CardDefinition {
    CardDefinition {
        name: "Unruly Mob",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![on_other_dies(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Clarion Cathars — {3}{W} 3/3 Human Knight. ETB create a 1/1 white Human.
pub fn clarion_cathars() -> CardDefinition {
    CardDefinition {
        name: "Clarion Cathars",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: white_human_token(),
        })],
        ..Default::default()
    }
}

/// Homestead Courage — {W} Sorcery. Put a +1/+1 counter on target creature you
/// control. It gains vigilance until end of turn. Flashback {W}.
pub fn homestead_courage() -> CardDefinition {
    CardDefinition {
        name: "Homestead Courage",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[w()]))],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Flare of Faith — {1}{W} Instant. Target creature gets +2/+2; if it's a
/// Human, it gets +3/+3 and gains indestructible until end of turn instead.
pub fn flare_of_faith() -> CardDefinition {
    CardDefinition {
        name: "Flare of Faith",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::Target(0),
                filter: SelectionRequirement::HasCreatureType(CreatureType::Human),
            },
            then: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ])),
            else_: Box::new(Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}

/// Ritual of Hope — {1}{W} Instant. Creatures you control get +1/+1. Coven —
/// if you control three or more creatures with different powers, +2/+1 instead.
pub fn ritual_of_hope() -> CardDefinition {
    let team = Selector::EachPermanent(
        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
    );
    CardDefinition {
        name: "Ritual of Hope",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::CovenActive { who: PlayerRef::You },
            then: Box::new(Effect::PumpPT {
                what: team.clone(),
                power: Value::Const(2),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::PumpPT {
                what: team,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}

/// Sunset Revelry — {1}{W} Sorcery. Gain 4 life if an opponent has more life;
/// create two 1/1 Humans if an opponent controls more creatures; draw two cards
/// if an opponent has more cards in hand.
pub fn sunset_revelry() -> CardDefinition {
    let noop = || Box::new(Effect::Noop);
    CardDefinition {
        name: "Sunset Revelry",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::AnOpponentHasMoreLife,
                then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(4) }),
                else_: noop(),
            },
            Effect::If {
                cond: Predicate::AnOpponentControlsMoreCreatures,
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: white_human_token(),
                }),
                else_: noop(),
            },
            Effect::If {
                cond: Predicate::AnOpponentHasMoreCardsInHand,
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(2) }),
                else_: noop(),
            },
        ]),
        ..Default::default()
    }
}

/// Valorous Stance — {1}{W} Instant. Choose one — target creature gains
/// indestructible; or destroy target creature with toughness 4 or greater.
pub fn valorous_stance() -> CardDefinition {
    CardDefinition {
        name: "Valorous Stance",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ToughnessAtLeast(4)),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Sanctify — {1}{W} Sorcery. Destroy target artifact or enchantment. You gain
/// 3 life.
pub fn sanctify() -> CardDefinition {
    CardDefinition {
        name: "Sanctify",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                ),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
        ]),
        ..Default::default()
    }
}

/// Piercing Light — {W} Instant. Deals 2 damage to target attacking or blocking
/// creature. Scry 1.
pub fn piercing_light() -> CardDefinition {
    CardDefinition {
        name: "Piercing Light",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::IsAttacking.or(SelectionRequirement::IsBlocking),
                ),
                amount: Value::Const(2),
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Traveling Minister — {W} 1/1 Human Cleric. {T}: target creature gets +1/+0
/// until end of turn and you gain 1 life. Activate only as a sorcery.
pub fn traveling_minister() -> CardDefinition {
    CardDefinition {
        name: "Traveling Minister",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Resistance Squad — {2}{W} 3/2 Human Soldier. ETB, if you control another
/// Human, draw a card.
pub fn resistance_squad() -> CardDefinition {
    CardDefinition {
        name: "Resistance Squad",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::OtherThanSource
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Human))
                        .and(SelectionRequirement::ControlledByYou),
                ),
                n: Value::Const(1),
            },
            then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Loyal Gryff — {2}{W} 2/2 Hippogriff. Flash, flying. ETB you may return
/// another creature you control to its owner's hand.
pub fn loyal_gryff() -> CardDefinition {
    CardDefinition {
        name: "Loyal Gryff",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Hippogriff], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Return another creature you control to its owner's hand?".to_string(),
            body: Box::new(Effect::Move {
                what: Selector::take(
                    Selector::EachPermanent(
                        SelectionRequirement::OtherThanSource
                            .and(SelectionRequirement::Creature)
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    Value::Const(1),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..Default::default()
    }
}

/// Heron of Hope — {3}{W} 2/3 Bird. Flying. If you would gain life, you gain
/// that much plus 1 instead. {1}{W}: this creature gains lifelink until EOT.
pub fn heron_of_hope() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    use crate::effect::PlayerStaticTarget;
    CardDefinition {
        name: "Heron of Hope",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "If you would gain life, you gain that much plus 1 instead.",
            effect: StaticEffect::LifeGainBonus { target: PlayerStaticTarget::Controller, amount: 1 },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Search Party Captain — {3}{W} 2/2 Human Soldier. Costs {1} less for each
/// creature you attacked with this turn. ETB draw a card.
pub fn search_party_captain() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Search Party Captain",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Costs {1} less for each creature you attacked with this turn.",
            effect: StaticEffect::SelfCostReducedPerCreatureAttackedThisTurn { per: 1 },
        }],
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::Const(1) })],
        ..Default::default()
    }
}

// ── Blue ───────────────────────────────────────────────────────────────────

/// Larder Zombie — {U} 1/3 Zombie. Defender. Tap three untapped creatures you
/// control: Surveil 1.
pub fn larder_zombie() -> CardDefinition {
    CardDefinition {
        name: "Larder Zombie",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                3,
            )),
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Startle — {1}{U} Instant. Target creature gets -2/-0 until end of turn.
/// Create a 2/2 black Zombie with decayed. Draw a card.
pub fn startle() -> CardDefinition {
    CardDefinition {
        name: "Startle",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: decayed_zombie_token(),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Organ Hoarder — {3}{U} 3/2 Zombie. ETB look at the top three cards, put one
/// into your hand and the rest into your graveyard.
pub fn organ_hoarder() -> CardDefinition {
    CardDefinition {
        name: "Organ Hoarder",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(3),
            rest_to_graveyard: true,
            pick_filter: None,
            take: Some(Value::Const(1)),
            to_battlefield: false,
        })],
        ..Default::default()
    }
}

/// Dissipate — {1}{U}{U} Instant. Counter target spell. If it's countered this
/// way, exile it instead.
pub fn dissipate() -> CardDefinition {
    use crate::effect::CounteredSpellZone;
    CardDefinition {
        name: "Dissipate",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpellToZone {
            what: Selector::Target(0),
            zone: CounteredSpellZone::Exile,
        },
        ..Default::default()
    }
}

/// Scattered Thoughts — {3}{U} Instant. Look at the top four cards. Put two into
/// your hand and the rest into your graveyard.
pub fn scattered_thoughts() -> CardDefinition {
    CardDefinition {
        name: "Scattered Thoughts",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(4),
            rest_to_graveyard: true,
            pick_filter: None,
            take: Some(Value::Const(2)),
            to_battlefield: false,
        },
        ..Default::default()
    }
}

/// Vivisection — {3}{U} Sorcery. As an additional cost, sacrifice a creature.
/// Draw three cards.
pub fn vivisection() -> CardDefinition {
    CardDefinition {
        name: "Vivisection",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::SacrificeAndRemember {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature,
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
        ]),
        ..Default::default()
    }
}

// ── Black ──────────────────────────────────────────────────────────────────

/// Novice Occultist — {1}{B} 1/2 Human Wizard. When it dies, draw a card and
/// lose 1 life.
pub fn novice_occultist() -> CardDefinition {
    CardDefinition {
        name: "Novice Occultist",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![on_dies(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
        ]))],
        ..Default::default()
    }
}

/// Siege Zombie — {1}{B} 2/2 Zombie. Tap three untapped creatures you control:
/// Each opponent loses 1 life.
pub fn siege_zombie() -> CardDefinition {
    CardDefinition {
        name: "Siege Zombie",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                3,
            )),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Blood Pact — {2}{B} Instant. Target player draws two cards and loses 2 life.
pub fn blood_pact() -> CardDefinition {
    CardDefinition {
        name: "Blood Pact",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::Target(0), amount: Value::Const(2) },
            Effect::LoseLife { who: Selector::Target(0), amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Bat Whisperer — {3}{B} 4/2 Vampire. ETB, if an opponent lost life this turn,
/// create a 1/1 black Bat with flying.
pub fn bat_whisperer() -> CardDefinition {
    CardDefinition {
        name: "Bat Whisperer",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 4,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::PlayerLostLifeThisTurn { who: PlayerRef::EachOpponent },
            then: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: flying_bat_token(),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Arrogant Outlaw — {2}{B} 3/2 Vampire Noble. ETB, if an opponent lost life
/// this turn, each opponent loses 2 life and you gain 2 life.
pub fn arrogant_outlaw() -> CardDefinition {
    CardDefinition {
        name: "Arrogant Outlaw",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Noble],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::PlayerLostLifeThisTurn { who: PlayerRef::EachOpponent },
            then: Box::new(Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                },
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ])),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Eaten Alive — {B} Sorcery. As an additional cost, sacrifice a creature or
/// pay {3}{B}. Exile target creature or planeswalker.
pub fn eaten_alive() -> CardDefinition {
    CardDefinition {
        name: "Eaten Alive",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::SacrificeAndRemember {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature,
            },
            Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Gluttonous Guest — {2}{B} 1/4 Vampire. ETB create a Blood token. Whenever
/// you sacrifice a Blood token, you gain 1 life.
pub fn gluttonous_guest() -> CardDefinition {
    CardDefinition {
        name: "Gluttonous Guest",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crabomination_base::tokens::blood_token(),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Blood),
                    }),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
            },
        ],
        ..Default::default()
    }
}

/// Dormant Grove // Gnarled Grovestrider — {3}{G} Enchantment. At the beginning
/// of combat on your turn, put a +1/+1 counter on target creature you control;
/// then if that creature has toughness 6+, transform it into a 3/6 vigilant
/// Treefolk that gives your other creatures vigilance.
pub fn dormant_grove() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    use crate::game::types::TurnStep;
    let grovestrider = CardDefinition {
        name: "Gnarled Grovestrider",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Treefolk], ..Default::default() },
        power: 3,
        toughness: 6,
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control have vigilance.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                keyword: Keyword::Vigilance,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Dormant Grove",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::ToughnessOf(Box::new(Selector::Target(0))),
                        Value::Const(6),
                    ),
                    then: Box::new(Effect::Transform { what: Selector::This }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        back_face: Some(Box::new(grovestrider)),
        ..Default::default()
    }
}

/// Bloodsworn Squire // Bloodsworn Knight — {3}{B} 3/3 Vampire Soldier.
/// {1}{B}, Discard a card: gains indestructible until end of turn, tap it; then
/// if four or more creature cards are in your graveyard, transform it. Back is a
/// */* Vampire Knight (power/toughness = creature cards in your graveyard).
pub fn bloodsworn_squire() -> CardDefinition {
    use crate::card::{ActivatedAbility, DynamicPt};
    // Shared cost line: indestructible until end of turn, then tap self.
    let indestructible_then_tap = || {
        vec![
            Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            Effect::Tap { what: Selector::This },
        ]
    };
    let ability = |extra: Vec<Effect>| ActivatedAbility {
        mana_cost: cost(&[generic(1), b()]),
        discard_cost: Some((SelectionRequirement::Any, 1)),
        effect: Effect::Seq(indestructible_then_tap().into_iter().chain(extra).collect()),
        ..Default::default()
    };
    let knight = CardDefinition {
        name: "Bloodsworn Knight",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Knight],
            ..Default::default()
        },
        dynamic_pt: Some(DynamicPt::BasePlusCreaturesInControllerGraveyard { base: 0 }),
        activated_abilities: vec![ability(vec![])],
        ..Default::default()
    };
    CardDefinition {
        name: "Bloodsworn Squire",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ability(vec![Effect::If {
            cond: Predicate::ValueAtLeast(
                Value::CardsInGraveyardMatching {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature,
                },
                Value::Const(4),
            ),
            then: Box::new(Effect::Transform { what: Selector::This }),
            else_: Box::new(Effect::Noop),
        }])],
        back_face: Some(Box::new(knight)),
        ..Default::default()
    }
}

/// Catapult Fodder // Catapult Captain — {2}{B} 1/5 Zombie. At combat, if you
/// control 3+ creatures with toughness greater than power, transform it. Back:
/// 2/6 with {2}{B}, {T}, Sacrifice another creature: an opponent loses life
/// equal to the sacrificed creature's toughness.
pub fn catapult_fodder() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::game::types::TurnStep;
    let captain = CardDefinition {
        name: "Catapult Captain",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 2,
        toughness: 6,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            tap_cost: true,
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::SacrificedToughness,
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Catapult Fodder",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 1,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer),
            effect: Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::ToughnessGreaterThanPower),
                    ),
                    n: Value::Const(3),
                },
                then: Box::new(Effect::Transform { what: Selector::This }),
                else_: Box::new(Effect::Noop),
            },
        }],
        back_face: Some(Box::new(captain)),
        ..Default::default()
    }
}

/// Voldaren Bloodcaster // Bloodbat Summoner — {1}{B} 2/1 flying Vampire Wizard.
/// Whenever it or another nontoken creature you control dies, make a Blood; once
/// you control five+ Blood, transform it. Back: combat trigger turns a Blood into
/// a 2/2 flying-haste Bat.
pub fn voldaren_bloodcaster() -> CardDefinition {
    use crate::game::types::TurnStep;
    let blood = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(1),
        definition: crabomination_base::tokens::blood_token(),
    };
    let summoner = CardDefinition {
        name: "Bloodbat Summoner",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer),
            effect: Effect::BecomeCreature {
                what: target_filtered(
                    SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Blood)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(2),
                creature_types: vec![CreatureType::Bat],
                keywords: vec![Keyword::Flying, Keyword::Haste],
                duration: Duration::Permanent,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Voldaren Bloodcaster",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            // "This or another nontoken creature you control dies" → Blood.
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::NotToken,
                    }),
                effect: blood(),
            },
            // "Whenever you create a Blood token, if you control five or more
            // Blood tokens, transform this creature."
            TriggeredAbility {
                event: EventSpec::new(EventKind::TokenCreated, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Blood),
                    }),
                effect: Effect::If {
                    cond: Predicate::SelectorCountAtLeast {
                        sel: Selector::EachPermanent(
                            SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Blood)
                                .and(SelectionRequirement::ControlledByYou),
                        ),
                        n: Value::Const(5),
                    },
                    then: Box::new(Effect::Transform { what: Selector::This }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        back_face: Some(Box::new(summoner)),
        ..Default::default()
    }
}

/// Daring Sleuth // Bearer of Overwhelming Truths — {1}{U} 2/1 Human Rogue.
/// When you sacrifice a Clue, transform it. Back: 3/2 Human Wizard with prowess
/// that investigates on combat damage to a player.
pub fn daring_sleuth() -> CardDefinition {
    use crate::effect::shortcut::investigate;
    let bearer = CardDefinition {
        name: "Bearer of Overwhelming Truths",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Prowess],
        triggered_abilities: vec![on_combat_damage_to_player(investigate(1))],
        ..Default::default()
    };
    CardDefinition {
        name: "Daring Sleuth",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Clue),
                }),
            effect: Effect::Transform { what: Selector::This },
        }],
        back_face: Some(Box::new(bearer)),
        ..Default::default()
    }
}

/// Restless Bloodseeker // Bloodsoaked Reveler — {1}{B} 1/3 Vampire. End step,
/// if you gained life this turn, make a Blood token. Sacrifice two Blood: transform
/// (sorcery speed). Back: 3/3 with {4}{B}: each opponent loses 2, you gain 2.
pub fn restless_bloodseeker() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::game::types::TurnStep;
    // "At the beginning of your end step, if you gained life this turn, create
    // a Blood token." (printed on both faces).
    let blood_on_lifegain = || TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer)
            .with_filter(Predicate::LifeGainedThisTurnAtLeast {
                who: PlayerRef::You,
                at_least: Value::Const(1),
            }),
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: crabomination_base::tokens::blood_token(),
        },
    };
    let reveler = CardDefinition {
        name: "Bloodsoaked Reveler",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![blood_on_lifegain()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), b()]),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                },
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Restless Bloodseeker",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![blood_on_lifegain()],
        activated_abilities: vec![ActivatedAbility {
            sorcery_speed: true,
            sac_other_filter: Some((
                SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Blood),
                2,
            )),
            effect: Effect::Transform { what: Selector::This },
            ..Default::default()
        }],
        back_face: Some(Box::new(reveler)),
        ..Default::default()
    }
}

/// Courier Bat — {2}{B} 2/2 Bat. Flying. ETB, if you gained life this turn,
/// return up to one target creature card from your graveyard to your hand.
pub fn courier_bat() -> CardDefinition {
    CardDefinition {
        name: "Courier Bat",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bat], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::LifeGainedThisTurnAtLeast { who: PlayerRef::You, at_least: Value::Const(1) },
            then: Box::new(Effect::Move {
                what: target_filtered(
                    SelectionRequirement::InGraveyard.and(SelectionRequirement::Creature),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

// ── Green ──────────────────────────────────────────────────────────────────

/// Timberland Guide — {1}{G} 1/1 Human Scout. ETB put a +1/+1 counter on target
/// creature.
pub fn timberland_guide() -> CardDefinition {
    CardDefinition {
        name: "Timberland Guide",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Pestilent Wolf — {1}{G} 2/2 Wolf. {2}{G}: this creature gains deathtouch
/// until end of turn.
pub fn pestilent_wolf() -> CardDefinition {
    CardDefinition {
        name: "Pestilent Wolf",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
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

/// Brood Weaver — {3}{G} 2/4 Spider. Reach. When it dies, create a 1/2 green
/// Spider with reach.
pub fn brood_weaver() -> CardDefinition {
    CardDefinition {
        name: "Brood Weaver",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: reach_spider_token(),
        })],
        ..Default::default()
    }
}

/// Toxic Scorpion — {1}{G} 1/1 Scorpion. Deathtouch. ETB another target creature
/// you control gains deathtouch until end of turn.
pub fn toxic_scorpion() -> CardDefinition {
    CardDefinition {
        name: "Toxic Scorpion",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Scorpion], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(
                SelectionRequirement::OtherThanSource
                    .and(SelectionRequirement::Creature)
                    .and(SelectionRequirement::ControlledByYou),
            ),
            keyword: Keyword::Deathtouch,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Clear Shot — {2}{G} Instant. Target creature you control gets +1/+1 until end
/// of turn. It deals damage equal to its power to target creature you don't
/// control.
pub fn clear_shot() -> CardDefinition {
    CardDefinition {
        name: "Clear Shot",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
        ]),
        ..Default::default()
    }
}

/// Might of the Old Ways — {1}{G} Instant. Target creature gets +2/+2. Coven —
/// then if you control three or more creatures with different powers, draw a card.
pub fn might_of_the_old_ways() -> CardDefinition {
    CardDefinition {
        name: "Might of the Old Ways",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::CovenActive { who: PlayerRef::You },
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Crushing Canopy — {2}{G} Instant. Choose one — destroy target creature with
/// flying; or destroy target enchantment.
pub fn crushing_canopy() -> CardDefinition {
    CardDefinition {
        name: "Crushing Canopy",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::HasKeyword(Keyword::Flying)),
                ),
            },
            Effect::Destroy { what: target_filtered(SelectionRequirement::Enchantment) },
        ]),
        ..Default::default()
    }
}

/// Rural Recruit — {3}{G} 1/1 Human Peasant. Training. ETB create a 3/1 green
/// Boar.
pub fn rural_recruit() -> CardDefinition {
    use crate::effect::shortcut::training;
    CardDefinition {
        name: "Rural Recruit",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Peasant],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![
            training(),
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: boar_3_1_token(),
            }),
        ],
        ..Default::default()
    }
}

/// Hamlet Vanguard — {2}{G} */* Human Warrior. Ward {2}. Enters with two +1/+1
/// counters for each other Human you control.
pub fn hamlet_vanguard() -> CardDefinition {
    CardDefinition {
        name: "Hamlet Vanguard",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::Times(
                Box::new(Value::Const(2)),
                Box::new(Value::count(Selector::EachPermanent(
                    SelectionRequirement::OtherThanSource
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Human))
                        .and(SelectionRequirement::ControlledByYou),
                ))),
            ),
        )),
        ..Default::default()
    }
}

/// Willow Geist — {G} 1/1 Treefolk Spirit. Trample. Whenever one or more cards
/// leave your graveyard, put a +1/+1 counter on it. Dies → gain life equal to
/// its power.
pub fn willow_geist() -> CardDefinition {
    CardDefinition {
        name: "Willow Geist",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk, CreatureType::Spirit],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
            on_dies(Effect::GainLife {
                who: Selector::You,
                amount: Value::PowerOf(Box::new(Selector::This)),
            }),
        ],
        ..Default::default()
    }
}

/// Packsong Pup — {1}{G} 1/1 Wolf. At the beginning of combat on your turn, if
/// you control another Wolf or Werewolf, put a +1/+1 counter on it. Dies → gain
/// life equal to its power.
pub fn packsong_pup() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Packsong Pup",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::BeginCombat),
                    EventScope::ActivePlayer,
                )
                .with_filter(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::OtherThanSource
                            .and(SelectionRequirement::ControlledByYou)
                            .and(
                                SelectionRequirement::HasCreatureType(CreatureType::Wolf)
                                    .or(SelectionRequirement::HasCreatureType(CreatureType::Werewolf)),
                            ),
                    ),
                    n: Value::Const(1),
                }),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
            on_dies(Effect::GainLife {
                who: Selector::You,
                amount: Value::PowerOf(Box::new(Selector::This)),
            }),
        ],
        ..Default::default()
    }
}

/// Reclusive Taxidermist — {1}{G} 1/2 Human Druid. Gets +3/+2 while four or more
/// creature cards are in your graveyard. {T}: add one mana of any color.
pub fn reclusive_taxidermist() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    use crate::effect::shortcut::grant_tap_for_any_color;
    CardDefinition {
        name: "Reclusive Taxidermist",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        static_abilities: vec![
            StaticAbility {
                description: "Gets +3/+2 while four or more creature cards are in your graveyard.",
                effect: StaticEffect::PumpSelfIf {
                    condition: Predicate::SelectorCountAtLeast {
                        sel: Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: crate::card::Zone::Graveyard,
                            filter: SelectionRequirement::Creature,
                        },
                        n: Value::Const(4),
                    },
                    power: 3,
                    toughness: 2,
                    keywords: vec![],
                },
            },
            grant_tap_for_any_color(SelectionRequirement::Any),
        ],
        ..Default::default()
    }
}

/// Mulch — {1}{G} Sorcery. Reveal the top four cards. Put all lands into your
/// hand and the rest into your graveyard.
pub fn mulch() -> CardDefinition {
    CardDefinition {
        name: "Mulch",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(4),
            rest_to_graveyard: true,
            pick_filter: Some(SelectionRequirement::Land),
            take: None,
            to_battlefield: false,
        },
        ..Default::default()
    }
}

/// Tapping at the Window — {1}{G} Sorcery. Look at the top three cards. You may
/// reveal a creature card and put it into your hand. Rest to graveyard.
/// Flashback {2}{G}.
pub fn tapping_at_the_window() -> CardDefinition {
    CardDefinition {
        name: "Tapping at the Window",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(2), g()]))],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(3),
            rest_to_graveyard: true,
            pick_filter: Some(SelectionRequirement::Creature),
            take: Some(Value::Const(1)),
            to_battlefield: false,
        },
        ..Default::default()
    }
}

/// Splendid Reclamation — {3}{G} Sorcery. Return all land cards from your
/// graveyard to the battlefield tapped.
pub fn splendid_reclamation() -> CardDefinition {
    CardDefinition {
        name: "Splendid Reclamation",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: crate::card::Zone::Graveyard,
                filter: SelectionRequirement::Land,
            },
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        },
        ..Default::default()
    }
}

// ── Red ────────────────────────────────────────────────────────────────────

/// Voldaren Stinger — {R} 1/1 Vampire Warrior. First strike while attacking.
/// {2}{R}: +2/+0 until end of turn.
pub fn voldaren_stinger() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Voldaren Stinger",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "First strike as long as this creature is attacking.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::EntityMatches {
                    what: Selector::This,
                    filter: SelectionRequirement::IsAttacking,
                },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::FirstStrike],
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Abandon the Post — {1}{R} Sorcery. Up to two target creatures can't block
/// this turn. Flashback {3}{R}.
pub fn abandon_the_post() -> CardDefinition {
    CardDefinition {
        name: "Abandon the Post",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), r()]))],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            filter: SelectionRequirement::Creature,
            effect: Box::new(Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}

/// Daybreak Combatants — {2}{R} 2/2 Human Warrior. Haste. ETB target creature
/// gets +2/+0 until end of turn.
pub fn daybreak_combatants() -> CardDefinition {
    CardDefinition {
        name: "Daybreak Combatants",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Neonate's Rush — {2}{R} Instant. Costs {1} less if you control a Vampire.
/// Deals 1 damage to target creature and 1 to its controller. Draw a card.
pub fn neonates_rush() -> CardDefinition {
    CardDefinition {
        name: "Neonate's Rush",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_control: vec![(
            SelectionRequirement::HasCreatureType(CreatureType::Vampire)
                .and(SelectionRequirement::ControlledByYou),
            1,
        )],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(1),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(1),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Rending Flame — {2}{R} Instant. Deals 5 damage to target creature or
/// planeswalker. If it's a Spirit, also deals 2 damage to its controller.
pub fn rending_flame() -> CardDefinition {
    CardDefinition {
        name: "Rending Flame",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(5),
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Spirit),
                },
                then: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::Const(2),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// End the Festivities — {R} Sorcery. Deals 1 damage to each opponent and each
/// creature and planeswalker they control.
pub fn end_the_festivities() -> CardDefinition {
    CardDefinition {
        name: "End the Festivities",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    SelectionRequirement::ControlledByOpponent.and(
                        SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                    ),
                ),
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

/// Raze the Effigy — {R} Instant. Choose one — destroy target artifact; or
/// target attacking creature gets +2/+2 until end of turn.
pub fn raze_the_effigy() -> CardDefinition {
    CardDefinition {
        name: "Raze the Effigy",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(SelectionRequirement::Artifact) },
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::IsAttacking),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Belligerent Guest — {2}{R} 3/2 Vampire. Trample. Combat damage to a player →
/// create a Blood token.
pub fn belligerent_guest() -> CardDefinition {
    CardDefinition {
        name: "Belligerent Guest",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crabomination_base::tokens::blood_token(),
            },
        }],
        ..Default::default()
    }
}

/// Frenzied Devils — {4}{R} 3/3 Devil. Haste. Whenever you cast a noncreature
/// spell, this creature gets +2/+2 until end of turn.
pub fn frenzied_devils() -> CardDefinition {
    use crate::effect::shortcut::cast_is_noncreature;
    CardDefinition {
        name: "Frenzied Devils",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Devil], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_noncreature()),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

// ── Black / Blue / Artifact (remainder) ──────────────────────────────────────

/// Undead Butler — {1}{B} 1/2 Zombie. ETB mill three. When it dies, you may
/// exile it; if you do, return target creature card from your graveyard to your
/// hand.
pub fn undead_butler() -> CardDefinition {
    CardDefinition {
        name: "Undead Butler",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![
            etb(Effect::Mill { who: Selector::You, amount: Value::Const(3) }),
            on_dies(Effect::MayDo {
                description: "Exile Undead Butler to return a creature card from your graveyard?"
                    .to_string(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Exile { what: Selector::This },
                    Effect::Move {
                        what: target_filtered(
                            SelectionRequirement::InGraveyard.and(SelectionRequirement::Creature),
                        ),
                        to: ZoneDest::Hand(PlayerRef::You),
                    },
                ])),
            }),
        ],
        ..Default::default()
    }
}

/// Mindleech Ghoul — {1}{B} 2/2 Zombie. Exploit. When it exploits a creature,
/// each opponent exiles a card from their hand.
pub fn mindleech_ghoul() -> CardDefinition {
    use crate::effect::shortcut::exploit;
    CardDefinition {
        name: "Mindleech Ghoul",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![exploit(Effect::ExileFromHand {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Morkrut Behemoth — {4}{B} 7/6 Zombie Giant. Menace. (The "sacrifice a
/// creature or pay {1}{B}" additional cast cost is omitted — body only.)
pub fn morkrut_behemoth() -> CardDefinition {
    CardDefinition {
        name: "Morkrut Behemoth",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Giant],
            ..Default::default()
        },
        power: 7,
        toughness: 6,
        keywords: vec![Keyword::Menace],
        ..Default::default()
    }
}

/// Demonic Bargain — {2}{B} Sorcery. Exile the top thirteen cards of your
/// library, then search your library for a card, put it into your hand, then
/// shuffle.
pub fn demonic_bargain() -> CardDefinition {
    CardDefinition {
        name: "Demonic Bargain",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ExileTopOfLibrary {
                who: Selector::You,
                amount: Value::Const(13),
                link_to_source: false,
                face_down: false,
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Any,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        ..Default::default()
    }
}

/// Thirst for Discovery — {2}{U} Instant. Draw three cards, then discard two
/// cards unless you discard a basic land card (modeled as discard two).
pub fn thirst_for_discovery() -> CardDefinition {
    CardDefinition {
        name: "Thirst for Discovery",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(2),
                random: false,
            },
        ]),
        ..Default::default()
    }
}

/// Blood Fountain — {B} Artifact. ETB create a Blood token. {3}{B}, {T},
/// Sacrifice this: return up to two target creature cards from your graveyard
/// to your hand.
pub fn blood_fountain() -> CardDefinition {
    CardDefinition {
        name: "Blood Fountain",
        cost: cost(&[b()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: crabomination_base::tokens::blood_token(),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                filter: SelectionRequirement::InGraveyard.and(SelectionRequirement::Creature),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Batch 2: day/night, threaten, conditional removal ────────────────────────

/// Component Collector — {2}{U} 1/4 Homunculus. Becomes day on entry if neither
/// day nor night. Whenever day/night flips, you may tap or untap target nonland
/// permanent.
pub fn component_collector() -> CardDefinition {
    CardDefinition {
        name: "Component Collector",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Homunculus], ..Default::default() },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![
            etb(Effect::If {
                cond: Predicate::Not(Box::new(Predicate::Any(vec![
                    Predicate::IsDay,
                    Predicate::IsNight,
                ]))),
                then: Box::new(Effect::BecomeDay),
                else_: Box::new(Effect::Noop),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DayNightChanged, EventScope::AnyPlayer),
                effect: Effect::MayDo {
                    description: "Tap or untap target nonland permanent?".to_string(),
                    body: Box::new(Effect::ChooseMode(vec![
                        Effect::Tap {
                            what: target_filtered(SelectionRequirement::Nonland),
                        },
                        Effect::Untap {
                            what: target_filtered(SelectionRequirement::Nonland),
                            up_to: None,
                        },
                    ])),
                },
            },
        ],
        ..Default::default()
    }
}

/// Stromkirk Bloodthief — {2}{B} 2/2 Vampire Rogue. At your end step, if an
/// opponent lost life this turn, put a +1/+1 counter on target Vampire you
/// control.
pub fn stromkirk_bloodthief() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Stromkirk Bloodthief",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer)
                .with_filter(Predicate::PlayerLostLifeThisTurn { who: PlayerRef::EachOpponent }),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::HasCreatureType(CreatureType::Vampire)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Voldaren Ambusher — {2}{R} 2/2 Vampire Archer. ETB, if an opponent lost life
/// this turn, deal X damage to up to one target creature or planeswalker, where
/// X is the number of Vampires you control.
pub fn voldaren_ambusher() -> CardDefinition {
    let vampires = Value::count(Selector::EachPermanent(
        SelectionRequirement::HasCreatureType(CreatureType::Vampire)
            .and(SelectionRequirement::ControlledByYou),
    ));
    CardDefinition {
        name: "Voldaren Ambusher",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Archer],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::PlayerLostLifeThisTurn { who: PlayerRef::EachOpponent },
            then: Box::new(Effect::ApplyToTargets {
                max_targets: 1,
                filter: SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                effect: Box::new(Effect::DealDamage { to: Selector::Target(0), amount: vampires }),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Chill of the Grave — {2}{U} Instant. Costs {1} less if you control a Zombie.
/// Tap target creature; it doesn't untap during its controller's next untap
/// step. Draw a card.
pub fn chill_of_the_grave() -> CardDefinition {
    CardDefinition {
        name: "Chill of the Grave",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_control: vec![(
            SelectionRequirement::HasCreatureType(CreatureType::Zombie)
                .and(SelectionRequirement::ControlledByYou),
            1,
        )],
        effect: Effect::Seq(vec![
            Effect::Tap { what: target_filtered(SelectionRequirement::Creature) },
            Effect::SkipNextUntap { what: Selector::Target(0) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Electric Revelation — {2}{R} Instant. As an additional cost, discard a card.
/// Draw two cards. Flashback {3}{R}.
pub fn electric_revelation() -> CardDefinition {
    CardDefinition {
        name: "Electric Revelation",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), r()]))],
        effect: Effect::Seq(vec![
            Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Bloody Betrayal — {2}{R} Sorcery. Gain control of target creature until end
/// of turn. Untap it; it gains haste. Create a Blood token.
pub fn bloody_betrayal() -> CardDefinition {
    CardDefinition {
        name: "Bloody Betrayal",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(SelectionRequirement::Creature),
                to: Some(PlayerRef::You),
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crabomination_base::tokens::blood_token(),
            },
        ]),
        ..Default::default()
    }
}

/// Wolf Strike — {2}{G} Instant. Target creature you control gets +2/+0 until
/// end of turn if it's night. Then it deals damage equal to its power to target
/// creature you don't control.
pub fn wolf_strike() -> CardDefinition {
    CardDefinition {
        name: "Wolf Strike",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::IsNight,
                then: Box::new(Effect::PumpPT {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    },
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
        ]),
        ..Default::default()
    }
}

/// Fateful Absence — {1}{W} Instant. Destroy target creature or planeswalker.
/// Its controller investigates.
pub fn fateful_absence() -> CardDefinition {
    CardDefinition {
        name: "Fateful Absence",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::Const(1),
                definition: crabomination_base::tokens::clue_token(),
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Bloodline Culling — {1}{B}{B} Instant. Choose one — target creature gets
/// -5/-5 until end of turn; or creature tokens get -2/-2 until end of turn.
pub fn bloodline_culling() -> CardDefinition {
    CardDefinition {
        name: "Bloodline Culling",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-5),
                toughness: Value::Const(-5),
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::IsToken),
                ),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Gavony Dawnguard — {1}{W}{W} 3/3 Human Soldier. Ward {1}. Becomes day on
/// entry if neither day nor night. Whenever day/night flips, look at the top
/// four cards; you may reveal a creature card and put it into your hand, rest
/// to the bottom.
pub fn gavony_dawnguard() -> CardDefinition {
    CardDefinition {
        name: "Gavony Dawnguard",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(1)])))],
        triggered_abilities: vec![
            etb(Effect::If {
                cond: Predicate::Not(Box::new(Predicate::Any(vec![
                    Predicate::IsDay,
                    Predicate::IsNight,
                ]))),
                then: Box::new(Effect::BecomeDay),
                else_: Box::new(Effect::Noop),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DayNightChanged, EventScope::AnyPlayer),
                effect: Effect::LookPickToHand {
                    who: PlayerRef::You,
                    count: Value::Const(4),
                    rest_to_graveyard: false,
                    pick_filter: Some(SelectionRequirement::Creature),
                    take: Some(Value::Const(1)),
                    to_battlefield: false,
                },
            },
        ],
        ..Default::default()
    }
}

// ── Batch 3: simple commons ──────────────────────────────────────────────────

/// Lightning Wolf — {3}{R} 4/3 Wolf. {1}{R}: gains first strike until end of
/// turn. Activate only as a sorcery.
pub fn lightning_wolf() -> CardDefinition {
    CardDefinition {
        name: "Lightning Wolf",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 4,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            sorcery_speed: true,
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ritual Guardian — {2}{W} 3/2 Human Soldier. Coven — at the beginning of
/// combat on your turn, if you control three or more creatures with different
/// powers, it gains lifelink until end of turn.
pub fn ritual_guardian() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Ritual Guardian",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer)
                .with_filter(Predicate::CovenActive { who: PlayerRef::You }),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Defend the Celestus — {2}{G}{G} Instant. Distribute three +1/+1 counters
/// among one, two, or three target creatures you control.
pub fn defend_the_celestus() -> CardDefinition {
    CardDefinition {
        name: "Defend the Celestus",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DistributeCounters {
            total: Value::Const(3),
            counter: CounterType::PlusOnePlusOne,
            filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            max_targets: 3,
        },
        ..Default::default()
    }
}

/// Rot-Tide Gargantua — {3}{B}{B} 5/4 Zombie Kraken. Exploit. When it exploits
/// a creature, each opponent sacrifices a creature of their choice.
pub fn rot_tide_gargantua() -> CardDefinition {
    use crate::effect::shortcut::exploit;
    CardDefinition {
        name: "Rot-Tide Gargantua",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Kraken],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        triggered_abilities: vec![exploit(Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachOpponent),
            count: Value::Const(1),
            filter: SelectionRequirement::Creature,
        })],
        ..Default::default()
    }
}

/// Weary Prisoner // Wrathful Jailbreaker — {3}{R} 2/6 Defender Werewolf.
/// Back: 6/6 that attacks each combat if able.
pub fn weary_prisoner() -> CardDefinition {
    werewolf_dfc(
        "Weary Prisoner",
        cost(&[generic(3), r()]),
        vec![CreatureType::Human, CreatureType::Werewolf],
        (2, 6),
        vec![Keyword::Defender],
        vec![],
        "Wrathful Jailbreaker",
        (6, 6),
        vec![Keyword::MustAttack],
        vec![],
    )
}

/// Lambholt Pacifist // Lambholt Butcher — {1}{G} 3/3 Human Shaman Werewolf
/// that can't attack unless you control a power-4+ creature; classic spells-
/// cast werewolf transform. Back: 4/4 Werewolf.
pub fn lambholt_pacifist() -> CardDefinition {
    use crate::effect::shortcut::{werewolf_day_transform, werewolf_night_transform};
    let butcher = CardDefinition {
        name: "Lambholt Butcher",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Werewolf], ..Default::default() },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![werewolf_night_transform()],
        ..Default::default()
    };
    CardDefinition {
        name: "Lambholt Pacifist",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman, CreatureType::Werewolf],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::CantAttackOrBlockUnlessYouControlCount {
            filter: Box::new(SelectionRequirement::Creature.and(SelectionRequirement::PowerAtLeast(4))),
            min: 1,
            attack_only: true,
        }],
        triggered_abilities: vec![werewolf_day_transform()],
        back_face: Some(Box::new(butcher)),
        ..Default::default()
    }
}

/// Cobbled Lancer — {U} 3/3 Zombie Horse; additional cost: exile a creature
/// card from your graveyard. {3}{U}, Exile this from your graveyard: draw.
pub fn cobbled_lancer() -> CardDefinition {
    use crate::card::{ActivatedAbility, AdditionalCastCost};
    CardDefinition {
        name: "Cobbled Lancer",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Horse],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        additional_cast_cost: vec![AdditionalCastCost::ExileFromGraveyard {
            filter: SelectionRequirement::Creature,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ground Pounder — {1}{G} 2/2 Goblin Warrior. {3}{G}: roll a d6; it gets
/// +X/+X until end of turn, where X is the result. Whenever you roll a 5 or
/// higher on a die, it gains trample until end of turn.
pub fn ground_pounder() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Ground Pounder",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            effect: Effect::RollDie {
                sides: 6,
                count: Value::Const(1),
                modifier: Value::Const(0),
                reroll_at_most: 0,
                results: vec![(1, 6, Effect::PumpPT {
                    what: Selector::This,
                    power: Value::LastDieRoll,
                    toughness: Value::LastDieRoll,
                    duration: Duration::EndOfTurn,
                })],
                on_doubles: None,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::RolledDice, EventScope::YourControl)
                .with_filter(Predicate::DieResultAtLeast(5)),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Augur of Autumn — {1}{G}{G} 2/3 Human Druid. You may play lands from the top
/// of your library. (The coven cast-creatures-from-top rider is omitted.)
pub fn augur_of_autumn() -> CardDefinition {
    use crate::card::{StaticAbility};
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Augur of Autumn",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "You may play lands from the top of your library.",
            effect: StaticEffect::PlayFromLibraryTop { filter: SelectionRequirement::Land },
        }],
        ..Default::default()
    }
}

/// Secrets of the Key — {U} Instant. Investigate. Flashback {3}{U}. (The
/// "investigate twice if cast from a graveyard" rider is omitted.)
pub fn secrets_of_the_key() -> CardDefinition {
    CardDefinition {
        name: "Secrets of the Key",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: crabomination_base::tokens::clue_token(),
        },
        keywords: vec![Keyword::Flashback(cost(&[generic(3), u()]))],
        ..Default::default()
    }
}

/// Diregraf Rebirth — {3}{B}{G} Sorcery. Return target creature card from your
/// graveyard to the battlefield. Flashback {5}{B}{G}. (The cost-reduction-per-
/// creature-that-died-this-turn rider is omitted.)
pub fn diregraf_rebirth() -> CardDefinition {
    CardDefinition {
        name: "Diregraf Rebirth",
        cost: cost(&[generic(3), b(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::InGraveyard),
            ),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        keywords: vec![Keyword::Flashback(cost(&[generic(5), b(), g()]))],
        ..Default::default()
    }
}

/// Edgar's Awakening — {3}{B}{B} Sorcery. Return target creature card from your
/// graveyard to the battlefield. (The discard-mode hand-return rider is omitted.)
pub fn edgars_awakening() -> CardDefinition {
    CardDefinition {
        name: "Edgar's Awakening",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::InGraveyard),
            ),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        ..Default::default()
    }
}

/// Ceremonial Knife — {1} Equipment. Equipped creature gets +1/+0 and makes a
/// Blood token whenever it deals combat damage. Equip {2}.
pub fn ceremonial_knife() -> CardDefinition {
    CardDefinition {
        name: "Ceremonial Knife",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 0,
            triggered_abilities: vec![on_combat_damage_to_player(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crabomination_base::tokens::blood_token(),
            })],
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── Batch 7: simple MID/VOW commons & uncommons ──────────────────────────────

/// Bleeding Edge — {1}{B}{B} Sorcery. Up to one target creature gets -2/-2 until
/// end of turn; Amass Zombies 2.
pub fn bleeding_edge() -> CardDefinition {
    use crate::effect::shortcut::amass_zombies;
    CardDefinition {
        name: "Bleeding Edge",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 1,
                filter: SelectionRequirement::Creature,
                effect: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(-2),
                    toughness: Value::Const(-2),
                    duration: Duration::EndOfTurn,
                }),
            },
            amass_zombies(2),
        ]),
        ..Default::default()
    }
}

/// Lantern Bearer // Lanterns' Lift — {U} 1/1 flying Spirit. Disturb {2}{U} into
/// an Aura granting +1/+1 and flying.
pub fn lantern_bearer() -> CardDefinition {
    let front = CardDefinition {
        name: "Lantern Bearer",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    disturb_aura_dfc(
        front,
        cost(&[generic(2), u()]),
        "Lanterns' Lift",
        EquipBonus { power: 1, toughness: 1, keywords: vec![Keyword::Flying], ..Default::default() },
    )
}
