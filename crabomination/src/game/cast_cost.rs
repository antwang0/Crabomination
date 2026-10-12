//! CR 601.2f — the cost INCREASES every cast pays, whatever base cost its
//! path chose (mana cost, flashback, escape, foretell, a may-play grant).
//! The hand path applies them inline; the zone paths share this.

use super::GameState;
use super::types::Target;
use crate::card::CardInstance;
use crate::mana::ManaCost;

impl GameState {
    /// Add the generic and colored taxes (Thalia, Sphere of Resistance, the
    /// Invasion Leeches) a cast of `card` by `p` pays, before its reductions.
    pub(crate) fn add_spell_taxes(&self, p: usize, card: &CardInstance, target: Option<&Target>, cost: &mut ManaCost) {
        cost.add_generic(super::actions::extra_cost_for_spell(self, p, card, target));
        cost.symbols.extend(super::actions::colored_spell_tax_for_spell(self, p, card).symbols);
    }
}

/// The spell a half-card cast puts on the stack, as the cost walks read it:
/// the half's name, cost and card types only (CR 709.3 split halves, 715.3
/// an Adventure, an Omen). The card stays whole on the stack — only the
/// cost filters (Thalia's "noncreature", an instant/sorcery discount) see
/// this.
pub(crate) fn half_spell_probe(
    card: &CardInstance,
    name: Option<crate::static_str_serde::StaticStr>,
    cost: &ManaCost,
    types: &[crate::card::CardType],
) -> CardInstance {
    let mut def = (*card.definition.arc()).clone();
    if let Some(n) = name {
        def.name = n;
    }
    def.cost = cost.clone();
    def.card_types = types.to_vec();
    def.subtypes = Default::default();
    def.supertypes = Vec::new();
    let mut probe = CardInstance::new(card.id, def, card.owner);
    probe.controller = card.controller;
    probe
}

impl GameState {
    /// CR 601.2f for a half-card cast: the taxes, then the generic and
    /// colored reductions, read against [`half_spell_probe`].
    pub(crate) fn apply_half_cast_cost_modifiers(
        &self,
        p: usize,
        probe: &CardInstance,
        target: Option<&Target>,
        from_graveyard: bool,
        cost: &mut ManaCost,
    ) {
        self.add_spell_taxes(p, probe, target, cost);
        let less = super::actions::cost_reduction_for_spell_full(self, p, probe, target, from_graveyard, false);
        if less > 0 {
            cost.reduce_generic(less);
        }
        super::actions::apply_colored_cost_statics(self, p, probe, cost);
    }
}

/// CR 119.4 — a life payment greater than 0 needs at least that much life;
/// paying 0 life is always legal, even at 0 or less life (a seat kept in
/// the game by Platinum Angel still casts its spells).
pub(crate) fn life_payable(life: i32, amount: u32) -> bool {
    amount == 0 || life >= amount as i32
}

impl GameState {
    /// Angel of Jubilation on the battlefield: no life payments and no creature
    /// sacrifices as costs of spells or activated abilities (CR 119.4b). The
    /// life lane gates the walk, so a board without one costs one bit test.
    pub(crate) fn cost_payments_restricted(&self) -> bool {
        self.cost_restriction_present(false)
    }

    /// Yasharn, Implacable Earth on the battlefield: no life payments and no
    /// *nonland* sacrifices as costs of spells or activated abilities.
    pub(crate) fn nonland_sacrifice_costs_restricted(&self) -> bool {
        self.cost_restriction_present(true)
    }

    /// The shared walk: either static bans paying life (`nonland_only` false
    /// answers "any ban"); `nonland_only` asks for Yasharn's alone.
    fn cost_restriction_present(&self, nonland_only: bool) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.has_life_static()
            && self.battlefield.iter().any(|c| {
                c.definition.static_abilities.iter().any(|sa| match self.active_static(&sa.effect, c) {
                    Some(StaticEffect::PlayersCantPayLifeOrSacrificeNonlandForCosts) => true,
                    Some(StaticEffect::PlayersCantPayLifeOrSacrificeCreaturesForCosts) => !nonland_only,
                    _ => false,
                })
            })
    }

    /// [`life_payable`] for a cost of a spell or activated ability `p` casts or
    /// activates: also barred by CR 119.4b (Angel of Jubilation) and CR 119.8
    /// ("can't lose life" — Platinum Emperion). Paying 0 is always legal.
    pub(crate) fn cost_life_payable(&self, p: usize, life: i32, amount: u32) -> bool {
        amount == 0
            || (life_payable(life, amount) && !self.cost_payments_restricted() && !self.player_cannot_lose_life_now(p))
    }

    /// May `c` be sacrificed to pay a cost (CR 701.21)? Not when it can't be
    /// sacrificed at all (Assault Suit), nor a creature under Angel of
    /// Jubilation — judged by its current types, so an artifact creature can't
    /// pay "Sacrifice an artifact" either (ruling 2012-05-01).
    pub(crate) fn sacrificeable_for_cost(&self, c: &CardInstance) -> bool {
        self.can_be_sacrificed(c.id)
            && !(self.cost_payments_restricted()
                && (self.computed_is_creature(c)
                    || (!self.computed_has_card_type(c, crate::card::CardType::Land)
                        && self.nonland_sacrifice_costs_restricted())))
    }

    /// [`Self::sacrificeable_for_cost`] by id; false off the battlefield.
    pub(crate) fn sacrificeable_for_cost_id(&self, id: crate::card::CardId) -> bool {
        self.battlefield_find(id).is_some_and(|c| self.sacrificeable_for_cost(c))
    }
}

impl GameState {
    /// CR 611.2b — `source` was cast: each `Duration::UntilSourceCast` grant it
    /// made ends (Emrakul, the Exigent Doom's "until this card is cast from
    /// exile"; the card can only be cast from exile once it made one).
    pub(crate) fn end_grants_until_cast(&mut self, source: crate::card::CardId) {
        let (ended, kept): (Vec<_>, Vec<_>) =
            std::mem::take(&mut self.grants_until_cast).into_iter().partition(|(_, s, _)| *s == source);
        self.grants_until_cast = kept;
        for (perm, _, ability) in ended {
            if let Some(c) = self.battlefield_find_mut(perm)
                && let Some(i) = c.granted_activated_abilities.iter().position(|a| *a == ability)
            {
                c.granted_activated_abilities.remove(i);
            }
        }
    }
}

impl GameState {
    /// The colors Painter's Servant adds to every card off the battlefield and
    /// every spell (CR 105.3): each one's chosen color. Gated on the life lane
    /// the static shares, so a board without one costs a bit test.
    pub(crate) fn painted_colors(&self) -> crate::mana::ColorSet {
        use crate::effect::StaticEffect;
        let mut set = crate::mana::ColorSet::empty();
        if !self.battlefield.has_life_static() {
            return set;
        }
        for c in self.battlefield.iter() {
            if let Some(color) = c.chosen_color
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(self.active_static(&sa.effect, c), Some(StaticEffect::ChosenColorEverywhere))
                })
            {
                set.insert(color);
            }
        }
        set
    }

    /// A card's colors off the battlefield (hand, graveyard, exile, library, the
    /// stack): printed, plus [`Self::painted_colors`].
    pub(crate) fn off_battlefield_colors(&self, card: &CardInstance) -> crate::mana::ColorSet {
        card.definition.printed_color_set().union(self.painted_colors())
    }
}
