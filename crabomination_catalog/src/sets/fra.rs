//! Reality Fracture (FRA, 2026-10-02): the main-set cards the Commander work
//! didn't reach. The rest of the set ships beside the decks that first
//! needed it (`decks::cmdr_*`). Tests in `tests/recent_b/fra.rs`.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, CardDefinition, CardId, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, ExileReturnZone, Keyword, LoyaltyAbility, MayPlayDuration, PlaneswalkerSubtype, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
    WardCost,
};
use crate::effect::shortcut::{etb, target_any, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, Selector, StaticEffect, VoteOption, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w};

use super::tap_add;

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn legendary_enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        ..Default::default()
    }
}

fn loyalty(cost: i32, effect: Effect) -> LoyaltyAbility {
    LoyaltyAbility { loyalty_cost: cost, effect, ..Default::default() }
}

/// CR 701.71a — "Empower Jace N".
fn empower(n: i32) -> Effect {
    Effect::EmpowerJace { who: PlayerRef::You, count: Value::Const(n) }
}

fn creature_or_walker() -> R {
    R::Creature.or(R::Planeswalker)
}

fn either_color(a: Color, b2: Color) -> R {
    R::HasColor(a).or(R::HasColor(b2))
}

fn your_creatures() -> Selector {
    Selector::EachPermanent(R::Creature.and(R::ControlledByYou))
}

fn pump(what: Selector, p: i32, t: i32) -> Effect {
    Effect::PumpPT { what, power: Value::Const(p), toughness: Value::Const(t), duration: Duration::EndOfTurn }
}

fn grant(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

fn plus_one(what: Selector) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: Value::ONE }
}

fn make(definition: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(definition) }
}

/// "a 2/2 colorless Wizard Soldier creature token named Cadet".
fn cadet() -> TokenDefinition {
    TokenDefinition {
        name: "Cadet".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard, CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    }
}

/// "a 4/4 green Beast creature token with trample".
fn beast() -> TokenDefinition {
    TokenDefinition {
        name: "Beast".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        keywords: vec![Keyword::Trample],
        ..Default::default()
    }
}

// ── White ───────────────────────────────────────────────────────────────────

/// Academic Ascent — {1}{W} instant. Target creature gets +2/+2 and gains
/// flying until end of turn. Empower Jace 2.
pub fn academic_ascent() -> CardDefinition {
    spell(
        "Academic Ascent",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            pump(target_filtered(R::Creature), 2, 2),
            grant(Selector::Target(0), Keyword::Flying),
            empower(2),
        ]),
    )
}

/// Memory Trap — {2}{W} enchantment. ETB: exile target nonland permanent an
/// opponent controls until this enchantment leaves the battlefield.
pub fn memory_trap() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Permanent.and(R::Nonland).and(R::ControlledByOpponent)),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..spell("Memory Trap", cost(&[generic(2), w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Refute Destiny — {1}{W} sorcery. Exile target green or blue creature or
/// planeswalker. Surveil 1.
pub fn refute_destiny() -> CardDefinition {
    spell(
        "Refute Destiny",
        cost(&[generic(1), w()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Exile { what: target_filtered(creature_or_walker().and(either_color(Color::Green, Color::Blue))) },
            Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
        ]),
    )
}

/// Shatterwing Pegasus — {2}{W} 2/3 flying Pegasus. {4}{W}: creatures you
/// control get +1/+1 until end of turn.
pub fn shatterwing_pegasus() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), w()]),
            effect: pump(your_creatures(), 1, 1),
            ..Default::default()
        }],
        ..creature("Shatterwing Pegasus", cost(&[generic(2), w()]), vec![CreatureType::Pegasus], 2, 3)
    }
}

// ── Blue ────────────────────────────────────────────────────────────────────

/// Cryotheory Adept — {1}{U} 2/1 Human Wizard, prowess. {3}{U}, exile it from
/// your graveyard: tap target creature and put a stun counter on it, as a
/// sorcery.
pub fn cryotheory_adept() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Prowess],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::Tap { what: target_filtered(R::Creature) },
                Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..creature("Cryotheory Adept", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 1)
    }
}

/// Mindseeker Oculus — {2}{U} 2/1 Homunculus. ETB: empower Jace 4.
pub fn mindseeker_oculus() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(empower(4))],
        ..creature("Mindseeker Oculus", cost(&[generic(2), u()]), vec![CreatureType::Homunculus], 2, 1)
    }
}

/// Theorist's Proxy — {1}{U} 0/3 Illusion, flash. ETB: empower Jace 3. {U},
/// sacrifice it: the next spell you cast this turn can't be countered.
pub fn theorists_proxy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(empower(3))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            sac_cost: true,
            effect: Effect::NextSpellCantBeCountered { filter: R::Any },
            ..Default::default()
        }],
        ..creature("Theorist's Proxy", cost(&[generic(1), u()]), vec![CreatureType::Illusion], 0, 3)
    }
}

// ── Black ───────────────────────────────────────────────────────────────────

/// Extrapolate the Impossible — {1}{B} sorcery. You may reveal exactly two
/// differently named cards you own from outside the game; an opponent chooses
/// one and you put it into your hand.
pub fn extrapolate_the_impossible() -> CardDefinition {
    spell("Extrapolate the Impossible", cost(&[generic(1), b()]), CardType::Sorcery, Effect::WishPairOpponentChooses)
}

/// Solve for Disappointment — {1}{B} sorcery. Target opponent reveals their
/// hand; you choose a nonland permanent card and they discard it. Empower
/// Jace 1.
pub fn solve_for_disappointment() -> CardDefinition {
    spell(
        "Solve for Disappointment",
        cost(&[generic(1), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DiscardChosen {
                from: target_filtered(R::OpponentPlayer),
                count: Value::ONE,
                filter: R::Nonland.and(R::PermanentCard),
            },
            empower(1),
        ]),
    )
}

/// Terminal Criticism — {1}{B} instant. Destroy target blue or red creature
/// or planeswalker. You gain 1 life.
pub fn terminal_criticism() -> CardDefinition {
    spell(
        "Terminal Criticism",
        cost(&[generic(1), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(creature_or_walker().and(either_color(Color::Blue, Color::Red))) },
            Effect::GainLife { who: Selector::You, amount: Value::ONE },
        ]),
    )
}

/// Theoretical Necromancer — {2}{B} 4/1 Vampire Warlock. {3}{B}, exile it
/// from your graveyard: return another target creature card from your
/// graveyard to your hand.
pub fn theoretical_necromancer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::OtherThanSource)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..creature(
            "Theoretical Necromancer",
            cost(&[generic(2), b()]),
            vec![CreatureType::Vampire, CreatureType::Warlock],
            4,
            1,
        )
    }
}

// ── Red ─────────────────────────────────────────────────────────────────────

/// Awaken the Inferno — {4}{R} sorcery. 6 damage to target creature or
/// planeswalker an opponent controls; a +1/+1 counter on up to one target
/// creature you control. Basic landcycling {2}.
pub fn awaken_the_inferno() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Typecycling(Box::new((cost(&[generic(2)]), R::IsBasicLand)))],
        ..spell(
            "Awaken the Inferno",
            cost(&[generic(4), r()]),
            CardType::Sorcery,
            Effect::OptionalTargets {
                min: 1,
                body: Box::new(Effect::Seq(vec![
                    Effect::DealDamage {
                        to: target_filtered(creature_or_walker().and(R::ControlledByOpponent)),
                        amount: Value::Const(6),
                    },
                    plus_one(Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByYou) }),
                ])),
            },
        )
    }
}

/// Essence Burn — {1}{R} instant. 5 damage to target black or green creature
/// or planeswalker; if that permanent would die this turn, exile it instead.
pub fn essence_burn() -> CardDefinition {
    spell(
        "Essence Burn",
        cost(&[generic(1), r()]),
        CardType::Instant,
        // The replacement goes in first: damage checks state-based actions
        // mid-resolution, so a lethal hit dies before a later step runs.
        Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn { what: Selector::Target(0) },
            Effect::DealDamage {
                to: target_filtered(creature_or_walker().and(either_color(Color::Black, Color::Green))),
                amount: Value::Const(5),
            },
        ]),
    )
}

/// Fulminous Forte — {2}{R} instant. Choose one: 1 damage to each creature
/// and planeswalker your opponents control; or 5 damage to target creature
/// or planeswalker.
pub fn fulminous_forte() -> CardDefinition {
    spell(
        "Fulminous Forte",
        cost(&[generic(2), r()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(creature_or_walker().and(R::ControlledByOpponent)),
                amount: Value::ONE,
            },
            Effect::DealDamage { to: target_filtered(creature_or_walker()), amount: Value::Const(5) },
        ]),
    )
}

/// Heartstring Puller — {3}{R} 3/1 trample Elf Sorcerer. ETB: a Cadet.
pub fn heartstring_puller() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(make(cadet()))],
        ..creature("Heartstring Puller", cost(&[generic(3), r()]), vec![CreatureType::Elf, CreatureType::Sorcerer], 3, 1)
    }
}

/// No Admittance — {1}{R} sorcery. 3 damage to any target. Empower Jace 1.
pub fn no_admittance() -> CardDefinition {
    spell(
        "No Admittance",
        cost(&[generic(1), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![Effect::DealDamage { to: target_any(), amount: Value::Const(3) }, empower(1)]),
    )
}

/// Skilled Battlecarver — {1}{R} 2/1 Human Warrior. First strike during your
/// turn. {1}{R}: +1/+0 until end of turn.
pub fn skilled_battlecarver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "During your turn, this creature has first strike.",
            effect: StaticEffect::WhileYourTurn {
                inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::FirstStrike }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: pump(Selector::This, 1, 0),
            ..Default::default()
        }],
        ..creature("Skilled Battlecarver", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Warrior], 2, 1)
    }
}

/// Tether Technician — {4}{R} 4/5 reach Minotaur Artificer. ETB: you may
/// discard a card; when you do, it deals 2 damage to any target (CR 603.12 —
/// the reflexive trigger targets after the discard).
pub fn tether_technician() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::MayDiscard {
            description: "Discard a card to deal 2 damage to any target?".into(),
            count: Value::ONE,
            then: Box::new(Effect::Reflexive {
                body: Box::new(Effect::DealDamage { to: target_any(), amount: Value::Const(2) }),
            }),
            else_: None,
        })],
        ..creature(
            "Tether Technician",
            cost(&[generic(4), r()]),
            vec![CreatureType::Minotaur, CreatureType::Artificer],
            4,
            5,
        )
    }
}

/// Violent Echoes — {2}{R}{R} instant. 6 damage to target creature or
/// planeswalker; if excess damage was dealt to it this way (CR 120.10),
/// empower Jace X, where X is that excess damage.
pub fn violent_echoes() -> CardDefinition {
    spell(
        "Violent Echoes",
        cost(&[generic(2), r(), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(creature_or_walker()), amount: Value::Const(6) },
            Effect::If {
                cond: Predicate::ExcessDamageDealtThisResolution,
                then: Box::new(Effect::EmpowerJace { who: PlayerRef::You, count: Value::ExcessDamageDealtThisResolution }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Wrath of the Bloodmane — {2}{R} instant, {1} less if you control a
/// legendary creature. 4 damage to target creature or planeswalker.
pub fn wrath_of_the_bloodmane() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast if you control a legendary creature.",
            effect: StaticEffect::SelfCostReducedIf {
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::Creature.and(R::HasSupertype(Supertype::Legendary)).and(R::ControlledByYou),
                )),
                amount: 1,
            },
        }],
        ..spell(
            "Wrath of the Bloodmane",
            cost(&[generic(2), r()]),
            CardType::Instant,
            Effect::DealDamage { to: target_filtered(creature_or_walker()), amount: Value::Const(4) },
        )
    }
}

// ── Green ───────────────────────────────────────────────────────────────────

/// Arcane Amphisbaena — {1}{G} 1/1 deathtouch Snake. ETB: empower Jace 2.
pub fn arcane_amphisbaena() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(empower(2))],
        ..creature("Arcane Amphisbaena", cost(&[generic(1), g()]), vec![CreatureType::Snake], 1, 1)
    }
}

/// Bestial Incursion — {3}{G} sorcery. A 4/4 green Beast with trample.
/// Flashback {5}{G}.
pub fn bestial_incursion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(5), g()]))],
        ..spell("Bestial Incursion", cost(&[generic(3), g()]), CardType::Sorcery, make(beast()))
    }
}

/// Flourishing Grapple — {G} instant. Target red or white creature or
/// planeswalker an opponent controls loses all abilities until end of turn;
/// target creature you control deals damage equal to its power to it.
pub fn flourishing_grapple() -> CardDefinition {
    spell(
        "Flourishing Grapple",
        cost(&[g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::LoseAllAbilities {
                what: target_filtered(
                    creature_or_walker().and(R::ControlledByOpponent).and(either_color(Color::Red, Color::White)),
                ),
                duration: Duration::EndOfTurn,
            },
            Effect::DealDamageEqualToPower {
                source: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByYou) },
                target: Selector::Target(0),
            },
        ]),
    )
}

/// Inspired Tethermage — {2}{G} 3/2 Elf Warrior. Whenever you put one or more
/// loyalty counters on a planeswalker, a +1/+1 counter on it. {6}: empower
/// Jace 2.
pub fn inspired_tethermage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterAdded(CounterType::Loyalty), EventScope::YouPutCounters)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Planeswalker }),
            effect: plus_one(Selector::This),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            effect: empower(2),
            ..Default::default()
        }],
        ..creature("Inspired Tethermage", cost(&[generic(2), g()]), vec![CreatureType::Elf, CreatureType::Warrior], 3, 2)
    }
}

/// Sureshot Sower — {1}{G} 3/1 reach Human Archer. {3}{G}, discard it:
/// destroy target creature with flying.
pub fn sureshot_sower() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))) },
            ..Default::default()
        }],
        ..creature("Sureshot Sower", cost(&[generic(1), g()]), vec![CreatureType::Human, CreatureType::Archer], 3, 1)
    }
}

/// Wrecking Gecko — {4}{G} 5/5 artifact Lizard Construct, ward {2}.
/// {6}{G}{G}: +4/+4 and trample until end of turn.
pub fn wrecking_gecko() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Ward(WardCost::generic(2))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6), g(), g()]),
            effect: Effect::Seq(vec![pump(Selector::This, 4, 4), grant(Selector::This, Keyword::Trample)]),
            ..Default::default()
        }],
        ..creature("Wrecking Gecko", cost(&[generic(4), g()]), vec![CreatureType::Lizard, CreatureType::Construct], 5, 5)
    }
}

// ── Multicolored ────────────────────────────────────────────────────────────

/// Charge the Sanctum — {2}{R/W} instant. Choose one: creatures you control
/// get +2/+0 until end of turn; or target creature gets +2/+0 and first
/// strike until end of turn, and a +1/+1 counter.
pub fn charge_the_sanctum() -> CardDefinition {
    spell(
        "Charge the Sanctum",
        ManaCost::new(vec![generic(2), hybrid(Color::Red, Color::White)]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            pump(your_creatures(), 2, 0),
            Effect::Seq(vec![
                pump(target_filtered(R::Creature), 2, 0),
                grant(Selector::Target(0), Keyword::FirstStrike),
                plus_one(Selector::Target(0)),
            ]),
        ]),
    )
}

/// Clash of Elements — {1}{U}{R} instant. Target nonland permanent's owner
/// puts it on top of their library and takes 2 damage from this, or puts it
/// on the bottom.
pub fn clash_of_elements() -> CardDefinition {
    let it = || target_filtered(R::Permanent.and(R::Nonland));
    let owner = || PlayerRef::OwnerOf(Box::new(Selector::Target(0)));
    spell(
        "Clash of Elements",
        cost(&[generic(1), u(), r()]),
        CardType::Instant,
        Effect::PlayerChoosesOne {
            who: owner(),
            options: vec![
                VoteOption::new(
                    "Put it on the bottom of your library",
                    Effect::Move { what: it(), to: ZoneDest::Library { who: owner(), pos: LibraryPosition::Bottom } },
                ),
                VoteOption::new(
                    "Put it on top of your library and take 2 damage",
                    Effect::Seq(vec![
                        Effect::Move { what: it(), to: ZoneDest::Library { who: owner(), pos: LibraryPosition::Top } },
                        Effect::DealDamage { to: Selector::You, amount: Value::Const(2) },
                    ]),
                ),
            ],
        },
    )
}

/// Desperate Futurescribe — {2}{W}{U} 3/4 flying Kor Scout. At the beginning
/// of combat on your turn, another target creature you control gets +1/+1
/// until end of turn — or a +1/+1 counter instead if you've scried or
/// surveilled this turn.
pub fn desperate_futurescribe() -> CardDefinition {
    let other = || target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource));
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::If {
                cond: Predicate::ScriedOrSurveiledThisTurn { who: PlayerRef::You },
                then: Box::new(plus_one(other())),
                else_: Box::new(pump(other(), 1, 1)),
            },
        }],
        ..creature(
            "Desperate Futurescribe",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Kor, CreatureType::Scout],
            3,
            4,
        )
    }
}

/// Grim Repriser — {B}{R} 2/2 Zombie Bard, prowess. {B}{R}: return it from
/// your graveyard to the battlefield with a finality counter, only if an
/// opponent has been dealt noncombat damage this turn.
pub fn grim_repriser() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Prowess],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), r()]),
            from_graveyard: true,
            condition: Some(Predicate::ValueAtLeast(
                Value::NoncombatDamageTakenThisTurn(PlayerRef::EachOpponent),
                Value::ONE,
            )),
            // It enters WITH the counter (CR 122.6), not gaining one after.
            effect: Effect::Seq(vec![
                Effect::SpellEntersWithCounters { what: Selector::This, kind: CounterType::Finality, amount: Value::ONE },
                Effect::Move { what: Selector::This, to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false } },
            ]),
            ..Default::default()
        }],
        ..creature("Grim Repriser", cost(&[b(), r()]), vec![CreatureType::Zombie, CreatureType::Bard], 2, 2)
    }
}

/// Mind Meanderer — {3}{G}{U}{U} 4/4 flying Bird Fish Illusion; vigilance
/// while you control a Jace planeswalker. ETB: it fights up to one target
/// creature an opponent controls.
pub fn mind_meanderer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "This creature has vigilance as long as you control a Jace planeswalker.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Vigilance,
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::Planeswalker.and(R::HasPlaneswalkerType(PlaneswalkerSubtype::Jace)).and(R::ControlledByYou),
                )),
            },
        }],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Fight {
                attacker: Selector::This,
                defender: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            }),
        })],
        ..creature(
            "Mind Meanderer",
            cost(&[generic(3), g(), u(), u()]),
            vec![CreatureType::Bird, CreatureType::Fish, CreatureType::Illusion],
            4,
            4,
        )
    }
}

/// Null Summoner — {2}{U}{B} 4/2 Human Warlock. When it enters, if you cast
/// it, target opponent reveals their hand and you exile a nonland card from
/// it. Threshold — while you have seven or more cards in your graveyard, you
/// may cast the exiled card, spending mana as though it were any type.
pub fn null_summoner() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SourceWasCast),
            effect: Effect::Seq(vec![
                Effect::ExileChosenFromHand {
                    from: target_filtered(R::OpponentPlayer),
                    count: Value::ONE,
                    filter: R::Nonland,
                    link_to_source: true,
                    face_down: false,
                },
                // The castability is the static ability's, so it lives as long
                // as this permanent does and wakes with threshold.
                Effect::GrantMayPlay {
                    what: Selector::CardExiledWithSource,
                    duration: MayPlayDuration::WhileSourceOnBattlefieldWithThreshold { source: CardId(0) },
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: true,
                },
            ]),
        }],
        ..creature("Null Summoner", cost(&[generic(2), u(), b()]), vec![CreatureType::Human, CreatureType::Warlock], 4, 2)
    }
}

/// Stingerquill Charm — {B}{R} instant. Choose one: 3 damage to any target;
/// target creature gains first strike and deathtouch until end of turn; or a
/// Cadet that gains haste until end of turn.
pub fn stingerquill_charm() -> CardDefinition {
    spell(
        "Stingerquill Charm",
        cost(&[b(), r()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
            Effect::Seq(vec![
                grant(target_filtered(R::Creature), Keyword::FirstStrike),
                grant(Selector::Target(0), Keyword::Deathtouch),
            ]),
            Effect::Seq(vec![make(cadet()), grant(Selector::LastCreatedToken, Keyword::Haste)]),
        ]),
    )
}

/// Stinging Vitriol — {B}{R} sorcery. 2 damage to target opponent; they
/// reveal their hand and discard a nonland card of your choice.
pub fn stinging_vitriol() -> CardDefinition {
    spell(
        "Stinging Vitriol",
        cost(&[b(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::OpponentPlayer), amount: Value::Const(2) },
            Effect::DiscardChosen { from: Selector::Target(0), count: Value::ONE, filter: R::Nonland },
        ]),
    )
}

/// Tam's Resistance — {1}{G/U} sorcery. A +1/+1 counter on up to one target
/// creature, which gains vigilance until end of turn. Empower Jace 4.
pub fn tams_resistance() -> CardDefinition {
    spell(
        "Tam's Resistance",
        ManaCost::new(vec![generic(1), hybrid(Color::Green, Color::Blue)]),
        CardType::Sorcery,
        Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                plus_one(target_filtered(R::Creature)),
                grant(Selector::Target(0), Keyword::Vigilance),
                empower(4),
            ])),
        },
    )
}

/// Vindictive Triumph — {W}{B}{B} instant. Exile target creature or
/// planeswalker. If its mana value was 3 or less, return it to the
/// battlefield tapped under your control; exile it at the next end step.
pub fn vindictive_triumph() -> CardDefinition {
    spell(
        "Vindictive Triumph",
        cost(&[w(), b(), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Move { what: target_filtered(creature_or_walker()), to: ZoneDest::Exile },
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::LastMoved, filter: R::ManaValueAtMost(3) },
                then: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::LastMoved,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    },
                    Effect::ExileAtNextEndStep { what: Selector::LastMoved },
                ])),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Kiora of Salt and Sand — {1}{G}{U} legendary 2/4 Merfolk Noble. Whenever
/// you attack, if you've activated a loyalty ability this turn, untap target
/// attacking creature; it can't be blocked this turn. Planeswalkers you
/// control have "[−8]: an 8/8 blue Leviathan with hexproof."
pub fn kiora_of_salt_and_sand() -> CardDefinition {
    let leviathan = TokenDefinition {
        name: "Leviathan".into(),
        power: 8,
        toughness: 8,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Leviathan], ..Default::default() },
        keywords: vec![Keyword::Hexproof],
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource)
                .with_filter(Predicate::ActivatedLoyaltyThisTurn { who: PlayerRef::You }),
            effect: Effect::Seq(vec![
                Effect::Untap { what: target_filtered(R::Creature.and(R::IsAttacking)), up_to: None },
                grant(Selector::Target(0), Keyword::Unblockable),
            ]),
        }],
        static_abilities: vec![StaticAbility {
            description: "Planeswalkers you control have \"[−8]: Create an 8/8 blue Leviathan creature token with hexproof.\"",
            effect: StaticEffect::PlaneswalkersHaveLoyaltyAbilities { abilities: vec![loyalty(-8, make(leviathan))] },
        }],
        ..creature(
            "Kiora of Salt and Sand",
            cost(&[generic(1), g(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Noble],
            2,
            4,
        )
    }
}

// ── Colorless, lands ────────────────────────────────────────────────────────

/// Keeper of the Quiet Hour — {3} 3/2 artifact Chimera. ETB: empower Jace 2.
pub fn keeper_of_the_quiet_hour() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(empower(2))],
        ..creature("Keeper of the Quiet Hour", cost(&[generic(3)]), vec![CreatureType::Chimera], 3, 2)
    }
}

/// Formidable Commons — land. Enters tapped unless you control a
/// planeswalker; {T}: add {B} or {G}.
pub fn formidable_commons() -> CardDefinition {
    CardDefinition {
        name: "Formidable Commons",
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped unless you control a planeswalker.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorExists(Selector::EachPermanent(R::Planeswalker.and(R::ControlledByYou))),
            },
        }],
        activated_abilities: vec![tap_add(Color::Black), tap_add(Color::Green)],
        ..Default::default()
    }
}

// ── Legendary ───────────────────────────────────────────────────────────────

/// Garruk, Veiled Butcher — {3}{B}{B}, loyalty 5. Opponents' dying creatures
/// are exiled instead. +2: up to one target creature gets -4/-1 until your
/// next turn. −2: each player sacrifices a creature; if you did, a 4/4 Beast
/// with trample. −3: each opponent discards two cards; you draw a card for
/// each opponent who didn't discard two nonland cards.
pub fn garruk_veiled_butcher() -> CardDefinition {
    CardDefinition {
        name: "Garruk, Veiled Butcher",
        cost: cost(&[generic(3), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Garruk], ..Default::default() },
        base_loyalty: 5,
        static_abilities: vec![StaticAbility {
            description: "If a creature an opponent controls would die, exile it instead.",
            effect: StaticEffect::ExileDyingOpponentCreatures { when_you_do: None, tokens_too: true },
        }],
        loyalty_abilities: vec![
            loyalty(
                2,
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::PumpPT {
                        what: target_filtered(R::Creature),
                        power: Value::Const(-4),
                        toughness: Value::Const(-1),
                        duration: Duration::UntilNextTurn,
                    }),
                },
            ),
            loyalty(
                -2,
                Effect::Seq(vec![
                    Effect::Sacrifice { who: Selector::Player(PlayerRef::EachPlayer), count: Value::ONE, filter: R::Creature },
                    Effect::If {
                        cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                        then: Box::new(make(beast())),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            ),
            // Each opponent is judged on their own discards, as Kroxa's are.
            loyalty(
                -3,
                Effect::ForEach {
                    selector: Selector::Player(PlayerRef::EachOpponent),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Discard {
                            who: Selector::Player(PlayerRef::Triggerer),
                            amount: Value::Const(2),
                            random: false,
                        },
                        Effect::If {
                            cond: Predicate::ValueAtMost(
                                Value::NonlandCardsDiscardedThisEffectBy(PlayerRef::Triggerer),
                                Value::ONE,
                            ),
                            then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                            else_: Box::new(Effect::Noop),
                        },
                    ])),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Way of the Deathbringer — {2}{B} legendary enchantment. Enters: empower
/// Jace 5. Planeswalkers you control have "[−2]: You may sacrifice a
/// creature. If you do, create a 4/4 green Beast with trample."
pub fn way_of_the_deathbringer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(empower(5))],
        static_abilities: vec![StaticAbility {
            description: "Planeswalkers you control have \"[−2]: You may sacrifice a creature. If you do, create a 4/4 green Beast creature token with trample.\"",
            effect: StaticEffect::PlaneswalkersHaveLoyaltyAbilities {
                abilities: vec![loyalty(
                    -2,
                    Effect::MaySacrifice {
                        description: "Sacrifice a creature for a 4/4 Beast?".into(),
                        filter: R::Creature,
                        count: Value::ONE,
                        then: Box::new(make(beast())),
                        else_: None,
                    },
                )],
            },
        }],
        ..legendary_enchantment("Way of the Deathbringer", cost(&[generic(2), b()]))
    }
}

/// Way of the Wildspeaker — {4}{G} legendary enchantment. Enters: empower
/// Jace 7. Planeswalkers you control have "[−4]: a 4/4 green Beast with
/// trample."
pub fn way_of_the_wildspeaker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(empower(7))],
        static_abilities: vec![StaticAbility {
            description: "Planeswalkers you control have \"[−4]: Create a 4/4 green Beast creature token with trample.\"",
            effect: StaticEffect::PlaneswalkersHaveLoyaltyAbilities { abilities: vec![loyalty(-4, make(beast()))] },
        }],
        ..legendary_enchantment("Way of the Wildspeaker", cost(&[generic(4), g()]))
    }
}
