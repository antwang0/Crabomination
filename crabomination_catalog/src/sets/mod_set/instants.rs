//! Modern-staple instants (interaction).

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, Keyword, SelectionRequirement, Subtypes,
    TokenDefinition,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{DelayedTriggerKind, Duration, PlayerRef, Predicate, Selector, Value, ZoneDest};
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
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Exile {
            what: target_filtered(SelectionRequirement::Creature),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Reverberate — {R}{R} Instant. "Copy target instant or sorcery spell.
/// You may choose new targets for the copy." Wired via
/// `Effect::CopySpellMayChooseTargets`.
pub fn reverberate() -> CardDefinition {
    CardDefinition {
        name: "Reverberate",
        cost: cost(&[r(), r()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        effect: Effect::CopySpellMayChooseTargets {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack.and(
                    SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                ),
            ),
            count: Value::Const(1),
        },
        ..Default::default()
    }
}

/// Fork — {R}{R} Instant. "Copy target instant or sorcery spell, except
/// that the copy is red. You may choose new targets for the copy." The
/// "copy is red" colour rewrite is cosmetic for the engine; the copy +
/// new-target choice rides `Effect::CopySpellMayChooseTargets`.
pub fn fork() -> CardDefinition {
    CardDefinition {
        name: "Fork",
        cost: cost(&[r(), r()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        effect: Effect::CopySpellMayChooseTargets {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack.and(
                    SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                ),
            ),
            count: Value::Const(1),
        },
        ..Default::default()
    }
}

/// Redirect — {U}{U} Instant. "You may choose new targets for target
/// spell." Repoints the targeted spell's primary target in place via
/// `Effect::ChooseNewTargetsForSpell` (CR 115.7).
pub fn redirect() -> CardDefinition {
    CardDefinition {
        name: "Redirect",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        effect: Effect::ChooseNewTargetsForSpell {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
        },
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Spell Pierce — {U} Instant. Counter target noncreature spell unless its
/// controller pays {2}.
pub fn spell_pierce() -> CardDefinition {
    CardDefinition {
        name: "Spell Pierce",
        cost: cost(&[u()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Mana Leak — {1}{U} Instant. Counter target spell unless its controller
/// pays {3}.
pub fn mana_leak() -> CardDefinition {
    CardDefinition {
        name: "Mana Leak",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(3)]),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Doom Blade — {1}{B} Instant. Destroy target nonblack creature.
pub fn doom_blade() -> CardDefinition {
    CardDefinition {
        name: "Doom Blade",
        cost: cost(&[generic(1), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Consider — {U} Instant. Surveil 1, then draw a card.
pub fn consider() -> CardDefinition {
    CardDefinition {
        name: "Consider",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Thought Scour — {U} Instant. Target player puts the top two cards of
/// their library into their graveyard. Draw a card.
pub fn thought_scour() -> CardDefinition {
    CardDefinition {
        name: "Thought Scour",
        cost: cost(&[u()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Tarfire — {R} Kindred Instant — Goblin. Tarfire deals 2 damage to
/// any target.
///
/// Push (modern_decks doc-sync): the `CardType::Kindred` tag is already
/// wired (was added to the enum in a prior push). This push completes
/// the printed type line by adding the Goblin creature subtype as well,
/// so Goblin-tribal payoffs (Goblin Bushwhacker, Wort, Boggart Auntie,
/// etc.) can recognise Tarfire on the stack via
/// `Predicate::EntityMatches(TriggerSource, HasCreatureType(Goblin))`.
/// Damage to any target is wired faithfully.
pub fn tarfire() -> CardDefinition {
    CardDefinition {
        name: "Tarfire",
        cost: cost(&[r()]),
        card_types: vec![CardType::Kindred, CardType::Instant],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Any),
            amount: Value::Const(2),
        },
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Pact of the Titan — {0} Instant. Create a 4/4 red Giant creature token.
/// "At the beginning of your next upkeep, pay {4}{R}. If you don't, you
/// lose the game." Same Pact pattern; the Giant token is built inline.
pub fn pact_of_the_titan() -> CardDefinition {
    CardDefinition {
        name: "Pact of the Titan",
        cost: crate::mana::ManaCost::default(),
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
                
                    static_abilities: vec![],
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Spell Snare — {U} Instant. Counter target spell with mana value 2.
/// The "exactly 2" filter is built from
/// `ManaValueAtLeast(2).and(ManaValueAtMost(2))`.
pub fn spell_snare() -> CardDefinition {
    CardDefinition {
        name: "Spell Snare",
        cost: cost(&[u()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Daze — {1}{U} Instant. Counter target spell unless its controller pays
/// {1}. Alternative cost: return an Island you control to its owner's
/// hand (the classic free-counter line) via `return_to_hand`.
pub fn daze() -> CardDefinition {
    use crate::card::{AlternativeCost, LandType};
    CardDefinition {
        name: "Daze",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(1)]),
        },
        alternative_cost: Some(AlternativeCost {
            return_to_hand: Some((SelectionRequirement::HasLandType(LandType::Island), 1)),
            ..Default::default()
        }),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Swan Song — {U} Instant. Counter target enchantment, instant, or sorcery
/// spell. Its controller creates a 2/2 blue Bird creature token with flying.
///
/// Token target is the **countered spell's controller** via
/// `PlayerRef::ControllerOf(Target(0))` — `stack_caster_for_card`
/// resolves a stack-resident spell back to its caster. Pre-fix this
/// was approximated as `EachOpponent` of the Swan Song caster
/// (equivalent in 2-player play but wrong in multiplayer).
pub fn swan_song() -> CardDefinition {
    CardDefinition {
        name: "Swan Song",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
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
                
                    static_abilities: vec![],
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Drown in Ichor — {1}{B} Sorcery. Drown in Ichor deals 3 damage to target
/// creature. Surveil 1.
pub fn drown_in_ichor() -> CardDefinition {
    CardDefinition {
        name: "Drown in Ichor",
        cost: cost(&[generic(1), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Pyrokinesis — {4}{R}{R} Instant. You may exile a red card from your
/// hand rather than pay this spell's mana cost. Pyrokinesis deals 4
/// damage divided as you choose among any number of target creatures.
///
/// Wires the existing pitch alt-cost (same shape as Force of Will) and the
/// `DealDamageDivided` primitive: 4 damage split among up to four target
/// creatures (AutoDecider spreads evenly; a UI/test decider chooses).
pub fn pyrokinesis() -> CardDefinition {
    use crate::card::AlternativeCost;
    CardDefinition {
        name: "Pyrokinesis",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamageDivided {
            total: Value::Const(4),
            filter: SelectionRequirement::Creature,
            max_targets: 4,
        },
        triggered_abilities: vec![],
        alternative_cost: Some(AlternativeCost {
            mana_cost: ManaCost::default(),
            life_cost: 0,
            exile_filter: Some(SelectionRequirement::HasColor(Color::Red)),
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: None,
            condition: None,
                    exile_from_graveyard_count: 0,
                    return_to_hand: None,
                    sacrifice_permanents: None,
            effect_override: None,
            dash: false,
            blitz: false,
            flash: false,
        }),
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Bloodchief's Thirst — {B} Sorcery, Kicker {2}{B}. Destroy target
/// creature or planeswalker with mana value ≤ 2; if kicked, destroy one of
/// any mana value (CR 702.32). The kicked branch's broader target filter
/// is enforced at cast time via kick-aware target validation.
pub fn bloodchiefs_thirst() -> CardDefinition {
    CardDefinition {
        name: "Bloodchief's Thirst",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Kicker(cost(&[generic(2), b()]))],
        effect: Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
            }),
            else_: Box::new(Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Planeswalker)
                        .and(SelectionRequirement::ManaValueAtMost(2)),
                ),
            }),
        },
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Cryptic Command — {1}{U}{U}{U} Instant. Choose two — counter target spell;
/// return target permanent to its owner's hand; tap all creatures your
/// opponents control; draw a card.
///
/// Faithful "choose two" via `Effect::ChooseN`: the four modes each own a
/// cast-time target slot in pick order (counter → slot 0, bounce → slot 1).
/// `picks: [0, 1]` is the AutoDecider default (counter + bounce, the most
/// reactive line); a UI/scripted decider can pick any two of the four.
pub fn cryptic_command() -> CardDefinition {
    CardDefinition {
        name: "Cryptic Command",
        cost: cost(&[generic(1), u(), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseN {
            picks: vec![0, 1],
            modes: vec![
                // Mode 0: counter target spell.
                Effect::CounterSpell {
                    what: target_filtered(SelectionRequirement::IsSpellOnStack),
                },
                // Mode 1: return target permanent to its owner's hand.
                Effect::Move {
                    what: target_filtered(SelectionRequirement::Permanent),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
                // Mode 2: tap all creatures your opponents control.
                Effect::ForEach {
                    selector: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent),
                    ),
                    body: Box::new(Effect::Tap { what: Selector::TriggerSource }),
                },
                // Mode 3: draw a card.
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ],
        },
        ..Default::default()
    }
}

/// Blossoming Defense — {G} Instant. Target creature you control gets +2/+2
/// and gains hexproof until end of turn.
pub fn blossoming_defense() -> CardDefinition {
    CardDefinition {
        name: "Blossoming Defense",
        cost: cost(&[g()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Treasure Cruise — {7}{U} Instant. Delve. Draw three cards.
///
/// Delve (CR 702.66) is wired via `Keyword::Delve` + the
/// `GameAction::CastSpellDelve` path: each graveyard card exiled while
/// casting pays {1} of the {7} generic, so a stocked graveyard turns this
/// into a one-mana "draw three". The "Draw 3" half is the resolved effect.
pub fn treasure_cruise() -> CardDefinition {
    CardDefinition {
        name: "Treasure Cruise",
        cost: cost(&[generic(7), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Delve],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Const(3),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Dig Through Time — {6}{U}{U} Instant. Delve. "Look at the top seven
/// cards of your library. Put two of them into your hand and the rest on
/// the bottom of your library in any order."
///
/// Delve (CR 702.66) is wired via `Keyword::Delve` — a full graveyard
/// turns this into {U}{U}. The selection half is approximated as
/// `Scry 7 → Draw 2`: scrying seven lets the controller see the top seven
/// and arrange them (the two keepers on top, the rest to the bottom),
/// then drawing two takes the keepers — gameplay-equivalent to the
/// printed "put two into your hand, the rest on the bottom". Same
/// approximation pattern as Stress Dream's "look at top 2, take 1".
pub fn dig_through_time() -> CardDefinition {
    CardDefinition {
        name: "Dig Through Time",
        cost: cost(&[generic(6), u(), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Delve],
        effect: Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(7),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Lose Focus — {U} Instant. Delve. Counter target spell unless its
/// controller pays {2}.
///
/// Delve (CR 702.66) is now wired via `Keyword::Delve`. The printed cost is
/// just {U}, so Delve has no generic to reduce on Lose Focus itself — but
/// the keyword is present for correctness (and for future graveyard-hate
/// payoffs that read it). Reuses `Effect::CounterUnlessPaid`.
pub fn lose_focus() -> CardDefinition {
    CardDefinition {
        name: "Lose Focus",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Delve],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(2)]),
        },
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterAbility {
            what: target_filtered(SelectionRequirement::Permanent),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Memory Lapse — {1}{U} Instant. Counter target spell. If countered this
/// way, put the spell on top of its owner's library instead of into the
/// graveyard.
///
/// Wired via `Effect::CounterSpellToZone { zone: OwnerLibraryTop }` (push
/// modern_decks): the on-stack card is lifted off the stack and placed on
/// top of its owner's library rather than routed to the graveyard. Matches
/// CR 701.5g's "instead" clause exactly.
pub fn memory_lapse() -> CardDefinition {
    use crate::effect::CounteredSpellZone;
    CardDefinition {
        name: "Memory Lapse",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterSpellToZone {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            zone: CounteredSpellZone::OwnerLibraryTop,
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Vines of Vastwood — {G} Instant, Kicker {G}{G}. Target creature gains
/// hexproof until end of turn (can't be targeted by opponents); if kicked,
/// it also gets +4/+4 EOT (CR 702.32).
pub fn vines_of_vastwood() -> CardDefinition {
    CardDefinition {
        name: "Vines of Vastwood",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Kicker(cost(&[g(), g()]))],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(4),
                    toughness: Value::Const(4),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
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

/// Gush — {4}{U} Instant. Draw two cards. Alternative cost: return two
/// Islands you control to their owner's hand (the free-draw line) via
/// `return_to_hand`.
pub fn gush() -> CardDefinition {
    use crate::card::{AlternativeCost, LandType};
    CardDefinition {
        name: "Gush",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Const(2),
        },
        alternative_cost: Some(AlternativeCost {
            return_to_hand: Some((SelectionRequirement::HasLandType(LandType::Island), 2)),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Intervention Pact — {0} Instant. "The next time a source of your
/// choice would deal damage to you this turn, prevent that damage.
/// You gain life equal to the damage prevented this way. At the
/// beginning of your next upkeep, pay {1}{W}{W}. If you don't, you
/// lose the game."
///
/// Approximation: the damage-prevention rider is collapsed to "you
/// gain 5 life" (a fixed value since the engine has no damage-
/// prevention shield). The delayed upkeep payment is faithfully wired
/// via `PayOrLoseGame`.
pub fn intervention_pact() -> CardDefinition {
    CardDefinition {
        name: "Intervention Pact",
        cost: ManaCost::new(vec![]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(5),
            },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::YourNextUpkeep,
                body: Box::new(Effect::PayOrLoseGame {
                    mana_cost: cost(&[generic(1), w(), w()]),
                    life_cost: 0,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Dissolve — {1}{U}{U} Instant. "Counter target spell. Scry 1."
pub fn dissolve() -> CardDefinition {
    CardDefinition {
        name: "Dissolve",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Flame Javelin — {2/R}{2/R}{2/R} Instant. "Flame Javelin deals 4 damage
/// to any target." Mono-hybrid pips wired as `{2/R}` each.
pub fn flame_javelin() -> CardDefinition {
    use crate::mana::mono_hybrid;
    CardDefinition {
        name: "Flame Javelin",
        cost: ManaCost::new(vec![
            mono_hybrid(2, Color::Red),
            mono_hybrid(2, Color::Red),
            mono_hybrid(2, Color::Red),
        ]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(4) },
        ..Default::default()
    }
}

/// Pongify — {U} Instant. "Destroy target creature. It can't be
/// regenerated. Its controller creates a 3/3 green Ape creature token."
pub fn pongify() -> CardDefinition {
    use crate::card::TokenDefinition;
    let ape = TokenDefinition {
        name: "Ape".into(),
        power: 3,
        toughness: 3,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ape],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    
        static_abilities: vec![],
    };
    CardDefinition {
        name: "Pongify",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            // Mint the Ape for the target's controller first, while the
            // creature is still on the battlefield.
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::Const(1),
                definition: ape,
            },
            Effect::DestroyNoRegen {
                what: target_filtered(SelectionRequirement::Creature),
            },
        ]),
        ..Default::default()
    }
}

/// Dark Banishing — {2}{B} Instant. "Destroy target nonblack creature. It
/// can't be regenerated." (various)
pub fn dark_banishing() -> CardDefinition {
    CardDefinition {
        name: "Dark Banishing",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DestroyNoRegen {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasColor(Color::Black).negate()),
            ),
        },
        ..Default::default()
    }
}

/// Grim Affliction — {2}{B} Instant. "Put a -1/-1 counter on target
/// creature. At the beginning of the next end step, proliferate."
pub fn grim_affliction() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Grim Affliction",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::MinusOneMinusOne,
                amount: Value::Const(1),
            },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::NextEndStep,
                body: Box::new(Effect::Proliferate),
            },
        ]),
        ..Default::default()
    }
}

/// Steady Progress — {2}{U} Instant. "Proliferate. Draw a card."
pub fn steady_progress() -> CardDefinition {
    CardDefinition {
        name: "Steady Progress",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Proliferate,
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Volt Charge — {2}{R} Instant. "Volt Charge deals 3 damage to any target.
/// Proliferate."
pub fn volt_charge() -> CardDefinition {
    CardDefinition {
        name: "Volt Charge",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(3) },
            Effect::Proliferate,
        ]),
        ..Default::default()
    }
}

/// Cackling Counterpart — {1}{U}{U} Instant. "Create a token that's a copy
/// of target creature you control. Flashback {5}{U}{U}." Token copy via
/// `Effect::CreateTokenCopyOf`; Flashback via `Keyword::Flashback`.
pub fn cackling_counterpart() -> CardDefinition {
    let flashback = ManaCost::new(vec![
        crate::mana::ManaSymbol::Generic(5),
        crate::mana::ManaSymbol::Colored(Color::Blue),
        crate::mana::ManaSymbol::Colored(Color::Blue),
    ]);
    CardDefinition {
        name: "Cackling Counterpart",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Flashback(flashback)],
        effect: Effect::CreateTokenCopyOf {
            who: PlayerRef::You,
            count: Value::Const(1),
            source: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            extra_creature_types: vec![],
            override_pt: None,
        },
        ..Default::default()
    }
}

/// Gods Willing — {W} Instant. "Target creature you control gains protection
/// from the color of your choice until end of turn. Scry 1." (THS)
pub fn gods_willing() -> CardDefinition {
    CardDefinition {
        name: "Gods Willing",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantProtectionFromChosenColor {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                duration: Duration::EndOfTurn,
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Mother of Runes — {W} Creature — Human Cleric 1/1. "{T}: Target creature
/// you control gains protection from the color of your choice until end of
/// turn." (Urza's Saga)
pub fn mother_of_runes() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Mother of Runes",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            tap_cost: true,
            effect: Effect::GrantProtectionFromChosenColor {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Brave the Elements — {W} Instant. "Creatures you control gain protection
/// from the color of your choice until end of turn." (M12)
pub fn brave_the_elements() -> CardDefinition {
    CardDefinition {
        name: "Brave the Elements",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::GrantProtectionFromChosenColor {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Apostle's Blessing — {1}{W/P} Instant. "Target artifact or creature you
/// control gains protection from artifacts or from the color of your choice
/// until end of turn." (Only the color-choice mode is modeled; the
/// "protection from artifacts" alternative isn't a single-keyword grant yet.)
pub fn apostles_blessing() -> CardDefinition {
    use crate::mana::ManaSymbol;
    CardDefinition {
        name: "Apostle's Blessing",
        cost: ManaCost {
            symbols: vec![ManaSymbol::Generic(1), ManaSymbol::Phyrexian(Color::White)],
        },
        card_types: vec![CardType::Instant],
        effect: Effect::GrantProtectionFromChosenColor {
            what: target_filtered(
                (SelectionRequirement::Artifact.or(SelectionRequirement::Creature))
                    .and(SelectionRequirement::ControlledByYou),
            ),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Victims of Night — {1}{B}{B} Instant. "Destroy target creature that's not a
/// Vampire, Werewolf, or Zombie." (DKA)
pub fn victims_of_night() -> CardDefinition {
    use crate::card::CreatureType;
    CardDefinition {
        name: "Victims of Night",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Vampire).negate())
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Werewolf).negate())
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Zombie).negate()),
            ),
        },
        ..Default::default()
    }
}

/// Vraska's Contempt — {2}{B}{B} Instant. "Exile target creature or
/// planeswalker. You gain 2 life." (XLN)
pub fn vraskas_contempt() -> CardDefinition {
    CardDefinition {
        name: "Vraska's Contempt",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Victim of Night — {B}{B} Instant. "Destroy target creature that isn't a
/// Vampire, Werewolf, or Zombie." (ISD)
pub fn victim_of_night() -> CardDefinition {
    CardDefinition {
        name: "Victim of Night",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.and(
                    SelectionRequirement::HasCreatureType(CreatureType::Vampire)
                        .or(SelectionRequirement::HasCreatureType(CreatureType::Werewolf))
                        .or(SelectionRequirement::HasCreatureType(CreatureType::Zombie))
                        .negate(),
                ),
            ),
        },
        ..Default::default()
    }
}

/// Trumpet Blast — {1}{R} Instant. "Attacking creatures get +2/+0 until end
/// of turn." (various)
pub fn trumpet_blast() -> CardDefinition {
    CardDefinition {
        name: "Trumpet Blast",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(SelectionRequirement::IsAttacking),
            power: Value::Const(2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}
