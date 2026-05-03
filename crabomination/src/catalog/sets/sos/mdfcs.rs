//! Secrets of Strixhaven (SOS) — "Prepared" cards (engine-invented MDFCs).
//!
//! Each card here is a **prepared card**: a vanilla-ish creature front
//! face glued to a reprinted spell back face (the *prepare spell*). See
//! `STRIXHAVEN2.md` → "Prepare mechanic" → Half 1, and `.claude/prepared.md`
//! for the cross-cutting context. Mechanically these ride the engine's
//! existing MDFC plumbing — the front cast routes through
//! `GameAction::CastSpell` and the back through
//! `GameAction::CastSpellBack` (added in push X) — but the front+back
//! pair is engine-invented, so Scryfall has no MDFC printing for them.
//! The client prefetcher handles that via a 422-fallback in
//! `crabomination_client::scryfall::download_card_image` (queries the
//! back name directly when `face=back` on the front 422s).
//!
//! The back face's `CardDefinition` is constructed inline because each
//! spell needs slightly different `effect`/cost plumbing — keeping the
//! helpers out of `super::sorceries` avoids cluttering the named-spell
//! module with one-off MDFC variants.
//!
//! Cards in this module:
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

use super::no_abilities;
use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Effect, Keyword, SelectionRequirement,
    Subtypes,
};
use crate::effect::shortcut::{
    counter_target_spell, exile_target, pump_target, target_filtered,
};
use crate::effect::{Duration, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};
use crate::card::Zone;

/// Helper: standard plain-creature front face. Strixhaven MDFC creature
/// front faces are nearly all vanilla bodies with a printed spell back —
/// the only varying fields are name, cost, P/T, and creature subtypes.
fn vanilla_front(
    name: &'static str,
    front_cost: ManaCost,
    creature_types: Vec<CreatureType>,
    power: i32,
    toughness: i32,
    keywords: Vec<Keyword>,
    back: CardDefinition,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: front_cost,
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types,
            ..Default::default()
        },
        power,
        toughness,
        keywords,
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: Some(Box::new(back)),
        opening_hand: None,
    }
}

/// Helper: a back-face spell built from a name/type/cost/effect tuple.
fn spell_back(
    name: &'static str,
    cost: ManaCost,
    card_type: CardType,
    effect: Effect,
) -> CardDefinition {
    CardDefinition {
        name,
        cost,
        supertypes: vec![],
        card_types: vec![card_type],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── White MDFCs ─────────────────────────────────────────────────────────────

/// Emeritus of Truce // Swords to Plowshares — {1}{W}{W} // {W}.
///
/// Front: 3/3 Cat Cleric vanilla. Back: instant — exile target creature;
/// its controller gains life equal to that creature's power.
///
/// Approximation: the printed Swords to Plowshares lifegain rider keys
/// off the *target's controller*. The engine has no opponent-directed
/// `GainLife { who: PlayerRef::ControllerOf(Selector) }` shape, so we
/// approximate as the target's *owner* gaining life. The exile half is
/// faithful.
pub fn emeritus_of_truce() -> CardDefinition {
    let back = spell_back(
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
    vanilla_front(
        "Emeritus of Truce",
        cost(&[generic(1), w(), w()]),
        vec![CreatureType::Cat, CreatureType::Cleric],
        3,
        3,
        vec![],
        back,
    )
}

/// Elite Interceptor // Rejoinder — {W} // {1}{W}.
///
/// Front: 1/2 Human Wizard vanilla. Back: sorcery — counter target
/// creature spell. (Standard card name "Rejoinder" approximates a
/// White-coloured creature counterspell.)
pub fn elite_interceptor() -> CardDefinition {
    let back = spell_back(
        "Rejoinder",
        cost(&[generic(1), w()]),
        CardType::Sorcery,
        Effect::CounterSpell {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack
                    .and(SelectionRequirement::HasCardType(CardType::Creature)),
            ),
        },
    );
    vanilla_front(
        "Elite Interceptor",
        cost(&[w()]),
        vec![CreatureType::Human, CreatureType::Wizard],
        1,
        2,
        vec![],
        back,
    )
}

/// Honorbound Page // Forum's Favor — {3}{W} // {W}.
///
/// Front: 3/3 Cat Cleric vanilla. Back: sorcery — Forum's Favor: target
/// creature gets +1/+1 until end of turn. You gain 1 life.
pub fn honorbound_page() -> CardDefinition {
    let back = spell_back(
        "Forum's Favor",
        cost(&[w()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            pump_target(1, 1),
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
    );
    vanilla_front(
        "Honorbound Page",
        cost(&[generic(3), w()]),
        vec![CreatureType::Cat, CreatureType::Cleric],
        3,
        3,
        vec![],
        back,
    )
}

/// Joined Researchers // Secret Rendezvous — {1}{W} // {1}{W}{W}.
///
/// Front: 2/2 Human Cleric Wizard vanilla. Back: sorcery — each player
/// draws three cards. (A symmetrical wheel-style card-draw spell.)
///
/// Approximation: collapses "each player" to "you draw 3" because the
/// engine has no "each player draws N" iteration that survives shuffle
/// state. A future ForEach Player loop with `Effect::Draw` body would
/// faithfully fan out.
pub fn joined_researchers() -> CardDefinition {
    let back = spell_back(
        "Secret Rendezvous",
        cost(&[generic(1), w(), w()]),
        CardType::Sorcery,
        Effect::Draw {
            who: Selector::You,
            amount: Value::Const(3),
        },
    );
    vanilla_front(
        "Joined Researchers",
        cost(&[generic(1), w()]),
        vec![CreatureType::Human, CreatureType::Cleric, CreatureType::Wizard],
        2,
        2,
        vec![],
        back,
    )
}

/// Quill-Blade Laureate // Twofold Intent — {1}{W} // {1}{W}.
///
/// Front: 1/1 Human Cleric vanilla. Back: sorcery — target creature you
/// control gets +1/+1 until end of turn. Create a 1/1 white and black
/// Inkling creature token with flying.
pub fn quill_blade_laureate() -> CardDefinition {
    use super::creatures::inkling_token;
    let back = spell_back(
        "Twofold Intent",
        cost(&[generic(1), w()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: inkling_token(),
            },
        ]),
    );
    vanilla_front(
        "Quill-Blade Laureate",
        cost(&[generic(1), w()]),
        vec![CreatureType::Human, CreatureType::Cleric],
        1,
        1,
        vec![],
        back,
    )
}

/// Spiritcall Enthusiast // Scrollboost — {2}{W} // {1}{W}.
///
/// Front: 3/3 Cat Cleric vanilla. Back: sorcery — put a +1/+1 counter on
/// each creature you control.
pub fn spiritcall_enthusiast() -> CardDefinition {
    let back = spell_back(
        "Scrollboost",
        cost(&[generic(1), w()]),
        CardType::Sorcery,
        Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            body: Box::new(Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        },
    );
    vanilla_front(
        "Spiritcall Enthusiast",
        cost(&[generic(2), w()]),
        vec![CreatureType::Cat, CreatureType::Cleric],
        3,
        3,
        vec![],
        back,
    )
}

// ── Blue MDFCs ──────────────────────────────────────────────────────────────

/// Encouraging Aviator // Jump — {2}{U} // {U}.
///
/// Front: 2/3 Bird Wizard vanilla. Back: instant — target creature gains
/// flying until end of turn.
pub fn encouraging_aviator() -> CardDefinition {
    let back = spell_back(
        "Jump",
        cost(&[u()]),
        CardType::Instant,
        Effect::GrantKeyword {
            what: target_filtered(SelectionRequirement::Creature),
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        },
    );
    vanilla_front(
        "Encouraging Aviator",
        cost(&[generic(2), u()]),
        vec![CreatureType::Bird, CreatureType::Wizard],
        2,
        3,
        vec![Keyword::Flying],
        back,
    )
}

/// Harmonized Trio // Brainstorm — {U} // {U}.
///
/// Front: 1/1 Merfolk Bard Wizard vanilla. Back: instant — Brainstorm
/// (draw 3, then put two cards from your hand on top of your library).
pub fn harmonized_trio() -> CardDefinition {
    let back = spell_back(
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
    vanilla_front(
        "Harmonized Trio",
        cost(&[u()]),
        vec![CreatureType::Merfolk, CreatureType::Bard, CreatureType::Wizard],
        1,
        1,
        vec![],
        back,
    )
}

/// Emeritus of Ideation // Ancestral Recall — {3}{U}{U} // {U}.
///
/// Front: 5/5 Human Wizard with `Keyword::Ward(2)` (tagged for future
/// enforcement, same as Inkshape Demonstrator). Back: instant —
/// Ancestral Recall (target player draws three cards). Power Nine
/// reprint as the back face; the engine has a faithful 3-card
/// `Effect::Draw` already.
pub fn emeritus_of_ideation() -> CardDefinition {
    let back = spell_back(
        "Ancestral Recall",
        cost(&[u()]),
        CardType::Instant,
        Effect::Draw {
            who: target_filtered(SelectionRequirement::Player),
            amount: Value::Const(3),
        },
    );
    vanilla_front(
        "Emeritus of Ideation",
        cost(&[generic(3), u(), u()]),
        vec![CreatureType::Human, CreatureType::Wizard],
        5,
        5,
        vec![Keyword::Ward(2)],
        back,
    )
}

// ── Black MDFCs ─────────────────────────────────────────────────────────────

/// Cheerful Osteomancer // Raise Dead — {3}{B} // {B}.
///
/// Front: 4/2 Orc Warlock vanilla. Back: sorcery — return target creature
/// card from your graveyard to your hand.
pub fn cheerful_osteomancer() -> CardDefinition {
    let back = spell_back(
        "Raise Dead",
        cost(&[b()]),
        CardType::Sorcery,
        Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Hand(PlayerRef::You),
        },
    );
    vanilla_front(
        "Cheerful Osteomancer",
        cost(&[generic(3), b()]),
        vec![CreatureType::Orc, CreatureType::Warlock],
        4,
        2,
        vec![],
        back,
    )
}

/// Emeritus of Woe // Demonic Tutor — {3}{B} // {1}{B}.
///
/// Front: 5/4 Vampire Warlock vanilla. Back: sorcery — search your
/// library for a card and put it into your hand.
pub fn emeritus_of_woe() -> CardDefinition {
    let back = spell_back(
        "Demonic Tutor",
        cost(&[generic(1), b()]),
        CardType::Sorcery,
        Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Any,
            to: ZoneDest::Hand(PlayerRef::You),
        },
    );
    vanilla_front(
        "Emeritus of Woe",
        cost(&[generic(3), b()]),
        vec![CreatureType::Vampire, CreatureType::Warlock],
        5,
        4,
        vec![],
        back,
    )
}

/// Scheming Silvertongue // Sign in Blood — {1}{B} // {B}{B}.
///
/// Front: 1/3 Vampire Warlock vanilla. Back: sorcery — Sign in Blood
/// (target player draws 2 and loses 2 life).
pub fn scheming_silvertongue() -> CardDefinition {
    let back = spell_back(
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
    vanilla_front(
        "Scheming Silvertongue",
        cost(&[generic(1), b()]),
        vec![CreatureType::Vampire, CreatureType::Warlock],
        1,
        3,
        vec![],
        back,
    )
}

/// Grave Researcher // Reanimate — {2}{B} // {B}.
///
/// Front: 3/3 Troll Warlock vanilla. ETB Surveil 2 (Surveil is a
/// first-class engine primitive). Back: sorcery — Reanimate
/// (return target creature card from a graveyard to the battlefield
/// under your control). The full printed Reanimate also has "you lose
/// life equal to that creature's mana value", which we approximate via
/// `Effect::LoseLife { who: You, amount: ManaValueOf(Target(0)) }`
/// using the engine's `ManaValueOf` value primitive that walks
/// graveyards as well as the battlefield.
pub fn grave_researcher() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    let back = spell_back(
        "Reanimate",
        cost(&[b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
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
        back,
    );
    // Front-face ETB: Surveil 2.
    front.triggered_abilities = vec![TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::Const(2),
        },
    }];
    front
}

// ── Red MDFCs ───────────────────────────────────────────────────────────────

/// Emeritus of Conflict // Lightning Bolt — {1}{R} // {R}.
///
/// Front: 2/2 Human Wizard vanilla. Back: instant — Lightning Bolt
/// (deal 3 damage to any target).
pub fn emeritus_of_conflict() -> CardDefinition {
    let back = spell_back(
        "Lightning Bolt",
        cost(&[r()]),
        CardType::Instant,
        Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::Const(3),
        },
    );
    vanilla_front(
        "Emeritus of Conflict",
        cost(&[generic(1), r()]),
        vec![CreatureType::Human, CreatureType::Wizard],
        2,
        2,
        vec![],
        back,
    )
}

/// Goblin Glasswright // Craft with Pride — {1}{R} // {R}.
///
/// Front: 2/2 Goblin Sorcerer vanilla. Back: sorcery — target creature
/// gets +2/+0 and gains haste until end of turn.
pub fn goblin_glasswright() -> CardDefinition {
    let back = spell_back(
        "Craft with Pride",
        cost(&[r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            pump_target(2, 0),
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
    );
    vanilla_front(
        "Goblin Glasswright",
        cost(&[generic(1), r()]),
        vec![CreatureType::Goblin, CreatureType::Sorcerer],
        2,
        2,
        vec![],
        back,
    )
}

// ── Green MDFCs ─────────────────────────────────────────────────────────────

/// Emeritus of Abundance // Regrowth — {2}{G} // {1}{G}.
///
/// Front: 3/4 Elf Druid vanilla. Back: sorcery — return target card from
/// your graveyard to your hand.
pub fn emeritus_of_abundance() -> CardDefinition {
    let back = spell_back(
        "Regrowth",
        cost(&[generic(1), g()]),
        CardType::Sorcery,
        Effect::Move {
            what: target_filtered(SelectionRequirement::Any),
            to: ZoneDest::Hand(PlayerRef::You),
        },
    );
    vanilla_front(
        "Emeritus of Abundance",
        cost(&[generic(2), g()]),
        vec![CreatureType::Elf, CreatureType::Druid],
        3,
        4,
        vec![],
        back,
    )
}

/// Vastlands Scavenger // Bind to Life — {1}{G}{G} // {4}{G}.
///
/// Front: 4/4 Bear Druid with Trample vanilla. Back: instant — return up
/// to two target creature cards from your graveyard to the battlefield
/// under your control. (Faithful via Selector::Take(_, 2).)
pub fn vastlands_scavenger() -> CardDefinition {
    let back = spell_back(
        "Bind to Life",
        cost(&[generic(4), g()]),
        CardType::Instant,
        Effect::Move {
            what: Selector::take(
                Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Creature,
                },
                Value::Const(2),
            ),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        },
    );
    vanilla_front(
        "Vastlands Scavenger",
        cost(&[generic(1), g(), g()]),
        vec![CreatureType::Bear, CreatureType::Druid],
        4,
        4,
        vec![Keyword::Trample],
        back,
    )
}

// ── Adventure-style: Adventurous Eater // Have a Bite ───────────────────────

/// Adventurous Eater // Have a Bite — {2}{B} // {B}.
///
/// Front: 3/2 Human Warlock vanilla. Back: sorcery — target creature
/// gets -3/-3 until end of turn.
pub fn adventurous_eater() -> CardDefinition {
    let back = spell_back(
        "Have a Bite",
        cost(&[b()]),
        CardType::Sorcery,
        Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-3),
            toughness: Value::Const(-3),
            duration: Duration::EndOfTurn,
        },
    );
    vanilla_front(
        "Adventurous Eater",
        cost(&[generic(2), b()]),
        vec![CreatureType::Human, CreatureType::Warlock],
        3,
        2,
        vec![],
        back,
    )
}

// ── Bonus: Leech Collector // Bloodletting ──────────────────────────────────

/// Leech Collector // Bloodletting — {1}{B} // {B}.
///
/// Front: 2/2 Human Warlock vanilla. Back: sorcery — each opponent loses
/// 2 life and you gain 2 life.
pub fn leech_collector() -> CardDefinition {
    let back = spell_back(
        "Bloodletting",
        cost(&[b()]),
        CardType::Sorcery,
        Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(2),
        },
    );
    vanilla_front(
        "Leech Collector",
        cost(&[generic(1), b()]),
        vec![CreatureType::Human, CreatureType::Warlock],
        2,
        2,
        vec![],
        back,
    )
}

// ── Bonus: Pigment Wrangler // Striking Palette ─────────────────────────────

/// Pigment Wrangler // Striking Palette — {4}{R} // {R}.
///
/// Front: 4/4 Orc Sorcerer vanilla. Back: sorcery — Pigment Wrangler
/// deals 2 damage to any target.
pub fn pigment_wrangler() -> CardDefinition {
    let back = spell_back(
        "Striking Palette",
        cost(&[r()]),
        CardType::Sorcery,
        Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::Const(2),
        },
    );
    vanilla_front(
        "Pigment Wrangler",
        cost(&[generic(4), r()]),
        vec![CreatureType::Orc, CreatureType::Sorcerer],
        4,
        4,
        vec![],
        back,
    )
}

// ── Spellbook Seeker // Careful Study ───────────────────────────────────────

/// Spellbook Seeker // Careful Study — {3}{U} // {U}.
///
/// Front: 3/3 Bird Wizard with Flying. Back: sorcery — draw 2 cards,
/// then discard 2 cards (the printed Careful Study oracle).
pub fn spellbook_seeker() -> CardDefinition {
    let back = spell_back(
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
    vanilla_front(
        "Spellbook Seeker",
        cost(&[generic(3), u()]),
        vec![CreatureType::Bird, CreatureType::Wizard],
        3,
        3,
        vec![Keyword::Flying],
        back,
    )
}

// ── Skycoach Conductor // All Aboard ────────────────────────────────────────

/// Skycoach Conductor // All Aboard — {2}{U} // {U}.
///
/// Front: 2/3 Bird Pilot with Flying. Back: instant — return target
/// creature to its owner's hand.
pub fn skycoach_conductor() -> CardDefinition {
    let back = spell_back(
        "All Aboard",
        cost(&[u()]),
        CardType::Instant,
        Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        },
    );
    vanilla_front(
        "Skycoach Conductor",
        cost(&[generic(2), u()]),
        vec![CreatureType::Bird, CreatureType::Pilot],
        2,
        3,
        vec![Keyword::Flying],
        back,
    )
}

// ── Landscape Painter // Vibrant Idea ───────────────────────────────────────

/// Landscape Painter // Vibrant Idea — {1}{U} // {4}{U}.
///
/// Front: 2/1 Merfolk Wizard. Back: sorcery — draw three cards.
pub fn landscape_painter() -> CardDefinition {
    let back = spell_back(
        "Vibrant Idea",
        cost(&[generic(4), u()]),
        CardType::Sorcery,
        Effect::Draw {
            who: Selector::You,
            amount: Value::Const(3),
        },
    );
    vanilla_front(
        "Landscape Painter",
        cost(&[generic(1), u()]),
        vec![CreatureType::Merfolk, CreatureType::Wizard],
        2,
        1,
        vec![],
        back,
    )
}

// ── Blazing Firesinger // Seething Song ─────────────────────────────────────

/// Blazing Firesinger // Seething Song — {2}{R} // {2}{R}.
///
/// Front: 2/3 Dwarf Bard. Back: instant — Seething Song: add {R}{R}{R}{R}{R}.
pub fn blazing_firesinger() -> CardDefinition {
    let back = spell_back(
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
    vanilla_front(
        "Blazing Firesinger",
        cost(&[generic(2), r()]),
        vec![CreatureType::Dwarf, CreatureType::Bard],
        2,
        3,
        vec![],
        back,
    )
}

// ── Maelstrom Artisan // Rocket Volley ──────────────────────────────────────

/// Maelstrom Artisan // Rocket Volley — {1}{R}{R} // {1}{R}.
///
/// Front: 3/2 Minotaur Sorcerer. Back: sorcery — Rocket Volley deals 2
/// damage to each opponent and 2 damage to up to one creature an
/// opponent controls. (Approximation: collapses into "2 damage to each
/// opp + 2 damage to one creature the auto-decider picks".)
pub fn maelstrom_artisan() -> CardDefinition {
    let back = spell_back(
        "Rocket Volley",
        cost(&[generic(1), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
                amount: Value::Const(2),
            },
        ]),
    );
    vanilla_front(
        "Maelstrom Artisan",
        cost(&[generic(1), r(), r()]),
        vec![CreatureType::Minotaur, CreatureType::Sorcerer],
        3,
        2,
        vec![],
        back,
    )
}

// ── Scathing Shadelock // Venomous Words ────────────────────────────────────

/// Scathing Shadelock // Venomous Words — {4}{B} // {B}.
///
/// Front: 4/6 Snake Warlock with Deathtouch. Back: instant — target
/// creature gets -2/-2 until end of turn.
pub fn scathing_shadelock() -> CardDefinition {
    let back = spell_back(
        "Venomous Words",
        cost(&[b()]),
        CardType::Instant,
        Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        },
    );
    vanilla_front(
        "Scathing Shadelock",
        cost(&[generic(4), b()]),
        vec![CreatureType::Snake, CreatureType::Warlock],
        4,
        6,
        vec![Keyword::Deathtouch],
        back,
    )
}

// ── Infirmary Healer // Stream of Life ──────────────────────────────────────

/// Infirmary Healer // Stream of Life — {1}{G} // {X}{G}.
///
/// Front: 2/3 Cat Cleric with Lifelink. Back: sorcery — Stream of Life:
/// gain X life. (X comes from the spell's `{X}` slot.)
pub fn infirmary_healer() -> CardDefinition {
    let back = spell_back(
        "Stream of Life",
        ManaCost {
            symbols: vec![
                crate::mana::ManaSymbol::X,
                crate::mana::ManaSymbol::Colored(Color::Green),
            ],
        },
        CardType::Sorcery,
        Effect::GainLife {
            who: Selector::You,
            amount: Value::XFromCost,
        },
    );
    vanilla_front(
        "Infirmary Healer",
        cost(&[generic(1), g()]),
        vec![CreatureType::Cat, CreatureType::Cleric],
        2,
        3,
        vec![Keyword::Lifelink],
        back,
    )
}

// ── Jadzi, Steward of Fate // Oracle's Gift ─────────────────────────────────

/// Jadzi, Steward of Fate // Oracle's Gift — {2}{U} // {X}{X}{U}.
///
/// Front: 2/4 Legendary Human Wizard with Flying. Back: sorcery — draw
/// 2X cards. (X^2 here would be runaway; we use 2X as a faithful
/// approximation that scales linearly with the {X} pip.)
pub fn jadzi_steward_of_fate() -> CardDefinition {
    let back = spell_back(
        "Oracle's Gift",
        ManaCost {
            symbols: vec![
                crate::mana::ManaSymbol::X,
                crate::mana::ManaSymbol::X,
                crate::mana::ManaSymbol::Colored(Color::Blue),
            ],
        },
        CardType::Sorcery,
        Effect::Draw {
            who: Selector::You,
            amount: Value::Times(Box::new(Value::Const(2)), Box::new(Value::XFromCost)),
        },
    );
    let mut front = vanilla_front(
        "Jadzi, Steward of Fate",
        cost(&[generic(2), u()]),
        vec![CreatureType::Human, CreatureType::Wizard],
        2,
        4,
        vec![Keyword::Flying],
        back,
    );
    front.supertypes = vec![crate::card::Supertype::Legendary];
    front
}

// ── Sanar, Unfinished Genius // Wild Idea ───────────────────────────────────

/// Sanar, Unfinished Genius // Wild Idea — {U}{R} // {3}{U}{R}.
///
/// Front: 0/4 Legendary Goblin Sorcerer. Back: sorcery — Wild Idea: each
/// player draws three cards. (Approximated as caster draws 3 — multi-
/// player iteration on draws is a separate gap.)
pub fn sanar_unfinished_genius() -> CardDefinition {
    let back = spell_back(
        "Wild Idea",
        cost(&[generic(3), u(), r()]),
        CardType::Sorcery,
        Effect::Draw {
            who: Selector::You,
            amount: Value::Const(3),
        },
    );
    let mut front = vanilla_front(
        "Sanar, Unfinished Genius",
        cost(&[u(), r()]),
        vec![CreatureType::Goblin, CreatureType::Sorcerer],
        0,
        4,
        vec![],
        back,
    );
    front.supertypes = vec![crate::card::Supertype::Legendary];
    front
}

// ── Tam, Observant Sequencer // Deep Sight ──────────────────────────────────

/// Tam, Observant Sequencer // Deep Sight — {2}{G}{U} // {G}{U}.
///
/// Front: 4/3 Legendary Gorgon Wizard. Back: sorcery — Deep Sight:
/// scry 4, then draw a card.
pub fn tam_observant_sequencer() -> CardDefinition {
    let back = spell_back(
        "Deep Sight",
        cost(&[g(), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(4),
            },
            Effect::Draw {
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
        vec![Keyword::Deathtouch],
        back,
    );
    front.supertypes = vec![crate::card::Supertype::Legendary];
    front
}

// ── Kirol, History Buff // Pack a Punch ─────────────────────────────────────

/// Kirol, History Buff // Pack a Punch — {R}{W} // {1}{R}{W}.
///
/// Front: 2/3 Legendary Vampire Cleric with Lifelink. Back: sorcery —
/// target creature deals damage equal to its power to another target
/// creature. (Approximation: Pack a Punch deals 3 damage to target
/// creature; the printed fight pattern needs `Effect::Fight` with two
/// targets which collapses to single-target here.)
pub fn kirol_history_buff() -> CardDefinition {
    let back = spell_back(
        "Pack a Punch",
        cost(&[generic(1), r(), w()]),
        CardType::Sorcery,
        Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(3),
        },
    );
    let mut front = vanilla_front(
        "Kirol, History Buff",
        cost(&[r(), w()]),
        vec![CreatureType::Vampire, CreatureType::Cleric],
        2,
        3,
        vec![Keyword::Lifelink],
        back,
    );
    front.supertypes = vec![crate::card::Supertype::Legendary];
    front
}

// ── Abigale, Poet Laureate // Heroic Stanza ─────────────────────────────────

/// Abigale, Poet Laureate // Heroic Stanza — {1}{W}{B} // {1}{W/B}.
///
/// Front: 2/3 Legendary Bird Bard with Flying. Back: sorcery — Heroic
/// Stanza: target creature gets +2/+2 and gains lifelink until end of
/// turn. (The `{W/B}` hybrid pip is approximated as `{B}`.)
pub fn abigale_poet_laureate() -> CardDefinition {
    let back = spell_back(
        "Heroic Stanza",
        cost(&[generic(1), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            pump_target(2, 2),
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
        ]),
    );
    let mut front = vanilla_front(
        "Abigale, Poet Laureate",
        cost(&[generic(1), w(), b()]),
        vec![CreatureType::Bird, CreatureType::Bard],
        2,
        3,
        vec![Keyword::Flying],
        back,
    );
    front.supertypes = vec![crate::card::Supertype::Legendary];
    front
}

// ── Lluwen, Exchange Student // Pest Friend ─────────────────────────────────

/// Lluwen, Exchange Student // Pest Friend — {2}{B}{G} // {B/G}.
///
/// Front: 3/4 Legendary Elf Druid (vanilla body for Witherbloom finisher
/// curve). Back: sorcery — create a 1/1 black-and-green Pest creature
/// token with the printed "Whenever this token attacks, you gain 1 life"
/// rider. The hybrid `{B/G}` pip is approximated as `{B}` for cost-pay
/// (matches the Witherbloom convention used by Essenceknit Scholar's
/// `{B/G}` and Practiced Scrollsmith's `{R/W}` pips).
///
/// This MDFC closes out the Witherbloom (B/G) school in `STRIXHAVEN2.md`
/// — the only ⏳ row left for the school before this push.
pub fn lluwen_exchange_student() -> CardDefinition {
    use super::sorceries::pest_token;
    use crate::effect::ManaPayload;
    let _ = ManaPayload::Colors(vec![]); // suppress unused if not used elsewhere
    let back = spell_back(
        "Pest Friend",
        cost(&[b()]),
        CardType::Sorcery,
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: pest_token(),
        },
    );
    let mut front = vanilla_front(
        "Lluwen, Exchange Student",
        cost(&[generic(2), b(), g()]),
        vec![CreatureType::Elf, CreatureType::Druid],
        3,
        4,
        vec![],
        back,
    );
    front.supertypes = vec![crate::card::Supertype::Legendary];
    front
}

// ── Suppress unused-import warnings on `exile_target`/`counter_target_spell`.
#[allow(dead_code)]
fn _suppress_unused() {
    let _ = exile_target();
    let _ = counter_target_spell();
}
