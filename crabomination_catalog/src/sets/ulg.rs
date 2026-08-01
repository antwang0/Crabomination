//! Urza's Legacy (ULG) gap closure. Tests in `classic_sets/ulg`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, StaticEffect, ZoneDest,
    shortcut::{deal, on_dies, target_any, target_filtered},
};
use crate::game::TurnStep;
use crate::mana::{Color, b, cost, g, generic, r, u, w, x};

fn creature(
    name: &'static str,
    c: crate::mana::ManaCost,
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

fn instant(name: &'static str, c: crate::mana::ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

fn sorcery(name: &'static str, c: crate::mana::ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

fn enchantment(name: &'static str, c: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        ..Default::default()
    }
}

/// An Aura that returns to its owner's hand when it hits the graveyard — the
/// ULG "sticky Aura" cycle (Cessation, Sleeper's Guile, Sluggishness).
fn sticky_aura(name: &'static str, c: crate::mana::ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        }],
        ..Default::default()
    }
}

/// The Phyrexian Carrier cycle — "{T}, Sacrifice this: target creature gets
/// -N/-N until end of turn."
fn carrier(name: &'static str, c: crate::mana::ManaCost, p: i32, t: i32, n: i32) -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-n),
                toughness: Value::Const(-n),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(name, c, vec![CreatureType::Phyrexian, CreatureType::Carrier], p, t)
    }
}

// ── Cycling commons ─────────────────────────────────────────────────────────

/// Bloated Toad — {2}{G} 2/2. Protection from blue; cycling {2}.
pub fn bloated_toad() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Protection(Color::Blue),
            Keyword::Cycling(cost(&[generic(2)])),
        ],
        ..creature("Bloated Toad", cost(&[generic(2), g()]), vec![CreatureType::Frog], 2, 2)
    }
}

/// Darkwatch Elves — {2}{G} 2/2. Protection from black; cycling {2}.
pub fn darkwatch_elves() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Protection(Color::Black),
            Keyword::Cycling(cost(&[generic(2)])),
        ],
        ..creature("Darkwatch Elves", cost(&[generic(2), g()]), vec![CreatureType::Elf], 2, 2)
    }
}

/// Iron Will — {W} Instant. +0/+4; cycling {2}.
pub fn iron_will() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        ..instant(
            "Iron Will",
            cost(&[w()]),
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(0),
                toughness: Value::Const(4),
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Radiant's Judgment — {2}{W} Instant. Destroy a power-4-or-greater creature;
/// cycling {2}.
pub fn radiants_judgment() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        ..instant(
            "Radiant's Judgment",
            cost(&[generic(2), w()]),
            Effect::Destroy { what: target_filtered(R::Creature.and(R::PowerAtLeast(4))) },
        )
    }
}

/// Swat — {1}{B}{B} Instant. Destroy a power-2-or-less creature; cycling {2}.
pub fn swat() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        ..instant(
            "Swat",
            cost(&[generic(1), b(), b()]),
            Effect::Destroy { what: target_filtered(R::Creature.and(R::PowerAtMost(2))) },
        )
    }
}

/// Rebuild — {2}{U} Instant. Bounce all artifacts; cycling {2}.
pub fn rebuild() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        ..instant(
            "Rebuild",
            cost(&[generic(2), u()]),
            Effect::Move {
                what: Selector::EachPermanent(R::Artifact),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        )
    }
}

// ── Instants & sorceries ────────────────────────────────────────────────────

/// About Face — {R} Instant. Switch a creature's power and toughness.
pub fn about_face() -> CardDefinition {
    instant(
        "About Face",
        cost(&[r()]),
        Effect::SwitchPT {
            what: target_filtered(R::Creature),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Burst of Energy — {W} Instant. Untap target permanent.
pub fn burst_of_energy() -> CardDefinition {
    instant(
        "Burst of Energy",
        cost(&[w()]),
        Effect::Untap { what: target_filtered(R::Permanent), up_to: None },
    )
}

/// Intervene — {U} Instant. Counter target spell that targets a creature.
pub fn intervene() -> CardDefinition {
    instant(
        "Intervene",
        cost(&[u()]),
        Effect::CounterSpell { what: target_filtered(R::SpellTargetsCreature) },
    )
}

/// Ostracize — {B} Sorcery. Target opponent reveals their hand; you pick a
/// creature card and they discard it.
pub fn ostracize() -> CardDefinition {
    sorcery(
        "Ostracize",
        cost(&[b()]),
        Effect::DiscardChosen {
            from: Selector::Player(PlayerRef::Target(0)),
            count: Value::ONE,
            filter: R::Creature,
        },
    )
}

/// Peace and Quiet — {1}{W} Instant. Destroy two target enchantments.
pub fn peace_and_quiet() -> CardDefinition {
    instant(
        "Peace and Quiet",
        cost(&[generic(1), w()]),
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 2,
            filter: R::Enchantment,
            effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
        },
    )
}

/// Rack and Ruin — {2}{R} Instant. Destroy two target artifacts.
pub fn rack_and_ruin() -> CardDefinition {
    instant(
        "Rack and Ruin",
        cost(&[generic(2), r()]),
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 2,
            filter: R::Artifact,
            effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
        },
    )
}

/// Purify — {3}{W}{W} Sorcery. Destroy all artifacts and enchantments.
pub fn purify() -> CardDefinition {
    sorcery(
        "Purify",
        cost(&[generic(3), w(), w()]),
        Effect::Destroy { what: Selector::EachPermanent(R::Artifact.or(R::Enchantment)) },
    )
}

/// Sick and Tired — {2}{B} Instant. Two target creatures each get -1/-1.
pub fn sick_and_tired() -> CardDefinition {
    instant(
        "Sick and Tired",
        cost(&[generic(2), b()]),
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 2,
            filter: R::Creature,
            effect: Box::new(Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            }),
        },
    )
}

/// Silk Net — {G} Instant. +1/+1 and reach.
pub fn silk_net() -> CardDefinition {
    instant(
        "Silk Net",
        cost(&[g()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Reach,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Hope and Glory — {1}{W} Instant. Untap two target creatures; each gets
/// +1/+1.
pub fn hope_and_glory() -> CardDefinition {
    instant(
        "Hope and Glory",
        cost(&[generic(1), w()]),
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 2,
            filter: R::Creature,
            effect: Box::new(Effect::Seq(vec![
                Effect::Untap { what: Selector::Target(0), up_to: None },
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
            ])),
        },
    )
}

/// Harmonic Convergence — {2}{G} Instant. Put all enchantments on top of their
/// owners' libraries.
pub fn harmonic_convergence() -> CardDefinition {
    instant(
        "Harmonic Convergence",
        cost(&[generic(2), g()]),
        Effect::Move {
            what: Selector::EachPermanent(R::Enchantment),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOfMoved,
                pos: crate::effect::LibraryPosition::Top,
            },
        },
    )
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Angelic Curator — {1}{W} 1/1. Flying, protection from artifacts.
pub fn angelic_curator() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Flying,
            Keyword::ProtectionFromCardType(CardType::Artifact),
        ],
        ..creature(
            "Angelic Curator",
            cost(&[generic(1), w()]),
            vec![CreatureType::Angel, CreatureType::Spirit],
            1,
            1,
        )
    }
}

/// Archivist — {2}{U}{U} 1/1. {T}: Draw a card.
pub fn archivist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature(
            "Archivist",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Beast of Burden — {6} Golem `*/*`, sized by every creature on the
/// battlefield.
pub fn beast_of_burden() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        dynamic_pt: Some(DynamicPt::AllCreaturesOnBattlefield),
        ..creature("Beast of Burden", cost(&[generic(6)]), vec![CreatureType::Golem], 0, 0)
    }
}

/// Defender of Chaos — {2}{R} 2/1. Flash, protection from white.
pub fn defender_of_chaos() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Protection(Color::White)],
        ..creature(
            "Defender of Chaos",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            1,
        )
    }
}

/// Defender of Law — {2}{W} 2/1. Flash, protection from red.
pub fn defender_of_law() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Protection(Color::Red)],
        ..creature(
            "Defender of Law",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            1,
        )
    }
}

/// Weatherseed Faeries — {2}{U} 2/1. Flying, protection from red.
pub fn weatherseed_faeries() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Protection(Color::Red)],
        ..creature("Weatherseed Faeries", cost(&[generic(2), u()]), vec![CreatureType::Faerie], 2, 1)
    }
}

/// Yavimaya Scion — {4}{G} 4/4. Protection from artifacts.
pub fn yavimaya_scion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::ProtectionFromCardType(CardType::Artifact)],
        ..creature("Yavimaya Scion", cost(&[generic(4), g()]), vec![CreatureType::Treefolk], 4, 4)
    }
}

/// Yavimaya Wurm — {4}{G}{G} 6/4 Trample.
pub fn yavimaya_wurm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..creature("Yavimaya Wurm", cost(&[generic(4), g(), g()]), vec![CreatureType::Wurm], 6, 4)
    }
}

/// Giant Cockroach — {3}{B} 4/2 vanilla.
pub fn giant_cockroach() -> CardDefinition {
    creature("Giant Cockroach", cost(&[generic(3), b()]), vec![CreatureType::Insect], 4, 2)
}

/// Plague Beetle — {B} 1/1 Swampwalk.
pub fn plague_beetle() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Swamp)],
        ..creature("Plague Beetle", cost(&[b()]), vec![CreatureType::Insect], 1, 1)
    }
}

/// Fog of Gnats — {B}{B} 1/1. Flying; {B}: regenerate.
pub fn fog_of_gnats() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Fog of Gnats", cost(&[b(), b()]), vec![CreatureType::Insect], 1, 1)
    }
}

/// Eviscerator — {3}{B}{B} 5/5. Protection from white; ETB lose 5 life.
pub fn eviscerator() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::White)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::LoseLife { who: Selector::You, amount: Value::Const(5) },
        }],
        ..creature(
            "Eviscerator",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Horror],
            5,
            5,
        )
    }
}

/// Ghitu Fire-Eater — {2}{R} 2/2. {T}, Sacrifice: damage equal to its power to
/// any target.
pub fn ghitu_fire_eater() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::DealDamage {
                to: target_any(),
                amount: Value::SacrificedPower,
            },
            ..Default::default()
        }],
        ..creature(
            "Ghitu Fire-Eater",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Nomad],
            2,
            2,
        )
    }
}

/// Goblin Medics — {2}{R} 1/1. Whenever it becomes tapped, 1 damage to any
/// target.
pub fn goblin_medics() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: deal(1, target_any()),
        }],
        ..creature(
            "Goblin Medics",
            cost(&[generic(2), r()]),
            vec![CreatureType::Goblin, CreatureType::Shaman],
            1,
            1,
        )
    }
}

/// Thornwind Faeries — {1}{U}{U} 1/1. Flying; {T}: 1 damage to any target.
pub fn thornwind_faeries() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: deal(1, target_any()),
            ..Default::default()
        }],
        ..creature(
            "Thornwind Faeries",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Faerie],
            1,
            1,
        )
    }
}

/// Expendable Troops — {1}{W} 2/1. {T}, Sacrifice: 2 damage to an attacking or
/// blocking creature.
pub fn expendable_troops() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: deal(2, target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking)))),
            ..Default::default()
        }],
        ..creature(
            "Expendable Troops",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            1,
        )
    }
}

/// Fleeting Image — {2}{U} 2/1. Flying; {1}{U}: bounce itself.
pub fn fleeting_image() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..creature("Fleeting Image", cost(&[generic(2), u()]), vec![CreatureType::Illusion], 2, 1)
    }
}

/// Vigilant Drake — {4}{U} 3/3. Flying; {2}{U}: untap itself.
pub fn vigilant_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Untap { what: Selector::This, up_to: None },
            ..Default::default()
        }],
        ..creature("Vigilant Drake", cost(&[generic(4), u()]), vec![CreatureType::Drake], 3, 3)
    }
}

/// King Crab — {4}{U}{U} 4/5. {1}{U}, {T}: put a green creature on top of its
/// owner's library.
pub fn king_crab() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            tap_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::HasColor(Color::Green))),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOfMoved,
                    pos: crate::effect::LibraryPosition::Top,
                },
            },
            ..Default::default()
        }],
        ..creature("King Crab", cost(&[generic(4), u(), u()]), vec![CreatureType::Crab], 4, 5)
    }
}

/// Devout Harpist — {W} 1/1. {T}: destroy an Aura attached to a creature.
pub fn devout_harpist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)),
            },
            ..Default::default()
        }],
        ..creature("Devout Harpist", cost(&[w()]), vec![CreatureType::Human], 1, 1)
    }
}

/// Tragic Poet — {W} 1/1. {T}, Sacrifice: return an enchantment card from your
/// graveyard to hand.
pub fn tragic_poet() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Enchantment.and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..creature("Tragic Poet", cost(&[w()]), vec![CreatureType::Human], 1, 1)
    }
}

/// Phyrexian Broodlings — {1}{B}{B} 2/2. {1}, Sacrifice a creature: +1/+1
/// counter.
pub fn phyrexian_broodlings() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Phyrexian Broodlings",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Minion],
            2,
            2,
        )
    }
}

/// Phyrexian Denouncer — {1}{B} 1/1 Carrier (-1/-1).
pub fn phyrexian_denouncer() -> CardDefinition {
    carrier("Phyrexian Denouncer", cost(&[generic(1), b()]), 1, 1, 1)
}

/// Phyrexian Debaser — {3}{B} 2/2 Carrier (-2/-2), Flying.
pub fn phyrexian_debaser() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..carrier("Phyrexian Debaser", cost(&[generic(3), b()]), 2, 2, 2)
    }
}

/// Phyrexian Defiler — {2}{B}{B} 3/3 Carrier (-3/-3).
pub fn phyrexian_defiler() -> CardDefinition {
    carrier("Phyrexian Defiler", cost(&[generic(2), b(), b()]), 3, 3, 3)
}

/// Phyrexian Plaguelord — {3}{B}{B} 4/4 Carrier (-4/-4), plus a free
/// sacrifice outlet for -1/-1.
pub fn phyrexian_plaguelord() -> CardDefinition {
    let mut def = carrier("Phyrexian Plaguelord", cost(&[generic(3), b(), b()]), 4, 4, 4);
    def.activated_abilities.push(ActivatedAbility {
        sac_other_filter: Some((R::Creature, 1)),
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    });
    def
}

/// Shivan Phoenix — {4}{R}{R} 3/4. Flying; returns to hand when it dies.
pub fn shivan_phoenix() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::Move {
            what: Selector::This,
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        })],
        ..creature("Shivan Phoenix", cost(&[generic(4), r(), r()]), vec![CreatureType::Phoenix], 3, 4)
    }
}

/// Weatherseed Treefolk — {2}{G}{G}{G} 5/3. Trample; returns to hand when it
/// dies.
pub fn weatherseed_treefolk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![on_dies(Effect::Move {
            what: Selector::This,
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        })],
        ..creature(
            "Weatherseed Treefolk",
            cost(&[generic(2), g(), g(), g()]),
            vec![CreatureType::Treefolk],
            5,
            3,
        )
    }
}

/// Viashino Sandscout — {1}{R} 2/1. Haste; bounces itself at end of turn.
pub fn viashino_sandscout() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        }],
        ..creature(
            "Viashino Sandscout",
            cost(&[generic(1), r()]),
            vec![CreatureType::Lizard, CreatureType::Scout],
            2,
            1,
        )
    }
}

/// Viashino Cutthroat — {2}{R}{R} 5/3. Haste; bounces itself at end of turn.
pub fn viashino_cutthroat() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        }],
        ..creature(
            "Viashino Cutthroat",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Lizard],
            5,
            3,
        )
    }
}

/// Multani, Maro-Sorcerer — {4}{G}{G} `*/*` sized by every hand, with shroud.
pub fn multani_maro_sorcerer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Shroud],
        dynamic_pt: Some(DynamicPt::AllPlayersHandTotal),
        ..creature(
            "Multani, Maro-Sorcerer",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Elemental, CreatureType::Sorcerer],
            0,
            0,
        )
    }
}

/// Radiant, Archangel — {3}{W}{W} 3/3 base. Flying, vigilance, and +1/+1 for
/// each other flyer on the battlefield.
pub fn radiant_archangel() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        dynamic_pt: Some(DynamicPt::BasePlusOtherFlyersOnBattlefield { base: 3 }),
        ..creature(
            "Radiant, Archangel",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Angel],
            3,
            3,
        )
    }
}

/// Anthroplasm — {2}{U}{U} 0/0. Enters with two +1/+1 counters; {X}, {T}
/// resets them to X.
pub fn anthroplasm() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne },
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::XFromCost,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Anthroplasm",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Shapeshifter],
            0,
            0,
        )
    }
}

/// Molten Hydra — {1}{R} 1/1. Grows for {1}{R}{R}; {T} cashes its counters in
/// for damage.
pub fn molten_hydra() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), r(), r()]),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::DealDamage {
                        to: target_any(),
                        amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne },
                    },
                    Effect::RemoveCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne },
                    },
                ]),
                ..Default::default()
            },
        ],
        ..creature("Molten Hydra", cost(&[generic(1), r()]), vec![CreatureType::Hydra], 1, 1)
    }
}

// ── Enchantments ────────────────────────────────────────────────────────────

/// Knighthood — {2}{W}. Your creatures have first strike.
pub fn knighthood() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have first strike.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::FirstStrike,
            },
        }],
        ..enchantment("Knighthood", cost(&[generic(2), w()]))
    }
}

/// Levitation — {2}{U}{U}. Your creatures have flying.
pub fn levitation() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have flying.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Flying,
            },
        }],
        ..enchantment("Levitation", cost(&[generic(2), u(), u()]))
    }
}

/// Engineered Plague — {2}{B}. As it enters, choose a creature type; all
/// creatures of that type get -1/-1.
pub fn engineered_plague() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::NameCreatureType { what: Selector::This },
        }],
        static_abilities: vec![StaticAbility {
            description: "All creatures of the chosen type get -1/-1.",
            effect: StaticEffect::AnthemForChosenType {
                power: -1,
                toughness: -1,
                exclude_source: false,
                opponents: false,
                all_players: true,
                per_counter: None,
            },
        }],
        ..enchantment("Engineered Plague", cost(&[generic(2), b()]))
    }
}

/// Ghitu War Cry — {2}{R}. {R}: target creature gets +1/+0.
pub fn ghitu_war_cry() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..enchantment("Ghitu War Cry", cost(&[generic(2), r()]))
    }
}

/// Delusions of Mediocrity — {3}{U}. Gain 10 on entry, lose 10 when it leaves.
pub fn delusions_of_mediocrity() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(10) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::LoseLife { who: Selector::You, amount: Value::Const(10) },
            },
        ],
        ..enchantment("Delusions of Mediocrity", cost(&[generic(3), u()]))
    }
}

/// Subversion — {3}{B}{B}. Each upkeep, drain each opponent for 1.
pub fn subversion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: crate::effect::shortcut::drain(1),
        }],
        ..enchantment("Subversion", cost(&[generic(3), b(), b()]))
    }
}

/// Second Chance — {2}{U}. At 5 or less life, sacrifice it for an extra turn.
pub fn second_chance() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                .with_filter(Predicate::PlayerLifeAtMost { who: PlayerRef::You, life: 5 }),
            effect: Effect::Seq(vec![
                Effect::SacrificeSource,
                Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE },
            ]),
        }],
        ..enchantment("Second Chance", cost(&[generic(2), u()]))
    }
}

/// Impending Disaster — {1}{R}. With seven-plus lands out, it sacrifices
/// itself and blows up every land.
pub fn impending_disaster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl).with_filter(
                Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Land),
                    n: Value::Const(7),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::SacrificeSource,
                Effect::Destroy { what: Selector::EachPermanent(R::Land) },
            ]),
        }],
        ..enchantment("Impending Disaster", cost(&[generic(1), r()]))
    }
}

/// Planar Collapse — {1}{W}. With four-plus creatures out, it sacrifices
/// itself and wraths the board.
pub fn planar_collapse() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl).with_filter(
                Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Creature),
                    n: Value::Const(4),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::SacrificeSource,
                Effect::CantBeRegeneratedThisTurn {
                    what: Selector::EachPermanent(R::Creature),
                },
                Effect::Destroy { what: Selector::EachPermanent(R::Creature) },
            ]),
        }],
        ..enchantment("Planar Collapse", cost(&[generic(1), w()]))
    }
}

/// Brink of Madness — {2}{B}{B}. Empty-handed on your upkeep, it trades itself
/// for an opponent's hand.
pub fn brink_of_madness() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                .with_filter(Predicate::ValueAtMost(
                    Value::HandSizeOf(PlayerRef::You),
                    Value::Const(0),
                )),
            effect: Effect::Seq(vec![
                Effect::SacrificeSource,
                Effect::Discard {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::HandSizeOf(PlayerRef::Target(0)),
                    random: false,
                },
            ]),
        }],
        ..enchantment("Brink of Madness", cost(&[generic(2), b(), b()]))
    }
}

// ── Auras ───────────────────────────────────────────────────────────────────

/// Cessation — {2}{W} Aura. The host can't attack; the Aura recurs.
pub fn cessation() -> CardDefinition {
    sticky_aura(
        "Cessation",
        cost(&[generic(2), w()]),
        EquipBonus { keywords: vec![Keyword::CantAttack], ..Default::default() },
    )
}

/// Sluggishness — {1}{R} Aura. The host can't block; the Aura recurs.
pub fn sluggishness() -> CardDefinition {
    sticky_aura(
        "Sluggishness",
        cost(&[generic(1), r()]),
        EquipBonus { keywords: vec![Keyword::CantBlock], ..Default::default() },
    )
}

/// Sleeper's Guile — {2}{B} Aura. The host has fear; the Aura recurs.
pub fn sleepers_guile() -> CardDefinition {
    sticky_aura(
        "Sleeper's Guile",
        cost(&[generic(2), b()]),
        EquipBonus { keywords: vec![Keyword::Fear], ..Default::default() },
    )
}

/// Granite Grip — {2}{R} Aura. +1/+0 per Mountain you control.
pub fn granite_grip() -> CardDefinition {
    CardDefinition {
        name: "Granite Grip",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            scale: Some(crate::card::EquipScale {
                filter: R::HasLandType(LandType::Mountain),
                per_power: 1,
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Iron Maiden — {3}. Each opponent's upkeep, it burns them for their hand
/// size minus four.
pub fn iron_maiden() -> CardDefinition {
    CardDefinition {
        name: "Iron Maiden",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::OpponentControl),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::NonNeg(Box::new(Value::Diff(
                    Box::new(Value::HandSizeOf(PlayerRef::ActivePlayer)),
                    Box::new(Value::Const(4)),
                ))),
            },
        }],
        ..Default::default()
    }
}

/// Wheel of Torture — {3}. Each opponent's upkeep, it burns them for three
/// minus their hand size.
pub fn wheel_of_torture() -> CardDefinition {
    CardDefinition {
        name: "Wheel of Torture",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::OpponentControl),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::NonNeg(Box::new(Value::Diff(
                    Box::new(Value::Const(3)),
                    Box::new(Value::HandSizeOf(PlayerRef::ActivePlayer)),
                ))),
            },
        }],
        ..Default::default()
    }
}

/// Jhoira's Toolbox — {2} 1/1 Insect. {2}: regenerate an artifact creature.
pub fn jhoiras_toolbox() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Regenerate {
                what: target_filtered(R::Creature.and(R::Artifact)),
            },
            ..Default::default()
        }],
        ..creature("Jhoira's Toolbox", cost(&[generic(2)]), vec![CreatureType::Insect], 1, 1)
    }
}

/// Thran Lens — {2}. All permanents are colorless.
pub fn thran_lens() -> CardDefinition {
    CardDefinition {
        name: "Thran Lens",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "All permanents are colorless.",
            effect: StaticEffect::GrantColorless {
                applies_to: Selector::EachPermanent(R::Permanent),
            },
        }],
        ..Default::default()
    }
}

/// Urza's Blueprints — {6}. Echo {6}; {T}: draw a card.
pub fn urzas_blueprints() -> CardDefinition {
    CardDefinition {
        name: "Urza's Blueprints",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Echo(cost(&[generic(6)]))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ring of Gix — {3}. Echo {3}; {1}, {T}: tap an artifact, creature, or land.
pub fn ring_of_gix() -> CardDefinition {
    CardDefinition {
        name: "Ring of Gix",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Echo(cost(&[generic(3)]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::Tap {
                what: target_filtered(R::Artifact.or(R::Creature).or(R::Land)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Thran War Machine — {4} 4/5 Construct. Echo {4}; attacks each combat if
/// able.
pub fn thran_war_machine() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Echo(cost(&[generic(4)])), Keyword::MustAttack],
        ..creature("Thran War Machine", cost(&[generic(4)]), vec![CreatureType::Construct], 4, 5)
    }
}

/// Simian Grunts — {2}{G} 3/4. Flash; echo {2}{G}.
pub fn simian_grunts() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Echo(cost(&[generic(2), g()]))],
        ..creature("Simian Grunts", cost(&[generic(2), g()]), vec![CreatureType::Ape], 3, 4)
    }
}

// ── Wave 2 ──────────────────────────────────────────────────────────────────

/// Bouncing Beebles — {2}{U} 2/2. Unblockable while the defender has an
/// artifact.
pub fn bouncing_beebles() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedIfDefenderControls(Box::new(R::Artifact))],
        ..creature("Bouncing Beebles", cost(&[generic(2), u()]), vec![CreatureType::Beeble], 2, 2)
    }
}

/// Gang of Elk — {5}{G} 5/4. +2/+2 per blocker when it becomes blocked.
pub fn gang_of_elk() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Times(
                    Box::new(Value::BlockersOf(Box::new(Selector::This))),
                    Box::new(Value::Const(2)),
                ),
                toughness: Value::Times(
                    Box::new(Value::BlockersOf(Box::new(Selector::This))),
                    Box::new(Value::Const(2)),
                ),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Gang of Elk",
            cost(&[generic(5), g()]),
            vec![CreatureType::Elk, CreatureType::Beast],
            5,
            4,
        )
    }
}

/// Last-Ditch Effort — {R} Instant. Sacrifice any number of creatures for that
/// much damage.
pub fn last_ditch_effort() -> CardDefinition {
    instant(
        "Last-Ditch Effort",
        cost(&[r()]),
        Effect::Seq(vec![
            Effect::SacrificeAnyNumber {
                who: PlayerRef::You,
                filter: R::Creature,
                per_each: Box::new(Effect::Noop),
            },
            Effect::DealDamage { to: target_any(), amount: Value::SacrificedCount },
        ]),
    )
}

/// Parch — {1}{R} Instant. Two to anything, or four to a blue creature.
pub fn parch() -> CardDefinition {
    instant(
        "Parch",
        cost(&[generic(1), r()]),
        Effect::ChooseMode(vec![
            deal(2, target_any()),
            deal(4, target_filtered(R::Creature.and(R::HasColor(Color::Blue)))),
        ]),
    )
}

/// Rank and File — {2}{B}{B} 3/3. ETB shrinks every green creature.
pub fn rank_and_file() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::HasColor(Color::Green))),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Rank and File",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Zombie],
            3,
            3,
        )
    }
}

/// Raven Familiar — {2}{U} 1/2. Flying, echo {2}{U}; ETB digs three for one.
pub fn raven_familiar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Echo(cost(&[generic(2), u()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(3),
                rest_to_graveyard: false,
                pick_filter: None,
                take: None,
                to_battlefield: false,
                gain_life_if_pick: None,
                gain_life_greatest_power_rest: false,
                optional: false,
                picked_lands_to_battlefield: false,
                rest_bottom_random: false,
                rest_to_exile: false,
            },
        }],
        ..creature("Raven Familiar", cost(&[generic(2), u()]), vec![CreatureType::Bird], 1, 2)
    }
}

/// Repopulate — {1}{G} Instant. Shuffle a graveyard's creatures back in;
/// cycling {2}.
pub fn repopulate() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        ..instant(
            "Repopulate",
            cost(&[generic(1), g()]),
            Effect::ShuffleFilteredGraveyardIntoLibrary {
                who: PlayerRef::Target(0),
                filter: R::Creature,
            },
        )
    }
}

/// Scrapheap — {3}. Gain 1 whenever an artifact or enchantment of yours dies.
pub fn scrapheap() -> CardDefinition {
    CardDefinition {
        name: "Scrapheap",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact.or(R::Enchantment),
                }),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Multani's Presence — {G}. Draw whenever one of your spells is countered.
pub fn multanis_presence() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCountered, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::OwnedByYou,
                }),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..enchantment("Multani's Presence", cost(&[g()]))
    }
}

/// Tethered Skirge — {2}{B} 2/2 Flying. Targeting it costs you a life.
pub fn tethered_skirge() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
            effect: Effect::LoseLife { who: Selector::You, amount: Value::ONE },
        }],
        ..creature(
            "Tethered Skirge",
            cost(&[generic(2), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Imp],
            2,
            2,
        )
    }
}

/// Palinchron — {5}{U}{U} 4/5. Flying; ETB untaps seven lands; {2}{U}{U}
/// bounces it.
pub fn palinchron() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Untap {
                what: Selector::EachPermanent(R::Land.and(R::Tapped)),
                up_to: Some(Value::Const(7)),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u(), u()]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..creature("Palinchron", cost(&[generic(5), u(), u()]), vec![CreatureType::Illusion], 4, 5)
    }
}

/// Viashino Heretic — {2}{R} 1/3. {1}{R}, {T}: blow up an artifact and burn its
/// controller for its mana value.
pub fn viashino_heretic() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
                },
                Effect::Destroy { what: target_filtered(R::Artifact) },
            ]),
            ..Default::default()
        }],
        ..creature("Viashino Heretic", cost(&[generic(2), r()]), vec![CreatureType::Lizard], 1, 3)
    }
}

/// Weatherseed Elf — {G} 1/1. {T}: hand out forestwalk.
pub fn weatherseed_elf() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Landwalk(LandType::Forest),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Weatherseed Elf", cost(&[g()]), vec![CreatureType::Elf], 1, 1)
    }
}

/// Tinker — {2}{U} Sorcery. Sacrifice an artifact to tutor any artifact onto
/// the battlefield.
pub fn tinker() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Artifact,
            count: 1,
        }],
        ..sorcery(
            "Tinker",
            cost(&[generic(2), u()]),
            Effect::Search {
                who: PlayerRef::You,
                filter: R::Artifact,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        )
    }
}

/// Viashino Bey — {2}{R}{R} 4/3. Its attack drags the rest of your team in.
pub fn viashino_bey() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::MustAttack,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Viashino Bey", cost(&[generic(2), r(), r()]), vec![CreatureType::Lizard], 4, 3)
    }
}

/// Pyromancy — {2}{R}{R}. {3}, discard at random: burn for the discard's mana
/// value.
pub fn pyromancy() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            discard_cost: Some((R::Any, 1)),
            discard_cost_random: true,
            effect: Effect::DealDamage {
                to: target_any(),
                amount: Value::LastDiscardedManaValue,
            },
            ..Default::default()
        }],
        ..enchantment("Pyromancy", cost(&[generic(2), r(), r()]))
    }
}

/// Treefolk Mystic — {3}{G} 2/4. Combat with it strips the other creature's
/// Auras.
pub fn treefolk_mystic() -> CardDefinition {
    let strip = Effect::Destroy {
        what: Selector::AttachedToMe(Box::new(Selector::CreaturesInCombatWith(Box::new(
            Selector::This,
        )))),
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: strip.clone(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                effect: strip,
            },
        ],
        ..creature("Treefolk Mystic", cost(&[generic(3), g()]), vec![CreatureType::Treefolk], 2, 4)
    }
}

/// Slow Motion — {2}{U} Aura. The host's controller pays {2} each upkeep or
/// loses it; the Aura recurs.
pub fn slow_motion() -> CardDefinition {
    let mut def = sticky_aura("Slow Motion", cost(&[generic(2), u()]), EquipBonus::default());
    def.equipped_bonus.as_mut().unwrap().triggered_abilities.push(TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
        effect: Effect::SacrificeSourceUnlessPay { cost: cost(&[generic(2)]) },
    });
    def
}

/// Walking Sponge — {1}{U} 1/1. {T}: strip flying, first strike, or trample.
pub fn walking_sponge() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::ChooseMode(vec![
                Effect::LoseKeywordThisTurn {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Flying,
                },
                Effect::LoseKeywordThisTurn {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::FirstStrike,
                },
                Effect::LoseKeywordThisTurn {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Trample,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Walking Sponge", cost(&[generic(1), u()]), vec![CreatureType::Sponge], 1, 1)
    }
}

/// Rivalry — {2}{R}. Each upkeep, the land leader takes 2.
pub fn rivalry() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
                .with_filter(Predicate::PlayerControlsMostOf {
                    who: PlayerRef::ActivePlayer,
                    filter: R::Land,
                }),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::Const(2),
            },
        }],
        ..enchantment("Rivalry", cost(&[generic(2), r()]))
    }
}

// ── Wave 3: the "if this permanent is an enchantment" animations ────────────

/// An Opal-cycle enchantment: when `event` fires and this is still an
/// enchantment, it permanently becomes a `p`/`t` creature of `types`.
fn opal(
    name: &'static str,
    c: crate::mana::ManaCost,
    event: EventSpec,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            // `with_filter` REPLACES, so fold the caller's gate in rather than
            // dropping it.
            event: {
                let still_an_enchantment = Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::Enchantment.and(R::Noncreature),
                };
                let combined = match event.filter.clone() {
                    Some(f) => Predicate::All(vec![f, still_an_enchantment]),
                    None => still_an_enchantment,
                };
                event.with_filter(combined)
            },
            effect: Effect::BecomeCreature {
                what: Selector::This,
                power: Value::Const(p),
                toughness: Value::Const(t),
                creature_types: types,
                keywords,
                duration: Duration::Permanent,
            },
        }],
        ..enchantment(name, c)
    }
}

/// No Mercy — {2}{B}{B}. Any creature that damages you dies.
pub fn no_mercy() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDamaged, EventScope::OpponentSourceDamagedYou),
            effect: Effect::Destroy { what: Selector::TriggerSource },
        }],
        ..enchantment("No Mercy", cost(&[generic(2), b(), b()]))
    }
}

/// Opal Avenger — {2}{W}. At 10 or less life it wakes up as a 3/5 Soldier.
pub fn opal_avenger() -> CardDefinition {
    opal(
        "Opal Avenger",
        cost(&[generic(2), w()]),
        EventSpec::new(EventKind::LifeLost, EventScope::YourControl)
            .with_filter(Predicate::PlayerLifeAtMost { who: PlayerRef::You, life: 10 }),
        vec![CreatureType::Soldier],
        3,
        5,
        vec![],
    )
}

/// Opal Champion — {2}{W}. An opponent's creature spell wakes it up as a 3/3
/// first-striking Knight.
pub fn opal_champion() -> CardDefinition {
    opal(
        "Opal Champion",
        cost(&[generic(2), w()]),
        EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
            Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
        ),
        vec![CreatureType::Knight],
        3,
        3,
        vec![Keyword::FirstStrike],
    )
}

/// Hidden Gibbons — {G}. An opponent's instant wakes it up as a 4/4 Ape.
pub fn hidden_gibbons() -> CardDefinition {
    opal(
        "Hidden Gibbons",
        cost(&[g()]),
        EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
            Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasCardType(CardType::Instant),
            },
        ),
        vec![CreatureType::Ape],
        4,
        4,
        vec![],
    )
}

/// Lurking Skirge — {1}{B}. A creature dying under an opponent wakes it up as
/// a 3/2 flying Phyrexian Imp.
pub fn lurking_skirge() -> CardDefinition {
    opal(
        "Lurking Skirge",
        cost(&[generic(1), b()]),
        EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
        vec![CreatureType::Phyrexian, CreatureType::Imp],
        3,
        2,
        vec![Keyword::Flying],
    )
}

/// Crawlspace — {3}. No more than two creatures can attack you each combat.
pub fn crawlspace() -> CardDefinition {
    CardDefinition {
        name: "Crawlspace",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "No more than two creatures can attack you each combat.",
            effect: StaticEffect::AttackerCapAgainstController { n: 2 },
        }],
        ..Default::default()
    }
}

/// Treacherous Link — {1}{B} Aura. Damage aimed at the host lands on its
/// controller instead.
pub fn treacherous_link() -> CardDefinition {
    CardDefinition {
        name: "Treacherous Link",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::DamageToThisGoesToItsController],
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── The last six ULG gaps ───────────────────────────────────────────────────

/// Angel's Trumpet — {3}. Vigilance for everyone, then each end step the active
/// player's idle creatures tap and bite them for that many.
pub fn angels_trumpet() -> CardDefinition {
    CardDefinition {
        name: "Angel's Trumpet",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "All creatures have vigilance.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature),
                keyword: Keyword::Vigilance,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::Seq(vec![
                Effect::Tap {
                    what: Selector::ControlledBy {
                        who: PlayerRef::ActivePlayer,
                        filter: R::And(
                            Box::new(R::Creature),
                            Box::new(R::And(
                                Box::new(R::Untapped),
                                Box::new(R::Not(Box::new(R::AttackedThisTurn))),
                            )),
                        ),
                    },
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ActivePlayer),
                    amount: Value::PermanentsTappedThisEffect,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Aura Flux — {2}{U}. Every other enchantment carries an upkeep {2} tax or
/// gets sacrificed.
pub fn aura_flux() -> CardDefinition {
    CardDefinition {
        name: "Aura Flux",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Other enchantments have \"At the beginning of your upkeep, sacrifice this enchantment unless you pay {2}.\"",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::And(Box::new(R::Enchantment), Box::new(R::OtherThanSource)),
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(
                        EventKind::StepBegins(TurnStep::Upkeep),
                        EventScope::YourControl,
                    ),
                    effect: Effect::UnlessPlayerPays {
                        who: PlayerRef::You,
                        cost: crate::card::WardCost::generic(2),
                        then: Box::new(Effect::SacrificeSource),
                        if_paid: None,
                    },
                }),
            },
        }],
        ..Default::default()
    }
}

/// Damping Engine — {4}. Whoever is ahead on permanents can't develop their
/// board unless they sacrifice something for the turn.
pub fn damping_engine() -> CardDefinition {
    CardDefinition {
        name: "Damping Engine",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "A player who controls more permanents than each other player can't play lands or cast artifact, creature, or enchantment spells.",
            effect: StaticEffect::MostPermanentsCantPlay,
        }],
        activated_abilities: vec![ActivatedAbility {
            any_player: true,
            sac_other_filter: Some((R::Any, 1)),
            effect: Effect::IgnoreStaticFromSourceThisTurn,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Martyr's Cause — {2}{W}. Sacrifice a creature to blank the next damage
/// event from a source of your choice.
pub fn martyrs_cause() -> CardDefinition {
    CardDefinition {
        name: "Martyr's Cause",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::PreventNextEventFromChosenSourceAnywhere,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Memory Jar — {5}. Everyone stashes their hand, draws seven, and gets the
/// stash back at the next end step.
pub fn memory_jar() -> CardDefinition {
    CardDefinition {
        name: "Memory Jar",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::EachPlayerExilesHandDrawsSeven,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Thran Weaponry — {4}. Echo {4}; stays tapped by choice to keep the team
/// pumped.
pub fn thran_weaponry() -> CardDefinition {
    CardDefinition {
        name: "Thran Weaponry",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Echo(cost(&[generic(4)])), Keyword::MayChooseNotToUntap],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::WhileSourceTapped,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
