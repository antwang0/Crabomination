//! Every keyboard shortcut the client has, in one table.
//!
//! The handlers ask [`Keybinds::just_pressed`] for an [`Action`] instead of
//! reading a `KeyCode` themselves, and the F1 overlay (`systems::ui`) is
//! drawn from [`BINDINGS`] and [`FIXED`]. Before this, the overlay was a
//! hand-kept list "sourced from the input handlers"; it had drifted to name
//! `P` as pass (it rolls the planar die) and to miss chat, equip, the seat
//! camera digits and the colour keys.
//!
//! A remappable binding lives in [`BINDINGS`]; [`Keybinds`] holds the key each
//! one is on now, so a remap is one write to the resource. Keys that are
//! not one action — the WASD/arrow cursor, Esc, the held modifiers, the
//! digit row, the colour letters a "choose a colour" prompt reads — are
//! [`FIXED`] rows: listed in the overlay, not remappable.

use bevy::prelude::*;
use std::collections::HashMap;

/// A remappable keyboard action.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Action {
    ToggleHelp,
    ToggleLog,
    DebugConsole,
    ExportState,
    Graveyard,
    Exile,
    LifeGraph,
    Chat,
    AnimSlower,
    AnimFaster,
    AnimReset,
    Pass,
    EndTurn,
    NextTurn,
    Attack,
    HoldPriority,
    TakeBack,
    Flip,
    AltCost,
    AbilityMenu,
    Cycle,
    Equip,
    Keep,
    Mulligan,
    SerumPowder,
    PlanarDie,
    RevealAgenda,
}

/// The overlay's groups, in the order it lists them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Section {
    General,
    YourTurn,
    Cards,
    Choices,
    OpeningHand,
    Variants,
}

impl Section {
    pub const ALL: [Section; 6] = [
        Section::General,
        Section::YourTurn,
        Section::Cards,
        Section::Choices,
        Section::OpeningHand,
        Section::Variants,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Section::General => "General",
            Section::YourTurn => "Your turn",
            Section::Cards => "Hand & cards",
            Section::Choices => "Choices",
            Section::OpeningHand => "Mulligan",
            Section::Variants => "Planechase & conspiracies",
        }
    }
}

/// One remappable binding: what it does and where it starts.
pub struct Binding {
    pub action: Action,
    pub section: Section,
    pub default: KeyCode,
    pub label: &'static str,
}

const fn b(action: Action, section: Section, default: KeyCode, label: &'static str) -> Binding {
    Binding { action, section, default, label }
}

/// Every remappable binding, once each (`tests::every_action_is_bound_once`).
pub const BINDINGS: &[Binding] = {
    use Action::*;
    use Section::*;
    &[
        b(ToggleHelp, General, KeyCode::F1, "Toggle this help (also ?)"),
        b(Chat, General, KeyCode::KeyT, "Chat"),
        b(Graveyard, General, KeyCode::KeyG, "Browse your graveyard (click a pile for others')"),
        b(Exile, General, KeyCode::KeyV, "Browse the exile zone"),
        b(LifeGraph, General, KeyCode::KeyI, "Per-turn life history graph"),
        b(ToggleLog, General, KeyCode::KeyO, "Collapse / expand the game log"),
        b(ExportState, General, KeyCode::KeyX, "Export game state"),
        b(DebugConsole, General, KeyCode::Backquote, "Debug console"),
        b(AnimSlower, General, KeyCode::BracketLeft, "Animations slower"),
        b(AnimFaster, General, KeyCode::BracketRight, "Animations faster"),
        b(AnimReset, General, KeyCode::Backslash, "Animation speed reset"),
        b(Pass, YourTurn, KeyCode::Space, "Pass priority"),
        b(EndTurn, YourTurn, KeyCode::KeyE, "End turn"),
        b(NextTurn, YourTurn, KeyCode::KeyN, "Next turn (skip to your main)"),
        b(Attack, YourTurn, KeyCode::KeyA, "Attack all · confirm attackers"),
        b(HoldPriority, YourTurn, KeyCode::KeyH, "Hold priority (auto-pass on/off)"),
        b(TakeBack, YourTurn, KeyCode::KeyZ, "Take back your last action (Shift: pick several)"),
        b(Flip, Cards, KeyCode::KeyF, "Flip double-faced card"),
        b(AltCost, Cards, KeyCode::KeyL, "Cast for alternative cost"),
        b(AbilityMenu, Cards, KeyCode::KeyM, "Open ability menu"),
        b(Cycle, Cards, KeyCode::KeyC, "Cycle the selected card"),
        b(Equip, Cards, KeyCode::KeyE, "Equip the hovered Equipment"),
        b(Keep, OpeningHand, KeyCode::KeyK, "Keep hand"),
        b(Mulligan, OpeningHand, KeyCode::KeyM, "Mulligan"),
        b(SerumPowder, OpeningHand, KeyCode::KeyP, "Serum Powder"),
        b(PlanarDie, Variants, KeyCode::KeyP, "Roll the planar die"),
        b(RevealAgenda, Variants, KeyCode::KeyR, "Reveal your hidden agenda"),
    ]
};

/// Overlay rows that are not one remappable action.
pub const FIXED: &[(Section, &str, &str)] = &[
    (Section::General, "Esc", "Cancel / close / back"),
    (Section::General, "Hover", "Preview card beside cursor"),
    (Section::General, "Ctrl (hold)", "Zoom camera to hovered card"),
    (Section::General, "Alt (hold)", "Centered card · counters & P/T"),
    (Section::General, "1 – 6", "Camera on a seat (again: back home)"),
    (Section::General, "Click phase chart", "Cycle stop / skip for that step"),
    (Section::Cards, "Click · Enter", "Play / cast / select target"),
    (Section::Cards, "Tab · arrows · WASD", "Move selection (Shift+Tab back)"),
    (Section::Cards, "Right-click", "Cancel · quick-play the card's hint"),
    (Section::Choices, "W U B R G", "Choose that colour"),
];

/// The key each remappable action is on now.
#[derive(Resource, Clone, Debug)]
pub struct Keybinds {
    keys: HashMap<Action, KeyCode>,
}

impl Default for Keybinds {
    fn default() -> Self {
        Self { keys: BINDINGS.iter().map(|b| (b.action, b.default)).collect() }
    }
}

impl Keybinds {
    pub fn key(&self, action: Action) -> KeyCode {
        self.keys.get(&action).copied().unwrap_or_else(|| binding(action).default)
    }

    /// Whether `action`'s key went down this frame.
    pub fn just_pressed(&self, keyboard: &ButtonInput<KeyCode>, action: Action) -> bool {
        keyboard.just_pressed(self.key(action))
    }

    /// The key's name as the overlay and a button label print it.
    pub fn label(&self, action: Action) -> String {
        key_label(self.key(action))
    }

    #[cfg_attr(not(test), allow(dead_code))] // the remap screen is next
    pub fn set(&mut self, action: Action, key: KeyCode) {
        self.keys.insert(action, key);
    }
}

pub fn binding(action: Action) -> &'static Binding {
    BINDINGS.iter().find(|b| b.action == action).expect("every Action is in BINDINGS")
}

/// A key's printed name: `KeyG` → "G", `Digit1` → "1", `BracketLeft` → "[".
pub fn key_label(key: KeyCode) -> String {
    let named = match key {
        KeyCode::Space => "Space",
        KeyCode::Enter => "Enter",
        KeyCode::Tab => "Tab",
        KeyCode::Backspace => "Backspace",
        KeyCode::Backquote => "`",
        KeyCode::BracketLeft => "[",
        KeyCode::BracketRight => "]",
        KeyCode::Backslash => "\\",
        KeyCode::Slash => "/",
        KeyCode::Semicolon => ";",
        KeyCode::Quote => "'",
        KeyCode::Comma => ",",
        KeyCode::Period => ".",
        KeyCode::Minus => "-",
        KeyCode::Equal => "=",
        KeyCode::ArrowUp => "↑",
        KeyCode::ArrowDown => "↓",
        KeyCode::ArrowLeft => "←",
        KeyCode::ArrowRight => "→",
        _ => "",
    };
    if !named.is_empty() {
        return named.to_string();
    }
    let debug = format!("{key:?}");
    debug
        .strip_prefix("Key")
        .or_else(|| debug.strip_prefix("Digit"))
        .unwrap_or(&debug)
        .to_string()
}

/// The overlay's rows for `section`: the remappable bindings at their current
/// keys, then the fixed rows.
pub fn help_rows(binds: &Keybinds, section: Section) -> Vec<(String, String)> {
    BINDINGS
        .iter()
        .filter(|b| b.section == section)
        .map(|b| (binds.label(b.action), b.label.to_string()))
        .chain(
            FIXED
                .iter()
                .filter(|(s, ..)| *s == section)
                .map(|(_, k, l)| (k.to_string(), l.to_string())),
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pairs that share a default key on purpose: they never both fire, because
    /// one only reads its key in a state where the other can't act.
    /// `default_keys_only_share_where_allowed` holds the rest apart.
    const SHARED_DEFAULTS: &[(Action, Action, &str)] = &[
        (Action::Equip, Action::EndTurn, "E ends the turn unless it is equipping a hovered Equipment"),
        (Action::Mulligan, Action::AbilityMenu, "the mulligan prompt owns the keyboard"),
        (Action::SerumPowder, Action::PlanarDie, "the mulligan prompt owns the keyboard"),
    ];

    /// Exhaustive over `Action`, so a new variant fails to compile until it is
    /// listed here — and then fails the test until it is in `BINDINGS`.
    fn all_actions() -> Vec<Action> {
        use Action::*;
        let all = [
            ToggleHelp, ToggleLog, DebugConsole, ExportState, Graveyard, Exile, LifeGraph, Chat, AnimSlower,
            AnimFaster, AnimReset, Pass, EndTurn, NextTurn, Attack, HoldPriority, TakeBack, Flip,
            AltCost, AbilityMenu, Cycle, Equip, Keep, Mulligan, SerumPowder, PlanarDie,
            RevealAgenda,
        ];
        for a in all {
            match a {
                ToggleHelp | ToggleLog | DebugConsole | ExportState | Graveyard | Exile | LifeGraph | Chat
                | AnimSlower | AnimFaster | AnimReset | Pass | EndTurn | NextTurn | Attack
                | HoldPriority | TakeBack | Flip | AltCost | AbilityMenu | Cycle | Equip | Keep
                | Mulligan | SerumPowder | PlanarDie | RevealAgenda => {}
            }
        }
        all.to_vec()
    }

    #[test]
    fn every_action_is_bound_once() {
        for a in all_actions() {
            let n = BINDINGS.iter().filter(|b| b.action == a).count();
            assert_eq!(n, 1, "{a:?} is in BINDINGS {n} times");
        }
    }

    /// Two actions on one default key both fire on one press unless a state
    /// keeps them apart — which is how `G` revealed a hidden agenda *and*
    /// opened the graveyard, and `E` on a hovered Equipment ended the turn.
    #[test]
    fn default_keys_only_share_where_allowed() {
        for (i, x) in BINDINGS.iter().enumerate() {
            for y in &BINDINGS[i + 1..] {
                if x.default != y.default {
                    continue;
                }
                let allowed = SHARED_DEFAULTS.iter().any(|(a, b, _)| {
                    (*a, *b) == (x.action, y.action) || (*b, *a) == (x.action, y.action)
                });
                assert!(allowed, "{:?} and {:?} both default to {:?}", x.action, y.action, x.default);
            }
        }
    }

    #[test]
    fn every_section_has_rows_and_the_overlay_follows_a_remap() {
        let mut binds = Keybinds::default();
        for s in Section::ALL {
            assert!(!help_rows(&binds, s).is_empty(), "{s:?} is empty");
        }
        assert!(help_rows(&binds, Section::General).contains(&("T".into(), "Chat".into())));
        binds.set(Action::Chat, KeyCode::Enter);
        assert!(help_rows(&binds, Section::General).contains(&("Enter".into(), "Chat".into())));
    }

    #[test]
    fn key_labels_read_as_printed() {
        assert_eq!(key_label(KeyCode::KeyG), "G");
        assert_eq!(key_label(KeyCode::Digit3), "3");
        assert_eq!(key_label(KeyCode::F1), "F1");
        assert_eq!(key_label(KeyCode::Backquote), "`");
    }
}
