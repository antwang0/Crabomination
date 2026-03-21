use crate::mana::{Color, ManaCost};

/// Unique runtime ID for a card instance within a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CardId(pub u32);

/// A single card type. Cards may have multiple types (e.g. Enchantment + Creature).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CardType {
    Land,
    Creature,
    Artifact,
    Enchantment,
    Planeswalker,
    Battle,
    Instant,
    Sorcery,
    Kindred,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Keyword {
    Flying,
    Haste,
    Vigilance,
    FirstStrike,
    Trample,
    Lifelink,
    Deathtouch,
    Reach,
}

/// Composable filter for valid targets of a spell or ability.
///
/// Build complex requirements via the `.and()`, `.or()`, `.not()` builder methods:
/// ```rust,ignore
/// // Terror: non-black, non-artifact creature
/// SelectionRequirement::Creature
///     .and(SelectionRequirement::HasColor(Color::Black).not())
///     .and(SelectionRequirement::Artifact.not())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionRequirement {
    /// Matches any valid game object (creature, permanent, or player).
    Any,
    /// Target must be a player.
    Player,
    /// Target must be a creature permanent.
    Creature,
    /// Target must be an artifact permanent.
    Artifact,
    /// Target must be a land permanent.
    Land,
    /// Target permanent must have the given color in its mana cost.
    HasColor(Color),
    /// Target permanent must have the given keyword.
    HasKeyword(Keyword),
    /// Target creature's power must be ≤ n.
    PowerAtMost(i32),
    /// Both sub-requirements must be satisfied.
    And(Box<SelectionRequirement>, Box<SelectionRequirement>),
    /// Either sub-requirement must be satisfied.
    Or(Box<SelectionRequirement>, Box<SelectionRequirement>),
    /// The sub-requirement must NOT be satisfied.
    Not(Box<SelectionRequirement>),
}

impl SelectionRequirement {
    pub fn and(self, other: Self) -> Self {
        Self::And(Box::new(self), Box::new(other))
    }
    pub fn or(self, other: Self) -> Self {
        Self::Or(Box::new(self), Box::new(other))
    }
    pub fn not(self) -> Self {
        Self::Not(Box::new(self))
    }
}

/// Effect produced when an instant/sorcery resolves, or an ability fires.
#[derive(Debug, Clone)]
pub enum SpellEffect {
    DealDamage {
        amount: u32,
        target: SelectionRequirement,
    },
    DestroyCreature {
        target: SelectionRequirement,
    },
    DrawCards {
        amount: u32,
    },
    /// Add the given colors of mana to the controller's pool.
    AddMana {
        colors: Vec<Color>,
    },
    /// Target creature gets +X/+Y until end of turn.
    PumpCreature {
        power_bonus: i32,
        toughness_bonus: i32,
    },
    /// Gain life.
    GainLife {
        amount: u32,
    },
    /// Destroy all creatures on the battlefield (e.g. Wrath of God).
    DestroyAllCreatures,
    /// The opposing player reveals the top card of their library;
    /// if it's a land, they put it into their hand (Goblin Guide attack trigger).
    RevealOpponentTopCard,
    /// The opposing player discards a card at random from their hand
    /// (Hypnotic Specter attack trigger — simplified from "deals combat damage").
    OpponentDiscardRandom,
}

// ── Triggered abilities ───────────────────────────────────────────────────────

/// When a triggered ability fires.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerCondition {
    /// Fires when this permanent enters the battlefield.
    EntersBattlefield,
    /// Fires when this creature attacks.
    Attacks,
}

/// An ability that fires automatically when its condition is met.
#[derive(Debug, Clone)]
pub struct TriggeredAbility {
    pub condition: TriggerCondition,
    pub effects: Vec<SpellEffect>,
}

/// An activated ability with a tap and/or mana cost.
#[derive(Debug, Clone)]
pub struct ActivatedAbility {
    pub tap_cost: bool,
    pub mana_cost: ManaCost,
    pub effects: Vec<SpellEffect>,
}

/// Static blueprint for a card; cloned into `CardInstance` at game-time.
#[derive(Debug, Clone)]
pub struct CardDefinition {
    pub name: &'static str,
    pub cost: ManaCost,
    /// All types this card has (may be more than one, e.g. `[Enchantment, Creature]`).
    pub card_types: Vec<CardType>,
    /// Base power (relevant for Creature types; 0 otherwise).
    pub power: i32,
    /// Base toughness (relevant for Creature types; 0 otherwise).
    pub toughness: i32,
    pub keywords: Vec<Keyword>,
    /// For instants/sorceries: the effects that resolve.
    pub spell_effects: Vec<SpellEffect>,
    /// For permanents: activated abilities (e.g. "{T}: Add {G}").
    pub activated_abilities: Vec<ActivatedAbility>,
    /// Triggered abilities that fire on ETB, attack, etc.
    pub triggered_abilities: Vec<TriggeredAbility>,
}

impl CardDefinition {
    pub fn is_land(&self) -> bool {
        self.card_types.contains(&CardType::Land)
    }
    pub fn is_creature(&self) -> bool {
        self.card_types.contains(&CardType::Creature)
    }
    pub fn is_instant(&self) -> bool {
        self.card_types.contains(&CardType::Instant)
    }
    pub fn is_sorcery(&self) -> bool {
        self.card_types.contains(&CardType::Sorcery)
    }
    pub fn is_permanent(&self) -> bool {
        self.card_types.iter().any(|t| {
            matches!(
                t,
                CardType::Land
                    | CardType::Creature
                    | CardType::Enchantment
                    | CardType::Artifact
                    | CardType::Planeswalker
            )
        })
    }

    pub fn base_power(&self) -> i32 {
        if self.is_creature() { self.power } else { 0 }
    }

    pub fn base_toughness(&self) -> i32 {
        if self.is_creature() { self.toughness } else { 0 }
    }
}

// ── Runtime card instance ─────────────────────────────────────────────────────

/// A card in play.  Tracks mutable game state layered on top of the static definition.
#[derive(Debug, Clone)]
pub struct CardInstance {
    pub id: CardId,
    pub definition: CardDefinition,
    /// Which player (index) owns this card; determines graveyard destination on death.
    pub owner: usize,
    pub tapped: bool,
    /// Combat damage marked on this permanent this turn.
    pub damage: u32,
    /// True while a creature has not yet been untapped under its controller's control.
    pub summoning_sick: bool,
    /// Temporary power modifier (expires at end of turn).
    pub power_bonus: i32,
    /// Temporary toughness modifier (expires at end of turn).
    pub toughness_bonus: i32,
}

impl CardInstance {
    pub fn new(id: CardId, definition: CardDefinition, owner: usize) -> Self {
        let summoning_sick = definition.is_creature();
        Self {
            id,
            definition,
            owner,
            tapped: false,
            damage: 0,
            summoning_sick,
            power_bonus: 0,
            toughness_bonus: 0,
        }
    }

    pub fn power(&self) -> i32 {
        self.definition.base_power() + self.power_bonus
    }

    pub fn toughness(&self) -> i32 {
        self.definition.base_toughness() + self.toughness_bonus
    }

    /// True if a creature has lethal damage (damage >= toughness).
    pub fn is_dead(&self) -> bool {
        self.definition.is_creature() && self.damage as i32 >= self.toughness()
    }

    pub fn has_keyword(&self, kw: &Keyword) -> bool {
        self.definition.keywords.contains(kw)
    }

    /// Can this creature be declared as an attacker?
    pub fn can_attack(&self) -> bool {
        self.definition.is_creature()
            && !self.tapped
            && (!self.summoning_sick || self.has_keyword(&Keyword::Haste))
    }

    /// Can this creature be declared as a blocker?
    pub fn can_block(&self) -> bool {
        self.definition.is_creature() && !self.tapped
    }

    /// Reset temporary pump effects at end of turn.
    pub fn clear_end_of_turn_effects(&mut self) {
        self.power_bonus = 0;
        self.toughness_bonus = 0;
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;

    #[test]
    fn grizzly_bears_base_stats() {
        let c = CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
        assert_eq!(c.power(), 2);
        assert_eq!(c.toughness(), 2);
    }

    #[test]
    fn new_creature_has_summoning_sickness() {
        let c = CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
        assert!(c.summoning_sick);
        assert!(!c.can_attack());
    }

    #[test]
    fn haste_creature_can_attack_with_summoning_sickness() {
        let c = CardInstance::new(CardId(0), catalog::goblin_guide(), 0);
        assert!(c.summoning_sick);
        assert!(c.can_attack());
    }

    #[test]
    fn tapped_creature_cannot_attack() {
        let mut c = CardInstance::new(CardId(0), catalog::goblin_guide(), 0);
        c.tapped = true;
        assert!(!c.can_attack());
    }

    #[test]
    fn tapped_creature_cannot_block() {
        let mut c = CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
        c.summoning_sick = false;
        c.tapped = true;
        assert!(!c.can_block());
    }

    #[test]
    fn creature_dies_at_lethal_damage() {
        let mut c = CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
        c.damage = 2;
        assert!(c.is_dead());
    }

    #[test]
    fn pump_keeps_creature_alive_through_damage() {
        let mut c = CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
        c.damage = 2;
        c.toughness_bonus = 3; // now 5 toughness
        assert!(!c.is_dead());
    }

    #[test]
    fn clear_end_of_turn_resets_bonuses() {
        let mut c = CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
        c.power_bonus = 3;
        c.toughness_bonus = 3;
        c.clear_end_of_turn_effects();
        assert_eq!(c.power(), 2);
        assert_eq!(c.toughness(), 2);
    }

    #[test]
    fn serra_angel_has_flying_and_vigilance() {
        let c = CardInstance::new(CardId(0), catalog::serra_angel(), 0);
        assert!(c.has_keyword(&Keyword::Flying));
        assert!(c.has_keyword(&Keyword::Vigilance));
    }

    #[test]
    fn land_is_not_creature() {
        let def = catalog::forest();
        assert!(def.is_land());
        assert!(!def.is_creature());
    }

    #[test]
    fn cmc_checks() {
        assert_eq!(catalog::lightning_bolt().cost.cmc(), 1); // {R}
        assert_eq!(catalog::grizzly_bears().cost.cmc(), 2); // {1}{G}
        assert_eq!(catalog::serra_angel().cost.cmc(), 5); // {3}{W}{W}
    }

    #[test]
    fn enchantment_creature_has_both_types() {
        let def = catalog::hopeful_eidolon();
        assert!(def.is_creature());
        assert!(def.card_types.contains(&CardType::Enchantment));
    }
}
