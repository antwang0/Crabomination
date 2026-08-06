//! Invasion (INV) gap-closing wave 2: the mana fixers, the Leech and Djinn
//! cycles, the Domain payoffs, the kicker commons and the utility shell.
//! Tests in `classic_sets/inv_gaps2`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect,
    Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::{LookPick, 
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

fn enters_tapped() -> StaticAbility {
    StaticAbility {
        description: "This land enters tapped.",
        effect: StaticEffect::EntersTapped { applies_to: Selector::This },
    }
}

fn tap_for(colors: Vec<Color>) -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::OfColors(colors, Value::ONE),
        },
        ..Default::default()
    }
}

fn tapland(name: &'static str, colors: Vec<Color>) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![tap_for(colors)],
        ..Default::default()
    }
}

/// Salt Marsh — the Dimir tapland.
pub fn salt_marsh() -> CardDefinition {
    tapland("Salt Marsh", vec![Color::Blue, Color::Black])
}

/// Shivan Oasis — the Gruul tapland.
pub fn shivan_oasis() -> CardDefinition {
    tapland("Shivan Oasis", vec![Color::Red, Color::Green])
}

/// Urborg Volcano — the Rakdos tapland.
pub fn urborg_volcano() -> CardDefinition {
    tapland("Urborg Volcano", vec![Color::Black, Color::Red])
}

fn sac_land(name: &'static str, taps: Color, cracks: Vec<Color>) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![
            tap_for(vec![taps]),
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(cracks),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Irrigation Ditch — {W}, or crack for {G}{U}.
pub fn irrigation_ditch() -> CardDefinition {
    sac_land("Irrigation Ditch", Color::White, vec![Color::Green, Color::Blue])
}

/// Sulfur Vent — {B}, or crack for {U}{R}.
pub fn sulfur_vent() -> CardDefinition {
    sac_land("Sulfur Vent", Color::Black, vec![Color::Blue, Color::Red])
}

/// Tinder Farm — {G}, or crack for {R}{W}.
pub fn tinder_farm() -> CardDefinition {
    sac_land("Tinder Farm", Color::Green, vec![Color::Red, Color::White])
}

fn cameo(name: &'static str, colors: Vec<Color>) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![tap_for(colors)],
        ..Default::default()
    }
}

/// Seashell Cameo — {W} or {U}.
pub fn seashell_cameo() -> CardDefinition {
    cameo("Seashell Cameo", vec![Color::White, Color::Blue])
}

/// Tigereye Cameo — {G} or {W}.
pub fn tigereye_cameo() -> CardDefinition {
    cameo("Tigereye Cameo", vec![Color::Green, Color::White])
}

/// Troll-Horn Cameo — {R} or {G}.
pub fn troll_horn_cameo() -> CardDefinition {
    cameo("Troll-Horn Cameo", vec![Color::Red, Color::Green])
}

fn attendant(name: &'static str, colors: Vec<Color>) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(colors) },
            ..Default::default()
        }],
        ..creature(name, cost(&[generic(5)]), vec![CreatureType::Golem], 3, 3)
    }
}

/// Rith's Attendant — {R}{G}{W}.
pub fn riths_attendant() -> CardDefinition {
    attendant("Rith's Attendant", vec![Color::Red, Color::Green, Color::White])
}

/// Treva's Attendant — {G}{W}{U}.
pub fn trevas_attendant() -> CardDefinition {
    attendant("Treva's Attendant", vec![Color::Green, Color::White, Color::Blue])
}

fn any_color_tap() -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
        ..Default::default()
    }
}

/// Lotus Guardian — {7} 4/4 flying artifact Dragon that taps for any colour.
pub fn lotus_guardian() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![any_color_tap()],
        ..creature("Lotus Guardian", cost(&[generic(7)]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Utopia Tree — {1}{G} 0/2 that taps for any colour.
pub fn utopia_tree() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![any_color_tap()],
        ..creature("Utopia Tree", cost(&[generic(1), g()]), vec![CreatureType::Plant], 0, 2)
    }
}

/// Phyrexian Lens — {3}. {T}, pay 1 life: add one mana of any colour.
pub fn phyrexian_lens() -> CardDefinition {
    CardDefinition {
        name: "Phyrexian Lens",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            life_cost: 1,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Nomadic Elf — {1}{G} 2/2. {1}{G}: add one mana of any colour.
pub fn nomadic_elf() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
            ..Default::default()
        }],
        ..creature(
            "Nomadic Elf",
            cost(&[generic(1), g()]),
            vec![CreatureType::Elf, CreatureType::Nomad],
            2,
            2,
        )
    }
}

/// Quirion Sentinel — {1}{G} 2/1 that rebates a mana of any colour on ETB.
pub fn quirion_sentinel() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::AnyOneColor(Value::ONE),
        })],
        ..creature(
            "Quirion Sentinel",
            cost(&[generic(1), g()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            2,
            1,
        )
    }
}

/// Quirion Trailblazer — {3}{G} 1/2 that may fetch a tapped basic.
pub fn quirion_trailblazer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search for a basic land?".into(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            }),
        })],
        ..creature(
            "Quirion Trailblazer",
            cost(&[generic(3), g()]),
            vec![CreatureType::Elf, CreatureType::Scout],
            1,
            2,
        )
    }
}

// ── The Leech cycle: a discounted body that taxes its own colour ────────────

/// "[Colour] spells you cast cost [pip] more to cast."
fn leech_tax(color: Color, pip: ManaCost, description: &'static str) -> StaticAbility {
    StaticAbility {
        description,
        effect: StaticEffect::ColoredSpellTax { filter: R::HasColor(color), more: pip },
    }
}

/// Alabaster Leech — {W} 1/3. White spells you cast cost {W} more.
pub fn alabaster_leech() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![leech_tax(
            Color::White,
            cost(&[w()]),
            "White spells you cast cost {W} more to cast.",
        )],
        ..creature("Alabaster Leech", cost(&[w()]), vec![CreatureType::Leech], 1, 3)
    }
}

/// Andradite Leech — {2}{B} 2/2 that pumps for {B}. Black spells cost {B} more.
pub fn andradite_leech() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![leech_tax(
            Color::Black,
            cost(&[b()]),
            "Black spells you cast cost {B} more to cast.",
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Andradite Leech", cost(&[generic(2), b()]), vec![CreatureType::Leech], 2, 2)
    }
}

/// Jade Leech — {2}{G}{G} 5/5. Green spells you cast cost {G} more.
pub fn jade_leech() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![leech_tax(
            Color::Green,
            cost(&[g()]),
            "Green spells you cast cost {G} more to cast.",
        )],
        ..creature("Jade Leech", cost(&[generic(2), g(), g()]), vec![CreatureType::Leech], 5, 5)
    }
}

/// Ruby Leech — {1}{R} 2/2 first strike. Red spells you cast cost {R} more.
pub fn ruby_leech() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![leech_tax(
            Color::Red,
            cost(&[r()]),
            "Red spells you cast cost {R} more to cast.",
        )],
        ..creature("Ruby Leech", cost(&[generic(1), r()]), vec![CreatureType::Leech], 2, 2)
    }
}

/// Sapphire Leech — {1}{U} 2/2 flier. Blue spells you cast cost {U} more.
pub fn sapphire_leech() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![leech_tax(
            Color::Blue,
            cost(&[u()]),
            "Blue spells you cast cost {U} more to cast.",
        )],
        ..creature("Sapphire Leech", cost(&[generic(1), u()]), vec![CreatureType::Leech], 2, 2)
    }
}

// ── The Djinn cycle: big bodies that shrink when their colour saturates ─────

/// "This creature gets -2/-2 as long as [colour] is the most common colour
/// among all permanents or is tied for most common."
fn djinn_shrink(color: Color, description: &'static str) -> StaticAbility {
    StaticAbility {
        description,
        effect: StaticEffect::PumpSelfIf {
            condition: Predicate::ColorIsMostCommonAmongPermanents(color),
            power: -2,
            toughness: -2,
            keywords: vec![],
        },
    }
}

/// Goham Djinn — {5}{B} 5/5 that regenerates for {1}{B}.
pub fn goham_djinn() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![djinn_shrink(
            Color::Black,
            "This creature gets -2/-2 as long as black is the most common color among all \
             permanents or is tied for most common.",
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Goham Djinn", cost(&[generic(5), b()]), vec![CreatureType::Djinn], 5, 5)
    }
}

/// Halam Djinn — {5}{R} 6/5 haste.
pub fn halam_djinn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        static_abilities: vec![djinn_shrink(
            Color::Red,
            "This creature gets -2/-2 as long as red is the most common color among all \
             permanents or is tied for most common.",
        )],
        ..creature("Halam Djinn", cost(&[generic(5), r()]), vec![CreatureType::Djinn], 6, 5)
    }
}

/// Ruham Djinn — {5}{W} 5/5 first strike.
pub fn ruham_djinn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![djinn_shrink(
            Color::White,
            "This creature gets -2/-2 as long as white is the most common color among all \
             permanents or is tied for most common.",
        )],
        ..creature("Ruham Djinn", cost(&[generic(5), w()]), vec![CreatureType::Djinn], 5, 5)
    }
}

/// Sulam Djinn — {5}{G} 6/6 trample.
pub fn sulam_djinn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![djinn_shrink(
            Color::Green,
            "This creature gets -2/-2 as long as green is the most common color among all \
             permanents or is tied for most common.",
        )],
        ..creature("Sulam Djinn", cost(&[generic(5), g()]), vec![CreatureType::Djinn], 6, 6)
    }
}

/// Zanam Djinn — {5}{U} 5/6 flier.
pub fn zanam_djinn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![djinn_shrink(
            Color::Blue,
            "This creature gets -2/-2 as long as blue is the most common color among all \
             permanents or is tied for most common.",
        )],
        ..creature("Zanam Djinn", cost(&[generic(5), u()]), vec![CreatureType::Djinn], 5, 6)
    }
}

/// Barrin's Unmaking — {1}{U}. Bounce a permanent of the most common colour.
pub fn barrins_unmaking() -> CardDefinition {
    instant(
        "Barrin's Unmaking",
        cost(&[generic(1), u()]),
        Effect::Move {
            what: target_filtered(R::Permanent.and(R::SharesMostCommonColor)),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
    )
}

/// Tsabo's Assassin — {2}{B}{B} 1/1 that snipes the board's dominant colour.
pub fn tsabos_assassin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DestroyNoRegen {
                what: target_filtered(R::Creature.and(R::SharesMostCommonColor)),
            },
            ..Default::default()
        }],
        ..creature(
            "Tsabo's Assassin",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Zombie, CreatureType::Assassin],
            1,
            1,
        )
    }
}

// ── Damage-threshold replacements ───────────────────────────────────────────

/// Callous Giant — {4}{R}{R} 4/4 that shrugs off every small hit.
pub fn callous_giant() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If a source would deal 3 or less damage to this creature, prevent \
                          that damage.",
            effect: StaticEffect::PreventSmallDamageToThis { max: 3 },
        }],
        ..creature("Callous Giant", cost(&[generic(4), r(), r()]), vec![CreatureType::Giant], 4, 4)
    }
}

/// Divine Presence — {2}{W}. Every big damage event is replaced by 3.
pub fn divine_presence() -> CardDefinition {
    CardDefinition {
        name: "Divine Presence",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If a source would deal 4 or more damage to a permanent or player, \
                          that source deals 3 damage to that permanent or player instead.",
            effect: StaticEffect::CapLargeDamage { at_least: 4, capped: 3 },
        }],
        ..Default::default()
    }
}

// ── Domain payoffs ──────────────────────────────────────────────────────────

fn domain() -> Value {
    Value::DomainCount(PlayerRef::You)
}

fn domain_pump(description: &'static str, applies_to: Selector) -> StaticAbility {
    StaticAbility {
        description,
        effect: StaticEffect::PumpPTByValue { applies_to, power: domain(), toughness: domain() },
    }
}

/// Kavu Scout — {2}{R} 0/2 that grows +1/+0 per basic land type.
pub fn kavu_scout() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Domain — This creature gets +1/+0 for each basic land type among \
                          lands you control.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::This,
                power: domain(),
                toughness: Value::ZERO,
            },
        }],
        ..creature(
            "Kavu Scout",
            cost(&[generic(2), r()]),
            vec![CreatureType::Kavu, CreatureType::Scout],
            0,
            2,
        )
    }
}

/// Wayfaring Giant — {5}{W} 1/3 with a Domain anthem on itself.
pub fn wayfaring_giant() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![domain_pump(
            "Domain — This creature gets +1/+1 for each basic land type among lands you control.",
            Selector::This,
        )],
        ..creature("Wayfaring Giant", cost(&[generic(5), w()]), vec![CreatureType::Giant], 1, 3)
    }
}

/// Strength of Unity — {3}{W} Aura with a Domain anthem.
pub fn strength_of_unity() -> CardDefinition {
    CardDefinition {
        name: "Strength of Unity",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![domain_pump(
            "Domain — Enchanted creature gets +1/+1 for each basic land type among lands you \
             control.",
            Selector::attached_to(Selector::This),
        )],
        ..Default::default()
    }
}

/// Wandering Stream — {2}{G}. Gain 2 life per basic land type.
pub fn wandering_stream() -> CardDefinition {
    sorcery(
        "Wandering Stream",
        cost(&[generic(2), g()]),
        Effect::GainLife { who: Selector::You, amount: Value::Times(Box::new(Value::Const(2)), Box::new(domain())) },
    )
}

/// Ordered Migration — {3}{W}{U}. A 1/1 flying Bird per basic land type.
pub fn ordered_migration() -> CardDefinition {
    sorcery(
        "Ordered Migration",
        cost(&[generic(3), w(), u()]),
        Effect::CreateToken {
            who: PlayerRef::You,
            count: domain(),
            definition: crate::card::TokenDefinition {
                name: "Bird".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Blue],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Bird],
                    ..Default::default()
                },
                keywords: vec![Keyword::Flying],
                ..Default::default()
            },
        },
    )
}

/// Power Armor — {4}. {3}, {T}: Domain-sized pump.
pub fn power_armor() -> CardDefinition {
    CardDefinition {
        name: "Power Armor",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: domain(),
                toughness: domain(),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Worldly Counsel — {1}{U}. Look at Domain cards, keep one, bottom the rest.
pub fn worldly_counsel() -> CardDefinition {
    instant(
        "Worldly Counsel",
        cost(&[generic(1), u()]),
        Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: domain(),
    ..Default::default()
})),
    )
}

/// Collapsing Borders — {3}{R}. Domain life, then a flat 3 to the face.
pub fn collapsing_borders() -> CardDefinition {
    CardDefinition {
        name: "Collapsing Borders",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::Player(PlayerRef::TriggerEventPlayer),
                    amount: Value::DomainCount(PlayerRef::TriggerEventPlayer),
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::TriggerEventPlayer),
                    amount: Value::Const(3),
                },
            ]),
        }],
        ..Default::default()
    }
}

// ── Kicker commons ──────────────────────────────────────────────────────────

fn kicked_etb(counters: i32, keywords: Vec<Keyword>) -> TriggeredAbility {
    let mut body = vec![Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(counters),
    }];
    body.extend(keywords.into_iter().map(|keyword| Effect::GrantKeyword {
        what: Selector::This,
        keyword,
        duration: Duration::Permanent,
    }));
    etb(Effect::If {
        cond: Predicate::SpellWasKicked,
        then: Box::new(Effect::Seq(body)),
        else_: Box::new(Effect::Noop),
    })
}

/// Kavu Aggressor — {2}{R} 3/2 that can't block. Kicker {4} for a counter.
pub fn kavu_aggressor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBlock, Keyword::Kicker(cost(&[generic(4)]))],
        triggered_abilities: vec![kicked_etb(1, vec![])],
        ..creature("Kavu Aggressor", cost(&[generic(2), r()]), vec![CreatureType::Kavu], 3, 2)
    }
}

/// Kavu Titan — {1}{G} 2/2. Kicker {2}{G} for three counters and trample.
pub fn kavu_titan() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(2), g()]))],
        triggered_abilities: vec![kicked_etb(3, vec![Keyword::Trample])],
        ..creature("Kavu Titan", cost(&[generic(1), g()]), vec![CreatureType::Kavu], 2, 2)
    }
}

/// Llanowar Elite — {G} 1/1 trample. Kicker {8} for five counters.
pub fn llanowar_elite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Kicker(cost(&[generic(8)]))],
        triggered_abilities: vec![kicked_etb(5, vec![])],
        ..creature("Llanowar Elite", cost(&[g()]), vec![CreatureType::Elf], 1, 1)
    }
}

/// Pincer Spider — {2}{G} 2/3 reach. Kicker {3} for a counter.
pub fn pincer_spider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Kicker(cost(&[generic(3)]))],
        triggered_abilities: vec![kicked_etb(1, vec![])],
        ..creature("Pincer Spider", cost(&[generic(2), g()]), vec![CreatureType::Spider], 2, 3)
    }
}

/// Pouncing Kavu — {1}{R} 1/1 first strike. Kicker {2}{R} for counters + haste.
pub fn pouncing_kavu() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Kicker(cost(&[generic(2), r()]))],
        triggered_abilities: vec![kicked_etb(2, vec![Keyword::Haste])],
        ..creature("Pouncing Kavu", cost(&[generic(1), r()]), vec![CreatureType::Kavu], 1, 1)
    }
}

/// Urborg Skeleton — {B} 0/1 regenerator. Kicker {3} for a counter.
pub fn urborg_skeleton() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(3)]))],
        triggered_abilities: vec![kicked_etb(1, vec![])],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Urborg Skeleton", cost(&[b()]), vec![CreatureType::Skeleton], 0, 1)
    }
}

/// Vodalian Serpent — {3}{U} 2/2 that needs an Island opposite. Kicker {2}.
pub fn vodalian_serpent() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::CanAttackOnlyIfDefenderControls(Box::new(R::HasLandType(LandType::Island))),
            Keyword::Kicker(cost(&[generic(2)])),
        ],
        triggered_abilities: vec![kicked_etb(4, vec![])],
        ..creature("Vodalian Serpent", cost(&[generic(3), u()]), vec![CreatureType::Serpent], 2, 2)
    }
}

/// Hypnotic Cloud — {1}{B}. Discard one, or three when kicked.
pub fn hypnotic_cloud() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(4)]))],
        ..sorcery(
            "Hypnotic Cloud",
            cost(&[generic(1), b()]),
            Effect::Discard {
                who: Selector::Target(0),
                amount: Value::IfPred {
                    pred: Box::new(Predicate::SpellWasKicked),
                    then: Box::new(Value::Const(3)),
                    else_: Box::new(Value::ONE),
                },
                random: false,
            },
        )
    }
}

/// Overload — {R}. Destroy a small artifact; kicked, a bigger one.
pub fn overload() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(2)]))],
        ..instant(
            "Overload",
            cost(&[r()]),
            Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::Destroy {
                    what: target_filtered(R::Artifact.and(R::ManaValueAtMost(5))),
                }),
                else_: Box::new(Effect::Destroy {
                    what: target_filtered(R::Artifact.and(R::ManaValueAtMost(2))),
                }),
            },
        )
    }
}

/// Prohibit — {1}{U}. Counter a cheap spell; kicked, a costlier one.
pub fn prohibit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(2)]))],
        ..instant(
            "Prohibit",
            cost(&[generic(1), u()]),
            Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::CounterSpell {
                    what: target_filtered(R::IsSpellOnStack.and(R::ManaValueAtMost(4))),
                }),
                else_: Box::new(Effect::CounterSpell {
                    what: target_filtered(R::IsSpellOnStack.and(R::ManaValueAtMost(2))),
                }),
            },
        )
    }
}

/// Savage Offensive — {1}{R}. Team first strike; kicked, +1/+1 too.
pub fn savage_offensive() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[g()]))],
        ..sorcery(
            "Savage Offensive",
            cost(&[generic(1), r()]),
            Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::PumpPT {
                        what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Scorching Lava — {1}{R}. Two damage; kicked, the victim is exiled instead.
pub fn scorching_lava() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[r()]))],
        ..instant(
            "Scorching Lava",
            cost(&[generic(1), r()]),
            Effect::Seq(vec![
                Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(2) },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::Seq(vec![
                        Effect::CantBeRegeneratedThisTurn { what: Selector::Target(0) },
                        Effect::ExileIfWouldDieThisTurn { what: Selector::Target(0) },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Shivan Emissary — {2}{R} 1/1. Kicked, it melts a nonblack creature.
pub fn shivan_emissary() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(1), b()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SpellWasKicked),
            effect: Effect::DestroyNoRegen {
                what: target_filtered(R::Creature.and(R::HasColor(Color::Black).negate())),
            },
        }],
        ..creature(
            "Shivan Emissary",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Tolarian Emissary — {2}{U} 1/2 flier. Kicked, it eats an enchantment.
pub fn tolarian_emissary() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Kicker(cost(&[generic(1), w()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SpellWasKicked),
            effect: Effect::Destroy { what: target_filtered(R::Enchantment) },
        }],
        ..creature(
            "Tolarian Emissary",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            2,
        )
    }
}

/// Urborg Emissary — {2}{B} 3/1. Kicked, it bounces a permanent.
pub fn urborg_emissary() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(1), u()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SpellWasKicked),
            effect: Effect::Move { what: target_filtered(R::Permanent), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
        }],
        ..creature(
            "Urborg Emissary",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            1,
        )
    }
}

/// Verduran Emissary — {2}{G} 2/3. Kicked, it shatters an artifact.
pub fn verduran_emissary() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(1), r()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SpellWasKicked),
            effect: Effect::DestroyNoRegen { what: target_filtered(R::Artifact) },
        }],
        ..creature(
            "Verduran Emissary",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            3,
        )
    }
}

// ── Protection bodies ───────────────────────────────────────────────────────

/// Llanowar Knight — {G}{W} 2/2 with protection from black.
pub fn llanowar_knight() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Black)],
        ..creature(
            "Llanowar Knight",
            cost(&[g(), w()]),
            vec![CreatureType::Elf, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Shivan Zombie — {B}{R} 2/2 with protection from white.
pub fn shivan_zombie() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::White)],
        ..creature(
            "Shivan Zombie",
            cost(&[b(), r()]),
            vec![CreatureType::Phyrexian, CreatureType::Barbarian, CreatureType::Zombie],
            2,
            2,
        )
    }
}

/// Vodalian Zombie — {U}{B} 2/2 with protection from green.
pub fn vodalian_zombie() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Green)],
        ..creature(
            "Vodalian Zombie",
            cost(&[u(), b()]),
            vec![CreatureType::Merfolk, CreatureType::Zombie],
            2,
            2,
        )
    }
}

/// Yavimaya Barbarian — {R}{G} 2/2 with protection from blue.
pub fn yavimaya_barbarian() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Blue)],
        ..creature(
            "Yavimaya Barbarian",
            cost(&[r(), g()]),
            vec![CreatureType::Elf, CreatureType::Barbarian],
            2,
            2,
        )
    }
}

/// Shoreline Raider — {2}{U} 2/2 with protection from Kavu.
pub fn shoreline_raider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::ProtectionFromCreatureType(CreatureType::Kavu)],
        ..creature("Shoreline Raider", cost(&[generic(2), u()]), vec![CreatureType::Merfolk], 2, 2)
    }
}

/// Obsidian Acolyte — {1}{W} 1/1 that hands out protection from black.
pub fn obsidian_acolyte() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Black)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Protection(Color::Black),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Obsidian Acolyte",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Crusading Knight — {2}{W}{W} 2/2 pro-black that feeds on opposing Swamps.
pub fn crusading_knight() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Black)],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+1 for each Swamp your opponents control.",
            effect: StaticEffect::PumpSelfByValue {
                amount: opponent_lands(LandType::Swamp),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        ..creature(
            "Crusading Knight",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Marauding Knight — {2}{B}{B} 2/2 pro-white that feeds on opposing Plains.
pub fn marauding_knight() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::White)],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+1 for each Plains your opponents control.",
            effect: StaticEffect::PumpSelfByValue {
                amount: opponent_lands(LandType::Plains),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        ..creature(
            "Marauding Knight",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Zombie, CreatureType::Knight],
            2,
            2,
        )
    }
}

fn opponent_lands(kind: LandType) -> Value {
    Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(R::ControlledByOpponent)),
        filter: R::HasLandType(kind),
    }
}

// ── Characteristic-defining bodies ──────────────────────────────────────────

/// Molimo, Maro-Sorcerer — {4}{G}{G}{G} trampler sized by your lands.
pub fn molimo_maro_sorcerer() -> CardDefinition {
    let lands = || Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(R::ControlledByYou)),
        filter: R::Land,
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Molimo's power and toughness are each equal to the number of lands \
                          you control.",
            effect: StaticEffect::SelfBasePtFromValue { power: lands(), toughness: lands() },
        }],
        ..creature(
            "Molimo, Maro-Sorcerer",
            cost(&[generic(4), g(), g(), g()]),
            vec![CreatureType::Elemental, CreatureType::Sorcerer],
            0,
            0,
        )
    }
}

/// Yavimaya Kavu — {2}{R}{G} sized by the red and green creature counts.
pub fn yavimaya_kavu() -> CardDefinition {
    let of_color = |k: Color| Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(R::Creature)),
        filter: R::HasColor(k),
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Yavimaya Kavu's power is equal to the number of red creatures on the \
                          battlefield and its toughness to the number of green creatures.",
            effect: StaticEffect::SelfBasePtFromValue {
                power: of_color(Color::Red),
                toughness: of_color(Color::Green),
            },
        }],
        ..creature(
            "Yavimaya Kavu",
            cost(&[generic(2), r(), g()]),
            vec![CreatureType::Kavu],
            0,
            0,
        )
    }
}

// ── Utility creatures ───────────────────────────────────────────────────────

/// Blazing Specter — {2}{B}{R} 2/2 flying haste that strips a card on connect.
pub fn blazing_specter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::TriggerEventPlayer),
                amount: Value::ONE,
                random: false,
            },
        }],
        ..creature(
            "Blazing Specter",
            cost(&[generic(2), b(), r()]),
            vec![CreatureType::Specter],
            2,
            2,
        )
    }
}

/// Riptide Crab — {1}{W}{U} 1/3 vigilant that cantrips on death.
pub fn riptide_crab() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![on_dies(draw(1))],
        ..creature("Riptide Crab", cost(&[generic(1), w(), u()]), vec![CreatureType::Crab], 1, 3)
    }
}

/// Vodalian Merchant — {1}{U} 1/2 looter on the way in.
pub fn vodalian_merchant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            draw(1),
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
        ]))],
        ..creature("Vodalian Merchant", cost(&[generic(1), u()]), vec![CreatureType::Merfolk], 1, 2)
    }
}

/// Crypt Angel — {4}{B} 3/3 flier, pro-white, that rebuys a blue or red body.
pub fn crypt_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Protection(Color::White)],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                R::Creature
                    .and(R::InYourGraveyard)
                    .and(R::HasColor(Color::Blue).or(R::HasColor(Color::Red))),
            ),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        })],
        ..creature("Crypt Angel", cost(&[generic(4), b()]), vec![CreatureType::Angel], 3, 3)
    }
}

/// Phyrexian Delver — {3}{B}{B} 3/2 reanimator that charges its own life.
pub fn phyrexian_delver() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::ManaValueOf(Box::new(Selector::LastMoved)),
            },
        ]))],
        ..creature(
            "Phyrexian Delver",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Zombie],
            3,
            2,
        )
    }
}

/// Rogue Kavu — {1}{R} 1/1 that swings bigger when it swings alone.
pub fn rogue_kavu() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::IsAttackingAlone,
                }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Rogue Kavu", cost(&[generic(1), r()]), vec![CreatureType::Kavu], 1, 1)
    }
}

/// Vicious Kavu — {1}{B}{R} 2/2 that pumps whenever it attacks.
pub fn vicious_kavu() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Vicious Kavu", cost(&[generic(1), b(), r()]), vec![CreatureType::Kavu], 2, 2)
    }
}

/// Voracious Cobra — {2}{R}{G} 2/2 first-striking deathtouch-by-trigger.
pub fn voracious_cobra() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToCreature, EventScope::SelfSource),
            effect: Effect::Destroy { what: Selector::TriggerSource },
        }],
        ..creature("Voracious Cobra", cost(&[generic(2), r(), g()]), vec![CreatureType::Snake], 2, 2)
    }
}

/// Phyrexian Reaper — {4}{B} 3/3 that eats a green blocker.
pub fn phyrexian_reaper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![blocked_by_color_kills(Color::Green)],
        ..creature(
            "Phyrexian Reaper",
            cost(&[generic(4), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Zombie],
            3,
            3,
        )
    }
}

/// Phyrexian Slayer — {3}{B} 2/2 flier that eats a white blocker.
pub fn phyrexian_slayer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![blocked_by_color_kills(Color::White)],
        ..creature(
            "Phyrexian Slayer",
            cost(&[generic(3), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Minion],
            2,
            2,
        )
    }
}

/// "Whenever this becomes blocked by a [colour] creature, destroy that
/// creature. It can't be regenerated." The colour test rides the effect's
/// selector, so a block by an off-colour creature resolves as a no-op.
fn blocked_by_color_kills(color: Color) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
        effect: Effect::DestroyNoRegen {
            what: Selector::MatchingAmong {
                inner: Box::new(Selector::BlockingCreatures),
                filter: R::HasColor(color),
            },
        },
    }
}

/// Sparring Golem — {3} 2/2 that swells with each blocker.
pub fn sparring_golem() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::CountOf(Box::new(Selector::BlockingCreatures)),
                toughness: Value::CountOf(Box::new(Selector::BlockingCreatures)),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Sparring Golem", cost(&[generic(3)]), vec![CreatureType::Golem], 2, 2)
    }
}

/// Urborg Shambler — {2}{B}{B} 4/3 that shrinks every other black body.
pub fn urborg_shambler() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other black creatures get -1/-1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::HasColor(Color::Black)).and(R::OtherThanSource),
                ),
                power: -1,
                toughness: -1,
            },
        }],
        ..creature("Urborg Shambler", cost(&[generic(2), b(), b()]), vec![CreatureType::Horror], 4, 3)
    }
}

/// Kavu Monarch — {2}{R}{R} 3/3 Kavu lord that grows on every new Kavu.
pub fn kavu_monarch() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Kavu creatures have trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::HasCreatureType(CreatureType::Kavu)),
                ),
                keyword: Keyword::Trample,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature
                        .and(R::HasCreatureType(CreatureType::Kavu))
                        .and(R::OtherThanSource),
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature("Kavu Monarch", cost(&[generic(2), r(), r()]), vec![CreatureType::Kavu], 3, 3)
    }
}

/// Kavu Lair — {2}{G}. Every fat creature that lands draws its controller a card.
pub fn kavu_lair() -> CardDefinition {
    CardDefinition {
        name: "Kavu Lair",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::PowerAtLeast(4)),
                }),
            effect: Effect::Draw { who: Selector::Player(PlayerRef::TriggerEventPlayer), amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Rewards of Diversity — {2}{W}. Four life whenever an opponent goes gold.
pub fn rewards_of_diversity() -> CardDefinition {
    CardDefinition {
        name: "Rewards of Diversity",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Multicolored,
                }),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
        }],
        ..Default::default()
    }
}

/// Urza's Filter — {4}. Gold spells cost {2} less for everyone.
pub fn urzas_filter() -> CardDefinition {
    CardDefinition {
        name: "Urza's Filter",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Multicolored spells cost {2} less to cast.",
            effect: StaticEffect::CostReduction { filter: R::Multicolored, amount: 2 },
        }],
        ..Default::default()
    }
}

/// Juntu Stakes — {2}. Small creatures stay tapped.
pub fn juntu_stakes() -> CardDefinition {
    CardDefinition {
        name: "Juntu Stakes",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Creatures with power 1 or less don't untap during their controllers' \
                          untap steps.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::EachPermanent(R::Creature.and(R::PowerAtMost(1))),
            },
        }],
        ..Default::default()
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Zap — {2}{R}. A ping with a cantrip.
pub fn zap() -> CardDefinition {
    instant(
        "Zap",
        cost(&[generic(2), r()]),
        Effect::Seq(vec![
            Effect::DealDamage { to: Selector::Target(0), amount: Value::ONE },
            draw(1),
        ]),
    )
}

/// Recover — {2}{B}. Rebuy a creature, then draw.
pub fn recover() -> CardDefinition {
    sorcery(
        "Recover",
        cost(&[generic(2), b()]),
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            draw(1),
        ]),
    )
}

/// Reviving Dose — {2}{W}. Three life and a card.
pub fn reviving_dose() -> CardDefinition {
    instant(
        "Reviving Dose",
        cost(&[generic(2), w()]),
        Effect::Seq(vec![Effect::GainLife { who: Selector::You, amount: Value::Const(3) }, draw(1)]),
    )
}

/// Restrain — {2}{W}. Fog one attacker, draw a card.
pub fn restrain() -> CardDefinition {
    instant(
        "Restrain",
        cost(&[generic(2), w()]),
        Effect::Seq(vec![
            Effect::PreventCombatDamageByTargetThisTurn {
                target: target_filtered(R::Creature.and(R::IsAttacking)),
            },
            draw(1),
        ]),
    )
}

/// Turf Wound — {2}{R}. Strip a land drop, draw a card.
pub fn turf_wound() -> CardDefinition {
    instant(
        "Turf Wound",
        cost(&[generic(2), r()]),
        Effect::Seq(vec![
            Effect::PlayerCantPlayLandsThisTurn { player: PlayerRef::Target(0) },
            draw(1),
        ]),
    )
}

/// Wallop — {1}{G}. Shoot down a blue or black flier.
pub fn wallop() -> CardDefinition {
    sorcery(
        "Wallop",
        cost(&[generic(1), g()]),
        Effect::Destroy {
            what: target_filtered(
                R::Creature
                    .and(R::HasKeyword(Keyword::Flying))
                    .and(R::HasColor(Color::Blue).or(R::HasColor(Color::Black))),
            ),
        },
    )
}

/// Recoil — {1}{U}{B}. Bounce a permanent, then its controller discards.
pub fn recoil() -> CardDefinition {
    instant(
        "Recoil",
        cost(&[generic(1), u(), b()]),
        Effect::Seq(vec![
            Effect::Move { what: target_filtered(R::Permanent), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
            Effect::Discard {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::ONE,
                random: false,
            },
        ]),
    )
}

/// Undermine — {U}{U}{B}. Counter and drain three.
pub fn undermine() -> CardDefinition {
    instant(
        "Undermine",
        cost(&[u(), u(), b()]),
        Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(3),
            },
        ]),
    )
}

/// Distorting Wake — {X}{U}{U}{U}. Bounce X nonland permanents.
pub fn distorting_wake() -> CardDefinition {
    sorcery(
        "Distorting Wake",
        cost(&[crate::mana::x(), u(), u(), u()]),
        Effect::ApplyToTargets {
            max_targets: 8,
            min_targets: 0,
            filter: R::Permanent.and(R::Nonland),
            effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) }),
        },
    )
}

/// Wash Out — {3}{U}. Bounce everything of a chosen colour.
pub fn wash_out() -> CardDefinition {
    sorcery(
        "Wash Out",
        cost(&[generic(3), u()]),
        Effect::Seq(vec![
            Effect::ChooseColorForSelf,
            Effect::Move {
                what: Selector::EachPermanent(R::HasChosenColorOfSource),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        ]),
    )
}

/// Lightning Dart — {1}{R}. One damage, or four to a white or blue creature.
pub fn lightning_dart() -> CardDefinition {
    instant(
        "Lightning Dart",
        cost(&[generic(1), r()]),
        Effect::DealDamage {
            to: target_filtered(R::Creature),
            amount: Value::IfPred {
                pred: Box::new(Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::HasColor(Color::White).or(R::HasColor(Color::Blue)),
                }),
                then: Box::new(Value::Const(4)),
                else_: Box::new(Value::ONE),
            },
        },
    )
}

/// Plague Spores — {4}{B}{R}. Kill a nonblack creature and a land.
pub fn plague_spores() -> CardDefinition {
    sorcery(
        "Plague Spores",
        cost(&[generic(4), b(), r()]),
        Effect::Seq(vec![
            Effect::DestroyNoRegen {
                what: target_filtered(R::Creature.and(R::HasColor(Color::Black).negate())),
            },
            Effect::DestroyNoRegen { what: target_filtered(R::Land) },
        ]),
    )
}

/// Manipulate Fate — {1}{U}. Exile three from your library, then draw.
pub fn manipulate_fate() -> CardDefinition {
    sorcery(
        "Manipulate Fate",
        cost(&[generic(1), u()]),
        Effect::Seq(vec![
            Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::Any,
                to: ZoneDest::Exile,
                count: Value::Const(3),
            },
            draw(1),
        ]),
    )
}

/// Liberate — {1}{W}. Blink one of your own creatures out of harm's way.
pub fn liberate() -> CardDefinition {
    instant(
        "Liberate",
        cost(&[generic(1), w()]),
        Effect::ExileReturnNextEndStep {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
        },
    )
}

/// Reviving Vapors — {2}{W}{U}. Take one of three, gain its mana value.
pub fn reviving_vapors() -> CardDefinition {
    instant(
        "Reviving Vapors",
        cost(&[generic(2), w(), u()]),
        Effect::Seq(vec![
            Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::Const(3),
                rest_to_graveyard: true,
    ..Default::default()
})),
            Effect::GainLife { who: Selector::You, amount: Value::ManaValueOf(Box::new(Selector::LastMoved)) },
        ]),
    )
}
