//! Spells and enchantments for the BRG and Goryo's demo decks.
//!
//! Several spells here ship as stubs (`Effect::Noop`) where their real
//! behavior needs engine features that don't yet exist (alternative pitch
//! costs, pact deferred upkeep payments, exile-at-end-of-turn replacements,
//! rebound, convoke / converge, etc.). Each carries a doc-comment marking
//! what's missing; promote as engine features land.

use super::super::no_abilities;
use crate::card::{
    AlternativeCost, CardDefinition, CardType, Effect, SelectionRequirement, Subtypes,
};
use crate::effect::shortcut::{counter_target_spell, target_filtered};
use crate::effect::{DelayedTriggerKind, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, generic, r, u, w};

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
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
    }
}

/// Plunge into Darkness — {1}{B} Instant. Modal — choose one:
///   • Sacrifice any number of creatures. You gain 3 life for each one.
///   • Pay any amount of life. Look at that many cards from the top of your
///     library, then put one into your hand and the rest on the bottom.
///
/// Engine model: `ChooseMode` between two simplified branches:
/// mode 0 — sacrifice one creature you control + gain 3 life;
/// mode 1 — Effect::Noop (the look-at-X / library manipulation requires a
/// custom decision UI we don't have yet).
/// AutoDecider picks mode 0, which matches the typical play (sac for life).
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
            // Mode 1: pay-X-life, look-at-X, pick one — unimplemented.
            Effect::Noop,
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
    }
}

/// Serum Powder — {3} Artifact. {T}: Add {1}. "Any time you could mulligan
/// and Serum Powder is in your hand, you may exile all the cards from your
/// hand, then draw that many cards." Stub: vanilla 3-cost mana rock for {1}.
/// TODO: opening-hand exile-and-redraw mulligan helper.
pub fn serum_powder() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::ManaPayload;
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
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
    }
}

/// Spoils of the Vault — {B} Instant. Name a card. Reveal cards from the top
/// of your library until you reveal the named card or 10 different cards.
/// Stub: Effect::Noop — reveal-until-find isn't supported by the engine.
/// TODO: wire name-and-reveal-until-find.
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
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
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
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
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
    }
}

/// Leyline of Sanctity — {2}{W}{W} Enchantment. "If Leyline of Sanctity is in
/// your opening hand, you may begin the game with it on the battlefield. You
/// have hexproof." Stub: vanilla 4-cost enchantment with no game effect.
/// TODO: opening-hand-into-play + you-have-hexproof static.
pub fn leyline_of_sanctity() -> CardDefinition {
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
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
    }
}

// ── Goryo's main-deck spells ────────────────────────────────────────────────

/// Ephemerate — {W} Instant. Exile target creature you control, then return
/// it to the battlefield under its owner's control. Rebound (cast-from-exile
/// next upkeep) is still TODO.
///
/// Modeled as `Seq([Exile target creature you control, Move target back to
/// the battlefield])`. The same target slot is re-resolved on the second
/// step — `Selector::Target(0)` finds the card in exile (the engine
/// `evaluate_requirement_static` falls through to graveyard / exile so the
/// target stays bound), and `move_card_to` walks all zones until it finds
/// the card. ETB triggers fire because `place_card_in_dest` now invokes
/// `fire_self_etb_triggers` on Battlefield zone changes.
pub fn ephemerate() -> CardDefinition {
    CardDefinition {
        name: "Ephemerate",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
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
    }
}

/// Faithful Mending — {1}{W} Sorcery. Discard up to two cards. Draw two
/// cards. Gain 2 life. Flashback {1}{B}.
///
/// The "up to two" discard is implemented as `Discard { count: 2 }`; the
/// loop breaks when the hand runs out, which is gameplay-equivalent for
/// the common line. Flashback {1}{B} is wired via `Keyword::Flashback`
/// and the existing flashback cast path.
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
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(flashback_cost)],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(2),
                random: false,
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
    }
}

/// Force of Negation — {1}{U}{U} Instant. Counter target noncreature spell.
/// Alternative cost: if it's not your turn, you may exile a blue card from
/// your hand rather than pay this spell's mana cost.
///
/// The "if it's not your turn" timing restriction on the alt cost is not
/// enforced by the engine — the player can pitch-cast Force of Negation on
/// their own turn. Practical impact is small (you wouldn't normally want to)
/// and will be revisited if it matters.
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
        }),
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
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
    }
}

/// Prismatic Ending — {W} Sorcery — Convoke. Exile target nonland permanent
/// with mana value less than or equal to the spell's converged value.
///
/// Without convoke / converge support, the converged value defaults to 1
/// (one white pip). The target slot's filter enforces "nonland permanent
/// with mana value ≤ 1", which catches one-drops like Psychic Frog,
/// Inquisition / Thoughtseize-grade hate bears, and most enablers without
/// touching the deck's threats.
/// TODO: full convoke + variable-converge integration.
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
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Permanent
                    .and(SelectionRequirement::Nonland)
                    .and(SelectionRequirement::ManaValueAtMost(1)),
            ),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
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
    }
}

// ── Goryo's sideboard spells ────────────────────────────────────────────────

/// Consign to Memory — {U} Instant. Counter target activated or triggered
/// ability OR counter target spell that's a card with the chosen name.
/// Stub: Effect::Noop — countering abilities (vs spells) isn't supported.
/// TODO: counter-an-ability primitive.
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
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
    }
}

/// Damping Sphere — {2} Artifact. "If a player has cast another spell this
/// turn, each spell that player casts costs {1} more to cast. Lands that tap
/// to add more than one mana enter producing only {C}."
/// Stub: vanilla 2-cost rock with no static effect.
/// TODO: cost-tax static + storm/sol-ring nerf.
pub fn damping_sphere() -> CardDefinition {
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
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
    }
}

/// Mystical Dispute — {2}{U} Instant. Counter target spell unless its
/// controller pays {3}. If the spell is blue, this costs {U} less.
/// Stub: simple counter at hard cost; conditional cost reduction omitted.
/// TODO: cost reduction "if target is blue".
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
        effect: counter_target_spell(),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
    }
}

/// Pest Control — {W}{B} Sorcery — Convoke. Destroy each nonland permanent
/// with mana value less than or equal to the spell's converged value.
///
/// Without convoke / converge support, the converged value defaults to 2
/// (one white + one black pip). Resolves into a global `Destroy` over every
/// nonland permanent with mana value ≤ 2 — wipes mana dorks, hatebears, and
/// the early creature curve without sweeping the heavy reanimator targets.
/// TODO: full convoke + variable-converge integration.
pub fn pest_control() -> CardDefinition {
    CardDefinition {
        name: "Pest Control",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: Selector::EachPermanent(
                SelectionRequirement::Nonland
                    .and(SelectionRequirement::ManaValueAtMost(2)),
            ),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
    }
}

/// Wrath of the Skies — {X}{W}{W} Sorcery. Destroy each nonland permanent
/// with mana value X. Convoke.
/// Stub: Effect::Noop.
/// TODO: convoke + destroy-CMC-=-X.
pub fn wrath_of_the_skies() -> CardDefinition {
    CardDefinition {
        name: "Wrath of the Skies",
        cost: cost(&[w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
    }
}
