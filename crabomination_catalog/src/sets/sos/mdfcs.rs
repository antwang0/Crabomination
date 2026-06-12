//! Secrets of Strixhaven (SOS) — Preparation cards.
//!
//! Preparation cards pair a creature with an inset "prepare spell"
//! (Adventure/Omen-style frame). They are **not** MDFCs: a preparation
//! card is a creature card in every zone, and the inset spell is never
//! cast from the hand. While the creature on the battlefield is
//! *prepared* (carries a `CounterType::Prepared` counter), its
//! controller may cast a copy of the inset spell via
//! `GameAction::CastPrepareSpell`, which unprepares the creature. The
//! spell definition rides `CardDefinition.prepare_spell`; "This
//! creature enters prepared." rides `enters_with_counters:
//! Some((CounterType::Prepared, Value::Const(1)))`.
//!
//! The prepare-spell `CardDefinition`s are constructed inline because
//! each spell needs slightly different `effect`/cost plumbing — keeping
//! the helpers out of `super::sorceries` avoids cluttering the
//! named-spell module with one-off preparation variants.
//!
//! Cards in this module (creature // prepare spell):
//! - Elite Interceptor // Rejoinder
//! - Emeritus of Truce // Swords to Plowshares
//! - Honorbound Page // Forum's Favor
//! - Joined Researchers // Secret Rendezvous
//! - Quill-Blade Laureate // Twofold Intent
//! - Spiritcall Enthusiast // Scrollboost
//! - Encouraging Aviator // Jump
//! - Harmonized Trio // Brainstorm
//! - Cheerful Osteomancer // Raise Dead
//! - Emeritus of Woe // Demonic Tutor
//! - Scheming Silvertongue // Sign in Blood
//! - Emeritus of Conflict // Lightning Bolt
//! - Goblin Glasswright // Craft with Pride
//! - Emeritus of Abundance // Regrowth
//! - Vastlands Scavenger // Bind to Life
//! - …and the rest of the cycle below.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, SelectionRequirement, Subtypes, TriggeredAbility, WardCost,
};
use crate::card::Zone;
use crate::effect::shortcut::{
    cards_in_graveyard_at_least, etb, mint_lorehold_spirits, mint_treasures, on_attack,
    pump_target, target_filtered,
};
use crate::effect::{Duration, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

/// Helper: standard plain-creature front. SOS preparation-card creature
/// halves are nearly all vanilla bodies with an inset prepare spell —
/// the only varying fields are name, cost, P/T, and creature subtypes.
fn vanilla_front(
    name: &'static str,
    front_cost: ManaCost,
    creature_types: Vec<CreatureType>,
    power: i32,
    toughness: i32,
    keywords: Vec<Keyword>,
    prepare: CardDefinition,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: front_cost,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types,
            ..Default::default()
        },
        power,
        toughness,
        keywords,
        prepare_spell: Some(Box::new(prepare)),
        ..Default::default()
    }
}

/// Helper: a prepare-spell definition built from a name/type/cost/effect
/// tuple.
fn spell_back(
    name: &'static str,
    cost: ManaCost,
    card_type: CardType,
    effect: Effect,
) -> CardDefinition {
    CardDefinition {
        name,
        cost,
        card_types: vec![card_type],
        effect,
        ..Default::default()
    }
}

/// "This creature enters prepared." — CR 614.12 enters-with-counters
/// replacement carrying one `Prepared` counter.
fn enters_prepared(mut card: CardDefinition) -> CardDefinition {
    card.enters_with_counters = Some((CounterType::Prepared, Value::Const(1)));
    card
}

/// "… becomes prepared." — drop one `Prepared` counter on the source.
fn becomes_prepared() -> Effect {
    Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::Prepared,
        amount: Value::Const(1),
    }
}

// ── White preparation cards ─────────────────────────────────────────────────

/// Emeritus of Truce // Swords to Plowshares — {1}{W}{W} // {W}.
///
/// Front: 3/3 Cat Cleric. "When this creature enters, target player
/// creates a 1/1 white and black Inkling creature token with flying.
/// Then if an opponent controls more creatures than you, this creature
/// becomes prepared."
///
/// Approximation: the engine's `Effect::CreateToken` mints for a
/// `PlayerRef`, and a trigger-scoped "target player" slot isn't wired
/// for it — the token is minted for *you* instead of a chosen target
/// player. The conditional "becomes prepared" rider is faithful via
/// `Predicate::AnOpponentControlsMoreCreatures`.
///
/// Prepare spell: instant — exile target creature; its controller gains
/// life equal to that creature's power.
///
/// Approximation: the printed Swords to Plowshares lifegain rider keys
/// off the *target's controller*. The engine has no opponent-directed
/// `GainLife { who: PlayerRef::ControllerOf(Selector) }` shape, so we
/// approximate as the target's *owner* gaining life. The exile half is
/// faithful.
pub fn emeritus_of_truce() -> CardDefinition {
    use super::creatures::inkling_token;
    let spell = spell_back(
        "Swords to Plowshares",
        cost(&[w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
            Effect::Exile {
                what: target_filtered(SelectionRequirement::Creature),
            },
        ]),
    );
    let mut front = vanilla_front(
        "Emeritus of Truce",
        cost(&[generic(1), w(), w()]),
        vec![CreatureType::Cat, CreatureType::Cleric],
        3,
        3,
        vec![],
        spell,
    );
    front.triggered_abilities.push(etb(Effect::Seq(vec![
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: inkling_token(),
        },
        Effect::If {
            cond: Predicate::AnOpponentControlsMoreCreatures,
            then: Box::new(becomes_prepared()),
            else_: Box::new(Effect::Noop),
        },
    ])));
    front
}

/// Elite Interceptor // Rejoinder — {W} // {1}{W}.
///
/// Front: 1/2 Human Wizard. This creature enters prepared.
///
/// Prepare spell: sorcery — "You may tap or untap target creature.
/// Draw a card." The "may" is modeled as a third Noop mode on the
/// `ChooseMode` (decline both tap and untap); the draw is unconditional.
pub fn elite_interceptor() -> CardDefinition {
    let spell = spell_back(
        "Rejoinder",
        cost(&[generic(1), w()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::ChooseMode(vec![
                Effect::Tap {
                    what: target_filtered(SelectionRequirement::Creature),
                },
                Effect::Untap {
                    what: target_filtered(SelectionRequirement::Creature),
                    up_to: None,
                },
                Effect::Noop,
            ]),
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
    );
    enters_prepared(vanilla_front(
        "Elite Interceptor",
        cost(&[w()]),
        vec![CreatureType::Human, CreatureType::Wizard],
        1,
        2,
        vec![],
        spell,
    ))
}

/// Honorbound Page // Forum's Favor — {3}{W} // {W}.
///
/// Front: 3/3 Cat Cleric with first strike. This creature enters
/// prepared.
///
/// Prepare spell: sorcery — target creature gets +1/+0 and gains flying
/// until end of turn.
pub fn honorbound_page() -> CardDefinition {
    let spell = spell_back(
        "Forum's Favor",
        cost(&[w()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            pump_target(1, 0),
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
        ]),
    );
    enters_prepared(vanilla_front(
        "Honorbound Page",
        cost(&[generic(3), w()]),
        vec![CreatureType::Cat, CreatureType::Cleric],
        3,
        3,
        vec![Keyword::FirstStrike],
        spell,
    ))
}

/// Joined Researchers // Secret Rendezvous — {1}{W} // {1}{W}{W}.
///
/// Front: 2/2 Human Cleric Wizard with first strike. "At the beginning
/// of your end step, if an opponent has more cards in hand than you,
/// this creature becomes prepared." (In multiplayer the hand-size read
/// over `PlayerRef::EachOpponent` collapses to the two-player case.)
///
/// Prepare spell: sorcery — printed "You and target opponent each draw
/// three cards."
///
/// Approximation: kept as the each-player fan-out via
/// `Selector::Player(PlayerRef::EachPlayer)` (same primitive Wheel of
/// Fortune uses in `lea::sorceries`) — equivalent to the printed text
/// in two-player games.
pub fn joined_researchers() -> CardDefinition {
    let spell = spell_back(
        "Secret Rendezvous",
        cost(&[generic(1), w(), w()]),
        CardType::Sorcery,
        Effect::Draw {
            who: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::Const(3),
        },
    );
    let mut front = vanilla_front(
        "Joined Researchers",
        cost(&[generic(1), w()]),
        vec![CreatureType::Human, CreatureType::Cleric, CreatureType::Wizard],
        2,
        2,
        vec![Keyword::FirstStrike],
        spell,
    );
    front.triggered_abilities.push(TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
        effect: Effect::If {
            cond: Predicate::ValueAtLeast(
                Value::HandSizeOf(PlayerRef::EachOpponent),
                Value::Sum(vec![Value::HandSizeOf(PlayerRef::You), Value::Const(1)]),
            ),
            then: Box::new(becomes_prepared()),
            else_: Box::new(Effect::Noop),
        },
    });
    front
}

/// Quill-Blade Laureate // Twofold Intent — {1}{W} // {1}{W}.
///
/// Front: 1/1 Human Cleric with double strike. This creature enters
/// prepared.
///
/// Prepare spell: sorcery — target creature gets +1/+0 and gains double
/// strike until end of turn.
pub fn quill_blade_laureate() -> CardDefinition {
    let spell = spell_back(
        "Twofold Intent",
        cost(&[generic(1), w()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            pump_target(1, 0),
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
        ]),
    );
    enters_prepared(vanilla_front(
        "Quill-Blade Laureate",
        cost(&[generic(1), w()]),
        vec![CreatureType::Human, CreatureType::Cleric],
        1,
        1,
        vec![Keyword::DoubleStrike],
        spell,
    ))
}

/// Spiritcall Enthusiast // Scrollboost — {2}{W} // {1}{W}.
///
/// Front: 3/3 Cat Cleric. "Whenever one or more tokens you control
/// enter, this creature becomes prepared." (The engine fires the
/// enters event per token; `AddCounter` of an already-present Prepared
/// counter is idempotent at count 1 per the prepared-flag convention.)
///
/// Prepare spell: sorcery — printed "One or two target creatures each
/// get +2/+2 until end of turn."
///
/// Approximation: the engine's target plumbing here is single-slot, so
/// the spell pumps one target creature +2/+2 until end of turn; the
/// optional second target is dropped.
pub fn spiritcall_enthusiast() -> CardDefinition {
    let spell = spell_back(
        "Scrollboost",
        cost(&[generic(1), w()]),
        CardType::Sorcery,
        pump_target(2, 2),
    );
    let mut front = vanilla_front(
        "Spiritcall Enthusiast",
        cost(&[generic(2), w()]),
        vec![CreatureType::Cat, CreatureType::Cleric],
        3,
        3,
        vec![],
        spell,
    );
    front.triggered_abilities.push(TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::IsToken,
            }),
        effect: becomes_prepared(),
    });
    front
}

// ── Blue preparation cards ──────────────────────────────────────────────────

/// Encouraging Aviator // Jump — {2}{U} // {U}.
///
/// Front: 2/3 Bird Wizard with flying. "Whenever this creature attacks,
/// it becomes prepared."
///
/// Prepare spell: instant — target creature gains flying until end of
/// turn.
pub fn encouraging_aviator() -> CardDefinition {
    let spell = spell_back(
        "Jump",
        cost(&[u()]),
        CardType::Instant,
        Effect::GrantKeyword {
            what: target_filtered(SelectionRequirement::Creature),
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        },
    );
    let mut front = vanilla_front(
        "Encouraging Aviator",
        cost(&[generic(2), u()]),
        vec![CreatureType::Bird, CreatureType::Wizard],
        2,
        3,
        vec![Keyword::Flying],
        spell,
    );
    front.triggered_abilities.push(on_attack(becomes_prepared()));
    front
}

/// Harmonized Trio // Brainstorm — {U} // {U}.
///
/// Front: 1/1 Merfolk Bard Wizard. "{T}, Tap two untapped creatures you
/// control: This creature becomes prepared."
///
/// Approximation: the engine's `tap_other_filter` activation cost taps
/// exactly **one** untapped matching permanent, so the cost taps this
/// creature plus one other untapped creature you control instead of the
/// printed two.
///
/// Prepare spell: instant — Brainstorm (draw 3, then put two cards from
/// your hand on top of your library).
pub fn harmonized_trio() -> CardDefinition {
    let spell = spell_back(
        "Brainstorm",
        cost(&[u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(3),
            },
            Effect::PutOnLibraryFromHand {
                who: PlayerRef::You,
                count: Value::Const(2),
            },
        ]),
    );
    let mut front = vanilla_front(
        "Harmonized Trio",
        cost(&[u()]),
        vec![CreatureType::Merfolk, CreatureType::Bard, CreatureType::Wizard],
        1,
        1,
        vec![],
        spell,
    );
    front.activated_abilities.push(ActivatedAbility {
        tap_cost: true,
        tap_other_filter: Some(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        ),
        effect: becomes_prepared(),
        ..Default::default()
    });
    front
}

// ── Black preparation cards ─────────────────────────────────────────────────

/// Cheerful Osteomancer // Raise Dead — {3}{B} // {B}.
///
/// Front: 4/2 Orc Warlock. This creature enters prepared.
///
/// Prepare spell: sorcery — return target creature card from your
/// graveyard to your hand.
pub fn cheerful_osteomancer() -> CardDefinition {
    let spell = spell_back(
        "Raise Dead",
        cost(&[b()]),
        CardType::Sorcery,
        Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Hand(PlayerRef::You),
        },
    );
    enters_prepared(vanilla_front(
        "Cheerful Osteomancer",
        cost(&[generic(3), b()]),
        vec![CreatureType::Orc, CreatureType::Warlock],
        4,
        2,
        vec![],
        spell,
    ))
}

/// Emeritus of Woe // Demonic Tutor — {3}{B} // {1}{B}.
///
/// Front: 5/4 Vampire Warlock. This creature enters prepared. "At the
/// beginning of your end step, if two or more creatures died this turn,
/// this creature becomes prepared."
///
/// Prepare spell: sorcery — search your library for a card and put it
/// into your hand.
pub fn emeritus_of_woe() -> CardDefinition {
    let spell = spell_back(
        "Demonic Tutor",
        cost(&[generic(1), b()]),
        CardType::Sorcery,
        Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Any,
            to: ZoneDest::Hand(PlayerRef::You),
        },
    );
    let mut front = enters_prepared(vanilla_front(
        "Emeritus of Woe",
        cost(&[generic(3), b()]),
        vec![CreatureType::Vampire, CreatureType::Warlock],
        5,
        4,
        vec![],
        spell,
    ));
    front.triggered_abilities.push(TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
        effect: Effect::If {
            cond: Predicate::CreaturesDiedThisTurnTotalAtLeast {
                at_least: Value::Const(2),
            },
            then: Box::new(becomes_prepared()),
            else_: Box::new(Effect::Noop),
        },
    });
    front
}

/// Scheming Silvertongue // Sign in Blood — {1}{B} // {B}{B}.
///
/// Front: 1/3 Vampire Warlock with flying and lifelink. "At the
/// beginning of your second main phase, if you gained 2 or more life
/// this turn, this creature becomes prepared."
///
/// Prepare spell: sorcery — Sign in Blood (target player draws 2 and
/// loses 2 life).
pub fn scheming_silvertongue() -> CardDefinition {
    let spell = spell_back(
        "Sign in Blood",
        cost(&[b(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Draw {
                who: target_filtered(SelectionRequirement::Player),
                amount: Value::Const(2),
            },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
            },
        ]),
    );
    let mut front = vanilla_front(
        "Scheming Silvertongue",
        cost(&[generic(1), b()]),
        vec![CreatureType::Vampire, CreatureType::Warlock],
        1,
        3,
        vec![Keyword::Flying, Keyword::Lifelink],
        spell,
    );
    front.triggered_abilities.push(TriggeredAbility {
        event: EventSpec::new(
            EventKind::StepBegins(TurnStep::PostCombatMain),
            EventScope::ActivePlayer,
        ),
        effect: Effect::If {
            cond: Predicate::LifeGainedThisTurnAtLeast {
                who: PlayerRef::You,
                at_least: Value::Const(2),
            },
            then: Box::new(becomes_prepared()),
            else_: Box::new(Effect::Noop),
        },
    });
    front
}

// ── Red preparation cards ───────────────────────────────────────────────────

/// Emeritus of Conflict // Lightning Bolt — {1}{R} // {R}.
///
/// Front: 2/2 Human Wizard with first strike. "Whenever you cast your
/// third spell each turn, this creature becomes prepared."
///
/// Prepare spell: instant — Lightning Bolt (deal 3 damage to any
/// target).
pub fn emeritus_of_conflict() -> CardDefinition {
    let spell = spell_back(
        "Lightning Bolt",
        cost(&[r()]),
        CardType::Instant,
        Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::Const(3),
        },
    );
    let mut front = vanilla_front(
        "Emeritus of Conflict",
        cost(&[generic(1), r()]),
        vec![CreatureType::Human, CreatureType::Wizard],
        2,
        2,
        vec![Keyword::FirstStrike],
        spell,
    );
    front.triggered_abilities.push(TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
            Predicate::SpellsCastThisTurnEquals {
                who: PlayerRef::You,
                count: Value::Const(3),
            },
        ),
        effect: becomes_prepared(),
    });
    front
}

/// Goblin Glasswright // Craft with Pride — {1}{R} // {R}.
///
/// Front: 2/2 Goblin Sorcerer. This creature enters prepared.
///
/// Prepare spell: sorcery — create a Treasure token.
pub fn goblin_glasswright() -> CardDefinition {
    let spell = spell_back(
        "Craft with Pride",
        cost(&[r()]),
        CardType::Sorcery,
        mint_treasures(1),
    );
    enters_prepared(vanilla_front(
        "Goblin Glasswright",
        cost(&[generic(1), r()]),
        vec![CreatureType::Goblin, CreatureType::Sorcerer],
        2,
        2,
        vec![],
        spell,
    ))
}

// ── Green preparation cards ─────────────────────────────────────────────────

/// Emeritus of Abundance // Regrowth — {2}{G} // {1}{G}.
///
/// Front: 3/4 Elf Druid with vigilance. This creature enters prepared.
/// "Whenever this creature attacks, if you control eight or more lands,
/// it becomes prepared."
///
/// Prepare spell: sorcery — return target card from your graveyard to
/// your hand.
pub fn emeritus_of_abundance() -> CardDefinition {
    let spell = spell_back(
        "Regrowth",
        cost(&[generic(1), g()]),
        CardType::Sorcery,
        Effect::Move {
            what: target_filtered(SelectionRequirement::Any),
            to: ZoneDest::Hand(PlayerRef::You),
        },
    );
    let mut front = enters_prepared(vanilla_front(
        "Emeritus of Abundance",
        cost(&[generic(2), g()]),
        vec![CreatureType::Elf, CreatureType::Druid],
        3,
        4,
        vec![Keyword::Vigilance],
        spell,
    ));
    front.triggered_abilities.push(on_attack(Effect::If {
        cond: Predicate::SelectorCountAtLeast {
            sel: Selector::EachPermanent(
                SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
            ),
            n: Value::Const(8),
        },
        then: Box::new(becomes_prepared()),
        else_: Box::new(Effect::Noop),
    }));
    front
}

/// Vastlands Scavenger // Bind to Life — {1}{G}{G} // {4}{G}.
///
/// Front: 4/4 Bear Druid with deathtouch. This creature enters
/// prepared.
///
/// Prepare spell: instant — printed "Mill seven cards. Then put a
/// creature card from among them onto the battlefield."
///
/// Approximation: the engine has no "from among the milled cards"
/// scratch selector, so after the mill we return one creature card from
/// your graveyard (auto-picked) to the battlefield — a pre-existing
/// graveyard creature can be chosen where the printed text restricts to
/// the seven just-milled cards.
pub fn vastlands_scavenger() -> CardDefinition {
    let spell = spell_back(
        "Bind to Life",
        cost(&[generic(4), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Mill {
                who: Selector::You,
                amount: Value::Const(7),
            },
            Effect::Move {
                what: Selector::take(
                    Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::Creature,
                    },
                    Value::Const(1),
                ),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
        ]),
    );
    enters_prepared(vanilla_front(
        "Vastlands Scavenger",
        cost(&[generic(1), g(), g()]),
        vec![CreatureType::Bear, CreatureType::Druid],
        4,
        4,
        vec![Keyword::Deathtouch],
        spell,
    ))
}

// ── Adventurous Eater // Have a Bite ────────────────────────────────────────

/// Adventurous Eater // Have a Bite — {2}{B} // {B}.
///
/// Front: 3/2 Human Warlock. This creature enters prepared.
///
/// Prepare spell: sorcery — put a +1/+1 counter on target creature. You
/// gain 1 life.
pub fn adventurous_eater() -> CardDefinition {
    let spell = spell_back(
        "Have a Bite",
        cost(&[b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
    );
    enters_prepared(vanilla_front(
        "Adventurous Eater",
        cost(&[generic(2), b()]),
        vec![CreatureType::Human, CreatureType::Warlock],
        3,
        2,
        vec![],
        spell,
    ))
}

// ── Leech Collector // Bloodletting ─────────────────────────────────────────

/// Leech Collector // Bloodletting — {1}{B} // {B}.
///
/// Front: 2/2 Human Warlock. "Whenever you gain life for the first time
/// each turn, this creature becomes prepared."
///
/// Prepare spell: sorcery — each opponent loses 2 life.
pub fn leech_collector() -> CardDefinition {
    let spell = spell_back(
        "Bloodletting",
        cost(&[b()]),
        CardType::Sorcery,
        Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(2),
        },
    );
    let mut front = vanilla_front(
        "Leech Collector",
        cost(&[generic(1), b()]),
        vec![CreatureType::Human, CreatureType::Warlock],
        2,
        2,
        vec![],
        spell,
    );
    front.triggered_abilities.push(TriggeredAbility {
        event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl).once_per_turn(),
        effect: becomes_prepared(),
    });
    front
}

// ── Pigment Wrangler // Striking Palette ────────────────────────────────────

/// Pigment Wrangler // Striking Palette — {4}{R} // {R}.
///
/// Front: 4/4 Orc Sorcerer with flying. This creature enters prepared.
///
/// Prepare spell: sorcery — printed "When you next cast an instant or
/// sorcery spell this turn, copy that spell. You may choose new targets
/// for the copy."
///
/// Approximation: the engine's `OnYourNextSpellCastThisTurn` delayed
/// trigger is consumed by the very next spell of *any* type — the body
/// is gated to copy only when that spell is an instant or sorcery, so
/// casting a creature first wastes the rider where the printed text
/// would keep waiting for an instant/sorcery.
pub fn pigment_wrangler() -> CardDefinition {
    use crate::effect::shortcut::cast_is_instant_or_sorcery;
    let spell = spell_back(
        "Striking Palette",
        cost(&[r()]),
        CardType::Sorcery,
        Effect::OnYourNextSpellCastThisTurn {
            body: Box::new(Effect::If {
                cond: cast_is_instant_or_sorcery(),
                then: Box::new(Effect::CopySpellMayChooseTargets {
                    what: Selector::TriggerSource,
                    count: Value::Const(1),
                }),
                else_: Box::new(Effect::Noop),
            }),
        },
    );
    enters_prepared(vanilla_front(
        "Pigment Wrangler",
        cost(&[generic(4), r()]),
        vec![CreatureType::Orc, CreatureType::Sorcerer],
        4,
        4,
        vec![Keyword::Flying],
        spell,
    ))
}

// ── Spellbook Seeker // Careful Study ───────────────────────────────────────

/// Spellbook Seeker // Careful Study — {3}{U} // {U}.
///
/// Front: 3/3 Bird Wizard with flying. This creature enters prepared.
///
/// Prepare spell: sorcery — draw 2 cards, then discard 2 cards (the
/// printed Careful Study oracle).
pub fn spellbook_seeker() -> CardDefinition {
    let spell = spell_back(
        "Careful Study",
        cost(&[u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(2),
                random: false,
            },
        ]),
    );
    enters_prepared(vanilla_front(
        "Spellbook Seeker",
        cost(&[generic(3), u()]),
        vec![CreatureType::Bird, CreatureType::Wizard],
        3,
        3,
        vec![Keyword::Flying],
        spell,
    ))
}

// ── Skycoach Conductor // All Aboard ────────────────────────────────────────

/// Skycoach Conductor // All Aboard — {2}{U} // {U}.
///
/// Front: 2/3 Bird Pilot with flash, flying, and vigilance. This
/// creature enters prepared.
///
/// Prepare spell: instant — exile target non-Pilot creature you
/// control, then return it to the battlefield under its owner's control
/// (the Ephemerate flicker shape: the second step re-resolves the bound
/// target in exile).
pub fn skycoach_conductor() -> CardDefinition {
    let spell = spell_back(
        "All Aboard",
        cost(&[u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::Not(Box::new(
                            SelectionRequirement::HasCreatureType(CreatureType::Pilot),
                        ))),
                ),
            },
            Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    tapped: false,
                },
            },
        ]),
    );
    enters_prepared(vanilla_front(
        "Skycoach Conductor",
        cost(&[generic(2), u()]),
        vec![CreatureType::Bird, CreatureType::Pilot],
        2,
        3,
        vec![Keyword::Flash, Keyword::Flying, Keyword::Vigilance],
        spell,
    ))
}

// ── Landscape Painter // Vibrant Idea ───────────────────────────────────────

/// Landscape Painter // Vibrant Idea — {1}{U} // {4}{U}.
///
/// Front: 2/1 Merfolk Wizard. This creature enters prepared.
///
/// Prepare spell: sorcery — draw two cards.
pub fn landscape_painter() -> CardDefinition {
    let spell = spell_back(
        "Vibrant Idea",
        cost(&[generic(4), u()]),
        CardType::Sorcery,
        Effect::Draw {
            who: Selector::You,
            amount: Value::Const(2),
        },
    );
    enters_prepared(vanilla_front(
        "Landscape Painter",
        cost(&[generic(1), u()]),
        vec![CreatureType::Merfolk, CreatureType::Wizard],
        2,
        1,
        vec![],
        spell,
    ))
}

// ── Blazing Firesinger // Seething Song ─────────────────────────────────────

/// Blazing Firesinger // Seething Song — {2}{R} // {2}{R}.
///
/// Front: 2/3 Dwarf Bard. This creature enters prepared.
///
/// Prepare spell: instant — Seething Song: add {R}{R}{R}{R}{R}.
pub fn blazing_firesinger() -> CardDefinition {
    let spell = spell_back(
        "Seething Song",
        cost(&[generic(2), r()]),
        CardType::Instant,
        Effect::AddMana {
            who: PlayerRef::You,
            pool: crate::effect::ManaPayload::Colors(vec![
                Color::Red, Color::Red, Color::Red, Color::Red, Color::Red,
            ]),
        },
    );
    enters_prepared(vanilla_front(
        "Blazing Firesinger",
        cost(&[generic(2), r()]),
        vec![CreatureType::Dwarf, CreatureType::Bard],
        2,
        3,
        vec![],
        spell,
    ))
}

// ── Maelstrom Artisan // Rocket Volley ──────────────────────────────────────

/// Maelstrom Artisan // Rocket Volley — {1}{R}{R} // {1}{R}.
///
/// Front: 3/2 Minotaur Sorcerer with haste. This creature enters
/// prepared.
///
/// Prepare spell: sorcery — destroy target nonbasic land.
pub fn maelstrom_artisan() -> CardDefinition {
    let spell = spell_back(
        "Rocket Volley",
        cost(&[generic(1), r()]),
        CardType::Sorcery,
        Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Land.and(SelectionRequirement::IsNonbasicLand),
            ),
        },
    );
    enters_prepared(vanilla_front(
        "Maelstrom Artisan",
        cost(&[generic(1), r(), r()]),
        vec![CreatureType::Minotaur, CreatureType::Sorcerer],
        3,
        2,
        vec![Keyword::Haste],
        spell,
    ))
}

// ── Scathing Shadelock // Venomous Words ────────────────────────────────────

/// Scathing Shadelock // Venomous Words — {4}{B} // {B}.
///
/// Front: 4/6 Snake Warlock. "At the beginning of your first main
/// phase, this creature becomes prepared."
///
/// Prepare spell: instant — target creature you control gets +2/+0 and
/// gains deathtouch until end of turn.
pub fn scathing_shadelock() -> CardDefinition {
    let spell = spell_back(
        "Venomous Words",
        cost(&[b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
        ]),
    );
    let mut front = vanilla_front(
        "Scathing Shadelock",
        cost(&[generic(4), b()]),
        vec![CreatureType::Snake, CreatureType::Warlock],
        4,
        6,
        vec![],
        spell,
    );
    front.triggered_abilities.push(TriggeredAbility {
        event: EventSpec::new(
            EventKind::StepBegins(TurnStep::PreCombatMain),
            EventScope::ActivePlayer,
        ),
        effect: becomes_prepared(),
    });
    front
}

// ── Infirmary Healer // Stream of Life ──────────────────────────────────────

/// Infirmary Healer // Stream of Life — {1}{G} // {X}{G}.
///
/// Front: 2/3 Cat Cleric. This creature enters prepared.
///
/// Prepare spell: sorcery — Stream of Life: target player gains X life.
/// (X comes from the spell's `{X}` slot.)
pub fn infirmary_healer() -> CardDefinition {
    let spell = spell_back(
        "Stream of Life",
        ManaCost {
            symbols: vec![
                crate::mana::ManaSymbol::X,
                crate::mana::ManaSymbol::Colored(Color::Green),
            ],
        },
        CardType::Sorcery,
        Effect::GainLife {
            who: target_filtered(SelectionRequirement::Player),
            amount: Value::XFromCost,
        },
    );
    enters_prepared(vanilla_front(
        "Infirmary Healer",
        cost(&[generic(1), g()]),
        vec![CreatureType::Cat, CreatureType::Cleric],
        2,
        3,
        vec![],
        spell,
    ))
}

// ── Jadzi, Steward of Fate // Oracle's Gift ─────────────────────────────────

/// Jadzi, Steward of Fate // Oracle's Gift — {2}{U} // {X}{X}{U}.
///
/// Front: 2/4 Legendary Human Wizard. This creature enters prepared.
/// "When this creature enters, draw two cards, then discard two cards."
///
/// Prepare spell: sorcery — create X 0/0 green and blue Fractal
/// creature tokens, then put X +1/+1 counters on each Fractal you
/// control.
pub fn jadzi_steward_of_fate() -> CardDefinition {
    let spell = spell_back(
        "Oracle's Gift",
        ManaCost {
            symbols: vec![
                crate::mana::ManaSymbol::X,
                crate::mana::ManaSymbol::X,
                crate::mana::ManaSymbol::Colored(Color::Blue),
            ],
        },
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::XFromCost,
                definition: crabomination_base::tokens::fractal_token(),
            },
            // Counters land on the freshly-minted batch in one shot —
            // a per-Fractal ForEach lets the SBA sweep the still-0/0
            // stragglers between iterations (only the first token
            // survived). Same pattern as Snarl Song / Wild Hypothesis.
            //
            // Approximation: printed text says *each Fractal you
            // control*; pre-existing Fractals (rare in practice) miss
            // the counters.
            Effect::AddCounter {
                what: Selector::LastCreatedTokens,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::XFromCost,
            },
        ]),
    );
    let mut front = enters_prepared(vanilla_front(
        "Jadzi, Steward of Fate",
        cost(&[generic(2), u()]),
        vec![CreatureType::Human, CreatureType::Wizard],
        2,
        4,
        vec![],
        spell,
    ));
    front.supertypes = vec![crate::card::Supertype::Legendary];
    front.triggered_abilities.push(etb(Effect::Seq(vec![
        Effect::Draw {
            who: Selector::You,
            amount: Value::Const(2),
        },
        Effect::Discard {
            who: Selector::You,
            amount: Value::Const(2),
            random: false,
        },
    ])));
    front
}

// ── Sanar, Unfinished Genius // Wild Idea ───────────────────────────────────

/// Sanar, Unfinished Genius // Wild Idea — {U}{R} // {3}{U}{R}.
///
/// Front: 0/4 Legendary Goblin Sorcerer. This creature enters prepared.
/// "{T}: Create a Treasure token. Activate only if you've cast an
/// instant or sorcery spell this turn."
///
/// Prepare spell: sorcery — search your library for an instant or
/// sorcery card, reveal it, put it into your hand, then shuffle.
pub fn sanar_unfinished_genius() -> CardDefinition {
    let spell = spell_back(
        "Wild Idea",
        cost(&[generic(3), u(), r()]),
        CardType::Sorcery,
        Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::HasCardType(CardType::Instant)
                .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            to: ZoneDest::Hand(PlayerRef::You),
        },
    );
    let mut front = enters_prepared(vanilla_front(
        "Sanar, Unfinished Genius",
        cost(&[u(), r()]),
        vec![CreatureType::Goblin, CreatureType::Sorcerer],
        0,
        4,
        vec![],
        spell,
    ));
    front.supertypes = vec![crate::card::Supertype::Legendary];
    front.activated_abilities.push(ActivatedAbility {
        tap_cost: true,
        effect: mint_treasures(1),
        condition: Some(Predicate::InstantsOrSorceriesCastThisTurnAtLeast {
            who: PlayerRef::You,
            at_least: Value::Const(1),
        }),
        ..Default::default()
    });
    front
}

// ── Tam, Observant Sequencer // Deep Sight ──────────────────────────────────

/// Tam, Observant Sequencer // Deep Sight — {2}{G}{U} // {G}{U}.
///
/// Front: 4/3 Legendary Gorgon Wizard. "Landfall — Whenever a land you
/// control enters, Tam becomes prepared."
///
/// Prepare spell: sorcery — you draw a card and gain 1 life.
pub fn tam_observant_sequencer() -> CardDefinition {
    let spell = spell_back(
        "Deep Sight",
        cost(&[g(), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
    );
    let mut front = vanilla_front(
        "Tam, Observant Sequencer",
        cost(&[generic(2), g(), u()]),
        // No `Gorgon` subtype today; use Wizard + close substitute Snake
        // (printed line is "Gorgon Wizard"; we approximate Gorgon as Snake).
        vec![CreatureType::Snake, CreatureType::Wizard],
        4,
        3,
        vec![],
        spell,
    );
    front.supertypes = vec![crate::card::Supertype::Legendary];
    front.triggered_abilities.push(TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::Land,
            }),
        effect: becomes_prepared(),
    });
    front
}

// ── Kirol, History Buff // Pack a Punch ─────────────────────────────────────

/// Kirol, History Buff // Pack a Punch — {R}{W} // {1}{R}{W}.
///
/// Front: 2/3 Legendary Vampire Cleric. "Whenever one or more cards
/// leave your graveyard, Kirol becomes prepared." (The engine fires
/// `CardLeftGraveyard` once per card; the prepared flag is idempotent
/// at one counter, matching the "one or more" batching.)
///
/// Prepare spell: sorcery — mill a card. Put two +1/+1 counters on
/// target creature. It gains trample until end of turn.
pub fn kirol_history_buff() -> CardDefinition {
    let spell = spell_back(
        "Pack a Punch",
        cost(&[generic(1), r(), w()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Mill {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
    );
    let mut front = vanilla_front(
        "Kirol, History Buff",
        cost(&[r(), w()]),
        vec![CreatureType::Vampire, CreatureType::Cleric],
        2,
        3,
        vec![],
        spell,
    );
    front.supertypes = vec![crate::card::Supertype::Legendary];
    front.triggered_abilities.push(TriggeredAbility {
        event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl),
        effect: becomes_prepared(),
    });
    front
}

// ── Abigale, Poet Laureate // Heroic Stanza ─────────────────────────────────

/// Abigale, Poet Laureate // Heroic Stanza — {1}{W}{B} // {1}{W/B}.
///
/// Front: 2/3 Legendary Bird Bard with flying. "Whenever you cast a
/// creature spell, Abigale becomes prepared."
///
/// Prepare spell: sorcery — put a +1/+1 counter on target creature.
/// (The `{W/B}` pip is a real `ManaSymbol::Hybrid(White, Black)`.)
pub fn abigale_poet_laureate() -> CardDefinition {
    let spell = spell_back(
        "Heroic Stanza",
        cost(&[generic(1), crate::mana::hybrid(Color::White, Color::Black)]),
        CardType::Sorcery,
        Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        },
    );
    let mut front = vanilla_front(
        "Abigale, Poet Laureate",
        cost(&[generic(1), w(), b()]),
        vec![CreatureType::Bird, CreatureType::Bard],
        2,
        3,
        vec![Keyword::Flying],
        spell,
    );
    front.supertypes = vec![crate::card::Supertype::Legendary];
    front.triggered_abilities.push(TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
            Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::HasCardType(CardType::Creature),
            },
        ),
        effect: becomes_prepared(),
    });
    front
}

// ── Lluwen, Exchange Student // Pest Friend ─────────────────────────────────

/// Lluwen, Exchange Student // Pest Friend — {2}{B}{G} // {B/G}.
///
/// Front: 3/4 Legendary Elf Druid. This creature enters prepared.
/// "Exile a creature card from your graveyard: This creature becomes
/// prepared. Activate only as a sorcery."
///
/// Prepare spell: sorcery — create a 1/1 black-and-green Pest creature
/// token with the printed "Whenever this token attacks, you gain 1
/// life" rider. The `{B/G}` pip is a real `ManaSymbol::Hybrid(Black,
/// Green)`, payable with either black or green (matches the Witherbloom
/// convention used by Essenceknit Scholar's `{B/G}` and Practiced
/// Scrollsmith's `{R/W}` pips).
pub fn lluwen_exchange_student() -> CardDefinition {
    use super::sorceries::pest_token;
    let spell = spell_back(
        "Pest Friend",
        cost(&[crate::mana::hybrid(Color::Black, Color::Green)]),
        CardType::Sorcery,
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: pest_token(),
        },
    );
    let mut front = enters_prepared(vanilla_front(
        "Lluwen, Exchange Student",
        cost(&[generic(2), b(), g()]),
        vec![CreatureType::Elf, CreatureType::Druid],
        3,
        4,
        vec![],
        spell,
    ));
    front.supertypes = vec![crate::card::Supertype::Legendary];
    front.activated_abilities.push(ActivatedAbility {
        exile_other_filter: Some((SelectionRequirement::Creature, 1)),
        sorcery_speed: true,
        effect: becomes_prepared(),
        ..Default::default()
    });
    front
}

// ── Campus Composer // Aqueous Aria ────────────────────────────────────────

/// Campus Composer // Aqueous Aria — {3}{U} // {4}{U}.
///
/// Front: 3/4 Merfolk Bard with Ward {2}. This creature enters
/// prepared.
///
/// Prepare spell: sorcery — create a 3/3 blue and red Elemental
/// creature token with flying.
pub fn campus_composer() -> CardDefinition {
    use super::sorceries::elemental_token;
    let spell = spell_back(
        "Aqueous Aria",
        cost(&[generic(4), u()]),
        CardType::Sorcery,
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: elemental_token(),
        },
    );
    enters_prepared(vanilla_front(
        "Campus Composer",
        cost(&[generic(3), u()]),
        vec![CreatureType::Merfolk, CreatureType::Bard],
        3,
        4,
        vec![Keyword::Ward(WardCost::generic(2))],
        spell,
    ))
}

// ── Emeritus of Ideation // Ancestral Recall ──────────────────────────────

/// Emeritus of Ideation // Ancestral Recall — {3}{U}{U} // {U}.
///
/// Front: 5/5 Human Wizard with flying and Ward {2}. This creature
/// enters prepared. "Whenever this creature attacks, you may exile
/// eight cards from your graveyard. If you do, it becomes prepared."
///
/// Prepare spell: instant — Ancestral Recall: target player draws 3
/// cards.
pub fn emeritus_of_ideation() -> CardDefinition {
    let spell = spell_back(
        "Ancestral Recall",
        cost(&[u()]),
        CardType::Instant,
        Effect::Draw {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Const(3),
        },
    );
    let mut front = enters_prepared(vanilla_front(
        "Emeritus of Ideation",
        cost(&[generic(3), u(), u()]),
        vec![CreatureType::Human, CreatureType::Wizard],
        5,
        5,
        vec![Keyword::Flying, Keyword::Ward(WardCost::generic(2))],
        spell,
    ));
    front.triggered_abilities.push(on_attack(Effect::If {
        cond: cards_in_graveyard_at_least(SelectionRequirement::Any, 8),
        then: Box::new(Effect::MayDo {
            description: "Exile eight cards from your graveyard to become prepared".to_string(),
            body: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: Selector::take(
                        Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Graveyard,
                            filter: SelectionRequirement::Any,
                        },
                        Value::Const(8),
                    ),
                    to: ZoneDest::Exile,
                },
                becomes_prepared(),
            ])),
        }),
        else_: Box::new(Effect::Noop),
    }));
    front
}

// ── Grave Researcher // Reanimate ─────────────────────────────────────────

/// Grave Researcher // Reanimate — {2}{B} // {B}.
///
/// Front: 3/3 Troll Warlock. "At the beginning of your upkeep, surveil
/// 1. Then if there are three or more creature cards in your graveyard,
/// this creature becomes prepared."
///
/// Prepare spell: sorcery — Reanimate: return target creature card from
/// your graveyard to the battlefield; you lose life equal to its mana
/// value.
pub fn grave_researcher() -> CardDefinition {
    let spell = spell_back(
        "Reanimate",
        cost(&[b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature,
                },
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
            },
        ]),
    );
    let mut front = vanilla_front(
        "Grave Researcher",
        cost(&[generic(2), b()]),
        vec![CreatureType::Troll, CreatureType::Warlock],
        3,
        3,
        vec![],
        spell,
    );
    front.triggered_abilities.push(TriggeredAbility {
        event: EventSpec::new(
            EventKind::StepBegins(TurnStep::Upkeep),
            EventScope::ActivePlayer,
        ),
        effect: Effect::Seq(vec![
            Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::If {
                cond: cards_in_graveyard_at_least(SelectionRequirement::Creature, 3),
                then: Box::new(becomes_prepared()),
                else_: Box::new(Effect::Noop),
            },
        ]),
    });
    front
}

// ── Strife Scholar // Awaken the Ages ─────────────────────────────────────

/// Strife Scholar // Awaken the Ages — {2}{R} // {5}{R}.
///
/// Front: 3/2 Orc Sorcerer with "Ward—Pay 2 life." This creature enters
/// prepared.
///
/// Prepare spell: sorcery — create two 2/2 red and white Spirit
/// creature tokens.
pub fn strife_scholar() -> CardDefinition {
    let spell = spell_back(
        "Awaken the Ages",
        cost(&[generic(5), r()]),
        CardType::Sorcery,
        mint_lorehold_spirits(2),
    );
    enters_prepared(vanilla_front(
        "Strife Scholar",
        cost(&[generic(2), r()]),
        vec![CreatureType::Orc, CreatureType::Sorcerer],
        3,
        2,
        vec![Keyword::Ward(WardCost::Life(2))],
        spell,
    ))
}
