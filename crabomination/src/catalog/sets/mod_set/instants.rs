//! Modern-staple instants (interaction).

use super::no_abilities;
use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, Keyword, SelectionRequirement, Subtypes,
    TokenDefinition,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{DelayedTriggerKind, Duration, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

/// Path to Exile — {W} Instant. Exile target creature; its controller may
/// search their library for a basic land card, put it onto the battlefield
/// tapped, then shuffle.
///
/// Approximation: the basic-land tutor half is omitted (the engine has no
/// "search and put onto battlefield tapped" effect that's also player-
/// directed at the *target's controller*). Single-step exile is wired.
pub fn path_to_exile() -> CardDefinition {
    CardDefinition {
        name: "Path to Exile",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Exile {
            what: target_filtered(SelectionRequirement::Creature),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Fatal Push — {B} Instant. Destroy target creature with mana value 2 or
/// less. (Revolt clause — destroying a creature with mana value 4 or less
/// if a permanent left the battlefield this turn — is omitted; the base
/// half is what matters for the bulk of plays.)
pub fn fatal_push() -> CardDefinition {
    CardDefinition {
        name: "Fatal Push",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ManaValueAtMost(2)),
            ),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Spell Pierce — {U} Instant. Counter target noncreature spell unless its
/// controller pays {2}.
pub fn spell_pierce() -> CardDefinition {
    CardDefinition {
        name: "Spell Pierce",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack
                    .and(SelectionRequirement::Noncreature),
            ),
            mana_cost: cost(&[generic(2)]),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Mana Leak — {1}{U} Instant. Counter target spell unless its controller
/// pays {3}.
pub fn mana_leak() -> CardDefinition {
    CardDefinition {
        name: "Mana Leak",
        cost: cost(&[generic(1), u()]),
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
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Doom Blade — {1}{B} Instant. Destroy target nonblack creature.
pub fn doom_blade() -> CardDefinition {
    CardDefinition {
        name: "Doom Blade",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasColor(Color::Black).negate()),
            ),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Vapor Snag — {U} Instant. Return target creature to its owner's hand.
/// That creature's controller loses 1 life.
///
/// Order: life loss first, then bounce — by the time we resolve life loss,
/// the targeted creature is still on the battlefield so
/// `ControllerOf(Target(0))` finds it. (Owner is stable across zone changes
/// while controller is not.)
pub fn vapor_snag() -> CardDefinition {
    CardDefinition {
        name: "Vapor Snag",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(1),
            },
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Consider — {U} Instant. Surveil 1, then draw a card.
pub fn consider() -> CardDefinition {
    CardDefinition {
        name: "Consider",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Thought Scour — {U} Instant. Target player puts the top two cards of
/// their library into their graveyard. Draw a card.
pub fn thought_scour() -> CardDefinition {
    CardDefinition {
        name: "Thought Scour",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Mill {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Tarfire — {R} Tribal Instant. Tarfire deals 2 damage to any target.
///
/// Note: real Tarfire is a "Tribal Instant — Goblin", but the engine has
/// no Tribal card type (and no card in the catalog keys off of "is Goblin
/// spell"), so the tribal half is omitted. Damage to any target is wired
/// faithfully.
pub fn tarfire() -> CardDefinition {
    CardDefinition {
        name: "Tarfire",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Kindred, CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Any),
            amount: Value::Const(2),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Frantic Search — {2}{U} Instant. Draw two cards, then discard two cards.
/// Untap up to three lands.
///
/// ✅ Wired with `Effect::Untap.up_to: Some(Const(3))` (push V's
/// `up_to` cap) — exactly three lands untap, matching the printed
/// Oracle.
pub fn frantic_search() -> CardDefinition {
    CardDefinition {
        name: "Frantic Search",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(2),
                random: false,
            },
            Effect::Untap {
                what: Selector::EachPermanent(
                    SelectionRequirement::Land
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::Tapped),
                ),
                up_to: Some(Value::Const(3)),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Slaughter Pact — {0} Instant. Destroy target nonblack creature. "At the
/// beginning of your next upkeep, pay {2}{B}. If you don't, you lose the
/// game." Reuses the Pact-of-Negation primitive (`PayOrLoseGame` scheduled
/// via `DelayUntil(YourNextUpkeep)`).
pub fn slaughter_pact() -> CardDefinition {
    CardDefinition {
        name: "Slaughter Pact",
        cost: crate::mana::ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasColor(Color::Black).negate()),
                ),
            },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::YourNextUpkeep,
                body: Box::new(Effect::PayOrLoseGame {
                    mana_cost: cost(&[generic(2), b()]),
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
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Pact of the Titan — {0} Instant. Create a 4/4 red Giant creature token.
/// "At the beginning of your next upkeep, pay {4}{R}. If you don't, you
/// lose the game." Same Pact pattern; the Giant token is built inline.
pub fn pact_of_the_titan() -> CardDefinition {
    CardDefinition {
        name: "Pact of the Titan",
        cost: crate::mana::ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Giant".into(),
                    power: 4,
                    toughness: 4,
                    keywords: vec![],
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Red],
                    supertypes: vec![],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Giant],
                        ..Default::default()
                    },
                    activated_abilities: vec![],
                    triggered_abilities: vec![],
                },
            },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::YourNextUpkeep,
                body: Box::new(Effect::PayOrLoseGame {
                    mana_cost: cost(&[generic(4), r()]),
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
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Spell Snare — {U} Instant. Counter target spell with mana value 2.
/// The "exactly 2" filter is built from
/// `ManaValueAtLeast(2).and(ManaValueAtMost(2))`.
pub fn spell_snare() -> CardDefinition {
    CardDefinition {
        name: "Spell Snare",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterSpell {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack
                    .and(SelectionRequirement::ManaValueAtLeast(2))
                    .and(SelectionRequirement::ManaValueAtMost(2)),
            ),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Daze — {1}{U} Instant. Counter target spell unless its controller pays
/// {1}.
///
/// Approximation: the Force-spike alternative cost ("you may return an
/// Island you control to its owner's hand rather than pay this spell's
/// mana cost") is omitted — the alt-cost model only supports
/// exile-from-hand, not return-from-battlefield. Fixed-cost path is wired
/// faithfully.
pub fn daze() -> CardDefinition {
    CardDefinition {
        name: "Daze",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(1)]),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Swan Song — {U} Instant. Counter target enchantment, instant, or sorcery
/// spell. Its controller creates a 2/2 blue Bird creature token with flying.
///
/// Approximation: instead of giving the token to the spell's controller,
/// this version gives one to each of the caster's opponents — exactly
/// equivalent in 2-player play and harmless in multiplayer (slightly more
/// generous than the real text). The engine has no `ControllerOf` lookup
/// for stack/graveyard cards, so we can't pin the token to the precise
/// player whose spell got countered.
pub fn swan_song() -> CardDefinition {
    CardDefinition {
        name: "Swan Song",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::EachOpponent,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Bird".into(),
                    power: 2,
                    toughness: 2,
                    keywords: vec![Keyword::Flying],
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Blue],
                    supertypes: vec![],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Bird],
                        ..Default::default()
                    },
                    activated_abilities: vec![],
                    triggered_abilities: vec![],
                },
            },
            Effect::CounterSpell {
                what: target_filtered(
                    SelectionRequirement::HasCardType(CardType::Enchantment)
                        .or(SelectionRequirement::HasCardType(CardType::Instant))
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                ),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Drown in Ichor — {1}{B} Sorcery. Drown in Ichor deals 3 damage to target
/// creature. Surveil 1.
pub fn drown_in_ichor() -> CardDefinition {
    CardDefinition {
        name: "Drown in Ichor",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(3),
            },
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Paradoxical Outcome — {3}{U}{U} Instant. Return any number of non-land
/// permanents you control to their owners' hands. Draw a card for each
/// card returned this way.
///
/// Implemented via `ForEach { non-land permanents you control: bounce + draw 1 }`.
/// The selector is resolved once at the start, so cards moved to hand
/// don't affect later iterations of the loop.
pub fn paradoxical_outcome() -> CardDefinition {
    CardDefinition {
        name: "Paradoxical Outcome",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::ControlledByYou
                    .and(SelectionRequirement::Nonland),
            ),
            body: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ])),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Isolate — {W} Instant. Exile target permanent with mana value 1.
///
/// "Mana value 1" is built from
/// `ManaValueAtLeast(1) ∧ ManaValueAtMost(1)`. The filter rules out
/// 0-mana lands and tokens (mana value 0) and any 2+ MV permanents.
pub fn isolate() -> CardDefinition {
    CardDefinition {
        name: "Isolate",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Permanent
                    .and(SelectionRequirement::ManaValueAtLeast(1))
                    .and(SelectionRequirement::ManaValueAtMost(1)),
            ),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Pyrokinesis — {4}{R}{R} Instant. You may exile a red card from your
/// hand rather than pay this spell's mana cost. Pyrokinesis deals 4
/// damage divided as you choose among any number of target creatures.
///
/// Wires the existing pitch alt-cost (same shape as Force of Will). The
/// "divide 4 damage among any number of creatures" half is approximated
/// as 4 damage to a single creature target — the engine has no
/// damage-distribution primitive yet. AutoDecider picks the highest-
/// toughness opponent creature first.
pub fn pyrokinesis() -> CardDefinition {
    use crate::card::AlternativeCost;
    CardDefinition {
        name: "Pyrokinesis",
        cost: cost(&[generic(4), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(4),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: Some(AlternativeCost {
            mana_cost: ManaCost::default(),
            life_cost: 0,
            exile_filter: Some(SelectionRequirement::HasColor(Color::Red)),
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: None,
            mode_on_alt: None,
        }),
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Bone Shards — {B} Instant. As an additional cost, sacrifice a creature
/// or discard a card. Destroy target creature.
///
/// The modal additional cost is wired as a `ChooseMode([Sacrifice, Discard])`
/// run before the destroy. Mode 0 sacs a creature; mode 1 discards a card.
/// Either way the destroy then resolves on the targeted creature. This
/// reuses the same "cost-as-first-step" pattern as Thud, Plunge into
/// Darkness, and Crop Rotation — the engine doesn't yet model true
/// additional costs paid at cast time, but folding them into the
/// resolution sequence is gameplay-equivalent for the bulk of plays.
/// AutoDecider picks mode 0 (sacrifice) by default.
pub fn bone_shards() -> CardDefinition {
    CardDefinition {
        name: "Bone Shards",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::ChooseMode(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: SelectionRequirement::Creature,
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
                },
            ]),
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Creature),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Bloodchief's Thirst — {B} Sorcery. Destroy target creature or
/// planeswalker with mana value 2 or less. Kicker {1}{B}; if kicked,
/// destroy target creature or planeswalker with mana value 6 or less.
///
/// Approximation: ships the **base** mode (mana value ≤ 2). Kicker
/// support requires the alt-cost path to swap target filter at cast
/// time, which it does for blue spells (Mystical Dispute) but not for
/// the destroy-with-different-CMC pattern. That's a follow-up.
pub fn bloodchiefs_thirst() -> CardDefinition {
    CardDefinition {
        name: "Bloodchief's Thirst",
        cost: cost(&[b()]),
        supertypes: vec![],
        // Real Oracle: Sorcery. Kept here so the ChooseMode-less version
        // is consistent with sorcery-speed timing.
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Planeswalker)
                    .and(SelectionRequirement::ManaValueAtMost(2)),
            ),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Deadly Dispute — {1}{B} Sorcery. As an additional cost, sacrifice an
/// artifact or creature. Draw two cards and create a Treasure token.
///
/// Sac-as-additional-cost is folded into the resolved effect (cost-as-
/// first-step pattern, matching Thud / Cephalid Coliseum).
pub fn deadly_dispute() -> CardDefinition {
    CardDefinition {
        name: "Deadly Dispute",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::SacrificeAndRemember {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::Artifact)
                    .and(SelectionRequirement::ControlledByYou),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::treasure_token(),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Cryptic Command — {1}{U}{U}{U} Instant. Choose two — counter target spell;
/// return target permanent to its owner's hand; tap all creatures your
/// opponents control; draw a card.
///
/// "Choose two" is approximated as "pick one of these four bundled
/// modes" via `ChooseMode` (the engine has no multi-pick mode primitive
/// yet). Mode 0 — counter + bounce. Mode 1 — counter + tap-down. Mode
/// 2 — counter + draw. Mode 3 — bounce + draw. AutoDecider picks
/// mode 0 (counter + bounce) which is the most reactive line.
///
/// The "tap all creatures your opponents control" half is wired via
/// `ForEach(EachPermanent(Creature ∧ ControlledByOpponent))` + `Tap`.
pub fn cryptic_command() -> CardDefinition {
    CardDefinition {
        name: "Cryptic Command",
        cost: cost(&[generic(1), u(), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: counter target spell + bounce target permanent.
            // (Single shared target slot — both halves operate on the
            // same target. The engine doesn't surface multi-target
            // selection yet, so this slot resolves to the targeted
            // stack spell, and the bounce step then bounces the
            // permanent picked at cast time. With a single target
            // slot, players targeting their own permanent for bounce
            // get the counter no-op, which is acceptable.)
            Effect::Seq(vec![
                Effect::CounterSpell {
                    what: target_filtered(SelectionRequirement::IsSpellOnStack),
                },
                Effect::Move {
                    what: target_filtered(SelectionRequirement::Permanent),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
            ]),
            // Mode 1: counter + tap-all-opp-creatures.
            Effect::Seq(vec![
                Effect::CounterSpell {
                    what: target_filtered(SelectionRequirement::IsSpellOnStack),
                },
                Effect::ForEach {
                    selector: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent),
                    ),
                    body: Box::new(Effect::Tap { what: Selector::TriggerSource }),
                },
            ]),
            // Mode 2: counter + draw 1.
            Effect::Seq(vec![
                Effect::CounterSpell {
                    what: target_filtered(SelectionRequirement::IsSpellOnStack),
                },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ]),
            // Mode 3: bounce + draw 1.
            Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(SelectionRequirement::Permanent),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ]),
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Blossoming Defense — {G} Instant. Target creature you control gets +2/+2
/// and gains hexproof until end of turn.
pub fn blossoming_defense() -> CardDefinition {
    CardDefinition {
        name: "Blossoming Defense",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Treasure Cruise — {7}{U} Instant. Delve. Draw three cards.
///
/// The Delve cost-reduction (exile cards from your graveyard to pay {1}
/// each toward this cost) isn't modeled yet — Treasure Cruise is castable
/// at its full {7}{U} cost. The "Draw 3" half is wired faithfully so the
/// card has a real effect once a player can afford it; Delve support is
/// tracked as a future engine feature in CUBE_FEATURES.md.
pub fn treasure_cruise() -> CardDefinition {
    CardDefinition {
        name: "Treasure Cruise",
        cost: cost(&[generic(7), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Const(3),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Lose Focus — {U} Instant. Delve. Counter target spell unless its
/// controller pays {2}.
///
/// Delve is omitted (no cost reduction yet) — Lose Focus is castable at
/// its full {U} cost as a one-mana "{U}: counter unless they pay {2}".
/// Reuses `Effect::CounterUnlessPaid`. With Delve disabled the card is
/// strictly stronger than the printed Oracle (no graveyard exile cost),
/// but it still functions correctly against the unless-pay clause.
pub fn lose_focus() -> CardDefinition {
    CardDefinition {
        name: "Lose Focus",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(2)]),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Stifle — {U} Instant. Counter target activated or triggered ability.
///
/// Wired via `Effect::CounterAbility` (the Consign-to-Memory primitive).
/// Targets a permanent — the engine walks the stack top-down and removes
/// the topmost `StackItem::Trigger` whose `source` matches that permanent.
/// The "or activated ability" branch reuses the same primitive: an
/// activated ability also pushes a `StackItem::Trigger` with `source ==
/// the activator's permanent`, so a single CounterAbility variant covers
/// both cases.
pub fn stifle() -> CardDefinition {
    CardDefinition {
        name: "Stifle",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterAbility {
            what: target_filtered(SelectionRequirement::Permanent),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Memory Lapse — {1}{U} Instant. Counter target spell. If countered this
/// way, put the spell on top of its owner's library instead of into the
/// graveyard.
///
/// Approximation: just `CounterSpell` — the "place on top of library"
/// half needs a "send-stack-item-elsewhere-on-counter" hook the engine
/// doesn't surface. Functionally a one-shot Mana Leak for now; future
/// engine work can route the countered card through the right zone.
pub fn memory_lapse() -> CardDefinition {
    CardDefinition {
        name: "Memory Lapse",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterSpell {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Vines of Vastwood — {G} Instant. Kicker {G}{G}. Target creature can't
/// be the target of spells or abilities your opponents control this turn.
/// If kicked, that creature gets +4/+4 until end of turn.
///
/// Approximation: collapse to the kicked mode (paid {G} extra) and grant
/// +4/+4 EOT. The hexproof half (untargetable by opponents) is omitted —
/// the engine has no per-permanent "untargetable by opponents" duration
/// effect distinct from the global `Keyword::Hexproof`. Cost shown as the
/// kicker total {G}{G} so the engine doesn't surface a separate kicker
/// decision.
pub fn vines_of_vastwood() -> CardDefinition {
    CardDefinition {
        name: "Vines of Vastwood",
        cost: cost(&[g(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(4),
            toughness: Value::Const(4),
            duration: Duration::EndOfTurn,
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Turnabout — {2}{U}{U} Instant. "Choose one — Tap all artifacts target
/// player controls; or untap all artifacts target player controls; or
/// tap all creatures target player controls; or untap all creatures
/// target player controls; or tap all lands target player controls; or
/// untap all lands target player controls."
///
/// Wired as a `ChooseMode` with six modes — artifact tap/untap,
/// creature tap/untap, land tap/untap — each operating against
/// `EachPermanent` filtered by the type plus `ControlledByOpponent`.
/// Turnabout actually picks any player; we collapse to "the
/// opponent" since the typical use is locking down an opponent's
/// board. AutoDecider picks mode 4 (tap all lands) — the canonical
/// lock application.
pub fn turnabout() -> CardDefinition {
    fn each_opp(req: SelectionRequirement) -> Selector {
        Selector::EachPermanent(req.and(SelectionRequirement::ControlledByOpponent))
    }
    CardDefinition {
        name: "Turnabout",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Tap   { what: each_opp(SelectionRequirement::Artifact) },
            Effect::Untap { what: each_opp(SelectionRequirement::Artifact), up_to: None },
            Effect::Tap   { what: each_opp(SelectionRequirement::Creature) },
            Effect::Untap { what: each_opp(SelectionRequirement::Creature), up_to: None },
            Effect::Tap   { what: each_opp(SelectionRequirement::Land) },
            Effect::Untap { what: each_opp(SelectionRequirement::Land), up_to: None },
        ]),
        ..Default::default()
    }
}
