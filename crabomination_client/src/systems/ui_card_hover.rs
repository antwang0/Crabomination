//! Cards named by *UI* nodes — stack-panel tiles, game-log lines, the cards
//! in a decision modal — preview like the cards on the table: hovering one
//! shows its art and rules text beside it (`ui::hover_card_preview`), and
//! holding Alt opens the large peek (`ui::peek_popup`).
//!
//! Usage: attach `UiCardHover` plus Bevy's `Button` (for `Interaction`
//! tracking) to any UI node. One preview shows at a time; a hovered UI card
//! wins over the table card under it.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform};
use crabomination::card::CardId;

/// Hovering this UI node previews the card it names.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct UiCardHover {
    /// The art to show (`scryfall::card_asset_path`).
    pub path: String,
    /// The card's name, for its rules text.
    pub name: String,
    /// The game object, when the node names one: a permanent's live notes
    /// (counters, a lost ability, commander damage) come from it.
    pub id: Option<CardId>,
}

impl UiCardHover {
    pub fn card(name: &str, id: Option<CardId>) -> Self {
        UiCardHover { path: crate::scryfall::card_asset_path(name), name: name.to_string(), id }
    }
}

/// Marks the panel a UI card's preview sits beside instead of the card's
/// own node: a decision modal's or a zone browser's, where beside the
/// hovered card is on top of the next one in the row.
#[derive(Component)]
pub struct PreviewAnchor;

/// The UI card nodes the pointer can be on, and what their previews anchor
/// to.
#[derive(SystemParam)]
pub struct UiCardSources<'w, 's> {
    cards: Query<'w, 's, (Entity, &'static Interaction, &'static UiCardHover, &'static UiGlobalTransform, &'static ComputedNode)>,
    parents: Query<'w, 's, &'static ChildOf>,
    anchors: Query<'w, 's, (&'static UiGlobalTransform, &'static ComputedNode), With<PreviewAnchor>>,
}

impl UiCardSources<'_, '_> {
    /// The UI card the pointer is on, if any, with the rect (UI px) its
    /// preview goes beside: its nearest [`PreviewAnchor`], else its own.
    pub fn hovered(&self) -> Option<(&UiCardHover, Rect)> {
        let (entity, _, card, at, size) = self.cards.iter().find(|(_, i, ..)| !matches!(i, Interaction::None))?;
        let (at, size) = self.parents.iter_ancestors(entity).find_map(|a| self.anchors.get(a).ok()).unwrap_or((at, size));
        let scale = size.inverse_scale_factor();
        Some((card, Rect::from_center_size(at.translation * scale, size.size() * scale)))
    }
}
