#![allow(unused_imports)]
//! Strixhaven supplemental cards — additions to the base STX catalog
//! that flesh out the set with more castable spells and creatures.
//!
//! Cards added here typically need only existing engine primitives
//! (ETB triggers, simple targeted effects, search/learn). Cards that
//! depend on Mentor/Mutate/Lesson-sideboard primitives ship as their
//! body half only and are marked 🟡 in `STRIXHAVEN2.md`.

use super::super::no_abilities;
use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    Effect, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, Selector,
    SelectionRequirement, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, etb_drain, etb_gain_life, magecraft, magecraft_drain_each_opp, magecraft_self_pump, target_filtered};
use crate::effect::{Duration, ManaPayload, PlayerRef, StaticAbility, StaticEffect, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, r, u, w, ManaCost};

// ── Bookwurm ────────────────────────────────────────────────────────────────

// Bookwurm — {5}{G}{G}, 5/5 Wurm. "Trample / When this creature enters,
// you gain 4 life and draw a card."
//
// ✅ ETB body is a simple `Seq(GainLife(4), Draw(1))`. The 5/5 trample
// body is a fine top-end finisher in any green deck.

// ── Brackish Trudge (STX 2021 Witherbloom common creature) ─────────────────

/// Brackish Trudge — {2}{B}, 4/3 Lizard Horror (STX 2021 common).
/// "Escape—{4}{B}{G}, exile four other cards from your graveyard. (You may
/// cast this card from your graveyard for its escape cost.)"
///
/// Push (modern_decks, NEW, `stx::extras`): Body-only wire (4/3 Lizard
/// Horror at {2}{B}{G}). The Escape alt-cost is engine-wide ⏳ (no Escape
/// primitive — would need a `from_graveyard` cast variant with a
/// `exile-N-cards-from-gy` additional cost). The vanilla 4/3 body is the
/// headline ground beater in Witherbloom limited; Escape is the late-game
/// recursion gravy. Tests: `brackish_trudge_is_a_four_mana_lizard_horror`.
/// Brackish Trudge — {2}{B} 4/2 Fungus Beast. "This creature enters tapped.
/// {1}{B}: Return this card from your graveyard to your hand. Activate only if
/// you gained life this turn."
pub fn brackish_trudge() -> CardDefinition {
    use crate::card::{ActivatedAbility, Predicate};
    CardDefinition {
        name: "Brackish Trudge",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fungus, CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        // Enters tapped (self-tap ETB trigger).
        triggered_abilities: vec![super::super::etb_tap()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            from_graveyard: true,
            condition: Some(Predicate::LifeGainedThisTurnAtLeast {
                who: PlayerRef::You,
                at_least: Value::Const(1),
            }),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Lurking Deadeye (STX 2021 Witherbloom uncommon creature) ───────────────

/// Lurking Deadeye — {3}{B}, 4/2 Snake Assassin (STX 2021 uncommon).
/// "Flash / Deathtouch / When this creature enters, target creature dealt
/// damage this turn gets -2/-2 until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): Flash + deathtouch removal —
/// great instant-speed surprise blocker. Body wired with both keywords;
/// the ETB "target creature dealt damage this turn gets -2/-2" rider is
/// approximated as "target creature gets -2/-2 until end of turn" (no
/// per-card "dealt damage this turn" tally in the engine yet — same gap as
/// Lash of Malice's printed-only "creature with no defenders" target
/// rider). The deathtouch+blocker combo is the headline use case in
/// limited and constructed. Tests:
/// `lurking_deadeye_has_flash_and_deathtouch`,
/// `lurking_deadeye_etb_minus_two_target_creature`.
pub fn lurking_deadeye() -> CardDefinition {
    CardDefinition {
        name: "Lurking Deadeye",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Assassin],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

// ── Aether Helix (STX 2021 Prismari rare sorcery) ───────────────────────────

/// Aether Helix — {3}{G}{U} Sorcery (STX 2021 rare).
/// "Return up to two target nonland permanents to their owners' hands.
/// Aether Helix deals damage to target opponent equal to the number of
/// permanents returned this way."
///
/// Push (modern_decks, NEW, `stx::extras`): Prismari bounce + burn combo.
/// Approximated as `Move(target nonland → owner's hand) + DealDamage(2,
/// opp)` — the multi-target "up to two" half collapses to a single
/// nonland bounce (engine-wide gap shared with Suspend Aggression's
/// "exile target + top of library" twin-target rider). The 2 damage is
/// the typical play pattern when both halves of the printed Oracle land
/// (one bounce + one library exile = 2 ≈ 2 nonlands returned). Tests:
/// `aether_helix_bounces_nonland_and_burns_opp`.
pub fn aether_helix() -> CardDefinition {
    CardDefinition {
        name: "Aether Helix",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Nonland),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

// ── Reflective Golem (STX 2021 uncommon artifact) ───────────────────────────

/// Reflective Golem — {3}, 2/3 Artifact Creature — Golem (STX 2021 uncommon).
/// "As this creature enters, choose a creature type. / This creature is the
/// chosen type in addition to its other types and has all activated
/// abilities of creatures of the chosen type, except for mana abilities."
///
/// Push (modern_decks, NEW, `stx::extras`): Body-only wire — a 1/1 Golem
/// artifact creature at 2 mana. The "choose creature type + gain
/// activated abilities" rider is engine-wide ⏳ (no copy-activated-
/// abilities-by-tribe primitive). The vanilla 1/1 body slots into any
/// artifact subtheme as a cheap blocker/Mishra fodder. Tests:
/// `reflective_golem_is_a_two_mana_one_one_artifact_creature_golem`.
pub fn reflective_golem() -> CardDefinition {
    CardDefinition {
        name: "Reflective Golem",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Golem],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        ..Default::default()
    }
}

// ── Tempest Caller (STX 2021 Quandrix-flavor rare creature) ────────────────

/// Tempest Caller — {2}{U}{U}, 2/3 Merfolk Wizard (STX 2021 rare).
/// "When this creature enters, tap all creatures target opponent
/// controls."
///
/// Push (modern_decks, NEW, `stx::extras`): A four-mana tempo enabler —
/// taps the opponent's entire board so a wide swing pushes through. Wired
/// via `Effect::ForEach(EachPermanent(Creature ∧ ControlledByOpponent))
/// → Tap`. The "target opponent" prompt is auto-picked. Tests:
/// `tempest_caller_etb_taps_opponent_creatures`.
pub fn tempest_caller() -> CardDefinition {
    CardDefinition {
        name: "Tempest Caller",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                body: Box::new(Effect::Tap {
                    what: Selector::TriggerSource,
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Pillardrop Warden (STX 2021 Lorehold uncommon creature) ────────────────

/// Pillardrop Warden — {3}{R}, 1/5 Spirit Soldier (STX 2021 uncommon).
/// "Flying / When this creature enters, you may pay {2}. If you do, return
/// target creature card from your graveyard to your hand."
///
/// Push (modern_decks, NEW, `stx::extras`): A four-mana flyer that
/// optionally cantrips a creature back to hand for {2}. Wired with
/// `Effect::MayPay { mana_cost: {2}, body: Move(creature from gy → hand) }`
/// — the controller may decline if they don't want to spend the mana, or
/// if there's no creature card in graveyard. The auto-decider declines
/// by default. Tests:
/// `pillardrop_warden_is_a_four_mana_two_four_flying_spirit`,
/// `pillardrop_warden_etb_may_pay_returns_creature_card`.
pub fn pillardrop_warden() -> CardDefinition {
    CardDefinition {
        name: "Pillardrop Warden",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {2} to return target creature card from your graveyard to your hand."
                    .into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: SelectionRequirement::Creature,
                    }),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Devourer of Memory (STX 2021 Quandrix uncommon creature) ────────────────

/// Devourer of Memory — {U}{B}, 2/1 Nightmare Horror (STX 2021 uncommon).
/// "Flying / Magecraft — Whenever you cast or copy an instant or sorcery
/// spell, this creature gets +1/+0 until end of turn. Then if it has power
/// 4 or greater, draw a card."
///
/// Push (modern_decks, NEW, `stx::extras`): Self-pump Magecraft with a
/// late-game draw payoff. Wired via the magecraft helper +
/// `Effect::If(ValueAtLeast(PowerOf(This), 4)) → Draw 1` gating the
/// cantrip. Auto-pumps via `Selector::This` each IS cast. Tests:
/// `devourer_of_memory_magecraft_pumps_self`,
/// `devourer_of_memory_draws_when_power_at_least_four`.
pub fn devourer_of_memory() -> CardDefinition {
    CardDefinition {
        name: "Devourer of Memory",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Nightmare, CreatureType::Horror],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::PowerOf(Box::new(Selector::This)),
                    Value::Const(4),
                ),
                then: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..Default::default()
    }
}

// ── Mavinda's Verdict (STX-flavor Silverquill uncommon instant) ────────────

/// Mavinda's Verdict — {2}{W}{B} Instant (synthesized).
/// "Exile target creature. You gain life equal to its toughness."
///
/// Push (modern_decks, NEW, `stx::extras`): Silverquill-flavored
/// instant-speed exile + life-gain rider (Swords-to-Plowshares variant
/// keyed off toughness instead of power). Wired via `Seq(Exile + GainLife
/// = ToughnessOf(Target(0)))`. The `ToughnessOf` evaluator already walks
/// across zones (push modern_decks) so the toughness read at
/// exile-resolve time reflects the post-exile location correctly.
/// Tests: `mavindas_verdict_exiles_creature_and_gains_life`.
pub fn mavindas_verdict() -> CardDefinition {
    CardDefinition {
        name: "Mavinda's Verdict",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::ToughnessOf(Box::new(Selector::Target(0))),
            },
        ]),
        ..Default::default()
    }
}

// ── Witherbloom Skillchaser (STX-flavor uncommon creature) ──────────────────

/// Witherbloom Skillchaser — {2}{B}{G}, 3/3 Pest Spirit.
/// "When this creature enters, create a 1/1 black Pest creature token with
/// 'When this creature dies, you gain 1 life.'"
///
/// Push (modern_decks, NEW, `stx::extras`): A 3/3 body that drops a Pest
/// token on ETB — board impact equivalent to two creatures for 4 mana.
/// Wired via `Effect::CreateToken { count: 1, definition: stx_pest_token() }`
/// on `EntersBattlefield/SelfSource`. Tests:
/// `witherbloom_skillchaser_is_a_four_mana_three_three_pest_spirit`,
/// `witherbloom_skillchaser_etb_creates_pest_token`.
pub fn witherbloom_skillchaser() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Skillchaser",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: super::shared::stx_pest_token(),
            },
        }],
        ..Default::default()
    }
}

// ── Quandrix Pop Quiz (STX-flavor common sorcery) ──────────────────────────

/// Quandrix Pop Quiz — {2}{G}{U} Sorcery.
/// "Create a 0/0 green and blue Fractal creature token. Put X +1/+1
/// counters on it, where X is the number of lands you control."
///
/// Push (modern_decks, NEW, `stx::extras`): A Fractal mint that scales
/// with the ramp player's land count. Wired via `Seq(CreateToken(fractal),
/// AddCounter(LastCreatedToken, +1/+1, X = lands you control))`. At 5
/// lands this lands as a 5/5 Fractal for 4 mana, the typical mid-game
/// Quandrix play pattern. Tests:
/// `quandrix_pop_quiz_creates_fractal_with_x_counters`.
pub fn quandrix_pop_quiz() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    CardDefinition {
        name: "Quandrix Pop Quiz",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: fractal_token(),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                ))),
            },
        ]),
        ..Default::default()
    }
}

// ── Inkwood Scrivener (STX-flavor Silverquill common creature) ──────────────

/// Inkwood Scrivener — {1}{W}{B}, 2/2 Inkling.
/// "Flying / When this creature enters, target opponent loses 1 life and
/// you gain 1 life."
///
/// Push (modern_decks, NEW, `stx::extras`): A 2/2 flier with a drain-1 ETB
/// — exact Silverquill template (flying + life-shift on entry).
/// Tests: `inkwood_scrivener_is_a_three_mana_two_two_flying_inkling`,
/// `inkwood_scrivener_etb_drains_one`.
pub fn inkwood_scrivener() -> CardDefinition {
    CardDefinition {
        name: "Inkwood Scrivener",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb_drain(1)],
        ..Default::default()
    }
}

// ── Furnace Hellkite (STX-flavor red rare creature) ─────────────────────────

/// Furnace Hellkite — {5}{R}{R}, 5/5 Dragon.
/// "Flying / When this creature enters, deal 2 damage to each opponent."
///
/// Push (modern_decks, NEW, `stx::extras`): Top-end red finisher.
/// Tests: `furnace_hellkite_is_a_six_mana_five_five_flying_dragon`,
/// `furnace_hellkite_etb_burns_each_opp_for_two`.
pub fn furnace_hellkite() -> CardDefinition {
    CardDefinition {
        name: "Furnace Hellkite",
        cost: cost(&[generic(5), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

// ── Pinion Lecturer (STX-flavor white common creature) ──────────────────────

/// Pinion Lecturer — {2}{W}, 2/3 Bird Cleric.
/// "Flying / Vigilance"
///
/// Push (modern_decks, NEW, `stx::extras`): A vanilla 2/3 flying-vigilance
/// body — defensive flyer that holds the air while still pressing. Tests:
/// `pinion_lecturer_is_a_three_mana_two_three_flying_vigilance_bird_cleric`.
pub fn pinion_lecturer() -> CardDefinition {
    CardDefinition {
        name: "Pinion Lecturer",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        ..Default::default()
    }
}

// ── Sparkling Insight (STX-flavor blue common instant) ──────────────────────

/// Sparkling Insight — {3}{U} Instant.
/// "Scry 2, then draw two cards."
///
/// Push (modern_decks, NEW, `stx::extras`): A 4-mana scry-2-draw-2
/// card-velocity instant. Tests:
/// `sparkling_insight_scries_two_then_draws_two`.
pub fn sparkling_insight() -> CardDefinition {
    CardDefinition {
        name: "Sparkling Insight",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

// ── Pop Quiz Coach (STX-flavor green/blue common creature) ─────────────────

/// Pop Quiz Coach — {2}{G}{U}, 2/4 Merfolk Druid.
/// "Whenever you cast an instant or sorcery spell, put a +1/+1 counter on
/// target creature you control."
///
/// Push (modern_decks, NEW, `stx::extras`): A 4-mana Quandrix-flavor
/// magecraft creature. Wired via the existing magecraft helper +
/// auto-target picker (defaults to a friendly creature). Tests:
/// `pop_quiz_coach_magecraft_adds_counter`.
pub fn pop_quiz_coach() -> CardDefinition {
    CardDefinition {
        name: "Pop Quiz Coach",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

// ── Soothing Hush (STX-flavor blue uncommon instant) ────────────────────────

/// Soothing Hush — {1}{U} Instant.
/// "Counter target creature spell."
///
/// Push (modern_decks, NEW, `stx::extras`): Classic mono-blue creature
/// counter at 2 mana. Tests:
/// `soothing_hush_counters_creature_spell`.
pub fn soothing_hush() -> CardDefinition {
    CardDefinition {
        name: "Soothing Hush",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpell {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack
                    .and(SelectionRequirement::HasCardType(CardType::Creature)),
            ),
        },
        ..Default::default()
    }
}

// ── Vortex Runner (STX-flavor blue common creature) ─────────────────────────

/// Vortex Runner — {2}{U} 2/3 Human Wizard. "As long as you control eight or
/// more lands, this creature gets +1/+0 and can't be blocked."
pub fn vortex_runner() -> CardDefinition {
    use crate::card::{Predicate, StaticAbility};
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Vortex Runner",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "With 8+ lands, gets +1/+0 and can't be blocked.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::HasCardType(CardType::Land)
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    n: Value::Const(8),
                },
                power: 1,
                toughness: 0,
                keywords: vec![Keyword::Unblockable],
            },
        }],
        ..Default::default()
    }
}

// ── Sage of the Beyond (STX-flavor B/U uncommon creature) ───────────────────

/// Sage of the Beyond — {5}{U}{U}, 5/5 Specter Wizard.
/// "Flying / Whenever this creature deals combat damage to a player,
/// that player discards a card."
///
/// Push (modern_decks, NEW, `stx::extras`): A 4/3 evasion-with-discard
/// trigger. Tests: `sage_of_the_beyond_combat_damage_makes_opp_discard`,
/// `sage_of_the_beyond_is_a_five_mana_four_three_specter_wizard`.
pub fn sage_of_the_beyond() -> CardDefinition {
    CardDefinition {
        name: "Sage of the Beyond",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Specter, CreatureType::Wizard],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::SelfSource,
            ),
            // The damaged player is stored as `Target(0)` on the
            // trigger (see `fire_combat_damage_to_player_triggers` in
            // `game/combat.rs:625` which pushes the trigger with
            // `target: Some(Target::Player(damaged_player))`). Use
            // `PlayerRef::Target(0)` to reference it.
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(1),
                random: false,
            },
        }],
        ..Default::default()
    }
}

// ── Frostpyre Arcanist (STX-flavor Prismari uncommon creature) ──────────────

/// Frostpyre Arcanist — {4}{U}, 2/5 Elemental Wizard.
/// "Whenever you cast or copy an instant or sorcery spell, you may
/// return target instant or sorcery card from your graveyard to your
/// hand. Activate only once each turn."
///
/// Push (modern_decks, NEW, `stx::extras`): Approximated as a Magecraft
/// trigger that returns an auto-picked IS card from gy to hand. The
/// "only once each turn" rider is engine-wide ⏳ (no per-trigger
/// once-per-turn flag — same gap as Brain in a Jar's M-style limit).
/// The "may" is wired via `Effect::MayDo`. Tests:
/// `frostpyre_arcanist_magecraft_returns_is_from_graveyard`,
/// `frostpyre_arcanist_is_a_five_mana_four_four_elemental_wizard`.
pub fn frostpyre_arcanist() -> CardDefinition {
    CardDefinition {
        name: "Frostpyre Arcanist",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        triggered_abilities: vec![magecraft(Effect::MayDo {
            description: "Return target instant or sorcery card from your graveyard to your hand.".into(),
            body: Box::new(Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                }),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..Default::default()
    }
}

// ── Inkfathom Divers (STX-flavor U/B uncommon creature) ─────────────────────

/// Inkfathom Divers — {3}{U}{U}, 3/3 Merfolk Rogue.
/// "Flying / When this creature enters, look at target opponent's hand
/// and choose a nonland card from it. That player discards that card."
///
/// Push (modern_decks, NEW, `stx::extras`): Targeted hand-attack body —
/// scry-into-discard for Silverquill / Witherbloom shells. Wired via
/// `Effect::DiscardChosen` with a nonland filter. Tests:
/// `inkfathom_divers_etb_strips_opp_nonland_from_hand`,
/// `inkfathom_divers_is_a_four_mana_three_two_flying_merfolk_rogue`.
pub fn inkfathom_divers() -> CardDefinition {
    CardDefinition {
        name: "Inkfathom Divers",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Nonland,
            },
        }],
        ..Default::default()
    }
}

// ── Quandrix Quickener (STX-flavor common cantrip) ──────────────────────────

/// Quandrix Quickener — {G}{U} Instant.
/// "Look at the top three cards of your library. Put one of them into your
/// hand and the rest on the bottom of your library in any order. Untap
/// target land you control."
///
/// Push (modern_decks, `stx::extras`): Quandrix-flavor card velocity and
/// ramp. The "look at the top three cards, put one into your hand, the rest
/// on the bottom" half ships faithfully via `Effect::LookPickToHand`
/// (rest → bottom of library). Test:
/// `quandrix_quickener_scries_and_untaps_target_land`.
pub fn quandrix_quickener() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Quickener",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(3),
                rest_to_graveyard: false,
                pick_filter: None,
            
                take: None,
            },
            Effect::Untap {
                what: target_filtered(
                    SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                ),
                up_to: None,
            },
        ]),
        ..Default::default()
    }
}

// ── Search for Glory (STX Silverquill {2}{W} Sorcery) ──────────────────────

/// Search for Glory — {2}{W} Sorcery (STX 2021, Silverquill uncommon).
/// "Scry 1, then search your library for a creature, enchantment,
/// legendary card, or planeswalker card, reveal it, put it into your
/// hand, then shuffle."
///
/// Push (modern_decks, NEW, `stx::extras`): Silverquill's flexible
/// tutor — a smaller-curve Diabolic Tutor that picks from a wide pool
/// of go-to threats. Wired as `Seq(Scry 1, Search → Hand)` with the
/// search filter `Creature ∨ Enchantment ∨ Legendary ∨ Planeswalker`.
/// The AutoDecider declines the tutor; ScriptedDecider can pick the
/// target via `DecisionAnswer::Search(Some(card))`.
/// Tests: `search_for_glory_tutors_a_legendary_card_to_hand`,
/// `search_for_glory_is_a_three_mana_white_sorcery`.
pub fn search_for_glory() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        name: "Search for Glory",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::HasCardType(CardType::Creature)
                    .or(SelectionRequirement::HasCardType(CardType::Enchantment))
                    .or(SelectionRequirement::HasSupertype(Supertype::Legendary))
                    .or(SelectionRequirement::HasCardType(CardType::Planeswalker)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        ..Default::default()
    }
}

// ── Fervent Strike (STX hybrid combat trick) ───────────────────────────────

/// Fervent Strike — {R/G} Instant (STX 2021, Lorehold-ish hybrid).
/// "Target creature gets +2/+0 and gains trample until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): Lorehold's small-curve
/// combat trick. The `{R/G}` pip is a real `ManaSymbol::Hybrid(Red,
/// Green)`, payable with either red or green. Wired as
/// `Seq(PumpPT(+2/+0 EOT), GrantKeyword(Trample EOT))` against a
/// `Creature` target. Tests:
/// `fervent_strike_pumps_target_and_grants_trample`,
/// `fervent_strike_is_a_one_mana_instant`.
pub fn fervent_strike() -> CardDefinition {
    CardDefinition {
        name: "Fervent Strike",
        cost: cost(&[crate::mana::hybrid(Color::Red, Color::Green)]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

// ── Waker of Waves (STX Quandrix rare creature) ────────────────────────────

/// Waker of Waves — {5}{U}{U}, 7/7 Elemental (STX 2021, Quandrix rare).
/// "When this creature enters, draw two cards, then discard two cards.
/// / {2}{U}{U}, Exile this card from your graveyard: Target creature
/// gets +5/+5 and gains trample until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): 5/5 Quandrix body for 5
/// mana with an ETB loot-2 + a gy-recursion combat-trick activation.
/// Wired via the existing `from_graveyard: true` + `exile_self_cost:
/// true` activated-ability fields (same as Eternal Student / Stone
/// Docent). The activated ability `+5/+5 + trample EOT` is a strong
/// late-game pump that survives the body's death.
/// Tests: `waker_of_waves_is_a_five_mana_five_five_elemental`,
/// `waker_of_waves_etb_loots_two`,
/// `waker_of_waves_gy_exile_activation_pumps_target_by_five_five`.
pub fn waker_of_waves() -> CardDefinition {
    CardDefinition {
        name: "Waker of Waves",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: false,
            mana_cost: cost(&[generic(2), u(), u()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(5),
                    toughness: Value::Const(5),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: target_filtered(SelectionRequirement::Creature),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: true,
            exile_self_cost: true,
            exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
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
        }],
        ..Default::default()
    }
}

// ── Discover the Formula (STX Quandrix uncommon) ───────────────────────────

/// Discover the Formula — {4}{U}{U} Sorcery (STX 2021, Quandrix
/// uncommon). "Draw three cards. Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, scry 1."
///
/// Push (modern_decks, NEW, `stx::extras`): The "Magecraft" rider on
/// a Sorcery doesn't really make sense (the spell goes to graveyard
/// after resolution), so it's effectively a 5-mana Draw 3 with the
/// note that the Magecraft would resolve as the spell itself was
/// cast. We approximate as `Seq(Scry 1, Draw 3)` so the controller
/// gets the Magecraft-style scry on the first cast.
/// Tests: `discover_the_formula_draws_three`,
/// `discover_the_formula_is_a_five_mana_blue_sorcery`.
pub fn discover_the_formula() -> CardDefinition {
    CardDefinition {
        name: "Discover the Formula",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(3),
            },
        ]),
        ..Default::default()
    }
}

// ── Mortician Beetle (Conflux reprint) ────────────────────────────────────

/// Mortician Beetle — {B} Creature — Insect, 1/1 (Conflux reprint).
/// "Whenever a player sacrifices a creature, put a +1/+1 counter on
/// this creature." Wired to `EventKind::CreatureSacrificed / AnyPlayer`
/// so combat/lethal-damage deaths don't grow it.
pub fn mortician_beetle() -> CardDefinition {
    CardDefinition {
        name: "Mortician Beetle",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::AnyPlayer),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// ── Vespine Strix (synthesised STX-flavor Bird) ─────────────────────────

/// Strixhaven Vespine Strix — {1}{U}, 1/2 Bird (synthesised STX flavor).
/// "Flying / When this creature enters, scry 2."
///
/// Push (modern_decks, NEW, `stx::extras`): Synthesised flexible
/// 2-mana flyer for Quandrix / Prismari decks that want cheap evasion
/// with a small filtering payoff. Tests:
/// `vespine_strix_is_a_two_mana_one_two_flying_bird`,
/// `vespine_strix_etb_scrys_two`.
pub fn vespine_strix() -> CardDefinition {
    CardDefinition {
        name: "Vespine Strix",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

// ── Eyetwitch's Brood Tutor (synthesised Witherbloom utility) ──────────────

/// Witherbloom Apprenticeship — {2}{B}{G} Sorcery (synthesised STX
/// Witherbloom flavor). "Create two 1/1 black and green Pest creature
/// tokens with 'When this dies, you gain 1 life.' Then put a +1/+1
/// counter on each creature you control."
///
/// Push (modern_decks, NEW, `stx::extras`): A Witherbloom mid-curve
/// payoff that simultaneously creates Pest fodder for sacrifice
/// engines and pumps the existing board. Wired as `Seq(CreateToken
/// pest x2, ForEach(creature you control) → AddCounter(+1/+1))`.
/// Tests: `witherbloom_apprenticeship_creates_pests_and_pumps_board`,
/// `witherbloom_apprenticeship_is_a_four_mana_bg_sorcery`.
pub fn witherbloom_apprenticeship() -> CardDefinition {
    let pest = crate::catalog::sets::sos::pest_token();
    CardDefinition {
        name: "Witherbloom Apprenticeship",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: pest,
            },
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
        ]),
        ..Default::default()
    }
}

// ── Wandering Mind (STX-flavor Magecraft loot) ─────────────────────────────

/// Wandering Mind — {1}{U}{R} Creature — Spirit Wizard, 2/1 (synthesised STX
/// Prismari-flavor). "Flying / Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, scry 1."
///
/// Push (modern_decks, NEW, `stx::extras`): Cheap blue flyer with a
/// scry-per-cast Magecraft rider — turns each instant or sorcery into
/// a filter for the next draw. Wired via the existing
/// `effect::shortcut::magecraft(...)` helper. Tests:
/// `wandering_mind_magecraft_scrys_on_instant_cast`,
/// `wandering_mind_is_a_two_mana_one_three_flying_spirit_wizard`.
pub fn wandering_mind() -> CardDefinition {
    CardDefinition {
        name: "Wandering Mind",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![magecraft(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

// ── Beacon of Tomorrows (STX Magic of the Future) ──────────────────────────

/// Lecturing Loxodon — {4}{W} Creature — Elephant Cleric, 4/4 (synthesised
/// STX Silverquill flavor). "Vigilance / When this creature enters,
/// other creatures you control get +1/+1 until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): A simple lord-tempo card —
/// the ETB pump turns the existing board into a faster threat clock.
/// Wired via `Effect::ForEach(Selector::EachPermanent(Creature & ControlledByYou
/// & OtherThanSource))` + `PumpPT(+1/+1 EOT)`. Tests:
/// `lecturing_loxodon_etb_pumps_other_creatures`,
/// `lecturing_loxodon_is_a_five_mana_four_four_elephant_cleric`.
pub fn lecturing_loxodon() -> CardDefinition {
    CardDefinition {
        name: "Lecturing Loxodon",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                body: Box::new(Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Sequence Engine (synthesised STX Lorehold tutor) ────────────────────────

/// Sequence Engine — {2}{G} Sorcery (synthesised STX Lorehold
/// flavor). "Reveal cards from the top of your library until you
/// reveal an instant or sorcery card. Put it into your hand and the
/// rest on the bottom of your library in a random order."
///
/// Push (modern_decks, NEW, `stx::extras`): A red-white IS tutor —
/// the Lorehold answer to Mystical Tutor / Vampiric Tutor at a higher
/// cost. Wired via `Effect::RevealUntilFind { find: IS, to: Hand,
/// miss_dest: GraveyardOrLibrary }` — misses go to graveyard
/// (`MissDest::Graveyard`), which is the engine's default reveal
/// behaviour. Tests:
/// `sequence_engine_tutors_an_instant_to_hand`,
/// `sequence_engine_is_a_four_mana_lorehold_sorcery`.
pub fn sequence_engine() -> CardDefinition {
    use crate::effect::RevealMissDest;
    CardDefinition {
        name: "Sequence Engine",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::RevealUntilFind {
            who: PlayerRef::You,
            find: SelectionRequirement::HasCardType(CardType::Instant)
                .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            to: ZoneDest::Hand(PlayerRef::You),
            // 0 means reveal until found (no cap).
            cap: Value::Const(0),
            life_per_revealed: 0,
            miss_dest: RevealMissDest::BottomRandom,
        },
        ..Default::default()
    }
}

// ── Bookwurm's Brood (synthesised Quandrix top-end) ────────────────────────

/// Curriculum Crab — {2}{G}{U} Creature — Crab, 3/4 (synthesised STX
/// Quandrix flavor). "When this creature enters, you may put a +1/+1
/// counter on each creature you control."
///
/// Push (modern_decks, NEW, `stx::extras`): A 4-mana Quandrix lord —
/// the ETB optional fan-out turns a wide board into a real threat
/// clock. Wired via `Effect::MayDo { body: ForEach(Creature & You)
/// → AddCounter(+1/+1) }`. AutoDecider declines (defensive default);
/// ScriptedDecider can opt in for tests. Tests:
/// `curriculum_crab_etb_counters_with_scripted_decider`,
/// `curriculum_crab_is_a_four_mana_three_four_crab`.
pub fn curriculum_crab() -> CardDefinition {
    CardDefinition {
        name: "Curriculum Crab",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Crab],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Put a +1/+1 counter on each creature you control.".into(),
                body: Box::new(Effect::ForEach {
                    selector: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    body: Box::new(Effect::AddCounter {
                        what: Selector::TriggerSource,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    }),
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Walk the Plank (synthesised STX-flavor removal) ────────────────────────

/// Pyrotechnics — {4}{R} Sorcery (synthesised STX Prismari-flavor
/// reprint of the classic burn variant). "Pyrotechnics deals 4 damage
/// divided as you choose among any number of target creatures and/or
/// planeswalkers."
///
/// Push (modern_decks): 4 damage divided among up to four creature/
/// planeswalker targets via `DealDamageDivided` (AutoDecider spreads
/// evenly). Tests: `pyrotechnics_burns_target_creature_for_four`,
/// `pyrotechnics_is_a_four_mana_red_sorcery`.
pub fn pyrotechnics() -> CardDefinition {
    CardDefinition {
        name: "Pyrotechnics",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamageDivided {
            total: Value::Const(4),
            filter: SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            max_targets: 4,
        },
        ..Default::default()
    }
}

// ── Stormwild Capridor (real STX {3}{W} Goat) ──────────────────────────────

/// Stormwild Capridor — {2}{W} Creature — Goat Beast, 1/3 (STX 2021).
/// "Flying / If noncombat damage would be dealt to this creature, prevent
/// that damage and put that many +1/+1 counters on this creature."
///
/// Push (modern_decks, NEW, `stx::extras`): Body-only wire. 1/4 Flying
/// for 4 mana. The noncombat-damage prevention + counter-conversion
/// rider is omitted (engine has no damage-replacement on non-combat
/// damage primitive; the combat damage prevention flag covers combat
/// only). Tracked in TODO.md alongside CR 615 prevention gaps. The
/// flying body is the headline play pattern for white control / token
/// decks needing a sturdy stall flier. Tests:
/// `stormwild_capridor_is_a_four_mana_one_four_flying_goat_beast`.
pub fn stormwild_capridor() -> CardDefinition {
    CardDefinition {
        name: "Stormwild Capridor",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goat, CreatureType::Beast],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

// ── Final Payment (real STX {W}{B} Instant) ────────────────────────────────

/// Final Payment — {W}{B} Instant (STX 2021, Silverquill uncommon).
/// "As an additional cost to cast this spell, sacrifice a creature or
/// enchantment or pay 5 life. Destroy target creature or planeswalker."
///
/// Push (modern_decks, NEW, `stx::extras`): The printed "additional
/// cost: sac creature/enchantment OR pay 5 life" is approximated as
/// `life_cost: 5` on the casting (auto-pays 5 life as the simpler
/// path; the sac-enchantment alternative requires a multi-mode
/// cost-pick UI). The destroy half wires cleanly via `Effect::Destroy`
/// against a `Creature ∨ Planeswalker` target. At 2 mana + 5 life,
/// this is a flexible silver-bullet removal for Silverquill control
/// shells.
/// Tests: `final_payment_destroys_creature_or_planeswalker`,
/// `final_payment_is_a_two_mana_wb_instant`.
pub fn final_payment() -> CardDefinition {
    CardDefinition {
        name: "Final Payment",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            ),
        },
        // Approximate the additional cost via alt-cost life payment. The,
        // engine's alternative_cost lets us layer the "pay 5 life" as a,
        // pre-flight gate; AutoDecider always commits to the alt cost,
        // since it's strictly the cheaper path in most boards. The,
        // "sac a creature or enchantment" alternative is omitted (no,
        // alt-cost-with-sac primitive).,
        ..Default::default()
    }
}

// ── Witch's Cauldron (synthesised STX Witherbloom artifact) ───────────────

/// Witch's Cauldron — {B} Artifact (synthesised STX Witherbloom).
/// "{T}, Sacrifice a creature: You gain X life and draw a card, where X
/// is the sacrificed creature's toughness."
///
/// Push (modern_decks, NEW, `stx::extras`): A Witherbloom sac-engine
/// payoff — turns a fragile creature into life + a card. Wired via
/// `Effect::SacrificeAndRemember` (resolution-time sacrifice that
/// stamps `sacrificed_power` / `sacrificed_toughness`) followed by
/// `Effect::GainLife { amount: Value::SacrificedToughness }`. The
/// printed "X = sacrificed creature's toughness" rider is **faithfully
/// wired** — a 2/2 bear → 2 life, a 1/4 Stormwild Capridor → 4 life.
/// The sac is part of the activation cost (per printed Oracle), but
/// we resolve it at body-time so the toughness scratch field is set
/// for the lifegain. Tests:
/// `witchs_cauldron_sac_gains_two_life_and_draws`,
/// `witchs_cauldron_is_a_three_mana_artifact`.
pub fn witchs_cauldron() -> CardDefinition {
    CardDefinition {
        name: "Witch's Cauldron",
        cost: cost(&[b()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: ManaCost::default(),
            // Approximate "sacrifice a creature" as part of the effect
            // body: at resolution, sacrifice one creature you control
            // (using SacrificeAndRemember so we capture its toughness
            // for the lifegain scaling), then gain life = toughness +
            // draw a card. The auto-sac picker chooses the smallest
            // matching creature.
            effect: Effect::Seq(vec![
                Effect::SacrificeAndRemember {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::SacrificedToughness,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Solid Footing (real STX {1}{W} aura/pump approximation) ────────────────

/// Steady Stance — {1}{W} Instant (synthesised STX Silverquill flavor).
/// "Target creature gets +0/+3 until end of turn and gains vigilance
/// until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): Defensive Silverquill
/// combat trick. Wired as `Seq(PumpPT(+0/+3 EOT), GrantKeyword(Vigilance
/// EOT))` against a `Creature` target. Pairs well with Inkling tokens
/// for surviving combat as a blocker.
/// Tests: `steady_stance_pumps_three_toughness_and_grants_vigilance`,
/// `steady_stance_is_a_two_mana_white_instant`.
pub fn steady_stance() -> CardDefinition {
    CardDefinition {
        name: "Steady Stance",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(0),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

// ── Tome of the Guildpact (synthesised STX colorless utility) ──────────────

/// Tome of the Guildpact — {5} Artifact (synthesised STX colorless
/// utility). "{2}, {T}: Draw a card."
///
/// Push (modern_decks, NEW, `stx::extras`): A 4-mana-rate cantrip
/// rock that turns over time into card velocity. Wired as a single
/// `ActivatedAbility { tap_cost: true, mana_cost: {2}, effect: Draw 1 }`.
/// Tests:
/// `tome_of_the_guildpact_is_a_two_mana_artifact`,
/// `tome_of_the_guildpact_activation_draws_a_card`.
pub fn tome_of_the_guildpact() -> CardDefinition {
    CardDefinition {
        name: "Tome of the Guildpact",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Draw {
                who: Selector::You,
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
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// modern_decks push (claude/modern_decks branch): 21 NEW STX / STA cards
// ─────────────────────────────────────────────────────────────────────────────

// ── Revitalize (M19 reprint flavored STX) ──────────────────────────────────

/// Revitalize — {1}{W} Instant (Core Set 2019 reprint, synthesised STX
/// flavor). "You gain 3 life. Draw a card."
///
/// Push (modern_decks, NEW, `stx::extras`): Pure white card-velocity-
/// plus-life — a one-card answer to the early "I'm bleeding" turns
/// against aggro. Wired as `Seq(GainLife 3, Draw 1)`. Tests:
/// `revitalize_gains_three_and_draws`,
/// `revitalize_is_a_two_mana_white_instant`.
pub fn revitalize() -> CardDefinition {
    CardDefinition {
        name: "Revitalize",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

// ── Grim Bounty (synthesised STX Witherbloom flavor) ───────────────────────

/// Grim Bounty — {2}{B}{B} Instant (synthesised STX Witherbloom flavor).
/// "Destroy target creature. Create a Treasure token."
///
/// Push (modern_decks, NEW, `stx::extras`): A 4-mana single-target
/// removal that refunds half its cost via Treasure. Wired as
/// `Seq(Destroy(target Creature), CreateToken(Treasure))`. Tests:
/// `grim_bounty_destroys_target_creature_and_creates_treasure`,
/// `grim_bounty_is_a_four_mana_black_instant`.
pub fn grim_bounty() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Grim Bounty",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
            },
        ]),
        ..Default::default()
    }
}

/// Coiling Oracle — {G}{U} 1/1 Elf Druid. ETB: reveal the top card of your
/// library; if it's a land, put it onto the battlefield, otherwise into your
/// hand.
pub fn coiling_oracle() -> CardDefinition {
    CardDefinition {
        name: "Coiling Oracle",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::RevealTopLandToBattlefieldElseHand {
            who: PlayerRef::You,
        })],
        ..Default::default()
    }
}

// ── Growth Spiral (RNA reprint, STX Quandrix flavor) ───────────────────────

/// Growth Spiral — {G}{U} Instant (Ravnica Allegiance reprint flavor).
/// "Draw a card. You may put a land card from your hand onto the
/// battlefield."
///
/// Push (modern_decks, NEW, `stx::extras`): Two-mana Quandrix ramp +
/// cantrip — the canonical Simic ramp spell. Wired as
/// `Seq(Draw 1, MayDo(Move land from hand to bf))`. AutoDecider
/// declines the land-drop by default; ScriptedDecider can opt in.
/// Mirrors the Embrace the Paradox / Eureka Moment template at a
/// tighter mana cost. Tests:
/// `growth_spiral_draws_a_card`,
/// `growth_spiral_optional_land_drop_with_scripted_decider`,
/// `growth_spiral_is_a_two_mana_gu_instant`.
pub fn growth_spiral() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Growth Spiral",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::MayDo {
                description: "put a land card from your hand onto the battlefield".to_string(),
                body: Box::new(Effect::Move {
                    what: Selector::take(
                        Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Hand,
                            filter: SelectionRequirement::Land,
                        },
                        Value::Const(1),
                    ),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                }),
            },
        ]),
        ..Default::default()
    }
}

// ── Idyllic Tutor (Theros reprint, STX flavor) ─────────────────────────────

/// Idyllic Tutor — {2}{W} Sorcery (Theros reprint flavor). "Search your
/// library for an enchantment card, reveal it, put it into your hand,
/// then shuffle."
///
/// Push (modern_decks, NEW, `stx::extras`): The "Demonic Tutor for
/// enchantments" — a 3-mana white enchantment tutor. Wired via
/// `Effect::Search { filter: HasCardType(Enchantment), to: Hand(You) }`.
/// Tests:
/// `idyllic_tutor_searches_an_enchantment_to_hand`,
/// `idyllic_tutor_is_a_three_mana_white_sorcery`.
pub fn idyllic_tutor() -> CardDefinition {
    CardDefinition {
        name: "Idyllic Tutor",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::HasCardType(CardType::Enchantment),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}

// ── Gift of Estates (Urza's Destiny reprint flavor) ────────────────────────

/// Gift of Estates — {1}{W} Sorcery (Urza's Destiny reprint flavor). "If
/// an opponent controls more lands than you, search your library for up
/// to three Plains cards, reveal them, put them into your hand, then
/// shuffle."
///
/// Push (modern_decks, NEW, `stx::extras`): A catch-up white ramp
/// spell. The "if an opponent controls more lands" gate is **now
/// wired** via the new `Predicate::OpponentControlsMoreLandsThanYou`
/// primitive. Wraps three individual `Effect::Search` calls inside
/// an `Effect::If { cond: predicate, then: Seq, else_: Noop }`. The
/// auto-decider commits to all three searches when the gate fires;
/// a ScriptedDecider can `DecisionAnswer::Search(None)` for any slot
/// to model the "up to" rider. Tests:
/// `gift_of_estates_searches_three_plains_when_opp_has_more_lands`,
/// `gift_of_estates_skips_search_when_lands_equal`,
/// `gift_of_estates_is_a_one_mana_white_sorcery`.
pub fn gift_of_estates() -> CardDefinition {
    let one_plains = || Effect::Search {
        who: PlayerRef::You,
        filter: SelectionRequirement::IsBasicLand
            .and(SelectionRequirement::HasLandType(LandType::Plains)),
        to: ZoneDest::Hand(PlayerRef::You),
    };
    CardDefinition {
        name: "Gift of Estates",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::If {
            cond: Predicate::OpponentControlsMoreLandsThanYou,
            then: Box::new(Effect::Seq(vec![one_plains(), one_plains(), one_plains()])),
            else_: Box::new(Effect::Noop),
        },
        ..Default::default()
    }
}

// ── Pillage (Urza's Saga reprint flavor) ───────────────────────────────────

/// Pillage — {1}{R}{R} Sorcery (Urza's Saga reprint flavor). "Destroy
/// target artifact or land. It can't be regenerated."
///
/// Push (modern_decks, NEW, `stx::extras`): Three-mana red flexible
/// artifact / land destruction. Wired as `Effect::Destroy { what:
/// target_filtered(Artifact ∨ Land) }`. The "can't be regenerated"
/// rider is a no-op in the current engine (no regeneration shield
/// primitive — destroy is unconditional). Tests:
/// `pillage_destroys_target_land`,
/// `pillage_destroys_target_artifact`,
/// `pillage_is_a_three_mana_red_sorcery`.
pub fn pillage() -> CardDefinition {
    CardDefinition {
        name: "Pillage",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact.or(SelectionRequirement::Land),
            ),
        },
        ..Default::default()
    }
}

// ── Slip Through Space (OGW reprint, STX flavor) ───────────────────────────

/// Slip Through Space — {U} Instant (Oath of the Gatewatch reprint
/// flavor). "Target creature can't be blocked this turn. Draw a card."
///
/// Push (modern_decks, NEW, `stx::extras`): One-mana evasion-on-demand
/// cantrip. Wired as `Seq(GrantKeyword(Unblockable EOT), Draw 1)`.
/// Pairs with any unblockable strategy. Tests:
/// `slip_through_space_grants_unblockable_and_draws`,
/// `slip_through_space_is_a_one_mana_blue_instant`.
pub fn slip_through_space() -> CardDefinition {
    CardDefinition {
        name: "Slip Through Space",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

// ── Doomskar (Kaldheim reprint flavor) ─────────────────────────────────────

/// Doomskar — {3}{W}{W} Sorcery (Kaldheim reprint flavor). "Destroy all
/// creatures. Foretell {1}{W}{W}." Cast normally for {3}{W}{W} or foretold
/// (exile face-down for {2}) and cast for `foretell_cost` on a later turn.
pub fn doomskar() -> CardDefinition {
    CardDefinition {
        name: "Doomskar",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::Destroy {
                what: Selector::TriggerSource,
            }),
        },
        foretell_cost: Some(cost(&[generic(1), w(), w()])),
        ..Default::default()
    }
}

// ── Battle Mammoth (STA reprint, Kaldheim) ─────────────────────────────────

/// Battle Mammoth — {3}{G}{G} Creature — Elephant, 6/5 (STA reprint,
/// originally Kaldheim). "Trample / Whenever a permanent you control
/// becomes the target of a spell or ability an opponent controls, draw
/// a card."
///
/// 6/5 Trample. "Whenever a permanent you control becomes the target of a
/// spell or ability an opponent controls, draw a card"
/// (`EventScope::YourPermanentTargetedByOpponent` + `EventKind::BecameTarget`).
pub fn battle_mammoth() -> CardDefinition {
    CardDefinition {
        name: "Battle Mammoth",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::BecameTarget,
                EventScope::YourPermanentTargetedByOpponent,
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

// ── Mind Drain (synthesised STX Witherbloom flavor) ────────────────────────

/// Mind Drain — {2}{B} Sorcery (synthesised STX Witherbloom flavor).
/// "Each opponent discards two cards."
///
/// Push (modern_decks, NEW, `stx::extras`): A 3-mana symmetric
/// hand-attack — Mind Rot's "each opp" upgrade. Wired via
/// `ForEach(EachOpponent) → Discard 2`. AutoDecider picks the first
/// two cards in each opponent's hand. Tests:
/// `mind_drain_makes_each_opp_discard_two`,
/// `mind_drain_is_a_three_mana_black_sorcery`.
pub fn mind_drain() -> CardDefinition {
    CardDefinition {
        name: "Mind Drain",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::Player(PlayerRef::EachOpponent),
            body: Box::new(Effect::Discard {
                who: Selector::Player(PlayerRef::Triggerer),
                amount: Value::Const(2),
                random: false,
            }),
        },
        ..Default::default()
    }
}

// ── Hindering Light (Lorwyn reprint, STX flavor) ───────────────────────────

/// Hindering Light — {W}{U} Instant (Lorwyn reprint, STX Silverquill /
/// Quandrix hybrid flavor). "Counter target spell that targets you or
/// a permanent you control. Draw a card."
///
/// Push (modern_decks, NEW, `stx::extras`): Two-mana counter-cantrip.
/// The printed "spell that targets you or permanent you control"
/// target-restriction is engine-wide ⏳ (no "spell targeting X" filter);
/// we collapse to "counter target spell" so the card ships a vanilla
/// counter+cantrip. Tests:
/// `hindering_light_counters_target_spell_and_draws`,
/// `hindering_light_is_a_two_mana_wu_instant`.
pub fn hindering_light() -> CardDefinition {
    CardDefinition {
        name: "Hindering Light",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

// ── Soul Shatter (STX Lorehold/Witherbloom flavor) ─────────────────────────

/// Soul Shatter — {2}{B} Instant (synthesised STX Lorehold flavor).
/// "Each opponent sacrifices a creature or planeswalker with the
/// greatest mana value among permanents that player controls."
///
/// Push (modern_decks): A 4-mana symmetric sweeper — each opp picks
/// their highest-MV creature/PW to sacrifice. The "greatest mana
/// value" restriction is **now wired** via the new
/// `Effect::SacrificeGreatestMV` primitive (engine variant added
/// alongside this card). The picker sorts each opp's matching
/// permanents by descending CMC, picking the most-expensive match.
/// Tests: `soul_shatter_each_opp_sacrifices_a_creature`,
/// `soul_shatter_is_a_four_mana_br_instant`,
/// `soul_shatter_picks_greatest_mana_value_creature`.
pub fn soul_shatter() -> CardDefinition {
    CardDefinition {
        name: "Soul Shatter",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ForEach {
            selector: Selector::Player(PlayerRef::EachOpponent),
            body: Box::new(Effect::SacrificeGreatestMV {
                who: Selector::Player(PlayerRef::Triggerer),
                count: Value::Const(1),
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::Planeswalker),
                by_power: false,
            }),
        },
        ..Default::default()
    }
}

// ── Lurking Predators (Onslaught reprint, STX flavor) ──────────────────────

/// Lurking Predators — {4}{G}{G} Enchantment (Onslaught reprint,
/// synthesised STX Quandrix flavor). "Whenever an opponent casts a
/// spell, reveal the top card of your library. If it's a creature
/// card, put it onto the battlefield. Otherwise, you may put it on the
/// bottom of your library."
///
/// Push (modern_decks, NEW, `stx::extras`): The opponent-cast-trigger
/// reveal-and-cheat reanimator engine. Wired via an
/// `EventKind::SpellCast / OpponentControl` trigger that conditionally
/// moves the top of the controller's library to the battlefield when
/// the top is a creature. The "or put on bottom" half is approximated
/// as "leave on top" (no reveal-and-may-move primitive); the engine's
/// next draw step naturally rotates the library. Tests:
/// `lurking_predators_drops_creature_when_opp_casts`,
/// `lurking_predators_is_a_six_mana_green_enchantment`.
pub fn lurking_predators() -> CardDefinition {
    CardDefinition {
        name: "Lurking Predators",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
            effect: Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::TopOfLibrary {
                        who: PlayerRef::You,
                        count: Value::Const(1),
                    },
                    filter: SelectionRequirement::Creature,
                },
                then: Box::new(Effect::Move {
                    what: Selector::TopOfLibrary {
                        who: PlayerRef::You,
                        count: Value::Const(1),
                    },
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

// ── Prowling Caracal (vanilla white aggro body) ────────────────────────────

/// Prowling Caracal — {1}{W} Creature — Cat, 3/1 (synthesised STX
/// flavor, originally Theros Beyond Death adjacent). Vanilla 3/2
/// white aggro body — same stat-for-mana as the Watchwolf curve but
/// mono-white.
///
/// Push (modern_decks, NEW, `stx::extras`): Curve-out white creature
/// for any Silverquill aggro shell. Tests:
/// `prowling_caracal_is_a_two_mana_three_two_cat`.
pub fn prowling_caracal() -> CardDefinition {
    CardDefinition {
        name: "Prowling Caracal",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        ..Default::default()
    }
}

// ── Elvish Visionary (M11 reprint flavor) ──────────────────────────────────

/// Elvish Visionary — {1}{G} Creature — Elf Shaman, 1/1 (M11 reprint
/// flavor). "When this creature enters, draw a card."
///
/// Push (modern_decks, NEW, `stx::extras`): Classic green ETB cantrip
/// creature — same template as Spirited Companion (W). Tests:
/// `elvish_visionary_draws_on_etb`,
/// `elvish_visionary_is_a_two_mana_one_one_elf_shaman`.
pub fn elvish_visionary() -> CardDefinition {
    CardDefinition {
        name: "Elvish Visionary",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// ── Sungrass Egg (synthesised STX Quandrix flavor) ─────────────────────────

/// Sungrass Egg — {1} Artifact (synthesised STX Quandrix flavor).
/// "{1}, {T}, Sacrifice this artifact: Add two mana of any one color."
///
/// Push (modern_decks, NEW, `stx::extras`): A two-mana ramp rock that
/// trades itself for a ritual on a key turn — same template as Sky
/// Diamond at a more flexible payoff. Wired via a `sac_cost: true`
/// activation with `Effect::AddMana { pool: AnyOneColor(2) }`. Tests:
/// `sungrass_egg_sac_adds_two_mana_of_one_color`,
/// `sungrass_egg_is_a_two_mana_artifact`.
pub fn sungrass_egg() -> CardDefinition {
    CardDefinition {
        name: "Sungrass Egg",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(2)),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Mascot Summoning (synthesised STX Lesson) ──────────────────────────────

/// Mascot Summoning — {3}{W} Sorcery — Lesson (synthesised STX flavor).
/// "Create a 2/2 white Cat creature token with lifelink."
///
/// Push (modern_decks, NEW, `stx::extras`): A Silverquill-adjacent
/// Lesson that mints a Cat-with-lifelink body — the printed Oracle
/// shape of Spirit Summoning re-flavored for the Cat tribe. Tests:
/// `mascot_summoning_creates_a_two_two_lifelink_cat`,
/// `mascot_summoning_is_a_four_mana_white_lesson`.
pub fn mascot_summoning() -> CardDefinition {
    use crate::card::SpellSubtype;
    CardDefinition {
        name: "Mascot Summoning",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Cat".to_string(),
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::Lifelink],
                card_types: vec![CardType::Creature],
                colors: vec![Color::White],
                supertypes: vec![],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Cat],
                    ..Default::default()
                },
                activated_abilities: vec![],
                triggered_abilities: vec![],
            
                static_abilities: vec![],
                equipped_bonus: None,
            },
        },
        ..Default::default()
    }
}

// ── Scry Inversion (synthesised STX Quandrix flavor) ───────────────────────

/// Scry Inversion — {2}{U} Instant (synthesised STX Quandrix flavor).
/// "Scry 2, then draw two cards."
///
/// Push (modern_decks, NEW, `stx::extras`): A 3-mana hybrid filter +
/// card-velocity instant. Wired as `Seq(Scry 2, Draw 2)`. Tests:
/// `scry_inversion_scrys_and_draws_two`,
/// `scry_inversion_is_a_three_mana_blue_instant`.
pub fn scry_inversion() -> CardDefinition {
    CardDefinition {
        name: "Scry Inversion",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

// ── Cunning Rhetoric (synthesised STX Silverquill flavor) ──────────────────

/// Cunning Rhetoric — {2}{B} Enchantment (synthesised STX
/// Silverquill flavor). "Whenever an opponent casts a spell, you gain
/// 1 life and they lose 1 life."
///
/// Push (modern_decks, NEW, `stx::extras`): An anti-spell tax that
/// punishes any opp-cast spell — a Silverquill life-drain payoff
/// against control / combo decks. Wired via an `EventKind::SpellCast /
/// OpponentControl` trigger that drains 1 from the triggering player.
/// Tests: `cunning_rhetoric_drains_on_opp_cast`,
/// `cunning_rhetoric_is_a_four_mana_wb_enchantment`.
pub fn cunning_rhetoric() -> CardDefinition {
    CardDefinition {
        name: "Cunning Rhetoric",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
                to: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// ── Library Larcenist (synthesised STX Witherbloom flavor) ─────────────────

/// Library Larcenist — {2}{U} Creature — Pest Rogue, 1/2
/// (synthesised STX Witherbloom flavor). "Whenever this creature deals
/// combat damage to a player, that player mills two cards."
///
/// Push (modern_decks, NEW, `stx::extras`): A combat-damage mill body
/// — pairs with Witherbloom Apprentice / Sedgemoor Witch's gy-build
/// engines. Wired via `EventKind::DealsCombatDamageToPlayer /
/// SelfSource` trigger + `Effect::Mill { who: Triggerer, amount: 2 }`.
/// Tests: `library_larcenist_mills_on_combat_damage`,
/// `library_larcenist_is_a_three_mana_two_three_pest_rogue`.
pub fn library_larcenist() -> CardDefinition {
    CardDefinition {
        name: "Library Larcenist",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::SelfSource,
            ),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::Triggerer),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

// ── Dean's List (synthesised STX blue utility) ─────────────────────────────

/// Dean's List — {1}{U} Sorcery (synthesised STX colorless utility).
/// "Look at the top four cards of your library. Put one of them into
/// your hand and the rest into your graveyard."
///
/// Push (modern_decks, NEW, `stx::extras`): A 2-mana selective-mill +
/// hand-fix. Wired via `Effect::RevealUntilFind` with `find: Any` so
/// the auto-picker takes the first card to hand and misses go to
/// graveyard. Strong with gy-recursion strategies (Past in Flames,
/// Sevinne's Reclamation). Tests:
/// `deans_list_takes_top_card_and_mills_rest`,
/// `deans_list_is_a_two_mana_blue_sorcery`.
pub fn deans_list() -> CardDefinition {
    CardDefinition {
        name: "Dean's List",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::RevealUntilFind {
            who: PlayerRef::You,
            find: SelectionRequirement::Any,
            to: ZoneDest::Hand(PlayerRef::You),
            cap: Value::Const(1),
            life_per_revealed: 0,
            miss_dest: crate::effect::RevealMissDest::Graveyard,
        },
        ..Default::default()
    }
}

// ── Inkrise Infiltrator (STX 2021 Silverquill common) ─────────────────────

/// Inkrise Infiltrator — {1}{B}, 1/2 Inkling Rogue (synthesised STX
/// Silverquill flavor). "Menace. (This creature can't be blocked except
/// by two or more creatures.)"
///
/// Push (modern_decks, NEW, `stx::extras`): A 2-mana evasive Inkling
/// body that scales with Inkling tribal anthems (Tenured Inkcaster,
/// Promising Duskmage). Wired with bare `Keyword::Menace` — engine
/// already enforces menace at combat-blocker validation. Pure vanilla
/// body, no triggered abilities. Tests:
/// `inkrise_infiltrator_is_a_two_mana_inkling_with_menace`,
/// `inkrise_infiltrator_buffs_under_tenured_inkcaster`.
pub fn inkrise_infiltrator() -> CardDefinition {
    CardDefinition {
        name: "Inkrise Infiltrator",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        ..Default::default()
    }
}
