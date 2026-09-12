# Client backlog (`crabomination_client`)

Bevy GUI work: visualization, UX, and refactors. Split out of
`ENGINE_BACKLOG.md` at the sixty-seventh pass, where it was ~400 lines
interleaved with engine items.

**The client *does* build in the routine container** — the base image just
lacks four system libs:

```sh
apt-get update && apt-get install -y libwayland-dev libasound2-dev \
    libudev-dev libxkbcommon-dev
cargo clippy -p crabomination_client --all-targets
```

(First build ~6 min. `apt-get install` without the preceding `update` 404s on
a stale index. A full debug build of the client is ~2.5 GB of `target/` and
can exhaust the session's disk allowance — `cargo clean -p
crabomination_client` afterwards.) So client code and its unit tests compile
and test here; what still can't happen headlessly is *running* the GUI (no
GPU, no display — see the `verifier-client` skill). Without those libs the
crate silently rots: it had been broken since `CounterType::Fungus` landed,
because two exhaustive counter-label matches were never updated.

Shipped rows were dropped in the same pass unless they carried an open
residual; bodies are otherwise verbatim.

## Paper-cut sweep (2026-09-12) — shipped, with residuals

A read of the client turned up four defects that read as bugs rather than
missing polish. All four are fixed; what each one *taught* is recorded here
because the shape recurs.

- ✅ **Four panels declared `Overflow::scroll_y()` and could not scroll.**
  `bevy_ui` 0.19 ships no wheel-to-scroll system — it contains no
  `MouseWheel` reader at all — so `ScrollPosition` is application-driven.
  Only the draft packs and the audit picker wired a handler, and each
  carried a private copy of it, identical apart from the marker type and
  the line-pixel constant. The game log (200-entry scrollback, clipped at
  420 px), the graveyard and exile browsers (85 vh), and the
  library-search decision grid were all clipped and unreachable. The
  search grid was the gameplay blocker: at 110 px tiles under a
  `max_height: 70vh` / `max_width: 85%` panel, roughly **13 × 4 ≈ 52 cards
  are reachable at 1080p and ~30 at 1440×900**, so a 60-card Demonic Tutor
  hid ~8 cards and a Commander search ~47 — with no way to reach them.
  Now one `systems::scroll::{Scrollable, handle_scroll}`, registered once
  in every app state; the two private handlers are deleted.
  - ⚠ **`ScrollPosition` is a required component of `Node`**, so tagging a
    panel is the whole fix — there is nothing to insert alongside it. The
    four `ScrollPosition::default()` calls the old code carried were inert.
  - ⚠ **A scrollable that is a flex item in a column also needs
    `min_height: Val::Px(0.0)`.** Flexbox's `min-height: auto` resolves to
    content height and outranks `max_height`, so the node grows to fit and
    never overflows. The audit picker documented this trap; the search grid
    had walked into it.
  - ⏳ Residual: the log renders newest-first and `update_log_text`
    rebuilds every row on each event, so a new entry arriving while the
    player is scrolled back shifts the view by a row. Fixing that is the
    stable-children work already queued below.
- ✅ **Gameplay keybinds fired while you were typing.**
  `handle_export_prompt_input`'s docstring states the rule — "the caller's
  input handlers should bail out early in that same frame to avoid
  double-binding" — and it was applied by hand at six sites and missed at
  four, while three of the six disagreed about whether the export prompt
  counted. So `k` in a chat line submitted Keep on a mulligan prompt, `b`
  picked Black in ChooseColor, `i` toggled the life graph, `[` halved
  animation speed, and `?` opened the help overlay. Now one
  `systems::input_guard::{TextInputGuard, text_input_active}`.
  - ⚠ **A system holding `ResMut<T>` of any guarded resource cannot take
    `TextInputGuard`** — `ResMut<T>` + `Res<T>` in one system is Bevy
    **B0002, which compiles cleanly and panics at schedule init**, i.e. at
    launch, on a machine with a GPU that neither CI nor this container
    has. `handle_export_keypress` owns `ResMut<ExportPromptState>` and hit
    exactly this. `System::initialize` raises the conflict and needs no
    resources present, so
    `input_guard::tests::every_text_input_guard_holder_has_a_legal_access_set`
    reproduces the launch check headlessly — verified to fail when the
    conflict is reintroduced. **Extend that test when a system gains the
    guard.**
  - Pointer input stays live while typing: clicking Keep with the chat bar
    open still works, only the shortcut is suppressed.
- ✅ **Editing your player name, then touching any in-game setting,
  reverted it.** `config::update()` re-read the file, edited that copy and
  wrote it back, leaving `ConfigStore` — built once at startup, never
  re-synced — stale. Every other path (`persist_stops`,
  `persist_animation_speed`, the settings menu) rewrites the *whole*
  document from the store, so the next one to fire restored the old name /
  join address / deck path. `update()` is deleted; `update_store()` takes
  `&mut ConfigStore` and is the only write path.
- ✅ **A hand-edited config bricked launch.** Native `load()` panicked on
  unparseable TOML, and the only UI that could have fixed the file was the
  one that wouldn't start. Now falls back to defaults and **leaves the
  broken file on disk** so the edit is recoverable; the wasm loader
  already did this and now shares `parse_or_default` with it.
  - ⚠ The wasm branch of `config.rs` **cannot be compile-checked here**:
    `cargo check --target wasm32-unknown-unknown` fails in the engine at
    `game/layers.rs:352` (`size_of::<AffectedIds>() <=
    size_of::<Vec<CardId>>()` is false on a 32-bit pointer). Pre-existing
    — verified identical on a clean tree.

Not fixed, and worth their own pass — each is systemic rather than a paper
cut, and the first two would make the third and fourth reviewable:

- ✅ **No z-layer table** — shipped as `theme::layer`. Twelve named bands,
  spaced by ten, replacing `-1, 0, 30, 35, 40, 45, 46, 60, 100, 1000`; the
  remap is monotonic, so no surface changed places. The twenty-one existing
  indices moved onto it and the surfaces that had *none* — thirteen
  `DecisionModal` roots, `GameOverModalRoot`, five cast-flow modals, the
  ability menu, both browsers, the help overlay — now sit at
  `layer::MODAL`, with the Alt-peek popup and pile tooltip above them at
  `layer::CARD_PEEK` (which is the prerequisite "Alt-Peek Inside Decision
  Modals" below was missing). `quality.rs`'s `GlobalZIndex(1000)` hack is
  gone: it existed only because "the mulligan / decision modals carry no
  z-index, so they'd otherwise render on top of this since they spawn
  later".
  - ⚠ The order is enforced by **eleven `const _: () = assert!(…)` in the
    `layer` module**, not by a test — a reorder fails the *build*, the
    same idiom `crabomination/src/game/layers.rs:352` uses for its struct
    sizes. Verified: pushing `MODAL` above `CARD_PEEK` fails with
    `error[E0080] … assertion failed: MODAL < CARD_PEEK`.
- ✅ **No modal focus arbiter** — shipped as `systems::esc`. Twelve
  independent `Escape` readers now consult one precomputed owner:
  `EscSurface` declares twelve surfaces topmost-first, `compute_esc_focus`
  walks them in order and names the first that is open, and each handler
  asks `esc.owns(…)`. The ten-predicate list in `handle_settings_toggle`
  is deleted — the pause menu splits into `close_settings_on_esc`
  (first in precedence) and `open_settings_on_esc` (opens only on an
  *unclaimed* press), so it no longer has to know any other surface
  exists. `EscConsumed` is retired.
  - ⚠ **Precedence is a precomputed resource, not a chained system set.**
    The obvious design does not fit this schedule:
    `cancel_pickers_on_escape` is pinned `.after(handle_game_input)` (so a
    click and an Esc in the same frame resolve as the click) while
    `handle_keyboard_cursor_input` is pinned `.before` it — a
    picker-before-selection chain closes that into a cycle Bevy rejects.
    Running in `PreUpdate` instead makes every consumer
    order-independent and moved nothing in the `Update` graph.
  - ⚠ **`compute_esc_focus` needs `.after(bevy::input::InputSystems)`.**
    `keyboard_input_system` also runs in `PreUpdate`, so without it Bevy
    may schedule the arbiter first and read *last* frame's
    `just_pressed`.
  - Deliberate behaviour change: the attacker-plan Esc site used to fall
    through on purpose, so one press cleared the plan *and* closed an open
    ability menu. The menu is an `EscSurface::Picker` and outranks the
    plan, so that is now two presses.
  - ⏳ Residual: `settings_menu.rs:61` (the *menu*-state settings panel)
    still reads `Escape` directly. It is in a different `AppState` with no
    rival surfaces, so it has nothing to arbitrate against — left alone
    deliberately.
- ⏳ **48 hand-tuned chip colours.** `player_stats.rs:64-160` gives each
  game concept its own dark tint (Monarch, Initiative, Storm, Ring, Crime,
  Void, Descend, …) — 48 backgrounds a player cannot learn, and most of
  the 230 raw `Color::srgb` literals in the client. Collapse into ~6
  semantic families (resource / threat / timing / lock / flag / neutral).
- ⏳ **The help overlay has already drifted.** `HELP_SECTIONS`
  (`ui.rs:459`) is a hand-maintained const — "keep in sync when a binding
  changes" — and is missing `T` (chat) and the ChooseColor `W`/`U`/`B`/`R`/
  `G`. It also can't express that `G` means graveyard *and* green. Derive
  the overlay and the handlers from one binding table; that is also the
  foundation for keybind remapping.
- ⏳ **`UiScale` is a stub, so text scaling is impossible, not merely
  absent.** `main.rs:1246` returns `1.0` unconditionally because non-1.0
  values grew the corner HUD over the hand area. With 837 `Val::Px`
  literals and **135 text nodes at ≤13 px** (75 at 13, 38 at 12, 18 at 11,
  4 at ≤10) on a 4K-capable client, this outranks most of Tier 1 below.
  Subsumed by "Responsive HUD Layout".

## Client / UI follow-ups (M15 run)

- ✅ ~~**Convoke/Improvise cast UI**~~ — shipped. Right-clicking a convokable
  hand card opens the helper picker (`HelperTapState` +
  `spawn_helper_tap_modal`); confirming submits `CastSpellConvoke` (or
  `CastSpellWaterbend`) with the ticked helpers, arming the targeting cursor
  first for targeted spells. Backed by `HandAffordances.convokable` (a
  full-helper dry-run probe) plus `KnownCard.has_convoke` / `has_improvise`
  so the picker knows which permanent types can help. Residual: the picker
  lists candidates but doesn't preview how much each tap saves.
- **Master of Predicaments' hand pick.** The chosen card is auto-picked
  (the mana value furthest from the line) and the guess is asked of the
  resolving decider rather than routed to the guesser's seat.

## Client — Visualization

### Counter Display
`PermanentView.counters` carries all counter types and counts, but there is no
in-world or HUD display.  Suggested: floating text labels above affected cards
showing `+1/+1 ×3`, `Lore: 2`, `Charge: 1`, `Poison: 3`, etc., using Bevy
`Text3d` or billboard sprites.

### Modified Power/Toughness Display
When a creature's P/T differs from its printed values (pump spells, counters,
static effects), the printed Scryfall art still shows the base stats.
`PermanentView` exposes both `power`/`toughness` (current) and `base_power`/
`base_toughness` (printed). Current surfacing of modifications:
- 🟡 `draw_pt_modified_overlays` (`systems/gizmos.rs`) draws a coloured ring
  around any creature whose computed P/T differs from its base (green
  buffed / red debuffed / yellow mixed).
- 🟡 The Alt-key counter tooltip (`systems/counter_tooltip.rs`) shows
  `current/printed (printed X/Y)` when modified.
- ⏳ Still missing: an in-world numeric P/T overlay anchored to the card
  itself. Bevy's `Text2d` doesn't depth-sort with 3-D meshes, so this
  needs either (a) a billboarded `Text3d`/quad with a generated texture
  per card, or (b) a screen-space `Node` projected each frame off
  `Camera::world_to_viewport(card_translation)`. (b) is the cheaper
  retrofit; sits well next to the existing alt-tooltip projector.

### Modified Loyalty Display
There is no static loyalty badge today; loyalty surfaces only via the
3-D counter coin column on each planeswalker
(`systems/counter_coins.rs`, `CounterType::Loyalty` material). The coin
count tracks the current loyalty correctly, but the printed starting
loyalty from the card art and the precise current number are both
absent at a glance. Same screen-space-overlay approach as the P/T
overlay above would carry a "L: N" badge.

### Exile Zone Browser
✅ Shipped — `V` toggles a browser listing exiled cards with per-card
source annotations (linked exile, cipher, foretell, …).

### Stun Counter Visualization
Static Prison and Rapier Wit add stun counters.  No indicator currently shows
that a permanent has a stun counter (i.e., won't untap next turn).  A small
badge or coloured ring on the card would communicate this clearly.

### Damage Overlays
When combat damage is assigned, show floating damage numbers rising off
affected creatures before SBA removes the dead ones.

### Card Tooltip with Full Oracle Text
Hovering over a card shows its Scryfall art via the peek popup, but not the
full rules text.  A tooltip panel (shown on hover or via a dedicated key)
displaying the oracle text would reduce the need to look cards up externally.

### Graveyard Order and Timestamps
The graveyard browser shows cards as a flat unordered list.  Preserving
insertion order (most recently added = top) matches player intuition and helps
with "top of graveyard" effects.

### Attacking / Blocking Arrow Polish
Gizmo arrows are drawn in `draw_blocking_gizmos.rs` and `draw_attacker_overlays.rs`.
Improvements:
- Colour-code arrows by blocked/unblocked status.
- Show combat damage assignment numbers on arrows.
- Animate arrows fading in/out on declare-attackers/blockers transitions.

### Token Labeling
Token cards in the 3D view use the Scryfall-fetched art path, which often
resolves to a generic back image.  A text overlay (name + P/T) on token cards
would disambiguate multiple different tokens on the battlefield.

### Card Art on the Stack
The stack panel (`game_ui.rs::update_stack_panel`) shows only a "SPELL /
TRIGGER" badge + name + controller text. Add a small card thumbnail
(~70×100 px) per row using `scryfall::card_asset_path` — the scry/search
modals (`decision_ui.rs:293-334`) already follow the exact `ImageNode`
pattern. MTG players read the stack by visual recognition; text-only is
a big information-density loss in critical priority decisions.

### Life-Total Animation + Damage Feedback
Life changes are instantaneous text mutations in `update_player_text` /
`update_p1_text`. Lerp the displayed life toward the true value over
~0.5s and spawn a floating "−4" / "+2" near the player portrait that
drifts up and fades. Hook off `GameEventWire::DamageDealt`, `LifeLost`,
`LifeGained`. Pulse the life text red on lethal threat.

### Mana Symbol Rendering (Costs + Pool)
Mana is rendered as text codes (`W:1 R:2`) in the player status, ability
costs, alt-cast modal, and decision modals. Adopt a mana-symbol font or
PNG atlas plus a text segmenter that splits `{2}{R}{R}` into icons +
numerals. Once the glyph primitive exists every mana surface benefits
(the pip-style mana-pool HUD already ships in `player_stats.rs`).

### Phase Chart Progress Indicator
`update_phase_chart` highlights only the current step in yellow. Add a
filled vertical bar growing through the steps (or a left-edge arrow) so
turn progression is visible at a glance. Optional: tint the chart
differently when it's the opponent's turn vs yours.

### Card Hover Polish
`animate_hover_lift` currently only translates the card on Y. Modern MTG
clients combine the lift with a small scale-up (×1.03–1.05), a tilt-
toward-camera (~5°), and a shadow boost — much more tactile. The
`CardHovered` marker is already tracked; just extend the animation.

---

## Client — UX

### UI backlog — competitor-parity sweep (2026-06-11)

Prioritized ideas from a parity review against Arena / MTGO / Cockatrice /
XMage, after the decision-coverage + stops + import session shipped.
Cross-references the detailed entries below where one exists.

**In-game, high impact**
- ✅ **Stack as a visual zone** — `update_stack_panel` now renders
  card-art tiles: the top item gets a large gold-framed tile with a
  "resolves next" line, the rest smaller thumbnail rows, each with a
  controller-colored edge strip (green = yours / orange = opponent's);
  the footer offers a "Let resolve ▶" button while the viewer holds
  priority (else "Waiting for <name>…"). Hidden items show the cardback.
  Remaining ⏳: hover a tile → large preview, click → scroll the log.
- ⏳ **Undo / mana-tap rollback** — see "Engine — Rollback / Undo system
  (plan)"; the minimal client slice (un-tap floated mana before a cast
  commits) is worth shipping ahead of the full plan.
- 🟡 **Battlefield organization at scale** — ✅ identical tokens cascade
  into piles with a ×N count chip (`creature_card_transform` +
  `token_badge.rs`); same-name lands already stacked. Remaining ⏳: a
  visible aura/equipment → host link (today attachment info lives only in
  tooltips).
- 🟡 **Cost-payment feedback** — ✅ the manual-tap banner live-updates
  with the remaining cost ("{1}{U} to go") as sources tap. Remaining ⏳:
  pre-highlighting which sources auto-tap would take.

**Quick wins**
- 🟡 **Clickable game log** — ✅ hovering a log line that names a card
  previews it (`ui_card_hover`, also wired onto stack-panel tiles).
  Remaining ⏳: click → flash the permanent on the board.
- ⏳ **Finish the hover oracle panel** — `ui::hover_info_lines` shows type
  line + keyword reminders; add triggered/activated-ability short text
  (see "X-ray card inspector").

**Bigger projects**
- 🟡 **Settings screen** — ✅ main-menu Settings panel
  (`systems/settings_menu.rs`): window mode (windowed / borderless),
  resolution presets, maximize-on-launch, render quality, animation
  speed, hand sorting — applied live and persisted. Remaining ⏳:
  keybind remapping, audio (once there is audio).
- ⏳ **Deck library** — save imported decks, list them in the menu, pick
  the opponent's deck, paste-from-clipboard import.
- ⏳ **Bo3 + sideboarding UI** — Learn/Lessons sideboard plumbing exists
  engine-side.
- ⏳ **Replay viewer** — see "Replay scrubber" (Tier 3 below).
- ⏳ **Accessibility pass** — colorblind-safe target rings (shape, not
  only color), text scaling, reduced-motion toggle, finish keyboard-only
  play; see "Theme variants".

### Conspire cast UI (follow-up)

Conspirable hand cards now highlight as alt-castable (`ClientView.
conspirable_hand`, surfaced via the legal-play chain in `systems/ui.rs`).
Remaining: a creature-picker flow to actually submit `CastSpellConspire`
(choose exactly two untapped creatures sharing a color, like the
sacrifice/convoke pickers) — until then the client can only cast such cards
without the conspire copy. Engine + affordance + server view all ship.

### UI Roadmap (push claude/modern_decks — session-derived)

Ordering layer over the detailed items below. Cross-references existing
entries instead of duplicating; tiers ordered by start-here leverage.

**Player Crest track** — promote 3-D disc into stat readout + state
indicator + click target. Slims the 2-D chip strip.
- Phase 3 ⏳ NEXT — damage/heal floaters. New `life_floaters.rs`:
  `PreviousLifeTotals` resource + `LifeFloater` component +
  `detect_life_changes` + `animate_life_floaters`. Re-uses Phase 1
  projection helper. Data already in `ClientView`.
- Phase 4 ⏳ — slim corner chip strips to `name · ♥ · ✋`, move mana pips
  to a bottom detail bar.
- Phase 5 ⏳ — team-coloured tint from `GameState.teams`; commander emblem
  when `PlayerView.commanders` non-empty.

**Tier 1**
- X-ray card inspector ⏳ — extend Hover-Dwell Card Preview (below) to
  render engine-truth rules text from `CardDefinition` plus current
  modifications (layer P/T, granted keywords, attachments, counter net,
  legal actions). Differentiator vs XMage/MTGO/Arena.
- Stop settings + auto-pass ✅ — per-step Auto/Stop/Skip overrides on
  the clickable phase chart (`systems/phase_bar.rs::StopConfig`), wired
  into `auto_advance_p0`; right-click = pass-until-step. Remaining:
  persistence via `config.rs` (in progress).
- Stack widget polish ⏳ — promote `update_stack_panel` to a permanent
  floating panel; hover for source-card preview; click to scroll log.

**Tier 2**
- Unify decision modals ⏳ — `decision_ui.rs` has 6 parallel pickers
  (scry/search/put-on-library/discard/mulligan/color). Refactor into one
  `Picker { items, min, max, ordered, confirm_label }`. See Decision
  Modal vs 3-D Hand Consistency.
- Token stacking ⏳ — group identical tokens with count badge.
- Valid-target affordance ⏳ — make `ValidTarget` pulse, dim non-targets.
- Card-name → log preview ⏳ — hover region pops Scryfall image. See
  Hover-Dwell Card Preview.
- Theme variants ⏳ — light / high-contrast / colorblind palette in
  `theme.rs`.

**Tier 3**
- Replay scrubber ⏳ — `GameSnapshot` recorder + Menu→Replay scrub UI.
- Touch / controller input ⏳ — Bevy supports touch; `kb_cursor.rs` and
  input paths are mouse-centric.
- Split `game_ui.rs` further ⏳ — the initial split into
  `systems/game_ui/{mod,crest,player_stats,buttons,popups}.rs` shipped;
  still to pull out: `sync_game_visuals` → `visual_sync.rs` (~1.1K lines),
  `handle_game_input` → `input.rs` (~800 lines).

**Session follow-ups**
- Step-change → clear attack plan ⏳ — tiny watcher on `View.is_changed()`
  calling `attacking.clear()` when leaving `DeclareAttackers`.
- Crest pip cluster ⏳ — disc-rim pips for poison / commander damage /
  first-spell tax / energy. Reuse `counter_coins.rs` palette.

### Undo / Take-Back
A "request take-back" action the opponent can approve would reduce frustration
from misclicks, especially during the targeting flow. **Full plan now lives at
"Engine — Rollback / Undo system (plan)"** (snapshot-based, four phases;
Phase 4 is this UI).

### Responsive Stack Display
The stack panel (bottom-center) is a fixed-width overlay.  On narrow windows
it can overlap the player panel.  Clamp its width to `min(420px, 40vw)` or
reposition it to the right sidebar.

### Per-Phase Auto-Stop Flags
✅ Shipped as click-to-cycle Auto/Stop/Skip on the phase chart, scoped to
your turns vs opponents' (`systems/phase_bar.rs`); right-click a step =
pass-until. Remaining: persist the configuration (see backlog above).

### Deck Browser
A pre-game or in-game panel listing the full deck composition (name + count
for each unique card) would help players understand the randomly-assembled cube
deck they are playing.

### Game Log Scrollback + Event Color-Coding
✅ Shipped: 200-entry scrollback, per-variant colors, color-blind glyphs,
turn dividers, ×N coalescing, player names. Remaining ⏳: clickable log
lines (hover-preview the named card — see backlog above) and event
filtering.

### Button Hover + Pressed Feedback
Action buttons (Pass / End Turn / Next Turn / Export plus modal buttons) have
no `Interaction::Hovered` / `Pressed` tinting and no tooltips. Introduce a
generic `interactive_button` helper that wires hover/press background changes
and tooltip strings, and apply it across `game_ui.rs` HUD buttons,
`decision_ui.rs` modal buttons, and `draft.rs` tab buttons. The current pass
button hard-codes 4 srgb branches per priority state with no hover feedback.

### Selective Attacker Picking
✅ Click-based per-attacker picking is wired (`game_ui/mod.rs`, the
"Attacker selection" block): click an own creature to toggle it into the
plan, click an opponent planeswalker / player disc / 2-D HUD chip to
reassign the last-added attacker's defender, Esc / right-click to clear,
and `A` / the Attack button submits the picked plan (falling back to
"attack all eligible at next opp" when the plan is empty). Selected
attackers render gizmo diamonds (`gizmos.rs`).

⏳ Bigger lift still open: **drag an arrow** from attacker to defender /
planeswalker as an alternative to click-to-assign.

### Hover-Dwell Card Preview
Today the only way to read full rules text is to hold Alt while hovering
(`ui.rs::peek_popup`). Add a hover-dwell state machine (~300ms over a card
→ fade in large preview near cursor, with viewport-edge clamping). Reuse
`scryfall::card_asset_path`. Extends "Card Tooltip with Full Oracle Text"
above but specifically calls out the dwell-timer + cursor-relative
placement that brings the UX in line with Arena / MTGO.

### Decision Modal vs 3-D Hand Consistency
Mulligan and PutOnLibrary modals are transparent overlays over the 3-D
hand (player clicks the 3-D cards). Scry / Search / Discard render their
own 2-D card grid. No design rule says which decisions go which way, so
users can't predict whether to click the 2-D modal cards or the 3-D table
cards. Pick one rule (e.g., "decisions on the viewer's own hand → 3-D +
banner; decisions on hidden zones → 2-D modal grid") and migrate.

### Right-Click Action Hint
`game_ui.rs::handle_game_input` dispatches right-click on a hand card to
either the alt-cast modal (`has_alternative_cost`), the MDFC flip
(`back_face_name`), or the ability menu (battlefield card). The user has
no visual hint about which their right-click will trigger. Add a small
corner glyph on the card or a cursor-change to signal "right-click for
alt cost" / "right-click to flip".

### Hand-Fan Spacing for Large Hands
`card/layout.rs:18` sets `HAND_CARD_SPACING = CARD_WIDTH * 0.85`. A
15-card hand (Frantic Search loops, no-mulligan shenanigans) spreads
off-screen. Clamp total fan width to a viewport-relative target and
reduce spacing proportionally when hand size > 7.

### Drag-and-Drop for Hand → Battlefield
Hand cards play via click. Drag-to-position or drag-to-target would add
tactile feel for both casting and selecting targets. Lower priority than
the in-place fixes; capture the intent here.

### Settings Menu
The animation-speed slider is currently wedged into the quality panel
(`quality.rs::setup_quality_panel`). A proper Settings panel (audio,
key rebinds, UI scale, accessibility) would cleanly separate these and
give a natural home for future global preferences.

### Auto-Pass Toggle
`auto_advance_p0` (`game_ui.rs:2000+`) decides for the player when to pass
priority. A toolbar toggle ("Auto-pass: On/Off") lets new players step
through their own turn priority-by-priority instead of having the engine
fast-forward.

### Alt-Peek Inside Decision Modals
Scry / search / discard modal cards are 180×250 (`decision_ui.rs:124`) —
fine for art, illegible for rules text. The Alt-hold peek-popup
(`ui.rs:90-92`, 340×475) works on 3-D cards but doesn't fire on 2-D
modal cards. Wire Alt-hover inside `decision_ui` modals to spawn the
same large preview.

---

## Client — Engineering / Refactor

These don't change the player-visible UI but unblock parallel work and
reduce ongoing churn. Sequence them when scope or merge conflicts on the
Client UI layer become a recurring problem.

### Split `game_ui.rs`
2,850 lines mixing setup, view→entity sync (~1,000 lines), input,
ability menu, alt-cast modal, and HUD updates. Inline comment at line 38
admits `handle_game_input` is bumping Bevy's 16-param `SystemParam`
limit. Split into `game_ui/hud.rs` (setup + `update_*` text/buttons),
`game_ui/sync.rs` (`sync_game_visuals` only), `game_ui/input.rs`
(`handle_game_input` + `auto_advance`), `game_ui/modals.rs` (ability
menu, alt-cast). Keep `GameLogicSet` + `ButtonState` in `mod.rs`.
Prerequisite for several upcoming features but invisible to users.

### Modal Builder Helper
`decision_ui.rs` has 6+ near-identical "overlay root + panel + close-on-
escape" spawn functions (`spawn_scry_modal`, `spawn_search_modal`,
`spawn_discard_modal`, `spawn_put_on_library_modal`,
`spawn_mulligan_modal`, `spawn_choose_color_modal`). Each new decision
requires ~30 lines of root/panel boilerplate. Introduce a builder:
`modal(commands, ui_fonts, title).body(|panel| {…}).buttons(|btns| {…}).spawn()`.
Could halve `decision_ui.rs`.

### Stable-Children for Stack Panel + Pile Tooltip
`update_stack_panel` (`game_ui.rs::update_stack_panel`) and the pile
tooltip (`ui.rs::pile_tooltip`) `despawn_children()` + rebuild on every
change. The pile tooltip has a TODO comment explicitly admitting "we
can't easily update the child text here, so just leave it" — i.e., the
tooltip shows stale data. Give children stable marker components
(`StackPanelRow(idx)`, `PileTooltipText`) and update text in place.
Also fixes visible tearing when unrelated `view` fields change.

### `DecisionView` Trait
`spawn_decision_ui` matches every `DecisionWire` variant and dispatches
to a separate `spawn_*_modal`; `handle_confirm`,
`handle_put_on_library_select`, etc. repeat the same per-variant
dispatch. A `trait DecisionView { fn spawn(...); fn confirm(...);
fn cancel(...); }` implemented per variant would centralize. Roll up
under the Modal Builder above when you tackle it.

### Move `format_event` to Engine Crate
`format_event` (`game_ui.rs:91-167`) is a 75-line match on
`GameEventWire`. Every new event type requires editing this client-side
function. Move to a `Display` / `fmt_for_log` impl on the wire type
itself in `crabomination/src/net.rs` so new event variants stay
self-contained. Pairs with the log-color-coding work above.

### Relocate `stack_card_transform`
`stack_card_transform` lives in `game_ui.rs:2752` but is a pure math /
layout helper. Move to `card/layout.rs` next to the other transform
helpers (`hand_card_transform`, `bf_card_transform`, `deck_position`).

### Responsive HUD Layout
Most HUD panels use hardcoded `Val::Px` margins and widths
(`game_ui.rs:295-575`: `max_width: 560`, `min_width: 420`,
`BROWSER_CARD_WIDTH: 220` × 4 cols = ~960 px island). At 720p the
bottom player panel collides with the stack panel + AttackAllPanel;
at 1440p+ everything sits in a small island. Audit `Val::Px` →
`Val::Percent` / `Val::Vw` / `Val::Vh` per panel and add a `UiScale`
resource. Subsumes the existing "Responsive Stack Display" entry above.

---
