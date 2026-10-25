//! Commander: the 2026-10-17 (`01Htv6cF`) frequency scan's short cards — the
//! missing cards of every unseated one- to three-card-gap deck, shortest
//! oracle text first. Tests in `tests/recent_b/cmdr_oct17_g.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect,
    Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, on_dies, target_any, target_filtered};
use crate::effect::{DelayedTriggerKind, Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w, x};
use crate::sets::{enters_tapped, etb_gain_one, tapland_untyped};
use std::sync::Arc;

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

fn kw(keywords: Vec<Keyword>, def: CardDefinition) -> CardDefinition {
    CardDefinition { keywords, ..def }
}

fn artifact_creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { card_types: vec![CardType::Artifact, CardType::Creature], ..creature(name, mana, types, p, t) }
}

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn aura(name: &'static str, mana: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

fn equipment(name: &'static str, mana: ManaCost, equip: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

fn act(mana: ManaCost, tap: bool, effect: Effect) -> ActivatedAbility {
    ActivatedAbility { mana_cost: mana, tap_cost: tap, effect, ..Default::default() }
}

fn pump(what: Selector, p: i32, t: i32) -> Effect {
    Effect::PumpPT { what, power: Value::Const(p), toughness: Value::Const(t), duration: Duration::EndOfTurn }
}

fn grant(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn gain(n: i32) -> Effect {
    Effect::GainLife { who: Selector::You, amount: Value::Const(n) }
}

fn mine(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn count_mine(filter: R) -> Value {
    Value::count(mine(filter))
}

fn token(name: &str, p: i32, t: i32, colors: Vec<Color>, types: Vec<CreatureType>, keywords: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords,
        ..Default::default()
    })
}

fn mint(n: i32, def: Arc<TokenDefinition>) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: def }
}

fn tap_any_color() -> ActivatedAbility {
    act(cost(&[]), true, Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyColors(Value::ONE) })
}

fn bounce(what: Selector) -> Effect {
    Effect::Move { what, to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))) }
}

fn spell_of_color(colors: &[Color]) -> R {
    colors.iter().map(|c| R::HasColor(*c)).reduce(|a, b| a.or(b)).unwrap_or(R::Any).and(R::IsSpellOnStack)
}

// ── Bodies ─────────────────────────────────────────────────────────────────

/// Armored Whirl Turtle — {2}{U} 0/5 Turtle.
pub fn armored_whirl_turtle() -> CardDefinition {
    creature("Armored Whirl Turtle", cost(&[generic(2), u()]), vec![CreatureType::Turtle], 0, 5)
}

/// Cylian Elf — {1}{G} 2/2 Elf Scout.
pub fn cylian_elf() -> CardDefinition {
    creature("Cylian Elf", cost(&[generic(1), g()]), vec![CreatureType::Elf, CreatureType::Scout], 2, 2)
}

/// Rip-Clan Crasher — {R}{G} 2/2 haste Human Warrior.
pub fn rip_clan_crasher() -> CardDefinition {
    kw(vec![Keyword::Haste], creature("Rip-Clan Crasher", cost(&[r(), g()]), vec![CreatureType::Human, CreatureType::Warrior], 2, 2))
}

/// Chardalyn Dragon — {6} 4/4 flying artifact Dragon.
pub fn chardalyn_dragon() -> CardDefinition {
    kw(vec![Keyword::Flying], artifact_creature("Chardalyn Dragon", cost(&[generic(6)]), vec![CreatureType::Dragon], 4, 4))
}

/// Pardic Wanderer — {6} 5/5 trample artifact Golem.
pub fn pardic_wanderer() -> CardDefinition {
    kw(vec![Keyword::Trample], artifact_creature("Pardic Wanderer", cost(&[generic(6)]), vec![CreatureType::Golem], 5, 5))
}

/// Seraph of Dawn — {2}{W}{W} 2/4 flying, lifelink Angel.
pub fn seraph_of_dawn() -> CardDefinition {
    kw(vec![Keyword::Flying, Keyword::Lifelink], creature("Seraph of Dawn", cost(&[generic(2), w(), w()]), vec![CreatureType::Angel], 2, 4))
}

/// Oakgnarl Warrior — {5}{G}{G} 5/7 vigilance, trample Treefolk Warrior.
pub fn oakgnarl_warrior() -> CardDefinition {
    kw(
        vec![Keyword::Vigilance, Keyword::Trample],
        creature("Oakgnarl Warrior", cost(&[generic(5), g(), g()]), vec![CreatureType::Treefolk, CreatureType::Warrior], 5, 7),
    )
}

/// Phantom Ninja — {1}{U}{U} 2/2 unblockable Illusion Ninja.
pub fn phantom_ninja() -> CardDefinition {
    kw(vec![Keyword::Unblockable], creature("Phantom Ninja", cost(&[generic(1), u(), u()]), vec![CreatureType::Illusion, CreatureType::Ninja], 2, 2))
}

/// Frog-Squirrels — {1}{G} 2/2 reach Frog Squirrel.
pub fn frog_squirrels() -> CardDefinition {
    kw(vec![Keyword::Reach], creature("Frog-Squirrels", cost(&[generic(1), g()]), vec![CreatureType::Frog, CreatureType::Squirrel], 2, 2))
}

/// Aardvark Sloth — {3}{W} 3/3 lifelink Sloth Beast.
pub fn aardvark_sloth() -> CardDefinition {
    kw(vec![Keyword::Lifelink], creature("Aardvark Sloth", cost(&[generic(3), w()]), vec![CreatureType::Sloth, CreatureType::Beast], 3, 3))
}

/// Rampaging Hippo — {4}{G}{G} 5/6 trample Hippo. Cycling {2}.
pub fn rampaging_hippo() -> CardDefinition {
    kw(
        vec![Keyword::Trample, Keyword::Cycling(cost(&[generic(2)]))],
        creature("Rampaging Hippo", cost(&[generic(4), g(), g()]), vec![CreatureType::Hippo], 5, 6),
    )
}

/// Angel of the God-Pharaoh — {4}{W}{W} 4/4 flying Angel. Cycling {2}.
pub fn angel_of_the_god_pharaoh() -> CardDefinition {
    kw(
        vec![Keyword::Flying, Keyword::Cycling(cost(&[generic(2)]))],
        creature("Angel of the God-Pharaoh", cost(&[generic(4), w(), w()]), vec![CreatureType::Angel], 4, 4),
    )
}

/// Anaconda — {3}{G} 3/3 Snake. Swampwalk.
pub fn anaconda() -> CardDefinition {
    kw(vec![Keyword::Landwalk(LandType::Swamp)], creature("Anaconda", cost(&[generic(3), g()]), vec![CreatureType::Snake], 3, 3))
}

/// Woodland Mystic — {1}{G} 1/1 Elf Druid. {T}: add {G}.
pub fn woodland_mystic() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![crate::sets::tap_add(Color::Green)],
        ..creature("Woodland Mystic", cost(&[generic(1), g()]), vec![CreatureType::Elf, CreatureType::Druid], 1, 1)
    }
}

/// Xenobotanist — {1}{G} 1/3 Vulcan Scientist. {T}: add one mana of any color.
pub fn xenobotanist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![tap_any_color()],
        ..creature("Xenobotanist", cost(&[generic(1), g()]), vec![CreatureType::Vulcan, CreatureType::Scientist], 1, 3)
    }
}

/// Signpost Scarecrow — {4} 2/4 vigilance artifact Scarecrow. {2}: one mana of
/// any color.
pub fn signpost_scarecrow() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![act(cost(&[generic(2)]), false, Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyColors(Value::ONE) })],
        ..artifact_creature("Signpost Scarecrow", cost(&[generic(4)]), vec![CreatureType::Scarecrow], 2, 4)
    }
}

/// Transdimensional Bovine — {2}{G} 0/4 flying Ox Avatar. {T}: two mana of any
/// one color.
pub fn transdimensional_bovine() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![act(cost(&[]), true, Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(2)) })],
        ..creature("Transdimensional Bovine", cost(&[generic(2), g()]), vec![CreatureType::Ox, CreatureType::Avatar], 0, 4)
    }
}

/// Third Path Savant — {2}{U} 2/3 Human Wizard. {7}: draw two cards.
pub fn third_path_savant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![act(cost(&[generic(7)]), false, draw(2))],
        ..creature("Third Path Savant", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 3)
    }
}

/// Tymora's Invoker — {1}{U} 1/3 Orc Rogue. {8}: draw two cards.
pub fn tymoras_invoker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![act(cost(&[generic(8)]), false, draw(2))],
        ..creature("Tymora's Invoker", cost(&[generic(1), u()]), vec![CreatureType::Orc, CreatureType::Rogue], 1, 3)
    }
}

/// Oracle of Nectars — {2}{G/W} 2/2 Elf Cleric. {X}, {T}: gain X life.
pub fn oracle_of_nectars() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![act(cost(&[x()]), true, Effect::GainLife { who: Selector::You, amount: Value::XFromCost })],
        ..creature("Oracle of Nectars", cost(&[generic(2), hybrid(Color::Green, Color::White)]), vec![CreatureType::Elf, CreatureType::Cleric], 2, 2)
    }
}

/// Akroan Jailer — {W} 1/1 Human Soldier. {2}{W}, {T}: tap target creature.
pub fn akroan_jailer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![act(cost(&[generic(2), w()]), true, Effect::Tap { what: target_filtered(R::Creature) })],
        ..creature("Akroan Jailer", cost(&[w()]), vec![CreatureType::Human, CreatureType::Soldier], 1, 1)
    }
}

/// Metallurgeon — {1}{W} 1/2 artifact Human Artificer. {W}, {T}: regenerate
/// target artifact.
pub fn metallurgeon() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![act(cost(&[w()]), true, Effect::Regenerate { what: target_filtered(R::Artifact) })],
        ..artifact_creature("Metallurgeon", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Artificer], 1, 2)
    }
}

/// Magus of the Bazaar — {1}{U} 0/1 Human Wizard. {T}: draw two, discard three.
pub fn magus_of_the_bazaar() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![act(
            cost(&[]),
            true,
            Effect::Seq(vec![draw(2), Effect::Discard { who: Selector::You, amount: Value::Const(3), random: false }]),
        )],
        ..creature("Magus of the Bazaar", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Wizard], 0, 1)
    }
}

/// Stonybrook Angler — {1}{U} 1/2 Merfolk Wizard. {1}{U}, {T}: tap or untap
/// target creature.
pub fn stonybrook_angler() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![act(cost(&[generic(1), u()]), true, Effect::TapOrUntap { what: target_filtered(R::Creature) })],
        ..creature("Stonybrook Angler", cost(&[generic(1), u()]), vec![CreatureType::Merfolk, CreatureType::Wizard], 1, 2)
    }
}

/// Frilled Sea Serpent — {4}{U}{U} 4/6 Serpent. {5}{U}{U}: can't be blocked
/// this turn.
pub fn frilled_sea_serpent() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![act(cost(&[generic(5), u(), u()]), false, grant(Selector::This, Keyword::Unblockable))],
        ..creature("Frilled Sea Serpent", cost(&[generic(4), u(), u()]), vec![CreatureType::Serpent], 4, 6)
    }
}

/// Ember-Eye Wolf — {1}{R} 1/2 haste Wolf. {1}{R}: +2/+0.
pub fn ember_eye_wolf() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![act(cost(&[generic(1), r()]), false, pump(Selector::This, 2, 0))],
        ..creature("Ember-Eye Wolf", cost(&[generic(1), r()]), vec![CreatureType::Wolf], 1, 2)
    }
}

/// Firehoof Cavalry — {W} 1/1 Human Berserker. {3}{R}: +2/+0 and trample.
pub fn firehoof_cavalry() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![act(
            cost(&[generic(3), r()]),
            false,
            Effect::Seq(vec![pump(Selector::This, 2, 0), grant(Selector::This, Keyword::Trample)]),
        )],
        ..creature("Firehoof Cavalry", cost(&[w()]), vec![CreatureType::Human, CreatureType::Berserker], 1, 1)
    }
}

/// Tarpan — {G} 1/1 Horse. Dies: gain 1 life.
pub fn tarpan() -> CardDefinition {
    CardDefinition { triggered_abilities: vec![on_dies(gain(1))], ..creature("Tarpan", cost(&[g()]), vec![CreatureType::Horse], 1, 1) }
}

/// Etched Familiar — {2}{B} 3/2 artifact Phyrexian Fox. Dies: each opponent
/// loses 2 life and you gain 2.
pub fn etched_familiar() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(crate::effect::shortcut::drain(2))],
        ..artifact_creature("Etched Familiar", cost(&[generic(2), b()]), vec![CreatureType::Phyrexian, CreatureType::Fox], 3, 2)
    }
}

/// Symbiote Spawn — {2}{B} 3/2 Symbiote Villain. Dies: drain each opponent 2.
pub fn symbiote_spawn() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(crate::effect::shortcut::drain(2))],
        ..creature("Symbiote Spawn", cost(&[generic(2), b()]), vec![CreatureType::Symbiote, CreatureType::Villain], 3, 2)
    }
}

/// Soaring Show-Off — {2}{U} 2/2 flying Bird Warrior. ETB: each player draws.
pub fn soaring_show_off() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE })],
        ..creature("Soaring Show-Off", cost(&[generic(2), u()]), vec![CreatureType::Bird, CreatureType::Warrior], 2, 2)
    }
}

/// Ferocious Pup — {2}{G} 0/1 Wolf. ETB: a 2/2 green Wolf token.
pub fn ferocious_pup() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(mint(1, token("Wolf", 2, 2, vec![Color::Green], vec![CreatureType::Wolf], vec![])))],
        ..creature("Ferocious Pup", cost(&[generic(2), g()]), vec![CreatureType::Wolf], 0, 1)
    }
}

/// Bazaar Trademage — {2}{U} 3/4 flying Human Wizard. ETB: draw two, discard
/// three.
pub fn bazaar_trademage() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            draw(2),
            Effect::Discard { who: Selector::You, amount: Value::Const(3), random: false },
        ]))],
        ..creature("Bazaar Trademage", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Wizard], 3, 4)
    }
}

/// Archaeomender — {2}{U} 2/3 Human Wizard. ETB: return target artifact card
/// from your graveyard to your hand.
pub fn archaeomender() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::HasCardType(CardType::Artifact).and(R::InYourGraveyard)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..creature("Archaeomender", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 3)
    }
}

/// Thopter Architect — {3}{W} 2/3 Human Artificer. Whenever an artifact you
/// control enters, target creature gains flying until end of turn.
pub fn thopter_architect() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact }),
            effect: grant(target_filtered(R::Creature), Keyword::Flying),
        }],
        ..creature("Thopter Architect", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Artificer], 2, 3)
    }
}

/// Chief of the Edge — {W}{B} 3/2 Human Warrior. Other Warriors you control
/// get +1/+0.
pub fn chief_of_the_edge() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Warrior creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: mine(R::Creature.and(R::HasCreatureType(CreatureType::Warrior)).and(R::OtherThanSource)),
                power: 1,
                toughness: 0,
            },
        }],
        ..creature("Chief of the Edge", cost(&[w(), b()]), vec![CreatureType::Human, CreatureType::Warrior], 3, 2)
    }
}

/// Assault Drone — {3}{B} 2/4 artifact Borg. Artifact creatures you control
/// get +1/+0.
pub fn assault_drone() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Artifact creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT { applies_to: mine(R::Creature.and(R::Artifact)), power: 1, toughness: 0 },
        }],
        ..artifact_creature("Assault Drone", cost(&[generic(3), b()]), vec![CreatureType::Borg], 2, 4)
    }
}

/// Wanderbrine Preacher — {1}{W} 2/2 Merfolk Cleric. Whenever it becomes
/// tapped, gain 2 life.
pub fn wanderbrine_preacher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility { event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource), effect: gain(2) }],
        ..creature("Wanderbrine Preacher", cost(&[generic(1), w()]), vec![CreatureType::Merfolk, CreatureType::Cleric], 2, 2)
    }
}

/// Gearsmith Prodigy — {U} 1/2 Human Artificer. +1/+0 while you control an
/// artifact.
pub fn gearsmith_prodigy() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+0 as long as you control an artifact.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorExists(Selector::ControlledBy { who: PlayerRef::You, filter: R::Artifact }),
                power: 1,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..creature("Gearsmith Prodigy", cost(&[u()]), vec![CreatureType::Human, CreatureType::Artificer], 1, 2)
    }
}

/// Flutterfox — {1}{W} 2/2 Fox. Flying while you control an artifact or
/// enchantment.
pub fn flutterfox() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as you control an artifact or enchantment, this creature has flying.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorExists(Selector::ControlledBy { who: PlayerRef::You, filter: R::Artifact.or(R::Enchantment) }),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Flying],
            },
        }],
        ..creature("Flutterfox", cost(&[generic(1), w()]), vec![CreatureType::Fox], 2, 2)
    }
}

/// God-Pharaoh's Faithful — {W} 0/4 Human Wizard. Whenever you cast a blue,
/// black, or red spell, gain 1 life.
pub fn god_pharaohs_faithful() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasColor(Color::Blue).or(R::HasColor(Color::Black)).or(R::HasColor(Color::Red)),
            }),
            effect: gain(1),
        }],
        ..creature("God-Pharaoh's Faithful", cost(&[w()]), vec![CreatureType::Human, CreatureType::Wizard], 0, 4)
    }
}

/// Drowner of Secrets — {2}{U} 1/3 Merfolk Wizard. Tap an untapped Merfolk you
/// control: target player mills a card.
pub fn drowner_of_secrets() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_others_cost: Some((R::HasCreatureType(CreatureType::Merfolk).and(R::Untapped), 1)),
            effect: Effect::Mill { who: target_filtered(R::Player), amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Drowner of Secrets", cost(&[generic(2), u()]), vec![CreatureType::Merfolk, CreatureType::Wizard], 1, 3)
    }
}

/// "{T}: You may put a land card from your hand onto the battlefield" (Llanowar
/// Scout, Scaled Herbalist).
fn land_from_hand() -> ActivatedAbility {
    act(
        cost(&[]),
        true,
        Effect::PutFromHandOntoBattlefield {
            who: PlayerRef::You,
            filter: R::Land,
            count: Value::ONE,
            tapped: false,
            haste: false,
            sacrifice_eot: false,
            return_eot: false,
            then: None,
        },
    )
}

/// Llanowar Scout — {1}{G} 1/3 Elf Scout. {T}: you may put a land card from
/// your hand onto the battlefield.
pub fn llanowar_scout() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![land_from_hand()],
        ..creature("Llanowar Scout", cost(&[generic(1), g()]), vec![CreatureType::Elf, CreatureType::Scout], 1, 3)
    }
}

/// Scaled Herbalist — {1}{G} 1/3 Lizard Druid. Same ability.
pub fn scaled_herbalist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![land_from_hand()],
        ..creature("Scaled Herbalist", cost(&[generic(1), g()]), vec![CreatureType::Lizard, CreatureType::Druid], 1, 3)
    }
}

/// Trailblazing Historian — {1}{R} 1/3 haste Human Shaman. {T}: another target
/// creature gains haste until end of turn.
pub fn trailblazing_historian() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![act(cost(&[]), true, grant(target_filtered(R::Creature.and(R::OtherThanSource)), Keyword::Haste))],
        ..creature("Trailblazing Historian", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Shaman], 1, 3)
    }
}

/// Niblis of the Urn — {1}{W} 1/1 flying Spirit. Attacks: you may tap target
/// creature.
pub fn niblis_of_the_urn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::ApplyToTargets {
            min_targets: 0,
            max_targets: 1,
            filter: R::Creature,
            effect: Box::new(Effect::Tap { what: Selector::Target(0) }),
        })],
        ..creature("Niblis of the Urn", cost(&[generic(1), w()]), vec![CreatureType::Spirit], 1, 1)
    }
}

/// Warteye Witch — {2}{B} 3/2 Goblin Shaman. Whenever it or another creature
/// you control dies, scry 1.
pub fn warteye_witch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        }],
        ..creature("Warteye Witch", cost(&[generic(2), b()]), vec![CreatureType::Goblin, CreatureType::Shaman], 3, 2)
    }
}

/// Silkbind Faerie — {2}{W/U} 1/3 flying Faerie Rogue. {1}{W/U}, {Q}: tap
/// target creature.
pub fn silkbind_faerie() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), hybrid(Color::White, Color::Blue)]),
            untap_self_cost: true,
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..creature("Silkbind Faerie", cost(&[generic(2), hybrid(Color::White, Color::Blue)]), vec![CreatureType::Faerie, CreatureType::Rogue], 1, 3)
    }
}

/// Aegis Automaton — {2} 0/3 artifact Construct. {4}{W}: return another target
/// creature you control to its owner's hand.
pub fn aegis_automaton() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![act(
            cost(&[generic(4), w()]),
            false,
            bounce(target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource))),
        )],
        ..artifact_creature("Aegis Automaton", cost(&[generic(2)]), vec![CreatureType::Construct], 0, 3)
    }
}

/// Ferrovore — {2}{R} 2/2 Beast. {R}, sacrifice an artifact: +3/+0.
pub fn ferrovore() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_other_filter: Some((R::Artifact, 1)),
            effect: pump(Selector::This, 3, 0),
            ..Default::default()
        }],
        ..creature("Ferrovore", cost(&[generic(2), r()]), vec![CreatureType::Beast], 2, 2)
    }
}

/// Skirsdag Flayer — {1}{B} 1/1 Human Cleric. {3}{B}, {T}, sacrifice a Human:
/// destroy target creature.
pub fn skirsdag_flayer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            tap_cost: true,
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Human), 1)),
            sac_other_may_be_source: true,
            effect: Effect::Destroy { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..creature("Skirsdag Flayer", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Cleric], 1, 1)
    }
}

/// Weirding Shaman — {1}{B} 2/1 Goblin Shaman. {3}{B}, sacrifice a Goblin: two
/// 1/1 black Goblin Rogues.
pub fn weirding_shaman() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Goblin), 1)),
            sac_other_may_be_source: true,
            effect: mint(2, token("Goblin Rogue", 1, 1, vec![Color::Black], vec![CreatureType::Goblin, CreatureType::Rogue], vec![])),
            ..Default::default()
        }],
        ..creature("Weirding Shaman", cost(&[generic(1), b()]), vec![CreatureType::Goblin, CreatureType::Shaman], 2, 1)
    }
}

/// Dauntless Aven — {2}{W} 2/1 flying Bird Warrior. Attacks: untap target
/// creature you control.
pub fn dauntless_aven() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::Untap { what: target_filtered(R::Creature.and(R::ControlledByYou)), up_to: None })],
        ..creature("Dauntless Aven", cost(&[generic(2), w()]), vec![CreatureType::Bird, CreatureType::Warrior], 2, 1)
    }
}

/// Swaggering Corsair — {2}{R} 2/2 Human Pirate. Raid — enters with a +1/+1
/// counter if you attacked this turn.
pub fn swaggering_corsair() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, crate::effect::shortcut::raid_value(1))),
        ..creature("Swaggering Corsair", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Pirate], 2, 2)
    }
}

/// Dreaded Bat-Cloud — {4}{B} 4/2 flying, deathtouch Bat. Costs {3} less if a
/// creature died this turn.
pub fn dreaded_bat_cloud() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {3} less to cast if a creature died this turn.",
            effect: StaticEffect::SelfCostReducedIfCreatureDiedThisTurn { amount: 3 },
        }],
        ..creature("Dreaded Bat-Cloud", cost(&[generic(4), b()]), vec![CreatureType::Bat], 4, 2)
    }
}

/// Malakir Cullblade — {1}{B} 1/1 Vampire Warrior. Whenever a creature an
/// opponent controls dies, a +1/+1 counter on it.
pub fn malakir_cullblade() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature("Malakir Cullblade", cost(&[generic(1), b()]), vec![CreatureType::Vampire, CreatureType::Warrior], 1, 1)
    }
}

/// Crystalline Entity — {8} 7/7 flying artifact Elemental. ETB, if you cast
/// it: destroy all nonartifact creatures.
pub fn crystalline_entity() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource).with_filter(Predicate::SourceWasCast),
            effect: Effect::Destroy { what: Selector::EachPermanent(R::Creature.and(R::Artifact.negate())) },
        }],
        ..artifact_creature("Crystalline Entity", cost(&[generic(8)]), vec![CreatureType::Elemental], 7, 7)
    }
}

/// Fourth Bridge Prowler — {B} 1/1 Human Rogue. ETB: you may have target
/// creature get -1/-1 until end of turn.
pub fn fourth_bridge_prowler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            min_targets: 0,
            max_targets: 1,
            filter: R::Creature,
            effect: Box::new(pump(Selector::Target(0), -1, -1)),
        })],
        ..creature("Fourth Bridge Prowler", cost(&[b()]), vec![CreatureType::Human, CreatureType::Rogue], 1, 1)
    }
}

/// Relentless Drednok — {B} 1/1 lifelink artifact Robot. {2}{B}, exile it from
/// your graveyard: draw a card, lose 1 life.
pub fn relentless_drednok() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: Effect::Seq(vec![draw(1), Effect::LoseLife { who: Selector::You, amount: Value::ONE }]),
            ..Default::default()
        }],
        ..artifact_creature("Relentless Drednok", cost(&[b()]), vec![CreatureType::Robot], 1, 1)
    }
}

// ── Noncreature permanents ─────────────────────────────────────────────────

/// Ogre's Cleaver — {2} Equipment, equip {5}. +5/+0.
pub fn ogres_cleaver() -> CardDefinition {
    equipment("Ogre's Cleaver", cost(&[generic(2)]), cost(&[generic(5)]), EquipBonus { power: 5, ..Default::default() })
}

/// Tormentor's Trident — {2} Equipment, equip {3}. +3/+0; attacks each combat
/// if able.
pub fn tormentors_trident() -> CardDefinition {
    equipment(
        "Tormentor's Trident",
        cost(&[generic(2)]),
        cost(&[generic(3)]),
        EquipBonus { power: 3, keywords: vec![Keyword::MustAttack], ..Default::default() },
    )
}

/// Arcane Flight — {U} Aura. +1/+1 and flying.
pub fn arcane_flight() -> CardDefinition {
    aura("Arcane Flight", cost(&[u()]), EquipBonus { power: 1, toughness: 1, keywords: vec![Keyword::Flying], ..Default::default() })
}

/// Asha's Favor — {2}{W} Aura. Flying, first strike, vigilance.
pub fn ashas_favor() -> CardDefinition {
    aura(
        "Asha's Favor",
        cost(&[generic(2), w()]),
        EquipBonus { keywords: vec![Keyword::Flying, Keyword::FirstStrike, Keyword::Vigilance], ..Default::default() },
    )
}

/// Arachnoform — {1}{G} Aura. +2/+2, reach, and every creature type.
pub fn arachnoform() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![crate::card::StaticAbility {
            description: "Enchanted creature is every creature type.",
            effect: crate::effect::StaticEffect::EveryCreatureType {
                applies_to: crate::effect::Selector::AttachedTo(Box::new(crate::effect::Selector::This)),
            },
        }],
        ..aura(
            "Arachnoform",
            cost(&[generic(1), g()]),
            EquipBonus { power: 2, toughness: 2, keywords: vec![Keyword::Reach], ..Default::default() },
        )
    }
}

/// Castle / Builder's Blessing — untapped creatures you control get +0/+2.
fn untapped_wall(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Untapped creatures you control get +0/+2.",
            effect: StaticEffect::PumpPT { applies_to: mine(R::Creature.and(R::Untapped)), power: 0, toughness: 2 },
        }],
        ..enchantment(name, mana)
    }
}

/// Castle — {3}{W} Enchantment.
pub fn castle() -> CardDefinition {
    untapped_wall("Castle", cost(&[generic(3), w()]))
}

/// Builder's Blessing — {3}{W} Enchantment.
pub fn builders_blessing() -> CardDefinition {
    untapped_wall("Builder's Blessing", cost(&[generic(3), w()]))
}

/// War Horn — {3} Artifact. Attacking creatures you control get +1/+0.
pub fn war_horn() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Attacking creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT { applies_to: mine(R::Creature.and(R::IsAttacking)), power: 1, toughness: 0 },
        }],
        ..artifact("War Horn", cost(&[generic(3)]))
    }
}

/// Zephyr Charge — {1}{U} Enchantment. {1}{U}: target creature gains flying
/// until end of turn.
pub fn zephyr_charge() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![act(cost(&[generic(1), u()]), false, grant(target_filtered(R::Creature), Keyword::Flying))],
        ..enchantment("Zephyr Charge", cost(&[generic(1), u()]))
    }
}

/// Unbender Tine — {2}{W}{U} Artifact. {T}: untap another target permanent.
pub fn unbender_tine() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![act(cost(&[]), true, Effect::Untap { what: target_filtered(R::Permanent.and(R::OtherThanSource)), up_to: None })],
        ..artifact("Unbender Tine", cost(&[generic(2), w(), u()]))
    }
}

/// Golem's Heart — {2} Artifact. Whenever a player casts an artifact spell,
/// you may gain 1 life.
pub fn golems_heart() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasCardType(CardType::Artifact) }),
            effect: Effect::MayDo { description: "Gain 1 life?".into(), body: Box::new(gain(1)) },
        }],
        ..artifact("Golem's Heart", cost(&[generic(2)]))
    }
}

/// Copper Tablet — {2} Artifact. At the beginning of each player's upkeep, 1
/// damage to that player.
pub fn copper_tablet() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::DealDamage { to: Selector::Player(PlayerRef::ActivePlayer), amount: Value::ONE },
        }],
        ..artifact("Copper Tablet", cost(&[generic(2)]))
    }
}

/// Hierophant's Chalice — {3} Artifact. ETB: target opponent loses 1 life and
/// you gain 1. {T}: add {C}.
pub fn hierophants_chalice() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(crate::effect::shortcut::drain_target_opponent(1))],
        activated_abilities: vec![crate::sets::tap_add_colorless()],
        ..artifact("Hierophant's Chalice", cost(&[generic(3)]))
    }
}

/// Selesnya Locket — {3} Artifact. {T}: add {G} or {W}. {G/W}×4, {T},
/// sacrifice it: draw two.
pub fn selesnya_locket() -> CardDefinition {
    let gw = || hybrid(Color::Green, Color::White);
    CardDefinition {
        activated_abilities: vec![
            act(cost(&[]), true, Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColors(vec![Color::Green, Color::White], Value::ONE) }),
            ActivatedAbility { mana_cost: cost(&[gw(), gw(), gw(), gw()]), tap_cost: true, sac_cost: true, effect: draw(2), ..Default::default() },
        ],
        ..artifact("Selesnya Locket", cost(&[generic(3)]))
    }
}

/// Staff of the Mind Magus — {3} Artifact. Whenever you cast a blue spell or
/// an Island you control enters, gain 1 life.
pub fn staff_of_the_mind_magus() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasColor(Color::Blue) }),
                effect: gain(1),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasLandType(LandType::Island) }),
                effect: gain(1),
            },
        ],
        ..artifact("Staff of the Mind Magus", cost(&[generic(3)]))
    }
}

/// Curse of the Bloody Tome — {2}{U} Aura Curse, enchant player. At the
/// beginning of enchanted player's upkeep, they mill two.
pub fn curse_of_the_bloody_tome() -> CardDefinition {
    CardDefinition {
        name: "Curse of the Bloody Tome",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Curse], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Player) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
                .with_filter(Predicate::IsTurnOf(PlayerRef::EnchantedPlayer)),
            effect: Effect::Mill { who: Selector::Player(PlayerRef::EnchantedPlayer), amount: Value::Const(2) },
        }],
        ..Default::default()
    }
}

/// "{T}: Add {C}{C}" sac-for-two lands: enters tapped, {T}: one, {T}, sacrifice:
/// two of the color (Ruins of Trokair, Svyelunite Temple).
fn sac_for_two_land(name: &'static str, color: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![
            crate::sets::tap_add(color),
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(color, Value::Const(2)) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ruins of Trokair — W sac-for-two land.
pub fn ruins_of_trokair() -> CardDefinition {
    sac_for_two_land("Ruins of Trokair", Color::White)
}

/// Svyelunite Temple — U sac-for-two land.
pub fn svyelunite_temple() -> CardDefinition {
    sac_for_two_land("Svyelunite Temple", Color::Blue)
}

/// The Marvel gain lands this scan found.
pub fn subterranean_cavern() -> CardDefinition {
    tapland_untyped("Subterranean Cavern", Color::Black, Color::Green, etb_gain_one())
}
/// Pym Technologies — G/U gain land.
pub fn pym_technologies() -> CardDefinition {
    tapland_untyped("Pym Technologies", Color::Green, Color::Blue, etb_gain_one())
}
/// Stark Industries — U/R gain land.
pub fn stark_industries() -> CardDefinition {
    tapland_untyped("Stark Industries", Color::Blue, Color::Red, etb_gain_one())
}
/// TCRI Building — U/R gain land.
pub fn tcri_building() -> CardDefinition {
    tapland_untyped("TCRI Building", Color::Blue, Color::Red, etb_gain_one())
}

/// The Strixhaven-planeswalker duals: enters tapped unless you control a
/// planeswalker; {T}: either color.
pub(super) fn walker_dual(name: &'static str, a: Color, b2: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped unless you control a planeswalker.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorExists(Selector::ControlledBy { who: PlayerRef::You, filter: R::Planeswalker }),
            },
        }],
        activated_abilities: vec![act(cost(&[]), true, Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColors(vec![a, b2], Value::ONE) })],
        ..Default::default()
    }
}

/// Innovative Commons — U/R.
pub fn innovative_commons() -> CardDefinition {
    walker_dual("Innovative Commons", Color::Blue, Color::Red)
}
/// Konstrari Annex — R/G.
pub fn konstrari_annex() -> CardDefinition {
    walker_dual("Konstrari Annex", Color::Red, Color::Green)
}
/// Transformative Commons — G/U.
pub fn transformative_commons() -> CardDefinition {
    walker_dual("Transformative Commons", Color::Green, Color::Blue)
}

// ── Spells ─────────────────────────────────────────────────────────────────

/// Ravages of War — {3}{W} Sorcery. Destroy all lands.
pub fn ravages_of_war() -> CardDefinition {
    spell("Ravages of War", cost(&[generic(3), w()]), CardType::Sorcery, Effect::Destroy { what: Selector::EachPermanent(R::Land) })
}

/// Bilbo's Deadly Slice — {1}{B}{B} Instant. Destroy target creature.
pub fn bilbos_deadly_slice() -> CardDefinition {
    spell("Bilbo's Deadly Slice", cost(&[generic(1), b(), b()]), CardType::Instant, Effect::Destroy { what: target_filtered(R::Creature) })
}

/// Dangerous Wager — {1}{R} Instant. Discard your hand, then draw two.
pub fn dangerous_wager() -> CardDefinition {
    spell(
        "Dangerous Wager",
        cost(&[generic(1), r()]),
        CardType::Instant,
        Effect::Seq(vec![Effect::Discard { who: Selector::You, amount: Value::Const(100), random: false }, draw(2)]),
    )
}

/// Bountiful Harvest — {4}{G} Sorcery. Gain 1 life per land you control.
pub fn bountiful_harvest() -> CardDefinition {
    spell("Bountiful Harvest", cost(&[generic(4), g()]), CardType::Sorcery, Effect::GainLife { who: Selector::You, amount: count_mine(R::Land) })
}

/// Festival of Trokin — {W} Sorcery. Gain 2 life per creature you control.
pub fn festival_of_trokin() -> CardDefinition {
    spell(
        "Festival of Trokin",
        cost(&[w()]),
        CardType::Sorcery,
        Effect::GainLife { who: Selector::You, amount: Value::Times(Box::new(Value::Const(2)), Box::new(count_mine(R::Creature))) },
    )
}

/// Death in the Family — {1}{B} Instant. Exile target creature with mana value
/// 3 or less.
pub fn death_in_the_family() -> CardDefinition {
    spell("Death in the Family", cost(&[generic(1), b()]), CardType::Instant, Effect::Exile { what: target_filtered(R::Creature.and(R::ManaValueAtMost(3))) })
}

/// Tidal Surge — {1}{U} Sorcery. Tap up to three target creatures without
/// flying.
pub fn tidal_surge() -> CardDefinition {
    spell(
        "Tidal Surge",
        cost(&[generic(1), u()]),
        CardType::Sorcery,
        Effect::ApplyToTargets {
            min_targets: 0,
            max_targets: 3,
            filter: R::Creature.and(R::HasKeyword(Keyword::Flying).negate()),
            effect: Box::new(Effect::Tap { what: Selector::Target(0) }),
        },
    )
}

/// Harrowing Journey — {4}{B} Sorcery. Target player draws three and loses 3.
pub fn harrowing_journey() -> CardDefinition {
    spell(
        "Harrowing Journey",
        cost(&[generic(4), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Draw { who: target_filtered(R::Player), amount: Value::Const(3) },
            Effect::LoseLife { who: Selector::Target(0), amount: Value::Const(3) },
        ]),
    )
}

/// Time Ebb — {2}{U} Sorcery. Put target creature on top of its owner's
/// library.
pub fn time_ebb() -> CardDefinition {
    spell(
        "Time Ebb",
        cost(&[generic(2), u()]),
        CardType::Sorcery,
        Effect::Move {
            what: target_filtered(R::Creature),
            to: ZoneDest::Library { who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))), pos: LibraryPosition::Top },
        },
    )
}

/// True Love's Kiss — {2}{W}{W} Instant. Exile target artifact or enchantment;
/// draw a card.
pub fn true_loves_kiss() -> CardDefinition {
    spell(
        "True Love's Kiss",
        cost(&[generic(2), w(), w()]),
        CardType::Instant,
        Effect::Seq(vec![Effect::Exile { what: target_filtered(R::Artifact.or(R::Enchantment)) }, draw(1)]),
    )
}

/// Silverstrike — {3}{W} Instant. Destroy target attacking creature; gain 3.
pub fn silverstrike() -> CardDefinition {
    spell(
        "Silverstrike",
        cost(&[generic(3), w()]),
        CardType::Instant,
        Effect::Seq(vec![Effect::Destroy { what: target_filtered(R::Creature.and(R::IsAttacking)) }, gain(3)]),
    )
}

/// Instill Infection — {3}{B} Instant. A -1/-1 counter on target creature;
/// draw a card.
pub fn instill_infection() -> CardDefinition {
    spell(
        "Instill Infection",
        cost(&[generic(3), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::MinusOneMinusOne, amount: Value::ONE },
            draw(1),
        ]),
    )
}

/// Control of the Court — {1}{R} Sorcery. Draw four, then discard three at
/// random.
pub fn control_of_the_court() -> CardDefinition {
    spell(
        "Control of the Court",
        cost(&[generic(1), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![draw(4), Effect::Discard { who: Selector::You, amount: Value::Const(3), random: true }]),
    )
}

/// Engulfing Eruption — {2}{R}{R} Sorcery. 5 damage to target creature.
pub fn engulfing_eruption() -> CardDefinition {
    spell("Engulfing Eruption", cost(&[generic(2), r(), r()]), CardType::Sorcery, Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(5) })
}

/// Bathe in Dragonfire — {2}{R} Sorcery. 4 damage to target creature.
pub fn bathe_in_dragonfire() -> CardDefinition {
    spell("Bathe in Dragonfire", cost(&[generic(2), r()]), CardType::Sorcery, Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(4) })
}

/// Righteousness — {W} Instant. Target blocking creature gets +7/+7.
pub fn righteousness() -> CardDefinition {
    spell("Righteousness", cost(&[w()]), CardType::Instant, pump(target_filtered(R::Creature.and(R::IsBlocking)), 7, 7))
}

/// Ornamental Courage — {G} Instant. Untap target creature; +1/+3.
pub fn ornamental_courage() -> CardDefinition {
    spell(
        "Ornamental Courage",
        cost(&[g()]),
        CardType::Instant,
        Effect::Seq(vec![Effect::Untap { what: target_filtered(R::Creature), up_to: None }, pump(Selector::Target(0), 1, 3)]),
    )
}

/// Guardians' Pledge — {1}{W}{W} Instant. White creatures you control get
/// +2/+2.
pub fn guardians_pledge() -> CardDefinition {
    spell("Guardians' Pledge", cost(&[generic(1), w(), w()]), CardType::Instant, pump(mine(R::Creature.and(R::HasColor(Color::White))), 2, 2))
}

/// Sworn Companions — {2}{W} Sorcery. Two 1/1 white lifelink Soldiers.
pub fn sworn_companions() -> CardDefinition {
    spell(
        "Sworn Companions",
        cost(&[generic(2), w()]),
        CardType::Sorcery,
        mint(2, token("Soldier", 1, 1, vec![Color::White], vec![CreatureType::Soldier], vec![Keyword::Lifelink])),
    )
}

/// Artificer's Epiphany — {2}{U} Instant. Draw two; discard one if you control
/// no artifacts.
pub fn artificers_epiphany() -> CardDefinition {
    spell(
        "Artificer's Epiphany",
        cost(&[generic(2), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            draw(2),
            Effect::If {
                cond: Predicate::SelectorExists(Selector::ControlledBy { who: PlayerRef::You, filter: R::Artifact }),
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }),
            },
        ]),
    )
}

/// Rallying Roar — {2}{W} Instant. Your creatures get +1/+1 and untap.
pub fn rallying_roar() -> CardDefinition {
    spell(
        "Rallying Roar",
        cost(&[generic(2), w()]),
        CardType::Instant,
        Effect::Seq(vec![pump(mine(R::Creature), 1, 1), Effect::Untap { what: mine(R::Creature), up_to: None }]),
    )
}

/// War Flare — {2}{R}{W} Instant. Your creatures get +2/+1 and untap.
pub fn war_flare() -> CardDefinition {
    spell(
        "War Flare",
        cost(&[generic(2), r(), w()]),
        CardType::Instant,
        Effect::Seq(vec![pump(mine(R::Creature), 2, 1), Effect::Untap { what: mine(R::Creature), up_to: None }]),
    )
}

/// Bull's Strength — {1}{G} Instant. +2/+2, trample, untap it.
pub fn bulls_strength() -> CardDefinition {
    spell(
        "Bull's Strength",
        cost(&[generic(1), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            pump(target_filtered(R::Creature), 2, 2),
            grant(Selector::Target(0), Keyword::Trample),
            Effect::Untap { what: Selector::Target(0), up_to: None },
        ]),
    )
}

/// Whirling Strike — {1}{R} Instant. +2/+0, first strike and trample.
pub fn whirling_strike() -> CardDefinition {
    spell(
        "Whirling Strike",
        cost(&[generic(1), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            pump(target_filtered(R::Creature), 2, 0),
            grant(Selector::Target(0), Keyword::FirstStrike),
            grant(Selector::Target(0), Keyword::Trample),
        ]),
    )
}

/// Flying Crane Technique — {3}{U}{R}{W} Instant. Untap your creatures; they
/// gain flying and double strike.
pub fn flying_crane_technique() -> CardDefinition {
    spell(
        "Flying Crane Technique",
        cost(&[generic(3), u(), r(), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Untap { what: mine(R::Creature), up_to: None },
            grant(mine(R::Creature), Keyword::Flying),
            grant(mine(R::Creature), Keyword::DoubleStrike),
        ]),
    )
}

/// Filter Out — {1}{U}{U} Instant. Return all noncreature, nonland permanents
/// to their owners' hands.
pub fn filter_out() -> CardDefinition {
    spell(
        "Filter Out",
        cost(&[generic(1), u(), u()]),
        CardType::Instant,
        Effect::Move { what: Selector::EachPermanent(R::Permanent.and(R::Creature.negate()).and(R::Nonland)), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
    )
}

/// Lucky Offering — {W} Sorcery. Destroy target artifact with mana value 3 or
/// less; gain 3.
pub fn lucky_offering() -> CardDefinition {
    spell(
        "Lucky Offering",
        cost(&[w()]),
        CardType::Sorcery,
        Effect::Seq(vec![Effect::Destroy { what: target_filtered(R::Artifact.and(R::ManaValueAtMost(3))) }, gain(3)]),
    )
}

/// Punish Ignorance — {W}{U}{U}{B} Instant. Counter target spell; its
/// controller loses 3 life and you gain 3.
pub fn punish_ignorance() -> CardDefinition {
    spell(
        "Punish Ignorance",
        cost(&[w(), u(), u(), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::LoseLife { who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))), amount: Value::Const(3) },
            gain(3),
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
        ]),
    )
}

/// Precise Redaction — {1}{U} Instant. Counter target white or black spell.
pub fn precise_redaction() -> CardDefinition {
    spell(
        "Precise Redaction",
        cost(&[generic(1), u()]),
        CardType::Instant,
        Effect::CounterSpell { what: target_filtered(spell_of_color(&[Color::White, Color::Black])) },
    )
}

/// Lifeforce — {G}{G} Enchantment. {G}{G}: counter target black spell.
pub fn lifeforce() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![act(cost(&[g(), g()]), false, Effect::CounterSpell { what: target_filtered(spell_of_color(&[Color::Black])) })],
        ..enchantment("Lifeforce", cost(&[g(), g()]))
    }
}

/// Scatter Ray — {1}{U} Instant. Counter target artifact or creature spell
/// unless its controller pays {4}.
pub fn scatter_ray() -> CardDefinition {
    spell(
        "Scatter Ray",
        cost(&[generic(1), u()]),
        CardType::Instant,
        Effect::CounterUnlessPaid {
            what: target_filtered(R::IsSpellOnStack.and(R::HasCardType(CardType::Artifact).or(R::HasCardType(CardType::Creature)))),
            mana_cost: cost(&[generic(4)]),
            exile: false,
            extra_generic: None,
            if_paid: None,
        },
    )
}

/// Boulder Dash — {1}{R} Sorcery. 2 damage to any target and 1 to another.
pub fn boulder_dash() -> CardDefinition {
    spell(
        "Boulder Dash",
        cost(&[generic(1), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
            Effect::DealDamage {
                to: Selector::TargetFiltered { slot: 1, filter: R::any_target().and(R::OtherThanTargetSlot(0)) },
                amount: Value::ONE,
            },
        ]),
    )
}

/// Amass the Components — {3}{U} Sorcery. Draw three, then put a card from
/// your hand on the bottom of your library.
pub fn amass_the_components() -> CardDefinition {
    spell(
        "Amass the Components",
        cost(&[generic(3), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![draw(3), Effect::PutCardsFromHandOnBottom { who: Selector::You, count: Value::ONE }]),
    )
}

/// Deglamer — {1}{G} Instant. Target artifact or enchantment's owner shuffles
/// it into their library.
pub fn deglamer() -> CardDefinition {
    spell(
        "Deglamer",
        cost(&[generic(1), g()]),
        CardType::Instant,
        Effect::Move {
            what: target_filtered(R::Artifact.or(R::Enchantment)),
            to: ZoneDest::Library { who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))), pos: LibraryPosition::Shuffled },
        },
    )
}

/// Orcish Cannonade — {1}{R}{R} Instant. 2 damage to any target, 3 to you;
/// draw a card.
pub fn orcish_cannonade() -> CardDefinition {
    spell(
        "Orcish Cannonade",
        cost(&[generic(1), r(), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
            Effect::DealDamage { to: Selector::You, amount: Value::Const(3) },
            draw(1),
        ]),
    )
}

/// Bad Deal — {4}{B}{B} Sorcery. You draw two, each opponent discards two;
/// each player loses 2 life.
pub fn bad_deal() -> CardDefinition {
    spell(
        "Bad Deal",
        cost(&[generic(4), b(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            draw(2),
            Effect::Discard { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2), random: false },
            Effect::LoseLife { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(2) },
        ]),
    )
}

/// Ray of Erasure — {U} Instant. Target player mills a card; draw a card at
/// the beginning of the next turn's upkeep.
pub fn ray_of_erasure() -> CardDefinition {
    spell(
        "Ray of Erasure",
        cost(&[u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Mill { who: target_filtered(R::Player), amount: Value::ONE },
            Effect::DelayUntil { kind: DelayedTriggerKind::NextUpkeep, body: Box::new(draw(1)) },
        ]),
    )
}

/// Violent Impact — {3}{R} Sorcery. Destroy target artifact or land. Cycling
/// {2}.
pub fn violent_impact() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        ..spell("Violent Impact", cost(&[generic(3), r()]), CardType::Sorcery, Effect::Destroy { what: target_filtered(R::Artifact.or(R::Land)) })
    }
}

/// Balance of Power — {3}{U}{U} Sorcery. Draw the difference if target
/// opponent has more cards in hand than you.
pub fn balance_of_power() -> CardDefinition {
    spell(
        "Balance of Power",
        cost(&[generic(3), u(), u()]),
        CardType::Sorcery,
        Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 1,
            filter: R::OpponentPlayer,
            effect: Box::new(Effect::Draw {
                who: Selector::You,
                amount: Value::NonNeg(Box::new(Value::Diff(
                    Box::new(Value::HandSizeOf(PlayerRef::Target(0))),
                    Box::new(Value::HandSizeOf(PlayerRef::You)),
                ))),
            }),
        },
    )
}

/// Dreadwaters — {3}{U} Sorcery. Target player mills X (X = lands you
/// control).
pub fn dreadwaters() -> CardDefinition {
    spell("Dreadwaters", cost(&[generic(3), u()]), CardType::Sorcery, Effect::Mill { who: target_filtered(R::Player), amount: count_mine(R::Land) })
}
