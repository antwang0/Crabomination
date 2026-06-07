use crate::card::{CardDefinition, CardType, Keyword, SelectionRequirement};
use crate::effect::shortcut::{awaken, deal, pump_target, return_target_to_hand, surge, target, target_filtered};
use crate::effect::{Effect, PlayerRef, Value};
use crate::mana::{ManaCost, b, colorless, cost, g, generic, r, u};
use crabomination_base::tokens::eldrazi_scion_token;

/// Murderous Compulsion — {1}{B} Sorcery. Destroy target tapped creature.
/// Madness {1}{B} (CR 702.35).
pub fn murderous_compulsion() -> CardDefinition {
    CardDefinition {
        name: "Murderous Compulsion",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Madness(ManaCost::new(vec![generic(1), b()]))],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::Tapped),
            ),
        },
        ..Default::default()
    }
}

/// Roil Spout — {1}{W}{U} Sorcery. Put target creature on top of its owner's
/// library. Awaken 4—{4}{W}{U} (land = target slot 1).
pub fn roil_spout() -> CardDefinition {
    use crate::effect::{LibraryPosition, ZoneDest};
    let base = Effect::Move {
        what: target_filtered(SelectionRequirement::Creature),
        to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Top },
    };
    CardDefinition {
        name: "Roil Spout",
        cost: cost(&[generic(1), crate::mana::w(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: base.clone(),
        alternative_cost: Some(awaken(4, cost(&[generic(4), crate::mana::w(), u()]), 1, base)),
        ..Default::default()
    }
}

/// Coastal Discovery — {3}{U} Sorcery. Draw two cards. Awaken 4—{5}{U}.
pub fn coastal_discovery() -> CardDefinition {
    use crate::effect::Selector;
    let base = Effect::Draw { who: Selector::You, amount: Value::Const(2) };
    CardDefinition {
        name: "Coastal Discovery",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Sorcery],
        effect: base.clone(),
        alternative_cost: Some(awaken(4, cost(&[generic(5), u()]), 0, base)),
        ..Default::default()
    }
}

/// Comparative Analysis — {3}{U} Instant. Target player draws two cards.
/// Surge {2}{U}.
pub fn comparative_analysis() -> CardDefinition {
    use crate::effect::Selector;
    CardDefinition {
        name: "Comparative Analysis",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Draw { who: Selector::Target(0), amount: Value::Const(2) },
        alternative_cost: Some(surge(cost(&[generic(2), u()]), false)),
        ..Default::default()
    }
}

/// Shoulder to Shoulder — {2}{W} Sorcery. Support 2, then draw a card.
pub fn shoulder_to_shoulder() -> CardDefinition {
    use crate::effect::Selector;
    CardDefinition {
        name: "Shoulder to Shoulder",
        cost: cost(&[generic(2), crate::mana::w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::SupportCounters { max_targets: 2, filter: SelectionRequirement::Creature },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Sheer Drop — {2}{W} Sorcery. Destroy target tapped creature.
/// Awaken 3—{5}{W} (land = target slot 1).
pub fn sheer_drop() -> CardDefinition {
    let base = Effect::Destroy {
        what: target_filtered(
            SelectionRequirement::Creature.and(SelectionRequirement::Tapped),
        ),
    };
    CardDefinition {
        name: "Sheer Drop",
        cost: cost(&[generic(2), crate::mana::w()]),
        card_types: vec![CardType::Sorcery],
        effect: base.clone(),
        alternative_cost: Some(awaken(3, cost(&[generic(5), crate::mana::w()]), 1, base)),
        ..Default::default()
    }
}

/// Mire's Malice — {3}{B} Sorcery. Target opponent discards two cards.
/// Awaken 3—{5}{B}.
pub fn mires_malice() -> CardDefinition {
    use crate::effect::Selector;
    let base = Effect::Discard {
        who: Selector::Player(PlayerRef::EachOpponent),
        amount: Value::Const(2),
        random: false,
    };
    CardDefinition {
        name: "Mire's Malice",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: base.clone(),
        alternative_cost: Some(awaken(3, cost(&[generic(5), b()]), 0, base)),
        ..Default::default()
    }
}

/// Allied Reinforcements — {3}{W} Sorcery. Create two 2/2 white Knight Ally tokens.
pub fn allied_reinforcements() -> CardDefinition {
    use crate::card::{CreatureType, Subtypes, TokenDefinition};
    use crabomination_base::mana::Color;
    let knight = TokenDefinition {
        name: "Knight".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Knight, CreatureType::Ally],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Allied Reinforcements",
        cost: cost(&[generic(3), crate::mana::w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: knight,
        },
        ..Default::default()
    }
}

/// Searing Light — {W} Instant. Destroy target attacking or blocking creature
/// with power 2 or less.
pub fn searing_light() -> CardDefinition {
    CardDefinition {
        name: "Searing Light",
        cost: cost(&[crate::mana::w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::PowerAtMost(2))
                    .and(
                        SelectionRequirement::IsAttacking.or(SelectionRequirement::IsBlocking),
                    ),
            ),
        },
        ..Default::default()
    }
}

/// Mutant's Prey — {G} Instant. Target creature you control with a +1/+1
/// counter on it fights target creature an opponent controls.
pub fn mutants_prey() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::Selector;
    CardDefinition {
        name: "Mutant's Prey",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Fight {
            attacker: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne)),
            },
            defender: Selector::TargetFiltered {
                slot: 1,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            },
        },
        ..Default::default()
    }
}

/// Corpse Churn — {1}{B} Instant. Mill three, then you may return a creature
/// card from your graveyard to your hand.
pub fn corpse_churn() -> CardDefinition {
    use crate::effect::Selector;
    CardDefinition {
        name: "Corpse Churn",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(3) },
            Effect::MayDo {
                description: "Return a creature card from your graveyard to your hand".into(),
                body: Box::new(Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        zone: crate::card::Zone::Graveyard,
                        who: PlayerRef::You,
                        filter: SelectionRequirement::Creature,
                    }),
                    to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Tears of Valakut — {1}{R} Instant. Can't be countered. Deals 5 damage to
/// target creature with flying.
pub fn tears_of_valakut() -> CardDefinition {
    CardDefinition {
        name: "Tears of Valakut",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::CantBeCountered],
        effect: deal(5, target_filtered(
            SelectionRequirement::Creature.and(SelectionRequirement::HasKeyword(Keyword::Flying)),
        )),
        ..Default::default()
    }
}

/// Sweep Away — {2}{U} Instant. Return target creature to its owner's hand.
/// (The attacking-only "put it on top of its library instead" rider is
/// dropped — modal post-bounce choice not modeled.)
pub fn sweep_away() -> CardDefinition {
    CardDefinition {
        name: "Sweep Away",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: return_target_to_hand(),
        ..Default::default()
    }
}

/// Warping Wail — {1}{C} Devoid Instant. Choose one — exile target creature
/// with power or toughness 1 or less; counter target sorcery; or create a
/// 1/1 Eldrazi Scion.
pub fn warping_wail() -> CardDefinition {
    CardDefinition {
        name: "Warping Wail",
        cost: cost(&[generic(1), colorless(1)]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        effect: Effect::ChooseMode(vec![
            Effect::Exile {
                what: target_filtered(SelectionRequirement::Creature.and(
                    SelectionRequirement::PowerAtMost(1).or(SelectionRequirement::ToughnessAtMost(1)),
                )),
            },
            Effect::CounterSpell {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack
                        .and(SelectionRequirement::HasCardType(CardType::Sorcery)),
                ),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: eldrazi_scion_token(),
            },
        ]),
        ..Default::default()
    }
}

/// Tar Snare — {2}{B} Devoid Instant. Target creature gets -3/-2 EOT.
pub fn tar_snare() -> CardDefinition {
    CardDefinition {
        name: "Tar Snare",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        effect: pump_target(-3, -2),
        ..Default::default()
    }
}

/// Witness the End — {3}{B} Devoid Sorcery. Target opponent loses 2 life
/// and discards two cards. (The printed "exiles two cards from hand" is
/// approximated as a discard — no exile-from-hand-by-the-owner primitive.)
pub fn witness_the_end() -> CardDefinition {
    use crate::effect::Selector;
    CardDefinition {
        name: "Witness the End",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Devoid],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
                random: false,
            },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Kozilek's Return — {2}{R} Devoid Instant. Deals 2 damage to each creature.
/// (The graveyard-recur rider on casting a 7+ MV Eldrazi is dropped.)
pub fn kozileks_return() -> CardDefinition {
    use crate::effect::Selector;
    CardDefinition {
        name: "Kozilek's Return",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::DealDamage {
                to: Selector::TriggerSource,
                amount: Value::Const(2),
            }),
        },
        ..Default::default()
    }
}

/// Scour from Existence — {7} Devoid Instant. Exile target permanent.
pub fn scour_from_existence() -> CardDefinition {
    CardDefinition {
        name: "Scour from Existence",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Instant],
        effect: Effect::Exile { what: target_filtered(SelectionRequirement::Permanent) },
        ..Default::default()
    }
}

/// Oblivion Strike — {3}{B} Devoid Sorcery. Exile target creature.
pub fn oblivion_strike() -> CardDefinition {
    CardDefinition {
        name: "Oblivion Strike",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Devoid],
        effect: Effect::Exile { what: target_filtered(SelectionRequirement::Creature) },
        ..Default::default()
    }
}

/// Complete Disregard — {2}{B} Devoid Instant. Exile target creature with
/// power 3 or less.
pub fn complete_disregard() -> CardDefinition {
    CardDefinition {
        name: "Complete Disregard",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(3)),
            ),
        },
        ..Default::default()
    }
}

/// Spatial Contortion — {1}{C} Instant. Target creature gets +3/-3 EOT.
pub fn spatial_contortion() -> CardDefinition {
    CardDefinition {
        name: "Spatial Contortion",
        cost: cost(&[generic(1), colorless(1)]),
        card_types: vec![CardType::Instant],
        effect: pump_target(3, -3),
        ..Default::default()
    }
}

/// Unnatural Endurance — {B} Devoid Instant. Target creature gets +2/+0
/// until end of turn and is regenerated.
pub fn unnatural_endurance() -> CardDefinition {
    CardDefinition {
        name: "Unnatural Endurance",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        effect: Effect::Seq(vec![
            pump_target(2, 0),
            Effect::Regenerate { what: target() },
        ]),
        ..Default::default()
    }
}

/// Call the Scions — {2}{G} Devoid Sorcery. Create two 1/1 Eldrazi Scions.
pub fn call_the_scions() -> CardDefinition {
    CardDefinition {
        name: "Call the Scions",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Devoid],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: eldrazi_scion_token(),
        },
        ..Default::default()
    }
}

/// Reality Hemorrhage — {1}{R} Devoid Instant. Deals 2 damage to any target.
pub fn reality_hemorrhage() -> CardDefinition {
    CardDefinition {
        name: "Reality Hemorrhage",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        effect: deal(2, target()),
        ..Default::default()
    }
}

/// Touch of the Void — {2}{R} Devoid Sorcery. Deals 3 damage to any target.
/// (The "if a creature dies this turn, exile it" rider is dropped.)
pub fn touch_of_the_void() -> CardDefinition {
    CardDefinition {
        name: "Touch of the Void",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Devoid],
        effect: deal(3, target()),
        ..Default::default()
    }
}

/// Natural State — {G} Instant. Destroy target artifact or enchantment with
/// mana value 3 or less.
pub fn natural_state() -> CardDefinition {
    CardDefinition {
        name: "Natural State",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact
                    .or(SelectionRequirement::Enchantment)
                    .and(SelectionRequirement::ManaValueAtMost(3)),
            ),
        },
        ..Default::default()
    }
}

/// Make a Stand — {2}{W} Instant. Creatures you control get +1/+0 and gain
/// indestructible until end of turn.
pub fn make_a_stand() -> CardDefinition {
    use crate::effect::shortcut::each_your_creature;
    use crate::effect::{Duration, Selector};
    CardDefinition {
        name: "Make a Stand",
        cost: cost(&[generic(2), crate::mana::w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ForEach {
                selector: each_your_creature(),
                body: Box::new(Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                }),
            },
            Effect::GrantKeyword {
                what: each_your_creature(),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Flaying Tendrils — {1}{B}{B} Devoid Sorcery. All creatures get -2/-2 until
/// end of turn; if a creature would die this turn, exile it instead.
pub fn flaying_tendrils() -> CardDefinition {
    use crate::effect::{Duration, Selector};
    CardDefinition {
        name: "Flaying Tendrils",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Devoid],
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn {
                what: Selector::EachPermanent(SelectionRequirement::Creature),
            },
            Effect::ForEach {
                selector: Selector::EachPermanent(SelectionRequirement::Creature),
                body: Box::new(Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(-2),
                    toughness: Value::Const(-2),
                    duration: Duration::EndOfTurn,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Mighty Leap — {1}{W} Instant. Target creature gets +2/+2 and gains flying
/// until end of turn.
pub fn mighty_leap() -> CardDefinition {
    use crate::effect::shortcut::target;
    use crate::effect::Duration;
    CardDefinition {
        name: "Mighty Leap",
        cost: cost(&[generic(1), crate::mana::w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            pump_target(2, 2),
            Effect::GrantKeyword {
                what: target(),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Boulder Salvo — {4}{R} Sorcery. Deals 4 damage to target creature.
/// Surge {1}{R}.
pub fn boulder_salvo() -> CardDefinition {
    use crate::effect::shortcut::target;
    CardDefinition {
        name: "Boulder Salvo",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage { to: target(), amount: Value::Const(4) },
        alternative_cost: Some(surge(cost(&[generic(1), r()]), false)),
        ..Default::default()
    }
}

/// Devour in Flames — {2}{R} Sorcery. Additional cost: return a land you
/// control to its owner's hand. Deals 5 damage to target creature or
/// planeswalker.
pub fn devour_in_flames() -> CardDefinition {
    use crate::card::AdditionalCastCost;
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Devour in Flames",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::ReturnToHand {
            filter: SelectionRequirement::Land,
            count: 1,
        }],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(5),
        },
        ..Default::default()
    }
}
