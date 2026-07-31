//! Urza's Saga (USG) gap closure. Tests in `classic_sets/usg`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CreatureType, EventKind,
    EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R, Subtypes,
    TriggeredAbility, Value,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest,
    shortcut::{target_any, target_filtered},
};
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

fn cycling_two() -> Keyword {
    Keyword::Cycling(cost(&[generic(2)]))
}

fn regenerate_for(c: crate::mana::ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: c,
        effect: Effect::Regenerate { what: Selector::This },
        ..Default::default()
    }
}

// ── Echo bodies ─────────────────────────────────────────────────────────────

/// Acridian — {1}{G} 2/4 with echo.
pub fn acridian() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Echo(cost(&[generic(1), g()]))],
        ..creature("Acridian", cost(&[generic(1), g()]), vec![CreatureType::Insect], 2, 4)
    }
}

/// Albino Troll — {1}{G} 3/3 with echo that regenerates.
pub fn albino_troll() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Echo(cost(&[generic(1), g()]))],
        activated_abilities: vec![regenerate_for(cost(&[generic(1), g()]))],
        ..creature("Albino Troll", cost(&[generic(1), g()]), vec![CreatureType::Troll], 3, 3)
    }
}

/// Cradle Guard — {1}{G}{G} 4/4 trampler with echo.
pub fn cradle_guard() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Echo(cost(&[generic(1), g(), g()]))],
        ..creature("Cradle Guard", cost(&[generic(1), g(), g()]), vec![CreatureType::Treefolk], 4, 4)
    }
}

/// Goblin Patrol — {R} 2/1 with echo.
pub fn goblin_patrol() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Echo(cost(&[r()]))],
        ..creature("Goblin Patrol", cost(&[r()]), vec![CreatureType::Goblin], 2, 1)
    }
}

/// Goblin War Buggy — {1}{R} 2/2 with haste and echo.
pub fn goblin_war_buggy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste, Keyword::Echo(cost(&[generic(1), r()]))],
        ..creature("Goblin War Buggy", cost(&[generic(1), r()]), vec![CreatureType::Goblin], 2, 2)
    }
}

/// Pouncing Jaguar — {G} 2/2 with echo.
pub fn pouncing_jaguar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Echo(cost(&[g()]))],
        ..creature("Pouncing Jaguar", cost(&[g()]), vec![CreatureType::Cat], 2, 2)
    }
}

// ── Cycling bodies ──────────────────────────────────────────────────────────

/// Disciple of Grace — {1}{W} 1/2 with protection from black and cycling.
pub fn disciple_of_grace() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Black), cycling_two()],
        ..creature(
            "Disciple of Grace",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            2,
        )
    }
}

/// Disciple of Law — {1}{W} 1/2 with protection from red and cycling.
pub fn disciple_of_law() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Red), cycling_two()],
        ..creature(
            "Disciple of Law",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            2,
        )
    }
}

/// Pendrell Drake — {3}{U} 2/3 flier with cycling.
pub fn pendrell_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, cycling_two()],
        ..creature("Pendrell Drake", cost(&[generic(3), u()]), vec![CreatureType::Drake], 2, 3)
    }
}

/// Sandbar Merfolk — {U} 1/1 with cycling.
pub fn sandbar_merfolk() -> CardDefinition {
    CardDefinition {
        keywords: vec![cycling_two()],
        ..creature("Sandbar Merfolk", cost(&[u()]), vec![CreatureType::Merfolk], 1, 1)
    }
}

/// Sandbar Serpent — {4}{U} 3/4 with cycling.
pub fn sandbar_serpent() -> CardDefinition {
    CardDefinition {
        keywords: vec![cycling_two()],
        ..creature("Sandbar Serpent", cost(&[generic(4), u()]), vec![CreatureType::Serpent], 3, 4)
    }
}

/// Clear — {1}{W}. Enchantment removal, or a card.
pub fn clear() -> CardDefinition {
    CardDefinition {
        keywords: vec![cycling_two()],
        ..instant(
            "Clear",
            cost(&[generic(1), w()]),
            Effect::Destroy { what: target_filtered(R::Enchantment) },
        )
    }
}

/// Expunge — {2}{B}. Unregenerable removal for a nonartifact, nonblack
/// creature, or a card.
pub fn expunge() -> CardDefinition {
    CardDefinition {
        keywords: vec![cycling_two()],
        ..instant(
            "Expunge",
            cost(&[generic(2), b()]),
            Effect::DestroyNoRegen {
                what: target_filtered(
                    R::Creature
                        .and(R::Not(Box::new(R::Artifact)))
                        .and(R::Not(Box::new(R::HasColor(Color::Black)))),
                ),
            },
        )
    }
}

/// Lay Waste — {3}{R}. A land, or a card.
pub fn lay_waste() -> CardDefinition {
    CardDefinition {
        keywords: vec![cycling_two()],
        ..sorcery(
            "Lay Waste",
            cost(&[generic(3), r()]),
            Effect::Destroy { what: target_filtered(R::Land) },
        )
    }
}

/// Scrap — {2}{R}. An artifact, or a card.
pub fn scrap() -> CardDefinition {
    CardDefinition {
        keywords: vec![cycling_two()],
        ..instant(
            "Scrap",
            cost(&[generic(2), r()]),
            Effect::Destroy { what: target_filtered(R::Artifact) },
        )
    }
}

/// Rescind — {1}{U}{U}. A bounce, or a card.
pub fn rescind() -> CardDefinition {
    CardDefinition {
        keywords: vec![cycling_two()],
        ..instant(
            "Rescind",
            cost(&[generic(1), u(), u()]),
            Effect::Move {
                what: target_filtered(R::Any),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        )
    }
}

// ── Plain bodies ────────────────────────────────────────────────────────────

/// Argothian Swine — {3}{G} 3/3 trampler.
pub fn argothian_swine() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..creature("Argothian Swine", cost(&[generic(3), g()]), vec![CreatureType::Boar], 3, 3)
    }
}

/// Blanchwood Treefolk — {4}{G} 4/5.
pub fn blanchwood_treefolk() -> CardDefinition {
    creature("Blanchwood Treefolk", cost(&[generic(4), g()]), vec![CreatureType::Treefolk], 4, 5)
}

/// Crazed Skirge — {3}{B} 2/2 with flying and haste.
pub fn crazed_skirge() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        ..creature(
            "Crazed Skirge",
            cost(&[generic(3), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Imp],
            2,
            2,
        )
    }
}

/// Guma — {2}{R} 2/2 with protection from blue.
pub fn guma() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Blue)],
        ..creature("Guma", cost(&[generic(2), r()]), vec![CreatureType::Cat], 2, 2)
    }
}

/// Goblin Spelunkers — {2}{R} 2/2 mountainwalker.
pub fn goblin_spelunkers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Mountain)],
        ..creature(
            "Goblin Spelunkers",
            cost(&[generic(2), r()]),
            vec![CreatureType::Goblin, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Hawkeater Moth — {3}{G} 1/2 with flying and shroud.
pub fn hawkeater_moth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Shroud],
        ..creature("Hawkeater Moth", cost(&[generic(3), g()]), vec![CreatureType::Insect], 1, 2)
    }
}

/// Pegasus Charger — {2}{W} 2/1 with flying and first strike.
pub fn pegasus_charger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        ..creature("Pegasus Charger", cost(&[generic(2), w()]), vec![CreatureType::Pegasus], 2, 1)
    }
}

/// Serra Zealot — {W} 1/1 first striker.
pub fn serra_zealot() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        ..creature(
            "Serra Zealot",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Sanguine Guard — {1}{B}{B} 2/2 first striker that regenerates.
pub fn sanguine_guard() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        activated_abilities: vec![regenerate_for(cost(&[generic(1), b()]))],
        ..creature(
            "Sanguine Guard",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Zombie, CreatureType::Knight],
            2,
            2,
        )
    }
}

// ── Bodies with abilities ───────────────────────────────────────────────────

/// Angelic Page — {1}{W} 1/1 flier that pumps whoever's in combat.
pub fn angelic_page() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Angelic Page",
            cost(&[generic(1), w()]),
            vec![CreatureType::Angel, CreatureType::Spirit],
            1,
            1,
        )
    }
}

/// Argothian Elder — {3}{G} 2/2 that untaps two lands.
pub fn argothian_elder() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 2,
                filter: R::Land,
                effect: Box::new(Effect::Untap { what: Selector::Target(0), up_to: None }),
            },
            ..Default::default()
        }],
        ..creature(
            "Argothian Elder",
            cost(&[generic(3), g()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            2,
            2,
        )
    }
}

/// Blood Vassal — {2}{B} 2/2 that cashes itself in for two black.
pub fn blood_vassal() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Black, Value::Const(2)),
            },
            ..Default::default()
        }],
        ..creature("Blood Vassal", cost(&[generic(2), b()]), vec![CreatureType::Thrull], 2, 2)
    }
}

/// Carrion Beetles — {B} 1/1 graveyard hate.
pub fn carrion_beetles() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            tap_cost: true,
            effect: Effect::ExileUpToNFromGraveyards {
                count: Value::Const(3),
                of: Some(PlayerRef::Target(0)),
                single: true,
            },
            ..Default::default()
        }],
        ..creature("Carrion Beetles", cost(&[b()]), vec![CreatureType::Insect], 1, 1)
    }
}

/// Disruptive Student — {2}{U} 1/1 that taxes a spell.
pub fn disruptive_student() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::CounterUnlessPaid {
                what: Selector::Target(0),
                mana_cost: cost(&[generic(1)]),
                exile: false,
                extra_generic: None,
            },
            ..Default::default()
        }],
        ..creature(
            "Disruptive Student",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Elvish Herder — {G} 1/1 that hands out trample.
pub fn elvish_herder() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Elvish Herder", cost(&[g()]), vec![CreatureType::Elf], 1, 1)
    }
}

/// Elvish Lyrist — {G} 1/1 that trades itself for an enchantment.
pub fn elvish_lyrist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Enchantment) },
            ..Default::default()
        }],
        ..creature("Elvish Lyrist", cost(&[g()]), vec![CreatureType::Elf], 1, 1)
    }
}

/// Fire Ants — {2}{R} 2/1 that sweeps the ground for one.
pub fn fire_ants() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: Selector::EachPermanent(
                    R::Creature
                        .and(R::OtherThanSource)
                        .and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                ),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature("Fire Ants", cost(&[generic(2), r()]), vec![CreatureType::Insect], 2, 1)
    }
}

/// Horseshoe Crab — {2}{U} 1/3 that untaps for {U}.
pub fn horseshoe_crab() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::Untap { what: Selector::This, up_to: None },
            ..Default::default()
        }],
        ..creature("Horseshoe Crab", cost(&[generic(2), u()]), vec![CreatureType::Crab], 1, 3)
    }
}

/// Hopping Automaton — {3} 2/2 that trades a point for flight.
pub fn hopping_automaton() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Hopping Automaton", cost(&[generic(3)]), vec![CreatureType::Construct], 2, 2)
    }
}

/// Mobile Fort — {4} 0/6 Wall that can charge once a turn.
pub fn mobile_fort() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            once_per_turn: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(3),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                },
                Effect::AttackDespiteDefenderThisTurn { what: Selector::This },
            ]),
            ..Default::default()
        }],
        ..creature("Mobile Fort", cost(&[generic(4)]), vec![CreatureType::Wall], 0, 6)
    }
}

/// Monk Realist — {1}{W} 1/1 that breaks an enchantment on arrival.
pub fn monk_realist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Destroy { what: target_filtered(R::Enchantment) },
        }],
        ..creature(
            "Monk Realist",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Monk, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Order of Yawgmoth — {2}{B}{B} 2/2 with fear that strips a card on contact.
pub fn order_of_yawgmoth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fear],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
                random: false,
            },
        }],
        ..creature(
            "Order of Yawgmoth",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Zombie, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Phyrexian Ghoul — {2}{B} 2/2 that eats its friends for size.
pub fn phyrexian_ghoul() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Phyrexian Ghoul",
            cost(&[generic(2), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Zombie],
            2,
            2,
        )
    }
}

/// Priest of Gix — {2}{B} 2/1 that pays for itself on arrival.
pub fn priest_of_gix() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Black, Value::Const(3)),
            },
        }],
        ..creature(
            "Priest of Gix",
            cost(&[generic(2), b()]),
            vec![
                CreatureType::Phyrexian,
                CreatureType::Human,
                CreatureType::Cleric,
                CreatureType::Minion,
            ],
            2,
            1,
        )
    }
}

/// Ravenous Skirge — {2}{B} 1/1 flier that swells when it attacks.
pub fn ravenous_skirge() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Ravenous Skirge",
            cost(&[generic(2), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Imp],
            1,
            1,
        )
    }
}

/// Sanctum Custodian — {2}{W} 1/2 that taps for a two-point shield.
pub fn sanctum_custodian() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PreventNextDamage { target: target_any(), amount: Value::Const(2) },
            ..Default::default()
        }],
        ..creature(
            "Sanctum Custodian",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            2,
        )
    }
}

/// Sanctum Guardian — {1}{W}{W} 1/4 that blanks one damage event.
pub fn sanctum_guardian() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::PreventNextEventFromChosenSourceAnywhere,
            ..Default::default()
        }],
        ..creature(
            "Sanctum Guardian",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            4,
        )
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Breach — {2}{B}. +2/+0 and fear.
pub fn breach() -> CardDefinition {
    instant(
        "Breach",
        cost(&[generic(2), b()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Fear,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Gaea's Bounty — {2}{G}. Two Forests to hand.
pub fn gaeas_bounty() -> CardDefinition {
    sorcery(
        "Gaea's Bounty",
        cost(&[generic(2), g()]),
        Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::HasLandType(LandType::Forest),
            to: ZoneDest::Hand(PlayerRef::You),
            count: Value::Const(2),
        },
    )
}

/// Goblin Offensive — {X}{1}{R}{R}. X Goblins.
pub fn goblin_offensive() -> CardDefinition {
    sorcery(
        "Goblin Offensive",
        cost(&[x(), generic(1), r(), r()]),
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::XFromCost,
            definition: crate::card::TokenDefinition {
                name: "Goblin".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Red],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Goblin],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
    )
}

/// Headlong Rush — {1}{R}. First strike for the attackers.
pub fn headlong_rush() -> CardDefinition {
    instant(
        "Headlong Rush",
        cost(&[generic(1), r()]),
        Effect::GrantKeyword {
            what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
            keyword: Keyword::FirstStrike,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Heat Ray — {X}{R}. X to a creature.
pub fn heat_ray() -> CardDefinition {
    instant(
        "Heat Ray",
        cost(&[x(), r()]),
        Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::XFromCost },
    )
}

/// Hibernation — {2}{U}. Every green permanent goes home.
pub fn hibernation() -> CardDefinition {
    instant(
        "Hibernation",
        cost(&[generic(2), u()]),
        Effect::Move {
            what: Selector::EachPermanent(R::HasColor(Color::Green)),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
    )
}

/// Humble — {1}{W}. A creature becomes a vanilla 0/1.
pub fn humble() -> CardDefinition {
    instant(
        "Humble",
        cost(&[generic(1), w()]),
        Effect::Seq(vec![
            Effect::LoseAllAbilities {
                what: target_filtered(R::Creature),
                duration: Duration::EndOfTurn,
            },
            Effect::SetBasePT {
                what: Selector::Target(0),
                power: Value::ZERO,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Raze — {R}. Trade a land for a land.
pub fn raze() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::Land,
            count: 1,
        }],
        ..sorcery("Raze", cost(&[r()]), Effect::Destroy { what: target_filtered(R::Land) })
    }
}

/// Redeem — {1}{W}. Two creatures take nothing this turn.
pub fn redeem() -> CardDefinition {
    instant(
        "Redeem",
        cost(&[generic(1), w()]),
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::PreventAllDamageThisTurn { target: Selector::Target(0) }),
        },
    )
}

/// Rewind — {2}{U}{U}. A counter that pays you back.
pub fn rewind() -> CardDefinition {
    instant(
        "Rewind",
        cost(&[generic(2), u(), u()]),
        Effect::Seq(vec![
            Effect::CounterSpell { what: Selector::Target(0) },
            Effect::Untap {
                what: Selector::ControlledBy { who: PlayerRef::You, filter: R::Land },
                up_to: Some(Value::Const(4)),
            },
        ]),
    )
}
