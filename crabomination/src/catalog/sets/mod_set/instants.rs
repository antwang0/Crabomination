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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        card_types: vec![CardType::Instant],
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
        back_face: None,
        opening_hand: None,
    }
}

/// Frantic Search — {2}{U} Instant. Draw two cards, then discard two cards.
/// Untap up to three lands.
///
/// Approximation: "up to three lands" is implemented as "untap every land
/// you control" — a small overshoot, but the spirit of the card (refill
/// mana mid-turn) lands intact. A precise three-target version would need
/// a multi-target prompt that the engine doesn't yet expose.
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
                    name: "Giant",
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
                },
            },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::YourNextUpkeep,
                body: Box::new(Effect::PayOrLoseGame {
                    mana_cost: cost(&[generic(4), r()]),
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
                    name: "Bird",
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
        back_face: None,
        opening_hand: None,
    }
}

/// Drown in Ichor — {1}{B} Instant. Drown in Ichor deals 3 damage to target
/// creature. Surveil 1.
pub fn drown_in_ichor() -> CardDefinition {
    CardDefinition {
        name: "Drown in Ichor",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
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
        back_face: None,
        opening_hand: None,
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
        cost: cost(&[generic(3), u(), u()]),
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        }),
        back_face: None,
        opening_hand: None,
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
        card_types: vec![CardType::Instant],
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
    }
}
