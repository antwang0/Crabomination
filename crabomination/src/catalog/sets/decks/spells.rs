//! Spells and enchantments for the BRG and Goryo's demo decks.
//!
//! Several spells here ship as stubs (`Effect::Noop`) where their real
//! behavior needs engine features that don't yet exist (alternative pitch
//! costs, pact deferred upkeep payments, exile-at-end-of-turn replacements,
//! rebound, convoke / converge, etc.). Each carries a doc-comment marking
//! what's missing; promote as engine features land.

use super::super::no_abilities;
use crate::card::{
    AlternativeCost, CardDefinition, CardType, Effect, Predicate, SelectionRequirement, Subtypes,
};
use crate::effect::shortcut::{counter_target_spell, target_filtered};
use crate::effect::{DelayedTriggerKind, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, generic, r, u, w, x};

// ── BRG main-deck spells ─────────────────────────────────────────────────────

/// Pact of Negation — {0} Instant. Counter target spell. "At the beginning of
/// your next upkeep, pay {3}{U}{U}. If you don't, you lose the game."
pub fn pact_of_negation() -> CardDefinition {
    use crate::mana::{ManaCost, u as u_mana};
    CardDefinition {
        name: "Pact of Negation",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            counter_target_spell(),
            Effect::DelayUntil {
                kind: DelayedTriggerKind::YourNextUpkeep,
                body: Box::new(Effect::PayOrLoseGame {
                    mana_cost: cost(&[generic(3), u_mana(), u_mana()]),
                    life_cost: 0,
                }),
                capture: None,
            },
        ]),
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

/// Plunge into Darkness — {1}{B} Instant. Modal — choose one:
///   • Sacrifice any number of creatures. You gain 3 life for each one.
///   • Pay any amount of life. Look at that many cards from the top of your
///     library, then put one into your hand and the rest on the bottom.
///
/// Engine model: `ChooseMode` between two simplified branches.
/// * **Mode 0** — sacrifice one creature you control + gain 3 life.
/// * **Mode 1** — pay 4 life + Search-your-library-into-hand. The full
///   Oracle is "pay X life, look at the top X, take one, bottom the rest";
///   we collapse that to "pay 4 (a typical X), tutor any card directly"
///   reusing the same Search primitive Spoils of the Vault rides on. The
///   tutored card is chosen via the standard `SearchLibrary` decision.
///
/// AutoDecider picks mode 0 (sac for life), which matches the typical play.
pub fn plunge_into_darkness() -> CardDefinition {
    CardDefinition {
        name: "Plunge into Darkness",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: Sacrifice a creature, gain 3 life. (Simplified from
            // "any number of creatures" — currently sacrifices one.)
            Effect::Seq(vec![
                Effect::SacrificeAndRemember {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(3),
                },
            ]),
            // Mode 1: pay 4 life, then tutor any card from your library.
            // Approximation of "pay X life, look at top X, take one,
            // bottom the rest": picks 4 as a representative X.
            Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::You,
                    amount: Value::Const(4),
                },
                Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Any,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ]),
        ]),
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

/// Serum Powder — {3} Artifact. {T}: Add {1}. "Any time you could mulligan
/// and Serum Powder is in your hand, you may exile all the cards from your
/// hand, then draw that many cards."
///
/// Wired: `opening_hand: Some(MulliganHelper)` flags this card as a
/// mulligan-time helper. During the mulligan window, the UI/decider can
/// answer with `DecisionAnswer::SerumPowder(card_id)` to exile the entire
/// hand and draw a fresh seven without bumping the London-mulligan ladder
/// (so multiple powders can stack safely). Outside of mulligans the card
/// is a vanilla {3} mana rock for {1}.
pub fn serum_powder() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::{ManaPayload, OpeningHandEffect};
    use crate::mana::ManaCost;
    CardDefinition {
        name: "Serum Powder",
        cost: cost(&[generic(3)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::Const(1)),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: Some(OpeningHandEffect::MulliganHelper),
    }
}

/// Spoils of the Vault — {B} Instant. Name a card. Reveal cards from the top
/// of your library until you reveal the named card or 10 different cards.
/// Put the named card into your hand. You lose 1 life for each card revealed.
///
/// Wired to `Effect::RevealUntilFind` with `find: Any` (no name primitive
/// yet — the first card off the top wins), `cap: 10`, and `life_per_revealed:
/// 1`. The engine mills every miss into the graveyard and deducts 1 life
/// per card revealed (matching the Oracle's variable life cost). With
/// `find: Any` only one card is ever revealed in practice, so the life
/// cost approximates 1 life — much cheaper than a "named tutor". A future
/// naming primitive can swap `Any` for `HasName(...)` to restore the full
/// reveal-until-named-card semantics.
pub fn spoils_of_the_vault() -> CardDefinition {
    CardDefinition {
        name: "Spoils of the Vault",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::RevealUntilFind {
            who: PlayerRef::You,
            find: SelectionRequirement::Any,
            to: ZoneDest::Hand(PlayerRef::You),
            cap: Value::Const(10),
            life_per_revealed: 1,
        },
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

/// Summoner's Pact — {0} Instant. Search your library for a green creature,
/// reveal, put into hand, shuffle. "At the beginning of your next upkeep,
/// pay {2}{G}{G}. If you don't, you lose the game."
pub fn summoners_pact() -> CardDefinition {
    use crate::mana::{ManaCost, g as g_mana};
    CardDefinition {
        name: "Summoner's Pact",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::HasColor(crate::mana::Color::Green)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::YourNextUpkeep,
                body: Box::new(Effect::PayOrLoseGame {
                    mana_cost: cost(&[generic(2), g_mana(), g_mana()]),
                    life_cost: 0,
                }),
                capture: None,
            },
        ]),
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

/// Thud — {R} Sorcery. As an additional cost, sacrifice a creature. Thud
/// deals damage equal to the sacrificed creature's power to any target.
///
/// The "additional cost" sacrifice is modelled as the first step of the
/// effect tree (rather than at cast time): on resolution, Thud picks a
/// creature its controller has and sacrifices it, recording the sacrificed
/// power so the subsequent `DealDamage` can read it. With `AutoDecider`
/// the engine auto-picks the first eligible creature; a future UI can
/// surface the choice via a dedicated decision.
pub fn thud() -> CardDefinition {
    CardDefinition {
        name: "Thud",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::SacrificeAndRemember {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
            },
            Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::SacrificedPower,
            },
        ]),
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

// ── BRG sideboard spells ─────────────────────────────────────────────────────

/// Inquisition of Kozilek — {B} Sorcery. Target opponent reveals their hand;
/// you choose a nonland card with mana value 3 or less; they discard it.
///
/// Targeting is approximated as `EachOpponent` (correct in 2-player). The
/// caster auto-picks the first nonland card with mana value ≤ 3.
pub fn inquisition_of_kozilek() -> CardDefinition {
    CardDefinition {
        name: "Inquisition of Kozilek",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DiscardChosen {
            from: Selector::Player(PlayerRef::EachOpponent),
            count: Value::Const(1),
            filter: SelectionRequirement::Nonland
                .and(SelectionRequirement::ManaValueAtMost(3)),
        },
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

/// Leyline of Sanctity — {2}{W}{W} Enchantment. "If Leyline of Sanctity is in
/// your opening hand, you may begin the game with it on the battlefield. You
/// have hexproof."
///
/// Wired with two pieces:
///   * `opening_hand: Some(StartInPlay)` — `apply_opening_hand_effects` moves
///     the card from hand to its owner's battlefield post-mulligan.
///   * `StaticAbility(ControllerHasHexproof)` — `check_target_legality`
///     refuses any opponent-controlled targeting that resolves to the
///     leyline's controller as a `Target::Player(_)`.
pub fn leyline_of_sanctity() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::{OpeningHandEffect, StaticEffect};
    CardDefinition {
        name: "Leyline of Sanctity",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "You have hexproof.",
            effect: StaticEffect::ControllerHasHexproof,
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: Some(OpeningHandEffect::StartInPlay {
            tapped: false,
            extra: Effect::Noop,
        }),
    }
}

// ── Goryo's main-deck spells ────────────────────────────────────────────────

/// Ephemerate — {W} Instant. Exile target creature you control, then return
/// it to the battlefield under its owner's control. **Rebound**: cast from
/// hand → exile on resolution, schedule a "may cast from exile next upkeep"
/// trigger that re-runs the flicker effect with a fresh auto-target.
///
/// Modeled as `Seq([Exile target creature you control, Move target back to
/// the battlefield])`. The same target slot is re-resolved on the second
/// step — `Selector::Target(0)` finds the card in exile (the engine
/// `evaluate_requirement_static` falls through to graveyard / exile so the
/// target stays bound), and `move_card_to` walks all zones until it finds
/// the card. ETB triggers fire because `place_card_in_dest` now invokes
/// `fire_self_etb_triggers` on Battlefield zone changes. Rebound is wired
/// via `Keyword::Rebound`: the cast-from-hand resolution path detects it
/// and pushes a `YourNextUpkeep` `DelayedTrigger` whose body is the
/// spell's effect — the body re-targets fresh on fire.
pub fn ephemerate() -> CardDefinition {
    CardDefinition {
        name: "Ephemerate",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![crate::card::Keyword::Rebound],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
            },
            Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
        ]),
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

/// Faithful Mending — {1}{W} Sorcery. Discard up to two cards. Draw two
/// cards. Gain 2 life. Flashback {1}{B}.
///
/// "Up to two" is exposed as a `ChooseMode` with three options (discard 2,
/// discard 1, discard 0). Mode 0 is the full discard so `AutoDecider` (and
/// the bot) keep the gameplay-optimal choice for this deck — Faithful
/// Mending is most often cast to dump a fatty for reanimation. Flashback
/// {1}{B} is wired via `Keyword::Flashback` + the existing flashback cast
/// path.
pub fn faithful_mending() -> CardDefinition {
    use crate::card::Keyword;
    use crate::mana::ManaCost as Mc;
    let flashback_cost = Mc {
        symbols: vec![
            crate::mana::ManaSymbol::Generic(1),
            crate::mana::ManaSymbol::Colored(Color::Black),
        ],
    };
    CardDefinition {
        name: "Faithful Mending",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(flashback_cost)],
        effect: Effect::Seq(vec![
            Effect::ChooseMode(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(2),
                    random: false,
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
                },
                Effect::Noop,
            ]),
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]),
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

/// Force of Negation — {1}{U}{U} Instant. Counter target noncreature spell.
/// Alternative cost: "If it's not your turn, you may exile a blue card from
/// your hand rather than pay this spell's mana cost." `not_your_turn_only`
/// gates the alt cast in `cast_spell_alternative`.
pub fn force_of_negation() -> CardDefinition {
    CardDefinition {
        name: "Force of Negation",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterSpell {
            what: target_filtered(SelectionRequirement::Noncreature),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: Some(AlternativeCost {
            mana_cost: ManaCost::default(),
            life_cost: 0,
            exile_filter: Some(SelectionRequirement::HasColor(Color::Blue)),
            evoke_sacrifice: false,
            not_your_turn_only: true,
            target_filter: None,
        }),
        back_face: None,
        opening_hand: None,
    }
}

/// Goryo's Vengeance — {1}{B} Instant. Return target legendary creature card
/// from a graveyard to the battlefield. It gains haste. Exile it at the
/// beginning of the next end step.
///
/// Implemented as `Seq([Move(target legendary → BF), GrantKeyword(Haste, EOT),
/// DelayUntil(NextEndStep, Exile(Target(0)))])`.
pub fn goryos_vengeance() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Goryo's Vengeance",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasSupertype(crate::card::Supertype::Legendary)),
                ),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            // Grant the reanimated creature haste until end of turn so it
            // can swing immediately.
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: crate::card::Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::NextEndStep,
                // Exile the same creature we just reanimated. The captured
                // target slot is preserved on the delayed trigger.
                body: Box::new(Effect::Exile {
                    what: Selector::Target(0),
                }),
                capture: None,
            },
        ]),
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

/// Prismatic Ending — {W} Sorcery — Convoke. Exile target nonland permanent
/// with mana value less than or equal to the spell's converged value.
///
/// Targeted; the cast-time filter is just `Permanent ∧ Nonland` (no CMC
/// constraint, since converged value is only known after paying). At
/// resolution, `If(ManaValueOf(Target) ≤ ConvergedValue, Exile, Noop)`
/// gates the exile. Caster pays {W} for converge=1; convoke + non-white
/// generic payments raise converge.
pub fn prismatic_ending() -> CardDefinition {
    CardDefinition {
        name: "Prismatic Ending",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::If {
            cond: Predicate::ValueAtMost(
                Value::ManaValueOf(Box::new(Selector::Target(0))),
                Value::ConvergedValue,
            ),
            then: Box::new(Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
            }),
            else_: Box::new(Effect::Noop),
        },
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

/// Thoughtseize — {B} Sorcery. Target player reveals their hand; you choose
/// a nonland card; they discard it. You lose 2 life.
///
/// Targeting is approximated as `EachOpponent` (2P-correct). The caster
/// auto-picks the first nonland card.
pub fn thoughtseize() -> CardDefinition {
    CardDefinition {
        name: "Thoughtseize",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Nonland,
            },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
        ]),
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

// ── Goryo's sideboard spells ────────────────────────────────────────────────

/// Consign to Memory — {U} Instant. "Counter target activated or
/// triggered ability OR counter target legendary spell." Modal:
///
/// - **Mode 0**: target a permanent → counter the topmost trigger
///   sourced from it (`Effect::CounterAbility`).
/// - **Mode 1**: target a legendary spell on the stack → counter it
///   (`Effect::CounterSpell` over `IsSpellOnStack ∧ HasSupertype(Legendary)`).
///
/// `AutoDecider` picks mode 0 by default — the Goryo's matchup wants
/// the ability-counter half (Atraxa ETB, Devourer Scry, etc.). The
/// mode-1 branch is reachable via UI / scripted decisions.
pub fn consign_to_memory() -> CardDefinition {
    CardDefinition {
        name: "Consign to Memory",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: counter target ability (the Goryo's-matchup default).
            Effect::CounterAbility {
                what: target_filtered(SelectionRequirement::Permanent),
            },
            // Mode 1: counter target legendary spell.
            Effect::CounterSpell {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack
                        .and(SelectionRequirement::HasSupertype(crate::card::Supertype::Legendary)),
                ),
            },
        ]),
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

/// Damping Sphere — {2} Artifact. "If a player has cast another spell this
/// turn, each spell that player casts costs {1} more to cast. Lands that tap
/// to add more than one mana enter producing only {C}."
///
/// Both halves wired:
///   * `AdditionalCostAfterFirstSpell` — cast paths consult per-player
///     `spells_cast_this_turn`; if ≥ 1 the spell pays an extra {1}.
///   * `LandsTapColorlessOnly` — `play_land` consults
///     `lands_tap_colorless_only_active()`; multi-mana / dual-color lands
///     have their mana abilities replaced with a single `{T}: Add {C}` on
///     ETB. Single-color basics and one-ability single-color non-basics
///     pass through unchanged.
pub fn damping_sphere() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Damping Sphere",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![
            StaticAbility {
                description: "Each spell a player casts after their first this turn costs {1} more.",
                effect: StaticEffect::AdditionalCostAfterFirstSpell {
                    filter: SelectionRequirement::Any,
                    amount: 1,
                },
            },
            StaticAbility {
                description: "Lands that tap to add more than one mana enter producing only {C}.",
                effect: StaticEffect::LandsTapColorlessOnly,
            },
        ],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Mystical Dispute — {2}{U} Instant. Counter target spell unless its
/// controller pays {3}. If the spell is blue, this costs {U} less.
///
/// The "{U} less if blue" is modeled as an alternative cost: pay {U}
/// instead of {2}{U}, and the alt cost's `target_filter` requires the
/// targeted stack spell to be blue. The "unless they pay {3}" rider is
/// wired via `Effect::CounterUnlessPaid` — at resolution the engine
/// auto-pays on behalf of the spell's controller; if affordable the
/// spell stays, otherwise it's countered.
pub fn mystical_dispute() -> CardDefinition {
    CardDefinition {
        name: "Mystical Dispute",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(3)]),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[u()]),
            life_cost: 0,
            exile_filter: None,
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: Some(SelectionRequirement::HasColor(Color::Blue)),
        }),
        back_face: None,
        opening_hand: None,
    }
}

/// Pest Control — {W}{B} Sorcery — Convoke. Destroy each nonland permanent
/// with mana value less than or equal to the spell's converged value.
///
/// Convoke lets the caster tap creatures to contribute generic mana toward
/// the cost. Converge counts distinct colors of mana spent — at minimum 2
/// (one white + one black pip) when cast for {W}{B}, but if the caster
/// also pays the generic part with red/blue/green mana, the converged
/// value scales up. Effect uses `ForEach + If` over `Nonland` permanents,
/// destroying any whose `ManaValueOf` is ≤ `Value::ConvergedValue`.
pub fn pest_control() -> CardDefinition {
    use crate::card::Keyword;
    CardDefinition {
        name: "Pest Control",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Convoke],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Nonland),
            body: Box::new(Effect::If {
                cond: Predicate::ValueAtMost(
                    Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                    Value::ConvergedValue,
                ),
                then: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
                else_: Box::new(Effect::Noop),
            }),
        },
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

/// Wrath of the Skies — {X}{W}{W} Sorcery — Convoke. Destroy each nonland
/// permanent with mana value X.
///
/// Implemented as `ForEach(EachPermanent(Nonland))` body that destroys the
/// current entity if its mana value equals X (the X paid into the spell's
/// cost). `Value::ManaValueOf(Selector::TriggerSource)` reads the iterated
/// permanent's CMC. Convoke now lets the caster tap creatures to pay
/// generic mana toward the X cost.
pub fn wrath_of_the_skies() -> CardDefinition {
    use crate::card::Keyword;
    CardDefinition {
        name: "Wrath of the Skies",
        cost: cost(&[x(), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Convoke],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Nonland),
            body: Box::new(Effect::If {
                cond: Predicate::All(vec![
                    Predicate::ValueAtLeast(
                        Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                        Value::XFromCost,
                    ),
                    Predicate::ValueAtMost(
                        Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                        Value::XFromCost,
                    ),
                ]),
                then: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
                else_: Box::new(Effect::Noop),
            }),
        },
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
